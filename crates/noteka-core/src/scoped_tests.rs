use crate::{contracts::*, scoped::*, workspace::Workspace};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

fn sample(schema: &Value, root: &Value) -> Value {
    if let Some(r) = schema["$ref"].as_str() {
        return sample(root.pointer(r.strip_prefix('#').unwrap()).unwrap(), root);
    }
    if let Some(v) = schema.get("const") {
        return v.clone();
    }
    if let Some(v) = schema["enum"].as_array() {
        return v[0].clone();
    }
    for key in ["oneOf", "anyOf"] {
        if let Some(a) = schema[key].as_array() {
            return sample(&a[0], root);
        }
    }
    let kind = schema["type"]
        .as_str()
        .or_else(|| schema["type"].as_array().and_then(|a| a[0].as_str()));
    match kind {
        Some("object") => {
            let mut v = json!({});
            if let Some(required) = schema["required"].as_array() {
                for key in required {
                    let key = key.as_str().unwrap();
                    v[key] = sample(&schema["properties"][key], root);
                }
            }
            v
        }
        Some("array") => json!([]),
        Some("string") => json!(""),
        Some("integer" | "number") => json!(0),
        Some("boolean") => json!(false),
        _ => Value::Null,
    }
}
fn run(s: &LocalScope, op: &str, input: Value) -> Response {
    s.execute_json(json!({"operation":op,"input":input}), "mcp")
}
fn data(r: Response) -> Value {
    assert!(r.ok, "{:?}", r.error);
    r.data.unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("workspace"), Some("UTC".into())).unwrap();
    (t, w)
}
fn raw(c: &rusqlite::Connection) -> BTreeMap<String, Vec<Vec<String>>> {
    let names = c
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();
    names
        .into_iter()
        .map(|name| {
            let mut stmt = c
                .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))
                .unwrap();
            let n = stmt.column_count();
            let mut rows = stmt
                .query_map([], |r| {
                    (0..n)
                        .map(|i| Ok(format!("{:?}", r.get_ref(i)?)))
                        .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
                })
                .unwrap()
                .collect::<std::result::Result<Vec<_>, _>>()
                .unwrap();
            rows.sort();
            (name, rows)
        })
        .collect()
}
fn files(p: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    for e in std::fs::read_dir(p).unwrap() {
        let e = e.unwrap();
        if e.path().is_file() {
            out.insert(
                e.file_name().to_string_lossy().into(),
                std::fs::read(e.path()).unwrap(),
            );
        }
    }
    out
}

#[test]
fn exhaustive_policy_matches_all_typed_operations_and_catalog_without_changing_it() {
    let cat = crate::describe();
    let mut names = Vec::new();
    for (name, schema) in cat["inputs"].as_object().unwrap() {
        let op: Operation =
            serde_json::from_value(json!({"operation":name,"input":sample(schema,schema)}))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            Some(operation_access(&op)),
            named_access(name, Policy::new(false, false).unwrap()),
            "{name}"
        );
        names.push(name.as_str());
    }
    assert_eq!(names.len(), 53);
    assert_eq!(READ.len(), 25);
    assert_eq!(WRITE.len(), 20);
    assert_eq!(EXCLUDED.len(), 7);
    for (w, r, n) in [(false, false, 25), (true, false, 45), (true, true, 46)] {
        let p = Policy::new(w, r).unwrap();
        assert_eq!(
            names
                .iter()
                .filter(|name| p.allows(named_access(name, p).unwrap()))
                .count(),
            n
        );
    }
    assert!(Policy::new(false, true).is_err());
    assert!(named_access("future.operation", Policy::new(true, true).unwrap()).is_none());
    assert_eq!(cat, crate::describe());
}

