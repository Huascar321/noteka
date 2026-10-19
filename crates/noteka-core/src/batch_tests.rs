use crate::{
    batch_plan, batch_storage as store, batches,
    contracts::*,
    error::*,
    mutations::MutationContext,
    workspace::{DB_NAME, Workspace},
};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
fn v(target: &str, step: &str, export: &str) -> Value {
    json!({"kind":"value","target":target,"source":{"step":step,"export":export}})
}
fn k(target: &str, step: &str) -> Value {
    json!({"kind":"property_key","target":target,"source":{"step":step,"export":"property_id"}})
}
fn step(key: &str, op: &str, input: Value, bindings: Vec<Value>) -> Value {
    json!({"key":key,"operation":op,"input":input,"bindings":bindings})
}
fn plan(steps: Vec<Value>, checks: Vec<Value>) -> BatchPlan {
    serde_json::from_value(json!({"version":1,"batch_id":"00000000-0000-0000-0000-000000000001","steps":steps,"checks":checks})).unwrap()
}
fn simple() -> BatchPlan {
    plan(
        vec![step(
            "note",
            "entity.create",
            json!({"kind":"note","title":"created_at literal"}),
            vec![],
        )],
        vec![],
    )
}
fn full() -> BatchPlan {
    plan(
        vec![
            step(
                "collection",
                "collection.create",
                json!({"name":"Datos"}),
                vec![],
            ),
            step(
                "weight",
                "property.define",
                json!({"collection_id":null,"expected_revision":null,"spec":{"name":"Peso","type":"number"}}),
                vec![
                    v("/collection_id", "collection", "collection_id"),
                    v("/expected_revision", "collection", "collection_revision"),
                ],
            ),
            step(
                "link",
                "property.define",
                json!({"collection_id":null,"expected_revision":null,"spec":{"name":"Relación","type":"relation","target_collection_id":null}}),
                vec![
                    v("/collection_id", "collection", "collection_id"),
                    v("/expected_revision", "weight", "collection_revision"),
                    v("/spec/target_collection_id", "collection", "collection_id"),
                ],
            ),
            step(
                "calculation",
                "formula.set",
                json!({"collection_id":null,"expected_revision":null,"name":"Doble","expression":{"op":"binary","operator":"mul","left":{"op":"property","property_id":null},"right":{"op":"number","value":2}}}),
                vec![
                    v("/collection_id", "collection", "collection_id"),
                    v("/expected_revision", "link", "collection_revision"),
                    v("/expression/left/property_id", "weight", "property_id"),
                ],
            ),
            step(
                "first",
                "record.create",
                json!({"collection_id":null,"title":"Uno","values":{"a/b~c":3}}),
                vec![
                    v("/collection_id", "collection", "collection_id"),
                    k("/values/a~1b~0c", "weight"),
                ],
            ),
            step(
                "second",
                "record.create",
                json!({"collection_id":null,"title":"Dos","values":{"peso":1,"relación":[null]}}),
                vec![
                    v("/collection_id", "collection", "collection_id"),
                    v("/values/relación/0", "first", "entity_id"),
                    k("/values/relación", "link"),
                    k("/values/peso", "weight"),
                ],
            ),
            step(
                "page",
                "entity.create",
                json!({"kind":"note","title":"Página original"}),
                vec![],
            ),
            step(
                "attach",
                "record.attach",
                json!({"entity_id":null,"collection_id":null,"expected_revision":null,"values":{"peso":1}}),
                vec![
                    v("/entity_id", "page", "entity_id"),
                    v("/collection_id", "collection", "collection_id"),
                    v("/expected_revision", "page", "entity_revision"),
                    k("/values/peso", "weight"),
                ],
            ),
            step(
                "update",
                "record.update",
                json!({"entity_id":null,"collection_id":null,"expected_revision":null,"values":{"peso":2}}),
                vec![
                    v("/entity_id", "page", "entity_id"),
                    v("/collection_id", "collection", "collection_id"),
                    v("/expected_revision", "attach", "entity_revision"),
                    k("/values/peso", "weight"),
                ],
            ),
            step(
                "view",
                "view.save",
                json!({"collection_id":null,"name":"Vista","layout":"table","config":{"columns":[null,null],"sorts":[{"property_id":null,"descending":true}]}}),
                vec![
                    v("/collection_id", "collection", "collection_id"),
                    v("/config/columns/0", "collection", "title_property_id"),
                    v("/config/columns/1", "calculation", "property_id"),
                    v("/config/sorts/0/property_id", "weight", "property_id"),
                ],
            ),
            step(
                "embedded",
                "entity.update",
                json!({"id":null,"expected_revision":null,"document":{"schema_version":3,"blocks":[{"id":"original-block","type":"collection_view","content":[],"attrs":{"collection_id":null,"view_id":null}}]}}),
                vec![
                    v("/id", "page", "entity_id"),
                    v("/expected_revision", "update", "entity_revision"),
                    v(
                        "/document/blocks/0/attrs/collection_id",
                        "collection",
                        "collection_id",
                    ),
                    v("/document/blocks/0/attrs/view_id", "view", "view_id"),
                ],
            ),
        ],
        vec![
            json!({"key":"all","input":{"collection_id":null,"view_id":null,"limit":100},"bindings":[v("/collection_id","collection","collection_id"),v("/view_id","view","view_id")]}),
        ],
    )
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("workspace"), Some("UTC".into())).unwrap();
    (t, w)
}
fn hash(c: &Connection) -> String {
    let b = store::Budget::new(c).unwrap();
    store::base_hash(c, &b).unwrap()
}
fn reason(e: CoreError) -> String {
    e.details["reason"].as_str().unwrap_or("").into()
}

