//! Fixtures de almacenamiento sintéticas; la integración CLI usa XLSX reales aparte.
use crate::{
    contracts::*,
    extraction_storage as store,
    workspace::{self, Workspace},
    xlsx_storage as xs,
};
use noteka_xlsx_protocol::{CellValue, Visibility, Workbook};
use rusqlite::params;
use serde_json::{Value, json};

fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn seed(w: &mut Workspace, book: Workbook) -> ExtractionJob {
    let source = w
        .path
        .parent()
        .unwrap()
        .join(format!("{}.xlsx", workspace::id()));
    std::fs::write(&source, crate::xlsx_tests::source(1)).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"xlsx_cells"})))
        .unwrap()
        .job;
    let p = xs::prepare(book).unwrap();
    let id = workspace::id();
    let tx = w.conn.unchecked_transaction().unwrap();
    tx.execute("INSERT INTO extraction_generations SELECT ?1,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?2,created_at,1 FROM extraction_jobs WHERE id=?3", params![id,p.chunks.len() as u32,j.id]).unwrap();
    tx.execute(
        "UPDATE extraction_jobs SET state='succeeded',generation_id=?1,attempts=1 WHERE id=?2",
        params![id, j.id],
    )
    .unwrap();
    xs::publish(&tx, &id, &j.id, &p).unwrap();
    tx.commit().unwrap();
    w.extraction_get(IdInput { id: j.id }).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace, ExtractionJob) {
    let t = tempfile::tempdir().unwrap();
    let mut w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    let mut book = crate::xlsx_tests::book(31);
    book.sheets[0].id = 7;
    book.sheets[0].visibility = Visibility::VeryHidden;
    book.sheets[0].cells[0].cached = Some(CellValue::Text("<script>dato</script>".into()));
    book.sheets[0].cells[1].cached = None;
    let mut prefix = crate::xlsx_tests::book(1).sheets.remove(0);
    prefix.id = 1;
    prefix.name = "Prefix".into();
    book.sheets.insert(0, prefix);
    let job = seed(&mut w, book);
    (t, w, job)
}
fn input(j: &ExtractionJob) -> ReadXlsxRange {
    dto(json!({"id":j.id,"sheet_id":7,"range":"A1:C30","header_row":1,"limit":20}))
}
#[test]
fn sparse_range_headers_ordinals_pagination_pure_read_and_restore() {
    let (t, w, j) = setup();
    let before = crate::extraction_tests::raw(&w.conn);
    let one = w.extraction_xlsx_range(input(&j)).unwrap();
    assert_eq!(one.sheet.visibility, Visibility::VeryHidden);
    assert_eq!(one.manifest.cell_count, 32);
    assert_eq!(one.headers.len(), 3);
    assert_eq!(one.headers[0].reference, "A1");
    assert_eq!(one.headers[0].item.as_ref().unwrap().ordinal, 1);
    assert!(one.headers[1].item.is_none());
    assert!(one.headers[2].item.is_none());
    assert_eq!(one.items.len(), 20);
    assert_eq!(one.items[1].cell.cached, None);
    assert_eq!(one.matching_cell_count, 30);
    assert!(!one.covers_range && !one.covers_generation);
    let mut i = input(&j);
    i.cursor = one.next_cursor.clone();
    let two = w.extraction_xlsx_range(i).unwrap();
    assert_eq!(two.items.len(), 10);
    assert_eq!(two.items[0].ordinal, 21);
    assert!(two.next_cursor.is_none());
    assert!(!two.covers_range && !two.covers_generation);
    let mut i = input(&j);
    i.limit = Some(64);
    i.header_row = None;
    let all = w.extraction_xlsx_range(i).unwrap();
    assert!(all.covers_range && !all.covers_generation);
    assert!(all.headers.is_empty());
    assert_eq!(crate::extraction_tests::raw(&w.conn), before);
    w.backup(&t.path().join("backup")).unwrap();
    Workspace::restore(&t.path().join("backup"), &t.path().join("restored")).unwrap();
    let restored = Workspace::open(&t.path().join("restored")).unwrap();
    assert_eq!(
        restored.extraction_xlsx_range(input(&j)).unwrap().items,
        one.items
    );
    let mut i = input(&j);
    i.cursor = one.next_cursor;
    assert!(restored.extraction_xlsx_range(i).is_err());
}
#[test]
fn cursor_binds_every_criterion_and_generation() {
    let (_t, w, j) = setup();
    let cursor = w
        .extraction_xlsx_range(input(&j))
        .unwrap()
        .next_cursor
        .unwrap();
    for key in ["sheet_id", "range", "header_row", "limit", "id"] {
        let mut i = serde_json::to_value(input(&j)).unwrap();
        i["cursor"] = json!(cursor);
        i[key] = match key {
            "sheet_id" => json!(1),
            "range" => json!("A1:B30"),
            "header_row" => json!(2),
            "limit" => json!(21),
            _ => json!(workspace::id()),
        };
        assert!(w.extraction_xlsx_range(dto(i)).is_err(), "{key}");
    }
    let mut parsed: Value = serde_json::from_str(&cursor).unwrap();
    for ordinal in [0, 30, 99999] {
        parsed["base"]["ordinal"] = json!(ordinal);
        let mut i = input(&j);
        i.cursor = Some(parsed.to_string());
        assert!(w.extraction_xlsx_range(i).is_err());
    }
    parsed["base"]["ordinal"] = json!(20);
    parsed["base"]["generation_id"] = json!(workspace::id());
    let mut i = input(&j);
    i.cursor = Some(parsed.to_string());
    assert!(w.extraction_xlsx_range(i).is_err());
    // Reemplazo canónico atómico, sin forzar versiones ni omitir claves foráneas.
    let old = j.generation.as_ref().unwrap();
    let p = xs::load(&w.conn, &old.id, &j.id, old.fragment_count).unwrap();
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
    xs::publish(&tx, &id, &j.id, &p).unwrap();
    tx.execute(
        "UPDATE extraction_jobs SET generation_id=?1 WHERE id=?2",
        params![id, j.id],
    )
    .unwrap();
    tx.commit().unwrap();
    let mut i = input(&j);
    i.cursor = Some(cursor);
    assert!(w.extraction_xlsx_range(i).is_err());
    assert!(w.extraction_xlsx_range(input(&j)).is_ok());
}
#[test]
fn whole_book_and_source_integrity_are_checked_outside_range() {
    for sql in [
        "UPDATE extraction_xlsx_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_xlsx_chunks)",
        "UPDATE extraction_xlsx_generations SET cell_count=cell_count+1",
        "UPDATE extraction_xlsx_generations SET representation_sha256=replace(representation_sha256,'a','b')",
    ] {
        let (_t, w, j) = setup();
        w.conn.execute_batch(sql).unwrap();
        let mut i = input(&j);
        i.range = "A1".into();
        assert!(w.extraction_xlsx_range(i).is_err(), "{sql}");
    }
    let (_t, w, j) = setup();
    std::fs::write(
        w.path.join("attachments").join(&j.attachment_id),
        b"changed",
    )
    .unwrap();
    assert!(w.extraction_xlsx_range(input(&j)).is_err());
}
#[test]
fn strict_range_boundaries_empty_cells_and_coverage() {
    let (_t, mut w, j) = setup();
    for (range, header, valid) in [
        ("A1:BL1", Some(1), true),
        ("A1:BM1", None, false),
        ("XFC1048576:XFD1048576", None, true),
        ("XFD1048577", None, false),
        ("XFE1", None, false),
        ("A0", None, false),
        ("A01", None, false),
        ("a1", None, false),
        ("$A$1", None, false),
        ("B2:A1", None, false),
        ("A1:B2:C3", None, false),
        (" A1", None, false),
        ("A1:B2", Some(3), false),
        ("A1:B2", Some(0), false),
    ] {
        let mut i = input(&j);
        i.range = range.into();
        i.header_row = header;
        assert_eq!(i.validate().is_ok(), valid, "{range}");
    }
    let mut i = input(&j);
    i.range = "B1:C30".into();
    let empty = w.extraction_xlsx_range(i).unwrap();
    assert!(
        empty.items.is_empty()
            && empty.next_cursor.is_none()
            && empty.covers_range
            && !empty.covers_generation
    );
    let mut i = input(&j);
    i.sheet_id = 99;
    assert_eq!(
        w.extraction_xlsx_range(i).unwrap_err().details["reason"],
        "xlsx_sheet_not_found"
    );
    let j = seed(&mut w, crate::xlsx_tests::book(0));
    let p = w
        .extraction_xlsx_range(dto(
            json!({"id":j.id,"sheet_id":1,"range":"A1","header_row":1}),
        ))
        .unwrap();
    assert!(p.covers_range && p.covers_generation && p.headers[0].item.is_none());
}
#[test]
fn ranges_are_strict_non_batchable_shared_read_operations() {
    let (_t, w, j) = setup();
    let read =
        crate::scoped::LocalScope::new(&w.path, crate::scoped::Policy::new(false, false).unwrap())
            .unwrap();
    let input = serde_json::to_value(input(&j)).unwrap();
    let op = dto(json!({"operation":"extraction.xlsx_range","input":input}));
    assert_eq!(
        crate::scoped::operation_access(&op),
        crate::scoped::Access::Read
    );
    let before = crate::extraction_tests::raw(&w.conn);
    let result = read.execute_json(serde_json::to_value(op).unwrap(), "test");
    assert!(result.ok);
    assert_eq!(crate::extraction_tests::raw(&w.conn), before);
    let mut bad = input.clone();
    bad["helper"] = json!("/bin/sh");
    assert!(serde_json::from_value::<ReadXlsxRange>(bad).is_err());
    for limit in [0, 65] {
        let mut bad = input.clone();
        bad["limit"] = json!(limit);
        assert!(dto::<ReadXlsxRange>(bad).validate().is_err());
    }
    let plan = json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"x","operation":"extraction.xlsx_range","input":input}]});
    assert!(serde_json::from_value::<BatchPlan>(plan).is_err());
    assert_eq!(
        store::job(&w.conn, &w.path, &j.id).unwrap().state,
        ExtractionState::Succeeded
    );
}
