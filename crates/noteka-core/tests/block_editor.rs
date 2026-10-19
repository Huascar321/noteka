use noteka_core::{contracts::Response, execute_json, workspace::Workspace};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{fs, path::Path};

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
    fn db(&self) -> Connection {
        Connection::open(self.path.join("noteka.sqlite")).unwrap()
    }
    fn import(&self, name: &str, bytes: &[u8]) -> Value {
        let p = self.temp.path().join(name);
        fs::write(&p, bytes).unwrap();
        self.ok("attachment.import", json!({"source_path":p}))
    }
}
fn rich() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/rich-document.json")).unwrap()
}
fn error(r: Response, code: &str) {
    assert!(!r.ok);
    assert_eq!(
        serde_json::to_value(r.error.unwrap().code).unwrap(),
        json!(code)
    );
}

#[test]
fn rich_roundtrip_ids_conflicts_and_validation_are_atomic() {
    let f = Fixture::new();
    let d = rich();
    let n = f.ok(
        "entity.create",
        json!({"kind":"note","title":"Rica","document":d}),
    );
    assert_eq!(n["document"], d);
    assert_eq!(n["document_editable"], true);
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["document"], d);
    let mut candidates = vec![];
    for (pointer, value) in [
        ("/blocks/0/id", json!("h")),
        ("/blocks/1/attrs", json!({"level":2,"future":1})),
        ("/blocks/1/attrs/level", json!(4)),
        (
            "/blocks/0/content/2/marks/0/href",
            json!("javascript:alert(1)"),
        ),
        ("/blocks/2/children/0/type", json!("paragraph")),
        ("/blocks/4/children/0/attrs/checked", json!("true")),
        (
            "/blocks/7/content",
            json!([{"type":"text","text":"code","marks":[{"type":"bold"}]}]),
        ),
        (
            "/blocks/0/content/0/marks",
            json!([{"type":"code"},{"type":"bold"}]),
        ),
    ] {
        let mut x = d.clone();
        *x.pointer_mut(pointer)
            .unwrap_or_else(|| panic!("{pointer}")) = value;
        candidates.push(x);
    }
    let mut unknown = d.clone();
    unknown["blocks"][0]["future"] = json!({"keep":true});
    candidates.push(unknown);
    for x in candidates {
        error(
            f.call(
                "entity.update",
                json!({"id":n["id"],"expected_revision":1,"title":"No guardar","document":x}),
            ),
            "validation_error",
        );
        assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["document"], d);
    }
    f.ok(
        "entity.update",
        json!({"id":n["id"],"expected_revision":1,"title":"CLI"}),
    );
    error(
        f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":1,"document":d}),
        ),
        "revision_conflict",
    );
    assert_eq!(
        f.ok("changes.list", json!({"entity_id":n["id"]}))["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn attachments_copy_references_failed_import_and_retention() {
    let f = Fixture::new();
    let a = f.import("source.txt", b"original");
    fs::write(f.temp.path().join("source.txt"), b"changed").unwrap();
    assert_eq!(
        f.ok("attachment.read", json!({"id":a["id"]}))["base64"],
        "b3JpZ2luYWw="
    );
    let d = json!({"schema_version":2,"blocks":[{"id":"file","type":"file","content":[],"attrs":{"attachment_id":a["id"],"name":"source.txt"}}]});
    let n = f.ok(
        "entity.create",
        json!({"kind":"note","title":"Adjunto","document":d}),
    );
    let mut invalid = d.clone();
    invalid["blocks"][0]["attrs"]["attachment_id"] = json!(uuid::Uuid::new_v4().to_string());
    error(
        f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":1,"document":invalid}),
        ),
        "not_found",
    );
    let mut image = d.clone();
    image["blocks"][0]["type"] = json!("image");
    image["blocks"][0]["attrs"] = json!({"attachment_id":a["id"],"alt":"texto"});
    error(
        f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":1,"document":image}),
        ),
        "validation_error",
    );
    f.ok(
        "entity.update",
        json!({"id":n["id"],"expected_revision":1,"document":{"schema_version":2,"blocks":[]}}),
    );
    assert!(f.call("attachment.read", json!({"id":a["id"]})).ok);
    f.db().execute_batch("CREATE TRIGGER fail_import BEFORE INSERT ON attachments BEGIN SELECT RAISE(ABORT,'injected'); END;").unwrap();
    let before = fs::read_dir(f.path.join("attachments")).unwrap().count();
    error(
        f.call(
            "attachment.import",
            json!({"source_path":f.temp.path().join("source.txt")}),
        ),
        "storage_error",
    );
    assert_eq!(
        fs::read_dir(f.path.join("attachments")).unwrap().count(),
        before
    );
    error(
        f.call(
            "attachment.import",
            json!({"source_path":f.temp.path().join("absent")}),
        ),
        "storage_error",
    );
    let big = f.temp.path().join("big");
    fs::File::create(&big)
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    error(
        f.call("attachment.import", json!({"source_path":big})),
        "validation_error",
    );
    fs::write(
        f.path.join("attachments").join(a["id"].as_str().unwrap()),
        b"altered",
    )
    .unwrap();
    error(
        f.call("attachment.read", json!({"id":a["id"]})),
        "storage_error",
    );
    error(
        f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":2,"document":d}),
        ),
        "storage_error",
    );
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["revision"], 2);
}

