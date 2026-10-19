//! Bounded admission *before* rmcp creates handler tasks. Control notifications
//! are consumed here, not queued behind domain work or spawned without a bound.
use crate::catalog::{INPUT_CAP, OUTPUT_CAP};
use noteka_core::scoped::LocalScope;
use rmcp::{
    ErrorData, RoleServer,
    model::*,
    transport::{Transport, async_rw::JsonRpcMessageCodec},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::{Semaphore, mpsc, watch},
    time::Instant,
};
use tokio_util::{bytes::BytesMut, codec::Decoder};

pub struct RequestState {
    pub cancelled: AtomicBool,
    pub completed: Mutex<Option<(String, bool)>>,
}
pub struct Shared {
    pub scope: LocalScope,
    pub core: Arc<Semaphore>,
    requests: Mutex<HashMap<RequestId, Arc<RequestState>>>,
    stop: watch::Sender<Option<Instant>>,
}
impl Shared {
    pub fn new(scope: LocalScope) -> Arc<Self> {
        Arc::new(Self {
            scope,
            core: Arc::new(Semaphore::new(1)),
            requests: Mutex::default(),
            stop: watch::channel(None).0,
        })
    }
    pub fn shutdown(&self, at: Instant) {
        self.stop.send_if_modified(|v| {
            if v.is_none() {
                *v = Some(at);
                true
            } else {
                false
            }
        });
        self.scope.stop();
    }
    pub fn stopping(&self) -> bool {
        self.stop.borrow().is_some()
    }
    pub fn signal(&self) -> watch::Receiver<Option<Instant>> {
        self.stop.subscribe()
    }
    pub fn state(&self, id: &RequestId) -> Option<Arc<RequestState>> {
        self.requests
            .lock()
            .expect("request state")
            .get(id)
            .cloned()
    }
    fn retire(&self, id: &RequestId, expected: &Arc<RequestState>) {
        let mut m = self.requests.lock().expect("request state");
        if m.get(id).is_some_and(|s| Arc::ptr_eq(s, expected)) {
            m.remove(id);
        }
    }
}
pub fn busy() -> ErrorData {
    serde_json::from_value(
        json!({"code":-32000,"message":"Servidor ocupado.","data":{"reason":"server_busy"}}),
    )
    .expect("static error")
}
pub fn cancelled() -> ErrorData {
    ErrorData::invalid_request("Solicitud cancelada.", None)
}
pub fn delivery(request_id: &str, write: bool) -> ErrorData {
    ErrorData::internal_error(
        "Resultado no entregable; consultá el estado antes de repetir una mutación.",
        Some(
            json!({"reason":"response_too_large","request_id":request_id,"core_completed":true,"effects_may_be_committed":write}),
        ),
    )
}
pub async fn run_core<F>(
    shared: Arc<Shared>,
    state: Arc<RequestState>,
    effects: bool,
    work: F,
) -> Result<noteka_core::contracts::Response, ErrorData>
where
    F: FnOnce() -> noteka_core::contracts::Response + Send + 'static,
{
    if state.cancelled.load(Ordering::Acquire) || shared.stopping() {
        return Err(cancelled());
    }
    let permit = shared
        .core
        .clone()
        .try_acquire_owned()
        .map_err(|_| busy())?;
    let state_work = state.clone();
    let shared_work = shared.clone();
    let response = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        if state_work.cancelled.load(Ordering::Acquire) || shared_work.stopping() {
            return Err(cancelled());
        }
        let response = work();
        *state_work.completed.lock().expect("completion") =
            Some((response.request_id.clone(), effects));
        Ok(response)
    })
    .await
    .map_err(|_| {
        ErrorData::internal_error("Ejecución local interrumpida; consultá estado.", None)
    })??;
    if state.cancelled.load(Ordering::Acquire) || shared.stopping() {
        return Err(cancelled());
    }
    Ok(response)
}
pub struct LimitedWriter {
    bytes: Vec<u8>,
    limit: usize,
}
impl io::Write for LimitedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("message cap"));
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn encode<T: Serialize>(v: &T, cap: usize) -> Result<Vec<u8>, serde_json::Error> {
    let mut w = LimitedWriter {
        bytes: Vec::new(),
        limit: cap,
    };
    serde_json::to_writer(&mut w, v)?;
    Ok(w.bytes)
}
struct Outgoing {
    bytes: Vec<u8>,
    request: Option<(RequestId, Arc<RequestState>)>,
}
#[derive(Clone)]
struct Output {
    tx: mpsc::Sender<Outgoing>,
    shared: Arc<Shared>,
}
impl Output {
    fn enqueue(&self, mut message: ServerJsonRpcMessage, tracked: bool) -> io::Result<()> {
        let id = match &message {
            JsonRpcMessage::Response(r) => Some(r.id.clone()),
            JsonRpcMessage::Error(e) => e.id.clone(),
            _ => None,
        };
        let request = if tracked {
            id.as_ref()
                .and_then(|id| self.shared.state(id).map(|s| (id.clone(), s)))
        } else {
            None
        };
        // An inbound ID can itself be very large. Even the delivery error must
        // fit; inability to send that error is terminal, never partial JSON.
        let bytes = match encode(&message, OUTPUT_CAP) {
            Ok(bytes) => bytes,
            Err(_) => {
                let completed = request
                    .as_ref()
                    .and_then(|(_, s)| s.completed.lock().expect("completion").clone());
                let error = if let Some((request_id, effects)) = completed {
                    delivery(&request_id, effects)
                } else {
                    ErrorData::internal_error(
                        "Mensaje excede el límite.",
                        Some(json!({"reason":"response_too_large"})),
                    )
                };
                message = ServerJsonRpcMessage::error(error, id);
                encode(&message, OUTPUT_CAP).map_err(io::Error::other)?
            }
        };
        self.tx.try_send(Outgoing { bytes, request }).map_err(|_| {
            self.shared.shutdown(Instant::now());
            io::Error::other("output saturated")
        })
    }
}
pub struct BoundedTransport {
    rx: mpsc::Receiver<ClientJsonRpcMessage>,
    output: Output,
}
impl Transport<RoleServer> for BoundedTransport {
    type Error = io::Error;
    fn send(
        &mut self,
        message: ServerJsonRpcMessage,
    ) -> impl Future<Output = io::Result<()>> + Send + 'static {
        let output = self.output.clone();
        async move { output.enqueue(message, true) }
    }
    async fn receive(&mut self) -> Option<ClientJsonRpcMessage> {
        self.rx.recv().await
    }
    async fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn start<R: AsyncRead + Unpin + Send + 'static, W: AsyncWrite + Unpin + Send + 'static>(
    reader: R,
    writer: W,
    shared: Arc<Shared>,
) -> BoundedTransport {
    let (tx, rx) = mpsc::channel(4);
    let (out_tx, out_rx) = mpsc::channel(4);
    let output = Output {
        tx: out_tx,
        shared: shared.clone(),
    };
    tokio::spawn(read_loop(reader, tx, output.clone(), shared.clone()));
    tokio::spawn(write_loop(writer, out_rx, shared));
    BoundedTransport { rx, output }
}
async fn read_loop<R: AsyncRead + Unpin>(
    reader: R,
    tx: mpsc::Sender<ClientJsonRpcMessage>,
    output: Output,
    shared: Arc<Shared>,
) {
    let mut reader = BufReader::with_capacity(8192, reader);
    let mut stop = shared.signal();
    let mut line = BytesMut::new();
    let mut codec = JsonRpcMessageCodec::<Value>::new_with_max_length(INPUT_CAP);
    let mut initialized_forwarded = false;
    // No decode_eof: an unterminated line is not an admitted request.
    loop {
        if shared.stopping() {
            break;
        }
        let buf = tokio::select! { biased; _=stop.changed()=>break,buf=reader.fill_buf()=>match buf { Ok(b) if !b.is_empty()=>b,_=>break } };
        let take = buf
            .iter()
            .position(|b| *b == b'\n')
            .map_or(buf.len(), |p| p + 1);
        let complete = buf[take - 1] == b'\n';
        if line.len() + take > INPUT_CAP + usize::from(complete) {
            eprintln!("MCP: input_limit.");
            break;
        }
        line.extend_from_slice(&buf[..take]);
        reader.consume(take);
        if !complete {
            continue;
        }
        let value = match codec.decode(&mut line) {
            Ok(Some(v)) => v,
            _ => {
                if output
                    .enqueue(
                        ServerJsonRpcMessage::error(
                            ErrorData::parse_error("JSON inválido.", None),
                            None,
                        ),
                        false,
                    )
                    .is_err()
                {
                    break;
                }
                continue;
            }
        };
        let id = value
            .get("id")
            .and_then(|v| serde_json::from_value::<RequestId>(v.clone()).ok());
        // rmcp's Option<Map> treats explicit JSON null as omitted. The MCP
        // argument contract is an object, so reject null/scalars/arrays before
        // that normalization; only an actually absent arguments key is empty.
        if value["method"] == "tools/call"
            && value["params"]
                .get("arguments")
                .is_some_and(|args| !args.is_object())
        {
            if output
                .enqueue(
                    ServerJsonRpcMessage::error(
                        ErrorData::invalid_params("arguments debe ser un objeto.", None),
                        id,
                    ),
                    false,
                )
                .is_err()
            {
                break;
            }
            continue;
        }
        // The SDK accepts self-contained inline requests selecting an older
        // application version. This adapter deliberately supports 2025 only
        // through its initialize/initialized lifecycle, not that third mode.
        if id.is_some()
            && value["method"] != "initialize"
            && value["params"]["_meta"]["io.modelcontextprotocol/protocolVersion"] == "2025-11-25"
        {
            if output.enqueue(ServerJsonRpcMessage::error(ErrorData::invalid_request("Legacy requiere initialize/initialized y metadata de sesión, no negociación inline.",None),id),false).is_err() { break; }
            continue;
        }
        let message: ClientJsonRpcMessage = match serde_json::from_value(value) {
            Ok(v) => v,
            Err(_) => {
                if output
                    .enqueue(
                        ServerJsonRpcMessage::error(
                            ErrorData::invalid_request("Mensaje MCP inválido.", None),
                            id,
                        ),
                        false,
                    )
                    .is_err()
                {
                    break;
                }
                continue;
            }
        };
        match &message {
            JsonRpcMessage::Notification(n) => match &n.notification {
                ClientNotification::CancelledNotification(n) => {
                    if let Some(s) = n.params.request_id.as_ref().and_then(|id| shared.state(id)) {
                        s.cancelled.store(true, Ordering::Release);
                    }
                    continue;
                }
                ClientNotification::InitializedNotification(_) => {
                    if initialized_forwarded {
                        continue;
                    }
                    initialized_forwarded = true;
                }
                _ => continue,
            },
            JsonRpcMessage::Request(r) => {
                let admitted = {
                    let mut m = shared.requests.lock().expect("request state");
                    if m.contains_key(&r.id) {
                        Err(ErrorData::invalid_request("ID activo duplicado.", None))
                    } else if m.len() >= 4 {
                        Err(busy())
                    } else {
                        m.insert(
                            r.id.clone(),
                            Arc::new(RequestState {
                                cancelled: AtomicBool::new(false),
                                completed: Mutex::new(None),
                            }),
                        );
                        Ok(())
                    }
                };
                if let Err(error) = admitted {
                    if output
                        .enqueue(
                            ServerJsonRpcMessage::error(error, Some(r.id.clone())),
                            false,
                        )
                        .is_err()
                    {
                        break;
                    }
                    continue;
                }
            }
            _ => continue,
        }
        if let Err(error) = tx.try_send(message) {
            // An initialized notification may occupy a channel slot without
            // an active request ID. Refuse the excess request, rather than
            // terminating an otherwise valid pipelined legacy session.
            let message = error.into_inner();
            if let JsonRpcMessage::Request(r) = message {
                if let Some(state) = shared.state(&r.id) {
                    shared.retire(&r.id, &state);
                }
                if output
                    .enqueue(ServerJsonRpcMessage::error(busy(), Some(r.id)), false)
                    .is_ok()
                {
                    continue;
                }
            }
            break;
        }
    }
    shared.shutdown(Instant::now());
}
async fn write_loop<W: AsyncWrite + Unpin>(
    mut writer: W,
    mut rx: mpsc::Receiver<Outgoing>,
    shared: Arc<Shared>,
) {
    let mut stop = shared.signal();
    loop {
        if shared.stopping() {
            break;
        }
        let message = tokio::select! { biased; _=stop.changed()=>break,v=rx.recv()=>match v { Some(v)=>v,None=>break } };
        let (id, state) = match &message.request {
            Some((id, s)) => (Some(id), Some(s)),
            None => (None, None),
        };
        if !state.is_some_and(|s| s.cancelled.load(Ordering::Acquire)) {
            let started = Instant::now();
            let result = tokio::select! { biased;
                _=stop.changed()=>break,
                r=tokio::time::timeout(Duration::from_secs(5),async {
                    writer.write_all(&message.bytes).await?;
                    // Retire before publishing the delimiter. A client may
                    // legitimately reuse its ID as soon as that newline is
                    // visible, even before this task resumes from write_all.
                    if let (Some(id),Some(state))=(id,state) { shared.retire(id,state); }
                    writer.write_all(b"\n").await?;writer.flush().await
                })=>r,
            };
            if !matches!(result, Ok(Ok(()))) {
                shared.shutdown(started);
                eprintln!("MCP: output_closed.");
                break;
            }
        }
        if let (Some(id), Some(state)) = (id, state) {
            shared.retire(id, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noteka_core::{scoped::Policy, workspace::Workspace};
    fn setup() -> (tempfile::TempDir, Arc<Shared>) {
        let t = tempfile::tempdir().unwrap();
        let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
        let shared =
            Shared::new(LocalScope::new(&w.path, Policy::new(true, true).unwrap()).unwrap());
        (t, shared)
    }
    fn state() -> Arc<RequestState> {
        Arc::new(RequestState {
            cancelled: AtomicBool::new(false),
            completed: Mutex::new(None),
        })
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn cancellation_barriers_before_dispatch_after_commit_and_dropped_waiter() {
        let (_t, shared) = setup();
        let cancelled_state = state();
        cancelled_state.cancelled.store(true, Ordering::Release);
        let scope = shared.scope.clone();
        assert!(
            run_core(shared.clone(), cancelled_state, true, move || scope
                .execute_json(
                    json!({"operation":"entity.create","input":{"kind":"note","title":"never"}}),
                    "mcp"
                ))
            .await
            .is_err()
        );
        let held = state();
        let scope = shared.scope.clone();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let work = tokio::spawn(run_core(shared.clone(), held.clone(), true, move || {
            let response = scope.execute_json(
                json!({"operation":"entity.create","input":{"kind":"note","title":"committed"}}),
                "mcp",
            );
            assert!(response.ok);
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            response
        }));
        started_rx.await.unwrap();
        held.cancelled.store(true, Ordering::Release);
        let scope = shared.scope.clone();
        assert_eq!(
            run_core(shared.clone(), state(), false, move || scope.execute_json(
                json!({"operation":"entity.list","input":{}}),
                "mcp"
            ))
            .await
            .unwrap_err()
            .data
            .unwrap()["reason"],
            "server_busy"
        );
        work.abort();
        assert!(work.await.unwrap_err().is_cancelled());
        assert_eq!(shared.core.available_permits(), 0); // async abort did not free it
        release_tx.send(()).unwrap();
        let _permit =
            tokio::time::timeout(Duration::from_secs(3), shared.core.clone().acquire_owned())
                .await
                .unwrap()
                .unwrap();
        assert!(held.completed.lock().unwrap().as_ref().unwrap().1);
        let result = shared
            .scope
            .execute_json(json!({"operation":"entity.list","input":{}}), "mcp");
        assert_eq!(result.data.unwrap()["items"].as_array().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn pending_cancelled_output_is_suppressed_and_slots_are_retired() {
        let (_t, shared) = setup();
        let s = state();
        s.cancelled.store(true, Ordering::Release);
        let id = RequestId::Number(1);
        shared.requests.lock().unwrap().insert(id.clone(), s);
        let (tx, rx) = mpsc::channel(4);
        let output = Output {
            tx,
            shared: shared.clone(),
        };
        output
            .enqueue(ServerJsonRpcMessage::error(cancelled(), Some(id)), true)
            .unwrap();
        let (writer, mut reader) = tokio::io::duplex(32);
        drop(output);
        write_loop(writer, rx, shared.clone()).await;
        let mut bytes = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut reader, &mut bytes)
            .await
            .unwrap();
        assert!(bytes.is_empty());
        assert!(shared.requests.lock().unwrap().is_empty());
    }
    #[tokio::test(start_paused = true)]
    async fn blocked_writer_deadline_begins_at_first_write_and_output_queue_is_bounded() {
        let (_t, shared) = setup();
        let (tx, rx) = mpsc::channel(4);
        let output = Output {
            tx,
            shared: shared.clone(),
        };
        let (writer, _reader) = tokio::io::duplex(1);
        let task = tokio::spawn(write_loop(writer, rx, shared.clone()));
        let at = Instant::now();
        output
            .enqueue(ServerJsonRpcMessage::error(busy(), None), false)
            .unwrap();
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(5)).await;
        task.await.unwrap();
        assert_eq!(*shared.signal().borrow(), Some(at));
        let (_t, shared) = setup();
        let (tx, _rx) = mpsc::channel(4);
        let output = Output {
            tx,
            shared: shared.clone(),
        };
        for _ in 0..4 {
            output
                .enqueue(ServerJsonRpcMessage::error(busy(), None), false)
                .unwrap();
        }
        assert!(
            output
                .enqueue(ServerJsonRpcMessage::error(busy(), None), false)
                .is_err()
        );
        assert!(shared.stopping());
    }
    #[test]
    fn encoding_caps_and_post_execution_framing_overflow_do_not_lie_about_commit() {
        assert_eq!(encode(&"a".repeat(100), 102).unwrap().len(), 102);
        assert!(encode(&"a".repeat(101), 102).is_err());
        let (_t, shared) = setup();
        let s = state();
        *s.completed.lock().unwrap() = Some(("core-request-id".into(), false));
        let id = RequestId::Number(1);
        shared.requests.lock().unwrap().insert(id.clone(), s);
        let (tx, mut rx) = mpsc::channel(4);
        let output = Output { tx, shared };
        let result = CallToolResult::structured(json!({"huge":"\"".repeat(OUTPUT_CAP)}));
        output
            .enqueue(
                ServerJsonRpcMessage::response(
                    ServerResult::from(CallToolResponse::from(result)),
                    id,
                ),
                true,
            )
            .unwrap();
        let message: Value = serde_json::from_slice(&rx.try_recv().unwrap().bytes).unwrap();
        assert_eq!(message["error"]["code"], -32603);
        assert_eq!(message["error"]["data"]["request_id"], "core-request-id");
        assert_eq!(message["error"]["data"]["effects_may_be_committed"], false);
    }
    #[tokio::test]
    async fn four_requests_are_admitted_before_sdk_tasks_and_cancel_bypasses_full_input() {
        let (_t, shared) = setup();
        let (mut input, reader) = tokio::io::duplex(8192);
        let (writer, output) = tokio::io::duplex(8192);
        let transport = start(reader, writer, shared.clone());
        for id in 1..=5 {
            let mut frame = serde_json::to_vec(
                &json!({"jsonrpc":"2.0","id":id,"method":"tools/list","params":{}}),
            )
            .unwrap();
            frame.push(b'\n');
            input.write_all(&frame).await.unwrap();
        }
        let mut output = BufReader::new(output);
        let mut line = String::new();
        output.read_line(&mut line).await.unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], 5);
        assert_eq!(response["error"]["data"]["reason"], "server_busy");
        assert_eq!(transport.rx.len(), 4);
        assert_eq!(shared.requests.lock().unwrap().len(), 4);
        input.write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/cancelled\",\"params\":{\"requestId\":1}}\n{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"tools/list\",\"params\":{}}\n").await.unwrap();
        line.clear();
        output.read_line(&mut line).await.unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], 6);
        assert!(
            shared
                .state(&RequestId::Number(1))
                .unwrap()
                .cancelled
                .load(Ordering::Acquire)
        );
        assert_eq!(shared.core.available_permits(), 1);
        assert_eq!(transport.rx.len(), 4);
        drop(input);
        let mut stop = shared.signal();
        if !shared.stopping() {
            stop.changed().await.unwrap();
        }
        assert!(shared.stopping());
    }
    #[tokio::test]
    async fn legacy_notification_does_not_turn_input_saturation_into_terminal_shutdown() {
        let (_t, shared) = setup();
        let (mut input, reader) = tokio::io::duplex(8192);
        let (writer, output) = tokio::io::duplex(8192);
        let transport = start(reader, writer, shared.clone());
        input
            .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
            .await
            .unwrap();
        for id in 1..=4 {
            let mut frame = serde_json::to_vec(
                &json!({"jsonrpc":"2.0","id":id,"method":"tools/list","params":{}}),
            )
            .unwrap();
            frame.push(b'\n');
            input.write_all(&frame).await.unwrap();
        }
        let mut output = BufReader::new(output);
        let mut line = String::new();
        output.read_line(&mut line).await.unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], 4);
        assert_eq!(response["error"]["data"]["reason"], "server_busy");
        assert_eq!(transport.rx.len(), 4);
        assert_eq!(shared.requests.lock().unwrap().len(), 3);
        assert!(!shared.stopping());
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn eof_during_synchronous_read_keeps_slot_until_read_finishes_and_discards_delivery() {
        let (_t, shared) = setup();
        let s = state();
        let scope = shared.scope.clone();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let task = tokio::spawn(run_core(shared.clone(), s, false, move || {
            let response = scope.execute_json(json!({"operation":"entity.list","input":{}}), "mcp");
            assert!(response.ok);
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            response
        }));
        started_rx.await.unwrap();
        let (input, reader) = tokio::io::duplex(32);
        let (writer, _output) = tokio::io::duplex(32);
        let _transport = start(reader, writer, shared.clone());
        let mut stop = shared.signal();
        drop(input);
        stop.changed().await.unwrap();
        assert!(shared.stopping());
        assert_eq!(shared.core.available_permits(), 0);
        release_tx.send(()).unwrap();
        assert!(task.await.unwrap().is_err());
        assert_eq!(shared.core.available_permits(), 1);
    }
}
