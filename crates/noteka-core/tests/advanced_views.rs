use noteka_core::{contracts::Response, execute_json, workspace::Workspace};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::collections::BTreeSet;
struct Fixture {
    temp: tempfile::TempDir,
    path: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        Workspace::create(&path, Some("Pacific/Honolulu".into())).unwrap();
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
        self.ok("collection.create", json!({"name":"Views"}))["collection"]["id"]
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
    fn query(&self, c: &str, layout: &str, config: Value) -> Value {
        self.ok(
            "database.query",
            json!({"collection_id":c,"layout":layout,"config":config}),
        )
    }
    fn db(&self) -> Connection {
        Connection::open(self.path.join("noteka.sqlite")).unwrap()
    }
}
fn atom(id: &str, op: &str, value: Value) -> Value {
    json!({"op":"condition","filter":{"property_id":id,"op":op,"value":value}})
}
fn ids(page: &Value) -> BTreeSet<String> {
    page["records"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["entity"]["id"].as_str().unwrap().into())
        .collect()
}
fn invalid(r: Response) {
    assert!(!r.ok);
    assert_eq!(
        serde_json::to_value(r.error.unwrap().code).unwrap(),
        "validation_error"
    );
}

#[test]
fn compound_filters_exact_null_types_limits_and_visible_calculation_errors() {
    let f = Fixture::new();
    let c = f.collection();
    let money = f.property(&c, json!({"name":"Cost","type":"money","currency":"BOB"}));
    let paid = f.property(&c, json!({"name":"Paid","type":"checkbox"}));
    let number = f.property(&c, json!({"name":"Number","type":"number"}));
    let a = f.record(
        &c,
        "A",
        json!({money.clone():"1",paid.clone():false,number.clone():0}),
    );
    f.record(
        &c,
        "B",
        json!({money.clone():"2.000000001",paid.clone():true,number.clone():1}),
    );
    let high = f.record(
        &c,
        "C",
        json!({money.clone():"10",paid.clone():false,number.clone():2}),
    );
    let empty = f.record(&c, "Empty", json!({}));
    let tree = json!({"op":"all","children":[{"op":"any","children":[atom(&money,"lt",json!("2")),atom(&money,"ge",json!("10.000"))]},{"op":"not","child":atom(&number,"gt",json!(2))}]});
    let cfg = json!({"filters":[{"property_id":paid,"op":"eq","value":false}],"filter_tree":tree,"sorts":[{"property_id":money}]});
    let page = f.query(&c, "table", cfg.clone());
    assert_eq!(page["matching_count"], 2);
    assert_eq!(
        page["records"]["items"][0]["entity"]["id"],
        a["entity"]["id"]
    );
    assert_eq!(
        page["records"]["items"][1]["entity"]["id"],
        high["entity"]["id"]
    );
    let not = f.query(
        &c,
        "table",
        json!({"filter_tree":{"op":"not","child":atom(&money,"gt",json!("0"))}}),
    );
    assert_eq!(
        ids(&not),
        BTreeSet::from([empty["entity"]["id"].as_str().unwrap().into()])
    );
    let missing = f.query(
        &c,
        "table",
        json!({"filter_tree":atom(&paid,"is_empty",Value::Null)}),
    );
    assert_eq!(ids(&missing), ids(&not));
    for (op, count) in [("lt", 1), ("le", 2), ("gt", 1), ("ge", 2), ("ne", 3)] {
        assert_eq!(
            f.query(
                &c,
                "table",
                json!({"filter_tree":atom(&money,op,json!("2.000000001"))})
            )["matching_count"],
            count
        );
    }
    let mut deep = atom(&paid, "eq", json!(false));
    for _ in 0..9 {
        deep = json!({"op":"not","child":deep});
    }
    let mut many = atom(&paid, "eq", json!(false));
    for _ in 0..5 {
        many = json!({"op":"any","children":[many.clone(),many]});
    }
    many = json!({"op":"not","child":{"op":"not","child":many}});
    for tree in [
        deep,
        many,
        json!({"op":"all","children":[]}),
        atom(&paid, "gt", json!(false)),
        atom(&money, "ge", json!(2)),
        atom(&number, "lt", Value::Null),
        json!({"op":"any","children":[atom(&paid,"eq",json!(false)),atom("foreign","eq",json!(1))]}),
    ] {
        invalid(f.call(
            "database.query",
            json!({"collection_id":c,"config":{"filter_tree":tree}}),
        ));
    }
    let rev = f.ok("collection.get", json!({"id":c}))["collection"]["revision"].clone();
    let bundle=f.ok("formula.set",json!({"collection_id":c,"expected_revision":rev,"name":"Failed","expression":{"op":"binary","operator":"div","left":{"op":"decimal","value":"1"},"right":{"op":"decimal","value":"0"}}}));
    let failed = bundle["properties"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["spec"]["name"] == "Failed")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let error=f.call("database.query",json!({"collection_id":c,"config":{"filter_tree":{"op":"any","children":[atom(&paid,"eq",json!(false)),atom(failed,"eq",json!("0"))]}}}));
    assert_eq!(
        error.error.unwrap().details["calculation_error"]["code"],
        "division_by_zero"
    );
    assert_eq!(
        f.ok(
            "record.get",
            json!({"collection_id":c,"entity_id":a["entity"]["id"]})
        )["entity"]["revision"],
        1
    );
}

