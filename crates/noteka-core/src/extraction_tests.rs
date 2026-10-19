use crate::{
    batch_storage as batch, batches,
    contracts::*,
    error::*,
    extraction_storage as store,
    workspace::{self, DB_NAME, Workspace},
};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Barrier},
    time::Duration,
};

fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
fn add(w: &mut Workspace, name: &str, bytes: &[u8]) -> Attachment {
    let path = w.path.parent().unwrap().join(name);
    fs::write(&path, bytes).unwrap();
    w.attachment_import(ImportAttachment {
        source_path: path.to_string_lossy().into(),
    })
    .unwrap()
}
fn enqueue(w: &Workspace, a: &Attachment) -> ExtractionJob {
    w.extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"utf8_text"})))
        .unwrap()
        .job
}
fn get(w: &Workspace, id: &str) -> ExtractionJob {
    w.extraction_get(IdInput { id: id.into() }).unwrap()
}
fn enabled(w: &Workspace) {
    let c = w
        .extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    w.extraction_control(ExtractionControlInput::Set {
        paused: false,
        expected_revision: c.revision,
    })
    .unwrap();
}
fn retry(w: &Workspace, id: &str) -> ExtractionJob {
    let j = get(w, id);
    w.extraction_retry(ExtractionRevision {
        id: id.into(),
        expected_revision: j.revision,
    })
    .unwrap()
}
fn run(w: &Workspace, id: &str) -> ExtractionJob {
    w.extraction_run(RunExtraction {
        id: Some(id.into()),
    })
    .unwrap()
    .job
    .unwrap()
}
fn fragments(w: &Workspace, id: &str, limit: u32, cursor: Option<String>) -> FragmentPage {
    w.extraction_fragments(ReadFragments {
        id: id.into(),
        limit: Some(limit),
        cursor,
    })
    .unwrap()
}
fn reason(e: CoreError) -> String {
    e.details["reason"]
        .as_str()
        .unwrap_or("missing reason")
        .into()
}
fn base(w: &Workspace) -> String {
    batch::base_hash(&w.conn, &batch::Budget::new(&w.conn).unwrap()).unwrap()
}
pub(super) fn raw(conn: &Connection) -> Vec<(String, Vec<String>)> {
    let names: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    names
        .into_iter()
        .map(|name| {
            let mut s = conn.prepare(&format!("SELECT * FROM \"{name}\"")).unwrap();
            let cols = s.column_count();
            let mut rows = s
                .query_map([], |r| {
                    Ok((0..cols)
                        .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                        .collect::<Vec<_>>()
                        .join("|"))
                })
                .unwrap()
                .collect::<std::result::Result<Vec<_>, _>>()
                .unwrap();
            rows.sort();
            (name, rows)
        })
        .collect()
}
#[test]
fn explicit_queue_dedupes_and_publishes_literal_without_domain_writes() {
    let (_t, mut w) = setup();
    let bytes =
        "\u{feff}é😀\r\n<script>fetch('https://invalid')</script>\n$rm -rf ignored".repeat(600);
    let a = add(&mut w, "literal.MD", bytes.as_bytes());
    assert!(w.extraction_list(dto(json!({}))).unwrap().items.is_empty());
    assert!(
        w.extraction_control(ExtractionControlInput::Get {})
            .unwrap()
            .paused
    );
    let j = enqueue(&w, &a);
    assert_eq!(j.revision, 1);
    assert_eq!(j.attempts, 0);
    assert_eq!(
        w.extraction_run(RunExtraction::default())
            .unwrap()
            .reason
            .as_deref(),
        Some("queue_paused")
    );
    enabled(&w);
    let h = base(&w);
    let old = raw(&w.conn);
    assert_eq!(enqueue(&w, &a).id, j.id);
    assert_eq!(enqueue(&w, &a).revision, 1);
    let result = run(&w, &j.id);
    assert_eq!(result.state, ExtractionState::Succeeded);
    assert_eq!(result.attempts, 1);
    assert_eq!(result.revision, 3);
    assert_eq!(base(&w), h);
    let now = raw(&w.conn);
    assert_eq!(
        old.iter()
            .filter(|(n, _)| !n.starts_with("extraction_"))
            .collect::<Vec<_>>(),
        now.iter()
            .filter(|(n, _)| !n.starts_with("extraction_"))
            .collect::<Vec<_>>()
    );
    let page = fragments(&w, &j.id, 64, None);
    assert!(page.covers_generation);
    assert_eq!(
        page.items
            .iter()
            .map(|p| p.text.as_str())
            .collect::<String>(),
        bytes
    );
    assert_eq!(enqueue(&w, &a).attempts, 1);
    assert_eq!(
        w.extraction_run(RunExtraction::default())
            .unwrap()
            .reason
            .as_deref(),
        Some("queue_empty")
    );
    let retained = get(&w, &j.id).generation.unwrap().id;
    retry(&w, &j.id);
    let page = fragments(&w, &j.id, 64, None);
    assert!(page.result_from_previous_attempt);
    assert_eq!(page.generation.id, retained);
    let r = run(&w, &j.id);
    assert_ne!(r.generation.unwrap().id, retained);
    assert_eq!(r.attempts, 2);
    store::integrity(&w.conn).unwrap();
}
#[test]
fn empty_invalid_content_and_ineligible_sources_have_explicit_errors() {
    let (_t, mut w) = setup();
    enabled(&w);
    for (name, bytes, why) in [
        ("bad.txt", vec![0xff], "invalid_utf8"),
        ("nul.md", vec![0], "nul_in_text"),
    ] {
        let a = add(&mut w, name, &bytes);
        let j = enqueue(&w, &a);
        let e = w
            .extraction_run(RunExtraction {
                id: Some(j.id.clone()),
            })
            .unwrap_err();
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.details["reason"], why);
        assert_eq!(e.details["job_id"], j.id);
        let j = get(&w, &j.id);
        assert_eq!(j.state, ExtractionState::Failed);
        assert!(j.generation.is_none());
        assert_eq!(j.attempts, 1);
    }
    for (name, bytes) in [
        ("renamed.txt", b"%PDF-1.7".to_vec()),
        ("image.md", b"GIF89a".to_vec()),
        ("sheet.xlsx", b"PK".to_vec()),
        ("large.txt", vec![b'a'; MAX_EXTRACTION_BYTES as usize + 1]),
    ] {
        let a = add(&mut w, name, &bytes);
        let e = w
            .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"utf8_text"})))
            .unwrap_err();
        assert_eq!(e.exit_code(), 2);
    }
    let a = add(&mut w, "empty.txt", b"");
    let j = enqueue(&w, &a);
    run(&w, &j.id);
    let page = fragments(&w, &j.id, 64, None);
    assert!(page.covers_generation);
    assert!(page.items.is_empty());
    assert_eq!(page.generation.covered_bytes, 0);
    assert_eq!(
        reason(
            w.extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"pdf"})))
                .unwrap_err()
        ),
        "unsupported_extractor_config"
    );
    assert_eq!(reason(w.extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"utf8_text","config":{"version":2,"chunk_bytes":4096}}))).unwrap_err()),"unsupported_extractor_config");
    assert_eq!(
        reason(
            w.extraction_enqueue(dto(
                json!({"attachment_id":"../../escape","extractor":"utf8_text"})
            ))
            .unwrap_err()
        ),
        "invalid_id"
    );
    let a = add(&mut w, "separate.txt", b"");
    assert_ne!(enqueue(&w, &a).id, j.id);
}
#[test]
fn revisions_transitions_overflow_and_future_configs_are_preserved() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"a");
    let j = enqueue(&w, &a);
    assert_eq!(
        reason(
            w.extraction_retry(ExtractionRevision {
                id: j.id.clone(),
                expected_revision: 1
            })
            .unwrap_err()
        ),
        "invalid_state"
    );
    let j = w
        .extraction_cancel(ExtractionRevision {
            id: j.id,
            expected_revision: 1,
        })
        .unwrap();
    assert_eq!(j.state, ExtractionState::Cancelled);
    assert_eq!(
        w.extraction_cancel(ExtractionRevision {
            id: j.id.clone(),
            expected_revision: 2
        })
        .unwrap()
        .revision,
        2
    );
    assert_eq!(
        w.extraction_cancel(ExtractionRevision {
            id: j.id.clone(),
            expected_revision: 1
        })
        .unwrap_err()
        .exit_code(),
        4
    );
    let c = w
        .extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    assert_eq!(
        w.extraction_control(ExtractionControlInput::Set {
            paused: true,
            expected_revision: c.revision
        })
        .unwrap()
        .revision,
        c.revision
    );
    enabled(&w);
    assert_eq!(
        w.extraction_control(ExtractionControlInput::Set {
            paused: false,
            expected_revision: 1
        })
        .unwrap_err()
        .exit_code(),
        4
    );
    retry(&w, &j.id);
    run(&w, &j.id);
    let raw_config = "{ \"version\": 9, \"chunk_bytes\":4096, \"future\":{\"literal\":\"stay\"} }";
    let v: Value = serde_json::from_str(raw_config).unwrap();
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?2,config_hash=?3 WHERE id=?1",
            params![j.id, raw_config, store::config_hash(&v).unwrap()],
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE extraction_generations SET config=?2,config_hash=?3 WHERE job_id=?1",
            params![j.id, raw_config, store::config_hash(&v).unwrap()],
        )
        .unwrap();
    let future = get(&w, &j.id);
    assert!(!future.supported);
    assert_eq!(future.config, v);
    let c = w
        .extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    w.extraction_control(ExtractionControlInput::Set {
        paused: true,
        expected_revision: c.revision,
    })
    .unwrap();
    assert_eq!(
        reason(
            w.extraction_run(RunExtraction {
                id: Some(j.id.clone())
            })
            .unwrap_err()
        ),
        "unsupported_extractor_config"
    );
    assert_eq!(
        w.extraction_run(RunExtraction {
            id: Some("00000000-0000-0000-0000-000000000000".into())
        })
        .unwrap_err()
        .exit_code(),
        3
    );
    enabled(&w);
    assert_eq!(
        reason(
            w.extraction_retry(ExtractionRevision {
                id: j.id.clone(),
                expected_revision: future.revision
            })
            .unwrap_err()
        ),
        "unsupported_extractor_config"
    );
    assert_eq!(
        w.conn
            .query_row(
                "SELECT config FROM extraction_jobs WHERE id=?1",
                [&j.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        raw_config
    );
    w.conn
        .execute(
            "UPDATE extraction_jobs SET state='queued' WHERE id=?1",
            [&j.id],
        )
        .unwrap();
    assert_eq!(
        w.extraction_run(RunExtraction::default())
            .unwrap()
            .incompatible_count,
        1
    );
    assert_eq!(
        reason(
            w.extraction_run(RunExtraction {
                id: Some(j.id.clone())
            })
            .unwrap_err()
        ),
        "unsupported_extractor_config"
    );
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?2,config_hash=?3,revision=4294967295 WHERE id=?1",
            params![
                j.id,
                serde_json::to_string(&ExtractionConfig::default()).unwrap(),
                store::config_hash(&json!(ExtractionConfig::default())).unwrap()
            ],
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE extraction_generations SET config=?2,config_hash=?3 WHERE job_id=?1",
            params![
                j.id,
                serde_json::to_string(&ExtractionConfig::default()).unwrap(),
                store::config_hash(&json!(ExtractionConfig::default())).unwrap()
            ],
        )
        .unwrap();
    assert_eq!(
        reason(
            w.extraction_cancel(ExtractionRevision {
                id: j.id,
                expected_revision: u32::MAX
            })
            .unwrap_err()
        ),
        "counter_overflow"
    );
}
#[test]
fn pages_cursors_generations_and_reads_never_recover_or_write() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "long.txt", &vec![b'a'; 4096 * 70]);
    let j = enqueue(&w, &a);
    enabled(&w);
    run(&w, &j.id);
    let first = fragments(&w, &j.id, 64, None);
    assert_eq!(first.items.len(), 64);
    assert!(!first.covers_generation);
    let last = fragments(&w, &j.id, 64, first.next_cursor.clone());
    assert_eq!(last.items.len(), 6);
    assert!(!last.covers_generation);
    assert!(last.next_cursor.is_none());
    let before = raw(&w.conn);
    get(&w, &j.id);
    fragments(&w, &j.id, 64, None);
    w.extraction_list(dto(json!({}))).unwrap();
    w.extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    assert_eq!(raw(&w.conn), before);
    for input in [
        json!({"id":j.id,"limit":0}),
        json!({"id":j.id,"limit":65}),
        json!({"id":j.id,"cursor":"{}"}),
        json!({"id":j.id,"limit":32,"cursor":first.next_cursor}),
    ] {
        assert!(w.extraction_fragments(dto(input)).is_err());
    }
    retry(&w, &j.id);
    run(&w, &j.id);
    assert!(
        w.extraction_fragments(ReadFragments {
            id: j.id.clone(),
            limit: Some(64),
            cursor: first.next_cursor
        })
        .is_err()
    );
    for index in 0..5 {
        let a = add(&mut w, &format!("{index}.txt"), b"q");
        enqueue(&w, &a);
    }
    w.conn
        .execute(
            "UPDATE extraction_jobs SET created_at='2026-10-13T00:00:00Z'",
            [],
        )
        .unwrap();
    let first = w.extraction_list(dto(json!({"limit":2}))).unwrap();
    let next = w
        .extraction_list(dto(json!({"limit":2,"cursor":first.next_cursor})))
        .unwrap();
    assert!(first.items[1].id < next.items[0].id);
    assert!(
        w.extraction_list(dto(json!({"limit":3,"cursor":first.next_cursor})))
            .is_err()
    );
    assert!(
        w.extraction_list(dto(
            json!({"limit":2,"state":"queued","cursor":first.next_cursor})
        ))
        .is_err()
    );
    assert!(
        w.extraction_list(dto(
            json!({"cursor":last.next_cursor.clone().unwrap_or_else(||"{}".into())})
        ))
        .is_err()
    );
    fs::write(w.path.join("attachments").join(&a.id), b"changed").unwrap();
    assert_eq!(
        reason(w.extraction_fragments(dto(json!({"id":j.id}))).unwrap_err()),
        "input_stale"
    );
    assert_eq!(get(&w, &j.id).state, ExtractionState::Succeeded);
    retry(&w, &j.id);
    assert_eq!(
        reason(
            w.extraction_run(RunExtraction {
                id: Some(j.id.clone())
            })
            .unwrap_err()
        ),
        "input_stale"
    );
    assert_eq!(get(&w, &j.id).state, ExtractionState::Stale);
}
#[test]
fn ownership_pause_resume_cancel_retry_aba_and_other_writer_are_fenced() {
    for mode in ["pause", "cancel", "reclaim", "expire", "foreign"] {
        let (_t, mut w) = setup();
        let a = add(&mut w, "a.txt", b"literal");
        let j = enqueue(&w, &a);
        enabled(&w);
        let mut other = Workspace::open(&w.path).unwrap();
        let mut hook = |stage: &str| {
            if stage == "claimed" {
                // Otro escritor puede editar mientras el worker está fuera del lock.
                other
                    .entity_create(
                        dto(json!({"kind":"note","title":"concurrent"})),
                        "test",
                        "other",
                    )
                    .unwrap();
                assert_eq!(
                    reason(other.extraction_run(RunExtraction::default()).unwrap_err()),
                    "worker_active"
                );
                let job = get(&other, &j.id);
                assert_eq!(job.state, ExtractionState::Running);
                assert!(job.ownership_active);
                assert_eq!(
                    reason(
                        other
                            .extraction_retry(ExtractionRevision {
                                id: j.id.clone(),
                                expected_revision: job.revision
                            })
                            .unwrap_err()
                    ),
                    "invalid_state"
                );
                match mode {
                    "pause" => {
                        let c = other
                            .extraction_control(ExtractionControlInput::Get {})
                            .unwrap();
                        other
                            .extraction_control(ExtractionControlInput::Set {
                                paused: true,
                                expected_revision: c.revision,
                            })
                            .unwrap();
                        assert_eq!(get(&other, &j.id).state, ExtractionState::Queued);
                        enabled(&other);
                    }
                    "cancel" | "reclaim" => {
                        other
                            .extraction_cancel(ExtractionRevision {
                                id: j.id.clone(),
                                expected_revision: job.revision,
                            })
                            .unwrap();
                        if mode == "reclaim" {
                            retry(&other, &j.id);
                            run(&other, &j.id);
                        }
                    }
                    "expire" | "foreign" => {
                        if mode == "expire" {
                            other.conn.execute("UPDATE extraction_runtime SET lease_until='2000-01-01T00:00:00Z'",[]).unwrap();
                        } else {
                            other
                                .conn
                                .execute("UPDATE extraction_runtime SET scope=?1", ["0".repeat(64)])
                                .unwrap();
                        }
                        let before = raw(&other.conn);
                        let job = get(&other, &j.id);
                        assert!(!job.ownership_active);
                        assert!(job.recoverable);
                        assert_eq!(raw(&other.conn), before);
                        let e = other
                            .extraction_run(RunExtraction {
                                id: Some(j.id.clone()),
                            })
                            .unwrap_err();
                        assert_eq!(e.details["state"], "failed");
                        assert_eq!(get(&other, &j.id).attempts, 1);
                    }
                    _ => unreachable!(),
                }
            }
        };
        let out = w
            .extraction_run_impl(
                RunExtraction {
                    id: Some(j.id.clone()),
                },
                Some(&mut hook),
                None,
            )
            .unwrap();
        assert!(matches!(out.outcome, ExtractionOutcome::Interrupted));
        let now = get(&w, &j.id);
        assert_eq!(now.attempts, if mode == "reclaim" { 2 } else { 1 });
        assert_eq!(now.generation.is_some(), mode == "reclaim");
        store::integrity(&w.conn).unwrap();
    }
}
#[test]
fn two_independent_connections_claim_once_with_barriers() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"literal");
    let j = enqueue(&w, &a);
    enabled(&w);
    let claimed = Arc::new(Barrier::new(2));
    let resume = Arc::new(Barrier::new(2));
    let (path, id, c, r) = (
        w.path.clone(),
        j.id.clone(),
        claimed.clone(),
        resume.clone(),
    );
    let worker = std::thread::spawn(move || {
        let w = Workspace::open(&path).unwrap();
        let mut h = |stage: &str| {
            if stage == "claimed" {
                c.wait();
                r.wait();
            }
        };
        w.extraction_run_impl(RunExtraction { id: Some(id) }, Some(&mut h), None)
            .unwrap()
    });
    claimed.wait();
    assert_eq!(
        reason(w.extraction_run(RunExtraction::default()).unwrap_err()),
        "worker_active"
    );
    w.entity_create(
        dto(json!({"kind":"note","title":"writer between claim/publication"})),
        "test",
        "parallel",
    )
    .unwrap();
    resume.wait();
    assert!(matches!(
        worker.join().unwrap().outcome,
        ExtractionOutcome::Succeeded
    ));
    assert_eq!(get(&w, &j.id).attempts, 1);
}
#[test]
fn publication_failure_deadline_stale_and_cleanup_pending_keep_previous_generation() {
    for mode in ["trigger", "deadline", "source", "busy"] {
        let (_t, mut w) = setup();
        let a = add(&mut w, "a.txt", b"valid previous");
        let j = enqueue(&w, &a);
        enabled(&w);
        let prior = run(&w, &j.id).generation.unwrap().id;
        retry(&w, &j.id);
        let other = Workspace::open(&w.path).unwrap();
        let mut locked = false;
        let mut h = |stage: &str| {
            if stage == "before_publish" {
                match mode {
                    "trigger"=>other.conn.execute_batch("CREATE TRIGGER fail_fragment BEFORE INSERT ON extraction_fragments BEGIN SELECT RAISE(ABORT,'synthetic'); END").unwrap(),
                    "source"=>fs::write(other.path.join("attachments").join(&a.id),b"invalid now").unwrap(),
                    "busy"=>{other.conn.execute_batch("BEGIN IMMEDIATE").unwrap();locked=true;}
                    _=>{},
                }
            }
        };
        let e = w
            .extraction_run_impl(
                RunExtraction {
                    id: Some(j.id.clone()),
                },
                Some(&mut h),
                if mode == "deadline" {
                    Some(Duration::ZERO)
                } else {
                    None
                },
            )
            .unwrap_err();
        if locked {
            other.conn.execute_batch("ROLLBACK").unwrap();
            assert_eq!(e.details["cleanup_pending"], true);
        } else {
            assert!(e.details.get("cleanup_pending").is_none());
        }
        let now = get(&w, &j.id);
        assert_eq!(now.generation.unwrap().id, prior);
        assert!(now.result_from_previous_attempt);
        assert_eq!(
            now.state,
            match mode {
                "source" => ExtractionState::Stale,
                "busy" => ExtractionState::Running,
                _ => ExtractionState::Failed,
            }
        );
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM extraction_generations", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            1
        );
        assert!(
            w.conn
                .query_row("SELECT 1", [], |r| r.get::<_, u32>(0))
                .is_ok()
        );
    }
}
#[test]
fn backup_restore_inherits_inactive_ownership_and_original_remains_live() {
    let (t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"backup original");
    let j = enqueue(&w, &a);
    enabled(&w);
    let prior = run(&w, &j.id).generation.unwrap().id;
    retry(&w, &j.id);
    let backup = t.path().join("backup");
    let dest = t.path().join("restore");
    let mut h = |stage: &str| {
        if stage == "claimed" {
            w.backup(&backup).unwrap();
            Workspace::restore(&backup, &dest).unwrap();
            let restored = Workspace::open(&dest).unwrap();
            let before = raw(&restored.conn);
            assert_eq!(before, raw(&w.conn)); // Incluye control, revisiones, token y generación.
            let inherited = get(&restored, &j.id);
            assert_eq!(inherited.state, ExtractionState::Running);
            assert!(inherited.recoverable);
            assert!(!inherited.ownership_active);
            assert_eq!(fragments(&restored, &j.id, 64, None).generation.id, prior);
            assert_eq!(raw(&restored.conn), before);
            let e = restored
                .extraction_run(RunExtraction {
                    id: Some(j.id.clone()),
                })
                .unwrap_err();
            assert_eq!(e.details["reason"], "scope_mismatch");
            assert!(get(&w, &j.id).ownership_active);
            assert_eq!(get(&restored, &j.id).revision, inherited.revision + 1);
            assert_eq!(get(&restored, &j.id).attempts, inherited.attempts);
        }
    };
    w.extraction_run_impl(
        RunExtraction {
            id: Some(j.id.clone()),
        },
        Some(&mut h),
        None,
    )
    .unwrap();
}
fn simple() -> BatchPlan {
    dto(
        json!({"version":1,"batch_id":"00000000-0000-0000-0000-000000000001","steps":[{"key":"note","operation":"entity.create","input":{"kind":"note","title":"batch original"}}]}),
    )
}
#[test]
fn batch_receipt_preserves_current_extraction_but_control_aba_conflicts() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"literal");
    enabled(&w);
    let plan = simple();
    let preview = batches::preview(&w.path, plan.clone(), "test", "preview").unwrap();
    let j = enqueue(&w, &a);
    let original = run(&w, &j.id).generation.unwrap().id;
    retry(&w, &j.id);
    let job = get(&w, &j.id);
    w.extraction_cancel(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: job.revision,
    })
    .unwrap();
    let before = get(&w, &j.id);
    batches::apply(
        &w.path,
        ApplyBatch {
            plan: plan.clone(),
            expected: preview.receipt,
        },
        "test",
        "apply",
    )
    .unwrap();
    let now = get(&w, &j.id);
    assert_eq!(now.revision, before.revision);
    assert_eq!(now.generation.unwrap().id, original);
    assert_eq!(now.state, ExtractionState::Cancelled);
    let mut plan = plan;
    plan.batch_id = "00000000-0000-0000-0000-000000000002".into();
    let preview = batches::preview(&w.path, plan.clone(), "test", "preview2").unwrap();
    let c = w
        .extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    w.extraction_control(ExtractionControlInput::Set {
        paused: true,
        expected_revision: c.revision,
    })
    .unwrap();
    enabled(&w);
    assert_eq!(
        reason(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan,
                    expected: preview.receipt
                },
                "test",
                "apply2"
            )
            .unwrap_err()
        ),
        "base_hash_mismatch"
    );
    let snapshot = batch::snapshot(&w.conn).unwrap();
    assert_eq!(raw(&snapshot), raw(&w.conn));
    w.conn
        .execute_batch("CREATE INDEX wrong_area ON extraction_jobs(state)")
        .unwrap();
    assert_eq!(
        reason(batch::structure(&w.conn).unwrap_err()),
        "unsupported_batch_schema"
    );
}

