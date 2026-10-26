//! Persistencia y geometría sintéticas; aceptación nativa separada del contrato.
use crate::{
    contracts::*,
    pdf_ocr_storage as ps,
    workspace::{self, Workspace},
};
use rusqlite::params;
use serde_json::{Value, json};
fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace, ExtractionJob) {
    let t = tempfile::tempdir().unwrap();
    let mut w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    let mut d = crate::pdf_ocr_tests::document(2, 3);
    let r = &mut d.pages[1].recognition.regions;
    r[0].bbox = dto(json!({"x":"0.1","y":"0.7","width":"0.1","height":"0.1"}));
    r[1].bbox = dto(json!({"x":"0.3","y":"0.7","width":"0.1","height":"0.1"}));
    r[2].bbox = dto(json!({"x":"0.6","y":"0.4","width":"0.1","height":"0.1"}));
    let j = crate::pdf_ocr_storage_tests::seed(&mut w, d);
    (t, w, j)
}
fn input(j: &ExtractionJob) -> ReadPdfOcrTable {
    dto(
        json!({"id":j.id,"page_number":2,"x_edges":["0","0.5","1"],"y_min":"0","y_max":"1","header_row":1,"limit":2}),
    )
}
#[test]
fn table_keeps_empty_cells_multi_regions_global_ordinals_and_pure_restore() {
    let (t, w, j) = setup();
    let before = crate::extraction_tests::raw(&w.conn);
    let p = w.extraction_pdf_ocr_table(input(&j)).unwrap();
    assert_eq!((p.row_count, p.column_count, p.cell_count), (2, 2, 4));
    assert_eq!(p.manifest.page_count, 2);
    assert_eq!(p.page.raster.page_number, 2);
    assert_eq!(
        p.items[0]
            .regions
            .iter()
            .map(|r| r.ordinal)
            .collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(
        p.items[0].regions[0].text,
        "evidencia región 0 café 🚀\r\n<script>"
    );
    assert!(p.items[1].regions.is_empty() && !p.covers_table && !p.covers_generation);
    let mut i = input(&j);
    i.cursor = p.next_cursor.clone();
    let q = w.extraction_pdf_ocr_table(i).unwrap();
    assert_eq!(q.items[0].ordinal, 2);
    assert!(q.items[0].regions.is_empty());
    assert_eq!(q.items[1].regions[0].ordinal, 5);
    assert!(q.next_cursor.is_none() && !q.covers_table && !q.covers_generation);
    let mut i = input(&j);
    i.limit = Some(64);
    assert!(w.extraction_pdf_ocr_table(i).unwrap().covers_table);
    assert_eq!(before, crate::extraction_tests::raw(&w.conn));
    w.backup(&t.path().join("backup")).unwrap();
    Workspace::restore(&t.path().join("backup"), &t.path().join("restore")).unwrap();
    let restored = Workspace::open(&t.path().join("restore")).unwrap();
    assert_eq!(
        restored.extraction_pdf_ocr_table(input(&j)).unwrap().items,
        p.items
    );
    let mut i = input(&j);
    i.cursor = p.next_cursor;
    assert!(restored.extraction_pdf_ocr_table(i).is_err());
}
#[test]
fn table_cursor_binds_full_choice_generation_limit_and_operation() {
    let (_t, w, j) = setup();
    let cursor = w
        .extraction_pdf_ocr_table(input(&j))
        .unwrap()
        .next_cursor
        .unwrap();
    for (key, value) in [
        ("page_number", json!(1)),
        ("x_edges", json!(["0", "1"])),
        ("y_min", json!("0.1")),
        ("y_max", json!("0.9")),
        ("header_row", json!(null)),
        ("limit", json!(3)),
        ("id", json!(workspace::id())),
    ] {
        let mut i = serde_json::to_value(input(&j)).unwrap();
        i[key] = value;
        i["cursor"] = json!(cursor);
        assert!(w.extraction_pdf_ocr_table(dto(i)).is_err(), "{key}");
    }
    for (pointer, value) in [
        ("/base/ordinal", json!(0)),
        ("/base/ordinal", json!(4)),
        ("/base/ordinal", json!(9999)),
        ("/base/generation_id", json!(workspace::id())),
        ("/base/operation", json!("extraction.pdf_ocr_regions")),
    ] {
        let mut c: Value = serde_json::from_str(&cursor).unwrap();
        if let Some(v) = c.pointer_mut(pointer) {
            *v = value;
        } else {
            continue;
        }
        let mut i = input(&j);
        i.cursor = Some(c.to_string());
        assert!(w.extraction_pdf_ocr_table(i).is_err(), "{pointer}");
    }
    let old = j.generation.as_ref().unwrap();
    let p = ps::load(&w.conn, &old.id, &j.id, old.fragment_count).unwrap();
    let id = workspace::id();
    let tx = w.conn.unchecked_transaction().unwrap();
    tx.execute(
        "UPDATE extraction_jobs SET generation_id=NULL WHERE id=?1",
        [&j.id],
    )
    .unwrap();
    tx.execute("DELETE FROM extraction_generations WHERE id=?1", [&old.id])
        .unwrap();
    tx.execute("INSERT INTO extraction_generations SELECT ?1,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?2,created_at,1 FROM extraction_jobs WHERE id=?3",params![id,p.chunks.len() as u32,j.id]).unwrap();
    ps::publish(&tx, &id, &j.id, &p).unwrap();
    tx.execute(
        "UPDATE extraction_jobs SET generation_id=?1 WHERE id=?2",
        params![id, j.id],
    )
    .unwrap();
    tx.commit().unwrap();
    let mut i = input(&j);
    i.cursor = Some(cursor);
    assert!(w.extraction_pdf_ocr_table(i).is_err());
    assert!(w.extraction_pdf_ocr_table(input(&j)).is_ok());
}
#[test]
fn table_validates_whole_canonical_and_original_outside_requested_page() {
    for sql in [
        "UPDATE extraction_pdf_ocr_chunks SET text='x'||substr(text,2) WHERE ordinal=0",
        "UPDATE extraction_pdf_ocr_generations SET region_count=region_count+1",
        "UPDATE extraction_pdf_ocr_generations SET representation_sha256=replace(representation_sha256,'a','b')",
        "UPDATE extraction_generations SET config='{}'",
    ] {
        let (_t, w, j) = setup();
        w.conn.execute_batch(sql).unwrap();
        assert!(w.extraction_pdf_ocr_table(input(&j)).is_err(), "{sql}");
    }
    let (_t, w, j) = setup();
    std::fs::write(
        w.path.join("attachments").join(&j.attachment_id),
        b"changed",
    )
    .unwrap();
    assert!(w.extraction_pdf_ocr_table(input(&j)).is_err());
}
#[test]
fn table_empty_outside_crossing_and_previous_generation_are_explicit() {
    let (_t, w, j) = setup();
    let mut i = input(&j);
    i.y_min = "0.9".into();
    i.header_row = None;
    let p = w.extraction_pdf_ocr_table(i).unwrap();
    assert!(p.items.is_empty() && p.covers_table && !p.covers_generation);
    assert_eq!(
        p.outside.iter().map(|r| r.ordinal).collect::<Vec<_>>(),
        [3, 4, 5]
    );
    let mut i = input(&j);
    i.x_edges = vec!["0".into(), "0.15".into(), "1".into()];
    assert_eq!(
        w.extraction_pdf_ocr_table(i).unwrap_err().details["reason"],
        "pdf_table_crossing_region"
    );
    let mut i = input(&j);
    i.page_number = 3;
    assert_eq!(
        w.extraction_pdf_ocr_table(i).unwrap_err().details["reason"],
        "pdf_table_page_not_found"
    );
    w.conn
        .execute(
            "UPDATE extraction_jobs SET state='failed',attempts=2 WHERE id=?1",
            [&j.id],
        )
        .unwrap();
    assert!(
        w.extraction_pdf_ocr_table(input(&j))
            .unwrap()
            .result_from_previous_attempt
    );
}
#[test]
fn table_is_strict_scoped_read_only_and_non_batchable() {
    let (_t, w, j) = setup();
    let raw = serde_json::to_value(input(&j)).unwrap();
    for (key, value) in [
        ("helper", json!("/bin/sh")),
        ("source_sha256", json!("0".repeat(64))),
        ("recognition", json!({})),
    ] {
        let mut v = raw.clone();
        v[key] = value;
        assert!(serde_json::from_value::<ReadPdfOcrTable>(v).is_err());
    }
    for (key, value) in [
        ("limit", json!(0)),
        ("limit", json!(65)),
        ("page_number", json!(0)),
        ("page_number", json!(21)),
        ("x_edges", json!(["0"])),
        ("header_row", json!(0)),
        ("header_row", json!(65)),
        ("cursor", json!(" ".repeat(2049))),
    ] {
        let mut v = raw.clone();
        v[key] = value;
        assert!(dto::<ReadPdfOcrTable>(v).validate().is_err());
    }
    let scope =
        crate::scoped::LocalScope::new(&w.path, crate::scoped::Policy::new(false, false).unwrap())
            .unwrap();
    let op = json!({"operation":"extraction.pdf_ocr_table","input":raw});
    assert_eq!(
        crate::scoped::operation_access(&dto(op.clone())),
        crate::scoped::Access::Read
    );
    assert!(scope.execute_json(op, "test").ok);
    assert!(serde_json::from_value::<BatchPlan>(json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"read","operation":"extraction.pdf_ocr_table","input":raw}]})).is_err());
}

