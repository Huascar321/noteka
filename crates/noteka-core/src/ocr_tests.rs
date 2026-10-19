//! Fixtures de persistencia sintéticas; no se presentan como salida de inferencia.
use crate::{
    attachment_search as api,
    batch_storage::Budget,
    contracts::*,
    error::*,
    extraction_storage as store, ocr_storage as os,
    workspace::{self, Workspace},
};
use noteka_ocr_protocol::{COORDINATE_SPACE, LIMITATIONS, v2::OcrRecognition};
use rusqlite::params;
use serde_json::{Value, json};
use std::fs;
fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
pub(crate) fn recognition(n: u32) -> OcrRecognition {
    dto(
        json!({"version":1,"image":{"format":"png","width":1200,"height":300,"orientation":1,"oriented_width":1200,"oriented_height":300},"engine":{"revision":3,"recognition_level":"fast","languages":["es-ES","en-US"],"language_correction":false,"automatic_language_detection":false,"os_version":"13.0.0","os_build":"22A380"},"coordinate_space":COORDINATE_SPACE,"limitations":LIMITATIONS,"regions":(0..n).map(|i|json!({"ordinal":i,"text":format!("evidencia región {i} café 🚀\r\n<script>"),"confidence":"0.5","bbox":{"x":"0.1","y":"0.2","width":"0.8","height":"0.3"}})).collect::<Vec<_>>()}),
    )
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
pub(crate) fn seed(w: &mut Workspace, r: OcrRecognition) -> ExtractionJob {
    let source = w
        .path
        .parent()
        .unwrap()
        .join(format!("synthetic-{}.png", workspace::id()));
    fs::write(&source, b"synthetic source; no inference").unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"image_ocr"})))
        .unwrap()
        .job;
    let p = os::prepare(r).unwrap();
    let id = workspace::id();
    let tx = w.conn.unchecked_transaction().unwrap();
    tx.execute("INSERT INTO extraction_generations SELECT ?1,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?2,created_at,1 FROM extraction_jobs WHERE id=?3",params![id,p.chunks.len() as u32,j.id]).unwrap();
    tx.execute(
        "UPDATE extraction_jobs SET state='succeeded',generation_id=?1,attempts=1 WHERE id=?2",
        params![id, j.id],
    )
    .unwrap();
    os::publish(&tx, &id, &j.id, &p).unwrap();
    tx.commit().unwrap();
    w.extraction_get(IdInput { id: j.id }).unwrap()
}
fn read(
    w: &Workspace,
    j: &ExtractionJob,
    limit: u32,
    cursor: Option<String>,
) -> Result<OcrRegionPage> {
    w.extraction_ocr_regions(ReadOcrRegions {
        id: j.id.clone(),
        limit: Some(limit),
        cursor,
    })
}
fn rev(w: &Workspace) -> u32 {
    crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision
}
fn put(w: &Workspace, j: &ExtractionJob) -> Result<AttachmentIndexResult> {
    api::index(
        &w.path,
        dto(
            json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":rev(w)}),
        ),
    )
}
fn search(w: &Workspace, q: &str) -> Result<AttachmentSearchPage> {
    api::search(
        &w.path,
        dto(json!({"query":q,"extractor":"image_ocr","limit":64})),
    )
}
#[test]
fn region_reader_full_integrity_pagination_scope_and_previous_attempt() {
    let (_t, mut w) = setup();
    let r = recognition(70);
    let j = seed(&mut w, r.clone());
    let launches = crate::ocr_supervisor::spawn_count();
    let p = read(&w, &j, 32, None).unwrap();
    assert_eq!(p.manifest.region_count, 70);
    assert_eq!(p.items, r.regions[..32]);
    assert!(!p.covers_generation);
    let c = p.next_cursor.unwrap();
    assert!(read(&w, &j, 20, Some(c.clone())).is_err());
    let p = read(&w, &j, 32, Some(c)).unwrap();
    assert_eq!(p.items, r.regions[32..64]);
    let p = read(&w, &j, 32, p.next_cursor).unwrap();
    assert_eq!(p.items, r.regions[64..]);
    assert!(p.next_cursor.is_none());
    assert!(!p.covers_generation);
    for limit in [0, 65] {
        assert!(read(&w, &j, limit, None).is_err());
    }
    let old = j.generation.as_ref().unwrap().id.clone();
    w.extraction_retry(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: j.revision,
    })
    .unwrap();
    assert!(read(&w, &j, 64, None).unwrap().result_from_previous_attempt);
    assert_eq!(read(&w, &j, 64, None).unwrap().generation.id, old);
    assert_eq!(crate::ocr_supervisor::spawn_count(), launches);
}
#[test]
fn empty_result_still_has_manifest_and_explicit_index_selection() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, recognition(0));
    let p = read(&w, &j, 20, None).unwrap();
    assert!(p.items.is_empty());
    assert!(p.covers_generation);
    assert_eq!(p.generation.fragment_count, 1);
    assert_eq!(
        search(&w, "evidencia")
            .unwrap()
            .coverage
            .unindexed_supported_generations,
        1
    );
    put(&w, &j).unwrap();
    let p = search(&w, "evidencia").unwrap();
    assert!(p.items.is_empty());
    assert_eq!(p.coverage.indexed_generations, 1);
    assert_eq!(p.coverage.fragments, 0);
}
#[test]
fn explicit_selection_noop_metadata_not_searchable_and_source_is_checked() {
    let (t, mut w) = setup();
    let j = seed(&mut w, recognition(2));
    assert!(search(&w, "evidencia").unwrap().items.is_empty());
    put(&w, &j).unwrap();
    let revision = rev(&w);
    assert_eq!(
        serde_json::to_value(put(&w, &j).unwrap()).unwrap()["changed"],
        false
    );
    assert_eq!(rev(&w), revision);
    let p = search(&w, "evidencia").unwrap();
    assert_eq!(p.items.len(), 2);
    assert_eq!(p.items[0].text, recognition(2).regions[0].text);
    assert!(
        matches!(&p.items[0].locator,AttachmentTextLocator::ImageOcrRegionText {region_ordinal:0,confidence,..} if confidence=="0.5")
    );
    assert!(search(&w, "bottom").unwrap().items.is_empty());
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let launches = crate::ocr_supervisor::spawn_count();
    Workspace::restore(&backup, &t.path().join("restored")).unwrap();
    let restored = Workspace::open(&t.path().join("restored")).unwrap();
    assert_eq!(search(&restored, "evidencia").unwrap().items.len(), 2);
    assert_eq!(crate::ocr_supervisor::spawn_count(), launches);
    for p in fs::read_dir(w.path.join("attachments")).unwrap() {
        fs::write(p.unwrap().path(), b"changed").unwrap();
    }
    assert!(read(&w, &j, 20, None).is_err());
    assert!(search(&w, "evidencia").is_err());
}
#[test]
fn corruption_last_chunk_metadata_or_postings_rejects_whole_result() {
    for sql in [
        "UPDATE extraction_ocr_chunks SET text=substr(text,1,length(text)-1)||'x' WHERE ordinal=(SELECT max(ordinal) FROM extraction_ocr_chunks)",
        "UPDATE extraction_ocr_generations SET region_count=region_count+1",
        "UPDATE extraction_ocr_chunks SET job_id='foreign'",
        "UPDATE extraction_search_fts SET text='forged'",
    ] {
        let (_t, mut w) = setup();
        let j = seed(&mut w, recognition(70));
        put(&w, &j).unwrap();
        w.conn.pragma_update(None, "foreign_keys", false).unwrap();
        w.conn.execute_batch(sql).unwrap();
        assert!(search(&w, "evidencia").is_err(), "{sql}");
        if !sql.contains("search_fts") {
            assert!(read(&w, &j, 1, None).is_err());
            assert!(Workspace::open(&w.path).is_err());
        }
    }
}
#[test]
fn projection_reconstructs_unicode_with_exact_ranges_and_region_boundaries() {
    let mut r = recognition(2);
    r.regions[0].text = format!("{}🚀 Ñ\n{}", "a".repeat(4095), "é".repeat(4096));
    r.regions[1].text = "final".into();
    let c = rusqlite::Connection::open_in_memory().unwrap();
    let b = Budget::new(&c).unwrap();
    let parts = crate::ocr_index_projection::project(&r, &b).unwrap();
    assert_eq!(parts[0].text.len(), 4095);
    assert_eq!(
        parts
            .iter()
            .filter(|p| matches!(
                p.locator,
                AttachmentTextLocator::ImageOcrRegionText {
                    region_ordinal: 0,
                    ..
                }
            ))
            .map(|p| p.text.as_str())
            .collect::<String>(),
        r.regions[0].text
    );
    for (ordinal, f) in parts.iter().enumerate() {
        assert_eq!(f.ordinal, ordinal as u32);
        if let AttachmentTextLocator::ImageOcrRegionText {
            region_ordinal,
            region_bytes,
            text_byte_start,
            text_byte_end,
            ..
        } = &f.locator
        {
            let text = &r.regions[*region_ordinal as usize].text;
            assert_eq!(*region_bytes, text.len() as u32);
            assert_eq!(
                f.text,
                text[*text_byte_start as usize..*text_byte_end as usize]
            );
        }
    }
    drop(b);
    let b = Budget::with_duration(&c, std::time::Duration::ZERO).unwrap();
    assert!(crate::ocr_index_projection::project(&r, &b).is_err());
}
#[test]
fn canonical_representation_rejects_unknown_noncanonical_and_size_excess() {
    let mut r = recognition(1);
    r.regions[0].confidence = "5e-1".into();
    assert!(os::prepare(r).is_err());
    let p = os::prepare(recognition(70)).unwrap();
    assert!(p.chunks.iter().all(|s| s.len() <= 4096));
    assert!(p.chunks.len() > 1);
    let mut r = recognition(9);
    for region in &mut r.regions {
        region.text = "a".repeat(131072);
    }
    assert!(os::prepare(r).is_err());
}
#[test]
fn migration013_failure_rolls_back_all_rows_and_backup_can_restore_v12() {
    for before in [true, false] {
        let (t, mut w) = setup();
        w.conn.execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;UPDATE workspace_meta SET schema_version=12;PRAGMA user_version=12").unwrap();
        if before {
            w.conn.execute_batch("CREATE TRIGGER fail13 BEFORE UPDATE OF schema_version ON workspace_meta WHEN NEW.schema_version=13 BEGIN SELECT RAISE(ABORT,'injected');END").unwrap();
        }
        let e = w.migrate(!before).unwrap_err();
        assert_eq!(workspace::version(&w.conn).unwrap(), 12);
        assert!(
            !w.conn
                .prepare("SELECT 1 FROM sqlite_schema WHERE name='extraction_ocr_generations'")
                .unwrap()
                .exists([])
                .unwrap()
        );
        let backup = std::path::Path::new(e.details["backup_path"].as_str().unwrap());
        assert!(backup.join("manifest.json").exists());
        assert!(!t.path().join("restored").exists());
    }
}
#[test]
fn future_config_is_preserved_and_does_not_execute() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, recognition(1));
    let raw = " {\"version\":2,\"chunk_bytes\":4096,\"future\":{\"keep\":1.00}} ";
    let v: Value = serde_json::from_str(raw).unwrap();
    let hash = store::config_hash(&v).unwrap();
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?1,config_hash=?2 WHERE id=?3",
            params![raw, hash, j.id],
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE extraction_generations SET config=?1,config_hash=?2 WHERE job_id=?3",
            params![raw, hash, j.id],
        )
        .unwrap();
    let launch = crate::ocr_supervisor::spawn_count();
    let job = w.extraction_get(IdInput { id: j.id.clone() }).unwrap();
    assert!(!job.supported);
    assert!(read(&w, &j, 20, None).is_err());
    assert!(put(&w, &j).is_err());
    assert_eq!(
        w.conn
            .query_row(
                "SELECT config FROM extraction_jobs WHERE id=?1",
                [j.id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        raw
    );
    assert_eq!(crate::ocr_supervisor::spawn_count(), launch);
}

#[test]
fn selection_is_fenced_and_ocr_derived_state_survives_batch_apply() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, recognition(2));
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"OCR batch"}}]}),
    );
    let base = crate::batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap();
    let preview = crate::batches::preview(&w.path, plan.clone(), "test", "preview").unwrap();
    let input = dto(
        json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":rev(&w)}),
    );
    let e = api::index_with_hook(&w.path, input, &mut |stage| {
        if stage == "prepared" {
            api::index(
                &w.path,
                AttachmentIndexInput::Reset {
                    expected_revision: rev(&w),
                },
            )?;
        }
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(e.code, ErrorCode::RevisionConflict));
    put(&w, &j).unwrap();
    let before = read(&w, &j, 20, None).unwrap();
    assert_eq!(
        base,
        crate::batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap()
    );
    crate::batches::apply(
        &w.path,
        ApplyBatch {
            plan,
            expected: preview.receipt,
        },
        "test",
        "apply",
    )
    .unwrap();
    assert_eq!(read(&w, &j, 20, None).unwrap().items, before.items);
    assert_eq!(search(&w, "evidencia").unwrap().items.len(), 2);
}
#[cfg(target_os = "macos")]
#[test]
fn real_blank_image_runs_explicitly_and_ocr_publication_preserves_fences() {
    for stage in [
        "none",
        "source_read",
        "extracted",
        "before_commit",
        "metadata_changed",
    ] {
        let (t, mut w) = setup();
        let file = t.path().join("blank.png");
        fs::write(
            &file,
            include_bytes!("../../../tests/fixtures/ocr/blank.png"),
        )
        .unwrap();
        let a = w
            .attachment_import(ImportAttachment {
                source_path: file.to_string_lossy().into(),
            })
            .unwrap();
        let j = w
            .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"image_ocr"})))
            .unwrap()
            .job;
        assert!(matches!(
            w.extraction_run(RunExtraction {
                id: Some(j.id.clone())
            })
            .unwrap()
            .outcome,
            ExtractionOutcome::Idle
        ));
        let c = w
            .extraction_control(ExtractionControlInput::Get {})
            .unwrap();
        w.extraction_control(ExtractionControlInput::Set {
            paused: false,
            expected_revision: c.revision,
        })
        .unwrap();
        let j = w
            .extraction_run(RunExtraction { id: Some(j.id) })
            .unwrap()
            .job
            .unwrap();
        assert_eq!(j.state, ExtractionState::Succeeded);
        assert!(read(&w, &j, 20, None).unwrap().items.is_empty());
        let old = j.generation.clone().unwrap().id;
        let j = w
            .extraction_retry(ExtractionRevision {
                id: j.id.clone(),
                expected_revision: j.revision,
            })
            .unwrap();
        let mut reached = false;
        let result = w.extraction_run_impl(
            RunExtraction {
                id: Some(j.id.clone()),
            },
            Some(&mut |at| {
                if (at == stage || (stage=="metadata_changed" && at=="extracted")) && stage != "none" {
                    reached = true;
                    if stage=="metadata_changed" {
                        w.conn.execute("UPDATE attachments SET name='renamed.jpg',mime='image/jpeg' WHERE id=?1",[&j.attachment_id]).unwrap();
                    } else if at == "before_commit" {
                        std::thread::sleep(std::time::Duration::from_millis(2100));
                    } else {
                        let c = w
                            .extraction_control(ExtractionControlInput::Get {})
                            .unwrap();
                        w.extraction_control(ExtractionControlInput::Set {
                            paused: true,
                            expected_revision: c.revision,
                        })
                        .unwrap();
                    }
                }
            }),
            if stage == "before_commit" {
                Some(std::time::Duration::from_millis(2000))
            } else {
                None
            },
        );
        if stage == "metadata_changed" {
            assert_eq!(
                result.as_ref().unwrap_err().details["reason"],
                "ocr_source_format_mismatch"
            );
            w.conn
                .execute(
                    "UPDATE attachments SET name='blank.png',mime='image/png' WHERE id=?1",
                    [&j.attachment_id],
                )
                .unwrap();
        }
        assert!(stage == "none" || reached, "hook {stage} no se alcanzó");
        let current = w.extraction_get(IdInput { id: j.id.clone() }).unwrap();
        if stage == "none" {
            assert_eq!(
                result.unwrap().job.unwrap().state,
                ExtractionState::Succeeded
            );
            assert_ne!(current.generation.unwrap().id, old);
        } else {
            assert_eq!(current.generation.unwrap().id, old);
            assert!(read(&w, &j, 20, None).unwrap().result_from_previous_attempt);
        }
    }
}
#[test]
fn shared_retention_counts_ocr_and_discounts_only_its_own_job() {
    let (_t, mut w) = setup();
    let a = seed(&mut w, recognition(2));
    let b = seed(&mut w, recognition(3));
    let bytes = |id: &str| {
        u64::from({
            w.conn
                .query_row(
                    "SELECT representation_bytes FROM extraction_ocr_generations WHERE job_id=?1",
                    [id],
                    |r| r.get::<_, u32>(0),
                )
                .unwrap()
        })
    };
    assert_eq!(os::retained(&w.conn, &a.id).unwrap(), bytes(&b.id));
    assert_eq!(
        os::retained(&w.conn, "none").unwrap(),
        bytes(&a.id) + bytes(&b.id)
    );
    let b_bytes = bytes(&b.id);
    let text = crate::attachment_search_tests::add(&mut w, b"literal shared budget", "utf8_text");
    assert_eq!(
        os::retained(&w.conn, &a.id).unwrap(),
        b_bytes + u64::from(text.source_bytes)
    );
}