#[test]
fn permissions_precede_open_and_mixed_dtos_cannot_bypass_them() {
    let (t, w) = setup();
    let read = LocalScope::new(&w.path, Policy::new(false, false).unwrap()).unwrap();
    let write = LocalScope::new(&w.path, Policy::new(true, false).unwrap()).unwrap();
    let cat = crate::describe();
    let before = raw(&w.conn);
    std::fs::rename(&w.path, t.path().join("moved")).unwrap();
    for name in WRITE
        .iter()
        .chain(EXCLUDED)
        .chain(["extraction.run"].iter())
    {
        let schema = &cat["inputs"][name];
        let r = run(&read, name, sample(schema, schema));
        assert!(!r.ok, "{name}");
        assert_eq!(
            r.error.unwrap().details["reason"],
            "permission_denied",
            "{name}"
        );
    }
    for name in EXCLUDED.iter().chain(["extraction.run"].iter()) {
        let schema = &cat["inputs"][name];
        assert_eq!(
            run(&write, name, sample(schema, schema))
                .error
                .unwrap()
                .details["reason"],
            "permission_denied",
            "{name}"
        );
    }
    for (name, input) in [
        (
            "extraction.control",
            json!({"action":"set","paused":false,"expected_revision":1}),
        ),
        (
            "extraction.index",
            json!({"action":"reset","expected_revision":1}),
        ),
        (
            "extraction.index",
            json!({"action":"put","job_id":"x","expected_generation_id":"y","expected_revision":1}),
        ),
    ] {
        assert_eq!(
            run(&read, name, input).error.unwrap().details["reason"],
            "permission_denied"
        );
    }
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn scope_creation_is_physically_pure_and_versions_never_migrate() {
    let (t, w) = setup();
    let before = files(&w.path);
    let _s = LocalScope::new(&w.path, Policy::new(true, true).unwrap()).unwrap();
    assert_eq!(before, files(&w.path));
    for v in [
        crate::workspace::SCHEMA_VERSION - 1,
        crate::workspace::SCHEMA_VERSION + 1,
    ] {
        w.conn
            .execute("UPDATE workspace_meta SET schema_version=?1", [v])
            .unwrap();
        w.conn.pragma_update(None, "user_version", v).unwrap();
        let b = files(&w.path);
        assert!(matches!(
            LocalScope::new(&w.path, Policy::new(false, false).unwrap())
                .unwrap_err()
                .code,
            crate::error::ErrorCode::UnsupportedWorkspaceVersion
        ));
        assert_eq!(b, files(&w.path));
    }
    assert!(matches!(
        LocalScope::new(&t.path().join("absent"), Policy::new(false, false).unwrap())
            .unwrap_err()
            .code,
        crate::error::ErrorCode::NotFound
    ));
    assert_eq!(std::fs::read_dir(t.path()).unwrap().count(), 1);
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let invalid = t
            .path()
            .join(std::ffi::OsString::from_vec(b"invalid-\xff".to_vec()));
        let before = files(&w.path);
        assert!(matches!(
            LocalScope::new(&invalid, Policy::new(false, false).unwrap())
                .unwrap_err()
                .code,
            crate::error::ErrorCode::ValidationError
        ));
        assert_eq!(before, files(&w.path));
        assert!(!invalid.exists());
    }
}

#[test]
fn readonly_main_wal_and_unknown_content_are_preserved() {
    let (_t, mut w) = setup();
    let e = w
        .entity_create(
            serde_json::from_value(json!({"kind":"note","title":"WAL scoped"})).unwrap(),
            "cli",
            "base",
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE documents SET json=?1 WHERE entity_id=?2",
            ["{\"schema_version\":999,\"unknown\":42}", &e.id],
        )
        .unwrap();
    let s = LocalScope::new(&w.path, Policy::new(false, false).unwrap()).unwrap();
    let c = s.open(false).unwrap();
    assert!(c.conn.execute("DELETE FROM entities", []).is_err());
    drop(c);
    let before = raw(&w.conn);
    let got = data(run(&s, "entity.get", json!({"id":e.id})));
    assert_eq!(got["title"], "WAL scoped");
    assert_eq!(got["document"]["unknown"], 42);
    let cat = crate::describe();
    for name in READ {
        let schema = &cat["inputs"][name];
        let _ = run(&s, name, sample(schema, schema));
    }
    assert_eq!(before, raw(&w.conn));
    for input in [
        json!({"id":e.id,"workspace":"elsewhere"}),
        json!({"id":e.id,"origin":"cli"}),
        json!({"id":e.id,"request_id":"forged"}),
    ] {
        assert!(!run(&s, "entity.get", input).ok);
    }
    assert!(
        !s.execute_json(
            json!({"workspace":"elsewhere","operation":"entity.get","input":{"id":e.id}}),
            "mcp"
        )
        .ok
    );
    assert!(!run(&s, "entity.get", json!({"id":"a".repeat(2_097_153)})).ok);
    let mut deep = json!(0);
    for _ in 0..33 {
        deep = json!({"x":deep});
    }
    assert!(!run(&s, "entity.get", deep).ok);
}

