use crate::{
    batch_storage, batches,
    contracts::*,
    extraction_storage as store,
    extraction_tests::raw,
    workspace::{self, Workspace},
};
use rusqlite::params;
use serde_json::json;
use std::fs;

fn dto<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
pub(crate) fn source(pages: &[Vec<u8>]) -> Vec<u8> {
    let mut objects = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        Vec::new(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
    ];
    let mut kids = Vec::new();
    for p in pages {
        let id = objects.len() + 1;
        kids.push(format!("{id} 0 R"));
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] /Contents {} 0 R >>",
                id + 1
            )
            .into_bytes(),
        );
        let content = if p.is_empty() {
            Vec::new()
        } else {
            format!(
                "BT /F1 12 Tf <{}> Tj ET\n",
                p.iter().map(|n| format!("{n:02x}")).collect::<String>()
            )
            .into_bytes()
        };
        let mut stream = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        stream.extend(content);
        stream.extend(b"\nendstream");
        objects.push(stream);
    }
    objects[1] = format!(
        "<< /Type /Pages /Count {} /Kids [{}] /Resources << /Font << /F1 3 0 R >> >> >>",
        pages.len(),
        kids.join(" ")
    )
    .into_bytes();
    let mut bytes = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (n, o) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend(format!("{} 0 obj\n", n + 1).as_bytes());
        bytes.extend(o);
        bytes.extend(b"\nendobj\n");
    }
    let start = bytes.len();
    bytes.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
    for n in offsets {
        bytes.extend(format!("{n:010} 00000 n \n").as_bytes());
    }
    bytes.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{start}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}
fn add(w: &mut Workspace, name: &str, bytes: &[u8], extractor: &str) -> ExtractionJob {
    let p = w.path.parent().unwrap().join(name);
    fs::write(&p, bytes).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: p.to_string_lossy().into(),
        })
        .unwrap();
    w.extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":extractor})))
        .unwrap()
        .job
}
fn enable(w: &Workspace) {
    let c = w
        .extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    w.extraction_control(ExtractionControlInput::Set {
        paused: false,
        expected_revision: c.revision,
    })
    .unwrap();
}
fn run(w: &Workspace, j: &ExtractionJob) -> ExtractionJob {
    w.extraction_run(RunExtraction {
        id: Some(j.id.clone()),
    })
    .unwrap()
    .job
    .unwrap()
}
fn retry(w: &Workspace, j: &ExtractionJob) -> ExtractionJob {
    let j = w.extraction_get(IdInput { id: j.id.clone() }).unwrap();
    w.extraction_retry(ExtractionRevision {
        id: j.id,
        expected_revision: j.revision,
    })
    .unwrap()
}

