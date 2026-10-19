use noteka_core::{execute_json, workspace::Workspace};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::fs;
struct Fixture {
    temp: tempfile::TempDir,
    path: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        Workspace::create(&path, Some("UTC".into())).unwrap();
        Self { temp, path }
    }
    fn call(&self, op: &str, input: Value) -> noteka_core::contracts::Response {
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
    fn db(&self) -> Connection {
        Connection::open(self.path.join("noteka.sqlite")).unwrap()
    }
    fn rich(&self) -> Value {
        self.ok("entity.create",json!({"kind":"note","title":"Acuerdo CRE","document":serde_json::from_str::<Value>(include_str!("../../../tests/fixtures/rich-document.json")).unwrap()}))
    }
    fn search(&self, q: &str) -> Value {
        self.ok("search.query", json!({"query":q}))
    }
}
#[test]
fn nested_provenance_filters_pagination_and_literal_inputs() {
    let f = Fixture::new();
    let n = f.rich();
    let project = f.ok("entity.create", json!({"kind":"project","title":"P"}));
    f.ok("relation.add",json!({"source_id":n["id"],"target_id":project["id"],"kind":"project","expected_revision":1}));
    let hit = &f.search("Cita")["items"][0];
    assert_eq!(hit["entity_id"], n["id"]);
    assert_eq!(hit["block_id"], "qp");
    assert_eq!(hit["revision"], 2);
    assert!(hit["excerpt"].as_str().unwrap().contains("Cita"));
    assert_eq!(
        f.ok(
            "search.query",
            json!({"query":"Cita","related_id":project["id"]})
        )["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        f.ok("search.query", json!({"query":"Cita","kind":"task"}))["items"],
        json!([])
    );
    assert_eq!(f.search("titulo")["items"][0]["block_id"], "h");
    let mut ids = vec![];
    for i in 0..5 {
        ids.push(
            f.ok(
                "entity.create",
                json!({"kind":"task","title":format!("Tokencomun {i}")}),
            )["id"]
                .clone(),
        );
    }
    let mut cursor = Value::Null;
    let mut seen = vec![];
    loop {
        let p = f.ok(
            "search.query",
            json!({"query":"Tokencomun","limit":2,"cursor":cursor}),
        );
        seen.extend(
            p["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e["entity_id"].clone()),
        );
        cursor = p["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }
    ids.sort_by_key(Value::to_string);
    seen.sort_by_key(Value::to_string);
    assert_eq!(seen, ids);
    for q in [
        "SELECT * FROM entities; DROP TABLE entities; --",
        "Cita OR secreto",
        "\"",
        "title:Acuerdo",
        "Cita*",
        "NEAR(Cita)",
    ] {
        assert!(f.call("search.query", json!({"query":q})).ok, "{q}");
    }
    assert_eq!(
        f.ok("entity.get", json!({"id":n["id"]}))["title"],
        "Acuerdo CRE"
    );
    for input in [
        json!({"query":" "}),
        json!({"query":"x".repeat(257)}),
        json!({"query":"x ".repeat(17)}),
        json!({"query":"Cita","limit":0}),
        json!({"query":"Cita","limit":101}),
        json!({"query":"Cita","cursor":"no"}),
        json!({"query":"Cita","cursor":"100001"}),
        json!({"query":"Cita","related_id":n["id"]}),
    ] {
        assert_eq!(
            serde_json::to_value(f.call("search.query", input).error.unwrap().code).unwrap(),
            "validation_error"
        );
    }
}
#[test]
fn updates_archiving_stale_revisions_rebuild_and_atomic_index_failure() {
    let f = Fixture::new();
    let n = f.rich();
    let d = json!({"schema_version":2,"blocks":[{"id":"stable","type":"paragraph","content":[{"type":"text","text":"Reemplazo vigente"}]}]});
    f.ok(
        "entity.update",
        json!({"id":n["id"],"expected_revision":1,"document":d}),
    );
    assert_eq!(f.search("Cita")["items"], json!([]));
    assert_eq!(f.search("Reemplazo")["items"][0]["revision"], 2);
    let db = f.db();
    db.execute(
        "UPDATE search_fts SET revision=1 WHERE entity_id=?1",
        [n["id"].as_str().unwrap()],
    )
    .unwrap();
    assert_eq!(f.search("Reemplazo")["items"], json!([]));
    f.ok("search.rebuild", json!({}));
    assert_eq!(f.search("Reemplazo")["items"][0]["revision"], 2);
    db.execute_batch("CREATE TRIGGER fail_index BEFORE INSERT ON search_fts_content BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let r = f.call(
        "entity.update",
        json!({"id":n["id"],"expected_revision":2,"title":"No confirmar"}),
    );
    assert!(!r.ok);
    assert!(!f.call("search.rebuild", json!({})).ok);
    db.execute_batch("DROP TRIGGER fail_index").unwrap();
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["revision"], 2);
    assert_eq!(f.search("Reemplazo")["items"][0]["revision"], 2);
    f.ok(
        "entity.archive",
        json!({"id":n["id"],"expected_revision":2}),
    );
    assert_eq!(f.search("Reemplazo")["items"], json!([]));
    assert_eq!(
        f.ok("search.query", json!({"query":"Reemplazo","archived":true}))["items"][0]["revision"],
        3
    );
    f.ok(
        "entity.restore",
        json!({"id":n["id"],"expected_revision":3}),
    );
    assert_eq!(f.search("Reemplazo")["items"][0]["entity_id"], n["id"]);
}
#[test]
fn rebuild_unknown_and_migration_backup_preserve_originals() {
    let f = Fixture::new();
    let n = f.rich();
    let unknown = json!({"schema_version":99,"blocks":[{"id":"future","type":"unknown","text":"Noindexable"}],"retain":{"x":42}});
    let db = f.db();
    db.execute(
        "UPDATE documents SET json=?1 WHERE entity_id=?2",
        rusqlite::params![unknown.to_string(), n["id"].as_str().unwrap()],
    )
    .unwrap();
    let before = f.ok("entity.get", json!({"id":n["id"]}));
    f.ok("search.rebuild", json!({}));
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]})), before);
    assert_eq!(f.search("Noindexable")["items"], json!([]));
    assert_eq!(f.search("Acuerdo")["items"][0]["block_id"], Value::Null);
    db.execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations")
        .unwrap();
    db.execute_batch(
        "DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;DROP TABLE templates; DROP TABLE calculations; DROP TABLE views; DROP TABLE property_values; DROP TABLE memberships; DROP TABLE properties; DROP TABLE collections; DROP TABLE search_fts;UPDATE workspace_meta SET schema_version=2;PRAGMA user_version=2;",
    )
    .unwrap();
    drop(db);
    let original = f.ok("entity.get", json!({"id":n["id"]}));
    assert_eq!(original, before);
    assert_eq!(
        f.ok("workspace.open", json!({}))["schema_version"],
        noteka_core::workspace::SCHEMA_VERSION
    );
    assert_eq!(f.search("Acuerdo")["items"][0]["revision"], 1);
    let backups = fs::read_dir(f.temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("noteka-pre-migration-")
        })
        .collect::<Vec<_>>();
    assert!(!backups.is_empty());
    let restored = f.temp.path().join("restored");
    Workspace::restore(&backups[0], &restored).unwrap();
    let restored = execute_json(
        json!({"workspace":restored,"command":{"operation":"entity.get","input":{"id":n["id"]}}}),
        "test",
    );
    assert_eq!(restored.data.unwrap()["document"], unknown);
}