fn seed_jobs(w: &Workspace, a: &Attachment, count: u32, state: &str) {
    let tx = store::immediate(&w.conn).unwrap();
    let now = workspace::now();
    let base: u32 = tx
        .query_row("SELECT count(*) FROM extraction_jobs", [], |r| r.get(0))
        .unwrap();
    let mut insert=tx.prepare("INSERT INTO extraction_jobs(id,attachment_id,source_sha256,source_bytes,extractor,extractor_version,config,config_hash,state,revision,attempts,created_at,updated_at) VALUES (?1,?2,?3,?4,'future',1,?5,?6,?7,1,0,?8,?8)").unwrap();
    for index in 0..count {
        let config = json!({"version":2,"fixture":base+index});
        insert
            .execute(params![
                workspace::id(),
                a.id,
                a.sha256,
                a.bytes as u32,
                serde_json::to_string(&config).unwrap(),
                store::config_hash(&config).unwrap(),
                state,
                now
            ])
            .unwrap();
    }
    drop(insert);
    tx.commit().unwrap();
}
#[test]
fn pending_total_retry_and_concurrent_admission_respect_exact_caps() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "seed.txt", b"");
    seed_jobs(&w, &a, 999, "queued");
    let b = add(&mut w, "b.txt", b"b");
    let c = add(&mut w, "c.txt", b"c");
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = vec![];
    for a in [&b, &c] {
        let (p, id, bar) = (w.path.clone(), a.id.clone(), barrier.clone());
        handles.push(std::thread::spawn(move || {
            let w = Workspace::open(&p).unwrap();
            bar.wait();
            w.extraction_enqueue(dto(json!({"attachment_id":id,"extractor":"utf8_text"})))
        }));
    }
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    let rejected = results.iter().find_map(|r| r.as_ref().err()).unwrap();
    assert_eq!(rejected.details["reason"], "pending_count_limit");
    let j = results.into_iter().find_map(Result::ok).unwrap().job;
    assert!(
        w.extraction_enqueue(dto(
            json!({"attachment_id":j.attachment_id,"extractor":"utf8_text"})
        ))
        .unwrap()
        .deduplicated
    );
    w.extraction_cancel(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: 1,
    })
    .unwrap();
    retry(&w, &j.id);
    let rejected = if j.attachment_id == b.id { c } else { b };
    assert_eq!(
        reason(
            w.extraction_enqueue(dto(
                json!({"attachment_id":rejected.id,"extractor":"utf8_text"})
            ))
            .unwrap_err()
        ),
        "pending_count_limit"
    );
    w.conn
        .execute("UPDATE extraction_jobs SET state='cancelled'", [])
        .unwrap();
    seed_jobs(&w, &a, 8999, "cancelled");
    let accepted = enqueue(&w, &rejected);
    assert_eq!(
        w.conn
            .query_row("SELECT count(*) FROM extraction_jobs", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        10000
    );
    let extra = add(&mut w, "extra.txt", b"x");
    assert_eq!(
        reason(
            w.extraction_enqueue(dto(
                json!({"attachment_id":extra.id,"extractor":"utf8_text"})
            ))
            .unwrap_err()
        ),
        "job_count_limit"
    );
    assert_eq!(enqueue(&w, &rejected).id, accepted.id);
    assert_eq!(retry(&w, &j.id).attempts, 0);
}
#[test]
fn derived_cap_replacement_and_escaped_pages_use_real_generations() {
    let (_t, mut w) = setup();
    enabled(&w);
    let bytes = vec![1; MAX_EXTRACTION_BYTES as usize];
    let mut jobs = vec![];
    for index in 0..32 {
        let a = add(&mut w, &format!("{index}.txt"), &bytes);
        let j = enqueue(&w, &a);
        run(&w, &j.id);
        jobs.push(j);
    }
    assert_eq!(
        w.conn
            .query_row(
                "SELECT sum(covered_bytes) FROM extraction_generations",
                [],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
        MAX_DERIVED_BYTES as u32
    );
    let page = fragments(&w, &jobs[0].id, 64, None);
    let escaped = serde_json::to_vec(&page).unwrap();
    assert!(escaped.len() > 1_500_000);
    assert!(escaped.len() < EXTRACTION_RESPONSE_BYTES);
    retry(&w, &jobs[0].id);
    run(&w, &jobs[0].id); // replacement descuenta exactamente el mismo MiB
    let a = add(&mut w, "over.txt", b"x");
    let j = enqueue(&w, &a);
    assert_eq!(
        reason(
            w.extraction_run(RunExtraction {
                id: Some(j.id.clone())
            })
            .unwrap_err()
        ),
        "derived_size_limit"
    );
    assert!(get(&w, &j.id).generation.is_none());
    assert_eq!(get(&w, &j.id).state, ExtractionState::Failed);
    store::integrity(&w.conn).unwrap();
}
#[test]
fn unsafe_originals_and_expired_control_recovery_are_not_hidden_noops() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"original");
    let j = enqueue(&w, &a);
    let mut changed_during_read = false;
    let e = store::source(&w.path, &a, || {
        if !changed_during_read {
            fs::write(
                w.path.join("attachments").join(&a.id),
                vec![b'x'; MAX_EXTRACTION_BYTES as usize + 1],
            )?;
            changed_during_read = true;
        }
        Ok(())
    })
    .unwrap_err();
    assert_eq!(reason(e), "input_stale");
    fs::write(w.path.join("attachments").join(&a.id), b"original").unwrap();
    enabled(&w);
    run(&w, &j.id);
    let original = w.path.join("attachments").join(&a.id);
    let elsewhere = w.path.parent().unwrap().join("elsewhere");
    fs::rename(&original, &elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &original).unwrap();
    let before = raw(&w.conn);
    assert_eq!(
        reason(w.extraction_fragments(dto(json!({"id":j.id}))).unwrap_err()),
        "input_stale"
    );
    assert_eq!(raw(&w.conn), before);
    fs::remove_file(&original).unwrap();
    assert_eq!(
        reason(w.extraction_fragments(dto(json!({"id":j.id}))).unwrap_err()),
        "input_stale"
    );
    fs::rename(&elsewhere, &original).unwrap();
    retry(&w, &j.id);
    let other = Workspace::open(&w.path).unwrap();
    let mut h = |stage: &str| {
        if stage == "claimed" {
            other
                .conn
                .execute(
                    "UPDATE extraction_runtime SET lease_until='2000-01-01T00:00:00Z'",
                    [],
                )
                .unwrap();
            let before = raw(&other.conn);
            let c = other
                .extraction_control(ExtractionControlInput::Get {})
                .unwrap();
            other
                .extraction_control(ExtractionControlInput::Set {
                    paused: false,
                    expected_revision: c.revision,
                })
                .unwrap();
            assert_eq!(raw(&other.conn), before);
            other
                .extraction_control(ExtractionControlInput::Set {
                    paused: true,
                    expected_revision: c.revision,
                })
                .unwrap();
            assert_eq!(get(&other, &j.id).state, ExtractionState::Failed);
            assert_eq!(
                get(&other, &j.id).error.unwrap().details["reason"],
                "worker_interrupted"
            );
        }
    };
    assert!(matches!(
        w.extraction_run_impl(
            RunExtraction {
                id: Some(j.id.clone())
            },
            Some(&mut h),
            None
        )
        .unwrap()
        .outcome,
        ExtractionOutcome::Interrupted
    ));
}
#[test]
fn backup_rejects_missing_v8_table_matching_hash_and_retains_future_configs() {
    let (t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"backup");
    let j = enqueue(&w, &a);
    let config = json!({"version":7,"extension":{"raw":"preserve"}});
    let config_raw = "{\"version\":7, \"extension\":{\"raw\":\"preserve\"}}";
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?2,config_hash=?3 WHERE id=?1",
            params![j.id, config_raw, store::config_hash(&config).unwrap()],
        )
        .unwrap();
    let backup = t.path().join("backup");
    let dest = t.path().join("restore");
    w.backup(&backup).unwrap();
    Workspace::restore(&backup, &dest).unwrap();
    let restored = Workspace::open(&dest).unwrap();
    assert!(!get(&restored, &j.id).supported);
    assert_eq!(
        restored
            .conn
            .query_row(
                "SELECT config FROM extraction_jobs WHERE id=?1",
                [&j.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        config_raw
    );
    assert!(matches!(
        Workspace::restore(&backup, &dest).unwrap_err().code,
        ErrorCode::DestinationNotEmpty
    ));
    let bad = Connection::open(backup.join(DB_NAME)).unwrap();
    bad.execute_batch("DROP TABLE extraction_fragments")
        .unwrap();
    drop(bad);
    let manifest = backup.join("manifest.json");
    let mut v: Value = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    v["sha256"] = json!(workspace::hash(&backup.join(DB_NAME)).unwrap());
    fs::write(&manifest, serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(matches!(
        Workspace::restore(&backup, &t.path().join("bad-restore"))
            .unwrap_err()
            .code,
        ErrorCode::InvalidBackup
    ));
    assert!(!t.path().join("bad-restore").exists());
    w.conn
        .execute(
            "UPDATE extraction_jobs SET state='succeeded' WHERE id=?1",
            [&j.id],
        )
        .unwrap();
    assert_eq!(
        reason(store::integrity(&w.conn).unwrap_err()),
        "invalid_job_generation"
    );
    assert!(w.backup(&t.path().join("bad-source-backup")).is_err());
    assert!(!t.path().join("bad-source-backup").exists());
}
#[test]
fn final_lease_recheck_sql_interrupt_and_crash_do_not_confirm_results() {
    for mode in ["lease", "sql", "crash"] {
        let (_t, mut w) = setup();
        let a = add(&mut w, "a.txt", b"literal");
        let j = enqueue(&w, &a);
        enabled(&w);
        let mut clock = None;
        let mut h = |stage: &str| {
            if mode == "lease" && stage == "before_commit" {
                clock = Some(store::test_time(
                    chrono::Utc::now() + chrono::Duration::seconds(60),
                ));
            }
            if mode == "sql" && stage == "claimed" {
                let e=w.conn.query_row("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<1000000000) SELECT sum(x) FROM n",[],|r|r.get::<_,i64>(0)).unwrap_err();
                assert!(
                    matches!(e,rusqlite::Error::SqliteFailure(ref e,_) if e.code==rusqlite::ErrorCode::OperationInterrupted)
                );
            }
            if mode == "crash" && stage == "claimed" {
                panic!("synthetic worker crash after confirmed claim");
            }
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            w.extraction_run_impl(
                RunExtraction {
                    id: Some(j.id.clone()),
                },
                Some(&mut h),
                if mode == "sql" {
                    Some(Duration::from_millis(5))
                } else {
                    None
                },
            )
        }));
        drop(clock);
        if mode == "crash" {
            assert!(result.is_err());
            let opened = Workspace::open(&w.path).unwrap();
            let before = raw(&opened.conn);
            assert_eq!(get(&opened, &j.id).state, ExtractionState::Running);
            assert_eq!(raw(&opened.conn), before);
            w.conn
                .execute(
                    "UPDATE extraction_runtime SET lease_until='2000-01-01T00:00:00Z'",
                    [],
                )
                .unwrap();
            assert_eq!(
                w.extraction_run(RunExtraction::default())
                    .unwrap()
                    .recovered_count,
                1
            );
            assert_eq!(get(&w, &j.id).state, ExtractionState::Failed);
        } else {
            let e = result.unwrap().unwrap_err();
            assert_eq!(
                e.details["reason"],
                if mode == "lease" {
                    "worker_interrupted"
                } else {
                    "execution_deadline"
                }
            );
        }
        assert!(get(&w, &j.id).generation.is_none());
        assert!(
            w.conn
                .query_row("SELECT 1", [], |r| r.get::<_, u32>(0))
                .is_ok()
        );
    }
}
#[test]
fn concurrent_retry_uses_one_remaining_pending_slot() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"");
    let b = add(&mut w, "b.txt", b"");
    let x = enqueue(&w, &a);
    let y = enqueue(&w, &b);
    for j in [&x, &y] {
        w.extraction_cancel(ExtractionRevision {
            id: j.id.clone(),
            expected_revision: 1,
        })
        .unwrap();
    }
    seed_jobs(&w, &a, 999, "queued");
    let barrier = Arc::new(Barrier::new(3));
    let mut threads = vec![];
    for j in [x, y] {
        let (p, bar) = (w.path.clone(), barrier.clone());
        threads.push(std::thread::spawn(move || {
            let w = Workspace::open(&p).unwrap();
            bar.wait();
            w.extraction_retry(ExtractionRevision {
                id: j.id,
                expected_revision: 2,
            })
        }));
    }
    barrier.wait();
    let out = threads
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(out.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        out.iter().find_map(|r| r.as_ref().err()).unwrap().details["reason"],
        "pending_count_limit"
    );
}
#[test]
fn response_size_before_read_return_and_scoped_cursors_are_enforced() {
    let (t, mut w) = setup();
    let a = add(&mut w, "a.txt", &vec![b'x'; 4096 * 3]);
    let first = enqueue(&w, &a);
    enabled(&w);
    run(&w, &first.id);
    let b = add(&mut w, "b.txt", b"");
    enqueue(&w, &b);
    let cursor = w
        .extraction_list(dto(json!({"limit":1})))
        .unwrap()
        .next_cursor
        .unwrap();
    let fragment_cursor = fragments(&w, &first.id, 1, None).next_cursor.unwrap();
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let dest = t.path().join("restored");
    Workspace::restore(&backup, &dest).unwrap();
    let restored = Workspace::open(&dest).unwrap();
    assert_eq!(
        reason(
            restored
                .extraction_list(dto(json!({"limit":1,"cursor":cursor})))
                .unwrap_err()
        ),
        "cursor_scope_mismatch"
    );
    assert_eq!(
        reason(
            restored
                .extraction_fragments(dto(
                    json!({"id":first.id,"limit":1,"cursor":fragment_cursor})
                ))
                .unwrap_err()
        ),
        "cursor_scope_mismatch"
    );
    let mut wrong: Value = serde_json::from_str(&cursor).unwrap();
    wrong["operation"] = json!("home.query");
    assert!(
        w.extraction_list(dto(json!({"limit":1,"cursor":wrong.to_string()})))
            .is_err()
    );
    assert!(
        w.extraction_fragments(dto(json!({"id":first.id,"cursor":cursor})))
            .is_err()
    );
    let tx = store::immediate(&w.conn).unwrap();
    let now = workspace::now();
    for index in 0..100 {
        let id = workspace::id();
        let gid = workspace::id();
        let config = json!({"version":9,"fixture":index,"padding":"a".repeat(8000)});
        let raw = serde_json::to_string(&config).unwrap();
        let hash = store::config_hash(&config).unwrap();
        let error = CoreError {
            code: ErrorCode::ValidationError,
            message: "future error".into(),
            details: json!({"diagnostic":"x".repeat(3800)}),
        };
        tx.execute("INSERT INTO extraction_jobs(id,attachment_id,source_sha256,source_bytes,extractor,extractor_version,config,config_hash,state,revision,attempts,created_at,updated_at,error,generation_id) VALUES (?1,?2,?3,0,'future',9,?4,?5,'failed',1,1,?6,?6,?7,?8)",params![id,b.id,b.sha256,raw,hash,now,serde_json::to_string(&error).unwrap(),gid]).unwrap();
        tx.execute(
            "INSERT INTO extraction_generations VALUES (?1,?2,?3,?4,'future',9,?5,?6,0,0,0,?7,1)",
            params![gid, id, b.id, b.sha256, raw, hash, now],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    let before = raw(&w.conn);
    assert_eq!(
        reason(
            w.extraction_list(dto(json!({"state":"failed","limit":100})))
                .unwrap_err()
        ),
        "response_size_limit"
    );
    assert_eq!(raw(&w.conn), before);
    assert_eq!(reason(w.extraction_fragments(dto(json!({"id":w.extraction_list(dto(json!({"state":"failed","limit":1}))).unwrap().items[0].id}))).unwrap_err()),"unsupported_generation_config");
}
#[test]
#[ignore = "Fixture de proceso hijo; sólo se invoca desde su test supervisor"]
fn crash_worker_process_fixture() {
    let path = std::env::var_os("NOTEKA_EXTRACTION_TEST_PATH").expect("harness path");
    let id = std::env::var("NOTEKA_EXTRACTION_TEST_ID").expect("harness job");
    let w = Workspace::open(std::path::Path::new(&path)).unwrap();
    let mode = std::env::var("NOTEKA_EXTRACTION_TEST_MODE").unwrap_or_else(|_| "crash".into());
    if mode == "compete" {
        assert_eq!(
            reason(w.extraction_run(RunExtraction::default()).unwrap_err()),
            "worker_active"
        );
        std::process::exit(74);
    }
    let mut h = |stage: &str| {
        if stage == "claimed" {
            if mode == "crash" {
                std::process::exit(73);
            }
            use std::io::{Read, Write};
            println!("NOTEKA_TEST_CLAIMED");
            std::io::stdout().flush().unwrap();
            let mut byte = [0];
            std::io::stdin().read_exact(&mut byte).unwrap();
        }
    };
    w.extraction_run_impl(RunExtraction { id: Some(id) }, Some(&mut h), None)
        .unwrap();
    if mode == "crash" {
        panic!("claim hook was not reached");
    }
}
#[test]
fn two_real_worker_processes_share_one_ownership_and_allow_other_edits() {
    use std::io::{BufRead, Write};
    use std::process::{Command, Stdio};
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"independent processes");
    let j = enqueue(&w, &a);
    enabled(&w);
    let mut worker = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "extraction_tests::crash_worker_process_fixture",
            "--ignored",
            "--nocapture",
        ])
        .env("NOTEKA_EXTRACTION_TEST_PATH", &w.path)
        .env("NOTEKA_EXTRACTION_TEST_ID", &j.id)
        .env("NOTEKA_EXTRACTION_TEST_MODE", "hold")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = worker.stdout.take().unwrap();
    let mut reader = std::io::BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        assert!(
            reader.read_line(&mut line).unwrap() > 0,
            "worker exited before confirmed claim"
        );
        if line.trim() == "NOTEKA_TEST_CLAIMED" {
            break;
        }
    }
    let second = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "extraction_tests::crash_worker_process_fixture",
            "--ignored",
        ])
        .env("NOTEKA_EXTRACTION_TEST_PATH", &w.path)
        .env("NOTEKA_EXTRACTION_TEST_ID", &j.id)
        .env("NOTEKA_EXTRACTION_TEST_MODE", "compete")
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(74));
    w.entity_create(
        dto(json!({"kind":"note","title":"other process may edit"})),
        "test",
        "process-writer",
    )
    .unwrap();
    worker.stdin.take().unwrap().write_all(b"x").unwrap();
    let out = worker.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(get(&w, &j.id).attempts, 1);
    assert_eq!(get(&w, &j.id).state, ExtractionState::Succeeded);
}
#[test]
fn independent_process_crash_after_claim_requires_explicit_recovery_and_retry() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"process crash");
    let j = enqueue(&w, &a);
    enabled(&w);
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "extraction_tests::crash_worker_process_fixture",
            "--ignored",
        ])
        .env("NOTEKA_EXTRACTION_TEST_PATH", &w.path)
        .env("NOTEKA_EXTRACTION_TEST_ID", &j.id)
        .output()
        .unwrap();
    assert_eq!(status.status.code(), Some(73));
    let opened = Workspace::open(&w.path).unwrap();
    let before = raw(&opened.conn);
    let running = get(&opened, &j.id);
    assert_eq!(running.state, ExtractionState::Running);
    assert_eq!(running.attempts, 1);
    assert!(running.generation.is_none());
    assert_eq!(raw(&opened.conn), before);
    {
        let _clock = store::test_time(chrono::Utc::now() + chrono::Duration::seconds(60));
        let out = opened.extraction_run(RunExtraction::default()).unwrap();
        assert_eq!(out.recovered_count, 1);
        assert!(matches!(out.outcome, ExtractionOutcome::Idle));
        assert_eq!(get(&opened, &j.id).state, ExtractionState::Failed);
    }
    assert_eq!(get(&opened, &j.id).attempts, 1);
    retry(&opened, &j.id);
    assert_eq!(run(&opened, &j.id).attempts, 2);
}