#[test]
fn pdf_admission_safe_source_dedup_future_and_paused_never_launch_helper() {
    use std::sync::{Arc, Barrier};
    let (_t, mut w) = setup();
    let bytes = source(&[b"PDF".to_vec()]);
    let launches = crate::pdf_supervisor::spawn_count();
    let j = add(&mut w, "a.PDF", &bytes, "pdf_text");
    w.extraction_run(RunExtraction::default()).unwrap();
    assert_eq!(crate::pdf_supervisor::spawn_count(), launches);
    let mut a = crate::attachments::metadata(&w.conn, &j.attachment_id).unwrap();
    for mime in ["application/pdf", "application/octet-stream"] {
        a.mime = mime.into();
        store::eligible_for(&a, "pdf_text").unwrap();
    }
    for (name, mime) in [
        ("a.txt", "application/pdf"),
        ("a.pdf", "text/plain"),
        ("a", "application/pdf"),
        ("a.pdf.exe", "application/pdf"),
    ] {
        a.name = name.into();
        a.mime = mime.into();
        assert_eq!(
            store::eligible_for(&a, "pdf_text").unwrap_err().details["reason"],
            "unsupported_source_format"
        );
        w.conn
            .execute(
                "UPDATE attachments SET name=?1,mime=?2 WHERE id=?3",
                params![name, mime, a.id],
            )
            .unwrap();
        let before = raw(&w.conn);
        assert_eq!(
            w.extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"pdf_text"})))
                .unwrap_err()
                .details["reason"],
            "unsupported_source_format"
        );
        assert_eq!(raw(&w.conn), before);
    }
    w.conn
        .execute(
            "UPDATE attachments SET name='a.PDF',mime='application/pdf' WHERE id=?1",
            [&a.id],
        )
        .unwrap();
    a = crate::attachments::metadata(&w.conn, &a.id).unwrap();
    let path = w.path.join("attachments").join(&a.id);
    let mut changed = false;
    assert_eq!(
        store::source(&w.path, &a, || {
            if !changed {
                fs::write(&path, vec![b'x'; 1_048_577])?;
                changed = true;
            }
            Ok(())
        })
        .unwrap_err()
        .details["reason"],
        "input_stale"
    );
    fs::write(&path, &bytes).unwrap();
    let mut corrupted = bytes.clone();
    corrupted[20] ^= 1;
    fs::write(&path, corrupted).unwrap();
    assert_eq!(
        store::source(&w.path, &a, || Ok(())).unwrap_err().details["reason"],
        "input_stale"
    );
    fs::write(&path, &bytes).unwrap();
    let elsewhere = path.with_extension("elsewhere");
    fs::rename(&path, &elsewhere).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &path).unwrap();
    assert_eq!(
        store::source(&w.path, &a, || Ok(())).unwrap_err().details["reason"],
        "input_stale"
    );
    fs::remove_file(&path).unwrap();
    fs::rename(elsewhere, path).unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let threads = (0..2)
        .map(|_| {
            let barrier = barrier.clone();
            let root = w.path.clone();
            let id = a.id.clone();
            std::thread::spawn(move || {
                let w = Workspace::open(&root).unwrap();
                barrier.wait();
                w.extraction_enqueue(dto(json!({"attachment_id":id,"extractor":"pdf_text"})))
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    for t in threads {
        let out = t.join().unwrap();
        assert!(out.deduplicated);
        assert_eq!(out.job.id, j.id);
    }
    let raw_config = "{ \"version\":9, \"future\":{\"opaque\":true} }";
    let value: serde_json::Value = serde_json::from_str(raw_config).unwrap();
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?1,config_hash=?2 WHERE id=?3",
            params![raw_config, store::config_hash(&value).unwrap(), j.id],
        )
        .unwrap();
    enable(&w);
    assert!(
        !w.extraction_get(IdInput { id: j.id.clone() })
            .unwrap()
            .supported
    );
    assert_eq!(
        w.extraction_run(RunExtraction { id: Some(j.id) })
            .unwrap_err()
            .details["reason"],
        "unsupported_extractor_config"
    );
    assert_eq!(
        w.conn
            .query_row("SELECT config FROM extraction_jobs", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        raw_config
    );
    assert_eq!(crate::pdf_supervisor::spawn_count(), launches);
}

#[test]
fn pdf_source_change_and_each_table_failure_roll_back_before_cleanup_recovery() {
    for failure in [
        "source_read",
        "before_publish",
        "extraction_pdf_generations",
        "extraction_pdf_pages",
        "extraction_pdf_fragments",
        "cleanup",
    ] {
        let (_t, mut w) = setup();
        let bytes = source(&[b"Original".to_vec()]);
        let j = add(&mut w, "a.pdf", &bytes, "pdf_text");
        enable(&w);
        let prior = run(&w, &j).generation.unwrap().id;
        retry(&w, &j);
        let path = w.path.join("attachments").join(&j.attachment_id);
        if failure.starts_with("extraction_pdf_") {
            w.conn.execute_batch(&format!("CREATE TEMP TRIGGER fail_pdf BEFORE INSERT ON {failure} BEGIN SELECT RAISE(ABORT,'test'); END")).unwrap();
        }
        let mut hook = |at: &str| {
            if at == failure || (failure == "cleanup" && at == "source_read") {
                let mut altered = bytes.clone();
                altered[20] ^= 1;
                fs::write(&path, altered).unwrap();
            }
            if failure == "cleanup" && at == "before_cleanup" {
                w.conn.execute_batch("CREATE TEMP TRIGGER fail_cleanup BEFORE UPDATE ON extraction_jobs WHEN NEW.state='stale' BEGIN SELECT RAISE(ABORT,'test'); END").unwrap();
            }
        };
        let e = w
            .extraction_run_impl(
                RunExtraction {
                    id: Some(j.id.clone()),
                },
                Some(&mut hook),
                None,
            )
            .unwrap_err();
        assert_eq!(e.details["job_id"], j.id);
        assert_eq!(e.details["attempt"], 2);
        fs::write(&path, &bytes).unwrap();
        assert_eq!(read(&w, &j, 64, None).generation.id, prior);
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM extraction_pdf_generations", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            1
        );
        if failure == "cleanup" {
            assert_eq!(e.details["cleanup_pending"], true);
            w.conn.execute_batch("DROP TRIGGER fail_cleanup").unwrap();
            let clock = store::test_time(chrono::Utc::now() + chrono::Duration::seconds(60));
            w.extraction_run(RunExtraction::default()).unwrap();
            drop(clock);
            retry(&w, &j);
            run(&w, &j);
        }
    }
}

#[test]
fn pdf_batches_snapshot_all_tables_reject_stale_control_source_schema_and_physical_overflow() {
    let (_t, mut w) = setup();
    let j = add(
        &mut w,
        "a.pdf",
        &source(&[b"Before preview".to_vec(), vec![]]),
        "pdf_text",
    );
    enable(&w);
    run(&w, &j);
    let mut plan: BatchPlan = dto(
        json!({"version":1,"batch_id":"00000000-0000-0000-0000-000000000017","steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"snapshot"}}]}),
    );
    let before = raw(&w.conn);
    let preview = batches::preview(&w.path, plan.clone(), "test", "pdf").unwrap();
    assert_eq!(raw(&w.conn), before);
    let snap = batch_storage::snapshot(&w.conn).unwrap();
    let copied = raw(&snap);
    for name in [
        "extraction_pdf_generations",
        "extraction_pdf_pages",
        "extraction_pdf_fragments",
    ] {
        assert_eq!(
            before.iter().find(|(n, _)| n == name).unwrap(),
            copied.iter().find(|(n, _)| n == name).unwrap()
        );
    }
    batches::apply(
        &w.path,
        ApplyBatch {
            plan: plan.clone(),
            expected: preview.receipt,
        },
        "test",
        "pdf",
    )
    .unwrap();
    for mode in ["control_aba", "source"] {
        plan.batch_id = workspace::id();
        let p = batches::preview(&w.path, plan.clone(), "test", "before").unwrap();
        if mode == "control_aba" {
            let c = w
                .extraction_control(ExtractionControlInput::Get {})
                .unwrap();
            let c = w
                .extraction_control(ExtractionControlInput::Set {
                    paused: true,
                    expected_revision: c.revision,
                })
                .unwrap();
            w.extraction_control(ExtractionControlInput::Set {
                paused: false,
                expected_revision: c.revision,
            })
            .unwrap();
        } else {
            w.conn
                .execute(
                    "UPDATE attachments SET sha256=printf('%064d',0) WHERE id=?1",
                    [&j.attachment_id],
                )
                .unwrap();
        }
        let before = raw(&w.conn);
        assert_eq!(
            batches::apply(
                &w.path,
                ApplyBatch {
                    plan: plan.clone(),
                    expected: p.receipt
                },
                "test",
                "stale"
            )
            .unwrap_err()
            .details["reason"],
            "base_hash_mismatch"
        );
        assert_eq!(raw(&w.conn), before);
        if mode == "source" {
            w.conn
                .execute(
                    "UPDATE attachments SET sha256=?1 WHERE id=?2",
                    params![j.source_sha256, j.attachment_id],
                )
                .unwrap();
        }
    }
    for sql in [
        "ALTER TABLE extraction_pdf_pages ADD COLUMN future TEXT",
        "DROP TABLE extraction_pdf_pages",
    ] {
        let (_t, mut other) = setup();
        let j = add(&mut other, "a.pdf", &source(&[b"X".to_vec()]), "pdf_text");
        enable(&other);
        run(&other, &j);
        other.conn.execute_batch(sql).unwrap();
        let before = raw(&other.conn);
        assert_eq!(
            batch_storage::structure(&other.conn).unwrap_err().details["reason"],
            "unsupported_batch_schema"
        );
        let e = batches::preview(&other.path, plan.clone(), "test", "bad").unwrap_err();
        if sql.starts_with("DROP") {
            assert!(matches!(e.code, crate::error::ErrorCode::StorageError));
        } else {
            assert_eq!(e.details["reason"], "unsupported_batch_schema");
        }
        assert_eq!(raw(&other.conn), before);
    }
    for cap in [16384, 16385] {
        w.conn.pragma_update(None, "max_page_count", cap).unwrap();
        w.conn
            .execute_batch("CREATE TABLE padding(v BLOB)")
            .unwrap();
        while w
            .conn
            .execute("INSERT INTO padding VALUES(zeroblob(1048576))", [])
            .is_ok()
        {}
        while w
            .conn
            .execute("INSERT INTO padding VALUES(zeroblob(3000))", [])
            .is_ok()
        {}
        w.conn.execute_batch("DROP TABLE padding").unwrap();
        assert_eq!(
            w.conn
                .pragma_query_value(None, "page_count", |r| r.get::<_, u32>(0))
                .unwrap(),
            cap
        );
        if cap == 16384 {
            let snap = batch_storage::snapshot(&w.conn).unwrap();
            assert_eq!(
                snap.query_row("SELECT count(*) FROM extraction_pdf_pages", [], |r| r
                    .get::<_, u32>(0))
                    .unwrap(),
                2
            );
        } else {
            assert_eq!(
                batch_storage::snapshot(&w.conn).unwrap_err().details["reason"],
                "snapshot_size_limit"
            );
        }
    }
}
fn read(w: &Workspace, j: &ExtractionJob, limit: u32, cursor: Option<String>) -> PdfFragmentPage {
    w.extraction_pdf_fragments(ReadPdfFragments {
        id: j.id.clone(),
        limit: Some(limit),
        cursor,
    })
    .unwrap()
}

#[test]
fn real_pdf_pagination_reconstruction_and_literal_contract_remain_separate() {
    let (_t, mut w) = setup();
    let j = add(
        &mut w,
        "multi.PDF",
        &source(&[vec![0xe9; 4096 * 33], vec![], b"Last".to_vec()]),
        "pdf_text",
    );
    assert_eq!(
        w.extraction_run(RunExtraction::default())
            .unwrap()
            .reason
            .as_deref(),
        Some("queue_paused")
    );
    assert_eq!(
        w.extraction_get(IdInput { id: j.id.clone() })
            .unwrap()
            .attempts,
        0
    );
    enable(&w);
    let hash =
        batch_storage::base_hash(&w.conn, &batch_storage::Budget::new(&w.conn).unwrap()).unwrap();
    let originals = raw(&w.conn)
        .into_iter()
        .filter(|(n, _)| !n.starts_with("extraction_"))
        .collect::<Vec<_>>();
    let done = run(&w, &j);
    assert_eq!(done.state, ExtractionState::Succeeded);
    assert_eq!(
        hash,
        batch_storage::base_hash(&w.conn, &batch_storage::Budget::new(&w.conn).unwrap()).unwrap()
    );
    assert_eq!(
        originals,
        raw(&w.conn)
            .into_iter()
            .filter(|(n, _)| !n.starts_with("extraction_"))
            .collect::<Vec<_>>()
    );
    let first = read(&w, &j, 32, None);
    assert_eq!(first.manifest.page_count, 3);
    assert!(!first.covers_generation);
    assert_eq!(first.items.len(), 32);
    let mut text = first
        .items
        .iter()
        .map(|f| f.text.as_str())
        .collect::<String>();
    let mut next = first.next_cursor.clone();
    while let Some(c) = next {
        let p = read(&w, &j, 32, Some(c));
        assert!(!p.covers_generation);
        text.extend(p.items.into_iter().map(|f| f.text));
        next = p.next_cursor;
    }
    assert_eq!(text, format!("{}\nLast\n", "é".repeat(4096 * 33)));
    assert!(first.generation.complete);
    assert_eq!(
        first.generation.covered_bytes,
        first.generation.source_bytes
    );
    assert_ne!(
        first.manifest.extracted_bytes,
        first.generation.covered_bytes
    );
    assert_eq!(
        w.extraction_fragments(ReadFragments {
            id: j.id.clone(),
            limit: None,
            cursor: None
        })
        .unwrap_err()
        .details["reason"],
        "unsupported_generation_config"
    );
    let literal = add(&mut w, "literal.txt", "é😀\r\n".as_bytes(), "utf8_text");
    run(&w, &literal);
    assert_eq!(
        w.extraction_pdf_fragments(ReadPdfFragments {
            id: literal.id,
            limit: None,
            cursor: None
        })
        .unwrap_err()
        .details["reason"],
        "unsupported_generation_config"
    );
    retry(&w, &j);
    assert!(read(&w, &j, 64, None).result_from_previous_attempt);
    run(&w, &j);
    assert_eq!(
        w.extraction_pdf_fragments(ReadPdfFragments {
            id: j.id.clone(),
            limit: Some(32),
            cursor: first.next_cursor
        })
        .unwrap_err()
        .details["reason"],
        "cursor_scope_mismatch"
    );
    let empty = add(&mut w, "empty.pdf", &source(&[vec![]]), "pdf_text");
    run(&w, &empty);
    let p = read(&w, &empty, 64, None);
    assert!(p.items.is_empty() && p.covers_generation);
    assert_eq!(p.manifest.pages[0].state, PdfPageState::NoText);
}

#[test]
fn pdf_backup_integrity_and_read_snapshot_are_pure_and_corruption_fails_closed() {
    let (t, mut w) = setup();
    let j = add(&mut w, "a.pdf", &source(&[vec![b'a'; 9000]]), "pdf_text");
    enable(&w);
    run(&w, &j);
    let queued = add(&mut w, "queued.pdf", &source(&[vec![]]), "pdf_text");
    let first = read(&w, &j, 1, None);
    let before = raw(&w.conn);
    read(&w, &j, 64, None);
    assert_eq!(raw(&w.conn), before);
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let restored = t.path().join("restored");
    Workspace::restore(&backup, &restored).unwrap();
    let other = Workspace::open(&restored).unwrap();
    assert_eq!(read(&other, &j, 64, None).manifest.extracted_bytes, 9001);
    assert_eq!(
        other
            .extraction_pdf_fragments(ReadPdfFragments {
                id: j.id.clone(),
                limit: Some(1),
                cursor: first.next_cursor
            })
            .unwrap_err()
            .details["reason"],
        "cursor_scope_mismatch"
    );
    let repair_hash = format!(
        "UPDATE extraction_pdf_pages SET text_sha256='{}'",
        first.manifest.pages[0].text_sha256
    );
    for (sql, repair) in [
        (
            "UPDATE extraction_pdf_pages SET text_sha256=printf('%064d',0)",
            repair_hash.as_str(),
        ),
        (
            "UPDATE extraction_pdf_generations SET extracted_bytes=extracted_bytes+1",
            "UPDATE extraction_pdf_generations SET extracted_bytes=extracted_bytes-1",
        ),
        (
            "UPDATE extraction_pdf_fragments SET text_byte_start=text_byte_start+1,text_byte_end=text_byte_end+1 WHERE ordinal=0",
            "UPDATE extraction_pdf_fragments SET text_byte_start=text_byte_start-1,text_byte_end=text_byte_end-1 WHERE ordinal=0",
        ),
        (
            "INSERT INTO extraction_fragments SELECT generation_id,job_id,0,'x',0,1,1,1 FROM extraction_pdf_generations",
            "DELETE FROM extraction_fragments",
        ),
        (
            "UPDATE extraction_pdf_fragments SET text='b'||substr(text,2) WHERE ordinal=0",
            "UPDATE extraction_pdf_fragments SET text='a'||substr(text,2) WHERE ordinal=0",
        ),
    ] {
        // Daño confirmado: no envolver el lector en SAVEPOINT (su BEGIN fallaría
        // por nesting, ocultando el motivo real de integridad que debe probarse).
        w.conn.execute_batch(sql).unwrap();
        let damaged = raw(&w.conn);
        assert!(
            w.extraction_pdf_fragments(ReadPdfFragments {
                id: j.id.clone(),
                limit: None,
                cursor: None
            })
            .is_err(),
            "{sql}"
        );
        assert!(store::integrity(&w.conn).is_err(), "{sql}");
        let e = w
            .extraction_run(RunExtraction {
                id: Some(queued.id.clone()),
            })
            .unwrap_err();
        assert!(
            e.details["reason"]
                .as_str()
                .unwrap()
                .starts_with("invalid_pdf"),
            "{sql}"
        );
        assert_eq!(
            w.extraction_get(IdInput {
                id: queued.id.clone()
            })
            .unwrap()
            .attempts,
            0
        );
        assert_eq!(raw(&w.conn), damaged);
        w.conn.execute_batch(repair).unwrap();
    }
    let bytes = fs::read(w.path.join("attachments").join(&j.attachment_id)).unwrap();
    fs::write(w.path.join("attachments").join(&j.attachment_id), b"stale").unwrap();
    assert!(
        w.extraction_pdf_fragments(ReadPdfFragments {
            id: j.id.clone(),
            limit: None,
            cursor: None
        })
        .is_err()
    );
    fs::write(w.path.join("attachments").join(&j.attachment_id), bytes).unwrap();
    assert_eq!(raw(&w.conn), before);
}

#[test]
fn pdf_publication_fences_preserve_prior_generation_at_every_stage() {
    for stage in [
        "source_read",
        "extracted",
        "before_publish",
        "before_commit",
    ] {
        let (_t, mut w) = setup();
        let j = add(&mut w, "a.pdf", &source(&[b"Prior".to_vec()]), "pdf_text");
        enable(&w);
        let prior = run(&w, &j).generation.unwrap().id;
        retry(&w, &j);
        let mut hook = |at: &str| {
            if at == stage {
                if at == "before_commit" {
                    w.conn.execute_batch("SELECT 1").unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(60));
                } else {
                    let other = Workspace::open(&w.path).unwrap();
                    let c = other
                        .extraction_control(ExtractionControlInput::Get {})
                        .unwrap();
                    other
                        .extraction_control(ExtractionControlInput::Set {
                            paused: true,
                            expected_revision: c.revision,
                        })
                        .unwrap();
                }
            }
        };
        let result = w.extraction_run_impl(
            RunExtraction {
                id: Some(j.id.clone()),
            },
            Some(&mut hook),
            if stage == "before_commit" {
                Some(std::time::Duration::from_millis(50))
            } else {
                None
            },
        );
        assert!(
            result.is_err() || matches!(result.unwrap().outcome, ExtractionOutcome::Interrupted)
        );
        assert_eq!(read(&w, &j, 64, None).generation.id, prior);
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM extraction_pdf_generations", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            1
        );
    }
}

#[test]
fn v8_migration_is_additive_transactional_and_does_not_recover_running_jobs() {
    let (t, mut w) = setup();
    let literal = add(&mut w, "a.txt", b"original", "utf8_text");
    enable(&w);
    run(&w, &literal);
    w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;UPDATE workspace_meta SET schema_version=8;PRAGMA user_version=8").unwrap();
    w.conn
        .execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations")
        .unwrap();
    let before = raw(&w.conn);
    let e = w.migrate(true).unwrap_err();
    assert_eq!(workspace::version(&w.conn).unwrap(), 8);
    assert_eq!(raw(&w.conn), before);
    let backup = std::path::Path::new(e.details["backup_path"].as_str().unwrap());
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(backup.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], 8);
    w.migrate(false).unwrap();
    assert_eq!(
        workspace::version(&w.conn).unwrap(),
        workspace::SCHEMA_VERSION
    );
    let after = raw(&w.conn);
    for (name, rows) in before {
        if name != "workspace_meta" {
            assert_eq!(
                &rows,
                &after.iter().find(|(n, _)| n == &name).unwrap().1,
                "{name}"
            );
        }
    }
    assert_eq!(
        w.extraction_fragments(ReadFragments {
            id: literal.id,
            limit: None,
            cursor: None
        })
        .unwrap()
        .items[0]
            .text,
        "original"
    );
    w.conn.execute_batch(&format!("UPDATE workspace_meta SET schema_version={0};PRAGMA user_version={0};PRAGMA wal_checkpoint(TRUNCATE)",workspace::SCHEMA_VERSION+1)).unwrap();
    let path = w.path.clone();
    drop(w);
    let bytes = fs::read(path.join(workspace::DB_NAME)).unwrap();
    assert!(Workspace::open(&path).is_err());
    assert_eq!(fs::read(path.join(workspace::DB_NAME)).unwrap(), bytes);
    drop(t);
}

#[test]
fn mixed_pdf_batch_snapshot_preserves_new_results_and_excludes_only_derivatives() {
    let (_t, mut w) = setup();
    let j = add(
        &mut w,
        "a.pdf",
        &source(&[b"Original".to_vec()]),
        "pdf_text",
    );
    enable(&w);
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":"00000000-0000-0000-0000-000000000001","steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"mixed"}}]}),
    );
    let preview = batches::preview(&w.path, plan.clone(), "test", "preview").unwrap();
    let id = run(&w, &j).generation.unwrap().id;
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
    assert_eq!(read(&w, &j, 64, None).generation.id, id);
    let snap = batch_storage::snapshot(&w.conn).unwrap();
    assert_eq!(
        snap.query_row("SELECT text FROM extraction_pdf_fragments", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "Original\n"
    );
    assert_eq!(plan.version, 1);
    // Las tablas completas entran al snapshot, pero no a la base de originales.
    let h =
        batch_storage::base_hash(&w.conn, &batch_storage::Budget::new(&w.conn).unwrap()).unwrap();
    w.conn
        .execute(
            "UPDATE extraction_pdf_fragments SET text=?1 WHERE ordinal=0",
            params!["Altered!\n"],
        )
        .unwrap();
    assert_eq!(
        h,
        batch_storage::base_hash(&w.conn, &batch_storage::Budget::new(&w.conn).unwrap()).unwrap()
    );
    assert!(store::integrity(&w.conn).is_err());
}