#[test]
fn preview_apply_all_nine_operations_preserve_ids_diff_and_history() {
    let (_t, w) = setup();
    let before = hash(&w.conn);
    let p = full();
    let a = batches::preview(&w.path, p.clone(), "cli", "simulated").unwrap();
    let b = batches::preview(&w.path, p.clone(), "desktop", "other").unwrap();
    assert_eq!(a.receipt.effect_hash, b.receipt.effect_hash);
    assert_eq!(hash(&w.conn), before);
    assert!(!a.applied);
    assert_eq!(a.steps.len(), 11);
    assert_eq!(a.checks[0].data.matching_count, 3);
    assert_eq!(a.checks[0].data.records.items.len(), 3);
    assert_eq!(a.history_summary.len(), 6);
    let applied = batches::apply(
        &w.path,
        ApplyBatch {
            plan: p.clone(),
            expected: a.receipt.clone(),
        },
        "desktop",
        "real-request",
    )
    .unwrap();
    assert!(applied.applied);
    assert_eq!(
        json!(a.steps.iter().map(|s| &s.exports).collect::<Vec<_>>()),
        json!(applied.steps.iter().map(|s| &s.exports).collect::<Vec<_>>())
    );
    assert_eq!(applied.steps[6].data["id"], applied.steps[10].data["id"]);
    assert_eq!(
        applied.steps[10].data["document"]["blocks"][0]["id"],
        "original-block"
    );
    assert_eq!(
        w.conn
            .query_row(
                "SELECT count(*) FROM changes WHERE request_id='real-request' AND origin='desktop'",
                [],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
        6
    );
    for kind in [
        "collection",
        "property",
        "calculation",
        "entity",
        "document",
        "membership",
        "value",
        "view",
    ] {
        assert!(applied.diff.iter().any(|d| d.kind == kind), "{kind}");
    }
    assert_ne!(hash(&w.conn), before);
    assert_eq!(
        reason(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan: p,
                    expected: a.receipt
                },
                "cli",
                "repeat"
            )
            .unwrap_err()
        ),
        "base_hash_mismatch"
    );
}
#[test]
fn invalid_step_and_check_leave_all_original_tables_and_fts_unchanged() {
    let (_t, w) = setup();
    let before = hash(&w.conn);
    let mut p = full();
    p.steps[5].input["values"]["peso"] = json!("invalid-number");
    let e = batches::preview(&w.path, p, "cli", "bad").unwrap_err();
    assert_eq!(e.details["step_key"], "second");
    assert_eq!(e.details["phase"], "step");
    assert_eq!(hash(&w.conn), before);
    let mut p = full();
    p.checks[0].input["limit"] = json!(1);
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "bad").unwrap_err()),
        "incomplete_check"
    );
    assert_eq!(hash(&w.conn), before);
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM search_fts", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        0
    );
}
#[test]
fn receipts_bind_plan_scope_original_state_and_effect_before_commit() {
    let (t, mut w) = setup();
    let p = simple();
    let a = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let before = hash(&w.conn);
    let mut expected = a.receipt.clone();
    expected.effect_hash = "a".repeat(64);
    assert_eq!(
        reason(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan: p.clone(),
                    expected
                },
                "cli",
                "bad"
            )
            .unwrap_err()
        ),
        "effect_hash_mismatch"
    );
    assert_eq!(hash(&w.conn), before);
    let mut modified = p.clone();
    modified.steps[0].input["title"] = json!("Different");
    assert_eq!(
        reason(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan: modified,
                    expected: a.receipt.clone()
                },
                "cli",
                "bad"
            )
            .unwrap_err()
        ),
        "plan_hash_mismatch"
    );
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let restored = t.path().join("restored");
    Workspace::restore(&backup, &restored).unwrap();
    assert_eq!(
        reason(
            batches::apply(
                &restored,
                ApplyBatch {
                    plan: p.clone(),
                    expected: a.receipt.clone()
                },
                "cli",
                "bad"
            )
            .unwrap_err()
        ),
        "receipt_scope_mismatch"
    );
    w.entity_create(
        serde_json::from_value(json!({"kind":"note","title":"Escritora ajena"})).unwrap(),
        "cli",
        "writer",
    )
    .unwrap();
    assert_eq!(
        reason(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan: p,
                    expected: a.receipt
                },
                "cli",
                "bad"
            )
            .unwrap_err()
        ),
        "base_hash_mismatch"
    );
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
}
#[test]
fn plan_limits_strict_shapes_and_previous_exports() {
    for count in [512, 513] {
        let p = plan(
            vec![
                step(
                    "note",
                    "entity.create",
                    json!({"kind":"note","title":"N"}),
                    vec![],
                ),
                step(
                    "record",
                    "record.create",
                    json!({"collection_id":"literal","title":"R","values":{"relation":vec![Value::Null;count]}}),
                    (0..count)
                        .map(|i| v(&format!("/values/relation/{i}"), "note", "entity_id"))
                        .collect(),
                ),
            ],
            vec![],
        );
        assert_eq!(batch_plan::validate(&p).is_ok(), count <= 512);
    }
    for n in [1, 100, 101] {
        let steps = (0..n)
            .map(|i| {
                step(
                    &format!("n{i}"),
                    "entity.create",
                    json!({"kind":"note","title":"N"}),
                    vec![],
                )
            })
            .collect();
        assert_eq!(batch_plan::validate(&plan(steps, vec![])).is_ok(), n <= 100);
    }
    for n in [0, 10, 11] {
        let mut p = simple();
        p.checks = (0..n)
            .map(|i| {
                serde_json::from_value(
                    json!({"key":format!("q{i}"),"input":{"collection_id":"literal"}}),
                )
                .unwrap()
            })
            .collect();
        assert_eq!(batch_plan::validate(&p).is_ok(), n <= 10);
    }
    let mut p = simple();
    p.version = 2;
    assert_eq!(
        reason(batch_plan::validate(&p).unwrap_err()),
        "unsupported_plan_version"
    );
    let mut p = full();
    p.steps[1].bindings[0].source.step = "future".into();
    assert_eq!(
        reason(batch_plan::validate(&p).unwrap_err()),
        "non_previous_reference"
    );
    let mut p = full();
    p.steps[1].bindings[0].source.export = BatchExport::EntityId;
    assert_eq!(
        reason(batch_plan::validate(&p).unwrap_err()),
        "unavailable_export"
    );
    for key in ["", "0key", "with space", "ñ", &"x".repeat(65)] {
        let mut p = simple();
        p.steps[0].key = key.into();
        assert!(batch_plan::validate(&p).is_err());
    }
    let mut p = full();
    p.steps[1].key = p.steps[0].key.clone();
    assert_eq!(
        reason(batch_plan::validate(&p).unwrap_err()),
        "invalid_or_duplicate_key"
    );
    for op in [
        "changes.preview",
        "changes.apply",
        "workspace.create",
        "attachment.import",
        "database.query",
        "template.instantiate",
        "relation.add",
    ] {
        let mut p = json!(simple());
        p["steps"][0]["operation"] = json!(op);
        assert!(serde_json::from_value::<BatchPlan>(p).is_err());
    }
    for field in ["sql", "operation", "nested"] {
        let mut p = json!(simple());
        p[field] = json!(true);
        assert!(serde_json::from_value::<BatchPlan>(p).is_err());
    }
    let mut p = simple();
    p.steps[0].input["unknown"] = json!(true);
    assert!(batch_plan::validate(&p).is_err());
    for pointer in ["", "x", "/a~2b", "/a~"] {
        assert!(batch_plan::pointer(pointer).is_err());
    }
    assert_eq!(batch_plan::pointer("/a~1b~0c").unwrap(), vec!["a/b~c"]);
    let mut p = simple();
    p.steps[0].input["payload"] = json!({"huge":"x".repeat(2_097_152)});
    assert!(batch_plan::validate(&p).is_err());
    let mut deep = json!(null);
    for _ in 0..34 {
        deep = json!({"nested":deep});
    }
    let mut p = simple();
    p.steps[0].input["payload"] = deep;
    assert!(batch_plan::validate(&p).is_err());
}
#[test]
fn bindings_reject_text_currency_unknown_attrs_wrong_type_and_collisions() {
    let (_t, w) = setup();
    for target in ["/title", "/payload/created_at"] {
        let mut p = simple();
        p.steps.push(
            serde_json::from_value(step(
                "other",
                "entity.create",
                json!({"kind":"note","title":"Literal","payload":{"created_at":"literal"}}),
                vec![v(target, "note", "entity_id")],
            ))
            .unwrap(),
        );
        assert_eq!(
            reason(batches::preview(&w.path, p, "cli", "bad").unwrap_err()),
            "forbidden_binding_slot"
        );
    }
    let mut p = full();
    p.steps[1].bindings[0].source.export = BatchExport::TitlePropertyId;
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "bad").unwrap_err()),
        "export_type_mismatch"
    );
    let mut p = full();
    let preview = batches::preview(&w.path, p.clone(), "cli", "ok").unwrap();
    let id = preview.steps[1].exports[&BatchExport::PropertyId].clone();
    let ExportValue::Id(id) = id else { panic!() };
    p.steps[4].input["values"][id] = json!(4);
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "bad").unwrap_err()),
        "property_key_collision"
    );
    let mut p = full();
    p.steps[5].bindings[1].source.export = BatchExport::CollectionId;
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "bad").unwrap_err()),
        "export_type_mismatch"
    );
}
#[test]
fn wal_snapshot_page_size_and_sql_extensions_fail_closed_without_migration() {
    let (_t, mut w) = setup();
    w.conn.execute_batch("PRAGMA wal_autocheckpoint=0").unwrap();
    w.entity_create(
        serde_json::from_value(json!({"kind":"note","title":"En WAL"})).unwrap(),
        "cli",
        "wal",
    )
    .unwrap();
    assert!(
        std::fs::metadata(w.path.join(format!("{DB_NAME}-wal")))
            .unwrap()
            .len()
            > 0
    );
    let snapshot = store::snapshot(&w.conn).unwrap();
    assert_eq!(hash(&snapshot), hash(&w.conn));
    w.conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE; PRAGMA page_size=8192; VACUUM;").unwrap();
    let snapshot = store::snapshot(&w.conn).unwrap();
    assert_eq!(
        snapshot
            .pragma_query_value(None, "page_size", |r| r.get::<_, u32>(0))
            .unwrap(),
        8192
    );
    assert_eq!(hash(&snapshot), hash(&w.conn));
    for sql in [
        "CREATE TABLE extension(id TEXT PRIMARY KEY,secret TEXT)",
        "CREATE TRIGGER extra AFTER INSERT ON entities BEGIN SELECT 1; END",
        "ALTER TABLE entities ADD COLUMN future TEXT",
    ] {
        let (_t, w) = setup();
        w.conn.execute_batch(sql).unwrap();
        let before = hash(&w.conn);
        assert_eq!(
            reason(batches::preview(&w.path, simple(), "cli", "bad").unwrap_err()),
            "unsupported_batch_schema"
        );
        assert_eq!(hash(&w.conn), before);
    }
    let (_t, w) = setup();
    w.conn
        .execute_batch("CREATE INDEX compatible ON entities(title)")
        .unwrap();
    assert!(batches::preview(&w.path, simple(), "cli", "ok").is_ok());
    for version in 0..=crate::workspace::SCHEMA_VERSION + 1 {
        if version == crate::workspace::SCHEMA_VERSION {
            continue;
        }
        let (_t, w) = setup();
        w.conn.execute_batch(&format!("UPDATE workspace_meta SET schema_version={version};PRAGMA user_version={version};PRAGMA wal_checkpoint(TRUNCATE);")).unwrap();
        let bytes = std::fs::read(w.path.join(DB_NAME)).unwrap();
        assert!(matches!(
            batches::preview(&w.path, simple(), "cli", "bad")
                .unwrap_err()
                .code,
            ErrorCode::UnsupportedWorkspaceVersion
        ));
        assert_eq!(std::fs::read(w.path.join(DB_NAME)).unwrap(), bytes);
    }
    let t = tempfile::tempdir().unwrap();
    assert!(batches::preview(&t.path().join("absent"), simple(), "cli", "bad").is_err());
    assert!(!t.path().join("absent").exists());
}
#[test]
fn base_hash_preserves_raw_unknown_json_and_ignores_only_fts_content() {
    let (_t, mut w) = setup();
    let e = w
        .entity_create(
            serde_json::from_value(json!({"kind":"note","title":"N"})).unwrap(),
            "cli",
            "create",
        )
        .unwrap();
    let h = hash(&w.conn);
    w.search_rebuild().unwrap();
    assert_eq!(hash(&w.conn), h);
    w.conn
        .execute(
            "UPDATE entities SET payload=?2 WHERE id=?1",
            params![e.id, "{ \"future\":true }"],
        )
        .unwrap();
    let raw = hash(&w.conn);
    assert_ne!(raw, h);
    w.conn
        .execute(
            "UPDATE entities SET payload=?2 WHERE id=?1",
            params![e.id, "{\"future\":true}"],
        )
        .unwrap();
    assert_ne!(hash(&w.conn), raw);
    let h = hash(&w.conn);
    w.conn
        .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
        .unwrap();
    assert_eq!(hash(&w.conn), h);
    let a = json!({"a":1,"b":[1,2]});
    assert_eq!(
        store::json_hash("vector", &a).unwrap(),
        "c6e20f0a1b0fde6a7ea9ce17a282504541867e13ceb21b1a7078c537b8e81748"
    );
    let b: Value = serde_json::from_str("{\"b\":[1,2],\"a\":1}").unwrap();
    assert_eq!(
        store::json_hash("vector", &a).unwrap(),
        store::json_hash("vector", &b).unwrap()
    );
    assert_ne!(
        store::json_hash("vector", &a).unwrap(),
        store::json_hash("vector", &json!({"a":1,"b":[2,1]})).unwrap()
    );
}
#[test]
fn budget_cleans_up_after_error_and_unwind_and_maps_only_interrupt() {
    let c = Connection::open_in_memory().unwrap();
    {
        let b = store::Budget::with_duration(&c, Duration::ZERO).unwrap();
        assert_eq!(reason(b.check().unwrap_err()), "execution_deadline");
        let error=c.query_row("WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<100000) SELECT sum(x) FROM n",[],|r|r.get::<_,i64>(0)).unwrap_err();
        assert!(matches!(b.map(error.into()).code, ErrorCode::WorkspaceBusy));
        assert!(matches!(
            b.map(c.execute("invalid SQL", []).unwrap_err().into()).code,
            ErrorCode::StorageError
        ));
    }
    assert_eq!(
        c.query_row("SELECT 1", [], |r| r.get::<_, i32>(0)).unwrap(),
        1
    );
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _b = store::Budget::with_duration(&c, Duration::ZERO).unwrap();
        panic!("controlled test");
    }));
    assert_eq!(
        c.query_row("SELECT 2", [], |r| r.get::<_, i32>(0)).unwrap(),
        2
    );
    let mut target = Connection::open_in_memory().unwrap();
    assert_eq!(
        reason(store::copy(&c, &mut target, Instant::now()).unwrap_err()),
        "backup_deadline"
    );
}
#[test]
fn independent_writer_lock_timeout_and_visibility() {
    let (_t, w) = setup();
    let p = simple();
    let preview = batches::preview(&w.path, p.clone(), "cli", "preview").unwrap();
    let other = Connection::open(w.path.join(DB_NAME)).unwrap();
    other.execute_batch("BEGIN IMMEDIATE").unwrap();
    let start = Instant::now();
    let e = batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        "cli",
        "apply",
    )
    .unwrap_err();
    assert!(matches!(e.code, ErrorCode::WorkspaceBusy));
    assert!(start.elapsed() >= Duration::from_millis(1900));
    assert!(start.elapsed() < Duration::from_secs(5));
    other.execute_batch("ROLLBACK").unwrap();
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        0
    );
}

