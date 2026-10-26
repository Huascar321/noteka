//! Pruebas de persistencia con datos sintéticos; la inferencia nativa se prueba por CLI.
use crate::{
    contracts::*,
    extraction_storage as store, pdfkit_storage as ps,
    workspace::{self, Workspace},
};
use noteka_pdfkit_protocol::{Document, Engine, LIMITATIONS, Page, PageStatus};
use rusqlite::params;
use serde_json::{Value, json};
use std::fs;
fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
pub(crate) fn document(n: u32) -> Document {
    let pages: Vec<_> = (1..=n)
        .map(|number| Page {
            number,
            status: if number % 2 == 0 {
                PageStatus::NoText
            } else {
                PageStatus::Text
            },
            text: if number % 2 == 0 {
                String::new()
            } else {
                format!("evidencia {number} café 漢 🚀\r\n<script>")
            },
        })
        .collect();
    Document {
        version: 1,
        engine: Engine {
            api: "PDFKit.PDFPage.attributedString".into(),
            os_version: "13.0.0".into(),
            os_build: "22A380".into(),
        },
        limitations: LIMITATIONS.iter().map(|s| s.to_string()).collect(),
        page_count: n,
        text_bytes: pages.iter().map(|p| p.text.len() as u32).sum(),
        pages,
    }
}
pub(crate) fn seed(w: &mut Workspace, d: Document) -> ExtractionJob {
    let source = w
        .path
        .parent()
        .unwrap()
        .join(format!("{}.pdf", workspace::id()));
    fs::write(
        &source,
        crate::pdf_tests::source(&[b"Synthetic PDFKit fence".to_vec()]),
    )
    .unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"pdfkit_text"})))
        .unwrap()
        .job;
    let p = ps::prepare(d).unwrap();
    let id = workspace::id();
    let tx = w.conn.unchecked_transaction().unwrap();
    tx.execute("INSERT INTO extraction_generations SELECT ?1,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?2,created_at,1 FROM extraction_jobs WHERE id=?3",params![id,p.chunks.len() as u32,j.id]).unwrap();
    tx.execute(
        "UPDATE extraction_jobs SET state='succeeded',generation_id=?1,attempts=1 WHERE id=?2",
        params![id, j.id],
    )
    .unwrap();
    ps::publish(&tx, &id, &j.id, &p).unwrap();
    tx.commit().unwrap();
    w.extraction_get(IdInput { id: j.id }).unwrap()
}
fn read(
    w: &Workspace,
    j: &ExtractionJob,
    limit: u32,
    cursor: Option<String>,
) -> crate::error::Result<PdfkitPagePage> {
    w.extraction_pdfkit_pages(ReadPdfkitPages {
        id: j.id.clone(),
        limit: Some(limit),
        cursor,
    })
}
#[test]
fn all_physical_pages_empty_text_unicode_and_cursor_scope_survive_reopen() {
    let (_t, mut w) = setup();
    let d = document(100);
    let j = seed(&mut w, d.clone());
    let p = read(&w, &j, 32, None).unwrap();
    assert_eq!(p.items, d.pages[..32]);
    assert!(!p.covers_generation);
    assert!(read(&w, &j, 20, p.next_cursor.clone()).is_err());
    let mut items = p.items;
    let mut cursor = p.next_cursor;
    while cursor.is_some() {
        let p = read(&w, &j, 32, cursor).unwrap();
        items.extend(p.items);
        cursor = p.next_cursor;
    }
    assert_eq!(items, d.pages);
    assert_eq!(
        read(&w, &j, 64, None).unwrap().manifest.text_bytes,
        d.text_bytes
    );
    for limit in [0, 65] {
        assert!(read(&w, &j, limit, None).is_err());
    }
    let other = seed(&mut w, document(1));
    let mut c: Value =
        serde_json::from_str(&read(&w, &j, 32, None).unwrap().next_cursor.unwrap()).unwrap();
    c["job_id"] = json!(other.id);
    assert!(read(&w, &j, 32, Some(c.to_string())).is_err());
    w.extraction_retry(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: j.revision,
    })
    .unwrap();
    assert!(read(&w, &j, 64, None).unwrap().result_from_previous_attempt);
    let path = w.path.clone();
    drop(w);
    let w = Workspace::open(&path).unwrap();
    assert_eq!(read(&w, &j, 32, None).unwrap().items, d.pages[..32]);
}
#[test]
fn corruption_any_chunk_descriptor_or_owner_rejects_the_entire_generation() {
    for sql in [
        "UPDATE extraction_pdfkit_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_pdfkit_chunks)",
        "UPDATE extraction_pdfkit_generations SET page_count=page_count-1",
        "UPDATE extraction_pdfkit_generations SET text_bytes=text_bytes+1",
        "UPDATE extraction_pdfkit_generations SET profile='future'",
        "UPDATE extraction_pdfkit_chunks SET ordinal=ordinal+100",
        "DELETE FROM extraction_pdfkit_chunks WHERE ordinal=1",
        "PRAGMA foreign_keys=OFF;UPDATE extraction_pdfkit_chunks SET job_id='foreign';PRAGMA foreign_keys=ON",
    ] {
        let (_t, mut w) = setup();
        let j = seed(&mut w, document(100));
        w.conn.execute_batch(sql).unwrap();
        assert!(read(&w, &j, 1, None).is_err(), "{sql}");
        assert!(ps::integrity(&w.conn).is_err(), "{sql}");
    }
}
#[test]
fn backup_restore_snapshots_originals_and_retention_include_canonical_json() {
    let (t, mut w) = setup();
    let j = seed(&mut w, document(70));
    let manifest = read(&w, &j, 32, None).unwrap().manifest;
    assert_eq!(
        ps::retained(&w.conn, "").unwrap(),
        u64::from(manifest.representation_bytes)
    );
    assert_eq!(ps::retained(&w.conn, &j.id).unwrap(), 0);
    let o = crate::ocr_tests::seed(&mut w, crate::ocr_tests::recognition(2));
    assert!(ps::retained(&w.conn, "").unwrap() > u64::from(manifest.representation_bytes));
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"preservar"}}]}),
    );
    let preview = crate::batches::preview(&w.path, plan.clone(), "test", "pdfkit").unwrap();
    crate::batches::apply(
        &w.path,
        ApplyBatch {
            plan,
            expected: preview.receipt,
        },
        "test",
        "pdfkit",
    )
    .unwrap();
    assert_eq!(read(&w, &j, 32, None).unwrap().manifest, manifest);
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let restored = t.path().join("restored");
    Workspace::restore(&backup, &restored).unwrap();
    let r = Workspace::open(&restored).unwrap();
    let mut left = serde_json::to_value(read(&w, &j, 32, None).unwrap()).unwrap();
    let mut right = serde_json::to_value(read(&r, &j, 32, None).unwrap()).unwrap();
    let cursor = left["next_cursor"].as_str().unwrap().to_string();
    assert!(read(&r, &j, 32, Some(cursor)).is_err());
    left.as_object_mut().unwrap().remove("next_cursor");
    right.as_object_mut().unwrap().remove("next_cursor");
    assert_eq!(left, right);

    assert!(
        r.extraction_ocr_regions(ReadOcrRegions {
            id: o.id,
            limit: None,
            cursor: None
        })
        .is_ok()
    );
    fs::write(
        w.path.join("attachments").join(&j.attachment_id),
        b"changed",
    )
    .unwrap();
    assert!(read(&w, &j, 32, None).is_err());
}
#[test]
fn migration14_rollback_before_and_after_version_is_atomic() {
    for timing in ["BEFORE", "AFTER"] {
        let (_t, mut w) = setup();
        w.conn.execute_batch("DROP TABLE extraction_pdf_ocr_chunks;DROP TABLE extraction_pdf_ocr_generations;DROP TABLE extraction_pdfkit_chunks;DROP TABLE extraction_pdfkit_generations;DROP TABLE IF EXISTS semantic_chunks;DROP TABLE IF EXISTS semantic_control;DROP TABLE IF EXISTS semantic_generations;DROP TABLE IF EXISTS semantic_profiles;DROP TABLE IF EXISTS agent_runs;
DROP TABLE IF EXISTS extraction_pdf_structures;
UPDATE workspace_meta SET schema_version=13;PRAGMA user_version=13;").unwrap();
        w.conn.execute_batch(&format!("CREATE TRIGGER fail14 {timing} UPDATE OF schema_version ON workspace_meta WHEN NEW.schema_version=14 BEGIN SELECT RAISE(ABORT,'injected');END")).unwrap();
        let error = w.migrate(false).unwrap_err();
        assert_eq!(workspace::version(&w.conn).unwrap(), 13);
        assert!(
            std::path::Path::new(error.details["backup_path"].as_str().unwrap())
                .join("manifest.json")
                .is_file()
        );
        assert!(
            !w.conn
                .prepare("SELECT 1 FROM sqlite_schema WHERE name='extraction_pdfkit_chunks'")
                .unwrap()
                .exists([])
                .unwrap()
        );
    }
}
#[test]
fn unsupported_config_is_preserved_and_fts_excludes_native_metadata() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, document(3));
    let revision = crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision;
    assert!(crate::attachment_search::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":revision}))).is_ok());
    let raw = " {\"version\":2,\"chunk_bytes\":4096,\"future\":{\"keep\":1.00}} ";
    let value: Value = serde_json::from_str(raw).unwrap();
    let hash = store::config_hash(&value).unwrap();
    for table in ["extraction_jobs", "extraction_generations"] {
        w.conn
            .execute(
                &format!(
                    "UPDATE {table} SET config=?1,config_hash=?2 WHERE {}=?3",
                    if table == "extraction_jobs" {
                        "id"
                    } else {
                        "job_id"
                    }
                ),
                params![raw, hash, j.id],
            )
            .unwrap();
    }
    assert!(
        !w.extraction_get(IdInput { id: j.id.clone() })
            .unwrap()
            .supported
    );
    assert!(read(&w, &j, 32, None).is_err());
    let actual: String = w
        .conn
        .query_row(
            "SELECT config FROM extraction_jobs WHERE id=?1",
            [j.id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(actual, raw);
}

#[cfg(target_os = "macos")]
#[test]
fn real_worker_respects_cancel_pause_stale_source_and_deadline_before_commit() {
    for stage in ["cancel", "pause", "source", "before_commit", "success"] {
        let (_t, mut w) = setup();
        let old = seed(&mut w, document(2));
        let ctl = w.extraction_control(dto(json!({"action":"get"}))).unwrap();
        w.extraction_control(dto(
            json!({"action":"set","paused":false,"expected_revision":ctl.revision}),
        ))
        .unwrap();
        w.extraction_retry(ExtractionRevision {
            id: old.id.clone(),
            expected_revision: old.revision,
        })
        .unwrap();
        let mut reached = false;
        let result=w.extraction_run_impl(RunExtraction{id:Some(old.id.clone())},Some(&mut |at| {
            if at=="extracted" && ["cancel","pause","source"].contains(&stage) {
                reached=true;
                match stage {
                    "cancel" => {let j=w.extraction_get(IdInput{id:old.id.clone()}).unwrap();w.extraction_cancel(ExtractionRevision{id:j.id,expected_revision:j.revision}).unwrap();},
                    "pause" => {let c=w.extraction_control(dto(json!({"action":"get"}))).unwrap();w.extraction_control(dto(json!({"action":"set","paused":true,"expected_revision":c.revision}))).unwrap();},
                    "source" => {fs::write(w.path.join("attachments").join(&old.attachment_id),b"changed").unwrap();},
                    _=>unreachable!(),
                }
            }
            if at=="before_commit" && stage=="before_commit" {reached=true;std::thread::sleep(std::time::Duration::from_millis(1100));}
        }), if stage=="before_commit" {Some(std::time::Duration::from_millis(1000))} else {None});
        assert!(
            stage == "success" || reached,
            "{stage}: helper did not reach fence"
        );
        let current = w.extraction_get(IdInput { id: old.id.clone() }).unwrap();
        if stage == "success" {
            assert!(result.is_ok());
            assert_ne!(current.generation.unwrap().id, old.generation.unwrap().id);
        } else {
            if ["cancel", "pause"].contains(&stage) {
                if let Ok(out) = result {
                    assert!(matches!(out.outcome, ExtractionOutcome::Interrupted));
                    assert_eq!(out.reason.as_deref(), Some("ownership_lost"));
                }
            } else {
                assert!(result.is_err(), "{stage}");
            }
            assert_eq!(current.generation.unwrap().id, old.generation.unwrap().id);
        }
    }
}