#[test]
fn mixed_text_budget_counts_actual_utf8_bytes_not_pdf_source_and_replaces_once() {
    let (_t, mut w) = setup();
    let literal = add(&mut w, "literal.txt", &vec![b'x'; 1_048_576], "utf8_text");
    enable(&w);
    run(&w, &literal);
    // Semilla SQLite estructural: no pretende ser evidencia del parser. Permite
    // fijar exactamente 32 MiB sin 31 ejecuciones/recompresiones redundantes.
    let prepared = crate::pdf_storage::prepare(vec![noteka_pdf_protocol::PageText {
        page_number: 1,
        text: "é".repeat(524_288),
    }])
    .unwrap();
    let mut first = None;
    for n in 0..31 {
        let j = add(
            &mut w,
            &format!("seed-{n}.pdf"),
            &source(&[b"Seed".to_vec()]),
            "pdf_text",
        );
        let id = workspace::id();
        let tx = store::immediate(&w.conn).unwrap();
        tx.execute("INSERT INTO extraction_generations(id,job_id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,covered_bytes,fragment_count,published_at,attempt) VALUES (?1,?2,?3,?4,'pdf_text',1,?5,?6,?7,?7,?8,?9,1)",params![id,j.id,j.attachment_id,j.source_sha256,serde_json::to_string(&j.config).unwrap(),j.config_hash,j.source_bytes,prepared.fragments.len() as u32,workspace::now()]).unwrap();
        crate::pdf_storage::publish(&tx, &id, &j.id, &prepared).unwrap();
        tx.execute("UPDATE extraction_jobs SET state='succeeded',attempts=1,revision=3,generation_id=?2 WHERE id=?1",params![j.id,id]).unwrap();
        tx.commit().unwrap();
        if first.is_none() {
            first = Some(j);
        }
    }
    assert_eq!(
        crate::pdf_storage::retained(&w.conn, "").unwrap(),
        MAX_DERIVED_BYTES
    );
    store::integrity(&w.conn).unwrap();
    let empty = add(&mut w, "no-text.pdf", &source(&[vec![]]), "pdf_text");
    run(&w, &empty);
    assert_eq!(read(&w, &empty, 64, None).manifest.extracted_bytes, 0);
    let over = add(&mut w, "over.pdf", &source(&[b"X".to_vec()]), "pdf_text");
    let before = crate::pdf_storage::retained(&w.conn, "").unwrap();
    assert_eq!(
        w.extraction_run(RunExtraction { id: Some(over.id) })
            .unwrap_err()
            .details["reason"],
        "derived_size_limit"
    );
    assert_eq!(crate::pdf_storage::retained(&w.conn, "").unwrap(), before);
    let first = first.unwrap();
    retry(&w, &first);
    run(&w, &first);
    assert_eq!(read(&w, &first, 64, None).manifest.extracted_bytes, 5);
    assert_eq!(
        crate::pdf_storage::retained(&w.conn, "").unwrap(),
        MAX_DERIVED_BYTES - 1_048_576 + 5
    );
}