#[test]
fn reserved_ids_roles_ordinals_and_collision_never_upsert() {
    let (_t, w) = setup();
    let p = full();
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let id = preview.steps[0].data["collection"]["id"].as_str().unwrap();
    w.conn
        .execute("INSERT INTO collections VALUES (?1,'Reserved',1)", [id])
        .unwrap();
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "p").unwrap_err()),
        "reserved_id_collision"
    );
    let mut c = MutationContext::batch("cli", "request", uuid::Uuid::nil());
    c.step("fixed");
    let a = c.id("entity");
    assert_eq!(a, "30a8fcdc-5ad0-8c9c-beb8-2402e368f9e4");
    let b = c.id("entity");
    assert_ne!(a, b);
    c.step("fixed");
    assert_eq!(a, c.id("entity"));
    assert_ne!(a, c.id("property"));
    assert_eq!(uuid::Uuid::parse_str(&a).unwrap().get_version_num(), 8);
}

#[test]
fn checks_default_to_complete_and_error_free_partial_is_explicit() {
    let (_t, w) = setup();
    let mut p = full();
    p.checks[0].input["limit"] = json!(1);
    p.checks[0].require_complete = false;
    assert_eq!(
        batches::preview(&w.path, p.clone(), "cli", "p")
            .unwrap()
            .checks[0]
            .data
            .records
            .items
            .len(),
        1
    );
    p.checks[0].require_complete = true;
    p.checks[0].input["limit"] = json!(100);
    p.checks[0].input["cursor"] = json!("1");
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "p").unwrap_err()),
        "incomplete_check"
    );
    let mut p = full();
    p.steps[3].input["expression"] = json!({"op":"binary","operator":"div","left":{"op":"property","property_id":null},"right":{"op":"number","value":0}});
    // La vista ordena Peso, no el cálculo: el error permanece como dato de las filas.
    assert_eq!(
        reason(batches::preview(&w.path, p.clone(), "cli", "p").unwrap_err()),
        "calculation_errors"
    );
    p.checks[0].require_no_calculation_errors = false;
    assert!(batches::preview(&w.path, p, "cli", "p").is_ok());
}

