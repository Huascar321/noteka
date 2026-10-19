use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn cli(path: &std::path::Path, group: &str, action: &str, input: &Value, expected: i32) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_noteka"))
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
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(expected),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}
#[test]
fn batch_cli_stdin_files_receipts_transport_parity_and_structured_failures() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    let plan = json!({"version":1,"batch_id":"00000000-0000-0000-0000-000000000001","steps":[{"key":"note","operation":"entity.create","input":{"kind":"note","title":"Batch CLI"}}]});
    let preview = cli(&path, "changes", "preview", &plan, 0);
    let core = noteka_core::execute_json(
        json!({"workspace":path,"command":{"operation":"changes.preview","input":plan}}),
        "desktop",
    );
    assert!(core.ok);
    assert_eq!(preview["data"]["receipt"], core.data.unwrap()["receipt"]);
    assert!(
        cli(&path, "entity", "list", &json!({}), 0)["data"]["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let input = json!({"plan":plan,"expected":preview["data"]["receipt"]});
    let file = temp.path().join("apply.json");
    std::fs::write(&file, input.to_string()).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_noteka"))
        .args([
            "--json",
            "--workspace",
            path.to_str().unwrap(),
            "changes",
            "apply",
            "--input",
            file.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let applied: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(applied["data"]["applied"], true);
    assert_eq!(
        applied["data"]["steps"][0]["exports"],
        preview["data"]["steps"][0]["exports"]
    );
    let history = cli(&path, "changes", "list", &json!({}), 0);
    assert_eq!(
        history["data"]["items"][0]["request_id"],
        applied["request_id"]
    );
    assert_eq!(history["data"]["items"][0]["origin"], "cli");
    assert_eq!(
        cli(&path, "changes", "apply", &input, 4)["error"]["details"]["reason"],
        "base_hash_mismatch"
    );
    assert_eq!(
        cli(&path, "changes", "apply", &json!({"plan":plan}), 2)["error"]["code"],
        "validation_error"
    );
    let huge = json!({"plan":"x".repeat(2_097_152)});
    assert_eq!(
        cli(&path, "changes", "preview", &huge, 2)["error"]["code"],
        "validation_error"
    );
    let help = Command::new(env!("CARGO_BIN_EXE_noteka"))
        .args(["changes", "--help"])
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("preview") && help.contains("apply") && help.contains("list"));
}
#[test]
fn home_views_strict_errors_and_projection_parity() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    cli(
        &path,
        "entity",
        "create",
        &json!({"kind":"task","title":"Synthetic","payload":{"due_date":"2026-10-12"}}),
        0,
    );
    for view in [
        "tasks_today",
        "tasks_inbox",
        "tasks_all",
        "tasks_done",
        "notes_all",
        "notes_inbox",
        "projects_active",
        "projects_planned",
        "projects_board",
        "tags_recent",
        "tags_alphabetical",
        "tags_types",
        "people_contacts",
        "people_new",
        "people_updated",
    ] {
        let input = json!({"view":view,"as_of":"2026-10-12"});
        let real = cli(&path, "home", "query", &input, 0);
        let core = noteka_core::execute_json(
            json!({"workspace":path,"command":{"operation":"home.query","input":input}}),
            "desktop",
        );
        assert!(core.ok);
        assert_eq!(real["data"], core.data.unwrap());
    }
    let error = cli(
        &path,
        "home",
        "query",
        &json!({"view":"tasks_today","extra":true}),
        2,
    );
    assert_eq!(error["error"]["code"], "validation_error");
}
#[test]
fn concurrent_cli_writer_and_home_reads_keep_reference_metadata_coherent() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    let task = cli(
        &path,
        "entity",
        "create",
        &json!({"kind":"task","title":"Concurrent","payload":{"state":"todo"}}),
        0,
    )["data"]
        .clone();
    let writer_path = path.clone();
    let task_id = task["id"].clone();
    let writer = std::thread::spawn(move || {
        for revision in 1..=30 {
            cli(
                &writer_path,
                "entity",
                "update",
                &json!({"id":task_id,"expected_revision":revision,"payload":{"state":if revision%2==1{"done"}else{"todo"}}}),
                0,
            );
        }
    });
    for _ in 0..60 {
        let r = noteka_core::execute_json(
            json!({"workspace":path,"command":{"operation":"home.query","input":{"view":"tasks_all"}}}),
            "desktop",
        );
        assert!(r.ok, "{:?}", r.error);
        let page = r.data.unwrap();
        let item = &page["items"][0];
        let rev = item["entity"]["revision"].as_u64().unwrap();
        assert_eq!(
            item["metadata"]["state"],
            if rev % 2 == 1 { "todo" } else { "done" }
        );
    }
    writer.join().unwrap();
    assert_eq!(
        cli(&path, "entity", "get", &json!({"id":task["id"]}), 0)["data"]["revision"],
        31
    );
}
#[test]
fn templates_instances_embedded_documents_and_recovery_match_core() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    let c = cli(
        &path,
        "collection",
        "create",
        &json!({"name":"Templates"}),
        0,
    )["data"]["collection"]["id"]
        .clone();
    let v = cli(
        &path,
        "view",
        "save",
        &json!({"collection_id":c,"name":"Shared","layout":"gallery","config":{}}),
        0,
    )["data"]["id"]
        .clone();
    let save = json!({"collection_id":c,"expected_collection_revision":1,"name":"Synthetic template","definition":{"schema_version":1,"title":"Synthetic instance","document":{"schema_version":3,"blocks":[{"id":"ref","type":"collection_view","content":[],"attrs":{"collection_id":c,"view_id":v}}]},"values":{}}});
    let t = cli(&path, "template", "save", &save, 0)["data"].clone();
    assert_eq!(
        cli(&path, "template", "list", &json!({"collection_id":c}), 0)["data"]["items"],
        json!([t])
    );
    let core = noteka_core::execute_json(
        json!({"workspace":path,"command":{"operation":"template.get","input":{"id":t["id"]}}}),
        "desktop",
    );
    assert!(core.ok);
    assert_eq!(
        core.data.unwrap(),
        cli(&path, "template", "get", &json!({"id":t["id"]}), 0)["data"]
    );
    let instantiate = json!({"id":t["id"],"expected_template_revision":1,"expected_collection_revision":1,"values":{}});
    let one = cli(&path, "template", "instantiate", &instantiate, 0)["data"].clone();
    let two = cli(&path, "template", "instantiate", &instantiate, 0)["data"].clone();
    assert_ne!(one["entity"]["id"], two["entity"]["id"]);
    assert_ne!(
        one["entity"]["document"]["blocks"][0]["id"],
        two["entity"]["document"]["blocks"][0]["id"]
    );
    assert_eq!(
        cli(
            &path,
            "entity",
            "get",
            &json!({"id":one["entity"]["id"]}),
            0
        )["data"],
        one["entity"]
    );
    let mut update = save.clone();
    update["id"] = t["id"].clone();
    update["expected_revision"] = json!(1);
    assert_eq!(
        cli(&path, "template", "save", &update, 0)["data"]["revision"],
        2
    );
    assert_eq!(
        cli(&path, "template", "save", &update, 4)["error"]["code"],
        "revision_conflict"
    );
    cli(&path, "template", "instantiate", &instantiate, 4);
    cli(&path, "template", "get", &json!({"id":"missing"}), 3);
    cli(
        &path,
        "template",
        "list",
        &json!({"collection_id":c,"limit":0}),
        2,
    );
    let mut bad = instantiate;
    bad["expected_template_revision"] = json!(2);
    bad["title"] = Value::Null;
    cli(&path, "template", "instantiate", &bad, 2);
    let mut future = t["definition"].clone();
    future["future"] = json!({"keep":true});
    rusqlite::Connection::open(path.join("noteka.sqlite"))
        .unwrap()
        .execute(
            "UPDATE templates SET definition=?2 WHERE id=?1",
            rusqlite::params![t["id"].as_str().unwrap(), future.to_string()],
        )
        .unwrap();
    bad.as_object_mut().unwrap().remove("title");
    cli(&path, "template", "instantiate", &bad, 6);
    assert_eq!(
        cli(&path, "template", "get", &json!({"id":t["id"]}), 0)["data"]["definition"],
        future
    );
    let backup = temp.path().join("backup");
    let restored = temp.path().join("restored");
    cli(
        &path,
        "workspace",
        "backup",
        &json!({"destination":backup}),
        0,
    );
    cli(
        &path,
        "workspace",
        "restore",
        &json!({"backup":backup,"destination":restored}),
        0,
    );
    assert_eq!(
        cli(&restored, "template", "get", &json!({"id":t["id"]}), 0)["data"]["definition"],
        future
    );
    assert_eq!(
        cli(
            &restored,
            "database",
            "query",
            &json!({"collection_id":c,"view_id":v}),
            0
        )["data"]["matching_count"],
        2
    );
}

