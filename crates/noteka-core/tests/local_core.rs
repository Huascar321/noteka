use noteka_core::{
    contracts::*,
    error::ErrorCode,
    execute_json,
    workspace::{Workspace, default_path},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

struct Fixture {
    _temp: tempfile::TempDir,
    path: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("workspace");
        Workspace::create(&path, Some("America/La_Paz".into())).unwrap();
        Self { _temp: temp, path }
    }
    fn call(&self, op: &str, input: Value) -> Response {
        execute_json(
            json!({"workspace":self.path,"command":{"operation":op,"input":input}}),
            "test",
        )
    }
    fn ok(&self, op: &str, input: Value) -> Value {
        let r = self.call(op, input);
        assert!(r.ok, "{:?}", r.error);
        r.data.unwrap()
    }
    fn note(&self) -> Value {
        self.ok(
            "entity.create",
            json!({"kind":"note","title":"Nota","document":doc()}),
        )
    }
    fn db(&self) -> Connection {
        let c = Connection::open(self.path.join("noteka.sqlite")).unwrap();
        c.busy_timeout(Duration::from_millis(2000)).unwrap();
        c.pragma_update(None, "foreign_keys", true).unwrap();
        c
    }
}
fn doc() -> Value {
    json!({"schema_version":1,"blocks":[{"id":"a","type":"paragraph","content":[{"type":"text","text":"Uno"}]},{"id":"b","type":"paragraph","content":[{"type":"text","text":"Dos"}]}]})
}
fn code(response: Response) -> String {
    assert!(!response.ok);
    serde_json::to_value(response.error.unwrap().code)
        .unwrap()
        .as_str()
        .unwrap()
        .into()
}