#[cfg(target_os = "macos")]
#[test]
fn actual_ocr_publication_enforces_exact_four_format_retention_without_eviction() {
    let (_t, mut w) = setup();
    let source =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ocr/blank.png");
    let image = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    let job = w
        .extraction_enqueue(dto(
            json!({"attachment_id":image.id,"extractor":"image_ocr"}),
        ))
        .unwrap()
        .job;
    let ctl = w.extraction_control(dto(json!({"action":"get"}))).unwrap();
    w.extraction_control(dto(
        json!({"action":"set","paused":false,"expected_revision":ctl.revision}),
    ))
    .unwrap();
    let done = w
        .extraction_run(RunExtraction { id: Some(job.id) })
        .unwrap()
        .job
        .unwrap();
    let ocr_bytes = read(&w, &done, 20, None)
        .unwrap()
        .manifest
        .representation_bytes;
    crate::attachment_search_tests::add(&mut w, &vec![b't'; 1_048_576], "utf8_text");

    // Derivados SQL sintéticos para fijar la frontera: no prueban parsing.
    // La ejecución OCR y el reemplazo/exceso sí usan el helper nativo real.
    let install = |w: &Workspace, j: &ExtractionJob, count: u32| {
        let id = workspace::id();
        w.conn.execute("INSERT INTO extraction_generations SELECT ?1,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?2,created_at,1 FROM extraction_jobs WHERE id=?3",params![id,count,j.id]).unwrap();
        w.conn.execute("UPDATE extraction_jobs SET state='succeeded',generation_id=?1,attempts=1 WHERE id=?2",params![id,j.id]).unwrap();
        id
    };
    let pdf_source = w.path.parent().unwrap().join("retained.pdf");
    fs::write(
        &pdf_source,
        crate::pdf_tests::source(&[b"fixture".to_vec()]),
    )
    .unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: pdf_source.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"pdf_text"})))
        .unwrap()
        .job;
    let p = crate::pdf_storage::prepare(vec![noteka_pdf_protocol::PageText {
        page_number: 1,
        text: "p".repeat(1_048_576 - ocr_bytes as usize),
    }])
    .unwrap();
    let id = install(&w, &j, p.fragments.len() as u32);
    crate::pdf_storage::publish(&w.conn, &id, &j.id, &p).unwrap();
    let p = crate::xlsx_storage::prepare(crate::xlsx_tests::sized_book(1_048_576)).unwrap();
    for n in 0..30 {
        let path = w.path.parent().unwrap().join(format!("retained-{n}.xlsx"));
        fs::write(&path, b"synthetic source; no parsing").unwrap();
        let a = w
            .attachment_import(ImportAttachment {
                source_path: path.to_string_lossy().into(),
            })
            .unwrap();
        let j = w
            .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"xlsx_cells"})))
            .unwrap()
            .job;
        let id = install(&w, &j, p.chunks.len() as u32);
        crate::xlsx_storage::publish(&w.conn, &id, &j.id, &p).unwrap();
    }
    store::integrity(&w.conn).unwrap();
    assert_eq!(os::retained(&w.conn, "").unwrap(), MAX_DERIVED_BYTES);
    w.extraction_retry(ExtractionRevision {
        id: done.id.clone(),
        expected_revision: done.revision,
    })
    .unwrap();
    let replaced = w
        .extraction_run(RunExtraction {
            id: Some(done.id.clone()),
        })
        .unwrap()
        .job
        .unwrap();
    assert_ne!(
        replaced.generation.as_ref().unwrap().id,
        done.generation.unwrap().id
    );
    assert_eq!(os::retained(&w.conn, "").unwrap(), MAX_DERIVED_BYTES);
    put(&w, &replaced).unwrap();
    let selected_revision = rev(&w);
    let a = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    let extra = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"image_ocr"})))
        .unwrap()
        .job;
    assert_eq!(
        w.extraction_run(RunExtraction { id: Some(extra.id) })
            .unwrap_err()
            .details["reason"],
        "derived_size_limit"
    );
    assert_eq!(os::retained(&w.conn, "").unwrap(), MAX_DERIVED_BYTES);
    assert_eq!(rev(&w), selected_revision);
    assert_eq!(
        read(&w, &replaced, 20, None).unwrap().generation.id,
        replaced.generation.unwrap().id
    );
    store::integrity(&w.conn).unwrap();
}