#[test]
fn real_processes_all_groups_and_parity() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    cli(&path, "workspace", "open", &json!({}), 0);
    assert_eq!(
        cli(&path, "operations", "describe", &json!({}), 0)["data"]["api_version"],
        1
    );
    let payload: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/note.json")).unwrap();
    let n = cli(&path, "entity", "create", &payload, 0)["data"].clone();
    let entity_id = &n["id"];
    assert_eq!(
        cli(&path, "entity", "get", &json!({"id":entity_id}), 0)["data"]["document"],
        payload["document"]
    );
    cli(&path, "entity", "list", &json!({}), 0);
    let p = cli(
        &path,
        "entity",
        "create",
        &json!({"kind":"project","title":"CRE"}),
        0,
    )["data"]
        .clone();
    let r = cli(
        &path,
        "relation",
        "add",
        &json!({"source_id":entity_id,"target_id":p["id"],"kind":"project","expected_revision":1}),
        0,
    )["data"]
        .clone();
    cli(&path, "relation", "list", &json!({"entity_id":p["id"]}), 0);
    cli(
        &path,
        "relation",
        "remove",
        &json!({"id":r["relation"]["id"],"expected_revision":2}),
        0,
    );
    cli(
        &path,
        "entity",
        "update",
        &json!({"id":entity_id,"expected_revision":3,"title":"CLI actualizado"}),
        0,
    );
    cli(
        &path,
        "entity",
        "archive",
        &json!({"id":entity_id,"expected_revision":4}),
        0,
    );
    cli(
        &path,
        "entity",
        "restore",
        &json!({"id":entity_id,"expected_revision":5}),
        0,
    );
    let history = cli(&path, "changes", "list", &json!({"entity_id":entity_id}), 0);
    assert_eq!(history["data"]["items"].as_array().unwrap().len(), 6);
    let desktop = noteka_core::execute_json(
        json!({"workspace":path,"command":{"operation":"entity.get","input":{"id":entity_id}}}),
        "desktop",
    );
    assert_eq!(
        desktop.data.unwrap(),
        cli(&path, "entity", "get", &json!({"id":entity_id}), 0)["data"]
    );
    // Ambos adaptadores aplican la misma modificación, no sólo lectura.
    let other = temp.path().join("other");
    cli(&other, "workspace", "create", &json!({}), 0);
    let other_note = noteka_core::execute_json(
        json!({"workspace":other,"command":{"operation":"entity.create","input":payload}}),
        "desktop",
    )
    .data
    .unwrap();
    let ui=noteka_core::execute_json(json!({"workspace":other,"command":{"operation":"entity.update","input":{"id":other_note["id"],"expected_revision":1,"title":"Equivalente"}}}),"desktop").data.unwrap();
    let cli_updated = cli(
        &path,
        "entity",
        "update",
        &json!({"id":entity_id,"expected_revision":6,"title":"Equivalente"}),
        0,
    )["data"]
        .clone();
    assert_eq!(ui["title"], cli_updated["title"]);
    assert_eq!(ui["document"], cli_updated["document"]);
    let backup = temp.path().join("backup");
    cli(
        &path,
        "workspace",
        "backup",
        &json!({"destination":backup}),
        0,
    );
    cli(
        &path,
        "workspace",
        "restore",
        &json!({"backup":backup,"destination":temp.path().join("restore")}),
        0,
    );
    let input = temp.path().join("get.json");
    std::fs::write(&input, json!({"id":entity_id}).to_string()).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_noteka"))
        .args([
            "--json",
            "--workspace",
            path.to_str().unwrap(),
            "--input",
            input.to_str().unwrap(),
            "entity",
            "get",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap()["ok"]
            .as_bool()
            .unwrap()
    );
}
#[test]
fn json_channels_exit_codes_and_invalid_arguments() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    cli(
        &path,
        "entity",
        "create",
        &json!({"kind":"note","title":""}),
        2,
    );
    cli(&path, "entity", "get", &json!({"id":"absent"}), 3);
    let n = cli(
        &path,
        "entity",
        "create",
        &json!({"kind":"note","title":"Test"}),
        0,
    )["data"]
        .clone();
    cli(
        &path,
        "entity",
        "update",
        &json!({"id":n["id"],"expected_revision":9,"title":"Bad"}),
        4,
    );
    let c = rusqlite::Connection::open(path.join("noteka.sqlite")).unwrap();
    c.execute_batch(
        "CREATE TRIGGER fail BEFORE INSERT ON changes BEGIN SELECT RAISE(ABORT,'injected'); END;",
    )
    .unwrap();
    cli(
        &path,
        "entity",
        "update",
        &json!({"id":n["id"],"expected_revision":1,"title":"Bad"}),
        5,
    );
    c.execute_batch("DROP TRIGGER fail").unwrap();
    c.execute(
        "UPDATE documents SET json=?2 WHERE entity_id=?1",
        rusqlite::params![
            n["id"].as_str().unwrap(),
            json!({"schema_version":99,"blocks":[]}).to_string()
        ],
    )
    .unwrap();
    cli(
        &path,
        "entity",
        "update",
        &json!({"id":n["id"],"expected_revision":1,"document":{"schema_version":1,"blocks":[]}}),
        6,
    );
    c.execute_batch("BEGIN IMMEDIATE").unwrap();
    cli(
        &path,
        "entity",
        "archive",
        &json!({"id":n["id"],"expected_revision":1}),
        4,
    );
    c.execute_batch("ROLLBACK").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_noteka"))
        .args(["--json", "entity", "bogus"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap()["error"]["code"],
        "validation_error"
    );
    let input = temp.path().join("invalid.json");
    std::fs::write(&input, "{no-json").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_noteka"))
        .args([
            "--json",
            "--input",
            input.to_str().unwrap(),
            "entity",
            "create",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(serde_json::from_slice::<Value>(&out.stdout).is_ok());
}

#[test]
fn attachment_and_export_groups_use_the_same_core() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    let source = temp.path().join("fixture.txt");
    std::fs::write(&source, b"local original").unwrap();
    let a = cli(
        &path,
        "attachment",
        "import",
        &json!({"source_path":source}),
        0,
    )["data"]
        .clone();
    assert_eq!(
        cli(&path, "attachment", "read", &json!({"id":a["id"]}), 0)["data"]["base64"],
        "bG9jYWwgb3JpZ2luYWw="
    );
    let d = json!({"schema_version":2,"blocks":[{"id":"file","type":"file","content":[],"attrs":{"attachment_id":a["id"],"name":"fixture.txt"}}]});
    let n = cli(
        &path,
        "entity",
        "create",
        &json!({"kind":"note","title":"Con original","document":d}),
        0,
    )["data"]
        .clone();
    let export = temp.path().join("export");
    assert!(
        cli(
            &path,
            "document",
            "export",
            &json!({"id":n["id"],"destination":export,"format":"json"}),
            0
        )["data"]["warnings"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        std::fs::read(export.join("attachments").join(a["id"].as_str().unwrap())).unwrap(),
        b"local original"
    );
    cli(
        &path,
        "document",
        "export",
        &json!({"id":n["id"],"destination":export,"format":"json"}),
        2,
    );
}
#[test]
fn workflow_query_and_contextual_capture_match_desktop() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(
        &path,
        "workspace",
        "create",
        &json!({"timezone":"America/La_Paz"}),
        0,
    );
    let p = cli(
        &path,
        "entity",
        "create",
        &json!({"kind":"project","title":"CRE"}),
        0,
    )["data"]
        .clone();
    let t=cli(&path,"entity","create",&json!({"kind":"task","title":"Plan","payload":{"planned_date":"2026-10-12","priority":"high"},"context":{"kind":"project","target_id":p["id"]}}),0)["data"].clone();
    let input = json!({"view":"my_day","as_of":"2026-10-12"});
    let from_cli = cli(&path, "workflow", "query", &input, 0)["data"].clone();
    assert_eq!(from_cli["items"][0]["id"], t["id"]);
    let desktop = noteka_core::execute_json(
        json!({"workspace":path,"command":{"operation":"workflow.query","input":input}}),
        "desktop",
    );
    assert_eq!(desktop.data.unwrap(), from_cli);
    assert_eq!(
        cli(
            &path,
            "workflow",
            "query",
            &json!({"view":"context","related_id":p["id"]}),
            0
        )["data"]["items"][0]["id"],
        t["id"]
    );
    cli(
        &path,
        "workflow",
        "query",
        &json!({"view":"my_day","as_of":"2026-02-30"}),
        2,
    );
}
#[test]
fn search_query_rebuild_and_provenance_match_desktop() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    let n=cli(&path,"entity","create",&json!({"kind":"note","title":"Acuerdo","document":{"schema_version":2,"blocks":[{"id":"source","type":"paragraph","content":[{"type":"text","text":"Evidencia textual"}]}]}}),0)["data"].clone();
    let input = json!({"query":"Evidencia"});
    let result = cli(&path, "search", "query", &input, 0)["data"].clone();
    assert_eq!(result["items"][0]["entity_id"], n["id"]);
    assert_eq!(result["items"][0]["block_id"], "source");
    assert_eq!(
        noteka_core::execute_json(
            json!({"workspace":path,"command":{"operation":"search.query","input":input}}),
            "desktop"
        )
        .data
        .unwrap(),
        result
    );
    assert_eq!(
        cli(&path, "search", "rebuild", &json!({}), 0)["data"]["entities"],
        1
    );
    cli(&path, "search", "query", &json!({"query":""}), 2);
}
#[test]
fn collections_records_views_and_relations_match_desktop() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    let c = cli(&path, "collection", "create", &json!({"name":"Members"}), 0)["data"]["collection"]
        .clone();
    assert_eq!(
        cli(&path, "collection", "list", &json!({}), 0)["data"]["items"][0],
        c
    );
    let p=cli(&path,"property","define",&json!({"collection_id":c["id"],"expected_revision":1,"spec":{"name":"Cuota","type":"money","currency":"BOB"}}),0)["data"].clone();
    let n=cli(&path,"record","create",&json!({"collection_id":c["id"],"title":"Member","values":{p["id"].as_str().unwrap():"24.28"}}),0)["data"].clone();
    let q = json!({"collection_id":c["id"]});
    let result = cli(&path, "database", "query", &q, 0)["data"].clone();
    assert_eq!(
        noteka_core::execute_json(
            json!({"workspace":path,"command":{"operation":"database.query","input":q}}),
            "desktop"
        )
        .data
        .unwrap(),
        result
    );
    let cfg = json!({"sorts":[{"property_id":p["id"]}]});
    let v = cli(
        &path,
        "view",
        "save",
        &json!({"collection_id":c["id"],"name":"Cuotas","layout":"list","config":cfg}),
        0,
    )["data"]
        .clone();
    assert_eq!(
        cli(&path, "collection", "get", &json!({"id":c["id"]}), 0)["data"]["views"][0],
        v
    );
    let update = json!({"collection_id":c["id"],"entity_id":n["entity"]["id"],"expected_revision":1,"values":{p["id"].as_str().unwrap():"48.57"}});
    cli(&path, "record", "update", &update, 0);
    cli(&path, "record", "update", &update, 4);
    let second =
        cli(&path, "collection", "create", &json!({"name":"Other"}), 0)["data"]["collection"]
            .clone();
    cli(
        &path,
        "record",
        "attach",
        &json!({"collection_id":second["id"],"entity_id":n["entity"]["id"],"expected_revision":2}),
        0,
    );
    let rel=cli(&path,"property","define",&json!({"collection_id":c["id"],"expected_revision":2,"spec":{"name":"Link","type":"relation","target_collection_id":second["id"]}}),0)["data"].clone();
    cli(
        &path,
        "record",
        "update",
        &json!({"collection_id":c["id"],"entity_id":n["entity"]["id"],"expected_revision":3,"values":{rel["id"].as_str().unwrap():[n["entity"]["id"]]}}),
        0,
    );
    assert_eq!(
        cli(
            &path,
            "record",
            "inverse",
            &json!({"property_id":rel["id"],"target_id":n["entity"]["id"]}),
            0
        )["data"]["items"][0]["entity"]["id"],
        n["entity"]["id"]
    );
    assert_eq!(
        cli(
            &path,
            "record",
            "get",
            &json!({"collection_id":c["id"],"entity_id":n["entity"]["id"]}),
            0
        )["data"]["values"][p["id"].as_str().unwrap()],
        "48.57"
    );
    cli(
        &path,
        "database",
        "query",
        &json!({"collection_id":c["id"],"config":{"columns":["ajena"]}}),
        2,
    );
}
#[test]
fn formula_set_and_derived_records_match_desktop_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(&path, "workspace", "create", &json!({}), 0);
    let c = cli(
        &path,
        "collection",
        "create",
        &json!({"name":"Calculations"}),
        0,
    )["data"]["collection"]
        .clone();
    let b=cli(&path,"formula","set",&json!({"collection_id":c["id"],"expected_revision":1,"name":"Importe","expression":{"op":"decimal","value":"145.69","currency":"BOB"}}),0)["data"].clone();
    let p = b["properties"][1]["id"].clone();
    let r = cli(
        &path,
        "record",
        "create",
        &json!({"collection_id":c["id"],"title":"Original"}),
        0,
    )["data"]
        .clone();
    assert_eq!(r["values"][p.as_str().unwrap()], "145.69");
    assert_eq!(r["entity"]["revision"], 1);
    let input = json!({"collection_id":c["id"],"entity_id":r["entity"]["id"]});
    let from_cli = cli(&path, "record", "get", &input, 0)["data"].clone();
    assert_eq!(
        noteka_core::execute_json(
            json!({"workspace":path,"command":{"operation":"record.get","input":input}}),
            "desktop"
        )
        .data
        .unwrap(),
        from_cli
    );
    cli(
        &path,
        "formula",
        "set",
        &json!({"collection_id":c["id"],"expected_revision":1,"property_id":p,"name":"Importe","expression":{"op":"decimal","value":"1","currency":"BOB"}}),
        4,
    );
    cli(
        &path,
        "formula",
        "set",
        &json!({"collection_id":c["id"],"expected_revision":2,"name":"No código","expression":{"op":"sql","value":"DELETE FROM entities"}}),
        2,
    );
}