#[test]
fn every_read_class_succeeds_on_readonly_main_with_committed_wal_and_preserves_originals() {
    let (t, mut w) = setup();
    let write = LocalScope::new(&w.path, Policy::new(true, true).unwrap()).unwrap();
    let collection = data(run(
        &write,
        "collection.create",
        json!({"name":"Synthetic reads"}),
    ))["collection"]
        .clone();
    let property = data(run(
        &write,
        "property.define",
        json!({"collection_id":collection["id"],"expected_revision":collection["revision"],"spec":{"name":"Self relation","type":"relation","target_collection_id":collection["id"]}}),
    ));
    let record = data(run(
        &write,
        "record.create",
        json!({"collection_id":collection["id"],"title":"Synthetic record"}),
    ))["entity"]
        .clone();
    let revision = data(run(
        &write,
        "collection.get",
        json!({"id":collection["id"]}),
    ))["collection"]["revision"]
        .clone();
    let template = data(run(
        &write,
        "template.save",
        json!({"collection_id":collection["id"],"expected_collection_revision":revision,"name":"Synthetic template","definition":{"schema_version":1,"title":"Synthetic","document":{"schema_version":2,"blocks":[]},"values":{}}}),
    ));
    let mut jobs = Vec::new();
    for (name, bytes, extractor) in [
        ("text.txt", b"Synthetic readonly text".to_vec(), "utf8_text"),
        (
            "file.pdf",
            crate::pdf_tests::source(&[b"Synthetic readonly PDF".to_vec()]),
            "pdf_text",
        ),
        ("file.xlsx", crate::xlsx_tests::source(2), "xlsx_cells"),
    ] {
        let source = t.path().join(name);
        std::fs::write(&source, &bytes).unwrap();
        let a = w
            .attachment_import(ImportAttachment {
                source_path: source.to_string_lossy().into(),
            })
            .unwrap();
        let j = data(run(
            &write,
            "extraction.enqueue",
            json!({"attachment_id":a.id,"extractor":extractor}),
        ))["job"]
            .clone();
        let control = data(run(&write, "extraction.control", json!({"action":"get"})));
        data(run(
            &write,
            "extraction.control",
            json!({"action":"set","paused":false,"expected_revision":control["revision"]}),
        ));
        let done = data(run(&write, "extraction.run", json!({"id":j["id"]})))["job"].clone();
        assert_eq!(done["state"], "succeeded");
        jobs.push(done);
        assert_eq!(std::fs::read(source).unwrap(), bytes);
    }
    for job in jobs.iter().filter(|j| j["extractor"] != "xlsx_cells") {
        let index = data(run(&write, "extraction.index", json!({"action":"get"})));
        data(run(
            &write,
            "extraction.index",
            json!({"action":"put","job_id":job["id"],"expected_generation_id":job["generation"]["id"],"expected_revision":index["revision"]}),
        ));
    }
    assert!(
        w.path
            .join(format!("{}-wal", crate::workspace::DB_NAME))
            .metadata()
            .unwrap()
            .len()
            > 32
    );
    let ocr = crate::ocr_tests::seed(&mut w, crate::ocr_tests::recognition(2));
    let reads = [
        ("changes.list", json!({})),
        (
            "changes.preview",
            json!({"version":1,"batch_id":"ca2f333a-1b56-4e43-af7a-ef9148c472d1","steps":[{"key":"one","operation":"collection.create","input":{"name":"simulation only"}}]}),
        ),
        ("collection.get", json!({"id":collection["id"]})),
        ("collection.list", json!({})),
        ("database.query", json!({"collection_id":collection["id"]})),
        ("entity.get", json!({"id":record["id"]})),
        ("entity.list", json!({})),
        ("extraction.control", json!({"action":"get"})),
        ("extraction.fragments", json!({"id":jobs[0]["id"]})),
        ("extraction.get", json!({"id":jobs[0]["id"]})),
        ("extraction.index", json!({"action":"get"})),
        ("extraction.list", json!({})),
        ("extraction.ocr_regions", json!({"id":ocr.id})),
        ("extraction.pdf_fragments", json!({"id":jobs[1]["id"]})),
        ("extraction.search", json!({"query":"Synthetic"})),
        ("extraction.xlsx_cells", json!({"id":jobs[2]["id"]})),
        (
            "home.query",
            json!({"view":"notes_all","as_of":"2026-10-15"}),
        ),
        ("operations.describe", json!({})),
        (
            "record.get",
            json!({"collection_id":collection["id"],"entity_id":record["id"]}),
        ),
        (
            "record.inverse",
            json!({"property_id":property["id"],"target_id":record["id"]}),
        ),
        ("relation.list", json!({"entity_id":record["id"]})),
        ("search.query", json!({"query":"Synthetic"})),
        ("template.get", json!({"id":template["id"]})),
        ("template.list", json!({"collection_id":collection["id"]})),
        ("workflow.query", json!({"view":"inbox"})),
    ];
    assert_eq!(reads.iter().map(|(op, _)| *op).collect::<Vec<_>>(), READ);
    let reader = LocalScope::new(&w.path, Policy::new(false, false).unwrap()).unwrap();
    let before = raw(&w.conn);
    let originals = files(&w.path.join("attachments"));
    for (op, input) in reads {
        let response = run(&reader, op, input);
        assert!(response.ok, "{op}: {:?}", response.error);
        assert_eq!(before, raw(&w.conn), "{op}");
    }
    assert_eq!(originals, files(&w.path.join("attachments")));
}