#[test]
fn populated_v12_index_is_verified_before_ocr_tables_exist() {
    // Fixture SQL de regresión del orden de migración; la aceptación histórica
    // independiente usa test-ocr-migration.py y el binario v12 durable.
    let (_t, mut w) = setup();
    let j = crate::attachment_search_tests::add(&mut w, b"migration literal evidence", "utf8_text");
    put(&w, &j).unwrap();
    let expected =
        serde_json::to_value(api::search(&w.path, dto(json!({"query":"migration"}))).unwrap())
            .unwrap();
    w.conn.execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;UPDATE workspace_meta SET schema_version=12;PRAGMA user_version=12").unwrap();
    w.backup(&w.path.parent().unwrap().join("prior-v12"))
        .unwrap();
    let opened = Workspace::open(&w.path).unwrap();
    assert_eq!(opened.info().unwrap().schema_version, 13);
    assert_eq!(
        serde_json::to_value(api::search(&opened.path, dto(json!({"query":"migration"}))).unwrap())
            .unwrap(),
        expected
    );
}

#[test]
fn valid_ocr_generation_can_exceed_projection_fragment_cap_without_partial_selection() {
    let (_t, mut w) = setup();
    let original = seed(&mut w, recognition(2));
    put(&w, &original).unwrap();
    let c = rusqlite::Connection::open_in_memory().unwrap();
    let b = Budget::new(&c).unwrap();
    let mut r = recognition(512);
    assert_eq!(
        crate::ocr_index_projection::project(&r, &b).unwrap().len(),
        512
    );
    r.regions[0].text = "a".repeat(4097);
    os::prepare(r.clone()).unwrap();
    let j = seed(&mut w, r);
    let revision = rev(&w);
    assert_eq!(
        put(&w, &j).unwrap_err().details["reason"],
        "index_budget_exceeded"
    );
    assert_eq!(rev(&w), revision);
    assert_eq!(search(&w, "evidencia").unwrap().items.len(), 2);
    assert_eq!(read(&w, &j, 64, None).unwrap().manifest.region_count, 512);
}