#[test]
fn pdf_backups_during_prepare_publish_commit_restore_one_fenced_generation() {
    let (t, mut w) = setup();
    let j = add(&mut w, "a.pdf", &source(&[b"Prior".to_vec()]), "pdf_text");
    enable(&w);
    let prior = run(&w, &j).generation.unwrap().id;
    retry(&w, &j);
    let other = Workspace::open(&w.path).unwrap();
    let mut hook = |stage: &str| {
        if !["source_read", "before_commit", "committed"].contains(&stage) {
            return;
        }
        let backup = t.path().join(format!("backup-{stage}"));
        other.backup(&backup).unwrap();
        let dest = t.path().join(format!("restored-{stage}"));
        Workspace::restore(&backup, &dest).unwrap();
        let restored = Workspace::open(&dest).unwrap();
        let before = raw(&restored.conn);
        let p = read(&restored, &j, 64, None);
        assert_eq!(raw(&restored.conn), before);
        if stage != "committed" {
            assert_eq!(p.generation.id, prior);
            let running = restored
                .extraction_get(IdInput { id: j.id.clone() })
                .unwrap();
            assert!(running.recoverable && !running.ownership_active);
            assert_eq!(
                restored
                    .extraction_run(RunExtraction {
                        id: Some(j.id.clone())
                    })
                    .unwrap_err()
                    .details["reason"],
                "scope_mismatch"
            );
            assert!(
                other
                    .extraction_get(IdInput { id: j.id.clone() })
                    .unwrap()
                    .ownership_active
            );
        } else {
            assert_ne!(p.generation.id, prior);
        }
    };
    w.extraction_run_impl(
        RunExtraction {
            id: Some(j.id.clone()),
        },
        Some(&mut hook),
        None,
    )
    .unwrap();
    for (index, sql) in [
        "UPDATE extraction_pdf_pages SET text_sha256=printf('%064d',0)",
        "DROP TABLE extraction_pdf_pages",
    ]
    .iter()
    .enumerate()
    {
        let backup = t.path().join(format!("bad-{index}"));
        w.backup(&backup).unwrap();
        let conn = rusqlite::Connection::open(backup.join(workspace::DB_NAME)).unwrap();
        conn.execute_batch(sql).unwrap();
        drop(conn);
        use sha2::{Digest, Sha256};
        let hash = format!(
            "{:x}",
            Sha256::digest(fs::read(backup.join(workspace::DB_NAME)).unwrap())
        );
        let mp = backup.join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&mp).unwrap()).unwrap();
        manifest["sha256"] = json!(hash);
        fs::write(mp, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let dest = t.path().join(format!("rejected-{index}"));
        assert!(Workspace::restore(&backup, &dest).is_err());
        assert!(!dest.exists());
    }
}