#[test]
fn table_retains_all_512_cells_and_reports_ambiguous_rows_without_partial_result() {
    let t = tempfile::tempdir().unwrap();
    let mut w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    let mut d = crate::pdf_ocr_tests::document(1, 512);
    for (i, r) in d.pages[0].recognition.regions.iter_mut().enumerate() {
        r.bbox = dto(
            json!({"x":((i%8) as f64/8.0+0.0125).to_string(),"y":(0.99-(i/8) as f64*0.014).to_string(),"width":"0.0625","height":"0.005"}),
        );
    }
    let j = crate::pdf_ocr_storage_tests::seed(&mut w, d);
    let mut i: ReadPdfOcrTable = dto(
        json!({"id":j.id,"page_number":1,"x_edges":["0","0.125","0.25","0.375","0.5","0.625","0.75","0.875","1"],"y_min":"0","y_max":"1","header_row":64,"limit":64}),
    );
    let mut seen = Vec::new();
    loop {
        let p = w.extraction_pdf_ocr_table(i.clone()).unwrap();
        assert_eq!((p.row_count, p.column_count, p.cell_count), (64, 8, 512));
        assert!(!p.covers_generation);
        seen.extend(
            p.items
                .into_iter()
                .flat_map(|c| c.regions)
                .map(|r| r.ordinal),
        );
        i.cursor = p.next_cursor;
        if i.cursor.is_none() {
            break;
        }
    }
    assert_eq!(seen, (0..512).collect::<Vec<_>>());
    let mut ambiguous = crate::pdf_ocr_tests::document(1, 3);
    for (r, y) in ambiguous.pages[0]
        .recognition
        .regions
        .iter_mut()
        .zip(["0.2", "0.4", "0.6"])
    {
        r.bbox = dto(json!({"x":"0.1","y":y,"width":"0.1","height":"0.3"}));
    }
    let j = crate::pdf_ocr_storage_tests::seed(&mut w, ambiguous);
    let i = dto(json!({"id":j.id,"page_number":1,"x_edges":["0","1"],"y_min":"0","y_max":"1"}));
    assert_eq!(
        w.extraction_pdf_ocr_table(i).unwrap_err().details["reason"],
        "pdf_table_ambiguous_rows"
    );
}