#[test]
fn future_documents_are_not_replaced_or_hidden_in_the_state_fingerprint() {
    let (_t, mut w) = setup();
    let e = w
        .entity_create(
            serde_json::from_value(json!({"kind":"note","title":"Futura"})).unwrap(),
            "cli",
            "create",
        )
        .unwrap();
    let future =
        "{\"schema_version\":99,\"blocks\":[{\"type\":\"future\",\"id\":\"f\",\"secret\":true}]}";
    w.conn
        .execute(
            "UPDATE documents SET schema_version=99,json=?2 WHERE entity_id=?1",
            params![e.id, future],
        )
        .unwrap();
    let p = plan(
        vec![step(
            "update",
            "entity.update",
            json!({"id":e.id,"expected_revision":1,"title":"Renombrada"}),
            vec![],
        )],
        vec![],
    );
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        "cli",
        "a",
    )
    .unwrap();
    assert_eq!(
        w.conn
            .query_row(
                "SELECT json FROM documents WHERE entity_id=?1",
                [e.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        future
    );
}

#[test]
fn snapshot_size_boundary_and_response_limit() {
    // PRAGMA page_count cuenta páginas libres también: el límite no depende de JSON.
    let (_t, w) = setup();
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("CREATE TABLE bytes(v BLOB);PRAGMA page_size=4096")
        .unwrap();
    c.pragma_update(None, "max_page_count", 16384).unwrap();
    c.execute(
        "INSERT INTO bytes VALUES (zeroblob(?1))",
        [64 * 1024 * 1024 - 100_000],
    )
    .unwrap();
    while c
        .execute("INSERT INTO bytes VALUES (zeroblob(3000))", [])
        .is_ok()
    {}
    assert_eq!(
        c.pragma_query_value(None, "page_count", |r| r.get::<_, u32>(0))
            .unwrap(),
        16384
    );
    assert_eq!(store::size(&c).unwrap(), 4096);
    c.pragma_update(None, "max_page_count", 16385).unwrap();
    c.execute("INSERT INTO bytes VALUES (zeroblob(3000))", [])
        .unwrap();
    assert_eq!(reason(store::size(&c).unwrap_err()), "snapshot_size_limit");
    let mut p = simple();
    p.steps[0].input["document"] = json!({"schema_version":1,"blocks":[{"id":"b","type":"paragraph","content":[{"type":"text","text":"x".repeat(600_000)}]}]});
    let mut second = p.steps[0].clone();
    second.key = "second".into();
    p.steps.push(second);
    assert!(batch_plan::validate(&p).is_ok());
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "p").unwrap_err()),
        "response_size_limit"
    );
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        0
    );
}