#[test]
fn workspace_selection_default_and_restart_are_local() {
    assert!(
        !default_path()
            .unwrap()
            .starts_with(std::env::current_dir().unwrap())
    );
    let f = Fixture::new();
    let g = Fixture::new();
    let n = f.note();
    let project = f.ok("entity.create", json!({"kind":"project","title":"CRE"}));
    let relation=f.ok("relation.add",json!({"source_id":n["id"],"target_id":project["id"],"kind":"project","expected_revision":1}));
    let reopened = Workspace::open(&f.path).unwrap();
    assert_eq!(reopened.info().unwrap().timezone, "America/La_Paz");
    drop(reopened);
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["revision"], 2);
    assert_eq!(
        f.ok("relation.list", json!({"entity_id":project["id"]}))["items"][0]["id"],
        relation["relation"]["id"]
    );
    assert_eq!(g.ok("entity.list", json!({}))["items"], json!([]));
    assert_eq!(
        code(g.call("entity.get", json!({"id":n["id"]}))),
        "not_found"
    );
    assert_eq!(
        code(execute_json(
            json!({"workspace":f.path.join("absent"),"command":{"operation":"workspace.open","input":{}}}),
            "test"
        )),
        "not_found"
    );
}
#[test]
fn all_types_defaults_and_invalid_payloads() {
    let f = Fixture::new();
    for (kind, payload) in [
        ("note", json!({})),
        ("task", json!({})),
        ("project", json!({})),
        ("tag", json!({"classification":"area"})),
        ("person", json!({})),
    ] {
        let e = f.ok(
            "entity.create",
            json!({"kind":kind,"title":" Original ","payload":payload}),
        );
        assert_eq!(e["title"], "Original");
        assert_eq!(e["revision"], 1);
        assert!(e["created_at"].as_str().unwrap().ends_with('Z'));
        if kind == "task" {
            assert_eq!(e["payload"]["state"], "todo");
        }
        if kind == "project" {
            assert_eq!(e["payload"]["state"], "planned");
        }
        let updated = f.ok(
            "entity.update",
            json!({"id":e["id"],"expected_revision":1,"title":"Nuevo"}),
        );
        assert_eq!(updated["id"], e["id"]);
        assert_eq!(updated["revision"], 2);
    }
    for input in [
        json!({"kind":"alien","title":"x"}),
        json!({"kind":"note","title":" \n\t "}),
        json!({"kind":"task","title":"x","payload":{"state":"planned"}}),
        json!({"kind":"tag","title":"x","payload":{"classification":"folder"}}),
        json!({"kind":"tag","title":"x"}),
        json!({"kind":"person","title":"x","payload":{"secret":"x"}}),
        json!({"kind":"note","title":"x","unknown":1}),
    ] {
        assert_eq!(code(f.call("entity.create", input)), "validation_error");
    }
    let task = f.ok("entity.create", json!({"kind":"task","title":"Hacer"}));
    f.ok(
        "entity.update",
        json!({"id":task["id"],"expected_revision":1,"payload":{"state":"done"}}),
    );
    assert_eq!(
        f.ok("entity.list", json!({"kind":"task","state":"done"}))["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        code(f.call(
            "entity.update",
            json!({"id":task["id"],"expected_revision":2,"payload":{"state":"invalid"}})
        )),
        "validation_error"
    );
    assert_eq!(
        f.ok("entity.get", json!({"id":task["id"]}))["payload"]["state"],
        "done"
    );
}
#[test]
fn tags_filters_and_stable_pagination() {
    let f = Fixture::new();
    for classification in ["area", "resource", "topic"] {
        let tag=f.ok("entity.create",json!({"kind":"tag","title":classification,"payload":{"classification":classification}}));
        assert!(
            f.ok(
                "entity.list",
                json!({"kind":"tag","classification":classification})
            )["items"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["id"] == tag["id"])
        );
        f.ok(
            "entity.update",
            json!({"id":tag["id"],"expected_revision":1,"payload":{"classification":"topic"}}),
        );
    }
    for _ in 0..12 {
        f.note();
    }
    // Mismo timestamp: ejercitar explícitamente el desempate por ID.
    f.db()
        .execute(
            "UPDATE entities SET created_at='2026-01-01T00:00:00Z' WHERE kind='note'",
            [],
        )
        .unwrap();
    let mut cursor = Value::Null;
    let mut ids = vec![];
    loop {
        let p = f.ok(
            "entity.list",
            json!({"kind":"note","limit":3,"cursor":cursor}),
        );
        for e in p["items"].as_array().unwrap() {
            ids.push(e["id"].as_str().unwrap().to_owned());
        }
        cursor = p["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }
    assert_eq!(ids.len(), 12);
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids, sorted);
    for input in [
        json!({"limit":0}),
        json!({"limit":201}),
        json!({"cursor":"not-json"}),
        json!({"classification":"folder"}),
        json!({"state":"maybe"}),
    ] {
        assert_eq!(code(f.call("entity.list", input)), "validation_error");
    }
}
#[test]
fn semantic_roundtrip_limits_and_atomic_validation() {
    let f = Fixture::new();
    let n = f.note();
    assert_eq!(n["document"], doc());
    let mut invalid = doc();
    invalid["blocks"][1]["id"] = json!("a");
    for d in [
        invalid,
        json!({"schema_version":99,"blocks":[]}),
        json!({"schema_version":1,"blocks":[{"id":"x","type":"unknown","content":[]}]}),
        json!({"schema_version":1,"blocks":[{"id":"x","type":"paragraph","content":[{"type":"text","text":"a".repeat(1_048_576)}]}]}),
    ] {
        assert_eq!(
            code(f.call(
                "entity.update",
                json!({"id":n["id"],"expected_revision":1,"title":"No debe guardarse","document":d})
            )),
            "validation_error"
        );
        let current = f.ok("entity.get", json!({"id":n["id"]}));
        assert_eq!(current["title"], n["title"]);
        assert_eq!(current["revision"], 1);
        assert_eq!(current["document"], doc());
    }
    let blocks: Vec<_> = (0..10001)
        .map(|i| json!({"id":i.to_string(),"type":"paragraph","content":[]}))
        .collect();
    assert_eq!(code(f.call("entity.update",json!({"id":n["id"],"expected_revision":1,"document":{"schema_version":1,"blocks":blocks}}))),"validation_error");
    let mut deep = json!(0);
    for _ in 0..34 {
        deep = json!({"nested":deep});
    }
    assert_eq!(
        code(f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":1,"document":deep})
        )),
        "validation_error"
    );
    assert_eq!(
        f.ok("changes.list", json!({"entity_id":n["id"]}))["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut at_limit = doc();
    at_limit["blocks"][0]["content"][0]["text"] = json!("a".repeat(1_048_000));
    f.ok(
        "entity.update",
        json!({"id":n["id"],"expected_revision":1,"document":at_limit}),
    );
}
#[test]
fn unknown_documents_preserve_every_field_and_archive() {
    let f = Fixture::new();
    for original in [
        json!({"schema_version":99,"future":{"keep":true},"blocks":[]}),
        json!({"schema_version":1,"blocks":[{"id":"future","type":"widget","data":{"keep":[1,2,3]}}]}),
        json!({"schema_version":1,"blocks":[],"unknown_property":"preserve"}),
    ] {
        let n = f.note();
        f.db()
            .execute(
                "UPDATE documents SET json=?2 WHERE entity_id=?1",
                rusqlite::params![n["id"].as_str().unwrap(), original.to_string()],
            )
            .unwrap();
        let current = f.ok("entity.get", json!({"id":n["id"]}));
        assert_eq!(current["document_editable"], false);
        assert_eq!(current["document"], original);
        assert_eq!(
            code(f.call(
                "entity.update",
                json!({"id":n["id"],"expected_revision":1,"document":doc()})
            )),
            "unsupported_document_format"
        );
        f.ok(
            "entity.archive",
            json!({"id":n["id"],"expected_revision":1}),
        );
        assert_eq!(
            f.ok("entity.get", json!({"id":n["id"]}))["document"],
            original
        );
    }
}
#[test]
fn relations_integrity_duplicates_remove_archive_and_restore() {
    let f = Fixture::new();
    let other = Fixture::new();
    let n = f.note();
    let foreign = other.note();
    let project = f.ok("entity.create", json!({"kind":"project","title":"CRE"}));
    let person = f.ok("entity.create", json!({"kind":"person","title":"Persona"}));
    let tag = f.ok(
        "entity.create",
        json!({"kind":"tag","title":"Área","payload":{"classification":"area"}}),
    );
    for (target, kind, error) in [
        (&foreign, "project", "not_found"),
        (&n, "project", "validation_error"),
        (&person, "project", "validation_error"),
    ] {
        assert_eq!(code(f.call("relation.add",json!({"source_id":n["id"],"target_id":target["id"],"kind":kind,"expected_revision":1}))),error);
    }
    let mut rev = 1;
    for (target, kind) in [(&project, "project"), (&tag, "tag"), (&person, "person")] {
        let r=f.ok("relation.add",json!({"source_id":n["id"],"target_id":target["id"],"kind":kind,"expected_revision":rev}));
        rev += 1;
        let duplicate=f.ok("relation.add",json!({"source_id":n["id"],"target_id":target["id"],"kind":kind,"expected_revision":rev}));
        assert_eq!(duplicate["changed"], false);
        assert_eq!(duplicate["relation"]["id"], r["relation"]["id"]);
        assert_eq!(duplicate["entity"]["revision"], rev);
        assert_eq!(
            f.ok("relation.list", json!({"entity_id":target["id"]}))["items"][0]["id"],
            r["relation"]["id"]
        );
    }
    assert!(
        f.db()
            .execute(
                "INSERT INTO relations VALUES ('bad','absent','absent','tag')",
                []
            )
            .is_err()
    );
    f.ok(
        "entity.archive",
        json!({"id":project["id"],"expected_revision":1}),
    );
    assert!(f.ok("entity.get", json!({"id":n["id"]}))["archived_at"].is_null());
    f.ok(
        "entity.archive",
        json!({"id":n["id"],"expected_revision":rev}),
    );
    rev += 1;
    assert_eq!(
        f.ok("entity.list", json!({"kind":"note"}))["items"],
        json!([])
    );
    assert_eq!(
        f.ok("entity.list", json!({"kind":"note","archived":true}))["items"][0]["id"],
        n["id"]
    );
    f.ok(
        "entity.restore",
        json!({"id":n["id"],"expected_revision":rev}),
    );
    rev += 1;
    let relations = f.ok("relation.list", json!({"entity_id":n["id"]}));
    assert_eq!(relations["items"].as_array().unwrap().len(), 3);
    f.ok(
        "relation.remove",
        json!({"id":relations["items"][0]["id"],"expected_revision":rev}),
    );
    assert_eq!(
        f.ok("relation.list", json!({"entity_id":n["id"]}))["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}
#[test]
fn competing_writers_history_missing_revision_and_storage_rollback() {
    let f = Fixture::new();
    let n = f.note();
    let mut a = Workspace::open(&f.path).unwrap();
    let mut b = Workspace::open(&f.path).unwrap();
    let input = UpdateEntity {
        id: n["id"].as_str().unwrap().into(),
        expected_revision: 1,
        title: Some("Primero".into()),
        document: None,
        payload: None,
    };
    a.entity_update(input.clone(), "desktop", "request-a")
        .unwrap();
    assert!(matches!(
        b.entity_update(input, "cli", "request-b").unwrap_err().code,
        ErrorCode::RevisionConflict
    ));
    for op in ["entity.update", "entity.archive", "entity.restore"] {
        assert_eq!(code(f.call(op, json!({"id":n["id"]}))), "validation_error");
    }
    let c = f.db();
    c.execute_batch("CREATE TRIGGER fail_change BEFORE INSERT ON changes BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    assert_eq!(code(f.call("entity.update",json!({"id":n["id"],"expected_revision":2,"title":"No commit","document":{"schema_version":1,"blocks":[]}}))),"storage_error");
    let after = f.ok("entity.get", json!({"id":n["id"]}));
    assert_eq!(after["title"], "Primero");
    assert_eq!(after["revision"], 2);
    assert_eq!(after["document"], doc());
    let history = f.ok("changes.list", json!({"entity_id":n["id"]}));
    assert_eq!(history["items"].as_array().unwrap().len(), 2);
    assert_eq!(history["items"][1]["origin"], "desktop");
    assert_eq!(history["items"][1]["request_id"], "request-a");
}
#[test]
fn busy_writer_times_out_without_mutation() {
    let f = Fixture::new();
    let n = f.note();
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE").unwrap();
    let start = Instant::now();
    assert_eq!(
        code(f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":1,"title":"Locked"})
        )),
        "workspace_busy"
    );
    assert!(start.elapsed() >= Duration::from_millis(1800));
    assert!(start.elapsed() < Duration::from_secs(5));
    lock.execute_batch("ROLLBACK").unwrap();
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["revision"], 1);
}
#[test]
fn live_backup_restore_integrity_and_no_overwrite() {
    let f = Fixture::new();
    let n = f.note();
    let connection = f.db();
    connection.execute_batch("BEGIN IMMEDIATE").unwrap();
    connection
        .execute(
            "UPDATE entities SET title='Uncommitted' WHERE id=?1",
            [n["id"].as_str().unwrap()],
        )
        .unwrap();
    let backup = f._temp.path().join("backup");
    f.ok("workspace.backup", json!({"destination":backup}));
    connection.execute_batch("ROLLBACK").unwrap();
    let destination = f._temp.path().join("restored");
    let info = f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":destination}),
    );
    assert_eq!(info["id"], f.ok("workspace.open", json!({}))["id"]);
    let restored = Workspace::open(&destination)
        .unwrap()
        .entity_get(&IdInput {
            id: n["id"].as_str().unwrap().into(),
        })
        .unwrap();
    assert_eq!(restored.title, "Nota");
    assert_eq!(restored.revision, 1);
    fs::write(destination.join("sentinel"), "keep").unwrap();
    assert_eq!(
        code(f.call(
            "workspace.restore",
            json!({"backup":backup,"destination":destination})
        )),
        "destination_not_empty"
    );
    assert_eq!(
        fs::read_to_string(destination.join("sentinel")).unwrap(),
        "keep"
    );
    fs::write(backup.join("noteka.sqlite"), "corrupt").unwrap();
    let bad = f._temp.path().join("bad");
    assert_eq!(
        code(f.call(
            "workspace.restore",
            json!({"backup":backup,"destination":bad})
        )),
        "invalid_backup"
    );
    assert!(!bad.exists());
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["title"], "Nota");
}
fn tree(path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = vec![];
    for e in fs::read_dir(path).unwrap() {
        let p = e.unwrap().path();
        if p.is_file() {
            out.push((
                p.file_name().unwrap().to_string_lossy().into(),
                fs::read(p).unwrap(),
            ));
        }
    }
    out.sort();
    out
}
#[test]
fn older_workspace_migrates_future_main_and_wal_remain_untouched() {
    let f = Fixture::new();
    let n = f.note();
    let p = f.ok("entity.create", json!({"kind":"project","title":"Previo"}));
    let relation = f.ok(
        "relation.add",
        json!({"source_id":n["id"],"target_id":p["id"],"kind":"project","expected_revision":1}),
    );
    let c = f.db();
    c.execute_batch(
        "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations;DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;DROP TABLE templates; DROP TABLE calculations; DROP TABLE views; DROP TABLE property_values; DROP TABLE memberships; DROP TABLE properties; DROP TABLE collections; DROP TABLE changes; DROP TABLE attachments; DROP TABLE search_fts; UPDATE workspace_meta SET schema_version=0; PRAGMA user_version=0;",
    )
    .unwrap();
    drop(c);
    assert_eq!(
        Workspace::open(&f.path)
            .unwrap()
            .info()
            .unwrap()
            .schema_version,
        noteka_core::workspace::SCHEMA_VERSION
    );
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["document"], doc());
    assert!(fs::read_dir(f._temp.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("noteka-pre-migration-")
    }));
    assert_eq!(
        f.ok("relation.list", json!({"entity_id":p["id"]}))["items"][0]["id"],
        relation["relation"]["id"]
    );
    let c = f.db();
    c.execute_batch("UPDATE workspace_meta SET schema_version=99; PRAGMA user_version=99;")
        .unwrap();
    let before = tree(&f.path);
    assert_eq!(
        code(f.call("workspace.open", json!({}))),
        "unsupported_workspace_version"
    );
    assert_eq!(tree(&f.path), before);
    drop(c);
    let before = tree(&f.path);
    assert_eq!(
        code(f.call("workspace.open", json!({}))),
        "unsupported_workspace_version"
    );
    assert_eq!(tree(&f.path), before);
}
#[test]
fn contract_catalog_is_complete_and_strict() {
    let r = execute_json(
        json!({"command":{"operation":"operations.describe","input":{}}}),
        "cli",
    );
    assert!(r.ok);
    let catalog = r.data.unwrap();
    assert_eq!(catalog["inputs"].as_object().unwrap().len(), 53);
    assert_eq!(catalog["outputs"].as_object().unwrap().len(), 53);
    assert!(catalog["contract_schema"]["$defs"]["Document"].is_object());
    assert_eq!(
        code(execute_json(
            json!({"workspace":null,"extra":"unsupported","command":{"operation":"operations.describe","input":{}}}),
            "cli"
        )),
        "validation_error"
    );
}
