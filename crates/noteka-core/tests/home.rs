use noteka_core::execute_json;
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::collections::BTreeSet;

const VIEWS: [&str; 15] = [
    "tasks_today",
    "tasks_inbox",
    "tasks_all",
    "tasks_done",
    "notes_all",
    "notes_inbox",
    "projects_active",
    "projects_planned",
    "projects_board",
    "tags_recent",
    "tags_alphabetical",
    "tags_types",
    "people_contacts",
    "people_new",
    "people_updated",
];
struct F {
    temp: tempfile::TempDir,
    path: std::path::PathBuf,
}
impl F {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let f = Self { temp, path };
        f.ok("workspace.create", json!({"timezone":"Pacific/Kiritimati"}));
        f
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
    fn create(&self, kind: &str, title: &str, payload: Value) -> Value {
        self.ok(
            "entity.create",
            json!({"kind":kind,"title":title,"payload":payload}),
        )
    }
    fn query(&self, v: &str) -> Value {
        self.ok("home.query", json!({"view":v,"as_of":"2026-10-12"}))
    }
    fn db(&self) -> Connection {
        Connection::open(self.path.join("noteka.sqlite")).unwrap()
    }
    fn link(&self, source: &Value, target: &Value, kind: &str) {
        let s = self.ok("entity.get", json!({"id":source["id"]}));
        self.ok("relation.add",json!({"source_id":s["id"],"target_id":target["id"],"kind":kind,"expected_revision":s["revision"]}));
    }
    fn raw(&self, e: &Value, p: Value) {
        self.db()
            .execute(
                "UPDATE entities SET payload=?2 WHERE id=?1",
                params![e["id"].as_str().unwrap(), p.to_string()],
            )
            .unwrap();
    }
    fn seed(&self, kind: &str, n: usize, p: Value) -> Vec<String> {
        let mut db = self.db();
        let tx = db.transaction().unwrap();
        let mut ids = vec![];
        for i in 0..n {
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute("INSERT INTO entities VALUES (?1,?2,?3,1,NULL,'2026-10-11T12:00:00.000000Z','2026-10-11T12:00:00.000000Z',?4)",params![id,kind,format!("Same {:03}",i%10),p.to_string()]).unwrap();
            ids.push(id);
        }
        tx.commit().unwrap();
        ids
    }
}
fn ids(page: &Value) -> BTreeSet<String> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["entity"]["id"].as_str().unwrap().into())
        .collect()
}
fn invalid(f: &F, input: Value) {
    let r = f.call("home.query", input);
    assert!(!r.ok);
    assert_eq!(
        serde_json::to_value(r.error.unwrap()).unwrap()["code"],
        "validation_error"
    );
}
fn item<'a>(page: &'a Value, e: &Value) -> &'a Value {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["entity"]["id"] == e["id"])
        .unwrap()
}

