use noteka_core::{contracts::Response, execute_json, workspace::Workspace};
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
    fn collection(&self, name: &str) -> String {
        self.ok("collection.create", json!({"name":name}))["collection"]["id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn property(&self, c: &str, spec: Value) -> String {
        let rev = self.ok("collection.get", json!({"id":c}))["collection"]["revision"].clone();
        self.ok(
            "property.define",
            json!({"collection_id":c,"expected_revision":rev,"spec":spec}),
        )["id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn record(&self, c: &str, title: &str, values: Value) -> Value {
        self.ok(
            "record.create",
            json!({"collection_id":c,"title":title,"values":values}),
        )
    }
    fn get(&self, c: &str, e: &Value) -> Value {
        self.ok(
            "record.get",
            json!({"collection_id":c,"entity_id":e["entity"]["id"]}),
        )
    }
    fn db(&self) -> Connection {
        Connection::open(self.path.join("noteka.sqlite")).unwrap()
    }
}
fn error(r: Response, code: &str) {
    assert!(!r.ok);
    assert_eq!(
        serde_json::to_value(r.error.unwrap().code).unwrap(),
        json!(code)
    );
}

#[test]
fn shared_identity_creation_atomicity_patch_projections_and_unknown_values() {
    let f = Fixture::new();
    let c = f.collection("Registros");
    let t = f.property(&c, json!({"name":"Texto","type":"text"}));
    let before = f.ok("entity.list", json!({}));
    error(
        f.call(
            "record.create",
            json!({"collection_id":c,"title":"No confirmar","values":{"ajena":1}}),
        ),
        "validation_error",
    );
    assert_eq!(f.ok("entity.list", json!({})), before);
    let task = f.ok(
        "entity.create",
        json!({"kind":"task","title":"Compartida","payload":{"priority":"high"}}),
    );
    let id = task["id"].clone();
    let record=f.ok("record.attach",json!({"collection_id":c,"entity_id":id,"expected_revision":1,"values":{t.clone():"Mantener"}}));
    assert_eq!(record["entity"]["id"], id);
    assert_eq!(record["entity"]["payload"], task["payload"]);
    let title_prop = f.ok("collection.get", json!({"id":c}))["properties"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let updated = f.ok(
        "entity.update",
        json!({"id":id,"expected_revision":2,"title":"Mismo ID"}),
    );
    assert_eq!(f.get(&c, &record)["values"][&title_prop], updated["title"]);
    let p = f.property(&c, json!({"name":"Fecha de creación","type":"created_at"}));
    assert_eq!(f.get(&c, &record)["values"][&p], task["created_at"]);
    let another = f.collection("Otra");
    let r = f.ok(
        "record.attach",
        json!({"collection_id":another,"entity_id":id,"expected_revision":3}),
    );
    assert_eq!(r["collection_ids"].as_array().unwrap().len(), 2);
    assert_eq!(f.get(&c, &record)["values"][&t], "Mantener");
    error(f.call("record.update",json!({"collection_id":c,"entity_id":id,"expected_revision":3,"values":{t.clone():"Stale"}})),"revision_conflict");
    let cleared = f.ok(
        "record.update",
        json!({"collection_id":c,"entity_id":id,"expected_revision":4,"values":{t.clone():null}}),
    );
    assert!(cleared["values"].get(&t).is_none());
    error(f.call("record.update",json!({"collection_id":c,"entity_id":id,"expected_revision":5,"values":{title_prop:"No escribir"}})),"validation_error");
    f.db()
        .execute(
            "INSERT INTO property_values VALUES (?1,?2,?3)",
            rusqlite::params![id.as_str().unwrap(), t, json!({"future":42}).to_string()],
        )
        .unwrap();
    error(f.call("record.update",json!({"collection_id":c,"entity_id":id,"expected_revision":5,"values":{t.clone():"No descartar"}})),"validation_error");
    assert_eq!(f.get(&c, &record)["values"][&t]["future"], 42);
}

#[test]
fn every_property_type_validates_values_without_coercion_or_loss() {
    let f = Fixture::new();
    let c = f.collection("Tipos");
    let source = f.temp.path().join("file");
    fs::write(&source, b"original").unwrap();
    let a = f.ok("attachment.import", json!({"source_path":source}));
    let cases = vec![
        (json!({"name":"text","type":"text"}), json!(""), json!(1)),
        (
            json!({"name":"num","type":"number"}),
            json!(1.25),
            json!("1.25"),
        ),
        (
            json!({"name":"decimal","type":"decimal"}),
            json!("1234567890123456789.123456789"),
            json!("1e3"),
        ),
        (
            json!({"name":"money","type":"money","currency":"BOB"}),
            json!("145.69"),
            json!(145.69),
        ),
        (
            json!({"name":"bool","type":"checkbox"}),
            json!(false),
            json!("false"),
        ),
        (
            json!({"name":"date","type":"date"}),
            json!("2028-02-29"),
            json!("2026-02-29"),
        ),
        (
            json!({"name":"range","type":"date_range"}),
            json!({"start":"2026-10-01","end":"2026-10-12"}),
            json!({"start":"2026-10-12","end":"2026-10-01"}),
        ),
        (
            json!({"name":"select","type":"select","options":["A","B"]}),
            json!("A"),
            json!("C"),
        ),
        (
            json!({"name":"multi","type":"multi_select","options":["A","B"]}),
            json!(["B", "A"]),
            json!(["A", "A"]),
        ),
        (
            json!({"name":"url","type":"url"}),
            json!("https://example.org/p"),
            json!("javascript:alert(1)"),
        ),
        (
            json!({"name":"email","type":"email"}),
            json!("fixture@example.org"),
            json!("fixture @example.org"),
        ),
        (
            json!({"name":"files","type":"files"}),
            json!([a["id"]]),
            json!(["absent"]),
        ),
    ];
    let r = f.record(&c, "Tipos", json!({}));
    let id = r["entity"]["id"].clone();
    let mut revision = 1;
    for (spec, valid, invalid) in cases {
        let p = f.property(&c, spec);
        let r=f.ok("record.update",json!({"collection_id":c,"entity_id":id,"expected_revision":revision,"values":{p.clone():valid}}));
        revision += 1;
        assert_eq!(r["values"][&p], valid);
        error(f.call("record.update",json!({"collection_id":c,"entity_id":id,"expected_revision":revision,"values":{p.clone():invalid}})),"validation_error");
        assert_eq!(f.get(&c, &r)["entity"]["revision"], revision);
    }
    for spec in [
        json!({"name":"badmoney","type":"money","currency":"bob"}),
        json!({"name":"badselect","type":"select","options":["A","A"]}),
        json!({"name":"code","type":"formula"}),
        json!({"name":"title2","type":"title"}),
        json!({"name":"unused","type":"text","options":["A"]}),
    ] {
        let rev = f.ok("collection.get", json!({"id":c}))["collection"]["revision"].clone();
        error(
            f.call(
                "property.define",
                json!({"collection_id":c,"expected_revision":rev,"spec":spec}),
            ),
            "validation_error",
        );
    }
}

#[test]
fn relationships_inverse_archived_targets_and_rollback() {
    let f = Fixture::new();
    let c = f.collection("Members");
    let target = f.collection("Subscriptions");
    let sub = f.record(&target, "Plan", json!({}));
    let p = f.property(
        &c,
        json!({"name":"Plan","type":"relation","target_collection_id":target}),
    );
    let r = f.record(&c, "Miembro", json!({p.clone():[sub["entity"]["id"]]}));
    let inverse = f.ok(
        "record.inverse",
        json!({"property_id":p,"target_id":sub["entity"]["id"]}),
    );
    assert_eq!(inverse["items"][0]["entity"]["id"], r["entity"]["id"]);
    f.ok(
        "entity.archive",
        json!({"id":sub["entity"]["id"],"expected_revision":1}),
    );
    assert_eq!(f.get(&c, &r)["values"][&p][0], sub["entity"]["id"]);
    error(f.call("record.update",json!({"collection_id":c,"entity_id":r["entity"]["id"],"expected_revision":1,"values":{p.clone():[r["entity"]["id"]]}})),"validation_error");
    let updated=f.ok("record.update",json!({"collection_id":c,"entity_id":r["entity"]["id"],"expected_revision":1,"values":{p.clone():[]}}));
    assert_eq!(updated["values"][&p], json!([]));
    assert_eq!(
        f.ok(
            "record.inverse",
            json!({"property_id":p,"target_id":sub["entity"]["id"]})
        )["items"],
        json!([])
    );
}

#[test]
fn view_filters_exact_decimal_order_pagination_and_schema_conflicts() {
    let f = Fixture::new();
    let c = f.collection("Vista");
    let p = f.property(&c, json!({"name":"Importe","type":"decimal"}));
    let b = f.property(&c, json!({"name":"Pagó","type":"checkbox"}));
    for (title, amount) in [
        ("negative", "-0.25"),
        ("two", "2.000000001"),
        ("ten", "10"),
        ("two-min", "2.000000000"),
    ] {
        f.record(&c, title, json!({p.clone():amount,b.clone():false}));
    }
    f.record(&c, "empty", json!({}));
    f.record(&c, "paid", json!({p.clone():"1",b.clone():true}));
    let cfg = json!({"columns":[p,b],"filters":[{"property_id":b,"op":"eq","value":false}],"sorts":[{"property_id":p}]});
    let view = f.ok(
        "view.save",
        json!({"collection_id":c,"name":"Pendientes","layout":"table","config":cfg}),
    );
    let saved = f.ok("collection.get", json!({"id":c}));
    assert_eq!(saved["views"][0], view);
    let mut cursor = Value::Null;
    let mut titles = vec![];
    loop {
        let page = f.ok(
            "database.query",
            json!({"collection_id":c,"view_id":view["id"],"limit":2,"cursor":cursor}),
        );
        titles.extend(
            page["records"]["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r["entity"]["title"].clone()),
        );
        cursor = page["records"]["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }
    assert_eq!(
        titles,
        json!(["negative", "two-min", "two", "ten"])
            .as_array()
            .unwrap()
            .clone()
    );
    let numeric = f.ok(
        "database.query",
        json!({"collection_id":c,"config":{"filters":[{"property_id":p,"op":"eq","value":"2"}]}}),
    );
    assert_eq!(numeric["records"]["items"][0]["entity"]["title"], "two-min");
    f.ok("view.save",json!({"collection_id":c,"id":view["id"],"expected_revision":1,"name":"Lista","layout":"list","config":cfg}));
    error(f.call("view.save",json!({"collection_id":c,"id":view["id"],"expected_revision":1,"name":"Old","layout":"table","config":cfg})),"revision_conflict");
    for cfg in [
        json!({"columns":["ajena"]}),
        json!({"filters":[{"property_id":p,"op":"contains","value":"1"}]}),
        json!({"filters":[{"property_id":b,"op":"eq","value":"false"}]}),
    ] {
        error(
            f.call("database.query", json!({"collection_id":c,"config":cfg})),
            "validation_error",
        );
    }
    error(
        f.call(
            "property.define",
            json!({"collection_id":c,"expected_revision":1,"spec":{"name":"stale","type":"text"}}),
        ),
        "revision_conflict",
    );
}

#[test]
fn migration_backup_originals_and_future_schema_are_preserved() {
    let f = Fixture::new();
    let conn = f.db();
    conn.execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations")
        .unwrap();
    conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;DROP TABLE templates;DROP TABLE calculations;DROP TABLE views;DROP TABLE property_values;DROP TABLE memberships;DROP TABLE properties;DROP TABLE collections;UPDATE workspace_meta SET schema_version=3;PRAGMA user_version=3;").unwrap();
    drop(conn);
    assert_eq!(
        f.ok("workspace.open", json!({}))["schema_version"],
        noteka_core::workspace::SCHEMA_VERSION
    );
    assert!(fs::read_dir(f.temp.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("noteka-pre-migration-")
    }));
    let c = f.collection("Backup");
    let p = f.property(&c, json!({"name":"Costo","type":"money","currency":"BOB"}));
    let file = f.temp.path().join("original");
    fs::write(&file, b"archivo en propiedad").unwrap();
    let attachment = f.ok("attachment.import", json!({"source_path":file}));
    let files = f.property(&c, json!({"name":"Originales","type":"files"}));
    let r = f.record(
        &c,
        "Recuperable",
        json!({p.clone():"145.69",files:[attachment["id"]]}),
    );
    let view=f.ok("view.save",json!({"collection_id":c,"name":"Recuperable","layout":"list","config":{"sorts":[{"property_id":p}]}}));
    let before = f.ok("collection.get", json!({"id":c}));
    let backup = f.temp.path().join("backup");
    f.ok("workspace.backup", json!({"destination":backup}));
    let restore = f.temp.path().join("restore");
    f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":restore}),
    );
    let restored = execute_json(
        json!({"workspace":restore,"command":{"operation":"record.get","input":{"collection_id":c,"entity_id":r["entity"]["id"]}}}),
        "test",
    );
    assert_eq!(restored.data.unwrap(), r);
    assert_eq!(
        execute_json(
            json!({"workspace":restore,"command":{"operation":"collection.get","input":{"id":c}}}),
            "test"
        )
        .data
        .unwrap(),
        before
    );
    assert_eq!(execute_json(json!({"workspace":restore,"command":{"operation":"attachment.read","input":{"id":attachment["id"]}}}),"test").data.unwrap()["base64"],"YXJjaGl2byBlbiBwcm9waWVkYWQ=");
    let future_view=json!({"id":view["id"],"collection_id":c,"revision":1,"name":"Recuperable","layout":"list","config":view["config"],"future":{"keep":42}}).to_string();
    f.db()
        .execute(
            "UPDATE views SET json=?1 WHERE id=?2",
            rusqlite::params![future_view, view["id"].as_str().unwrap()],
        )
        .unwrap();
    assert!(!f.call("view.save",json!({"collection_id":c,"id":view["id"],"expected_revision":1,"name":"No descartar","layout":"table","config":{}})).ok);
    assert_eq!(
        f.db()
            .query_row(
                "SELECT json FROM views WHERE id=?1",
                [view["id"].as_str().unwrap()],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        future_view
    );
    let spec =
        json!({"name":"Costo","type":"money","currency":"BOB","future":{"keep":42}}).to_string();
    f.db()
        .execute(
            "UPDATE properties SET spec=?1 WHERE id=?2",
            rusqlite::params![spec, p],
        )
        .unwrap();
    assert!(!f.call("collection.get", json!({"id":c})).ok);
    assert_eq!(
        f.db()
            .query_row("SELECT spec FROM properties WHERE id=?1", [p], |r| r
                .get::<_, String>(0))
            .unwrap(),
        spec
    );
}
#[test]
fn record_failures_rollback_values_memberships_history_and_index() {
    let f = Fixture::new();
    let c = f.collection("Atomicidad");
    let p = f.property(&c, json!({"name":"Texto","type":"text"}));
    let r = f.record(&c, "Original", json!({p.clone():"Inicial"}));
    let other = f.collection("Otra");
    let history = f.ok("changes.list", json!({"entity_id":r["entity"]["id"]}));
    let search = f.ok("search.query", json!({"query":"Original"}));
    f.db().execute_batch("CREATE TRIGGER injected BEFORE INSERT ON changes BEGIN SELECT RAISE(ABORT,'injected');END;").unwrap();
    error(f.call("record.update",json!({"collection_id":c,"entity_id":r["entity"]["id"],"expected_revision":1,"values":{p:"No confirmar"}})),"storage_error");
    error(
        f.call(
            "record.attach",
            json!({"collection_id":other,"entity_id":r["entity"]["id"],"expected_revision":1}),
        ),
        "storage_error",
    );
    error(
        f.call(
            "record.create",
            json!({"collection_id":c,"title":"No quedar","values":{}}),
        ),
        "storage_error",
    );
    assert_eq!(f.get(&c, &r), r);
    assert_eq!(
        f.ok("changes.list", json!({"entity_id":r["entity"]["id"]})),
        history
    );
    assert_eq!(f.ok("search.query", json!({"query":"Original"})), search);
    assert_eq!(
        f.ok("database.query", json!({"collection_id":other}))["records"]["items"],
        json!([])
    );
    assert_eq!(
        f.ok("entity.list", json!({}))["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn view_null_descending_empty_and_pagination_limits() {
    let f = Fixture::new();
    let c = f.collection("Orden");
    let num = f.property(&c, json!({"name":"Número","type":"number"}));
    let text = f.property(&c, json!({"name":"Texto","type":"text"}));
    for (title, n, s) in [
        ("cero", json!(-0.0), json!("")),
        ("alto", json!(10), json!("A")),
        ("sin", Value::Null, Value::Null),
    ] {
        f.record(&c, title, json!({num.clone():n,text.clone():s}));
    }
    let page = f.ok(
        "database.query",
        json!({"collection_id":c,"config":{"sorts":[{"property_id":num,"descending":true}]}}),
    );
    assert_eq!(
        page["records"]["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["entity"]["title"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["alto", "cero", "sin"]
    );
    assert_eq!(f.ok("database.query",json!({"collection_id":c,"config":{"filters":[{"property_id":num,"op":"eq","value":0}]}}))["records"]["items"].as_array().unwrap().len(),1);
    assert_eq!(
        f.ok(
            "database.query",
            json!({"collection_id":c,"config":{"filters":[{"property_id":text,"op":"is_empty"}]}})
        )["records"]["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    for input in [
        json!({"limit":0}),
        json!({"limit":101}),
        json!({"cursor":"-1"}),
        json!({"cursor":"10001"}),
        json!({"cursor":"text"}),
    ] {
        let mut input = input;
        input["collection_id"] = json!(c);
        error(f.call("database.query", input), "validation_error");
    }
    let current = f.ok("collection.get", json!({"id":c}))["collection"]["revision"].clone();
    error(f.call("property.define",json!({"collection_id":c,"expected_revision":current,"spec":{"name":"Número","type":"text"}})),"validation_error");
    f.ok(
        "entity.archive",
        json!({"id":page["records"]["items"][0]["entity"]["id"],"expected_revision":1}),
    );
    assert_eq!(
        f.ok(
            "database.query",
            json!({"collection_id":c,"config":{"archived":true}})
        )["records"]["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
