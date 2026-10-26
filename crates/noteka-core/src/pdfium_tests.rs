use crate::{
    attachment_search,
    attachment_search_contracts::{AttachmentIndexInput, AttachmentSearchInput, IndexedExtractor},
    attachment_search_storage,
    contracts::*,
    extraction_storage as store,
    extraction_tests::raw,
    pdf_tests::source,
    workspace::Workspace,
};
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
fn add(w: &mut Workspace, bytes: &[u8]) -> ExtractionJob {
    let path = w.path.parent().unwrap().join("fixture.pdf");
    fs::write(&path, bytes).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: path.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"pdfium_text"})))
        .unwrap()
        .job;
    enable(w);
    w.extraction_run(RunExtraction { id: Some(j.id) })
        .unwrap()
        .job
        .unwrap()
}

#[test]
fn pdfium_text_is_registered_and_preserves_other_extractors() {
    assert!(store::supported(
        "pdfium_text",
        1,
        &json!({"version":1,"chunk_bytes":4096})
    ));
    assert!(store::supported(
        "pdf_text",
        1,
        &json!({"version":1,"chunk_bytes":4096})
    ));
    assert!(!store::supported(
        "pdfium_text",
        2,
        &json!({"version":1,"chunk_bytes":4096})
    ));
}

#[test]
#[ignore = "requiere NOTEKA_PDFIUM_LIB y el binario noteka-pdfium-worker construido"]
fn real_pdfium_extraction_reads_pages_and_parser() {
    if std::env::var_os("NOTEKA_PDFIUM_LIB").is_none() {
        return;
    }
    let (_t, mut w) = setup();
    let job = add(&mut w, &source(&[b"hola pdfium".to_vec(), Vec::new()]));
    assert!(matches!(job.state, ExtractionState::Succeeded), "{job:?}");
    let page = w
        .extraction_pdf_fragments(ReadPdfFragments {
            id: job.id.clone(),
            limit: None,
            cursor: None,
        })
        .unwrap();
    assert_eq!(page.manifest.parser_name, "pdfium");
    assert!(page.manifest.parser_version.contains("7881"));
    assert_eq!(page.manifest.page_count, 2);
    assert_eq!(page.items[0].text, "hola pdfium");
    let revision = attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision;
    attachment_search::index(
        &w.path,
        AttachmentIndexInput::Put {
            job_id: job.id.clone(),
            expected_generation_id: job.generation.as_ref().unwrap().id.clone(),
            expected_revision: revision,
        },
    )
    .unwrap();
    let hits = attachment_search::search(
        &w.path,
        AttachmentSearchInput {
            query: "pdfium".into(),
            job_id: Some(job.id.clone()),
            attachment_id: None,
            extractor: Some(IndexedExtractor::PdfiumText),
            limit: Some(5),
            cursor: None,
        },
    )
    .unwrap();
    assert_eq!(hits.items[0].source.extractor, "pdfium_text");
    let _ = raw(&w.conn);
}
