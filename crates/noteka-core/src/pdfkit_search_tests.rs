//! Índice derivado con páginas sintéticas identificadas; corpus CLI verifica parsing real.
use crate::{
    attachment_search as api,
    batch_storage::Budget,
    contracts::*,
    extraction_storage as store, pdfkit_index_projection as projection, pdfkit_tests as fixture,
    workspace::{self, Workspace},
};
use serde_json::{Value, json};
fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
fn revision(w: &Workspace) -> u32 {
    crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision
}
fn put(w: &Workspace, j: &ExtractionJob) -> crate::error::Result<AttachmentIndexResult> {
    api::index(
        &w.path,
        dto(
            json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":revision(w)}),
        ),
    )
}
fn search(w: &Workspace, q: &str) -> crate::error::Result<AttachmentSearchPage> {
    api::search(
        &w.path,
        dto(json!({"query":q,"extractor":"pdfkit_text","limit":64})),
    )
}

#[test]
fn projection_reconstructs_unicode_pages_with_gaps_without_indexing_metadata() {
    let (_t, w) = setup();
    let mut d = fixture::document(4);
    d.pages[0].text = "é🚀漢\r\n<script>dato</script> ".repeat(400);
    d.text_bytes = d.pages.iter().map(|p| p.text.len() as u32).sum();
    let b = Budget::new(&w.conn).unwrap();
    let parts = projection::project(&d, &b).unwrap();
    assert!(parts.len() > 3);
    for page in &d.pages {
        let mut text = String::new();
        for f in parts.iter().filter(|p|matches!(p.locator,AttachmentTextLocator::PdfkitPageText{page_number,..} if page_number==page.number)) {
            assert_eq!(f.ordinal as usize,parts.iter().position(|p|p.ordinal==f.ordinal).unwrap());assert!(f.text.len()<=4096);
            let AttachmentTextLocator::PdfkitPageText{projection,profile,page_bytes,text_byte_start,text_byte_end,engine,limitations,..}=&f.locator else {panic!()};
            assert_eq!(*projection,PdfkitTextProjection::PdfkitPageTextV1);assert_eq!(profile,noteka_pdfkit_protocol::PROFILE);assert_eq!(*page_bytes,page.text.len() as u32);
            assert_eq!(*text_byte_start as usize,text.len());assert_eq!(f.text,page.text[*text_byte_start as usize..*text_byte_end as usize]);assert_eq!(engine,&d.engine);assert_eq!(limitations,&d.limitations);text.push_str(&f.text);
        }
        assert_eq!(text, page.text);
    }
}

#[test]
fn maximal_admitted_fragmentation_and_invalid_documents_keep_existing_caps() {
    let (_t, w) = setup();
    let b = Budget::new(&w.conn).unwrap();
    let mut admitted = None;
    for n in (250..=256).rev() {
        let mut d = fixture::document(100);
        for p in &mut d.pages {
            p.text = "z".into();
            p.status = noteka_pdfkit_protocol::PageStatus::Text;
        }
        d.pages[0].text =
            "a".repeat(4093) + &("🚀".to_owned() + &"a".repeat(4089)).repeat(n - 1) + "🚀";
        d.text_bytes = d.pages.iter().map(|p| p.text.len() as u32).sum();
        if crate::pdfkit_storage::prepare(d.clone()).is_ok() {
            admitted = Some(d);
            break;
        }
    }
    let mut d = admitted.unwrap();
    let parts = projection::project(&d, &b).unwrap();
    assert!((350..=356).contains(&parts.len()));
    assert!(parts.len() < 512);
    assert_eq!(MAX_EXTRACTION_FRAGMENTS, 512);
    d.pages.push(d.pages.last().unwrap().clone());
    d.page_count = 101;
    assert!(projection::project(&d, &b).is_err());
    let mut d = fixture::document(1);
    d.pages[0].text = "x".repeat(1_048_577);
    d.text_bytes = 1_048_577;
    assert!(projection::project(&d, &b).is_err());
}

#[test]
fn explicit_selection_empty_pages_noop_scope_and_metadata_exclusion() {
    let (_t, mut w) = setup();
    let j = fixture::seed(&mut w, fixture::document(70));
    assert!(search(&w, "evidencia").unwrap().items.is_empty());
    assert_eq!(
        search(&w, "evidencia")
            .unwrap()
            .coverage
            .unindexed_supported_generations,
        1
    );
    put(&w, &j).unwrap();
    let rev = revision(&w);
    assert!(
        !serde_json::to_value(put(&w, &j).unwrap()).unwrap()["changed"]
            .as_bool()
            .unwrap()
    );
    assert_eq!(revision(&w), rev);
    let p = search(&w, "evidencia").unwrap();
    assert_eq!(p.items.len(), 35);
    assert!(search(&w, "PDFKit").unwrap().items.is_empty());
    assert!(search(&w, "framework").unwrap().items.is_empty());
    for h in &p.items {
        assert!(
            matches!(h.locator,AttachmentTextLocator::PdfkitPageText{page_number,..} if page_number%2==1)
        );
    }
    let mut empty = fixture::document(1);
    empty.pages[0].status = noteka_pdfkit_protocol::PageStatus::NoText;
    empty.pages[0].text.clear();
    empty.text_bytes = 0;
    let e = fixture::seed(&mut w, empty);
    put(&w, &e).unwrap();
    assert_eq!(
        search(&w, "evidencia")
            .unwrap()
            .coverage
            .indexed_generations,
        2
    );
    w.extraction_retry(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: j.revision,
    })
    .unwrap();
    assert!(
        search(&w, "evidencia")
            .unwrap()
            .items
            .iter()
            .all(|h| h.result_from_previous_attempt)
    );
}

