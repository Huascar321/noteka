use noteka_core::{contracts::Response, execute_json, workspace::Workspace};
use rusqlite::Connection;
use serde_json::{Value, json};
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
    fn formula(&self, c: &str, name: &str, expression: Value) -> String {
        let bundle=self.ok("formula.set",json!({"collection_id":c,"expected_revision":self.rev(c),"name":name,"expression":expression}));
        bundle["properties"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["spec"]["name"] == name)
            .unwrap()["id"]
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
fn num(value: f64) -> Value {
    json!({"op":"number","value":value})
}
fn dec(value: &str) -> Value {
    json!({"op":"decimal","value":value})
}
fn prop(id: &str) -> Value {
    json!({"op":"property","property_id":id})
}
fn binary(operator: &str, left: Value, right: Value) -> Value {
    json!({"op":"binary","operator":operator,"left":left,"right":right})
}
fn calc_error(r: Response, code: &str) {
    assert!(!r.ok);
    assert_eq!(r.error.unwrap().details["calculation_error"]["code"], code);
}

#[test]
fn ast_types_short_circuit_null_exact_math_and_visible_errors() {
    let f = Fixture::new();
    let c = f.collection("Math");
    let p = f.property(&c, json!({"name":"Importe","type":"decimal"}));
    let n = f.property(&c, json!({"name":"Number","type":"number"}));
    let a = f.formula(&c, "Doble", binary("mul", prop(&p), dec("2")));
    let zero = f.formula(&c, "Error", binary("div", dec("1"), dec("0")));
    let guarded=f.formula(&c,"Condición",json!({"op":"if","condition":{"op":"boolean","value":true},"then":prop(&a),"otherwise":prop(&zero)}));
    let precision = f.formula(&c, "Tercio", binary("div", dec("1"), dec("3")));
    let huge = f.formula(
        &c,
        "Grande",
        binary("mul", dec("9999999999999999999999999999"), dec("1")),
    );
    let overflow = f.formula(
        &c,
        "Overflow",
        binary("add", dec("9999999999999999999999999999"), dec("1")),
    );
    let boolean = f.formula(
        &c,
        "Not",
        json!({"op":"not","value":binary("lt",prop(&n),num(3.))}),
    );
    let number = f.formula(&c, "Num", binary("div", prop(&n), num(2.)));
    let r = f.record(&c, "Entrada", json!({p.clone():"2.25",n.clone():4}));
    assert_eq!(r["values"][&a], "4.5");
    assert_eq!(r["values"][&guarded], "4.5");
    assert_eq!(r["values"][&number], 2.0);
    assert_eq!(r["values"][&boolean], true);
    assert_eq!(r["values"][&huge], "9999999999999999999999999999");
    for (id, code) in [
        (&zero, "division_by_zero"),
        (&precision, "precision_loss"),
        (&overflow, "overflow"),
    ] {
        assert_eq!(r["calculation_errors"][id]["code"], code);
        assert!(r["values"].get(id).is_none());
    }
    let null = f.record(&c, "Vacía", json!({}));
    assert_eq!(null["values"][&a], Value::Null);
    assert_eq!(null["values"][&boolean], Value::Null);
    calc_error(
        f.call(
            "database.query",
            json!({"collection_id":c,"config":{"sorts":[{"property_id":zero}]}}),
        ),
        "division_by_zero",
    );
    calc_error(f.call("database.query", json!({"collection_id":c,"config":{"filters":[{"property_id":zero,"op":"eq","value":"0"}]}})), "division_by_zero");
    let filtered = f.ok("database.query", json!({"collection_id":c,"config":{"filters":[{"property_id":a,"op":"eq","value":"4.500"}]}}));
    assert_eq!(filtered["records"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        filtered["records"]["items"][0]["entity"]["id"],
        r["entity"]["id"]
    );
    let text = f.formula(
        &c,
        "Texto calculado",
        json!({"op":"text","value":"texto local"}),
    );
    let text_filtered=f.ok("database.query",json!({"collection_id":c,"config":{"filters":[{"property_id":text,"op":"contains","value":"local"}]}}));
    assert_eq!(
        text_filtered["records"]["items"].as_array().unwrap().len(),
        2
    );
    assert!(!f.call("record.update",json!({"collection_id":c,"entity_id":r["entity"]["id"],"expected_revision":1,"values":{a.clone():"99"}})).ok);
    let before = f.get(&c, &r);
    assert_eq!(before["entity"]["revision"], 1);
    assert_eq!(before["values"][&p], "2.25");
}

#[test]
fn boolean_short_circuit_and_missing_values_have_explicit_semantics() {
    let f = Fixture::new();
    let c = f.collection("Boolean");
    let flag = f.property(&c, json!({"name":"Flag","type":"checkbox"}));
    let failed_boolean = binary("eq", binary("div", num(1.), num(0.)), num(1.));
    let and = f.formula(
        &c,
        "And",
        binary(
            "and",
            json!({"op":"boolean","value":false}),
            failed_boolean.clone(),
        ),
    );
    let or = f.formula(
        &c,
        "Or",
        binary(
            "or",
            json!({"op":"boolean","value":true}),
            failed_boolean.clone(),
        ),
    );
    let null = f.formula(
        &c,
        "Null",
        binary("and", prop(&flag), failed_boolean.clone()),
    );
    let branch = f.formula(&c,"Rama",json!({"op":"if","condition":prop(&flag),"then":failed_boolean,"otherwise":{"op":"boolean","value":false}}));
    let r = f.record(&c, "Vacío", json!({}));
    assert_eq!(r["values"][&and], false);
    assert_eq!(r["values"][&or], true);
    assert_eq!(r["values"][&null], Value::Null);
    assert_eq!(r["values"][&branch], Value::Null);
    assert_eq!(r["calculation_errors"], json!({}));
}

#[test]
fn persisted_metadata_mismatch_and_malformed_inverse_are_not_valid_results() {
    let f = Fixture::new();
    let a = f.collection("Sources");
    let b = f.collection("Targets");
    let p = f.formula(&a, "Amount", dec("1.25"));
    let rel = f.property(
        &a,
        json!({"name":"Related","type":"relation","target_collection_id":b}),
    );
    let inverse = f.formula(
        &b,
        "Count",
        json!({"op":"rollup","relation_id":rel,"aggregate":"count","inverse":true}),
    );
    let target = f.record(&b, "Target", json!({}));
    let source = f.record(&a, "Source", json!({rel.clone():[target["entity"]["id"]]}));
    f.db().execute("UPDATE properties SET spec=json_set(spec,'$.calculation_type.kind','number') WHERE id=?1",[&p]).unwrap();
    assert_eq!(
        f.get(&a, &source)["calculation_errors"][&p]["code"],
        "type_error"
    );
    let before = f.ok("collection.get", json!({"id":a}));
    calc_error(f.call("formula.set",json!({"collection_id":a,"expected_revision":before["collection"]["revision"],"name":"New","expression":num(1.)})),"type_error");
    assert_eq!(f.ok("collection.get", json!({"id":a})), before);
    f.db()
        .execute(
            "UPDATE property_values SET json=?1 WHERE entity_id=?2 AND property_id=?3",
            rusqlite::params![
                json!([target["entity"]["id"], target["entity"]["id"]]).to_string(),
                source["entity"]["id"].as_str().unwrap(),
                rel
            ],
        )
        .unwrap();
    let result = f.get(&b, &target);
    assert_eq!(
        result["calculation_errors"][&inverse]["code"],
        "invalid_value"
    );
    assert!(result["values"].get(&inverse).is_none());
    assert_eq!(result["entity"]["revision"], 1);
}

#[test]
fn dependency_height_is_bounded_independently_of_graph_traversal_order() {
    let f = Fixture::new();
    let a = f.collection("Chain A");
    let b = f.collection("Chain B");
    let ab = f.property(
        &a,
        json!({"name":"B","type":"relation","target_collection_id":b}),
    );
    let ba = f.property(
        &b,
        json!({"name":"A","type":"relation","target_collection_id":a}),
    );
    let mut previous = f.formula(&a, "Root", num(1.));
    for i in 1..=128 {
        let (collection, relation) = if i % 2 == 1 { (&b, &ba) } else { (&a, &ab) };
        previous = f.formula(
            collection,
            &format!("Level {i}"),
            json!({"op":"rollup","relation_id":relation,"property_id":previous,"aggregate":"sum"}),
        );
    }
    let before = f.ok("collection.get", json!({"id":b}));
    calc_error(f.call("formula.set",json!({"collection_id":b,"expected_revision":before["collection"]["revision"],"name":"Too deep","expression":{"op":"rollup","relation_id":ba,"property_id":previous,"aggregate":"sum"}})),"complexity_limit");
    assert_eq!(f.ok("collection.get", json!({"id":b})), before);
}
#[test]
fn invalid_types_currency_cycles_limits_and_old_writers_leave_schema_unchanged() {
    let f = Fixture::new();
    let c = f.collection("Seguro");
    let p = f.formula(&c, "A", num(1.));
    let b = f.formula(&c, "B", prop(&p));
    let before = f.ok("collection.get", json!({"id":c}));
    for expression in [
        binary("add", num(1.), dec("1")),
        binary(
            "add",
            json!({"op":"decimal","value":"1","currency":"BOB"}),
            json!({"op":"decimal","value":"1","currency":"USD"}),
        ),
        binary(
            "mul",
            json!({"op":"decimal","value":"1","currency":"BOB"}),
            json!({"op":"decimal","value":"1","currency":"BOB"}),
        ),
    ] {
        calc_error(f.call("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"name":"Inválida","expression":expression})),"type_error");
    }
    calc_error(f.call("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"property_id":p,"name":"A","expression":prop(&b)})),"cycle");
    calc_error(f.call("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"name":"Ajena","expression":prop("missing")})),"missing_reference");
    let mut depth = json!({"op":"boolean","value":true});
    for _ in 0..34 {
        depth = json!({"op":"not","value":depth});
    }
    let deep=f.call("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"name":"Profunda","expression":depth}));
    assert!(!deep.ok);
    assert!(deep.error.unwrap().message.contains("profundidad"));
    let mut wide = num(1.);
    for _ in 0..8 {
        wide = binary("add", wide.clone(), wide);
    }
    calc_error(f.call("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"name":"Excesiva","expression":wide})),"complexity_limit");
    let invalid=f.call("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"name":"Shell","expression":{"op":"shell","command":"rm -rf /"}}));
    assert!(!invalid.ok);
    assert_eq!(f.ok("collection.get", json!({"id":c})), before);
    assert_eq!(serde_json::to_value(f.call("formula.set",json!({"collection_id":c,"expected_revision":1,"property_id":p,"name":"A","expression":num(2.)})).error.unwrap().code).unwrap(),"revision_conflict");
    let money = f.formula(
        &c,
        "Moneda cociente",
        binary(
            "div",
            json!({"op":"decimal","value":"2","currency":"BOB"}),
            json!({"op":"decimal","value":"2","currency":"BOB"}),
        ),
    );
    assert_eq!(f.record(&c, "Evaluación", json!({}))["values"][&money], "1");
}
#[test]
fn rollups_direct_inverse_empty_error_and_related_recalculation() {
    let f = Fixture::new();
    let a = f.collection("Sources");
    let b = f.collection("Targets");
    let n = f.property(&b, json!({"name":"Valor","type":"number"}));
    let rel = f.property(
        &a,
        json!({"name":"Related","type":"relation","target_collection_id":b}),
    );
    let mut calculations = vec![];
    for aggregate in ["count", "sum", "avg", "min", "max"] {
        let expr = json!({"op":"rollup","relation_id":rel,"aggregate":aggregate,"property_id":if aggregate=="count"{Value::Null}else{json!(n)}});
        calculations.push((aggregate, f.formula(&a, aggregate, expr)));
    }
    let one = f.record(&b, "One", json!({n.clone():2}));
    let two = f.record(&b, "Two", json!({n.clone():4}));
    let empty = f.record(&b, "Empty", json!({}));
    let r = f.record(
        &a,
        "Parent",
        json!({rel.clone():[one["entity"]["id"],two["entity"]["id"],empty["entity"]["id"]]}),
    );
    for ((_, id), value) in calculations.iter().zip([3., 6., 3., 2., 4.]) {
        assert_eq!(r["values"][id], value);
    }
    let inverse = f.formula(
        &b,
        "Inversa",
        json!({"op":"rollup","relation_id":rel,"aggregate":"count","inverse":true}),
    );
    assert_eq!(f.get(&b, &one)["values"][&inverse], 1);
    f.ok("record.update",json!({"collection_id":b,"entity_id":one["entity"]["id"],"expected_revision":1,"values":{n.clone():10}}));
    assert_eq!(f.get(&a, &r)["values"][&calculations[1].1], 14.);
    assert_eq!(f.get(&a, &r)["entity"]["revision"], 1);
    f.ok(
        "entity.archive",
        json!({"id":two["entity"]["id"],"expected_revision":1}),
    );
    assert_eq!(f.get(&a, &r)["values"][&calculations[1].1], 14.);
    let no = f.record(&a, "Sin", json!({}));
    assert_eq!(no["values"][&calculations[0].1], 0);
    assert_eq!(no["values"][&calculations[1].1], 0);
    assert_eq!(no["values"][&calculations[2].1], Value::Null);
    let source_error = f.formula(&b, "Falla", binary("div", num(1.), num(0.)));
    let agg_error = f.formula(
        &a,
        "No omitir error",
        json!({"op":"rollup","relation_id":rel,"property_id":source_error,"aggregate":"sum"}),
    );
    assert_eq!(
        f.get(&a, &r)["calculation_errors"][&agg_error]["code"],
        "division_by_zero"
    );
    f.ok("record.update",json!({"collection_id":a,"entity_id":r["entity"]["id"],"expected_revision":1,"values":{rel.clone():[]}}));
    assert_eq!(f.get(&b, &one)["values"][&inverse], 0);
}
#[test]
fn subscription_fixture_exact_remainder_allocation_and_cost_changes() {
    let f = Fixture::new();
    let subscriptions = f.collection("Subscriptions");
    let members = f.collection("Members");
    let cost = f.property(
        &subscriptions,
        json!({"name":"Cost","type":"money","currency":"BOB"}),
    );
    let seats = f.property(&members, json!({"name":"Seats","type":"number"}));
    let paid = f.property(&members, json!({"name":"Paid","type":"checkbox"}));
    let rel = f.property(
        &members,
        json!({"name":"Subscription","type":"relation","target_collection_id":subscriptions}),
    );
    let sub = f.record(&subscriptions, "Plan", json!({cost.clone():"145.69"}));
    let amount = f.formula(
        &members,
        "Amount",
        json!({"op":"allocate","relation_id":rel,"cost_id":cost,"weight_id":seats,"scale":2}),
    );
    let mut records = vec![];
    for i in 0..5 {
        records.push(f.record(&members,&format!("Member {i}"),json!({seats.clone():if i==0{2}else{1},paid.clone():false,rel.clone():[sub["entity"]["id"]]})));
    }
    let query = f.ok("database.query", json!({"collection_id":members}));
    let mut total = 0;
    for r in query["records"]["items"].as_array().unwrap() {
        let amount = r["values"][&amount].as_str().unwrap();
        assert_eq!(
            amount,
            if r["values"][&seats] == 2 {
                "48.57"
            } else {
                "24.28"
            }
        );
        let (whole, fraction) = amount.split_once('.').unwrap();
        total += whole.parse::<i32>().unwrap() * 100 + fraction.parse::<i32>().unwrap();
    }
    assert_eq!(total, 14569);
    f.ok("record.update",json!({"collection_id":subscriptions,"entity_id":sub["entity"]["id"],"expected_revision":1,"values":{cost.clone():"145.70"}}));
    let current = f.ok("database.query", json!({"collection_id":members}));
    let ids = current["records"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["values"][&seats] == 1)
        .map(|r| r["entity"]["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    let smallest = *ids.iter().min().unwrap();
    for r in current["records"]["items"].as_array().unwrap() {
        assert_eq!(r["entity"]["revision"], 1);
        assert_eq!(
            r["values"][&amount],
            if r["values"][&seats] == 2 {
                "48.57"
            } else if r["entity"]["id"] == smallest {
                "24.29"
            } else {
                "24.28"
            }
        );
    }
    f.ok("record.update",json!({"collection_id":members,"entity_id":records[0]["entity"]["id"],"expected_revision":1,"values":{seats.clone():0}}));
    assert_eq!(
        f.get(&members, &records[0])["calculation_errors"][&amount]["code"],
        "invalid_value"
    );
    assert_eq!(f.get(&subscriptions, &sub)["values"][&cost], "145.70");
}
#[test]
fn formula_backups_migration_unknown_ast_and_failed_set_are_recoverable() {
    let f = Fixture::new();
    f.db()
        .execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations")
        .unwrap();
    f.db().execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;DROP TABLE templates;DROP TABLE calculations;UPDATE workspace_meta SET schema_version=4;PRAGMA user_version=4;").unwrap();
    assert_eq!(
        f.ok("workspace.open", json!({}))["schema_version"],
        noteka_core::workspace::SCHEMA_VERSION
    );
    let c = f.collection("Backup");
    let p = f.formula(&c, "Calculation", dec("1.25"));
    let r = f.record(&c, "Original", json!({}));
    let backup = f.temp.path().join("backup");
    f.ok("workspace.backup", json!({"destination":backup}));
    let restore = f.temp.path().join("restore");
    f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":restore}),
    );
    assert_eq!(execute_json(json!({"workspace":restore,"command":{"operation":"record.get","input":{"collection_id":c,"entity_id":r["entity"]["id"]}}}),"test").data.unwrap(),r);
    let before = f.ok("collection.get", json!({"id":c}));
    f.db().execute_batch("CREATE TRIGGER fail_calc BEFORE UPDATE ON collections BEGIN SELECT RAISE(ABORT,'injected');END;").unwrap();
    assert!(!f.call("formula.set",json!({"collection_id":c,"expected_revision":f.rev(&c),"property_id":p,"name":"Calculation","expression":dec("2.5")})).ok);
    assert_eq!(f.ok("collection.get", json!({"id":c})), before);
    f.db().execute_batch("DROP TRIGGER fail_calc").unwrap();
    let future = json!({"op":"decimal","value":"1.25","future":{"keep":42}}).to_string();
    f.db()
        .execute(
            "UPDATE calculations SET expression=?1 WHERE property_id=?2",
            rusqlite::params![future, p],
        )
        .unwrap();
    assert_eq!(
        f.get(&c, &r)["calculation_errors"][&p]["code"],
        "invalid_syntax"
    );
    calc_error(f.call("formula.set",json!({"collection_id":c,"expected_revision":before["collection"]["revision"],"property_id":p,"name":"Calculation","expression":dec("2.5")})),"invalid_syntax");
    let bundle_error = f.call("collection.get", json!({"id":c}));
    assert!(!bundle_error.ok);
    assert!(
        bundle_error
            .error
            .unwrap()
            .message
            .contains("no se convirtió")
    );
    let unknown_backup = f.temp.path().join("unknown-backup");
    f.ok("workspace.backup", json!({"destination":unknown_backup}));
    let unknown_restore = f.temp.path().join("unknown-restore");
    f.ok(
        "workspace.restore",
        json!({"backup":unknown_backup,"destination":unknown_restore}),
    );
    let restored = Connection::open(unknown_restore.join("noteka.sqlite")).unwrap();
    assert_eq!(
        restored
            .query_row(
                "SELECT expression FROM calculations WHERE property_id=?1",
                [&p],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        future
    );
    assert_eq!(
        f.db()
            .query_row(
                "SELECT expression FROM calculations WHERE property_id=?1",
                [p],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        future
    );
}