#[test]
fn preview_keeps_expired_runtime_and_future_jobs_without_recovery() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"snapshot");
    let j = enqueue(&w, &a);
    enabled(&w);
    let b = add(&mut w, "b.txt", b"q");
    let future = enqueue(&w, &b);
    let config = json!({"version":9,"future":{"raw":"literal"}});
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?2,config_hash=?3 WHERE id=?1",
            params![
                future.id,
                serde_json::to_string(&config).unwrap(),
                store::config_hash(&config).unwrap()
            ],
        )
        .unwrap();
    let other = Workspace::open(&w.path).unwrap();
    let mut h = |stage: &str| {
        if stage == "claimed" {
            other
                .conn
                .execute(
                    "UPDATE extraction_runtime SET lease_until='2000-01-01T00:00:00Z'",
                    [],
                )
                .unwrap();
            let before = raw(&other.conn);
            batches::preview(&other.path, simple(), "test", "expired-preview").unwrap();
            assert_eq!(raw(&other.conn), before);
            assert_eq!(get(&other, &j.id).state, ExtractionState::Running);
            assert!(!get(&other, &future.id).supported);
            let snapshot = batch::snapshot(&other.conn).unwrap();
            assert_eq!(raw(&snapshot), before);
        }
    };
    w.extraction_run_impl(
        RunExtraction {
            id: Some(j.id.clone()),
        },
        Some(&mut h),
        None,
    )
    .unwrap_err();
    assert!(get(&w, &j.id).generation.is_none());
}
#[test]
fn publication_atomic_reader_and_growth_after_claim_preserve_prior_generation() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"consistent reader");
    let j = enqueue(&w, &a);
    enabled(&w);
    let previous = run(&w, &j.id).generation.unwrap().id;
    retry(&w, &j.id);
    let other = Workspace::open(&w.path).unwrap();
    let mut reader = None;
    let mut h = |stage: &str| {
        if stage == "before_commit" {
            reader = Some(other.conn.unchecked_transaction().unwrap());
            let page = store::job(reader.as_ref().unwrap(), &other.path, &j.id).unwrap();
            assert_eq!(page.generation.unwrap().id, previous);
        }
    };
    let next = w
        .extraction_run_impl(
            RunExtraction {
                id: Some(j.id.clone()),
            },
            Some(&mut h),
            None,
        )
        .unwrap()
        .job
        .unwrap()
        .generation
        .unwrap()
        .id;
    assert_ne!(previous, next);
    assert_eq!(
        store::job(reader.as_ref().unwrap(), &other.path, &j.id)
            .unwrap()
            .generation
            .unwrap()
            .id,
        previous
    );
    drop(reader);
    assert_eq!(get(&other, &j.id).generation.unwrap().id, next);
    retry(&w, &j.id);
    let mut h = |stage: &str| {
        if stage == "claimed" {
            fs::write(
                w.path.join("attachments").join(&a.id),
                vec![b'x'; MAX_EXTRACTION_BYTES as usize + 1],
            )
            .unwrap();
        }
    };
    assert_eq!(
        reason(
            w.extraction_run_impl(
                RunExtraction {
                    id: Some(j.id.clone())
                },
                Some(&mut h),
                None
            )
            .unwrap_err()
        ),
        "input_stale"
    );
    assert_eq!(get(&w, &j.id).generation.unwrap().id, next);
    assert_eq!(get(&w, &j.id).state, ExtractionState::Stale);
}
#[test]
fn fragment_constraints_and_physical_derivative_snapshot_cap_fail_closed() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "a.txt", b"literal");
    let j = enqueue(&w, &a);
    enabled(&w);
    let generation = run(&w, &j.id).generation.unwrap();
    for (ordinal, text) in [(512, "x".into()), (1, "x".repeat(4097))] {
        let e = w
            .conn
            .execute(
                "INSERT INTO extraction_fragments VALUES (?1,?2,?3,?4,0,?5,1,1)",
                params![generation.id, j.id, ordinal, text, text.len() as u32],
            )
            .unwrap_err();
        assert!(matches!(e,rusqlite::Error::SqliteFailure(ref e,_) if e.extended_code==275));
    }
    assert!(
        w.conn
            .execute("UPDATE extraction_generations SET fragment_count=513", [])
            .is_err()
    );
    // Páginas libres surgidas de actividad derivada también son SQLite del snapshot.
    // Fixture de presión física sin elevar el cap lógico ni modificar originales.
    let plan = simple();
    let preview = batches::preview(&w.path, plan.clone(), "test", "before-growth").unwrap();
    w.conn
        .execute_batch("PRAGMA foreign_keys=OFF;PRAGMA ignore_check_constraints=ON")
        .unwrap();
    w.conn
        .execute(
            "INSERT INTO extraction_fragments VALUES (?1,?2,1,zeroblob(?3),0,?3,1,1)",
            params![generation.id, j.id, 64 * 1024 * 1024],
        )
        .unwrap();
    w.conn
        .execute(
            "DELETE FROM extraction_fragments WHERE generation_id=?1 AND ordinal=1",
            [&generation.id],
        )
        .unwrap();
    w.conn
        .execute_batch("PRAGMA ignore_check_constraints=OFF;PRAGMA foreign_keys=ON")
        .unwrap();
    store::integrity(&w.conn).unwrap();
    let before = get(&w, &j.id).generation.unwrap().id;
    assert_eq!(
        reason(batches::preview(&w.path, simple(), "test", "physical-limit").unwrap_err()),
        "snapshot_size_limit"
    );
    assert_eq!(
        reason(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan,
                    expected: preview.receipt
                },
                "test",
                "physical-apply"
            )
            .unwrap_err()
        ),
        "snapshot_size_limit"
    );
    assert_eq!(get(&w, &j.id).generation.unwrap().id, before);
    assert_eq!(
        fs::read(w.path.join("attachments").join(&a.id)).unwrap(),
        b"literal"
    );
}
#[test]
fn extraction_operations_are_rejected_as_batch_steps_and_checks_before_any_effect() {
    let (_t, w) = setup();
    let before = raw(&w.conn);
    for action in [
        "enqueue",
        "get",
        "list",
        "run",
        "cancel",
        "retry",
        "control",
        "fragments",
        "pdf_fragments",
        "index",
        "search",
    ] {
        let mut plan = serde_json::to_value(simple()).unwrap();
        plan["steps"][0]["operation"] = json!(format!("extraction.{action}"));
        let result = crate::execute_json(
            json!({"workspace":w.path,"command":{"operation":"changes.preview","input":plan}}),
            "test",
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().exit_code(), 2);
        assert_eq!(raw(&w.conn), before);
        let mut plan = serde_json::to_value(simple()).unwrap();
        plan["checks"] = json!([{"key":"invalid","input":{"operation":format!("extraction.{action}"),"input":{}}}]);
        let result = crate::execute_json(
            json!({"workspace":w.path,"command":{"operation":"changes.preview","input":plan}}),
            "test",
        );
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().exit_code(), 2);
        assert_eq!(raw(&w.conn), before);
    }
}
#[test]
fn concurrent_duplicate_enqueue_and_removed_note_reference_keep_original_provenance() {
    let (_t, mut w) = setup();
    let a = add(&mut w, "source.txt", b"reference independent");
    let barrier = Arc::new(Barrier::new(3));
    let mut threads = vec![];
    for _ in 0..2 {
        let (path, id, bar) = (w.path.clone(), a.id.clone(), barrier.clone());
        threads.push(std::thread::spawn(move || {
            let w = Workspace::open(&path).unwrap();
            bar.wait();
            w.extraction_enqueue(dto(json!({"attachment_id":id,"extractor":"utf8_text"})))
                .unwrap()
        }));
    }
    barrier.wait();
    let jobs = threads
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(jobs[0].job.id, jobs[1].job.id);
    assert_eq!(jobs.iter().filter(|j| j.deduplicated).count(), 1);
    assert_eq!(jobs[0].job.revision, 1);
    let j = &jobs[0].job;
    let note=w.entity_create(dto(json!({"kind":"note","title":"reference","document":{"schema_version":2,"blocks":[{"id":"file","type":"file","content":[],"attrs":{"attachment_id":a.id,"name":"source.txt"}}]}})),"test","note").unwrap();
    enabled(&w);
    run(&w, &j.id);
    w.entity_update(
        dto(
            json!({"id":note.id,"expected_revision":1,"document":{"schema_version":2,"blocks":[]}}),
        ),
        "test",
        "remove-ref",
    )
    .unwrap();
    let before = raw(&w.conn);
    let page = fragments(&w, &j.id, 64, None);
    assert_eq!(page.generation.attachment_id, a.id);
    assert_eq!(page.generation.source_sha256, a.sha256);
    assert_eq!(page.items[0].text, "reference independent");
    let serialized = serde_json::to_value(&page).unwrap();
    assert!(serialized.get("entity_id").is_none());
    assert!(serialized["items"][0].get("entity_id").is_none());
    assert_eq!(raw(&w.conn), before);
}