#[test]
fn corruption_outside_filter_or_page_and_source_tampering_fail_closed() {
    for sql in [
        "UPDATE extraction_pdfkit_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_pdfkit_chunks)",
        "UPDATE extraction_pdfkit_generations SET text_bytes=text_bytes+1",
        "UPDATE extraction_search_fts SET text='forged' WHERE ordinal=(SELECT max(ordinal) FROM extraction_search_fts)",
    ] {
        let (_t, mut w) = setup();
        let j = fixture::seed(&mut w, fixture::document(100));
        put(&w, &j).unwrap();
        w.conn.execute_batch(sql).unwrap();
        let err = api::search(
            &w.path,
            dto(json!({"query":"missing","extractor":"utf8_text","limit":1})),
        )
        .unwrap_err();
        assert!(matches!(err.code, crate::error::ErrorCode::StorageError));
        assert!(api::index(&w.path, dto(json!({"action":"get"}))).is_err());
    }
    let (_t, mut w) = setup();
    let j = fixture::seed(&mut w, fixture::document(3));
    put(&w, &j).unwrap();
    std::fs::write(w.path.join("attachments").join(j.attachment_id), b"changed").unwrap();
    assert!(search(&w, "evidencia").is_err());
}

#[test]
fn selected_pdfkit_backup_restore_and_batch_apply_preserve_derivatives() {
    let (t, mut w) = setup();
    let j = fixture::seed(&mut w, fixture::document(70));
    put(&w, &j).unwrap();
    let before = serde_json::to_value(search(&w, "evidencia").unwrap()).unwrap();
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let path = t.path().join("restored");
    Workspace::restore(&backup, &path).unwrap();
    let r = Workspace::open(&path).unwrap();
    assert_eq!(
        serde_json::to_value(search(&r, "evidencia").unwrap()).unwrap(),
        before
    );
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"preserve native index"}}]}),
    );
    let preview = crate::batches::preview(&w.path, plan.clone(), "test", "pdfkit-index").unwrap();
    crate::batches::apply(
        &w.path,
        ApplyBatch {
            plan,
            expected: preview.receipt,
        },
        "test",
        "pdfkit-index",
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(search(&w, "evidencia").unwrap()).unwrap(),
        before
    );
}

#[cfg(target_os = "macos")]
#[test]
fn real_replacement_invalidates_only_its_selection_and_old_cursor() {
    let (_t, mut w) = setup();
    let j = fixture::seed(&mut w, fixture::document(70));
    put(&w, &j).unwrap();
    let first = api::search(&w.path, dto(json!({"query":"evidencia","limit":1}))).unwrap();
    assert!(first.next_cursor.is_some());
    let ctl = w.extraction_control(dto(json!({"action":"get"}))).unwrap();
    w.extraction_control(dto(
        json!({"action":"set","paused":false,"expected_revision":ctl.revision}),
    ))
    .unwrap();
    w.extraction_retry(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: j.revision,
    })
    .unwrap();
    let done = w
        .extraction_run(RunExtraction { id: Some(j.id) })
        .unwrap()
        .job
        .unwrap();
    assert_eq!(done.state, ExtractionState::Succeeded);
    assert!(search(&w, "evidencia").unwrap().items.is_empty());
    assert_eq!(
        search(&w, "evidencia")
            .unwrap()
            .coverage
            .unindexed_supported_generations,
        1
    );
    let err = api::search(
        &w.path,
        dto(json!({"query":"evidencia","limit":1,"cursor":first.next_cursor})),
    )
    .unwrap_err();
    assert_eq!(err.details["reason"], "index_revision_changed");
    store::integrity(&w.conn).unwrap();
}

#[test]
#[ignore = "Medición adicional PDFKit/UTF-8 real en 8 MiB; conserva los cuatro gates previos"]
fn release_pdfkit_search_resources() {
    use std::time::Instant;
    let path =
        std::path::PathBuf::from(std::env::var("NOTEKA_PDFKIT_SEARCH_BENCH_WORKSPACE").unwrap());
    let w = Workspace::open(&path).unwrap();
    let index = api::index(&path, dto(json!({"action":"get","limit":64}))).unwrap();
    assert_eq!(
        serde_json::to_value(index).unwrap()["coverage"]["text_bytes"],
        8_388_608
    );
    let job: String = w.conn.query_row("SELECT j.id FROM extraction_jobs j JOIN extraction_search_entries e ON e.job_id=j.id WHERE j.extractor='pdfkit_text' ORDER BY e.slot DESC LIMIT 1", [], |r| r.get(0)).unwrap();
    let j = w.extraction_get(IdInput { id: job }).unwrap();
    let mut rows = Vec::new();
    for (name, input) in [
        ("query-mixed", json!({"query":"evidencia","limit":64})),
        (
            "query-native",
            json!({"query":"notekapdfkit","extractor":"pdfkit_text","limit":64}),
        ),
        ("query-zero", json!({"query":"ausente","limit":64})),
        (
            "query-page",
            json!({"query":"notekapdfkit","extractor":"pdfkit_text","limit":1}),
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
                    dto(
                        json!({"query":"notekapdfkit","extractor":"pdfkit_text","limit":1,"cursor":cursor}),
                    ),
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
                    json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":revision(&w)}),
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
        std::env::var("NOTEKA_PDFKIT_SEARCH_BENCH_REPORT").unwrap(),
        serde_json::to_vec_pretty(
            &json!({"text_bytes":8_388_608,"warmups":5,"samples":20,"results":rows}),
        )
        .unwrap(),
    )
    .unwrap();
}
