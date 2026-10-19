use super::*;
use std::{fs, os::unix::fs::PermissionsExt};

fn worker(script: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("worker");
    fs::write(&path, format!("#!/bin/sh\nulimit -c 0\n{script}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    (tmp, path)
}
fn run(script: &str, ms: u32) -> Error<()> {
    let (_tmp, path) = worker(script);
    let start = Instant::now();
    let error = read(&path, b"synthetic", ms, || Ok(())).unwrap_err();
    assert!(start.elapsed() < Duration::from_millis(u64::from(ms) + 2000));
    assert!(!format!("{error:?}").contains("private-source"));
    error
}
#[test]
fn source_budget_and_pre_spawn_guard() {
    let absent = Path::new("/no/such/noteka-pdfkit-helper");
    assert_eq!(
        read(absent, b"x", 0, || Ok::<_, ()>(())).unwrap_err(),
        Error::InvalidBudget
    );
    assert_eq!(
        read(absent, b"x", 5001, || Ok::<_, ()>(())).unwrap_err(),
        Error::InvalidBudget
    );
    assert_eq!(
        read(absent, b"", 5000, || Ok::<_, ()>(())).unwrap_err(),
        Error::SourceLimit
    );
    assert_eq!(
        read(absent, &vec![0; MAX_SOURCE + 1], 5000, || Ok::<_, ()>(())).unwrap_err(),
        Error::SourceLimit
    );
    assert_eq!(
        read(absent, b"x", 5000, || Ok::<_, ()>(())).unwrap_err(),
        Error::Unavailable
    );
    assert_eq!(
        read(absent, b"x", 5000, || Err("ownership_lost")).unwrap_err(),
        Error::Guard("ownership_lost")
    );
}
#[test]
fn hangs_pipes_exits_and_signal_have_bounded_sanitized_errors() {
    for script in [
        "exec /bin/sleep 10",
        "while :; do printf private-source; done",
        "while :; do printf private-source >&2; done",
    ] {
        assert!(matches!(
            run(script, 100),
            Error::Deadline | Error::Protocol
        ));
    }
    for (script, expected) in [
        ("exit 70", Error::Setup),
        ("exit 73", Error::ObservedMemoryLimit),
        ("exit 74", Error::Failed),
        ("exit 75", Error::Deadline),
        ("kill -TERM $$", Error::Failed),
        ("printf private-source", Error::Protocol),
        ("exec /usr/bin/head -c 8388609 /dev/zero", Error::Protocol),
        ("/usr/bin/head -c 8193 /dev/zero >&2", Error::Protocol),
    ] {
        assert_eq!(run(script, 5000), expected);
    }
}
#[test]
fn every_success_field_is_untrusted() {
    let value = serde_json::json!({"version":1,"profile":noteka_pdfkit_protocol::PROFILE,"outcome":{
        "status":"success","source_sha256":format!("{:x}",Sha256::digest(b"synthetic")),
        "document":{"version":1,"engine":{"api":"PDFKit.PDFPage.attributedString","os_version":"27.0.1","os_build":"26A434"},
        "limitations":noteka_pdfkit_protocol::LIMITATIONS,"page_count":1,"text_bytes":0,"pages":[{"number":1,"status":"no_text","text":""}]}}});
    let raw = serde_json::to_string(&value).unwrap();
    let (_tmp, path) = worker(&format!("/bin/cat >/dev/null\nprintf '%s' '{raw}'"));
    assert_eq!(
        read(&path, b"synthetic", 5000, || Ok::<_, ()>(()))
            .unwrap()
            .page_count,
        1
    );
    for (pointer, bad) in [
        ("/version", serde_json::json!(2)),
        ("/profile", serde_json::json!("other")),
        ("/outcome/source_sha256", serde_json::json!("0".repeat(64))),
        ("/outcome/document/page_count", serde_json::json!(2)),
    ] {
        let mut altered = value.clone();
        *altered.pointer_mut(pointer).unwrap() = bad;
        assert_eq!(
            run(
                &format!(
                    "/bin/cat >/dev/null\nprintf '%s' '{}'",
                    serde_json::to_string(&altered).unwrap()
                ),
                5000
            ),
            Error::Protocol
        );
    }
    assert_eq!(
        run(
            &format!("/bin/cat >/dev/null\nprintf private-source >&2\nprintf '%s' '{raw}'"),
            5000
        ),
        Error::Protocol
    );
}
#[test]
fn cancellation_and_deadline_reap_only_own_child() {
    for cancel in [false, true] {
        let (_tmp, path) = worker("exec /bin/sleep 10");
        LAST_PID.with(|pid| pid.set(None));
        let error = read(&path, b"synthetic", 300, || {
            if cancel && LAST_PID.with(|pid| pid.get().is_some()) {
                Err("cancelled")
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(
            error,
            if cancel {
                Error::Guard("cancelled")
            } else {
                Error::Deadline
            }
        );
        let pid = LAST_PID.with(|pid| pid.get().expect("own child spawned")) as libc::pid_t;
        // SAFETY: signal0 sólo consulta el PID propio registrado en spawn y recolectado.
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
    }
}