#[test]
fn filter_boundaries_and_successful_calculated_conditions_are_deterministic() {
    let f = Fixture::new();
    let c = f.collection();
    let number = f.property(&c, json!({"name":"Number","type":"number"}));
    f.record(&c, "Zero", json!({number.clone():0}));
    let one = f.record(&c, "One", json!({number.clone():1}));
    let rev = f.ok("collection.get", json!({"id":c}))["collection"]["revision"].clone();
    let bundle = f.ok("formula.set", json!({"collection_id":c,"expected_revision":rev,"name":"Double","expression":{"op":"binary","operator":"mul","left":{"op":"property","property_id":number},"right":{"op":"number","value":2}}}));
    let calc = bundle["properties"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["spec"]["name"] == "Double")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let config = json!({"filter_tree":{"op":"all","children":[atom(calc,"ge",json!(2)),{"op":"any","children":[atom(&number,"is_empty",Value::Null),atom(&number,"eq",json!(1))]}]},"sorts":[{"property_id":calc}]});
    let page = f.query(&c, "gallery", config);
    assert_eq!(
        ids(&page),
        BTreeSet::from([one["entity"]["id"].as_str().unwrap().into()])
    );
    assert_eq!(
        page["records"]["items"][0]["values"][calc].as_f64(),
        Some(2.0)
    );
    let condition = atom(&number, "ge", json!(0));
    let mut deep = condition.clone();
    for _ in 0..8 {
        deep = json!({"op":"not","child":deep});
    }
    assert_eq!(
        f.query(&c, "table", json!({"filter_tree":deep}))["matching_count"],
        2
    );
    let legacy = vec![json!({"property_id":number,"op":"ge","value":0}); 16];
    let tree = json!({"op":"all","children":vec![condition.clone();16]});
    assert_eq!(
        f.query(&c, "table", json!({"filters":legacy,"filter_tree":tree}))["matching_count"],
        2
    );
    invalid(f.call("database.query",json!({"collection_id":c,"config":{"filters":legacy,"filter_tree":{"op":"all","children":vec![condition.clone();17]}}})));
    // 32 hojas, 31 grupos binarios y un NOT: 64 nodos válidos; el 65 falla.
    let mut binary = condition;
    for _ in 0..5 {
        binary = json!({"op":"all","children":[binary.clone(),binary]});
    }
    let exact = json!({"op":"not","child":binary});
    assert_eq!(
        f.query(&c, "table", json!({"filter_tree":exact}))["matching_count"],
        0
    );
    invalid(f.call(
        "database.query",
        json!({"collection_id":c,"config":{"filter_tree":{"op":"not","child":exact}}}),
    ));
}