#[test]
fn before_commit_storage_failure_and_reader_snapshot_are_atomic() {
    let (_t, w) = setup();
    let p = full();
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let before = hash(&w.conn);
    let request = ApplyBatch {
        plan: p,
        expected: preview.receipt,
    };
    let reader = Connection::open(w.path.join(DB_NAME)).unwrap();
    reader.execute_batch("BEGIN").unwrap();
    assert_eq!(
        reader
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        0
    );
    let error = batches::apply_injected(&w.path, request.clone(), &|tx| {
        assert_eq!(
            tx.query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            3
        );
        assert_eq!(
            reader
                .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
                .unwrap(),
            0
        );
        tx.execute_batch("SELECT * FROM injected_missing_table")?;
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(error.code, ErrorCode::StorageError));
    assert_eq!(hash(&w.conn), before);
    batches::apply_injected(&w.path, request, &|_| Ok(())).unwrap();
    assert_eq!(
        reader
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        0
    );
    reader.execute_batch("COMMIT").unwrap();
    assert_eq!(
        reader
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        3
    );
}

#[test]
fn timestamp_metadata_normalization_never_hides_unknown_payload_or_civil_dates() {
    let (_t, mut w) = setup();
    let e = w
        .entity_create(
            serde_json::from_value(json!({"kind":"note","title":"N"})).unwrap(),
            "cli",
            "create",
        )
        .unwrap();
    w.conn.execute("UPDATE entities SET payload=?2 WHERE id=?1",params![e.id,r#"{"inbox":true,"created_at":"literal","updated_at":"literal","date":"2026-10-13"}"#]).unwrap();
    let p = plan(
        vec![step(
            "update",
            "entity.update",
            json!({"id":e.id,"expected_revision":1,"title":"Renamed"}),
            vec![],
        )],
        vec![],
    );
    let a = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let b = batches::preview(&w.path, p.clone(), "desktop", "p2").unwrap();
    assert_eq!(a.receipt.effect_hash, b.receipt.effect_hash);
    assert_eq!(
        a.diff[0].after.as_ref().unwrap()["payload"]["created_at"],
        "literal"
    );
    w.conn.execute("UPDATE entities SET payload=?2 WHERE id=?1",params![e.id,r#"{"inbox":true,"created_at":"changed","updated_at":"literal","date":"2026-10-13"}"#]).unwrap();
    let changed = batches::preview(&w.path, p.clone(), "cli", "p3").unwrap();
    assert_ne!(changed.receipt.effect_hash, a.receipt.effect_hash);
    assert_eq!(
        reason(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan: p,
                    expected: a.receipt
                },
                "cli",
                "apply"
            )
            .unwrap_err()
        ),
        "base_hash_mismatch"
    );
}

#[test]
fn pinned_backup_includes_one_complete_state_despite_independent_wal_writer() {
    let (_t, mut w) = setup();
    let first = w
        .entity_create(
            serde_json::from_value(json!({"kind":"note","title":"Before"})).unwrap(),
            "cli",
            "create",
        )
        .unwrap();
    let reader = Connection::open_with_flags(
        w.path.join(DB_NAME),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let tx = reader.unchecked_transaction().unwrap();
    store::current(&tx).unwrap();
    let before = hash(&tx);
    w.entity_update(
        serde_json::from_value(json!({"id":first.id,"expected_revision":1,"title":"After"}))
            .unwrap(),
        "cli",
        "writer",
    )
    .unwrap();
    let mut target = Connection::open_in_memory().unwrap();
    target
        .pragma_update(None, "page_size", store::size(&tx).unwrap() as i64)
        .unwrap();
    store::copy(&tx, &mut target, Instant::now() + Duration::from_secs(2)).unwrap();
    assert_eq!(hash(&target), before);
    assert_ne!(hash(&target), hash(&w.conn));
}

#[test]
fn checks_cover_51_and_101_rows_without_certifying_skipped_rows() {
    let (_t, mut w) = setup();
    let c = w
        .collection_create(CreateCollection {
            name: "Many".into(),
        })
        .unwrap();
    for i in 0..101 {
        w.record_create(
            serde_json::from_value(
                json!({"collection_id":c.collection.id,"title":format!("Row {i}")}),
            )
            .unwrap(),
            "cli",
            "seed",
        )
        .unwrap();
    }
    let mut p = simple();
    p.checks = vec![
        serde_json::from_value(
            json!({"key":"all","input":{"collection_id":c.collection.id,"limit":100}}),
        )
        .unwrap(),
    ];
    assert_eq!(
        reason(batches::preview(&w.path, p.clone(), "cli", "p").unwrap_err()),
        "incomplete_check"
    );
    p.checks[0].require_complete = false;
    let first = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    assert_eq!(first.checks[0].data.matching_count, 101);
    assert_eq!(first.checks[0].data.records.items.len(), 100);
    p.checks[0].input["cursor"] = json!(first.checks[0].data.records.next_cursor);
    p.checks[0].require_complete = true;
    assert_eq!(
        reason(batches::preview(&w.path, p.clone(), "cli", "p").unwrap_err()),
        "incomplete_check"
    );
    p.checks[0].input.as_object_mut().unwrap().remove("cursor");
    w.conn.execute("UPDATE entities SET archived_at='2026-10-13' WHERE id IN (SELECT entity_id FROM memberships WHERE collection_id=?1 LIMIT 50)",[c.collection.id]).unwrap();
    let result = batches::preview(&w.path, p, "cli", "p").unwrap();
    assert_eq!(result.checks[0].data.matching_count, 51);
    assert_eq!(result.checks[0].data.records.items.len(), 51);
    let mut p = full();
    p.steps[3].input["expression"]["operator"] = json!("div");
    p.steps[3].input["expression"]["right"]["value"] = json!(0);
    p.steps[9].bindings.last_mut().unwrap().source.step = "calculation".into();
    p.checks[0].require_no_calculation_errors = false;
    assert!(batches::preview(&w.path, p, "cli", "p").is_err()); // Sort por cálculo inválido sigue fallando.
}

#[test]
fn explicit_migration_of_v0_remains_batch_compatible() {
    let (_t, w) = setup();
    w.conn
        .execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations")
        .unwrap();
    w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
    w.conn.execute_batch("DROP TABLE templates;DROP TABLE calculations;DROP TABLE views;DROP TABLE property_values;DROP TABLE memberships;DROP TABLE properties;DROP TABLE collections;DROP TABLE changes;DROP TABLE attachments;DROP TABLE search_fts;UPDATE workspace_meta SET schema_version=0;PRAGMA user_version=0").unwrap();
    let root = w.path.clone();
    drop(w);
    let migrated = Workspace::open(&root).unwrap();
    assert!(batches::preview(&migrated.path, simple(), "cli", "p").is_ok());
}

#[test]
fn deadline_detected_before_commit_rolls_back_and_does_not_retry() {
    let (_t, w) = setup();
    let p = simple();
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let e = batches::apply_injected(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        &|_| {
            std::thread::sleep(Duration::from_millis(5010));
            Ok(())
        },
    )
    .unwrap_err();
    assert!(matches!(e.code, ErrorCode::WorkspaceBusy));
    assert_eq!(reason(e), "execution_deadline");
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        0
    );
}

#[test]
fn timestamp_properties_are_typed_but_coverage_changes_still_conflict() {
    let (_t, w) = setup();
    let mut p = full();
    p.steps.insert(4,serde_json::from_value(step("clock","property.define",json!({"collection_id":null,"expected_revision":null,"spec":{"name":"Created","type":"created_at"}}),vec![v("/collection_id","collection","collection_id"),v("/expected_revision","calculation","collection_revision")])).unwrap());
    let a = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    std::thread::sleep(Duration::from_millis(5));
    let b = batches::preview(&w.path, p.clone(), "cli", "p2").unwrap();
    assert_eq!(a.receipt.effect_hash, b.receipt.effect_hash);
    batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: a.receipt,
        },
        "cli",
        "apply",
    )
    .unwrap();
}

#[test]
fn batch_recovery_preserves_versions_unknown_formats_fts_and_attachment_originals() {
    let (t, mut w) = setup();
    let complete = full();
    let preview = batches::preview(&w.path, complete.clone(), "cli", "p").unwrap();
    let applied = batches::apply(
        &w.path,
        ApplyBatch {
            plan: complete,
            expected: preview.receipt,
        },
        "cli",
        "a",
    )
    .unwrap();
    let collection = applied.steps[0].data["collection"]["id"].as_str().unwrap();
    w.conn
        .execute(
            "INSERT INTO properties VALUES ('future-property',?1,'Future',?2)",
            params![
                collection,
                r#"{"name":"Future","type":"future","unrecognized":true}"#
            ],
        )
        .unwrap();
    w.conn
        .execute(
            "INSERT INTO calculations VALUES ('future-property',?1)",
            [r#"{ "op":"future","unknown":[1,2] }"#],
        )
        .unwrap();
    w.conn
        .execute(
            "INSERT INTO templates VALUES ('future-template',?1,'Future',1,?2)",
            params![collection, r#"{ "version":99,"unknown":true }"#],
        )
        .unwrap();
    let source = t.path().join("synthetic.txt");
    std::fs::write(&source, b"original attachment\n").unwrap();
    let attachment = w
        .attachment_import(ImportAttachment {
            source_path: source.to_str().unwrap().into(),
        })
        .unwrap();
    let mut ids = vec![];
    for version in [1, 2, 3, 99] {
        let entity = w
            .entity_create(
                serde_json::from_value(json!({"kind":"note","title":format!("Version {version}")}))
                    .unwrap(),
                "cli",
                "seed",
            )
            .unwrap();
        let raw = if version == 99 {
            r#"{ "schema_version":99,"blocks":[{"id":"future","type":"future","unknown":true}] }"#
                .to_string()
        } else {
            serde_json::to_string(&json!({"schema_version":version,"blocks":[]})).unwrap()
        };
        w.conn
            .execute(
                "UPDATE documents SET schema_version=?2,json=?3 WHERE entity_id=?1",
                params![entity.id, version, raw],
            )
            .unwrap();
        ids.push(entity.id);
    }
    let untouched = |conn: &Connection| {
        let sql = "SELECT json FROM documents UNION ALL SELECT spec FROM properties UNION ALL SELECT expression FROM calculations UNION ALL SELECT definition FROM templates ORDER BY 1";
        conn.prepare(sql)
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap()
    };
    let originals = untouched(&w.conn);
    let p = plan(
        vec![step(
            "rename",
            "entity.update",
            json!({"id":ids[3],"expected_revision":1,"title":"Future preserved"}),
            vec![],
        )],
        vec![],
    );
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    assert_eq!(untouched(&w.conn), originals);
    batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        "cli",
        "a",
    )
    .unwrap();
    assert_eq!(untouched(&w.conn), originals);
    crate::attachments::verify(&w.path, &attachment).unwrap();
    let before = hash(&w.conn);
    let fts: u32 = w
        .conn
        .query_row("SELECT count(*) FROM search_fts", [], |r| r.get(0))
        .unwrap();
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let root = w.path.clone();
    drop(w);
    let reopened = Workspace::open(&root).unwrap();
    assert_eq!(hash(&reopened.conn), before);
    let restored = t.path().join("restored");
    Workspace::restore(&backup, &restored).unwrap();
    let restored = Workspace::open(&restored).unwrap();
    assert_eq!(hash(&restored.conn), before);
    assert_eq!(untouched(&restored.conn), originals);
    crate::attachments::verify(&restored.path, &attachment).unwrap();
    assert_eq!(
        restored
            .conn
            .query_row("SELECT count(*) FROM search_fts", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        fts
    );
    let p = plan(
        vec![step(
            "replace",
            "entity.update",
            json!({"id":ids[3],"expected_revision":2,"document":{"schema_version":1,"blocks":[]}}),
            vec![],
        )],
        vec![],
    );
    assert!(matches!(
        batches::preview(&root, p, "cli", "p").unwrap_err().code,
        ErrorCode::UnsupportedDocumentFormat
    ));
    let p = plan(
        vec![step(
            "file",
            "entity.create",
            json!({"kind":"note","title":"Reference","document":{"schema_version":2,"blocks":[{"id":"file","type":"file","content":[],"attrs":{"attachment_id":attachment.id,"name":"Original"}}]}}),
            vec![],
        )],
        vec![],
    );
    let preview = batches::preview(&root, p.clone(), "cli", "p").unwrap();
    std::fs::remove_file(root.join("attachments").join(&attachment.id)).unwrap();
    assert!(matches!(
        batches::apply(
            &root,
            ApplyBatch {
                plan: p,
                expected: preview.receipt
            },
            "cli",
            "a"
        )
        .unwrap_err()
        .code,
        ErrorCode::StorageError
    ));
    assert_eq!(hash(&reopened.conn), before);
}

#[test]
fn bindings_do_not_interpolate_literal_ast_text_and_reserved_upserts_fail() {
    let (_t, w) = setup();
    let mut p = full();
    p.steps[3].input["expression"] = json!({"op":"text","value":"$weight property_id created_at"});
    p.steps[3].bindings.pop();
    let result = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let id = result.steps[3].exports[&BatchExport::PropertyId].clone();
    let ExportValue::Id(id) = id else { panic!() };
    assert_eq!(
        result.checks[0].data.records.items[0].values[&id],
        "$weight property_id created_at"
    );
    p.steps[3]
        .bindings
        .push(serde_json::from_value(v("/expression/value", "weight", "property_id")).unwrap());
    assert_eq!(
        reason(batches::preview(&w.path, p, "cli", "p").unwrap_err()),
        "forbidden_binding_slot"
    );
    for kind in ["formula", "view"] {
        let (_t, mut w) = setup();
        let p = full();
        let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
        let c = w
            .collection_create(CreateCollection {
                name: "Other".into(),
            })
            .unwrap();
        if kind == "formula" {
            let id = preview.steps[3].exports[&BatchExport::PropertyId].clone();
            let ExportValue::Id(id) = id else { panic!() };
            w.conn
                .execute(
                    "INSERT INTO properties VALUES (?1,?2,'Existing',?3)",
                    params![
                        id,
                        c.collection.id,
                        r#"{"name":"Existing","type":"number"}"#
                    ],
                )
                .unwrap();
        } else {
            w.conn
                .execute(
                    "INSERT INTO views VALUES (?1,?2,1,'{}')",
                    params![
                        preview.steps[9].data["id"].as_str().unwrap(),
                        c.collection.id
                    ],
                )
                .unwrap();
        }
        let before = hash(&w.conn);
        assert_eq!(
            reason(batches::preview(&w.path, p, "cli", "p").unwrap_err()),
            "reserved_id_collision"
        );
        assert_eq!(hash(&w.conn), before);
    }
}

#[test]
fn hundred_steps_execute_as_one_batch_and_canonical_path_aliases_share_scope() {
    let (_t, w) = setup();
    let p = plan(
        (0..100)
            .map(|i| {
                step(
                    &format!("n{i}"),
                    "entity.create",
                    json!({"kind":"note","title":format!("Note {i}")}),
                    vec![],
                )
            })
            .collect(),
        vec![],
    );
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    assert_eq!(preview.steps.len(), 100);
    let applied = batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        "cli",
        "apply",
    )
    .unwrap();
    assert_eq!(applied.history_summary.len(), 100);
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        100
    );
    let t = tempfile::tempdir().unwrap();
    let alias = t.path().join("alias");
    std::os::unix::fs::symlink(&w.path, &alias).unwrap();
    let p = simple();
    let preview = batches::preview(&alias, p.clone(), "cli", "p").unwrap();
    batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        "cli",
        "apply",
    )
    .unwrap();
}