#[test]
#[ignore = "Fixture de proceso hijo para supervisión PDF; se invoca desde su test padre"]
fn pdf_process_fixture() {
    use std::io::{Read, Write};
    let path = std::env::var("NOTEKA_EXTRACTION_TEST_PATH").unwrap();
    let id = std::env::var("NOTEKA_EXTRACTION_TEST_ID").unwrap();
    crate::pdf_supervisor::input_hook(|pid| {
        println!("NOTEKA_PDF_INPUT_SENT {pid}");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_exact(&mut [0]).unwrap();
    });
    let w = Workspace::open(std::path::Path::new(&path)).unwrap();
    let out = w.extraction_run(RunExtraction { id: Some(id) });
    // El padre verifica SQLite y el resultado final. No propagar raw objetos.
    if let Err(e) = out {
        println!("NOTEKA_PDF_ERROR {}", e.details["reason"]);
    }
}

#[test]
fn two_real_pdf_processes_edit_pause_cancel_aba_and_parent_kill_eight_rounds_each() {
    use std::io::{BufRead, Write};
    use std::process::{Command, Stdio};
    struct Guard(std::process::Child);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    for round in 0..40 {
        let (_t, mut w) = setup();
        let j = add(&mut w, "a.pdf", &source(&[vec![b'x'; 48_000]]), "pdf_text");
        enable(&w);
        let prior = run(&w, &j).generation.unwrap().id;
        retry(&w, &j);
        let child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "pdf_tests::pdf_process_fixture",
                "--ignored",
                "--nocapture",
            ])
            .env("NOTEKA_EXTRACTION_TEST_PATH", &w.path)
            .env("NOTEKA_EXTRACTION_TEST_ID", &j.id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut child = Guard(child);
        let mut reader = std::io::BufReader::new(child.0.stdout.take().unwrap());
        let mut line = String::new();
        let helper_pid = loop {
            line.clear();
            assert!(
                reader.read_line(&mut line).unwrap() > 0,
                "worker exited before input barrier"
            );
            if let Some(raw) = line.trim().strip_prefix("NOTEKA_PDF_INPUT_SENT ") {
                break raw.parse::<i32>().unwrap();
            }
        };
        // Segundo proceso independiente no puede reclamar el mismo intento.
        let competitor = Command::new(std::env::current_exe().unwrap())
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
        assert_eq!(competitor.status.code(), Some(74));
        if round == 0 {
            let backup = w.path.parent().unwrap().join("active-helper-backup");
            w.backup(&backup).unwrap();
            let dest = w.path.parent().unwrap().join("active-helper-restore");
            Workspace::restore(&backup, &dest).unwrap();
            let restored = Workspace::open(&dest).unwrap();
            assert_eq!(read(&restored, &j, 64, None).generation.id, prior);
            let inherited = restored
                .extraction_get(IdInput { id: j.id.clone() })
                .unwrap();
            assert!(inherited.recoverable && !inherited.ownership_active);
        }
        w.entity_create(
            dto(json!({"kind":"note","title":"edit during helper IO"})),
            "test",
            "pdf-concurrent-edit",
        )
        .unwrap();
        match round % 5 {
            0 => {}
            1 | 3 => {
                let c = w
                    .extraction_control(ExtractionControlInput::Get {})
                    .unwrap();
                w.extraction_control(ExtractionControlInput::Set {
                    paused: true,
                    expected_revision: c.revision,
                })
                .unwrap();
            }
            2 => {
                let running = w.extraction_get(IdInput { id: j.id.clone() }).unwrap();
                w.extraction_cancel(ExtractionRevision {
                    id: j.id.clone(),
                    expected_revision: running.revision,
                })
                .unwrap();
            }
            _ => {
                child.0.kill().unwrap();
                child.0.wait().unwrap();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(7);
                loop {
                    // PID sólo del helper recién creado por esta fixture, no ownership.
                    if unsafe { libc::kill(helper_pid, 0) } != 0 {
                        assert_eq!(
                            std::io::Error::last_os_error().raw_os_error(),
                            Some(libc::ESRCH)
                        );
                        break;
                    }
                    assert!(
                        std::time::Instant::now() < deadline,
                        "orphan helper remained resident"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(25));
                }
                let opened = Workspace::open(&w.path).unwrap();
                let before = raw(&opened.conn);
                assert_eq!(read(&opened, &j, 64, None).generation.id, prior);
                assert_eq!(raw(&opened.conn), before);
                let _clock = store::test_time(chrono::Utc::now() + chrono::Duration::seconds(60));
                opened.extraction_run(RunExtraction::default()).unwrap();
                drop(_clock);
                retry(&opened, &j);
                run(&opened, &j);
                continue;
            }
        }
        if round % 5 == 3 {
            let c = w
                .extraction_control(ExtractionControlInput::Get {})
                .unwrap();
            w.extraction_control(ExtractionControlInput::Set {
                paused: false,
                expected_revision: c.revision,
            })
            .unwrap();
        }
        child.0.stdin.take().unwrap().write_all(b"x").unwrap();
        assert!(child.0.wait().unwrap().success());
        let current = read(&w, &j, 64, None);
        if round % 5 == 0 {
            assert_ne!(current.generation.id, prior);
        } else {
            assert_eq!(current.generation.id, prior);
        }
        let job = w.extraction_get(IdInput { id: j.id.clone() }).unwrap();
        assert_eq!(job.attempts, 2);
        assert!(!job.ownership_active);
    }
}

