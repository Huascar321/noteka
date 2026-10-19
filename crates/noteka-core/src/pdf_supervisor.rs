//! Proceso descartable con pipes no bloqueantes; nunca espera un thread de IO.
use crate::{batch_storage::Budget, error::*, extraction_storage as store};
use noteka_pdf_protocol::{Header, MAX_STDERR_BYTES, MAX_STDOUT_BYTES, Outcome, PageText, Reply};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

#[cfg(test)]
type InputHook = Option<Box<dyn FnOnce(u32)>>;
#[cfg(test)]
thread_local! {
    static INPUT_HOOK: std::cell::RefCell<InputHook> = std::cell::RefCell::new(None);
    static SPAWNS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
pub(crate) fn spawn_count() -> u32 {
    SPAWNS.with(|n| n.get())
}
#[cfg(test)]
pub(crate) fn input_hook(hook: impl FnOnce(u32) + 'static) {
    INPUT_HOOK.with(|cell| {
        assert!(cell.borrow().is_none());
        *cell.borrow_mut() = Some(Box::new(hook));
    });
}

fn runtime() -> Result<PathBuf> {
    let exe = std::env::current_exe().map_err(|_| store::storage("pdf_worker_unavailable"))?;
    let mut dir = exe
        .parent()
        .ok_or_else(|| store::storage("pdf_worker_unavailable"))?;
    if dir
        .file_name()
        .is_some_and(|n| n == "deps" || n == "examples")
    {
        dir = dir
            .parent()
            .ok_or_else(|| store::storage("pdf_worker_unavailable"))?;
    }
    Ok(dir.join(if cfg!(windows) {
        "noteka-pdf-worker.exe"
    } else {
        "noteka-pdf-worker"
    }))
}
pub(crate) fn run(
    bytes: &[u8],
    budget: &Budget<'_>,
    check: impl FnMut() -> Result<()>,
) -> Result<Vec<PageText>> {
    run_at(&runtime()?, bytes, budget, check)
}
struct Process(Child);
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[cfg(unix)]
fn nonblocking(fd: &impl std::os::fd::AsRawFd) -> Result<()> {
    let fd = fd.as_raw_fd();
    // El descriptor es de un pipe vivo propiedad de Child; fcntl no retiene
    // punteros y se conservan todos los flags previos. No afecta al host/SQLite.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(store::storage("pdf_worker_failed"));
    }
    Ok(())
}
#[cfg(unix)]
pub(crate) fn run_at(
    path: &Path,
    bytes: &[u8],
    budget: &Budget<'_>,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Vec<PageText>> {
    check()?;
    if !std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_file()) {
        return Err(store::storage("pdf_worker_unavailable"));
    }
    if bytes.is_empty() {
        return Err(store::invalid("invalid_pdf"));
    }
    let header = Header {
        source_bytes: bytes.len() as u32,
        remaining_ms: budget.remaining_ms()?,
    }
    .encode()
    .map_err(|_| store::storage("pdf_worker_protocol"))?;
    #[cfg(test)]
    SPAWNS.with(|n| n.set(n.get() + 1));
    let child = Command::new(path)
        .env_clear()
        .current_dir(
            path.parent()
                .ok_or_else(|| store::storage("pdf_worker_unavailable"))?,
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| store::storage("pdf_worker_unavailable"))?;
    let mut child = Process(child);
    let mut stdin = Some(
        child
            .0
            .stdin
            .take()
            .ok_or_else(|| store::storage("pdf_worker_failed"))?,
    );
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| store::storage("pdf_worker_failed"))?;
    let mut stderr = child
        .0
        .stderr
        .take()
        .ok_or_else(|| store::storage("pdf_worker_failed"))?;
    nonblocking(stdin.as_ref().unwrap())?;
    nonblocking(&stdout)?;
    nonblocking(&stderr)?;
    let mut sent = 0;
    let mut output = Vec::new();
    let mut errors = 0usize;
    let mut out_done = false;
    let mut err_done = false;
    let status = loop {
        crate::scoped::check_shutdown()?;
        check()?;
        budget.check()?;
        if let Some(pipe) = stdin.as_mut() {
            let input = if sent < header.len() {
                &header[sent..]
            } else {
                &bytes[sent - header.len()..]
            };
            match pipe.write(input) {
                Ok(n) => {
                    sent += n;
                    if sent == header.len() + bytes.len() {
                        stdin = None;
                        #[cfg(test)]
                        INPUT_HOOK.with(|cell| {
                            if let Some(hook) = cell.borrow_mut().take() {
                                hook(child.0.id());
                            }
                        });
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                    stdin = None;
                }
                _ => return Err(store::storage("pdf_worker_failed")),
            }
        }
        // Como máximo un bloque por pipe por tick: un flood no evita fencing.
        let mut buf = [0u8; 65536];
        if !out_done {
            match stdout.read(&mut buf) {
                Ok(0) => out_done = true,
                Ok(n) => {
                    if output.len() + n > MAX_STDOUT_BYTES {
                        return Err(store::storage("pdf_worker_protocol"));
                    }
                    output.extend_from_slice(&buf[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                _ => return Err(store::storage("pdf_worker_failed")),
            }
        }
        if !err_done {
            match stderr.read(&mut buf) {
                Ok(0) => err_done = true,
                Ok(n) => {
                    errors += n;
                    if errors > MAX_STDERR_BYTES {
                        return Err(store::storage("pdf_worker_protocol"));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                _ => return Err(store::storage("pdf_worker_failed")),
            }
        }
        if let Some(status) = child
            .0
            .try_wait()
            .map_err(|_| store::storage("pdf_worker_failed"))?
            && out_done
            && err_done
        {
            break status;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    check()?;
    budget.check()?;
    if !status.success() {
        use std::os::unix::process::ExitStatusExt;
        if status.signal() == Some(libc::SIGXCPU) {
            return Err(store::error(ErrorCode::WorkspaceBusy, "execution_deadline"));
        }
        return Err(match status.code() {
            Some(73) => store::storage("pdf_memory_limit"),
            Some(75) => store::error(ErrorCode::WorkspaceBusy, "execution_deadline"),
            _ => store::storage("pdf_worker_failed"),
        });
    }
    if errors != 0 || sent != header.len() + bytes.len() {
        return Err(store::storage("pdf_worker_protocol"));
    }
    let reply: Reply =
        serde_json::from_slice(&output).map_err(|_| store::storage("pdf_worker_protocol"))?;
    if reply.protocol_version != noteka_pdf_protocol::PROTOCOL_VERSION
        || reply.extractor_version != 1
        || reply.parser_name != noteka_pdf_protocol::PARSER_NAME
        || reply.parser_version != noteka_pdf_protocol::PARSER_VERSION
    {
        return Err(store::storage("pdf_worker_incompatible"));
    }
    reply
        .validate()
        .map_err(|_| store::storage("pdf_worker_protocol"))?;
    match reply.outcome {
        Outcome::Success {
            source_sha256,
            pages,
        } => {
            if source_sha256 != format!("{:x}", Sha256::digest(bytes)) {
                return Err(store::storage("pdf_worker_protocol"));
            }
            Ok(pages)
        }
        Outcome::Error { reason } => {
            let reason = serde_json::to_value(reason)?
                .as_str()
                .ok_or_else(|| store::storage("pdf_worker_protocol"))?
                .to_owned();
            Err(match reason.as_str() {
                "pdf_worker_failed" | "pdf_worker_protocol" | "pdf_memory_limit" => {
                    store::storage(&reason)
                }
                "execution_deadline" => store::error(ErrorCode::WorkspaceBusy, &reason),
                _ => store::invalid(&reason),
            })
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn fake(script: &str, ms: u64) -> CoreError {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("worker");
        std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let budget = Budget::with_duration(&conn, Duration::from_millis(ms)).unwrap();
        let before = std::time::Instant::now();
        let e = run_at(&path, b"synthetic", &budget, || Ok(())).unwrap_err();
        assert!(before.elapsed() < Duration::from_millis(ms + 2000));
        e
    }
    #[test]
    fn private_pipes_hang_flood_crash_and_closed_errors_are_bounded() {
        assert_eq!(
            fake("exec /usr/bin/head -c 8388609 /dev/zero", 5000).details["reason"],
            "pdf_worker_protocol"
        );
        assert_eq!(
            fake("/usr/bin/head -c 8193 /dev/zero >&2", 1000).details["reason"],
            "pdf_worker_protocol"
        );
        for script in [
            "exec /bin/sleep 10",
            "while :; do printf '0123456789'; done",
            "while :; do printf 'private' >&2; done",
        ] {
            let e = fake(script, 100);
            assert!(
                ["execution_deadline", "pdf_worker_protocol"]
                    .contains(&e.details["reason"].as_str().unwrap())
            );
            assert!(!serde_json::to_string(&e).unwrap().contains("private"));
        }
        for (script, reason) in [
            ("exit 73", "pdf_memory_limit"),
            ("exit 74", "pdf_worker_failed"),
            ("exit 75", "execution_deadline"),
            ("kill -ABRT $$", "pdf_worker_failed"),
            ("printf 'not json'", "pdf_worker_protocol"),
        ] {
            // Los aborts de este fixture no necesitan core dump del proceso test.
            let e = fake(&format!("ulimit -c 0\n{script}"), 1000);
            assert_eq!(e.details["reason"], reason);
        }
    }
    #[test]
    fn handshake_hash_pages_and_cancellation_are_not_trusted() {
        let reply = Reply::new(Outcome::Success {
            source_sha256: format!("{:x}", Sha256::digest(b"synthetic")),
            pages: vec![PageText {
                page_number: 1,
                text: "A\n".into(),
            }],
        });
        let mut value = serde_json::to_value(reply).unwrap();
        for (field, bad) in [
            ("protocol_version", serde_json::json!(2)),
            ("parser_name", serde_json::json!("other")),
        ] {
            let old = value[field].clone();
            value[field] = bad;
            let raw = serde_json::to_string(&value).unwrap();
            assert_eq!(
                fake(&format!("/bin/cat >/dev/null\nprintf '%s' '{raw}'"), 1000).details["reason"],
                "pdf_worker_incompatible"
            );
            value[field] = old;
        }
        value["outcome"]["source_sha256"] = serde_json::json!("0".repeat(64));
        let raw = serde_json::to_string(&value).unwrap();
        assert_eq!(
            fake(&format!("/bin/cat >/dev/null\nprintf '%s' '{raw}'"), 1000).details["reason"],
            "pdf_worker_protocol"
        );
        value["outcome"]["pages"][0]["page_number"] = serde_json::json!(2);
        let raw = serde_json::to_string(&value).unwrap();
        assert_eq!(
            fake(&format!("/bin/cat >/dev/null\nprintf '%s' '{raw}'"), 1000).details["reason"],
            "pdf_worker_protocol"
        );
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let b = Budget::new(&conn).unwrap();
        let missing = Path::new("/no/such/worker");
        assert_eq!(
            run_at(missing, b"A", &b, || Ok(())).unwrap_err().details["reason"],
            "pdf_worker_unavailable"
        );
        assert_eq!(
            run_at(missing, b"A", &b, || Err(store::error(
                ErrorCode::WorkspaceBusy,
                "worker_interrupted"
            )))
            .unwrap_err()
            .details["reason"],
            "worker_interrupted"
        );
    }
}
#[cfg(not(unix))]
pub(crate) fn run_at(
    _: &Path,
    _: &[u8],
    _: &Budget<'_>,
    _: impl FnMut() -> Result<()>,
) -> Result<Vec<PageText>> {
    Err(store::storage("pdf_worker_unavailable"))
}