#[test]
fn binding_errors_identify_step_and_binding_and_keep_all_tables_unchanged() {
    let (_t, w) = setup();
    let before = hash(&w.conn);
    for kind in [
        "duplicate",
        "overlap",
        "missing",
        "noncanonical_index",
        "property_export",
    ] {
        let mut p = full();
        let s = &mut p.steps[5];
        let binding_index = match kind {
            "duplicate" => {
                s.bindings.push(s.bindings[1].clone());
                s.bindings.len() - 1
            }
            "overlap" => {
                s.bindings.push(
                    serde_json::from_value(v("/values/relación", "first", "entity_id")).unwrap(),
                );
                1
            }
            "missing" => {
                s.bindings[1].target = "/values/absent/0".into();
                1
            }
            "noncanonical_index" => {
                s.bindings[1].target = "/values/relación/00".into();
                1
            }
            "property_export" => {
                s.bindings[2].source.export = BatchExport::TitlePropertyId;
                s.bindings[2].source.step = "collection".into();
                2
            }
            _ => unreachable!(),
        };
        let error = batches::preview(&w.path, p, "cli", "invalid").unwrap_err();
        assert!(
            matches!(error.code, ErrorCode::ValidationError),
            "{kind}: {error:?}"
        );
        assert_eq!(error.details["step_key"], "second", "{kind}");
        assert_eq!(error.details["step_index"], 5, "{kind}");
        assert_eq!(error.details["binding_index"], binding_index, "{kind}");
        assert_eq!(hash(&w.conn), before);
    }
    let mut p = full();
    p.steps[1].key = p.steps[0].key.clone();
    let error = batch_plan::validate(&p).unwrap_err();
    assert_eq!(error.details["step_key"], "collection");
    assert_eq!(error.details["step_index"], 1);
}