#[test]
#[ignore = "Medición release explícita; corpus sintético fuera de Git"]
fn release_extraction_resources() {
    if cfg!(debug_assertions) {
        panic!("usar cargo test --release");
    }
    fn distribution(mut values: Vec<f64>) -> Value {
        values.sort_by(f64::total_cmp);
        json!({"p50_ms":values[values.len()/2],"p95_ms":values[(values.len()*95).div_ceil(100)-1],"max_ms":values[values.len()-1]})
    }
    for case in ["ascii", "multibyte", "escaped", "saturated"] {
        let (t, mut w) = setup();
        enabled(&w);
        let bytes = match case {
            "multibyte" => {
                let mut b = "é世😀\n"
                    .repeat(MAX_EXTRACTION_BYTES as usize / 10)
                    .into_bytes();
                b.resize(MAX_EXTRACTION_BYTES as usize, b'x');
                b
            }
            "escaped" => vec![1; MAX_EXTRACTION_BYTES as usize],
            _ => vec![b'x'; MAX_EXTRACTION_BYTES as usize],
        };
        let a = add(&mut w, "fixture.txt", &bytes);
        let j = enqueue(&w, &a);
        run(&w, &j.id);
        if case == "saturated" {
            for n in 1..32 {
                let a = add(&mut w, &format!("corpus-{n}.txt"), &bytes);
                let j = enqueue(&w, &a);
                run(&w, &j.id);
            }
            seed_jobs(&w, &a, 999, "queued");
            seed_jobs(&w, &a, 8969, "cancelled");
        }
        let mut measures = std::collections::BTreeMap::<String, Vec<f64>>::new();
        let mut response_bytes = 0;
        let mut fragment_response_bytes = 0;
        for index in 0..25 {
            retry(&w, &j.id);
            let start = std::time::Instant::now();
            let mut phases = std::collections::BTreeMap::new();
            let mut h = |stage: &str| {
                phases.insert(stage.to_string(), start.elapsed().as_secs_f64() * 1000.0);
            };
            let out = w
                .extraction_run_impl(
                    RunExtraction {
                        id: if case == "saturated" {
                            None
                        } else {
                            Some(j.id.clone())
                        },
                    },
                    Some(&mut h),
                    None,
                )
                .unwrap();
            let fstart = std::time::Instant::now();
            let f = fragments(&w, &j.id, 64, None);
            let fragment_ms = fstart.elapsed().as_secs_f64() * 1000.0;
            response_bytes=serde_json::to_vec(&json!({"api_version":1,"request_id":"00000000-0000-0000-0000-000000000000","ok":true,"data":out})).unwrap().len();
            fragment_response_bytes=serde_json::to_vec(&json!({"api_version":1,"request_id":"00000000-0000-0000-0000-000000000000","ok":true,"data":f})).unwrap().len();
            if index >= 5 {
                for (name, from, to) in [
                    ("claim_lock", "claim_locked", "claim_committed"),
                    ("read_hash", "claimed", "source_read"),
                    ("fragmentation", "source_read", "extracted"),
                    ("publication_lock", "publish_locked", "committed"),
                    ("total", "before_claim", "committed"),
                ] {
                    measures
                        .entry(name.into())
                        .or_default()
                        .push(phases[to] - phases[from]);
                }
                measures
                    .entry("fragments_read".into())
                    .or_default()
                    .push(fragment_ms);
            }
        }
        let page_count: u32 = w
            .conn
            .pragma_query_value(None, "page_count", |r| r.get(0))
            .unwrap();
        let page_size: u32 = w
            .conn
            .pragma_query_value(None, "page_size", |r| r.get(0))
            .unwrap();
        let counts:(u32,u32,u32)=w.conn.query_row("SELECT (SELECT count(*) FROM extraction_jobs),(SELECT count(*) FROM extraction_jobs WHERE state IN ('queued','running')),(SELECT coalesce(sum(covered_bytes),0) FROM extraction_generations)",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
        let stats = measures
            .into_iter()
            .map(|(k, v)| (k, distribution(v)))
            .collect::<std::collections::BTreeMap<_, _>>();
        println!(
            "EXTRACTION_METRICS {}",
            json!({"case":case,"workspace":w.path,"job_id":j.id,"warmups":5,"repetitions":20,"source_bytes":bytes.len(),"run_response_bytes":response_bytes,"fragment_response_bytes":fragment_response_bytes,"jobs":counts.0,"pending_after":counts.1,"retained_bytes":counts.2,"page_count":page_count,"page_size":page_size,"sqlite_bytes":u64::from(page_count)*u64::from(page_size),"phases":stats})
        );
        if std::env::var_os("NOTEKA_EXTRACTION_BENCH_KEEP").is_some() {
            let _ = t.keep();
        }
    }
}
