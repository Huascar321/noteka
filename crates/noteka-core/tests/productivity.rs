use noteka_core::{contracts::Response, execute_json, workspace::Workspace};
use rusqlite::Connection;
use serde_json::{Value, json};

struct Fixture {
    _temp: tempfile::TempDir,
    path: std::path::PathBuf,
}
impl Fixture {
    fn new(tz: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        Workspace::create(&path, Some(tz.into())).unwrap();
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
    fn create(&self, kind: &str, title: &str, payload: Value) -> Value {
        self.ok(
            "entity.create",
            json!({"kind":kind,"title":title,"payload":payload}),
        )
    }
    fn query(&self, view: &str) -> Value {
        self.ok("workflow.query", json!({"view":view,"as_of":"2026-10-12"}))
    }
    fn db(&self) -> Connection {
        Connection::open(self.path.join("noteka.sqlite")).unwrap()
    }
}
fn error(r: Response, expected: &str) {
    assert!(!r.ok);
    assert_eq!(
        serde_json::to_value(r.error.unwrap().code).unwrap(),
        json!(expected)
    );
}
fn has(page: &Value, entity: &Value) -> bool {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["id"] == entity["id"])
}

#[test]
fn task_dates_priority_completion_and_atomic_rejections() {
    let f = Fixture::new("America/La_Paz");
    let p = f.create(
        "project",
        "CRE",
        json!({"purpose":"Resultado","start_date":"2026-10-01","end_date":"2026-10-31"}),
    );
    let task=f.ok("entity.create",json!({"kind":"task","title":"Hacer","payload":{"priority":"high","due_date":"2026-10-12","planned_date":"2026-10-11"},"context":{"target_id":p["id"],"kind":"project"}}));
    assert_eq!(task["payload"]["state"], "todo");
    assert_eq!(task["payload"]["inbox"], true);
    let done = f.ok(
        "entity.update",
        json!({"id":task["id"],"expected_revision":1,"payload":{"state":"done","inbox":false}}),
    );
    assert_eq!(done["id"], task["id"]);
    assert_eq!(done["payload"]["planned_date"], "2026-10-11");
    assert_eq!(
        f.ok("relation.list", json!({"entity_id":task["id"]}))["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for payload in [
        json!({"due_date":"2026-02-30"}),
        json!({"planned_date":"2026-10-12T00:00:00Z"}),
        json!({"planned_date":"2026-1-2"}),
        json!({"due_date":"0000-01-01"}),
        json!({"priority":"urgent"}),
        json!({"priority":2}),
        json!({"inbox":"false"}),
        json!({"unknown":true}),
    ] {
        error(f.call("entity.update",json!({"id":task["id"],"expected_revision":2,"title":"No escribir","payload":payload})),"validation_error");
        assert_eq!(f.ok("entity.get", json!({"id":task["id"]})), done);
    }
    error(f.call("entity.update",json!({"id":p["id"],"expected_revision":1,"payload":{"start_date":"2026-11-01","end_date":"2026-10-01"}})),"validation_error");
    assert_eq!(
        f.ok("changes.list", json!({"entity_id":task["id"]}))["items"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    f.create("task", "Leap", json!({"due_date":"2028-02-29"}));
}

#[test]
fn workflows_inbox_day_pending_overdue_and_archive_are_explicit() {
    let f = Fixture::new("UTC");
    let note = f.create("note", "Captura", json!({}));
    let today = f.create(
        "task",
        "Hoy",
        json!({"planned_date":"2026-10-12","due_date":"2026-10-12"}),
    );
    let overdue = f.create(
        "task",
        "Vencida",
        json!({"due_date":"2026-10-11","inbox":false}),
    );
    let future = f.create(
        "task",
        "Futura",
        json!({"planned_date":"2026-10-13","due_date":"2026-10-20"}),
    );
    let done = f.create(
        "task",
        "Hecha",
        json!({"state":"done","planned_date":"2026-10-12","due_date":"2026-10-10"}),
    );
    let inbox = f.query("inbox");
    assert!(has(&inbox, &note) && has(&inbox, &today));
    assert!(!has(&inbox, &overdue) && !has(&inbox, &done));
    let day = f.query("my_day");
    assert!(has(&day, &today));
    assert!(!has(&day, &overdue) && !has(&day, &future) && !has(&day, &done));
    let due = f.query("overdue");
    assert!(has(&due, &overdue));
    assert!(!has(&due, &today) && !has(&due, &done));
    assert!(has(&f.query("pending"), &today));
    f.ok(
        "entity.update",
        json!({"id":note["id"],"expected_revision":1,"payload":{"inbox":false}}),
    );
    assert!(!has(&f.query("inbox"), &note));
    f.ok(
        "entity.archive",
        json!({"id":today["id"],"expected_revision":1}),
    );
    for view in ["my_day", "inbox", "pending"] {
        assert!(!has(&f.query(view), &today));
    }
    f.ok(
        "entity.restore",
        json!({"id":today["id"],"expected_revision":2}),
    );
    assert!(has(&f.query("my_day"), &today));
    for (state, active) in [
        ("planned", true),
        ("doing", true),
        ("ongoing", true),
        ("on_hold", false),
        ("done", false),
    ] {
        let p = f.create("project", state, json!({"state":state}));
        assert_eq!(has(&f.query("active_projects"), &p), active);
    }
}

#[test]
fn civil_timezone_pagination_context_and_invalid_queries() {
    let f = Fixture::new("Pacific/Kiritimati");
    let date = chrono::Utc::now()
        .with_timezone(&chrono_tz::Pacific::Kiritimati)
        .date_naive()
        .to_string();
    assert_eq!(
        f.ok("workflow.query", json!({"view":"my_day"}))["date"],
        date
    );
    let project = f.create("project", "P", json!({}));
    let tag = f.create("tag", "Área", json!({"classification":"area"}));
    let person = f.create("person", "Persona", json!({}));
    let mut ids = vec![];
    for i in 0..7 {
        let e=f.ok("entity.create",json!({"kind":"note","title":format!("Nota {i}"),"context":{"target_id":project["id"],"kind":"project"}}));
        ids.push(e["id"].clone());
    }
    let mut cursor = Value::Null;
    let mut seen = vec![];
    loop {
        let page = f.ok(
            "workflow.query",
            json!({"view":"context","related_id":project["id"],"limit":2,"cursor":cursor}),
        );
        seen.extend(
            page["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| e["id"].clone()),
        );
        cursor = page["next_cursor"].clone();
        if cursor.is_null() {
            break;
        }
    }
    assert_eq!(seen, ids);
    f.ok(
        "relation.add",
        json!({"source_id":ids[0],"target_id":tag["id"],"kind":"tag","expected_revision":1}),
    );
    f.ok(
        "relation.add",
        json!({"source_id":ids[0],"target_id":person["id"],"kind":"person","expected_revision":2}),
    );
    for target in [&tag, &person] {
        assert_eq!(
            f.ok(
                "workflow.query",
                json!({"view":"context","related_id":target["id"]})
            )["items"][0]["id"],
            ids[0]
        );
    }
    for input in [
        json!({"view":"context"}),
        json!({"view":"pending","related_id":project["id"]}),
        json!({"view":"context","related_id":ids[0]}),
        json!({"view":"my_day","as_of":"2026-10-12Z"}),
        json!({"view":"inbox","limit":0}),
        json!({"view":"inbox","limit":201}),
        json!({"view":"inbox","cursor":"invalid"}),
    ] {
        error(f.call("workflow.query", input), "validation_error");
    }
}

#[test]
fn contextual_creation_rolls_back_and_legacy_payload_is_not_rewritten() {
    let f = Fixture::new("UTC");
    let foreign = Fixture::new("UTC");
    let other = foreign.create("project", "Ajeno", json!({}));
    let person = f.create("person", "Persona", json!({}));
    let before = f.ok("entity.list", json!({}));
    error(f.call("entity.create",json!({"kind":"note","title":"No crear","context":{"target_id":other["id"],"kind":"project"}})),"not_found");
    error(f.call("entity.create",json!({"kind":"note","title":"No crear","context":{"target_id":person["id"],"kind":"project"}})),"validation_error");
    assert_eq!(f.ok("entity.list", json!({})), before);
    let n = f.create("note", "Legacy", json!({}));
    f.db()
        .execute(
            "UPDATE entities SET payload='{}' WHERE id=?1",
            [n["id"].as_str().unwrap()],
        )
        .unwrap();
    assert_eq!(
        f.ok("entity.get", json!({"id":n["id"]}))["payload"],
        json!({})
    );
    assert!(has(&f.query("inbox"), &n));
    assert_eq!(
        f.ok("entity.get", json!({"id":n["id"]}))["payload"],
        json!({})
    );
    f.db()
        .execute(
            "UPDATE entities SET payload=?1 WHERE id=?2",
            rusqlite::params![
                json!({"future":{"keep":true}}).to_string(),
                n["id"].as_str().unwrap()
            ],
        )
        .unwrap();
    let updated = f.ok(
        "entity.update",
        json!({"id":n["id"],"expected_revision":1,"title":"Título"}),
    );
    assert_eq!(updated["payload"]["future"]["keep"], true);
    error(
        f.call(
            "entity.update",
            json!({"id":n["id"],"expected_revision":2,"payload":{"inbox":false}}),
        ),
        "validation_error",
    );
    assert_eq!(
        f.ok("entity.get", json!({"id":n["id"]}))["payload"]["future"]["keep"],
        true
    );
}