#[test]
fn entity_create_record_keys_and_context_share_originals_and_observe_secondary_diff() {
    let (_t, w) = setup();
    let mut p = full();
    let first = &mut p.steps[4];
    first.operation = BatchOperation::EntityCreate;
    first.input = json!({"kind":"note","title":"Uno","record":{"collection_id":null,"values":{"a/b~c":3}},"context":{"kind":"project","target_id":null}});
    first.bindings = vec![
        serde_json::from_value(v("/record/collection_id", "collection", "collection_id")).unwrap(),
        serde_json::from_value(k("/record/values/a~1b~0c", "weight")).unwrap(),
        serde_json::from_value(v("/context/target_id", "project", "entity_id")).unwrap(),
    ];
    p.steps.insert(
        0,
        serde_json::from_value(step(
            "project",
            "entity.create",
            json!({"kind":"project","title":"Project","payload":{"state":"doing"}}),
            vec![],
        ))
        .unwrap(),
    );
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let relation = preview.diff.iter().find(|d| d.kind == "relation").unwrap();
    assert!(relation.before.is_none());
    assert_eq!(
        relation.after.as_ref().unwrap()["target_id"],
        preview.steps[0].data["id"]
    );
    let applied = batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        "cli",
        "apply",
    )
    .unwrap();
    assert_eq!(applied.checks[0].data.matching_count, 3);
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM relations", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
}