#[test]
fn attachments_backups_restore_exports_and_missing_files() {
    let f = Fixture::new();
    let a = f.import("image.png", b"\x89PNG\r\n\x1a\nsynthetic");
    let b = f.import("file.txt", b"payload");
    let mut d = rich();
    d["blocks"].as_array_mut().unwrap().extend([
        json!({"id":"image","type":"image","content":[],"attrs":{"attachment_id":a["id"],"alt":"Fixture"}}),
        json!({"id":"file","type":"file","content":[],"attrs":{"attachment_id":b["id"],"name":"file.txt"}})]);
    let n = f.ok(
        "entity.create",
        json!({"kind":"note","title":"Export","document":d}),
    );
    let project = f.ok(
        "entity.create",
        json!({"kind":"project","title":"Reference"}),
    );
    f.ok("relation.add",json!({"source_id":n["id"],"target_id":project["id"],"kind":"project","expected_revision":1}));
    let export = f.temp.path().join("export");
    f.ok(
        "document.export",
        json!({"id":n["id"],"destination":export,"format":"json"}),
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(export.join("document.json")).unwrap()).unwrap(),
        d
    );
    assert_eq!(
        fs::read(export.join("attachments").join(b["id"].as_str().unwrap())).unwrap(),
        b"payload"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(export.join("relations.json")).unwrap())
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    error(
        f.call(
            "document.export",
            json!({"id":n["id"],"destination":export,"format":"json"}),
        ),
        "destination_not_empty",
    );
    let md = f.temp.path().join("markdown");
    let result = f.ok(
        "document.export",
        json!({"id":n["id"],"destination":md,"format":"markdown"}),
    );
    assert!(!result["warnings"].as_array().unwrap().is_empty());
    let text = fs::read_to_string(md.join("document.md")).unwrap();
    assert!(text.contains("## Título"));
    assert!(text.contains("- [x] Hecho"));
    assert!(text.contains("````rust"));
    let backup = f.temp.path().join("backup");
    let lock = f.db();
    lock.execute_batch("BEGIN IMMEDIATE;UPDATE entities SET title='Uncommitted';")
        .unwrap();
    f.ok("workspace.backup", json!({"destination":backup}));
    lock.execute_batch("ROLLBACK").unwrap();
    let restored = f.temp.path().join("restored");
    f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":restored}),
    );
    let w = Workspace::open(&restored).unwrap();
    assert_eq!(
        serde_json::to_value(
            w.entity_get(&noteka_core::contracts::IdInput {
                id: n["id"].as_str().unwrap().into()
            })
            .unwrap()
        )
        .unwrap()["document"],
        d
    );
    assert_eq!(
        w.attachment_read(noteka_core::contracts::IdInput {
            id: b["id"].as_str().unwrap().into()
        })
        .unwrap()
        .base64,
        "cGF5bG9hZA=="
    );
    let managed = backup.join("attachments").join(a["id"].as_str().unwrap());
    fs::write(&managed, b"corrupt").unwrap();
    let altered = f.temp.path().join("altered");
    error(
        f.call(
            "workspace.restore",
            json!({"backup":backup,"destination":altered}),
        ),
        "invalid_backup",
    );
    assert!(!altered.exists());
    fs::remove_file(managed).unwrap();
    let bad = f.temp.path().join("bad");
    error(
        f.call(
            "workspace.restore",
            json!({"backup":backup,"destination":bad}),
        ),
        "invalid_backup",
    );
    assert!(!bad.exists());
    assert_eq!(f.ok("entity.get", json!({"id":n["id"]}))["revision"], 2);
}