#[test]
fn today_workflow_differences_civil_defaults_and_strict_cursors() {
    let f = F::new();
    let late = f.create("task", "Late", json!({"due_date":"2026-10-11"}));
    let today = f.create(
        "task",
        "Today",
        json!({"due_date":"2026-10-12","inbox":false}),
    );
    f.create("task", "Future", json!({"due_date":"2026-10-13"}));
    f.create(
        "task",
        "Done",
        json!({"due_date":"2026-10-11","state":"done"}),
    );
    let planned = f.create("task", "Planned only", json!({"planned_date":"2026-10-12"}));
    assert_eq!(
        ids(&f.query("tasks_today")),
        BTreeSet::from([
            late["id"].as_str().unwrap().into(),
            today["id"].as_str().unwrap().into()
        ])
    );
    let my = f.ok(
        "workflow.query",
        json!({"view":"my_day","as_of":"2026-10-12"}),
    );
    assert_eq!(my["items"][0]["id"], planned["id"]);
    assert_eq!(
        f.ok(
            "workflow.query",
            json!({"view":"overdue","as_of":"2026-10-12"})
        )["items"][0]["id"],
        late["id"]
    );
    let now = chrono::Utc::now()
        .with_timezone(&chrono_tz::Pacific::Kiritimati)
        .date_naive()
        .to_string();
    assert_eq!(
        f.ok("home.query", json!({"view":"tasks_today"}))["date"],
        now
    );
    for input in [
        json!({"view":"future"}),
        json!({"view":"tasks_today","as_of":"2026-02-30"}),
        json!({"view":"tasks_all","limit":0}),
        json!({"view":"tasks_all","limit":101}),
        json!({"view":"tasks_all","cursor":"[]"}),
        json!({"view":"tasks_all","extra":1}),
    ] {
        invalid(&f, input);
    }
    let first = f.ok(
        "home.query",
        json!({"view":"tasks_all","as_of":"2026-10-12","limit":1}),
    );
    let cursor = first["next_cursor"].clone();
    let second = f.ok(
        "home.query",
        json!({"view":"tasks_all","limit":1,"cursor":cursor}),
    );
    assert_eq!(second["date"], "2026-10-12");
    for input in [
        json!({"view":"tasks_done","limit":1,"cursor":cursor}),
        json!({"view":"tasks_all","as_of":"2026-10-13","limit":1,"cursor":cursor}),
        json!({"view":"tasks_all","cursor":cursor}),
    ] {
        invalid(&f, input);
    }
    let foreign = F::new();
    invalid(
        &foreign,
        json!({"view":"tasks_all","limit":1,"cursor":cursor}),
    );
    let mut raw: Value = serde_json::from_str(cursor.as_str().unwrap()).unwrap();
    raw["version"] = json!(99);
    invalid(
        &f,
        json!({"view":"tasks_all","limit":1,"cursor":raw.to_string()}),
    );
    let base: Value = serde_json::from_str(cursor.as_str().unwrap()).unwrap();
    for (key, value) in [
        ("offset", json!(0)),
        ("offset", json!(10001)),
        ("offset", json!(-1)),
        ("unexpected", json!(true)),
        ("limit", json!(99)),
    ] {
        let mut c = base.clone();
        c[key] = value;
        invalid(
            &f,
            json!({"view":"tasks_all","limit":1,"cursor":c.to_string()}),
        );
    }
    invalid(&f, json!({"view":"tasks_all","cursor":"x".repeat(2049)}));
    for day in ["0000-01-01", "10000-01-01", "2026-1-02", "2026-13-01"] {
        invalid(&f, json!({"view":"tasks_all","as_of":day}));
    }
    for limit in [1, 100] {
        assert_eq!(
            f.ok("home.query", json!({"view":"tasks_all","limit":limit}))["view"],
            "tasks_all"
        );
    }
}

