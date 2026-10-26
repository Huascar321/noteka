//! Persistencia sintética: no se presenta como OCR real; corpus real separado.
use crate::{
    contracts::*,
    pdf_ocr_storage as ps,
    workspace::{self, Workspace},
};
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
pub(crate) fn seed(w: &mut Workspace, mut d: crate::pdf_ocr_compositor::Document) -> ExtractionJob {
    let source = w
        .path
        .parent()
        .unwrap()
        .join(format!("{}.pdf", workspace::id()));
    fs::write(
        &source,
        crate::pdf_tests::source(&[b"Synthetic PDF OCR fence".to_vec()]),
    )
    .unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    d.source_sha256 = a.sha256.clone();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"pdf_ocr"})))
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
) -> crate::error::Result<PdfOcrRegionPage> {
    w.extraction_pdf_ocr_regions(ReadPdfOcrRegions {
        id: j.id.clone(),
        limit: Some(limit),
        cursor,
    })
}
#[test]
fn pdf_ocr_all_pages_global_regions_and_scope_survive_reopen() {
    let (_t, mut w) = setup();
    let mut d = crate::pdf_ocr_tests::document(20, 3);
    d.pages[1].recognition.regions.clear();
    let j = seed(&mut w, d);
    let before = crate::pdf_raster_supervisor::spawn_count();
    let vision = crate::ocr_supervisor::spawn_count();
    let p = read(&w, &j, 20, None).unwrap();
    assert_eq!(p.manifest.pages.len(), 20);
    assert_eq!(p.manifest.pages[1].region_count, 0);
    assert_eq!(p.manifest.region_count, 57);
    assert_eq!(p.items.len(), 20);
    assert!(!p.covers_generation);
    assert!(read(&w, &j, 21, p.next_cursor.clone()).is_err());
    let mut items = p.items;
    let mut cursor = p.next_cursor;
    while cursor.is_some() {
        let p = read(&w, &j, 20, cursor).unwrap();
        items.extend(p.items);
        cursor = p.next_cursor;
    }
    assert_eq!(items.len(), 57);
    for (i, r) in items.iter().enumerate() {
        assert_eq!(r.ordinal as usize, i);
        assert_ne!(r.page_number, 2);
        assert!(r.region_ordinal < 3);
    }
    w.extraction_retry(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: j.revision,
    })
    .unwrap();
    assert!(read(&w, &j, 64, None).unwrap().result_from_previous_attempt);
    let path = w.path.clone();
    drop(w);
    let w = Workspace::open(&path).unwrap();
    assert_eq!(read(&w, &j, 64, None).unwrap().items, items);
    assert_eq!(crate::pdf_raster_supervisor::spawn_count(), before);
    assert_eq!(crate::ocr_supervisor::spawn_count(), vision);
    let empty = seed(
        &mut Workspace::open(&path).unwrap(),
        crate::pdf_ocr_tests::document(2, 0),
    );
    let page = read(&w, &empty, 1, None).unwrap();
    assert!(page.items.is_empty() && page.covers_generation && page.next_cursor.is_none());
    assert_eq!(page.manifest.page_count, 2);
}
#[test]
fn pdf_ocr_every_chunk_metadata_and_owner_are_verified_outside_requested_slice() {
    for sql in [
        "UPDATE extraction_pdf_ocr_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_pdf_ocr_chunks)",
        "UPDATE extraction_pdf_ocr_generations SET text_bytes=text_bytes+1",
        "UPDATE extraction_pdf_ocr_generations SET region_count=region_count+1",
        "UPDATE extraction_pdf_ocr_generations SET page_count=page_count-1",
        "UPDATE extraction_pdf_ocr_generations SET profile='future'",
        "UPDATE extraction_pdf_ocr_chunks SET ordinal=ordinal+100",
        "DELETE FROM extraction_pdf_ocr_chunks WHERE ordinal=1",
        "PRAGMA foreign_keys=OFF;UPDATE extraction_pdf_ocr_chunks SET job_id='foreign';PRAGMA foreign_keys=ON",
    ] {
        let (_t, mut w) = setup();
        let j = seed(&mut w, crate::pdf_ocr_tests::document(20, 3));
        w.conn.execute_batch(sql).unwrap();
        assert!(read(&w, &j, 1, None).is_err(), "{sql}");
        assert!(ps::integrity(&w.conn).is_err(), "{sql}");
    }
}
#[test]
fn pdf_ocr_backup_restore_batches_and_all_format_retention_preserve_canonical() {
    let (t, mut w) = setup();
    let j = seed(&mut w, crate::pdf_ocr_tests::document(20, 3));
    let manifest = read(&w, &j, 20, None).unwrap().manifest;
    let o = crate::ocr_tests::seed(&mut w, crate::ocr_tests::recognition(2));
    let p = crate::pdfkit_tests::seed(&mut w, crate::pdfkit_tests::document(2));
    assert_eq!(
        ps::retained(&w.conn, "").unwrap(),
        u64::from(manifest.representation_bytes)
            + crate::pdfkit_storage::retained(&w.conn, "").unwrap()
    );
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"preservar"}}]}),
    );
    let preview = crate::batches::preview(&w.path, plan.clone(), "test", "pdf_ocr").unwrap();
    crate::batches::apply(
        &w.path,
        ApplyBatch {
            plan,
            expected: preview.receipt,
        },
        "test",
        "pdf_ocr",
    )
    .unwrap();
    assert_eq!(read(&w, &j, 20, None).unwrap().manifest, manifest);
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let restored = t.path().join("restored");
    Workspace::restore(&backup, &restored).unwrap();
    let r = Workspace::open(&restored).unwrap();
    let mut a = serde_json::to_value(read(&w, &j, 20, None).unwrap()).unwrap();
    let cursor = a["next_cursor"].as_str().unwrap().to_owned();
    assert!(read(&r, &j, 20, Some(cursor)).is_err());
    let mut b = serde_json::to_value(read(&r, &j, 20, None).unwrap()).unwrap();
    a.as_object_mut().unwrap().remove("next_cursor");
    b.as_object_mut().unwrap().remove("next_cursor");
    assert_eq!(a, b);
    assert!(
        r.extraction_ocr_regions(ReadOcrRegions {
            id: o.id,
            limit: None,
            cursor: None
        })
        .is_ok()
    );
    assert!(
        r.extraction_pdfkit_pages(ReadPdfkitPages {
            id: p.id,
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
    assert!(read(&w, &j, 1, None).is_err());
}
#[test]
fn pdf_ocr_cross_format_rows_cannot_be_certified_by_previous_readers() {
    let (_t, mut w) = setup();
    let o = crate::ocr_tests::seed(&mut w, crate::ocr_tests::recognition(1));
    let g = o.generation.as_ref().unwrap();
    w.conn
        .execute(
            "INSERT INTO extraction_pdf_ocr_generations VALUES (?1,?2,1,?3,?4,1,1,0,0,1)",
            params![
                g.id,
                o.id,
                crate::pdf_ocr_compositor::PROFILE,
                "0".repeat(64)
            ],
        )
        .unwrap();
    assert_eq!(
        w.extraction_ocr_regions(ReadOcrRegions {
            id: o.id,
            limit: None,
            cursor: None
        })
        .unwrap_err()
        .details["reason"],
        "invalid_pdf_ocr_integrity"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn pdf_ocr_real_worker_respects_cancel_pause_stale_source_and_deadline_before_commit() {
    for stage in ["cancel", "pause", "source", "before_commit", "success"] {
        let (_t, mut w) = setup();
        let old = seed(&mut w, crate::pdf_ocr_tests::document(2, 1));
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

fn select(w: &Workspace, j: &ExtractionJob) -> crate::error::Result<AttachmentIndexResult> {
    let rev = crate::attachment_search_storage::control(&w.conn)?.revision;
    crate::attachment_search::index(
        &w.path,
        dto(
            json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":rev}),
        ),
    )
}
#[test]
fn pdf_ocr_index_uses_only_region_text_and_full_typed_provenance() {
    let (_t, mut w) = setup();
    let mut d = crate::pdf_ocr_tests::document(3, 2);
    d.pages[1].recognition.regions.clear();
    d.pages[0].recognition.regions[0].text = "é€🚀 needle ".repeat(600);
    let j = seed(&mut w, d.clone());
    select(&w, &j).unwrap();
    let before = crate::pdf_raster_supervisor::spawn_count();
    let vision = crate::ocr_supervisor::spawn_count();
    let input = || dto(json!({"query":"needle","extractor":"pdf_ocr","job_id":j.id,"limit":64}));
    let hits = crate::attachment_search::search(&w.path, input()).unwrap();
    assert!(!hits.items.is_empty());
    for hit in hits.items {
        let AttachmentTextLocator::PdfOcrRegionText {
            projection,
            profile,
            page_number,
            region_ordinal,
            global_region_ordinal,
            raster,
            engine,
            bbox,
            confidence,
            coordinate_space,
            limitations,
            region_bytes,
            text_byte_start,
            text_byte_end,
        } = hit.locator
        else {
            panic!("typed locator")
        };
        assert_eq!(projection, PdfOcrTextProjection::PdfOcrRegionTextV1);
        assert_eq!(profile, crate::pdf_ocr_compositor::PROFILE);
        assert_eq!(
            (page_number, region_ordinal, global_region_ordinal),
            (1, 0, 0)
        );
        assert_eq!(coordinate_space, "rendered_page_normalized_bottom_left");
        let p = &d.pages[0];
        let r = &p.recognition.regions[0];
        assert_eq!(*raster, p.raster);
        assert_eq!(*engine, p.recognition.engine);
        assert_eq!(bbox, r.bbox);
        assert_eq!(confidence, r.confidence);
        assert_eq!(limitations, p.recognition.limitations);
        assert_eq!(region_bytes as usize, r.text.len());
        assert_eq!(
            hit.text,
            r.text[text_byte_start as usize..text_byte_end as usize]
        );
        assert!(hit.text.len() <= 4096);
    }
    for term in ["22A380", "rendered", "apple_cgpdf_vision", "confidence"] {
        assert!(
            crate::attachment_search::search(
                &w.path,
                dto(json!({"query":term,"extractor":"pdf_ocr"}))
            )
            .unwrap()
            .items
            .is_empty()
        );
    }
    assert_eq!(crate::pdf_raster_supervisor::spawn_count(), before);
    assert_eq!(crate::ocr_supervisor::spawn_count(), vision);
    // Rechaza corrupción fuera del primer resultado y aun con filtro que no coincide.
    w.conn.execute("UPDATE extraction_pdf_ocr_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_pdf_ocr_chunks)",[]).unwrap();
    assert!(
        crate::attachment_search::search(
            &w.path,
            dto(json!({"query":"unmatched","job_id":workspace::id(),"limit":1}))
        )
        .is_err()
    );
}
#[test]
fn pdf_ocr_index_512_513_region_frontier_is_atomic_and_never_truncates() {
    let (_t, mut w) = setup();
    let mut good = crate::pdf_ocr_tests::document(2, 256);
    for p in &mut good.pages {
        for r in &mut p.recognition.regions {
            r.text = "needle".into();
        }
    }
    let j = seed(&mut w, good);
    select(&w, &j).unwrap();
    let prior = crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision;
    let mut too_many = crate::pdf_ocr_tests::document(3, 171);
    for p in &mut too_many.pages {
        for r in &mut p.recognition.regions {
            r.text = "needle".into();
        }
    }
    let other = seed(&mut w, too_many);
    assert_eq!(
        select(&w, &other).unwrap_err().details["reason"],
        "index_budget_exceeded"
    );
    assert_eq!(
        crate::attachment_search_storage::control(&w.conn)
            .unwrap()
            .revision,
        prior
    );
    let page = crate::attachment_search::index(&w.path, dto(json!({"action":"get"}))).unwrap();
    let value = serde_json::to_value(page).unwrap();
    assert_eq!(value["coverage"]["indexed_generations"], 1);
    assert_eq!(value["coverage"]["fragments"], 512);
}

#[test]
#[ignore = "Release resource gate: real PDF OCR + UTF8, 8MiB, 5 warmups/20 samples"]
fn release_pdf_ocr_search_resources() {
    use crate::attachment_search as api;
    use std::time::Instant;
    let path =
        std::path::PathBuf::from(std::env::var("NOTEKA_PDF_OCR_SEARCH_BENCH_WORKSPACE").unwrap());
    let w = Workspace::open(&path).unwrap();
    let index = api::index(&path, dto(json!({"action":"get","limit":64}))).unwrap();
    assert_eq!(
        serde_json::to_value(index).unwrap()["coverage"]["text_bytes"],
        8_388_608
    );
    let job: String = w.conn.query_row("SELECT j.id FROM extraction_jobs j JOIN extraction_search_entries e ON e.job_id=j.id WHERE j.extractor='pdf_ocr' ORDER BY e.slot DESC LIMIT 1", [], |r| r.get(0)).unwrap();
    let j = w.extraction_get(IdInput { id: job }).unwrap();
    let mut rows = Vec::new();
    for (name, input) in [
        ("query-mixed", json!({"query":"evidencia","limit":64})),
        (
            "query-native",
            json!({"query":"noteka","extractor":"pdf_ocr","limit":64}),
        ),
        ("query-zero", json!({"query":"ausente","limit":64})),
        (
            "query-page",
            json!({"query":"noteka","extractor":"pdf_ocr","limit":1}),
        ),
    ] {
        for round in 0..25 {
            let start = Instant::now();
            let result = api::search(&path, dto(input.clone()));
            rows.push(json!({"case":name,"round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"ok":result.is_ok(),"error":result.as_ref().err()}));
            if name == "query-page"
                && let Ok(p) = result
                && let Some(cursor) = p.next_cursor
            {
                let start = Instant::now();
                let next = api::search(
                    &path,
                    dto(json!({"query":"noteka","extractor":"pdf_ocr","limit":1,"cursor":cursor})),
                );
                rows.push(json!({"case":"query-next","round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"ok":next.is_ok(),"error":next.err()}));
            }
        }
    }
    for name in ["put", "no-op"] {
        for round in 0..25 {
            if name == "put" {
                w.conn
                    .execute(
                        "DELETE FROM extraction_search_entries WHERE job_id=?1",
                        [&j.id],
                    )
                    .unwrap();
            }
            let mut locked = None;
            let mut lock_ms = None;
            let start = Instant::now();
            let result = api::index_with_hook(
                &path,
                dto(
                    json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":crate::attachment_search_storage::control(&w.conn).unwrap().revision}),
                ),
                &mut |stage| {
                    if stage == "locked" {
                        locked = Some(Instant::now());
                    }
                    if stage == "committed" {
                        lock_ms = locked.map(|t| t.elapsed().as_secs_f64() * 1000.);
                    }
                    Ok(())
                },
            );
            rows.push(json!({"case":name,"round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"lock_ms":lock_ms,"ok":result.is_ok(),"error":result.err()}));
        }
    }
    std::fs::write(
        std::env::var("NOTEKA_PDF_OCR_SEARCH_BENCH_REPORT").unwrap(),
        serde_json::to_vec_pretty(
            &json!({"text_bytes":8_388_608,"warmups":5,"samples":20,"results":rows}),
        )
        .unwrap(),
    )
    .unwrap();
}
