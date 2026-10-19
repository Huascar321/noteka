//! Corpus sintético reproducible, fuera del checkout; no descarga modelos ni contenido remoto.
use noteka_core::{contracts::*, execute_json, workspace::Workspace};
use rusqlite::{Connection, params};
use serde_json::json;
use std::time::Instant;
fn id(i: usize) -> String {
    format!("00000000-0000-4000-8000-{i:012}")
}
fn percentile(v: &mut [f64], p: f64) -> f64 {
    v.sort_by(f64::total_cmp);
    v[((v.len() as f64 * p).ceil() as usize - 1).min(v.len() - 1)]
}
fn main() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("w");
    let w = Workspace::create(&path, Some("UTC".into())).unwrap();
    let mut db = Connection::open(path.join("noteka.sqlite")).unwrap();
    let tx = db.transaction().unwrap();
    for i in 0..10_000 {
        let kind = if i < 3000 {
            "task"
        } else if i < 8000 {
            "note"
        } else if i < 8500 {
            "project"
        } else if i < 9000 {
            "tag"
        } else {
            "person"
        };
        let payload = match kind {
            "task" => {
                json!({"state":if i%4==0{"done"}else{"todo"},"inbox":i%2==0,"due_date":if i%3==0{"2026-10-11"}else{"2026-10-12"}})
            }
            "note" => json!({"inbox":i%2==0}),
            "project" => json!({"state":(["planned","doing","ongoing","done","on_hold"][i%5])}),
            "tag" => json!({"classification":(["area","resource","topic"][i%3])}),
            _ => json!({}),
        };
        tx.execute(
            "INSERT INTO entities VALUES (?1,?2,?3,1,NULL,?4,?4,?5)",
            params![
                id(i),
                kind,
                format!("Corpus Home {i:05}"),
                format!("2026-10-{:02}T12:00:00.000000Z", i % 12 + 1),
                payload.to_string()
            ],
        )
        .unwrap();
        if kind == "note" {
            let doc = json!({"schema_version":3,"blocks":[{"id":"body","type":"paragraph","content":[{"type":"text","text":"contenido pesado no expandido ".repeat(200)}]}]});
            tx.execute(
                "INSERT INTO documents VALUES (?1,3,?2)",
                params![id(i), doc.to_string()],
            )
            .unwrap();
        }
    }
    let mut links = 0;
    for i in 0..8000 {
        for target in [8500 + i % 500, 8500 + (i + 1) % 500] {
            tx.execute(
                "INSERT INTO relations VALUES (?1,?2,?3,'tag')",
                params![id(10000 + links), id(i), id(target)],
            )
            .unwrap();
            links += 1;
        }
        if i < 3000 {
            for target in [8000 + i % 500, 8000 + (i + 1) % 500] {
                tx.execute(
                    "INSERT INTO relations VALUES (?1,?2,?3,'project')",
                    params![id(10000 + links), id(i), id(target)],
                )
                .unwrap();
                links += 1;
            }
        }
    }
    tx.commit().unwrap();
    let views = [
        HomeView::TasksToday,
        HomeView::TasksInbox,
        HomeView::TasksAll,
        HomeView::TasksDone,
        HomeView::NotesAll,
        HomeView::NotesInbox,
        HomeView::ProjectsActive,
        HomeView::ProjectsPlanned,
        HomeView::ProjectsBoard,
        HomeView::TagsRecent,
        HomeView::TagsAlphabetical,
        HomeView::TagsTypes,
        HomeView::PeopleContacts,
        HomeView::PeopleNew,
        HomeView::PeopleUpdated,
    ];
    let mut reports = vec![];
    for view in views {
        let input = json!({"view":view,"as_of":"2026-10-12","limit":50});
        for _ in 0..3 {
            w.home_query(serde_json::from_value(input.clone()).unwrap())
                .unwrap();
        }
        let (mut direct, mut operation) = (vec![], vec![]);
        let mut bytes = 0;
        for _ in 0..30 {
            let start = Instant::now();
            let page = w
                .home_query(serde_json::from_value(input.clone()).unwrap())
                .unwrap();
            direct.push(start.elapsed().as_secs_f64() * 1000.);
            bytes = serde_json::to_vec(&page).unwrap().len();
            let start = Instant::now();
            let r = execute_json(
                json!({"workspace":path,"command":{"operation":"home.query","input":input}}),
                "benchmark",
            );
            assert!(r.ok, "{:?}", r.error);
            operation.push(start.elapsed().as_secs_f64() * 1000.);
        }
        reports.push(json!({"view":view,"direct_p50_ms":percentile(&mut direct,0.5),"direct_p95_ms":percentile(&mut direct,0.95),"operation_p50_ms":percentile(&mut operation,0.5),"operation_p95_ms":percentile(&mut operation,0.95),"response_bytes":bytes}));
    }
    println!("{}",serde_json::to_string_pretty(&json!({"entities":10000,"tasks":3000,"notes":5000,"projects":500,"tags":500,"people":1000,"links":links,"note_body_repeat":200,"warmups_each":3,"samples_each":30,"limit":50,"as_of":"2026-10-12","profile":"release","runtime_arch":std::env::consts::ARCH,"sqlite_bytes_excludes_wal":std::fs::metadata(path.join("noteka.sqlite")).unwrap().len(),"reports":reports,"limits":"10.000 candidates, 10.000 summary sources, 50.000 links, 200 page groups; boundaries tested separately","corpus":"Deterministic IDs/timestamps, SQLite v7 batch fixture, 5.000 documents v3 not selected by Home; hot caches; no UI, LLM, remote calls or downloads"})).unwrap());
}
