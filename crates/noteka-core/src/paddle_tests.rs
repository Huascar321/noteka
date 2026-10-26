use crate::{
    attachment_search,
    attachment_search_contracts::{AttachmentIndexInput, AttachmentSearchInput, IndexedExtractor},
    attachment_search_storage,
    contracts::*,
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
#[test]
fn paddle_ocr_is_registered_without_activating_models() {
    assert!(crate::extraction_storage::supported(
        "paddle_ocr",
        1,
        &json!({"version":1,"chunk_bytes":4096})
    ));
    assert!(!crate::extraction_storage::supported(
        "paddle_ocr",
        2,
        &json!({"version":1,"chunk_bytes":4096})
    ));
}

#[test]
#[ignore = "requiere NOTEKA_PADDLE_* y el entorno PaddleOCR instalado"]
fn real_paddle_ocr_extracts_page_text() {
    if std::env::var_os("NOTEKA_PADDLE_PYTHON").is_none() {
        return;
    }
    let (t, mut w) = setup();
    let path = w.path.parent().unwrap().join("fixture.pdf");
    fs::write(&path, source(&[b"hola paddle".to_vec()])).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: path.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"paddle_ocr"})))
        .unwrap()
        .job;
    enable(&w);
    let job = w
        .extraction_run(RunExtraction { id: Some(j.id) })
        .unwrap()
        .job
        .unwrap();
    assert!(matches!(job.state, ExtractionState::Succeeded), "{job:?}");
    let page = w
        .extraction_pdf_fragments(ReadPdfFragments {
            id: job.id.clone(),
            limit: None,
            cursor: None,
        })
        .unwrap();
    assert_eq!(page.manifest.parser_name, "paddleocr_ppstructurev3");
    assert_eq!(page.manifest.parser_version, "3.7.0");
    assert!(page.items[0].text.contains("hola paddle"));
    let structure = w
        .extraction_pdf_structure(ReadPdfStructure { id: job.id.clone() })
        .unwrap();
    assert_eq!(structure.manifest.format, "paddle_structure_v1");
    assert_eq!(structure.structure.pages[0].page_number, 1);
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let restored_path = t.path().join("restored");
    Workspace::restore(&backup, &restored_path).unwrap();
    let restored = Workspace::open(&restored_path).unwrap();
    let restored_structure = restored
        .extraction_pdf_structure(ReadPdfStructure { id: job.id.clone() })
        .unwrap();
    assert_eq!(
        restored_structure.manifest.structure_sha256,
        structure.manifest.structure_sha256
    );
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
            query: "paddle".into(),
            job_id: Some(job.id.clone()),
            attachment_id: None,
            extractor: Some(IndexedExtractor::PaddleOcr),
            limit: Some(5),
            cursor: None,
        },
    )
    .unwrap();
    assert_eq!(hits.items[0].source.extractor, "paddle_ocr");
    let _ = raw(&w.conn);
}