#[test]
fn replacement_is_rejected_by_generic_and_specialized_connections() {
    let (t, w) = setup();
    let s = LocalScope::new(&w.path, Policy::new(true, true).unwrap()).unwrap();
    let other = Workspace::create(&t.path().join("other"), Some("UTC".into())).unwrap();
    let before = raw(&other.conn);
    std::fs::rename(&w.path, t.path().join("original")).unwrap();
    std::fs::rename(&other.path, &w.path).unwrap();
    for (name, input) in [
        ("entity.list", json!({})),
        ("entity.create", json!({"kind":"note","title":"no"})),
        ("extraction.index", json!({"action":"get"})),
        (
            "extraction.index",
            json!({"action":"reset","expected_revision":1}),
        ),
        ("extraction.search", json!({"query":"x"})),
        (
            "changes.preview",
            json!({"version":1,"batch_id":"ca2f333a-1b56-4e43-af7a-ef9148c472d1","steps":[{"key":"one","operation":"collection.create","input":{"name":"no"}}]}),
        ),
    ] {
        let r = run(&s, name, input);
        assert!(!r.ok, "{name}");
        assert_eq!(
            r.error.unwrap().details["reason"],
            "workspace_scope_mismatch",
            "{name}"
        );
    }
    assert_eq!(before, raw(&other.conn));
}

