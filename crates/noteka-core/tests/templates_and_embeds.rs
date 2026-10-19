use noteka_core::{contracts::Response, execute_json, workspace::Workspace};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::PathBuf};

struct Fixture {
    temp: tempfile::TempDir,
    path: PathBuf,
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
    fn collection(&self) -> String {
        self.ok("collection.create", json!({"name":"Synthetic"}))["collection"]["id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn rev(&self, c: &str) -> Value {
        self.ok("collection.get", json!({"id":c}))["collection"]["revision"].clone()
    }
    fn property(&self, c: &str, spec: Value) -> String {
        self.ok(
            "property.define",
            json!({"collection_id":c,"expected_revision":self.rev(c),"spec":spec}),
        )["id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn view(&self, c: &str) -> String {
        self.ok(
            "view.save",
            json!({"collection_id":c,"name":"Originals","layout":"table","config":{}}),
        )["id"]
            .as_str()
            .unwrap()
            .into()
    }
    fn save(&self, c: &str, definition: Value) -> Value {
        self.ok("template.save",json!({"collection_id":c,"expected_collection_revision":self.rev(c),"name":"Page","definition":definition}))
    }
    fn instantiate(&self, t: &Value, values: Value) -> Value {
        self.ok("template.instantiate",json!({"id":t["id"],"expected_template_revision":t["revision"],"expected_collection_revision":self.rev(t["collection_id"].as_str().unwrap()),"values":values}))
    }
    fn db(&self) -> Connection {
        Connection::open(self.path.join("noteka.sqlite")).unwrap()
    }
    fn counts(&self) -> Vec<i64> {
        [
            "entities",
            "documents",
            "memberships",
            "property_values",
            "changes",
            "search_fts",
            "templates",
        ]
        .iter()
        .map(|name| {
            self.db()
                .query_row(&format!("SELECT count(*) FROM {name}"), [], |r| r.get(0))
                .unwrap()
        })
        .collect()
    }
}
fn definition(document: Value, values: Value) -> Value {
    json!({"schema_version":1,"title":"Instance","document":document,"values":values})
}
fn doc() -> Value {
    json!({"schema_version":2,"blocks":[{"id":"quote","type":"blockquote","content":[],"children":[{"id":"text","type":"paragraph","content":[{"type":"text","text":"Source only"}]}]}]})
}
fn embed(c: &str, v: &str) -> Value {
    json!({"schema_version":3,"blocks":[{"id":"ref","type":"collection_view","content":[],"attrs":{"collection_id":c,"view_id":v}}]})
}
fn error(r: Response, expected: &str) {
    assert!(!r.ok);
    assert_eq!(
        serde_json::to_value(r.error.unwrap().code).unwrap(),
        expected
    );
}
fn block_ids(d: &Value) -> BTreeSet<String> {
    fn walk(v: &Value, out: &mut BTreeSet<String>) {
        if let Some(id) = v["id"].as_str() {
            out.insert(id.into());
        }
        if let Some(children) = v["children"].as_array() {
            for c in children {
                walk(c, out);
            }
        }
    }
    let mut ids = BTreeSet::new();
    for b in d["blocks"].as_array().unwrap() {
        walk(b, &mut ids);
    }
    ids
}

#[test]
fn template_limits_strict_inputs_foreign_owners_and_missing_originals() {
    let f = Fixture::new();
    let c = f.collection();
    let other = f.collection();
    let t = f.save(&c, definition(doc(), json!({})));
    let base = json!({"collection_id":c,"expected_collection_revision":f.rev(&c),"name":"Valid","definition":t["definition"]});
    let before = f.counts();
    for (key, value) in [
        ("name", json!(" ")),
        ("name", json!("x".repeat(257))),
        ("future", json!(true)),
        ("id", t["id"].clone()),
        ("expected_revision", json!(1)),
    ] {
        let mut i = base.clone();
        i[key] = value;
        error(f.call("template.save", i), "validation_error");
        assert_eq!(f.counts(), before);
    }
    for title in [" ".into(), "x".repeat(4097)] {
        let mut i = base.clone();
        i["definition"]["title"] = json!(title);
        error(f.call("template.save", i), "validation_error");
        assert_eq!(f.counts(), before);
    }
    let mut i = base.clone();
    i["definition"]["values"] =
        Value::Object((0..129).map(|n| (n.to_string(), Value::Null)).collect());
    error(f.call("template.save", i), "validation_error");
    let mut i = base.clone();
    i["definition"]["document"]["blocks"][0]["content"] =
        json!([{"type":"text","text":"x".repeat(1_048_576)}]);
    error(f.call("template.save", i), "validation_error");
    let mut i = base;
    i["id"] = t["id"].clone();
    i["expected_revision"] = json!(1);
    i["collection_id"] = json!(other);
    error(f.call("template.save", i), "validation_error");
    let p = f.property(
        &c,
        json!({"name":"Relation","type":"relation","target_collection_id":other}),
    );
    let original = f.ok(
        "record.create",
        json!({"collection_id":c,"title":"Not a member of target"}),
    );
    error(f.call("template.save",json!({"collection_id":c,"expected_collection_revision":f.rev(&c),"name":"Bad relation","definition":definition(doc(),json!({p:[original["entity"]["id"]]}))})),"validation_error");
    let file = f.property(&c, json!({"name":"File","type":"files"}));
    error(f.call("template.save",json!({"collection_id":c,"expected_collection_revision":f.rev(&c),"name":"Bad file","definition":definition(doc(),json!({file:[uuid::Uuid::new_v4().to_string()]}))})),"validation_error");
    let mut future = t["definition"].clone();
    future["schema_version"] = json!(99);
    f.db()
        .execute(
            "UPDATE templates SET definition=?2 WHERE id=?1",
            params![t["id"].as_str().unwrap(), future.to_string()],
        )
        .unwrap();
    assert_eq!(
        f.ok("template.get", json!({"id":t["id"]}))["editable"],
        false
    );
    let before = f.counts();
    error(f.call("template.instantiate",json!({"id":t["id"],"expected_template_revision":1,"expected_collection_revision":f.rev(&c)})),"unsupported_template_format");
    assert_eq!(f.counts(), before);
}

#[test]
fn all_writable_defaults_strict_contracts_pagination_and_revisions() {
    let f = Fixture::new();
    let c = f.collection();
    let target = f.collection();
    let r = f.ok(
        "record.create",
        json!({"collection_id":target,"title":"Related original"}),
    );
    let source = f.temp.path().join("original.txt");
    std::fs::write(&source, "Original immutable").unwrap();
    let a = f.ok("attachment.import", json!({"source_path":source}));
    let mut values = json!({});
    for (kind, value, extras) in [
        ("text", json!("abc"), json!({})),
        ("number", json!(42), json!({})),
        ("decimal", json!("1.234567891"), json!({})),
        ("money", json!("145.69"), json!({"currency":"BOB"})),
        ("checkbox", json!(false), json!({})),
        ("date", json!("2024-02-29"), json!({})),
        (
            "date_range",
            json!({"start":"2024-02-28","end":"2024-03-01"}),
            json!({}),
        ),
        ("select", json!("a"), json!({"options":["a","b"]})),
        (
            "multi_select",
            json!(["a", "b"]),
            json!({"options":["a","b"]}),
        ),
        ("url", json!("https://example.test"), json!({})),
        ("email", json!("x@example.test"), json!({})),
        ("files", json!([a["id"]]), json!({})),
        (
            "relation",
            json!([r["entity"]["id"]]),
            json!({"target_collection_id":target}),
        ),
    ] {
        let mut spec = json!({"name":kind,"type":kind});
        spec.as_object_mut()
            .unwrap()
            .extend(extras.as_object().unwrap().clone());
        let p = f.property(&c, spec);
        values[&p] = value;
    }
    let t = f.save(&c, definition(doc(), values.clone()));
    assert_eq!(t["editable"], true);
    let instance = f.instantiate(&t, json!({}));
    for (key, value) in values.as_object().unwrap() {
        assert_eq!(&instance["values"][key], value);
    }
    assert_eq!(f.rev(&c), 14); // plantilla/instancia no cambian esquema
    let mut ids = BTreeSet::new();
    ids.insert(t["id"].as_str().unwrap().to_owned());
    for _ in 0..3 {
        ids.insert(
            f.save(&c, definition(doc(), json!({})))["id"]
                .as_str()
                .unwrap()
                .into(),
        );
    }
    let mut cursor = Value::Null;
    let mut got = BTreeSet::new();
    loop {
        let page = f.ok(
            "template.list",
            json!({"collection_id":c,"limit":2,"cursor":cursor}),
        );
        for x in page["items"].as_array().unwrap() {
            assert!(got.insert(x["id"].as_str().unwrap().to_owned()));
        }
        cursor = page["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }
    assert_eq!(got, ids);
    let input = json!({"collection_id":c,"expected_collection_revision":f.rev(&c),"id":t["id"],"expected_revision":1,"name":"Edited","definition":t["definition"]});
    assert_eq!(f.ok("template.save", input.clone())["revision"], 2);
    error(f.call("template.save", input), "revision_conflict");
    for limit in [0, 201] {
        error(
            f.call("template.list", json!({"collection_id":c,"limit":limit})),
            "validation_error",
        );
    }
    for (key, value) in [
        ("schema_version", json!(99)),
        ("future", json!(true)),
        ("title", json!("")),
    ] {
        let mut d = definition(doc(), json!({}));
        d[key] = value;
        error(f.call("template.save",json!({"collection_id":c,"expected_collection_revision":f.rev(&c),"name":"Invalid","definition":d})),"validation_error");
    }
    error(f.call("template.instantiate",json!({"id":t["id"],"expected_template_revision":2,"expected_collection_revision":f.rev(&c),"title":null})),"validation_error");
    let catalog = noteka_core::describe();
    for op in [
        "template.save",
        "template.get",
        "template.list",
        "template.instantiate",
    ] {
        assert!(catalog["inputs"][op].is_object());
        assert!(catalog["outputs"][op].is_object());
    }
}

#[test]
fn independent_instances_fresh_nested_ids_overrides_null_and_shared_originals() {
    let f = Fixture::new();
    let c = f.collection();
    let v = f.view(&c);
    let cost = f.property(&c, json!({"name":"Cost","type":"money","currency":"BOB"}));
    let note = f.property(&c, json!({"name":"Text","type":"text"}));
    f.ok("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"name":"Saved AST","expression":{"op":"number","value":2}}));
    let path = f.temp.path().join("a.txt");
    std::fs::write(&path, "Immutable").unwrap();
    let a = f.ok("attachment.import", json!({"source_path":path}));
    let mut d = doc();
    d["schema_version"] = json!(3);
    d["blocks"].as_array_mut().unwrap().extend([embed(&c,&v)["blocks"][0].clone(),json!({"id":"file","type":"file","content":[],"attrs":{"attachment_id":a["id"],"name":"a.txt"}})]);
    let t = f.save(
        &c,
        definition(
            d.clone(),
            json!({cost.clone():"145.69",note.clone():"remove me"}),
        ),
    );
    let one = f.instantiate(&t, json!({note.clone():null}));
    let two = f.instantiate(&t, json!({cost.clone():"48.57"}));
    assert_ne!(one["entity"]["id"], two["entity"]["id"]);
    assert_eq!(one["values"][&cost], "145.69");
    assert!(one["values"][&note].is_null());
    assert_eq!(two["values"][&cost], "48.57");
    assert_eq!(two["values"][&note], "remove me");
    let first = block_ids(&one["entity"]["document"]);
    let second = block_ids(&two["entity"]["document"]);
    assert!(first.is_disjoint(&second));
    assert!(first.is_disjoint(&block_ids(&d)));
    assert_eq!(first.len(), 4);
    assert_eq!(
        one["entity"]["document"]["blocks"][1]["attrs"],
        d["blocks"][1]["attrs"]
    );
    assert_eq!(
        one["entity"]["document"]["blocks"][2]["attrs"],
        d["blocks"][2]["attrs"]
    );
    assert_eq!(one["entity"]["payload"]["inbox"], false);
    f.ok(
        "entity.update",
        json!({"id":one["entity"]["id"],"expected_revision":1,"title":"Changed only one"}),
    );
    assert_eq!(
        f.ok("entity.get", json!({"id":two["entity"]["id"]}))["title"],
        "Instance"
    );
    assert_eq!(f.ok("template.get", json!({"id":t["id"]})), t);
    assert_eq!(
        f.ok("attachment.read", json!({"id":a["id"]}))["attachment"],
        a
    );
    let backup = f.temp.path().join("backup");
    let restored = f.temp.path().join("restored");
    f.ok("workspace.backup", json!({"destination":backup}));
    f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":restored}),
    );
    for (op, input) in [
        ("template.get", json!({"id":t["id"]})),
        ("collection.get", json!({"id":c})),
        (
            "record.get",
            json!({"collection_id":c,"entity_id":one["entity"]["id"]}),
        ),
        (
            "record.get",
            json!({"collection_id":c,"entity_id":two["entity"]["id"]}),
        ),
        ("attachment.read", json!({"id":a["id"]})),
    ] {
        let expected = f.ok(op, input.clone());
        let actual = execute_json(
            json!({"workspace":restored,"command":{"operation":op,"input":input}}),
            "test",
        );
        assert!(actual.ok, "{:?}", actual.error);
        assert_eq!(actual.data.unwrap(), expected);
    }
}

#[test]
fn invalid_defaults_overrides_schema_and_storage_failures_leave_no_partial_state() {
    let f = Fixture::new();
    let c = f.collection();
    let p = f.property(&c, json!({"name":"Money","type":"money","currency":"BOB"}));
    let t = f.save(&c, definition(doc(), json!({p.clone():"5.00"})));
    let input = json!({"id":t["id"],"expected_template_revision":1,"expected_collection_revision":f.rev(&c)});
    for patch in [
        json!({p.clone():1.2}),
        json!({uuid::Uuid::new_v4().to_string():true}),
    ] {
        let before = f.counts();
        let mut i = input.clone();
        i["values"] = patch;
        error(f.call("template.instantiate", i), "validation_error");
        assert_eq!(f.counts(), before);
    }
    for table in ["documents", "memberships", "property_values", "changes"] {
        f.db().execute_batch(&format!("CREATE TRIGGER injected BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT,'test'); END;")).unwrap();
        let before = f.counts();
        error(
            f.call("template.instantiate", input.clone()),
            "storage_error",
        );
        assert_eq!(f.counts(), before);
        f.db().execute_batch("DROP TRIGGER injected;").unwrap();
    }
    f.db()
        .execute_batch("ALTER TABLE search_fts RENAME TO temporarily_missing;")
        .unwrap();
    let before: i64 = f
        .db()
        .query_row("SELECT count(*) FROM entities", [], |r| r.get(0))
        .unwrap();
    error(
        f.call("template.instantiate", input.clone()),
        "storage_error",
    );
    assert_eq!(
        f.db()
            .query_row("SELECT count(*) FROM entities", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        before
    );
    f.db()
        .execute_batch("ALTER TABLE temporarily_missing RENAME TO search_fts;")
        .unwrap();
    let title_id = f.ok("collection.get", json!({"id":c}))["properties"][0]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    f.ok("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"name":"Calculated","expression":{"op":"number","value":2}}));
    error(
        f.call("template.instantiate", input.clone()),
        "revision_conflict",
    );
    let calc = f.ok("collection.get", json!({"id":c}))["properties"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["spec"]["name"] == "Calculated")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    for key in [&title_id, &calc] {
        let mut i = input.clone();
        i["expected_collection_revision"] = f.rev(&c);
        i["values"] = json!({key:null});
        error(f.call("template.instantiate", i), "validation_error");
    }
    // No ocultar un default inválido bajo override válido.
    let raw = definition(doc(), json!({p.clone():1.5}));
    f.db()
        .execute(
            "UPDATE templates SET definition=?2 WHERE id=?1",
            params![t["id"].as_str().unwrap(), raw.to_string()],
        )
        .unwrap();
    let mut i = input;
    i["expected_collection_revision"] = f.rev(&c);
    i["values"] = json!({p:"7.50"});
    let before = f.counts();
    error(f.call("template.instantiate", i), "validation_error");
    assert_eq!(f.counts(), before);
}

#[test]
fn future_template_fields_values_and_backups_are_preserved_without_replacement() {
    let f = Fixture::new();
    let c = f.collection();
    let p = f.property(&c, json!({"name":"Decimal","type":"decimal"}));
    let t = f.save(&c, definition(doc(), json!({p.clone():"1"})));
    let mut raw = t["definition"].clone();
    raw["future"] = json!({"keep":[1,2,3]});
    let bytes = serde_json::to_string_pretty(&raw).unwrap();
    f.db()
        .execute(
            "UPDATE templates SET definition=?2 WHERE id=?1",
            params![t["id"].as_str().unwrap(), bytes],
        )
        .unwrap();
    let got = f.ok("template.get", json!({"id":t["id"]}));
    assert_eq!(got["definition"], raw);
    assert_eq!(got["editable"], false);
    assert_eq!(
        f.ok("template.list", json!({"collection_id":c}))["items"][0],
        got
    );
    error(f.call("template.save",json!({"id":t["id"],"collection_id":c,"expected_revision":1,"expected_collection_revision":f.rev(&c),"name":"Overwrite","definition":t["definition"]})),"unsupported_template_format");
    error(f.call("template.instantiate",json!({"id":t["id"],"expected_template_revision":1,"expected_collection_revision":f.rev(&c)})),"unsupported_template_format");
    let backup = f.temp.path().join("backup");
    f.ok("workspace.backup", json!({"destination":backup}));
    let restored = f.temp.path().join("restored");
    f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":restored}),
    );
    assert_eq!(
        Connection::open(restored.join("noteka.sqlite"))
            .unwrap()
            .query_row("SELECT definition FROM templates", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        bytes
    );
    // Formato futuro en un valor dentro de definición conocida tampoco se descarta al actualizar.
    let raw = definition(doc(), json!({p:{"future_decimal":"1"}}));
    f.db()
        .execute(
            "UPDATE templates SET definition=?2 WHERE id=?1",
            params![t["id"].as_str().unwrap(), raw.to_string()],
        )
        .unwrap();
    error(f.call("template.save",json!({"id":t["id"],"collection_id":c,"expected_revision":1,"expected_collection_revision":f.rev(&c),"name":"Overwrite","definition":t["definition"]})),"unsupported_template_format");
}

#[test]
fn embedded_documents_validated_atomically_future_targets_not_converted() {
    let f = Fixture::new();
    let c = f.collection();
    let v = f.view(&c);
    let other = f.collection();
    let d = embed(&c, &v);
    let n = f.ok(
        "entity.create",
        json!({"kind":"note","title":"Container","document":d}),
    );
    let before = f.counts();
    let mut bad = vec![];
    for version in [1, 2, 99] {
        let mut x = d.clone();
        x["schema_version"] = json!(version);
        bad.push(x);
    }
    let mut x = d.clone();
    x["blocks"][0]["attrs"]["collection_id"] = json!(other);
    bad.push(x);
    let mut x = d.clone();
    x["blocks"][0]["attrs"]["view_id"] = json!(uuid::Uuid::new_v4().to_string());
    bad.push(x);
    let mut x = d.clone();
    x["blocks"][0]["attrs"]["layout"] = json!("table");
    bad.push(x);
    let mut x = d.clone();
    x["blocks"][0]["content"] = json!([{"type":"text","text":"Not a snapshot"}]);
    bad.push(x);
    let mut x = d.clone();
    x["blocks"][0]["children"] = json!([doc()["blocks"][0]]);
    bad.push(x);
    for bad in bad {
        assert!(
            !f.call(
                "entity.update",
                json!({"id":n["id"],"expected_revision":1,"document":bad})
            )
            .ok
        );
        assert_eq!(f.counts(), before);
        assert_eq!(f.ok("entity.get", json!({"id":n["id"]})), n);
    }
    let mut nested = doc();
    nested["schema_version"] = json!(3);
    nested["blocks"][0]["children"]
        .as_array_mut()
        .unwrap()
        .push(d["blocks"][0].clone());
    f.ok(
        "entity.update",
        json!({"id":n["id"],"expected_revision":1,"document":nested}),
    );
    let mut raw = f.ok("collection.get", json!({"id":c}))["views"][0].clone();
    raw["config"]["future"] = json!(true);
    let bytes = raw.to_string();
    f.db()
        .execute("UPDATE views SET json=?2 WHERE id=?1", params![v, bytes])
        .unwrap();
    f.ok(
        "entity.update",
        json!({"id":n["id"],"expected_revision":2,"document":d}),
    );
    assert!(
        !f.call("database.query", json!({"collection_id":c,"view_id":v}))
            .ok
    );
    assert_eq!(
        f.db()
            .query_row("SELECT json FROM views", [], |r| r.get::<_, String>(0))
            .unwrap(),
        bytes
    );
}

#[test]
fn export_search_and_complete_backup_do_not_materialize_referenced_records() {
    let f = Fixture::new();
    let c = f.collection();
    let v = f.view(&c);
    let mut d = doc();
    d["blocks"][0]["children"][0]["content"][0]["text"] = json!("Unique destination term");
    let original = f.ok(
        "record.create",
        json!({"collection_id":c,"title":"Target","document":d}),
    );
    let t = f.save(&c, definition(embed(&c, &v), json!({})));
    let one = f.instantiate(&t, json!({}));
    let two = f.instantiate(&t, json!({}));
    // Las instancias referencian una vista de su propia colección; no hay expansión recursiva.
    let ids = f.ok("database.query", json!({"collection_id":c,"view_id":v}))["records"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["entity"]["id"].clone())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    for format in ["json", "markdown"] {
        let destination = f.temp.path().join(format);
        let result = f.ok(
            "document.export",
            json!({"id":one["entity"]["id"],"format":format,"destination":destination}),
        );
        assert!(
            result["warnings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|w| w.as_str().unwrap().contains("transitivamente"))
        );
        let body = std::fs::read_to_string(destination.join(if format == "json" {
            "document.json"
        } else {
            "document.md"
        }))
        .unwrap();
        assert!(!body.contains("Unique destination term"));
        if format == "json" {
            assert_eq!(
                serde_json::from_str::<Value>(&body).unwrap(),
                one["entity"]["document"]
            );
        } else {
            assert!(body.contains(&v));
        }
    }
    f.ok("search.rebuild", json!({}));
    let hits = f.ok("search.query", json!({"query":"Unique destination"}));
    assert_eq!(hits["items"].as_array().unwrap().len(), 1);
    assert_eq!(hits["items"][0]["entity_id"], original["entity"]["id"]);
    let backup = f.temp.path().join("backup");
    f.ok("workspace.backup", json!({"destination":backup}));
    let restored = f.temp.path().join("restored");
    f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":restored}),
    );
    for (op, input, expected) in [
        ("template.get", json!({"id":t["id"]}), t),
        (
            "entity.get",
            json!({"id":one["entity"]["id"]}),
            one["entity"].clone(),
        ),
        (
            "entity.get",
            json!({"id":two["entity"]["id"]}),
            two["entity"].clone(),
        ),
    ] {
        let r = execute_json(
            json!({"workspace":restored,"command":{"operation":op,"input":input}}),
            "test",
        );
        assert!(r.ok);
        assert_eq!(r.data.unwrap(), expected);
    }
}