#[test]
fn board_declared_empty_groups_null_checkbox_and_updates_share_ids() {
    let f = Fixture::new();
    let c = f.collection();
    let state = f.property(
        &c,
        json!({"name":"State","type":"select","options":["Doing","Planned","Done"]}),
    );
    let flag = f.property(&c, json!({"name":"Flag","type":"checkbox"}));
    let r = f.record(
        &c,
        "Original",
        json!({state.clone():"Doing",flag.clone():false}),
    );
    let no = f.record(&c, "Unset", json!({}));
    let page = f.query(&c, "board", json!({"group_by":state}));
    assert_eq!(
        page["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| g["key"].clone())
            .collect::<Vec<_>>(),
        vec![json!("Doing"), json!("Planned"), json!("Done"), Value::Null]
    );
    assert_eq!(page["groups"][0]["entity_ids"], json!([r["entity"]["id"]]));
    assert_eq!(page["groups"][1]["entity_ids"], json!([]));
    assert_eq!(page["groups"][3]["entity_ids"], json!([no["entity"]["id"]]));
    let flags = f.query(&c, "board", json!({"group_by":flag}));
    assert_eq!(flags["groups"][0]["key"], false);
    assert_eq!(flags["groups"][0]["entity_ids"], json!([r["entity"]["id"]]));
    assert_eq!(
        flags["groups"][2]["entity_ids"],
        json!([no["entity"]["id"]])
    );
    f.ok("record.update",json!({"collection_id":c,"entity_id":r["entity"]["id"],"expected_revision":1,"values":{state.clone():"Done"}}));
    let updated = f.query(&c, "board", json!({"group_by":state}));
    assert_eq!(updated["groups"][0]["entity_ids"], json!([]));
    assert_eq!(
        updated["groups"][2]["entity_ids"],
        json!([r["entity"]["id"]])
    );
    assert_eq!(
        f.ok(
            "record.get",
            json!({"collection_id":c,"entity_id":no["entity"]["id"]})
        ),
        no
    );
}

#[test]
fn calendar_civil_leap_ranges_extreme_months_and_invalid_configs() {
    let f = Fixture::new();
    let c = f.collection();
    let date = f.property(&c, json!({"name":"Date","type":"date"}));
    let range = f.property(&c, json!({"name":"Range","type":"date_range"}));
    let one = f.record(
        &c,
        "Start",
        json!({date.clone():"2024-02-01",range.clone():{"start":"2024-01-31","end":"2024-02-02"}}),
    );
    let last = f.record(
        &c,
        "Leap",
        json!({date.clone():"2024-02-29",range.clone():{"start":"2024-02-29","end":"2024-03-02"}}),
    );
    f.record(
        &c,
        "Outside",
        json!({date.clone():"2024-03-01",range.clone():{"start":"2024-03-01","end":"2024-03-02"}}),
    );
    let empty = f.record(&c, "No date", json!({}));
    for p in [&date, &range] {
        let page = f.query(
            &c,
            "calendar",
            json!({"calendar":{"property_id":p,"month":"2024-02"}}),
        );
        assert_eq!(page["matching_count"], 3);
        assert_eq!(page["groups"].as_array().unwrap().len(), 30);
        assert_eq!(
            page["groups"][0]["entity_ids"],
            json!([one["entity"]["id"]])
        );
        assert_eq!(
            page["groups"][28]["entity_ids"],
            json!([last["entity"]["id"]])
        );
        assert_eq!(
            page["groups"][29]["entity_ids"],
            json!([empty["entity"]["id"]])
        );
        if p == &range {
            assert_eq!(
                page["groups"][1]["entity_ids"],
                json!([one["entity"]["id"]])
            );
        }
    }
    for (month, days) in [
        ("1900-02", 28),
        ("2000-02", 29),
        ("0001-01", 31),
        ("9999-12", 31),
    ] {
        let page = f.query(
            &c,
            "calendar",
            json!({"calendar":{"property_id":date,"month":month}}),
        );
        assert_eq!(page["groups"].as_array().unwrap().len(), days + 1);
    }
    let other = f.collection();
    let foreign = f.property(&other, json!({"name":"Date","type":"date"}));
    for calendar in [
        json!({"property_id":date,"month":"0000-01"}),
        json!({"property_id":date,"month":"2024-13"}),
        json!({"property_id":date,"month":"2024-2"}),
        json!({"property_id":date,"month":"2024-é"}),
        json!({"property_id":foreign,"month":"2024-02"}),
    ] {
        invalid(f.call("view.save",json!({"collection_id":c,"name":"Bad","layout":"calendar","config":{"calendar":calendar}})));
    }
    for (layout, config) in [
        ("board", json!({"group_by":date})),
        ("calendar", json!({})),
        ("gallery", json!({"group_by":date})),
        (
            "table",
            json!({"calendar":{"property_id":date,"month":"2024-02"}}),
        ),
    ] {
        invalid(f.call(
            "database.query",
            json!({"collection_id":c,"layout":layout,"config":config}),
        ));
    }
}

#[test]
fn every_layout_paginates_unique_records_and_groups_reference_only_page() {
    let f = Fixture::new();
    let c = f.collection();
    let flag = f.property(&c, json!({"name":"Flag","type":"checkbox"}));
    let date = f.property(&c, json!({"name":"Date","type":"date"}));
    for i in 0..5 {
        f.record(
            &c,
            &format!("Record {i}"),
            json!({flag.clone():false,date.clone():"2026-10-12"}),
        );
    }
    for (layout, cfg) in [
        ("table", json!({})),
        ("list", json!({})),
        ("gallery", json!({})),
        ("board", json!({"group_by":flag})),
        (
            "calendar",
            json!({"calendar":{"property_id":date,"month":"2026-10"}}),
        ),
    ] {
        let view = f.ok(
            "view.save",
            json!({"collection_id":c,"name":layout,"layout":layout,"config":cfg}),
        );
        let mut seen = BTreeSet::new();
        let mut cursor = Value::Null;
        loop {
            let page = f.ok(
                "database.query",
                json!({"collection_id":c,"view_id":view["id"],"limit":2,"cursor":cursor}),
            );
            assert_eq!(page["layout"], layout);
            assert_eq!(page["matching_count"], 5);
            let current = ids(&page);
            for id in &current {
                assert!(seen.insert(id.clone()));
            }
            for group in page["groups"].as_array().unwrap() {
                for id in group["entity_ids"].as_array().unwrap() {
                    assert!(current.contains(id.as_str().unwrap()));
                }
            }
            cursor = page["records"]["next_cursor"].clone();
            if cursor.is_null() {
                break;
            }
        }
        assert_eq!(seen.len(), 5);
        invalid(f.call(
            "database.query",
            json!({"collection_id":c,"view_id":view["id"],"layout":"table"}),
        ));
    }
}

#[test]
fn advanced_views_revision_conflicts_unknown_format_and_backup_preserve_raw() {
    let f = Fixture::new();
    let c = f.collection();
    let original = f.record(&c, "Original", json!({}));
    let v = f.ok(
        "view.save",
        json!({"collection_id":c,"name":"Gallery","layout":"gallery","config":{}}),
    );
    let updated=f.ok("view.save",json!({"collection_id":c,"id":v["id"],"expected_revision":1,"name":"Gallery 2","layout":"gallery","config":{}}));
    let conflict=f.call("view.save",json!({"collection_id":c,"id":v["id"],"expected_revision":1,"name":"Old","layout":"table","config":{}}));
    assert_eq!(
        serde_json::to_value(conflict.error.unwrap().code).unwrap(),
        "revision_conflict"
    );
    for i in 0..3 {
        let mut future = updated.clone();
        if i == 0 {
            future["config"]["future"] = json!({"keep":42});
        } else if i == 1 {
            future["layout"] = json!("future_layout");
        } else {
            future["config"]["filter_tree"] = json!({"op":"future_filter","keep":42});
        }
        let raw = future.to_string();
        f.db()
            .execute(
                "UPDATE views SET json=?1 WHERE id=?2",
                params![raw, v["id"].as_str().unwrap()],
            )
            .unwrap();
        assert!(!f.call("collection.get", json!({"id":c})).ok);
        assert!(!f.call("view.save",json!({"collection_id":c,"id":v["id"],"expected_revision":2,"name":"Do not overwrite","layout":"table","config":{}})).ok);
        let backup = f.temp.path().join(format!("backup-{i}"));
        let restore = f.temp.path().join(format!("restore-{i}"));
        f.ok("workspace.backup", json!({"destination":backup}));
        f.ok(
            "workspace.restore",
            json!({"backup":backup,"destination":restore}),
        );
        let restored = Connection::open(restore.join("noteka.sqlite")).unwrap();
        assert_eq!(
            restored
                .query_row(
                    "SELECT json FROM views WHERE id=?1",
                    [v["id"].as_str().unwrap()],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            raw
        );
        assert_eq!(
            f.db()
                .query_row(
                    "SELECT json FROM views WHERE id=?1",
                    [v["id"].as_str().unwrap()],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            raw
        );
        assert_eq!(
            f.ok(
                "record.get",
                json!({"collection_id":c,"entity_id":original["entity"]["id"]})
            ),
            original
        );
    }
}