#[test]
fn advanced_view_projections_filters_and_conflicts_match_desktop() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    cli(
        &path,
        "workspace",
        "create",
        &json!({"timezone":"Pacific/Honolulu"}),
        0,
    );
    let c = cli(&path, "collection", "create", &json!({"name":"Views"}), 0)["data"]["collection"]
        .clone();
    let flag=cli(&path,"property","define",&json!({"collection_id":c["id"],"expected_revision":1,"spec":{"name":"Flag","type":"checkbox"}}),0)["data"]["id"].clone();
    let date=cli(&path,"property","define",&json!({"collection_id":c["id"],"expected_revision":2,"spec":{"name":"Date","type":"date"}}),0)["data"]["id"].clone();
    let record=cli(&path,"record","create",&json!({"collection_id":c["id"],"title":"Original","values":{flag.as_str().unwrap():false,date.as_str().unwrap():"2024-02-29"}}),0)["data"].clone();
    for (layout, config) in [
        (
            "board",
            json!({"group_by":flag,"filter_tree":{"op":"not","child":{"op":"condition","filter":{"property_id":flag,"op":"eq","value":true}}}}),
        ),
        (
            "calendar",
            json!({"calendar":{"property_id":date,"month":"2024-02"}}),
        ),
        ("gallery", json!({})),
    ] {
        let view = cli(
            &path,
            "view",
            "save",
            &json!({"collection_id":c["id"],"name":layout,"layout":layout,"config":config}),
            0,
        )["data"]
            .clone();
        let input = json!({"collection_id":c["id"],"view_id":view["id"]});
        let page = cli(&path, "database", "query", &input, 0)["data"].clone();
        assert_eq!(
            noteka_core::execute_json(
                json!({"workspace":path,"command":{"operation":"database.query","input":input}}),
                "desktop"
            )
            .data
            .unwrap(),
            page
        );
        assert_eq!(page["records"]["items"][0], record);
        assert_eq!(page["layout"], layout);
        assert_eq!(page["matching_count"], 1);
        if layout == "calendar" {
            assert_eq!(
                page["groups"][28]["entity_ids"],
                json!([record["entity"]["id"]])
            );
        }
        cli(
            &path,
            "view",
            "save",
            &json!({"collection_id":c["id"],"id":view["id"],"expected_revision":1,"name":"Updated","layout":layout,"config":config}),
            0,
        );
        cli(
            &path,
            "view",
            "save",
            &json!({"collection_id":c["id"],"id":view["id"],"expected_revision":1,"name":"Old","layout":layout,"config":config}),
            4,
        );
    }
    cli(
        &path,
        "database",
        "query",
        &json!({"collection_id":c["id"],"layout":"calendar","config":{"calendar":{"property_id":flag,"month":"2024-02"}}}),
        2,
    );
}
