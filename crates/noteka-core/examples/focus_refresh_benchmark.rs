//! Backend-only synthetic Home refresh measurement. No personal workspace is opened.
use noteka_core::{contracts::HomeView, execute_json, workspace::Workspace};
use rusqlite::{Connection, params};
use serde_json::json;
use std::{path::Path, time::Instant};

fn id(i: usize) -> String {
    format!("00000000-0000-4000-8000-{i:012}")
}

fn percentile(values: &mut [f64], fraction: f64) -> f64 {
    values.sort_by(f64::total_cmp);
    values[((values.len() as f64 * fraction).ceil() as usize - 1).min(values.len() - 1)]
}

fn peak_rss_bytes() -> Option<u64> {
    #[cfg(unix)]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
            return None;
        }
        let max = unsafe { usage.assume_init() }.ru_maxrss as u64;
        #[cfg(target_os = "macos")]
        return Some(max);
        #[cfg(not(target_os = "macos"))]
        return Some(max * 1024);
    }
    #[cfg(not(unix))]
    None
}

fn refresh_cycle(path: &Path, views: &[HomeView]) {
    // Home's existing pool permits two IPC reads at a time. These are complete
    // core operations including preflight, not fabricated query timings.
    for pair in views.chunks(2) {
        std::thread::scope(|scope| {
            for &view in pair {
                scope.spawn(move || {
                    let result = execute_json(
                        json!({"workspace":path,"command":{"operation":"home.query", "input":{"view":view,"as_of":"2026-10-12","limit":50}}}),
                        "synthetic-benchmark",
                    );
                    assert!(result.ok, "{view:?}: {:?}", result.error);
                });
            }
        });
    }
}

fn main() {
    let total: usize = std::env::args().nth(1).expect("expected 1000 or 8000").parse().unwrap();
    assert!(total == 1_000 || total == 8_000);
    let task_end = total * 30 / 100;
    let note_end = total * 80 / 100;
    let project_end = total * 85 / 100;
    let tag_end = total * 90 / 100;
    let project_count = project_end - note_end;
    let tag_count = tag_end - project_end;

    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("workspace");
    let _workspace = Workspace::create(&path, Some("UTC".into())).unwrap();
    let mut db = Connection::open(path.join("noteka.sqlite")).unwrap();
    let tx = db.transaction().unwrap();
    for i in 0..total {
        let (kind, payload) = if i < task_end {
            ("task", json!({"state": if i % 4 == 0 {"done"} else {"todo"}, "inbox": i % 2 == 0, "due_date":"2026-10-12"}))
        } else if i < note_end {
            ("note", json!({"inbox":i % 2 == 0}))
        } else if i < project_end {
            ("project", json!({"state":(["planned","doing","ongoing","done","on_hold"][i % 5])}))
        } else if i < tag_end {
            ("tag", json!({"classification":(["area","resource","topic"][i % 3])}))
        } else {
            ("person", json!({}))
        };
        tx.execute("INSERT INTO entities VALUES (?1,?2,?3,1,NULL,?4,?4,?5)",
            params![id(i),kind,format!("Synthetic {kind} {i:05}"),"2026-10-12T12:00:00.000000Z",payload.to_string()]).unwrap();
        if kind == "note" {
            let document = json!({"schema_version":3,"blocks":[{"id":"body","type":"paragraph","content":[{"type":"text","text":"synthetic content ".repeat(20)}]}]});
            tx.execute("INSERT INTO documents VALUES (?1,3,?2)", params![id(i),document.to_string()]).unwrap();
        }
    }
    let mut links = 0;
    for i in 0..note_end {
        for target in [project_end + i % tag_count, project_end + (i + 1) % tag_count] {
            tx.execute("INSERT INTO relations VALUES (?1,?2,?3,'tag')", params![id(total + links), id(i), id(target)]).unwrap();
            links += 1;
        }
        if i < task_end {
            for target in [note_end + i % project_count, note_end + (i + 1) % project_count] {
                tx.execute("INSERT INTO relations VALUES (?1,?2,?3,'project')", params![id(total + links), id(i), id(target)]).unwrap();
                links += 1;
            }
        }
    }
    tx.commit().unwrap();
    let views = [HomeView::TasksToday, HomeView::NotesAll, HomeView::ProjectsBoard,
        HomeView::TagsTypes, HomeView::GoalsActivity, HomeView::PeopleContacts];
    for _ in 0..3 { refresh_cycle(&path, &views); }
    let mut elapsed = Vec::with_capacity(30);
    for _ in 0..30 {
        let start = Instant::now();
        refresh_cycle(&path, &views);
        elapsed.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    let p50 = percentile(&mut elapsed, 0.50);
    let p95 = percentile(&mut elapsed, 0.95);
    println!("{}", serde_json::to_string_pretty(&json!({
        "entities":total,"relations":links,"sqlite_version":rusqlite::version(),
        "samples":30,"warmups":3,"operations_per_cycle":views.len(),"max_concurrency":2,
        "cycle_ms_p50":p50,"cycle_ms_p95":p95,"samples_ms":elapsed,
        "peak_rss_bytes":peak_rss_bytes(),"database_bytes_without_wal":std::fs::metadata(path.join("noteka.sqlite")).unwrap().len(),
        "scope":"synthetic backend operations with preflight, hot cache; excludes GUI and IPC serialization"
    })).unwrap());
}
