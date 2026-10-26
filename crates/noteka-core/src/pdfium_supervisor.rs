//! Proceso descartable con pipes no bloqueantes; nunca espera un thread de IO.
use crate::{batch_storage::Budget, error::*, extraction_storage as store};
use noteka_pdfium_protocol::{
    Header, MAX_STDERR_BYTES, MAX_STDOUT_BYTES, Outcome, PageText, Reply,
};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

fn runtime() -> Result<PathBuf> {
    let exe = std::env::current_exe().map_err(|_| store::storage("pdfium_worker_unavailable"))?;
    let mut dir = exe
        .parent()
        .ok_or_else(|| store::storage("pdfium_worker_unavailable"))?;
    if dir
        .file_name()
        .is_some_and(|n| n == "deps" || n == "examples")
    {
        dir = dir
            .parent()
            .ok_or_else(|| store::storage("pdfium_worker_unavailable"))?;
    }
    Ok(dir.join(if cfg!(windows) {
        "noteka-pdfium-worker.exe"
    } else {
        "noteka-pdfium-worker"
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
        return Err(store::storage("pdfium_worker_failed"));
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
        return Err(store::storage("pdfium_worker_unavailable"));
    }
    if bytes.is_empty() {
        return Err(store::invalid("invalid_pdf"));
    }
    let header = Header {
        source_bytes: bytes.len() as u32,
        remaining_ms: budget.remaining_ms()?,
    }
    .encode()
    .map_err(|_| store::storage("pdfium_worker_protocol"))?;
    let mut command = Command::new(path);
    command.env_clear();
    if let Some(library) = std::env::var_os("NOTEKA_PDFIUM_LIB") {
        command.env("NOTEKA_PDFIUM_LIB", library);
    }
    let child = command
        .current_dir(
            path.parent()
                .ok_or_else(|| store::storage("pdfium_worker_unavailable"))?,
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| store::storage("pdfium_worker_unavailable"))?;
    let mut child = Process(child);
    let mut stdin = Some(
        child
            .0
            .stdin
            .take()
            .ok_or_else(|| store::storage("pdfium_worker_failed"))?,
    );
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| store::storage("pdfium_worker_failed"))?;
    let mut stderr = child
        .0
        .stderr
        .take()
        .ok_or_else(|| store::storage("pdfium_worker_failed"))?;
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
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                    stdin = None;
                }
                _ => return Err(store::storage("pdfium_worker_failed")),
            }
        }
        // Como máximo un bloque por pipe por tick: un flood no evita fencing.
        let mut buf = [0u8; 65536];
        if !out_done {
            match stdout.read(&mut buf) {
                Ok(0) => out_done = true,
                Ok(n) => {
                    if output.len() + n > MAX_STDOUT_BYTES {
                        return Err(store::storage("pdfium_worker_protocol"));
                    }
                    output.extend_from_slice(&buf[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                _ => return Err(store::storage("pdfium_worker_failed")),
            }
        }
        if !err_done {
            match stderr.read(&mut buf) {
                Ok(0) => err_done = true,
                Ok(n) => {
                    errors += n;
                    if errors > MAX_STDERR_BYTES {
                        return Err(store::storage("pdfium_worker_protocol"));
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                _ => return Err(store::storage("pdfium_worker_failed")),
            }
        }
        if let Some(status) = child
            .0
            .try_wait()
            .map_err(|_| store::storage("pdfium_worker_failed"))?
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
            Some(73) => store::storage("pdfium_memory_limit"),
            Some(75) => store::error(ErrorCode::WorkspaceBusy, "execution_deadline"),
            _ => store::storage("pdfium_worker_failed"),
        });
    }
    if errors != 0 || sent != header.len() + bytes.len() {
        return Err(store::storage("pdfium_worker_protocol"));
    }
    let reply: Reply =
        serde_json::from_slice(&output).map_err(|_| store::storage("pdfium_worker_protocol"))?;
    if reply.protocol_version != noteka_pdfium_protocol::PROTOCOL_VERSION
        || reply.extractor_version != 1
        || reply.parser_name != noteka_pdfium_protocol::PARSER_NAME
        || reply.parser_version != noteka_pdfium_protocol::PARSER_VERSION
    {
        return Err(store::storage("pdfium_worker_incompatible"));
    }
    reply
        .validate()
        .map_err(|_| store::storage("pdfium_worker_protocol"))?;
    match reply.outcome {
        Outcome::Success {
            source_sha256,
            pages,
        } => {
            if source_sha256 != format!("{:x}", Sha256::digest(bytes)) {
                return Err(store::storage("pdfium_worker_protocol"));
            }
            Ok(pages)
        }
        Outcome::Error { reason } => {
            let reason = serde_json::to_value(reason)?
                .as_str()
                .ok_or_else(|| store::storage("pdfium_worker_protocol"))?
                .to_owned();
            Err(match reason.as_str() {
                "pdfium_worker_failed" | "pdfium_worker_protocol" | "pdfium_memory_limit" => {
                    store::storage(&reason)
                }
                "pdfium_unavailable" => store::storage("pdfium_worker_unavailable"),
                "execution_deadline" => store::error(ErrorCode::WorkspaceBusy, &reason),
                _ => store::invalid(&reason),
            })
        }
    }
}