mod measurements {
    use super::*;
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        sync::atomic::{AtomicUsize, Ordering},
        time::Instant,
    };
    struct Measured;
    static LIVE: AtomicUsize = AtomicUsize::new(0);
    static PEAK: AtomicUsize = AtomicUsize::new(0);
    fn account(n: usize) {
        let previous = LIVE.fetch_add(n, Ordering::SeqCst);
        PEAK.fetch_max(previous + n, Ordering::SeqCst);
    }
    // SAFETY: sólo en el binario de unit tests. Se delega el contrato íntegro a
    // System, con Layout/punteros originales; no se cambia alineación ni cap.
    // Se cuentan Layouts Rust vivos, no malloc/SQLite/stack/RSS. Ninguna cuenta
    // asigna memoria ni impide dealloc/realloc; realloc fallido conserva la cuenta.
    unsafe impl GlobalAlloc for Measured {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let p = unsafe { System.alloc(l) };
            if !p.is_null() {
                account(l.size());
            }
            p
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            let p = unsafe { System.alloc_zeroed(l) };
            if !p.is_null() {
                account(l.size());
            }
            p
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            unsafe { System.dealloc(p, l) };
            LIVE.fetch_sub(l.size(), Ordering::SeqCst);
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            let next = unsafe { System.realloc(p, l, n) };
            if !next.is_null() {
                if n > l.size() {
                    account(n - l.size());
                } else {
                    LIVE.fetch_sub(l.size() - n, Ordering::SeqCst);
                }
            }
            next
        }
    }
    #[global_allocator]
    static ALLOCATOR: Measured = Measured;
    fn rss(who: libc::c_int) -> u64 {
        // SAFETY: rusage inicializado/escribible, selector constante de libc.
        let mut r: libc::rusage = unsafe { std::mem::zeroed() };
        assert_eq!(unsafe { libc::getrusage(who, &mut r) }, 0);
        r.ru_maxrss as u64 * if cfg!(target_os = "macos") { 1 } else { 1024 }
    }
    fn percentile(mut values: Vec<f64>, q: f64) -> f64 {
        values.sort_by(f64::total_cmp);
        values[((values.len() as f64 * q).ceil() as usize).saturating_sub(1)]
    }
    fn seed(w: &mut Workspace, bytes: &[u8]) {
        let p = crate::pdf_storage::prepare(vec![noteka_pdf_protocol::PageText {
            page_number: 1,
            text: "é".repeat(524_288),
        }])
        .unwrap();
        for n in 0..31 {
            let j = add(w, &format!("saturation-{n}.pdf"), bytes, "pdf_text");
            let id = workspace::id();
            let tx = store::immediate(&w.conn).unwrap();
            tx.execute("INSERT INTO extraction_generations(id,job_id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,covered_bytes,fragment_count,published_at,attempt) VALUES (?1,?2,?3,?4,'pdf_text',1,?5,?6,?7,?7,?8,?9,1)",params![id,j.id,j.attachment_id,j.source_sha256,serde_json::to_string(&j.config).unwrap(),j.config_hash,j.source_bytes,p.fragments.len() as u32,workspace::now()]).unwrap();
            crate::pdf_storage::publish(&tx, &id, &j.id, &p).unwrap();
            tx.execute("UPDATE extraction_jobs SET state='succeeded',attempts=1,revision=3,generation_id=?2 WHERE id=?1",params![j.id,id]).unwrap();
            tx.commit().unwrap();
        }
    }
    #[test]
    #[ignore = "Medición release explícita con corpus externo y 5 warmups/20 repeticiones"]
    fn release_pdf_resources() {
        if cfg!(debug_assertions) {
            panic!("Usar --release");
        }
        use std::io::Write;
        use std::process::{Command, Stdio};
        let root = std::path::PathBuf::from(std::env::var("NOTEKA_PDF_BENCH_ROOT").unwrap());
        assert!(root.is_absolute());
        let corpus: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("corpus-manifest.json")).unwrap()).unwrap();
        let cases = corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c.get("pages").is_some())
            .collect::<Vec<_>>();
        let largest = *cases
            .iter()
            .max_by_key(|c| {
                c["pages"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| p["text_bytes"].as_u64().unwrap())
                    .sum::<u64>()
            })
            .unwrap();
        let mut records = Vec::new();
        for (case, saturated) in cases
            .iter()
            .copied()
            .map(|c| (c, false))
            .chain(std::iter::once((largest, true)))
        {
            let name = case["name"].as_str().unwrap();
            let bytes = fs::read(root.join(format!("{name}.pdf"))).unwrap();
            let (_t, mut w) = setup();
            enable(&w);
            if saturated {
                seed(&mut w, &bytes);
            }
            let j = add(&mut w, "measured.pdf", &bytes, "pdf_text");
            let mut samples = Vec::new();
            let mut heap_peak = 0;
            for n in 0..25 {
                if n > 0 {
                    retry(&w, &j);
                }
                PEAK.store(LIVE.load(Ordering::SeqCst), Ordering::SeqCst);
                let began = Instant::now();
                let mut stages = std::collections::BTreeMap::new();
                let mut h = |stage: &str| {
                    stages.insert(stage.to_owned(), began.elapsed().as_secs_f64() * 1000.0);
                };
                w.extraction_run_impl(
                    RunExtraction {
                        id: Some(j.id.clone()),
                    },
                    Some(&mut h),
                    None,
                )
                .unwrap();
                let elapsed = began.elapsed().as_secs_f64() * 1000.0;
                if n >= 5 {
                    heap_peak = heap_peak.max(PEAK.load(Ordering::SeqCst));
                    samples.push([
                        elapsed,
                        stages["claim_committed"] - stages["before_claim"],
                        stages["committed"] - stages["before_publish"],
                        stages["claim_committed"] - stages["claim_locked"],
                        stages["committed"] - stages["publish_locked"],
                    ]);
                }
            }
            let page = read(&w, &j, 64, None);
            let expected = case["pages"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p["text_bytes"].as_u64().unwrap())
                .sum::<u64>();
            assert_eq!(u64::from(page.manifest.extracted_bytes), expected);
            let mut timings = serde_json::Map::new();
            for (i, label) in [
                "run",
                "claim",
                "publication",
                "claim_lock",
                "publication_lock",
            ]
            .iter()
            .enumerate()
            {
                let values = samples.iter().map(|s| s[i]).collect::<Vec<_>>();
                timings.insert(label.to_string(),json!({"p50_ms":percentile(values.clone(),0.5),"p95_ms":percentile(values,0.95)}));
            }
            let metric_exe = std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("examples/runtime_metrics");
            let mut child = Command::new(metric_exe)
                .env_clear()
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            {
                let mut input = child.stdin.take().unwrap();
                input
                    .write_all(
                        &noteka_pdf_protocol::Header {
                            source_bytes: bytes.len() as u32,
                            remaining_ms: 5000,
                        }
                        .encode()
                        .unwrap(),
                    )
                    .unwrap();
                input.write_all(&bytes).unwrap();
            }
            let out = child.wait_with_output().unwrap();
            assert!(out.status.success());
            let helper: serde_json::Value = serde_json::from_slice(&out.stderr).unwrap();
            let reply: noteka_pdf_protocol::Reply = serde_json::from_slice(&out.stdout).unwrap();
            reply.validate().unwrap();
            match reply.outcome {
                noteka_pdf_protocol::Outcome::Success { pages, .. } => assert_eq!(
                    pages.iter().map(|p| p.text.len() as u64).sum::<u64>(),
                    expected
                ),
                _ => panic!("runtime metrics failed"),
            };
            assert!(helper["heap_peak_requested_bytes"].as_u64().unwrap() <= 134_217_728);
            let pages: u64 = w
                .conn
                .pragma_query_value(None, "page_count", |r| r.get::<_, i64>(0))
                .unwrap() as u64;
            let size: u64 = w
                .conn
                .pragma_query_value(None, "page_size", |r| r.get::<_, i64>(0))
                .unwrap() as u64;
            records.push(json!({"case":name,"saturated_31_mib_fixture":saturated,"source_bytes":bytes.len(),"source_sha256":case["source_sha256"],"text_bytes":page.manifest.extracted_bytes,"private_reply_bytes":out.stdout.len(),"public_page_bytes":serde_json::to_vec(&page).unwrap().len(),"sqlite_bytes":pages*size,"parent_heap_requested_peak_bytes":heap_peak,"parent_peak_rss_bytes":rss(libc::RUSAGE_SELF),"waited_children_peak_rss_bytes":rss(libc::RUSAGE_CHILDREN),"helper_runtime_probe":helper,"timings":timings}));
        }
        let lock_p95 = records
            .iter()
            .flat_map(|r| {
                [
                    r["timings"]["claim_lock"]["p95_ms"].as_f64().unwrap(),
                    r["timings"]["publication_lock"]["p95_ms"].as_f64().unwrap(),
                ]
            })
            .fold(0.0, f64::max);
        println!(
            "NOTEKA_PDF_BENCH {}",
            json!({"warmups":5,"repetitions":20,"cases":records,"maximum_lock_p95_ms":lock_p95,"lock_target_ms":150,"lock_target_met":lock_p95<=150.0})
        );
        assert!(
            lock_p95 <= 150.0,
            "Gate lock p95 falló: investigar antes de cerrar"
        );
    }
}
