//! Helper XLSX descartable: bytes, pipes no bloqueantes y presupuesto del claim.
use crate::{batch_storage::Budget, error::*, extraction_storage as store};
use noteka_xlsx_protocol::{Header, MAX_STDERR, MAX_STDOUT, Outcome, Reply, Workbook};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

#[cfg(test)]
thread_local! {
    static SPAWNS:std::cell::Cell<u32>=const {std::cell::Cell::new(0)};
    static LAST_PID:std::cell::Cell<Option<u32>>=const {std::cell::Cell::new(None)};
}
#[cfg(test)]
pub(crate) fn spawn_count() -> u32 {
    SPAWNS.with(|n| n.get())
}
fn runtime() -> Result<PathBuf> {
    let exe = std::env::current_exe().map_err(|_| store::storage("xlsx_worker_unavailable"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| store::storage("xlsx_worker_unavailable"))?;
    let dir = if dir.file_name().is_some_and(|n| n == "deps") {
        dir.parent().unwrap_or(dir)
    } else {
        dir
    };
    Ok(dir.join("noteka-xlsx-worker"))
}
pub(crate) fn run(
    bytes: &[u8],
    b: &Budget<'_>,
    check: impl FnMut() -> Result<()>,
) -> Result<Workbook> {
    run_at(&runtime()?, bytes, b, check)
}
struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[cfg(unix)]
fn nonblocking<T: std::os::fd::AsRawFd>(pipe: &T) -> Result<()> {
    let fd = pipe.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(store::storage("xlsx_worker_protocol"));
    }
    Ok(())
}
#[cfg(unix)]
fn drain(r: &mut impl Read, out: &mut Vec<u8>, cap: usize) -> Result<()> {
    let mut buf = [0; 8192];
    loop {
        match r.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                if out.len() + n > cap {
                    return Err(store::storage("xlsx_worker_protocol"));
                }
                out.extend_from_slice(&buf[..n]);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => return Err(store::storage("xlsx_worker_protocol")),
        }
    }
}
#[cfg(unix)]
pub(crate) fn run_at(
    path: &Path,
    bytes: &[u8],
    b: &Budget<'_>,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Workbook> {
    crate::scoped::check_shutdown()?;
    b.check()?;
    check()?;
    let ms = b.remaining_ms()?.min(5000);
    if ms == 0 {
        return Err(store::error(ErrorCode::WorkspaceBusy, "execution_deadline"));
    }
    let header = Header {
        source_bytes: u32::try_from(bytes.len())
            .map_err(|_| store::invalid("source_size_limit"))?,
        remaining_ms: ms,
    }
    .encode()
    .map_err(|_| store::invalid("source_size_limit"))?;
    let mut input = Vec::with_capacity(header.len() + bytes.len());
    input.extend_from_slice(&header);
    input.extend_from_slice(bytes);
    let cwd = tempfile::tempdir().map_err(|_| store::storage("xlsx_worker_unavailable"))?;
    let mut child = Reap(
        Command::new(path)
            .env_clear()
            .current_dir(cwd.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| store::storage("xlsx_worker_unavailable"))?,
    );
    #[cfg(test)]
    SPAWNS.with(|n| n.set(n.get() + 1));
    #[cfg(test)]
    LAST_PID.with(|n| n.set(Some(child.0.id())));
    let mut stdin = Some(
        child
            .0
            .stdin
            .take()
            .ok_or_else(|| store::storage("xlsx_worker_protocol"))?,
    );
    let mut stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| store::storage("xlsx_worker_protocol"))?;
    let mut stderr = child
        .0
        .stderr
        .take()
        .ok_or_else(|| store::storage("xlsx_worker_protocol"))?;
    nonblocking(stdin.as_ref().unwrap())?;
    nonblocking(&stdout)?;
    nonblocking(&stderr)?;
    let (mut written, mut out, mut err) = (0, Vec::new(), Vec::new());
    let mut input_failed = false;
    let status = loop {
        crate::scoped::check_shutdown()?;
        b.check()?;
        check()?;
        if let Some(pipe) = stdin.as_mut() {
            match pipe.write(&input[written..]) {
                Ok(0) => return Err(store::storage("xlsx_worker_protocol")),
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
        drain(&mut stdout, &mut out, MAX_STDOUT)?;
        drain(&mut stderr, &mut err, MAX_STDERR)?;
        if let Some(s) = child
            .0
            .try_wait()
            .map_err(|_| store::storage("xlsx_worker_protocol"))?
        {
            break s;
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    drain(&mut stdout, &mut out, MAX_STDOUT)?;
    drain(&mut stderr, &mut err, MAX_STDERR)?;
    b.check()?;
    check()?;
    if !status.success() {
        return Err(match status.code() {
            Some(73) => store::invalid("xlsx_memory_limit"),
            Some(75) => store::error(ErrorCode::WorkspaceBusy, "execution_deadline"),
            _ => store::storage("xlsx_worker_failed"),
        });
    }
    if input_failed || written != input.len() || !err.is_empty() {
        return Err(store::storage("xlsx_worker_protocol"));
    }
    let reply = Reply::decode(&out, &format!("{:x}", Sha256::digest(bytes)))
        .map_err(|_| store::storage("xlsx_worker_protocol"))?;
    b.check()?;
    check()?;
    match reply.outcome {
        Outcome::Success { workbook, .. } => Ok(workbook),
        Outcome::Error { reason } => {
            let reason = serde_json::to_value(reason)?
                .as_str()
                .ok_or_else(|| store::storage("xlsx_worker_protocol"))?
                .to_owned();
            Err(store::invalid(&reason))
        }
    }
}
#[cfg(not(unix))]
pub(crate) fn run_at(
    _: &Path,
    _: &[u8],
    _: &Budget<'_>,
    _: impl FnMut() -> Result<()>,
) -> Result<Workbook> {
    Err(store::storage("xlsx_worker_unavailable"))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{fs, os::unix::fs::PermissionsExt, time::Instant};
    fn fake(script: &str, ms: u64) -> CoreError {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("worker");
        fs::write(&path, format!("#!/bin/sh\nulimit -c 0\n{script}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let b = Budget::with_duration(&conn, Duration::from_millis(ms)).unwrap();
        let start = Instant::now();
        let e = run_at(&path, b"synthetic", &b, || Ok(())).unwrap_err();
        assert!(start.elapsed() < Duration::from_millis(ms + 2000));
        assert!(!serde_json::to_string(&e).unwrap().contains("private"));
        e
    }
    #[test]
    fn pipes_hang_flood_and_exit_are_bounded() {
        for script in [
            "exec /bin/sleep 10",
            "while :; do printf private; done",
            "while :; do printf private >&2; done",
        ] {
            let e = fake(script, 100);
            assert!(
                ["execution_deadline", "xlsx_worker_protocol"]
                    .contains(&e.details["reason"].as_str().unwrap())
            );
        }
        for (s, r) in [
            ("exit 73", "xlsx_memory_limit"),
            ("exit 74", "xlsx_worker_failed"),
            ("exit 75", "execution_deadline"),
            ("kill -ABRT $$", "xlsx_worker_failed"),
            ("printf 'private'", "xlsx_worker_protocol"),
            (
                "exec /usr/bin/head -c 8388609 /dev/zero",
                "xlsx_worker_protocol",
            ),
            (
                "/usr/bin/head -c 8193 /dev/zero >&2",
                "xlsx_worker_protocol",
            ),
        ] {
            assert_eq!(fake(s, 5000).details["reason"], r);
        }
    }
    #[test]
    fn replies_are_untrusted_and_fence_precedes_spawn() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let b = Budget::new(&conn).unwrap();
        let missing = Path::new("/no/such/xlsx-helper");
        assert_eq!(
            run_at(missing, b"A", &b, || Ok(())).unwrap_err().details["reason"],
            "xlsx_worker_unavailable"
        );
        assert_eq!(
            run_at(missing, b"A", &b, || Err(store::error(
                ErrorCode::WorkspaceBusy,
                "ownership_lost"
            )))
            .unwrap_err()
            .details["reason"],
            "ownership_lost"
        );
        let golden = crate::xlsx_tests::book(1);
        let value = serde_json::to_value(Reply::new(Outcome::Success {
            source_sha256: format!("{:x}", Sha256::digest(b"synthetic")),
            workbook: golden,
        }))
        .unwrap();
        for (pointer, bad) in [
            ("/version", serde_json::json!(2)),
            ("/parser", serde_json::json!("other")),
            ("/outcome/source_sha256", serde_json::json!("0".repeat(64))),
            ("/outcome/workbook/date_1904", serde_json::json!("not_bool")),
        ] {
            let mut v = value.clone();
            *v.pointer_mut(pointer).unwrap() = bad;
            let raw = serde_json::to_string(&v).unwrap();
            assert_eq!(
                fake(&format!("/bin/cat >/dev/null\nprintf '%s' '{raw}'"), 1000).details["reason"],
                "xlsx_worker_protocol"
            );
        }
    }
    #[test]
    fn fence_and_deadline_kill_and_reap_the_owned_process() {
        for interrupted in [false, true] {
            let tmp = tempfile::tempdir().unwrap();
            let path = tmp.path().join("worker");
            fs::write(&path, "#!/bin/sh\nexec /bin/sleep 10\n").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            let conn = rusqlite::Connection::open_in_memory().unwrap();
            let b = Budget::with_duration(&conn, Duration::from_millis(1000)).unwrap();
            LAST_PID.with(|n| n.set(None));
            let e = run_at(&path, b"synthetic", &b, || {
                if interrupted && LAST_PID.with(|n| n.get().is_some()) {
                    Err(store::error(ErrorCode::WorkspaceBusy, "ownership_lost"))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
            assert_eq!(
                e.details["reason"],
                if interrupted {
                    "ownership_lost"
                } else {
                    "execution_deadline"
                }
            );
            let pid = LAST_PID.with(|n| n.get().expect("helper spawned")) as libc::pid_t;
            assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
    }
}