#[test]
fn valid_mutation_keeps_core_cas_origin_and_generated_request_ids() {
    let (_t, w) = setup();
    let s = LocalScope::new(&w.path, Policy::new(true, false).unwrap()).unwrap();
    let first = run(&s, "entity.create", json!({"kind":"note","title":"mcp"}));
    let req = first.request_id.clone();
    let e = data(first);
    let second = run(
        &s,
        "entity.update",
        json!({"id":e["id"],"expected_revision":1,"title":"updated"}),
    );
    assert_ne!(req, second.request_id);
    assert_eq!(data(second)["revision"], 2);
    assert!(matches!(
        run(
            &s,
            "entity.update",
            json!({"id":e["id"],"expected_revision":1,"title":"old"})
        )
        .error
        .unwrap()
        .code,
        crate::error::ErrorCode::RevisionConflict
    ));
    let hist = data(run(&s, "changes.list", json!({"entity_id":e["id"]})));
    assert_eq!(hist["items"].as_array().unwrap().len(), 2);
    assert!(
        hist["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["origin"] == "mcp")
    );
    s.stop();
    assert!(!run(&s, "entity.list", json!({})).ok);
}

#[test]
fn replacement_after_private_preflight_is_rejected_on_actual_connection_and_snapshot() {
    let (t, w) = setup();
    let s = LocalScope::new(&w.path, Policy::new(true, true).unwrap()).unwrap();
    let other = Workspace::create(&t.path().join("other"), Some("UTC".into())).unwrap();
    let before = raw(&other.conn);
    let mut hook = || {
        std::fs::rename(&w.path, t.path().join("old")).unwrap();
        std::fs::rename(&other.path, &w.path).unwrap();
    };
    let error = match s.open_with_hook(true, &mut hook) {
        Ok(_) => panic!("foreign workspace admitted"),
        Err(e) => e,
    };
    assert_eq!(error.details["reason"], "workspace_scope_mismatch");
    assert_eq!(before, raw(&other.conn));
    let (source, _) = crate::batch_storage::open(&w.path, false).unwrap();
    let snapshot = crate::batch_storage::snapshot(&source).unwrap();
    assert_eq!(
        s.check(&source).unwrap_err().details["reason"],
        "workspace_scope_mismatch"
    );
    assert_eq!(
        s.check(&snapshot).unwrap_err().details["reason"],
        "workspace_scope_mismatch"
    );
}

#[test]
fn expired_claim_future_jobs_and_index_damage_are_not_recovered_by_scoped_reads() {
    let (t, mut w) = setup();
    let path = t.path().join("source.txt");
    std::fs::write(&path, b"scoped synthetic source").unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: path.to_string_lossy().into(),
        })
        .unwrap();
    let job = w
        .extraction_enqueue(
            serde_json::from_value(json!({"attachment_id":a.id,"extractor":"utf8_text"})).unwrap(),
        )
        .unwrap()
        .job;
    let c = w
        .extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    w.extraction_control(ExtractionControlInput::Set {
        paused: false,
        expected_revision: c.revision,
    })
    .unwrap();
    let reader = LocalScope::new(&w.path, Policy::new(false, false).unwrap()).unwrap();
    let control = rusqlite::Connection::open(w.path.join(crate::workspace::DB_NAME)).unwrap();
    let mut hook = |stage: &str| {
        if stage == "claimed" {
            control
                .execute(
                    "UPDATE extraction_runtime SET lease_until='2000-01-01T00:00:00Z'",
                    [],
                )
                .unwrap();
            let before = raw(&control);
            let got = data(run(&reader, "extraction.get", json!({"id":job.id})));
            assert_eq!(got["state"], "running");
            assert_eq!(got["recoverable"], true);
            for (name, input) in [
                ("extraction.list", json!({})),
                ("extraction.control", json!({"action":"get"})),
                ("extraction.index", json!({"action":"get"})),
                ("extraction.search", json!({"query":"synthetic"})),
                (
                    "changes.preview",
                    json!({"version":1,"batch_id":"ca2f333a-1b56-4e43-af7a-ef9148c472d1","steps":[{"key":"one","operation":"collection.create","input":{"name":"simulation"}}]}),
                ),
            ] {
                assert!(run(&reader, name, input).ok, "{name}");
            }
            assert_eq!(raw(&control), before);
        }
    };
    assert!(
        w.extraction_run_impl(
            RunExtraction {
                id: Some(job.id.clone())
            },
            Some(&mut hook),
            None
        )
        .is_err()
    );
    let config = json!({"version":999,"future":{"literal":"preserve"}});
    control
        .execute(
            "UPDATE extraction_jobs SET config=?2,config_hash=?3 WHERE id=?1",
            rusqlite::params![
                job.id,
                serde_json::to_string(&config).unwrap(),
                crate::extraction_storage::config_hash(&config).unwrap()
            ],
        )
        .unwrap();
    let before = raw(&control);
    let got = data(run(&reader, "extraction.get", json!({"id":job.id})));
    assert_eq!(got["supported"], false);
    assert_eq!(got["config"], config);
    assert!(run(&reader, "extraction.index", json!({"action":"get"})).ok);
    assert_eq!(raw(&control), before);
    control
        .execute(
            "UPDATE extraction_search_control SET profile='future_profile'",
            [],
        )
        .unwrap();
    let before = raw(&control);
    assert!(!run(&reader, "extraction.index", json!({"action":"get"})).ok);
    assert!(!run(&reader, "extraction.search", json!({"query":"synthetic"})).ok);
    assert_eq!(before, raw(&control));
    assert_eq!(std::fs::read(path).unwrap(), b"scoped synthetic source");
}