#[test]
fn apply_rejects_invalid_receipts_stale_revisions_and_late_step_or_check_atomically() {
    let (_t, w) = setup();
    let p = full();
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    let before = hash(&w.conn);
    for field in [
        "version",
        "workspace_id",
        "path_hash",
        "plan_hash",
        "base_hash",
        "effect_hash",
    ] {
        let mut raw = json!(preview.receipt);
        raw[field] = if field == "version" {
            json!(2)
        } else {
            json!("bad")
        };
        let input = ApplyBatch {
            plan: p.clone(),
            expected: serde_json::from_value(raw).unwrap(),
        };
        assert_eq!(
            reason(batches::apply(&w.path, input, "cli", "apply").unwrap_err()),
            "invalid_receipt"
        );
        assert_eq!(hash(&w.conn), before);
    }
    for failure in ["revision", "step", "check"] {
        let mut changed = p.clone();
        match failure {
            "revision" => {
                changed.steps[1].bindings.pop();
                changed.steps[1].input["expected_revision"] = json!(99);
            }
            "step" => changed.steps[5].input["values"]["peso"] = json!("bad"),
            "check" => changed.checks[0].input["limit"] = json!(1),
            _ => unreachable!(),
        }
        // Un recibo no es una autorización: modificar su plan_hash no evita
        // revalidar los pasos/checks sobre el original ni hace commit parcial.
        let mut expected = preview.receipt.clone();
        expected.plan_hash = store::json_hash("noteka.batch.plan.v1", &json!(changed)).unwrap();
        let error = batches::apply(
            &w.path,
            ApplyBatch {
                plan: changed,
                expected,
            },
            "cli",
            "apply",
        )
        .unwrap_err();
        assert_eq!(
            error.details["phase"],
            if failure == "check" { "check" } else { "step" }
        );
        assert_eq!(hash(&w.conn), before);
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM search_fts", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            0
        );
    }
}

#[test]
fn omitted_payload_stays_byte_exact_and_empty_update_keeps_existing_revision_semantics() {
    let (_t, mut w) = setup();
    let entity = w
        .entity_create(
            serde_json::from_value(json!({"kind":"note","title":"Unchanged"})).unwrap(),
            "cli",
            "seed",
        )
        .unwrap();
    let raw = r#"{ "inbox":true, "future":{"created_at":"literal", "civil":"2026-10-13"} }"#;
    w.conn
        .execute(
            "UPDATE entities SET payload=?2 WHERE id=?1",
            params![entity.id, raw],
        )
        .unwrap();
    let p = plan(
        vec![step(
            "empty",
            "entity.update",
            json!({"id":entity.id,"expected_revision":1}),
            vec![],
        )],
        vec![],
    );
    let preview = batches::preview(&w.path, p.clone(), "cli", "p").unwrap();
    assert_eq!(preview.steps[0].data["revision"], 2);
    assert_eq!(
        preview
            .diff
            .iter()
            .find(|d| d.kind == "entity")
            .unwrap()
            .before
            .as_ref()
            .unwrap()["title"],
        "Unchanged"
    );
    batches::apply(
        &w.path,
        ApplyBatch {
            plan: p,
            expected: preview.receipt,
        },
        "cli",
        "apply",
    )
    .unwrap();
    assert_eq!(
        w.conn
            .query_row(
                "SELECT payload FROM entities WHERE id=?1",
                [&entity.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        raw
    );
    // También el wrapper individual pasa por el helper, sin perder el formato.
    w.entity_update(
        serde_json::from_value(json!({"id":entity.id,"expected_revision":2,"title":"New title"}))
            .unwrap(),
        "cli",
        "individual",
    )
    .unwrap();
    assert_eq!(
        w.conn
            .query_row(
                "SELECT payload FROM entities WHERE id=?1",
                [&entity.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        raw
    );
}
