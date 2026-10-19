use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn cli(path: &Path, group: &str, action: &str, input: Value, code: i32) -> Value {
    let mut p = Command::new(env!("CARGO_BIN_EXE_noteka"))
        .args([
            "--json",
            "--workspace",
            path.to_str().unwrap(),
            group,
            action,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    p.stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = p.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(out.stderr.is_empty());
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text.lines().count(), 1);
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], code == 0);
    v
}
#[test]
fn real_cli_explicit_extraction_parity_channels_and_errors() {
    let t = tempfile::tempdir().unwrap();
    let w = t.path().join("w");
    cli(&w, "workspace", "create", json!({"timezone":"UTC"}), 0);
    let text = "\u{feff}literal 😀\r\n<script>fetch('https://example.invalid')</script>\nno shell: $(touch should-not-exist)";
    let file = t.path().join("literal.md");
    std::fs::write(&file, text).unwrap();
    let a = cli(&w, "attachment", "import", json!({"source_path":file}), 0)["data"]["id"].clone();
    assert!(
        cli(&w, "extraction", "list", json!({}), 0)["data"]["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let j = cli(
        &w,
        "extraction",
        "enqueue",
        json!({"attachment_id":a,"extractor":"utf8_text"}),
        0,
    )["data"]["job"]
        .clone();
    let id = j["id"].clone();
    assert_eq!(
        cli(&w, "extraction", "run", json!({}), 0)["data"]["reason"],
        "queue_paused"
    );
    assert_eq!(
        cli(
            &w,
            "extraction",
            "enqueue",
            json!({"attachment_id":a,"extractor":"utf8_text"}),
            0
        )["data"]["deduplicated"],
        true
    );
    cli(
        &w,
        "extraction",
        "control",
        json!({"action":"set","paused":false,"expected_revision":1}),
        0,
    );
    let run = cli(&w, "extraction", "run", json!({"id":id}), 0);
    assert_eq!(run["data"]["outcome"], "succeeded");
    assert_eq!(run["data"]["job"]["attempts"], 1);
    let fragments = cli(&w, "extraction", "fragments", json!({"id":id}), 0);
    assert_eq!(fragments["data"]["items"][0]["text"], text);
    assert_eq!(fragments["data"]["covers_generation"], true);
    for (operation, input) in [
        ("get", json!({"id":id})),
        ("list", json!({})),
        ("fragments", json!({"id":id})),
        ("control", json!({"action":"get"})),
    ] {
        let result = cli(&w, "extraction", operation, input.clone(), 0);
        let core = noteka_core::execute_json(
            json!({"workspace":w,"command":{"operation":format!("extraction.{operation}"),"input":input}}),
            "desktop",
        );
        assert!(core.ok);
        assert_eq!(result["data"], core.data.unwrap());
    }
    cli(
        &w,
        "extraction",
        "retry",
        json!({"id":id,"expected_revision":3}),
        0,
    );
    assert_eq!(
        cli(&w, "extraction", "fragments", json!({"id":id}), 0)["data"]["result_from_previous_attempt"],
        true
    );
    cli(
        &w,
        "extraction",
        "cancel",
        json!({"id":id,"expected_revision":4}),
        0,
    );
    assert_eq!(
        cli(
            &w,
            "extraction",
            "cancel",
            json!({"id":id,"expected_revision":5}),
            0
        )["data"]["revision"],
        5
    );
    let conflict = cli(
        &w,
        "extraction",
        "cancel",
        json!({"id":id,"expected_revision":4}),
        4,
    );
    assert_eq!(conflict["error"]["code"], "revision_conflict");
    let core = noteka_core::execute_json(
        json!({"workspace":w,"command":{"operation":"extraction.cancel","input":{"id":id,"expected_revision":4}}}),
        "desktop",
    );
    assert!(!core.ok);
    assert_eq!(
        conflict["error"],
        serde_json::to_value(core.error.unwrap()).unwrap()
    );
    cli(
        &w,
        "extraction",
        "retry",
        json!({"id":id,"expected_revision":5}),
        0,
    );
    cli(&w, "extraction", "run", json!({"id":id}), 0);
    assert_eq!(
        cli(
            &w,
            "extraction",
            "get",
            json!({"id":"00000000-0000-0000-0000-000000000000"}),
            3
        )["error"]["code"],
        "not_found"
    );
    for (action, input) in [
        ("control", json!({"action":"get","paused":true})),
        ("control", json!({"action":"set","paused":true})),
        ("get", json!({"id":"../escape"})),
        ("run", json!({"token":"private"})),
        ("list", json!({"limit":101})),
        ("fragments", json!({"id":id,"limit":65})),
    ] {
        let error = cli(&w, "extraction", action, input.clone(), 2);
        let core = noteka_core::execute_json(
            json!({"workspace":w,"command":{"operation":format!("extraction.{action}"),"input":input}}),
            "desktop",
        );
        assert!(!core.ok);
        assert_eq!(
            error["error"],
            serde_json::to_value(core.error.unwrap()).unwrap()
        );
    }
    let bad = t.path().join("invalid.txt");
    std::fs::write(&bad, [0xff]).unwrap();
    let a = cli(&w, "attachment", "import", json!({"source_path":bad}), 0)["data"]["id"].clone();
    let j = cli(
        &w,
        "extraction",
        "enqueue",
        json!({"attachment_id":a,"extractor":"utf8_text"}),
        0,
    )["data"]["job"]
        .clone();
    let error = cli(&w, "extraction", "run", json!({"id":j["id"]}), 2);
    assert_eq!(error["error"]["details"]["reason"], "invalid_utf8");
    assert_eq!(error["error"]["details"]["state"], "failed");
    cli(&w, "extraction", "fragments", json!({"id":j["id"]}), 3);
    let path = w.join("attachments").join(a.as_str().unwrap());
    std::fs::remove_file(path).unwrap();
    cli(
        &w,
        "extraction",
        "retry",
        json!({"id":j["id"],"expected_revision":3}),
        0,
    );
    let error = cli(&w, "extraction", "run", json!({"id":j["id"]}), 5);
    assert_eq!(error["error"]["details"]["state"], "stale");
    assert!(!t.path().join("should-not-exist").exists());
}