#[test]
fn migration_v1_and_legacy_backup_preserve_document() {
    let f = Fixture::new();
    let n=f.ok("entity.create",json!({"kind":"note","title":"Legacy","document":{"schema_version":1,"blocks":[{"id":"old","type":"paragraph","content":[{"type":"text","text":"texto"}]}]}}));
    f.db().execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;DROP TABLE templates; DROP TABLE calculations; DROP TABLE views; DROP TABLE property_values; DROP TABLE memberships; DROP TABLE properties; DROP TABLE collections; DROP TABLE attachments;DROP TABLE search_fts;UPDATE workspace_meta SET schema_version=1;PRAGMA user_version=1;").unwrap();
    f.db()
        .execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations")
        .unwrap();
    let backup = f.temp.path().join("legacy");
    let copy = f.temp.path().join("copy");
    // Construir el manifiesto legacy desde la fixture SQL, sin migrar primero.
    fs::create_dir(&backup).unwrap();
    fs::copy(f.path.join("noteka.sqlite"), backup.join("noteka.sqlite")).unwrap();
    use sha2::{Digest, Sha256};
    let hash = format!(
        "{:x}",
        Sha256::digest(fs::read(backup.join("noteka.sqlite")).unwrap())
    );
    let id: String = f
        .db()
        .query_row("SELECT id FROM workspace_meta", [], |r| r.get(0))
        .unwrap();
    fs::write(
        backup.join("manifest.json"),
        json!({"format_version":1,"workspace_id":id,"schema_version":1,"sha256":hash}).to_string(),
    )
    .unwrap();
    Workspace::restore(&backup, &copy).unwrap();
    assert_eq!(
        Workspace::open(&copy)
            .unwrap()
            .info()
            .unwrap()
            .schema_version,
        noteka_core::workspace::SCHEMA_VERSION
    );
    let current = f.ok("entity.get", json!({"id":n["id"]}));
    assert_eq!(current["document"], n["document"]);
    assert_eq!(
        f.ok("workspace.open", json!({}))["schema_version"],
        noteka_core::workspace::SCHEMA_VERSION
    );
}

#[test]
fn unknown_v2_properties_are_recoverable_not_replaced() {
    let f = Fixture::new();
    let mut d = rich();
    d["blocks"][0]["future"] = json!({"keep":[1,2]});
    let n = f.ok("entity.create", json!({"kind":"note","title":"Unknown"}));
    f.db()
        .execute(
            "UPDATE documents SET schema_version=2,json=?1 WHERE entity_id=?2",
            rusqlite::params![d.to_string(), n["id"].as_str().unwrap()],
        )
        .unwrap();
    assert_eq!(
        f.ok("entity.get", json!({"id":n["id"]}))["document_editable"],
        false
    );
    error(
        f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":1,"document":rich()}),
        ),
        "unsupported_document_format",
    );
    let dest = f.temp.path().join("unknown");
    f.ok(
        "document.export",
        json!({"id":n["id"],"destination":dest,"format":"json"}),
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(dest.join("document.json")).unwrap()).unwrap(),
        d
    );
}

#[cfg(unix)]
#[test]
fn symlink_original_is_not_read_or_restored() {
    let f = Fixture::new();
    let a = f.import("file.txt", b"payload");
    let original = f.path.join("attachments").join(a["id"].as_str().unwrap());
    fs::remove_file(&original).unwrap();
    std::os::unix::fs::symlink(f.temp.path().join("file.txt"), &original).unwrap();
    error(
        f.call("attachment.read", json!({"id":a["id"]})),
        "storage_error",
    );
    error(
        f.call(
            "workspace.backup",
            json!({"destination":f.temp.path().join("backup")}),
        ),
        "storage_error",
    );
    assert!(!Path::new(&f.temp.path().join("backup")).exists());
}
