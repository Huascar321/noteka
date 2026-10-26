use super::*;
use crate::{
    agent_storage, attachment_search as lexical, attachment_search_contracts::AttachmentIndexInput,
    attachment_search_tests as fixtures, workspace::Workspace,
};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

fn setup() -> (tempfile::TempDir, Workspace, ExtractionJob) {
    let t = tempfile::tempdir().unwrap();
    let mut w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    let job = fixtures::add(&mut w, b"alpha beta gamma", "utf8_text");
    let revision = crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision;
    lexical::index(
        &w.path,
        AttachmentIndexInput::Put {
            job_id: job.id.clone(),
            expected_generation_id: job.generation.as_ref().unwrap().id.clone(),
            expected_revision: revision,
        },
    )
    .unwrap();
    (t, w, job)
}
fn config(endpoint: String) -> AgentProviderConfig {
    AgentProviderConfig {
        runtime: "openai_chat_completions_v1".into(),
        endpoint,
        model: "mock-chat".into(),
        timeout_ms: 5_000,
        max_tokens: 256,
        temperature: 0.0,
    }
}
fn ask_input(endpoint: String, question: &str) -> AgentAskInput {
    AgentAskInput {
        question: question.into(),
        provider: config(endpoint),
        limits: AgentLimits {
            max_steps: 4,
            max_context_bytes: 64 * 1024,
            max_tool_bytes: 64 * 1024,
            deadline_ms: 10_000,
        },
        allow_apply: false,
    }
}
fn content_response(content: String) -> String {
    serde_json::json!({"choices":[{"message":{"content":content}}]}).to_string()
}
fn read_request(stream: &mut std::net::TcpStream) -> Vec<u8> {
    let mut data = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = stream.read(&mut buffer).unwrap();
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..n]);
        if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&data[..pos]);
            let length = head
                .lines()
                .find_map(|line| {
                    line.split_once(':').and_then(|(k, v)| {
                        k.eq_ignore_ascii_case("content-length")
                            .then(|| v.trim().parse::<usize>().ok())
                            .flatten()
                    })
                })
                .unwrap_or(0);
            if data.len() >= pos + 4 + length {
                break;
            }
        }
    }
    data
}
fn mock_chat(responses: Vec<(String, Duration)>) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let handle = thread::spawn(move || {
        for (body_text, delay) in responses {
            let started = Instant::now();
            let (mut stream, _) = loop {
                match listener.accept() {
                    Ok(connection) => break connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        if started.elapsed() > Duration::from_secs(2) {
                            return;
                        }
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            let _ = read_request(&mut stream);
            thread::sleep(delay);
            let body = body_text;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (format!("http://{address}/v1/chat/completions"), handle)
}
fn citation(job: &ExtractionJob, quote: &str) -> serde_json::Value {
    serde_json::json!({
        "job_id": job.id,
        "generation_id": job.generation.as_ref().unwrap().id,
        "ordinal": 0,
        "quote": quote
    })
}
fn search_tool() -> String {
    serde_json::json!({"type":"tool_call","tool":"retrieval.search","input":{"query":"alpha","mode":"lexical"}}).to_string()
}

#[test]
fn final_with_verified_citation_succeeds() {
    let (_t, w, job) = setup();
    let final_content = serde_json::json!({
        "type":"final",
        "answer":"El fragmento menciona alpha.",
        "citations":[citation(&job, "alpha beta")],
        "insufficient_evidence":false
    })
    .to_string();
    let (url, handle) = mock_chat(vec![
        (content_response(search_tool()), Duration::ZERO),
        (content_response(final_content), Duration::ZERO),
    ]);
    let result = ask(&w.path, ask_input(url, "¿alpha?"), None).unwrap();
    handle.join().unwrap();
    assert_eq!(result.run.state, AgentRunState::Succeeded);
    assert_eq!(result.citations.len(), 1);
    assert!(result.citations[0].verified);
    assert_eq!(result.citations[0].job_id, job.id);
    assert_eq!(result.steps.len(), 3);
    let trace = trace(
        &w.path,
        AgentTraceInput::Get {
            id: result.run.id.clone(),
        },
        None,
    )
    .unwrap();
    let AgentTraceResult::Get(trace) = trace else {
        unreachable!()
    };
    assert_eq!(trace.run.state, AgentRunState::Succeeded);
    assert_eq!(trace.steps.len(), 3);
}

#[test]
fn invented_citation_fails() {
    let (_t, w, job) = setup();
    let final_content = serde_json::json!({
        "type":"final",
        "answer":"Inventado.",
        "citations":[citation(&job, "texto que no está")],
        "insufficient_evidence":false
    })
    .to_string();
    let (url, handle) = mock_chat(vec![
        (content_response(search_tool()), Duration::ZERO),
        (content_response(final_content), Duration::ZERO),
    ]);
    let result = ask(&w.path, ask_input(url, "¿alpha?"), None).unwrap();
    handle.join().unwrap();
    assert_eq!(result.run.state, AgentRunState::Failed);
    assert_eq!(result.run.error.as_deref(), Some("citation_unverified"));
    assert!(result.run.answer.is_none());
}

#[test]
fn answer_without_citation_or_insufficiency_fails() {
    let (_t, w, _) = setup();
    let final_content = serde_json::json!({
        "type":"final","answer":"Sin fuente","citations":[],"insufficient_evidence":false
    })
    .to_string();
    let (url, handle) = mock_chat(vec![(content_response(final_content), Duration::ZERO)]);
    let result = ask(&w.path, ask_input(url, "¿alpha?"), None).unwrap();
    handle.join().unwrap();
    assert_eq!(result.run.state, AgentRunState::Failed);
    assert_eq!(result.run.error.as_deref(), Some("citation_required"));
}

#[test]
fn insufficient_evidence_is_explicit() {
    let (_t, w, _) = setup();
    let final_content = serde_json::json!({
        "type":"final","answer":"No hay evidencia suficiente.","citations":[],"insufficient_evidence":true
    })
    .to_string();
    let (url, handle) = mock_chat(vec![(content_response(final_content), Duration::ZERO)]);
    let result = ask(&w.path, ask_input(url, "¿omega?"), None).unwrap();
    handle.join().unwrap();
    assert_eq!(result.run.state, AgentRunState::Succeeded);
    assert!(result.citations.is_empty());
}

#[test]
fn forbidden_tool_and_invalid_protocol_fail_with_trace() {
    let (_t, w, _) = setup();
    let (url, handle) = mock_chat(vec![(
        content_response(
            serde_json::json!({"type":"tool_call","tool":"shell","input":{}}).to_string(),
        ),
        Duration::ZERO,
    )]);
    let result = ask(&w.path, ask_input(url, "¿alpha?"), None).unwrap();
    handle.join().unwrap();
    assert_eq!(result.run.state, AgentRunState::Failed);
    assert_eq!(result.run.error.as_deref(), Some("agent_tool_not_allowed"));
    let (url, handle) = mock_chat(vec![(
        content_response("no es json".into()),
        Duration::ZERO,
    )]);
    let result = ask(&w.path, ask_input(url, "¿alpha?"), None).unwrap();
    handle.join().unwrap();
    assert_eq!(result.run.state, AgentRunState::Failed);
    assert_eq!(result.run.error.as_deref(), Some("agent_protocol_invalid"));
}

#[test]
fn unreachable_provider_leaves_queryable_trace() {
    let (_t, w, _) = setup();
    let input = ask_input("http://127.0.0.1:1/v1/chat/completions".into(), "¿alpha?");
    let result = ask(&w.path, input, None).unwrap();
    assert_eq!(result.run.state, AgentRunState::Failed);
    assert!(result.run.error.as_deref().is_some());
    let trace = trace(&w.path, AgentTraceInput::Get { id: result.run.id }, None).unwrap();
    assert!(matches!(trace, AgentTraceResult::Get(_)));
}

#[test]
fn cancellation_is_observed_between_processes() {
    let (_t, w, _) = setup();
    let final_content = serde_json::json!({
        "type":"final","answer":"Tarde","citations":[],"insufficient_evidence":true
    })
    .to_string();
    let (url, handle) = mock_chat(vec![(
        content_response(final_content),
        Duration::from_millis(500),
    )]);
    let path = w.path.clone();
    let input = ask_input(url, "¿alpha?");
    let asker = thread::spawn(move || ask(&path, input, None).unwrap());
    let mut id = None;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let page = match trace(
            &w.path,
            AgentTraceInput::List {
                limit: Some(10),
                cursor: None,
            },
            None,
        )
        .unwrap()
        {
            AgentTraceResult::List(page) => page,
            _ => unreachable!(),
        };
        if let Some(run) = page
            .items
            .into_iter()
            .find(|r| r.state == AgentRunState::Running)
        {
            id = Some(run.id);
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let id = id.expect("run running");
    cancel(&w.path, AgentCancelInput { id }, None).unwrap();
    let result = asker.join().unwrap();
    handle.join().unwrap();
    assert_eq!(result.run.state, AgentRunState::Cancelled);
}

#[test]
fn second_run_is_rejected_while_lease_is_active() {
    let (_t, w, _) = setup();
    let input = ask_input("http://127.0.0.1:1/v1/chat/completions".into(), "¿alpha?");
    let (c, _) = agent_storage::open(&w.path, true).unwrap();
    let lease = (Utc::now() + ChronoDuration::seconds(30))
        .to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    agent_storage::insert(
        &c,
        &workspace::id(),
        &input.provider,
        &input.question,
        &lease,
    )
    .unwrap();
    let error = ask(&w.path, input, None).unwrap_err();
    assert!(matches!(error.code, ErrorCode::WorkspaceBusy));
    assert_eq!(error.details["reason"], "agent_busy");
}

#[test]
fn apply_requires_a_preview_from_the_same_execution() {
    let (_t, w, _) = setup();
    let plan = BatchPlan {
        version: 1,
        batch_id: workspace::id(),
        steps: vec![BatchStep {
            key: "one".into(),
            operation: BatchOperation::EntityCreate,
            input: serde_json::json!({"kind":"note","title":"Agent note"}),
            bindings: vec![],
        }],
        checks: vec![],
    };
    let preview = crate::batches::preview(&w.path, plan.clone(), "agent", "run").unwrap();
    let mut previews = HashMap::new();
    previews.insert(
        (
            preview.receipt.plan_hash.clone(),
            preview.receipt.base_hash.clone(),
        ),
        preview.receipt.clone(),
    );
    let mut hits = HashMap::new();
    let value = serde_json::json!({"plan":plan,"expected":preview.receipt});
    dispatch(
        &w.path,
        &Tool::ChangesApply,
        value.clone(),
        true,
        &mut hits,
        &mut previews,
        None,
        "run",
    )
    .unwrap();
    let error = dispatch(
        &w.path,
        &Tool::ChangesApply,
        value,
        true,
        &mut hits,
        &mut previews,
        None,
        "run",
    )
    .unwrap_err();
    assert_eq!(error.details["reason"], "agent_apply_without_preview");
    let error = dispatch(
        &w.path,
        &Tool::ChangesApply,
        serde_json::json!({"plan":plan,"expected":preview.receipt}),
        false,
        &mut hits,
        &mut previews,
        None,
        "run",
    )
    .unwrap_err();
    assert_eq!(error.details["reason"], "agent_apply_not_allowed");
}

#[test]
fn migration_018_and_backup_preserve_traces() {
    let (t, w, _) = setup();
    let input = ask_input("http://127.0.0.1:1/v1/chat/completions".into(), "¿alpha?");
    let (c, _) = agent_storage::open(&w.path, true).unwrap();
    let id = workspace::id();
    let lease = (Utc::now() + ChronoDuration::seconds(30))
        .to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    agent_storage::insert(&c, &id, &input.provider, &input.question, &lease).unwrap();
    drop(c);
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let mut w = w;
    w.conn
        .execute_batch(
            "DROP TABLE agent_runs;DROP TABLE IF EXISTS extraction_pdf_structures;
UPDATE workspace_meta SET schema_version=17;PRAGMA user_version=17",
        )
        .unwrap();
    w.migrate(false).unwrap();
    assert_eq!(workspace::version(&w.conn).unwrap(), 20);
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM agent_runs", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        0
    );
    let destination = t.path().join("restored");
    Workspace::restore(&backup, &destination).unwrap();
    let restored = Workspace::open(&destination).unwrap();
    assert_eq!(
        restored
            .conn
            .query_row("SELECT count(*) FROM agent_runs", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        1
    );
}
