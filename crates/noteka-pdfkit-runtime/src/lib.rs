//! Consumidor privado PDFKit; no abre workspaces ni publica resultados.
pub mod canonical;
use noteka_pdfkit_protocol::{
    Document, Header, MAX_SOURCE, MAX_STDERR, MAX_STDOUT, Outcome, Reply, WorkerError,
};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Error<E> {
    Guard(E),
    InvalidBudget,
    SourceLimit,
    Unavailable,
    Protocol,
    Deadline,
    ObservedMemoryLimit,
    Setup,
    Failed,
    Rejected(WorkerError),
}
pub type Result<T, E> = std::result::Result<T, Error<E>>;

#[cfg(test)]
thread_local! {
    static LAST_PID: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
}

struct Deadline(Instant);
impl Deadline {
    fn check<E>(&self) -> Result<(), E> {
        if Instant::now() >= self.0 {
            Err(Error::Deadline)
        } else {
            Ok(())
        }
    }
    fn remaining<E>(&self) -> Result<u32, E> {
        let ms = self.0.saturating_duration_since(Instant::now()).as_millis();
        if ms == 0 {
            return Err(Error::Deadline);
        }
        Ok(ms.min(5000) as u32)
    }
}
struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(target_os = "macos")]
fn nonblocking<E>(pipe: &impl std::os::fd::AsRawFd) -> Result<(), E> {
    let fd = pipe.as_raw_fd();
    // SAFETY: descriptor propio vivo, sólo modifica sus flags O_NONBLOCK.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(Error::Protocol);
    }
    Ok(())
}
#[cfg(target_os = "macos")]
fn drain<E>(
    reader: &mut impl Read,
    out: &mut Vec<u8>,
    cap: usize,
    deadline: &Deadline,
) -> Result<(), E> {
    let mut buffer = [0; 8192];
    loop {
        deadline.check()?;
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                if out.len().checked_add(n).is_none_or(|len| len > cap) {
                    return Err(Error::Protocol);
                }
                out.extend_from_slice(&buffer[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(Error::Protocol),
        }
    }
}

/// Lee bytes por un helper explícito; el caller aporta su cerca/guardia de dominio.
/// Sólo el hijo directo pertenece a este consumidor. No mata grupos ajenos.
#[cfg(target_os = "macos")]
pub fn read<E>(
    helper: &Path,
    source: &[u8],
    milliseconds: u32,
    mut guard: impl FnMut() -> std::result::Result<(), E>,
) -> Result<Document, E> {
    if !(1..=5000).contains(&milliseconds) {
        return Err(Error::InvalidBudget);
    }
    let deadline = Deadline(Instant::now() + Duration::from_millis(u64::from(milliseconds)));
    guard().map_err(Error::Guard)?;
    if source.is_empty() || source.len() > MAX_SOURCE {
        return Err(Error::SourceLimit);
    }
    let helper = helper.canonicalize().map_err(|_| Error::Unavailable)?;
    if !helper.is_file() {
        return Err(Error::Unavailable);
    }
    let header = Header {
        source_bytes: source.len() as u32,
        remaining_ms: deadline.remaining()?,
    }
    .encode()
    .map_err(|_| Error::Protocol)?;
    let mut input = Vec::with_capacity(header.len() + source.len());
    input.extend_from_slice(&header);
    input.extend_from_slice(source);
    let cwd = tempfile::tempdir().map_err(|_| Error::Unavailable)?;
    deadline.check()?;
    guard().map_err(Error::Guard)?;
    let mut child = Reap(
        Command::new("/usr/bin/sandbox-exec")
            .args(["-p", "(version 1)(allow default)(deny network*)"])
            .arg(helper)
            .env_clear()
            .env("TMPDIR", cwd.path())
            .current_dir(cwd.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| Error::Unavailable)?,
    );
    #[cfg(test)]
    LAST_PID.with(|pid| pid.set(Some(child.0.id())));
    let mut stdin = Some(child.0.stdin.take().ok_or(Error::Protocol)?);
    let mut stdout = child.0.stdout.take().ok_or(Error::Protocol)?;
    let mut stderr = child.0.stderr.take().ok_or(Error::Protocol)?;
    nonblocking(stdin.as_ref().ok_or(Error::Protocol)?)?;
    nonblocking(&stdout)?;
    nonblocking(&stderr)?;
    let (mut written, mut out, mut err) = (0, Vec::new(), Vec::new());
    let mut input_failed = false;
    let status = loop {
        deadline.check()?;
        guard().map_err(Error::Guard)?;
        if let Some(pipe) = stdin.as_mut() {
            match pipe.write(&input[written..]) {
                Ok(0) => return Err(Error::Protocol),
                Ok(n) => written += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => {
                    input_failed = true;
                    stdin = None;
                }
            }
            if written == input.len() {
                stdin = None;
            }
        }
        drain(&mut stdout, &mut out, MAX_STDOUT, &deadline)?;
        drain(&mut stderr, &mut err, MAX_STDERR, &deadline)?;
        if let Some(status) = child.0.try_wait().map_err(|_| Error::Protocol)? {
            break status;
        }
        std::thread::sleep(
            Duration::from_millis(2).min(deadline.0.saturating_duration_since(Instant::now())),
        );
    };
    drain(&mut stdout, &mut out, MAX_STDOUT, &deadline)?;
    drain(&mut stderr, &mut err, MAX_STDERR, &deadline)?;
    deadline.check()?;
    guard().map_err(Error::Guard)?;
    if !status.success() {
        return Err(match status.code() {
            Some(70) => Error::Setup,
            Some(73) => Error::ObservedMemoryLimit,
            Some(75) => Error::Deadline,
            _ => Error::Failed,
        });
    }
    if input_failed || written != input.len() || !err.is_empty() {
        return Err(Error::Protocol);
    }
    let reply = Reply::decode(&out, &format!("{:x}", Sha256::digest(source)))
        .map_err(|_| Error::Protocol)?;
    deadline.check()?;
    guard().map_err(Error::Guard)?;
    match reply.outcome {
        Outcome::Success { document, .. } => Ok(*document),
        Outcome::Error { reason } => Err(Error::Rejected(reason)),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn read<E>(
    _: &Path,
    _: &[u8],
    _: u32,
    _: impl FnMut() -> std::result::Result<(), E>,
) -> Result<Document, E> {
    Err(Error::Unavailable)
}

#[cfg(all(test, target_os = "macos"))]
mod tests;