#[test]
fn every_view_orders_and_paginates_unique_originals_and_declared_groups() {
    let f = F::new();
    for (kind, p) in [
        (
            "task",
            json!({"state":"todo","due_date":"2026-10-12","inbox":true}),
        ),
        ("note", json!({})),
        ("project", json!({"state":"doing"})),
        ("tag", json!({"classification":"topic"})),
        ("person", json!({})),
    ] {
        f.seed(kind, 51, p);
    }
    f.seed("task", 51, json!({"state":"done"}));
    f.seed("project", 51, json!({"state":"planned"}));
    for view in VIEWS {
        let mut cursor = Value::Null;
        let mut seen = BTreeSet::new();
        let mut expected = 0;
        loop {
            let page = f.ok(
                "home.query",
                json!({"view":view,"as_of":"2026-10-12","cursor":cursor}),
            );
            if expected == 0 {
                expected = page["matching_count"].as_u64().unwrap();
            }
            let local = ids(&page);
            assert!(local.len() <= 50);
            for id in &local {
                assert!(seen.insert(id.clone()), "{view}");
            }
            for g in page["groups"].as_array().unwrap() {
                for id in g["entity_ids"].as_array().unwrap() {
                    assert!(local.contains(id.as_str().unwrap()));
                }
            }
            cursor = page["next_cursor"].clone();
            if cursor.is_null() {
                break;
            }
        }
        assert_eq!(seen.len() as u64, expected, "{view}");
        assert!(expected >= 51);
    }
    assert_eq!(
        f.query("projects_board")["groups"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert_eq!(f.query("tags_types")["groups"].as_array().unwrap().len(), 3);
    let old = f.ok(
        "workflow.query",
        json!({"view":"active_projects","limit":200}),
    );
    assert_eq!(old["items"].as_array().unwrap().len(), 102);
    assert_eq!(f.query("projects_active")["matching_count"], 51);
    let people = f.query("people_contacts");
    let titles = people["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["entity"]["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(titles.windows(2).all(|a| a[0] <= a[1]));
}

#[test]
fn summaries_context_duplicates_archive_and_unknown_fields_are_conservative() {
    let f = F::new();
    let a = f.create("project", "A", json!({"state":"doing"}));
    let b = f.create("project", "B", json!({}));
    let empty = f.create("project", "Empty", json!({}));
    let tag = f.create("tag", "Tag", json!({"classification":"area"}));
    let person = f.create("person", "Contact", json!({}));
    let t = f.create("task", "Shared", json!({"due_date":"2026-10-11"}));
    f.link(&t, &a, "project");
    f.link(&t, &b, "project");
    for i in 0..51 {
        let n = f.create(
            "task",
            &format!("T{i}"),
            json!({"state":if i%2==0{"done"}else{"todo"}}),
        );
        f.link(&n, &a, "project");
    }
    let note = f.create("note", "Original", json!({}));
    f.link(&note, &tag, "tag");
    f.link(&note, &b, "project");
    f.link(&note, &person, "person");
    let notes = f.seed("note", 51, json!({}));
    let db = f.db();
    for id in notes {
        db.execute(
            "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
            params![
                uuid::Uuid::new_v4().to_string(),
                id,
                tag["id"].as_str().unwrap()
            ],
        )
        .unwrap();
    }
    let page = f.query("projects_board");
    let s = &item(&page, &a)["summary"];
    assert_eq!(s["total"], 52);
    assert_eq!(s["done"], 26);
    assert_eq!(s["pending"], 26);
    assert_eq!(s["overdue"], 1);
    assert_eq!(s["progress_percent"], 50);
    assert!(item(&page, &empty)["summary"]["progress_percent"].is_null());
    assert_eq!(
        item(&f.query("tags_recent"), &tag)["summary"]["note_count"],
        52
    );
    let grouped = f.query("tasks_all");
    let occurrences = grouped["groups"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|g| g["entity_ids"].as_array().unwrap())
        .filter(|id| **id == t["id"])
        .count();
    assert_eq!(occurrences, 2);
    let revision = f.ok("entity.get", json!({"id":b["id"]}))["revision"].clone();
    f.ok(
        "entity.archive",
        json!({"id":b["id"],"expected_revision":revision}),
    );
    assert!(
        item(&f.query("notes_all"), &note)["context"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == b["id"] && !c["archived_at"].is_null())
    );
    f.raw(
        &t,
        json!({"state":"future","due_date":"2026-10-11","future":{"keep":true}}),
    );
    assert!(
        !item(&f.query("projects_board"), &a)["summary"]["available"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(f.query("tasks_today")["unavailable_count"], 1);
    f.raw(&note, json!({"future":{"keep":true}}));
    assert_eq!(
        item(&f.query("notes_all"), &note)["metadata"]["inbox"],
        true
    );
    f.raw(&t, json!({"state":null}));
    assert!(item(&f.query("tasks_all"), &t)["metadata"]["state"].is_null());
    f.raw(&note, json!([]));
    assert!(
        item(&f.query("notes_all"), &note)["metadata"]["issues"]
            .as_array()
            .unwrap()
            .contains(&json!("payload"))
    );
}

#[test]
fn preservation_backup_future_documents_and_scope_are_real() {
    let f = F::new();
    for version in [1, 2, 3] {
        f.ok("entity.create",json!({"kind":"note","title":format!("v{version}"),"document":{"schema_version":version,"blocks":[]}}));
    }
    let n = f.create("note", "Future", json!({}));
    f.db()
        .execute(
            "UPDATE documents SET json=?2,schema_version=99 WHERE entity_id=?1",
            params![
                n["id"].as_str().unwrap(),
                "{\"schema_version\":99,\"future\":true}"
            ],
        )
        .unwrap();
    let snapshot = || {
        let db = f.db();
        [
            "SELECT json_array(id,title,revision,payload,created_at,updated_at,archived_at) FROM entities ORDER BY id",
            "SELECT json FROM documents ORDER BY entity_id",
            "SELECT json_array(id,source_id,target_id,kind) FROM relations ORDER BY id",
            "SELECT json_array(id,request_id,entity_id,operation,origin,previous_revision,new_revision,created_at) FROM changes ORDER BY id",
            "SELECT text FROM search_fts ORDER BY entity_id,block_id",
        ]
        .iter()
        .map(|sql| {
            db.prepare(sql)
                .unwrap()
                .query_map([], |r| r.get::<_, String>(0))
                .unwrap()
                .collect::<std::result::Result<Vec<_>, _>>()
                .unwrap()
        })
        .collect::<Vec<_>>()
    };
    let before = snapshot();
    for v in VIEWS {
        f.query(v);
    }
    assert_eq!(before, snapshot());
    let backup = f.temp.path().join("backup");
    let restored = f.temp.path().join("restored");
    f.ok("workspace.backup", json!({"destination":backup}));
    f.ok(
        "workspace.restore",
        json!({"backup":backup,"destination":restored}),
    );
    for view in VIEWS {
        let r = execute_json(
            json!({"workspace":restored,"command":{"operation":"home.query","input":{"view":view,"as_of":"2026-10-12"}}}),
            "test",
        );
        assert!(r.ok);
        assert_eq!(r.data.unwrap(), f.query(view));
    }
    let p = f.ok("home.query", json!({"view":"notes_all","limit":1}));
    let r = execute_json(
        json!({"workspace":restored,"command":{"operation":"home.query","input":{"view":"notes_all","limit":1,"cursor":p["next_cursor"]}}}),
        "test",
    );
    assert!(!r.ok);
    let body = f.ok("entity.get", json!({"id":n["id"]}));
    assert_eq!(body["document"]["future"], true);
    assert!(!f.query("notes_all").to_string().contains("schema_version"));
}

#[test]
fn view_membership_orders_and_partial_metadata_are_explicit() {
    let f = F::new();
    let ordered = |view: &str| {
        f.query(view)["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["entity"]["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    let note_in = f.create("note", "Inbox", json!({}));
    let note_out = f.create("note", "Organizada", json!({"inbox":false}));
    assert_eq!(
        ids(&f.query("notes_all")),
        BTreeSet::from([
            note_in["id"].as_str().unwrap().into(),
            note_out["id"].as_str().unwrap().into()
        ])
    );
    assert_eq!(
        ordered("notes_inbox"),
        vec![note_in["id"].as_str().unwrap()]
    );
    let todo = f.create(
        "task",
        "Pendiente",
        json!({"inbox":false,"due_date":"2026-10-11"}),
    );
    let done = f.create("task", "Completa", json!({"state":"done","inbox":true}));
    let no_due = f.create("task", "Sin fecha", json!({"due_date":null}));
    assert_eq!(ordered("tasks_inbox"), vec![no_due["id"].as_str().unwrap()]);
    assert_eq!(ordered("tasks_done"), vec![done["id"].as_str().unwrap()]);
    assert_eq!(ordered("tasks_today"), vec![todo["id"].as_str().unwrap()]);
    let mut projects = vec![];
    for state in ["planned", "doing", "ongoing", "done", "on_hold"] {
        projects.push(f.create("project", state, json!({"state":state})));
    }
    assert_eq!(
        ordered("projects_active"),
        projects[1..3]
            .iter()
            .map(|p| p["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        ordered("projects_planned"),
        vec![projects[0]["id"].as_str().unwrap()]
    );
    assert_eq!(
        ordered("projects_board"),
        projects
            .iter()
            .map(|p| p["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    );
    for task in [&todo, &done, &no_due] {
        f.link(task, &projects[0], "project");
    }
    let summary = f.query("projects_planned")["items"][0]["summary"].clone();
    assert_eq!(summary["total"], 3);
    assert_eq!(summary["done"], 1);
    assert_eq!(summary["pending"], 2);
    assert_eq!(summary["overdue"], 1);
    assert_eq!(summary["progress_percent"], 33);
    let area = f.create("tag", "z", json!({"classification":"area"}));
    let resource = f.create("tag", "B", json!({"classification":"resource"}));
    let topic = f.create("tag", "a", json!({"classification":"topic"}));
    let db = f.db();
    for tag in [&area, &resource, &topic] {
        db.execute(
            "UPDATE entities SET updated_at='2026-10-11T12:00:00.000000Z' WHERE id=?1",
            params![tag["id"].as_str().unwrap()],
        )
        .unwrap();
    }
    let mut tied = vec![
        area["id"].as_str().unwrap().to_owned(),
        resource["id"].as_str().unwrap().to_owned(),
        topic["id"].as_str().unwrap().to_owned(),
    ];
    tied.sort();
    assert_eq!(ordered("tags_recent"), tied);
    db.execute(
        "UPDATE entities SET updated_at='2026-10-12T12:00:00.000000Z' WHERE id=?1",
        params![area["id"].as_str().unwrap()],
    )
    .unwrap();
    assert_eq!(ordered("tags_recent")[0], area["id"].as_str().unwrap());
    assert_eq!(
        ordered("tags_alphabetical"),
        vec![
            topic["id"].as_str().unwrap(),
            resource["id"].as_str().unwrap(),
            area["id"].as_str().unwrap()
        ]
    );
    assert_eq!(
        ordered("tags_types"),
        vec![
            area["id"].as_str().unwrap(),
            resource["id"].as_str().unwrap(),
            topic["id"].as_str().unwrap()
        ]
    );
    // Sólo los metadatos que deciden el filtro afectan pertenencia/disponibilidad.
    f.raw(
        &todo,
        json!({"state":"todo","due_date":"2026-10-11","priority":"future","inbox":null}),
    );
    assert_eq!(ordered("tasks_today"), vec![todo["id"].as_str().unwrap()]);
    assert_eq!(f.query("tasks_inbox")["unavailable_count"], 1);
    assert_eq!(
        item(&f.query("projects_planned"), &projects[0])["summary"]["progress_percent"],
        33
    );
    f.raw(&todo, json!({"due_date":"invalid"}));
    assert_eq!(f.query("tasks_today")["unavailable_count"], 1);
    assert_eq!(
        item(&f.query("projects_planned"), &projects[0])["summary"]["available"],
        false
    );
    f.raw(&note_in, Value::Null);
    assert_eq!(f.query("notes_all")["matching_count"], 2);
    assert_eq!(f.query("notes_inbox")["unavailable_count"], 1);
    f.raw(&topic, json!({"classification":null}));
    assert_eq!(f.query("tags_types")["groups"][3]["key"], "unavailable");
}

#[test]
fn candidate_source_group_limits_and_context_truncation_do_not_fake_success() {
    let f = F::new();
    let projects = f.seed("project", 201, json!({}));
    let task = f.create("task", "Many contexts", json!({}));
    let db = f.db();
    for id in &projects {
        db.execute(
            "INSERT INTO relations VALUES (?1,?2,?3,'project')",
            params![
                uuid::Uuid::new_v4().to_string(),
                task["id"].as_str().unwrap(),
                id
            ],
        )
        .unwrap();
    }
    invalid(&f, json!({"view":"tasks_all"}));
    db.execute(
        "DELETE FROM relations WHERE target_id IN (?1,?2)",
        params![projects[0], projects[1]],
    )
    .unwrap();
    let p = f.query("tasks_all");
    assert_eq!(p["items"][0]["context_total"], 199);
    assert_eq!(p["items"][0]["context"].as_array().unwrap().len(), 50);
    assert_eq!(p["groups"].as_array().unwrap().len(), 200);
    let g = F::new();
    g.seed("person", 10000, json!({}));
    assert_eq!(g.query("people_contacts")["matching_count"], 10000);
    g.seed("person", 1, json!({}));
    invalid(&g, json!({"view":"people_contacts"}));
    let h = F::new();
    let tag = h.create("tag", "Tag", json!({"classification":"topic"}));
    let notes = h.seed("note", 10000, json!({}));
    let mut db = h.db();
    let tx = db.transaction().unwrap();
    for id in notes {
        tx.execute(
            "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
            params![
                uuid::Uuid::new_v4().to_string(),
                id,
                tag["id"].as_str().unwrap()
            ],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    assert_eq!(
        h.query("tags_types")["items"][0]["summary"]["note_count"],
        10000
    );
    let extra = h.seed("note", 1, json!({}));
    db.execute(
        "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
        params![
            uuid::Uuid::new_v4().to_string(),
            extra[0],
            tag["id"].as_str().unwrap()
        ],
    )
    .unwrap();
    invalid(&h, json!({"view":"tags_types"}));
}

#[test]
fn incompatible_links_full_link_cap_and_summary_archive_ties() {
    let f = F::new();
    let tag = f.create("tag", "Tag", json!({"classification":"topic"}));
    let notes = f.seed("note", 2, json!({}));
    let db = f.db();
    for id in &notes {
        db.execute(
            "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
            params![
                uuid::Uuid::new_v4().to_string(),
                id,
                tag["id"].as_str().unwrap()
            ],
        )
        .unwrap();
    }
    let last = notes.iter().min().unwrap();
    assert_eq!(
        f.query("tags_types")["items"][0]["summary"]["last_note"]["id"],
        *last
    );
    f.ok("entity.archive", json!({"id":last,"expected_revision":1}));
    assert_eq!(
        f.query("tags_types")["items"][0]["summary"]["note_count"],
        1
    );
    f.ok("entity.restore", json!({"id":last,"expected_revision":2}));
    assert_eq!(
        f.query("tags_types")["items"][0]["summary"]["note_count"],
        2
    );
    let person = f.create("person", "Person", json!({}));
    db.execute(
        "INSERT INTO relations VALUES (?1,?2,?3,'project')",
        params![
            uuid::Uuid::new_v4().to_string(),
            notes[0],
            person["id"].as_str().unwrap()
        ],
    )
    .unwrap();
    let r = f.call("home.query", json!({"view":"notes_all"}));
    assert!(!r.ok);
    assert_eq!(
        serde_json::to_value(r.error.unwrap()).unwrap()["code"],
        "storage_error"
    );
    let h = F::new();
    let people = h.seed("person", 5001, json!({}));
    let tags = h.seed("tag", 10, json!({"classification":"area"}));
    let mut db = h.db();
    let tx = db.transaction().unwrap();
    for p in &people[..5000] {
        for t in &tags {
            tx.execute(
                "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
                params![uuid::Uuid::new_v4().to_string(), p, t],
            )
            .unwrap();
        }
    }
    tx.commit().unwrap();
    assert_eq!(
        h.query("tags_recent")["items"].as_array().unwrap().len(),
        10
    );
    db.execute(
        "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
        params![uuid::Uuid::new_v4().to_string(), people[5000], tags[0]],
    )
    .unwrap();
    invalid(&h, json!({"view":"tags_recent"}));
}

#[test]
fn timestamp_and_id_orders_civil_age_and_archived_today() {
    let f = F::new();
    let mut people = f.seed("person", 4, json!({}));
    people.sort();
    let db = f.db();
    for (i, id) in people.iter().enumerate() {
        db.execute(
            "UPDATE entities SET title=?2,created_at=?3,updated_at=?4 WHERE id=?1",
            params![
                id,
                if i < 2 { "á" } else { "A" },
                format!("2026-10-{:02}T12:00:00.000000Z", 10 + i),
                format!("2026-10-{:02}T12:00:00.000000Z", 13 - i)
            ],
        )
        .unwrap();
    }
    let ordered = |v: &str| {
        f.query(v)["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["entity"]["id"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ordered("people_contacts"),
        vec![
            people[2].clone(),
            people[3].clone(),
            people[0].clone(),
            people[1].clone()
        ]
    );
    assert_eq!(
        ordered("people_new"),
        people.iter().rev().cloned().collect::<Vec<_>>()
    );
    assert_eq!(ordered("people_updated"), people);
    let n = f.create("note", "Civil age", json!({}));
    db.execute(
        "UPDATE entities SET updated_at='2026-10-11T12:00:00Z' WHERE id=?1",
        params![n["id"].as_str().unwrap()],
    )
    .unwrap();
    assert_eq!(
        item(&f.query("notes_all"), &n)["metadata"]["updated_days_ago"],
        0
    );
    db.execute(
        "UPDATE entities SET updated_at='2026-10-13T12:00:00Z' WHERE id=?1",
        params![n["id"].as_str().unwrap()],
    )
    .unwrap();
    assert_eq!(
        item(&f.query("notes_all"), &n)["metadata"]["updated_days_ago"],
        -2
    );
    let t = f.create("task", "Archived", json!({"due_date":"2026-10-12"}));
    f.ok(
        "entity.archive",
        json!({"id":t["id"],"expected_revision":1}),
    );
    assert_eq!(f.query("tasks_today")["matching_count"], 0);
}
