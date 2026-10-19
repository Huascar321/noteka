use crate::{
    attachment_search as api, attachment_search_storage as index,
    batch_storage::Budget,
    contracts::*,
    error::*,
    extraction_tests::raw,
    workspace::{self, Workspace},
    xlsx_index_projection::project,
    xlsx_storage as xs,
    xlsx_tests::book,
};
use noteka_xlsx_protocol::{CellValue, Formula, FormulaKind, Visibility, Workbook};
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::{fs, time::Duration};

fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
// Fixture de persistencia: publica un Workbook conocido sin usar el parser como oracle.
// La equivalencia original->Workbook se prueba aparte por el corpus CLI independiente.
fn seed(w: &mut Workspace, wb: Workbook) -> ExtractionJob {
    let n: u32 = w
        .conn
        .query_row("SELECT count(*)+1 FROM attachments", [], |r| r.get(0))
        .unwrap();
    let source = w.path.parent().unwrap().join(format!("fixture-{n}.xlsx"));
    fs::write(&source, crate::xlsx_tests::source(n)).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"xlsx_cells"})))
        .unwrap()
        .job;
    let p = xs::prepare(wb).unwrap();
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
fn rev(w: &Workspace) -> u32 {
    index::control(&w.conn).unwrap().revision
}
fn input(w: &Workspace, j: &ExtractionJob) -> AttachmentIndexInput {
    dto(
        json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":rev(w)}),
    )
}
fn put(w: &Workspace, j: &ExtractionJob) -> Value {
    serde_json::to_value(api::index(&w.path, input(w, j)).unwrap()).unwrap()
}
fn get(w: &Workspace) -> Value {
    serde_json::to_value(api::index(&w.path, dto(json!({"action":"get"}))).unwrap()).unwrap()
}
fn search(w: &Workspace, q: &str) -> AttachmentSearchPage {
    api::search(&w.path, dto(json!({"query":q}))).unwrap()
}
fn projected(wb: &Workbook) -> Vec<AttachmentSearchFragment> {
    let c = Connection::open_in_memory().unwrap();
    let b = Budget::new(&c).unwrap();
    project(wb, &b).unwrap()
}

#[test]
fn projection_types_order_hidden_sheets_and_separate_formulas() {
    let mut wb = book(8);
    let values = [
        Some(CellValue::Text("Ñ café".into())),
        Some(CellValue::Number("+01.2300E-9999".into())),
        Some(CellValue::Boolean(true)),
        Some(CellValue::Boolean(false)),
        Some(CellValue::Error("#DIV/0!".into())),
        Some(CellValue::DateIso("2026-10-15T12:30:00Z".into())),
        Some(CellValue::Text(String::new())),
        None,
    ];
    for (c, v) in wb.sheets[0].cells.iter_mut().zip(values) {
        c.cached = v;
    }
    wb.sheets[0].cells[0].formula = Some(Formula {
        kind: FormulaKind::Shared,
        text: "SUM(A2:A5)".into(),
        reference: Some("A1:A2".into()),
        shared_index: Some(4),
    });
    let mut hidden = wb.sheets[0].clone();
    hidden.id = 7;
    hidden.name = "Hidden".into();
    hidden.visibility = Visibility::VeryHidden;
    hidden.cells[0].formula = None;
    wb.sheets.push(hidden);
    wb.sheets[0].cells[1].formula = Some(Formula {
        kind: FormulaKind::Shared,
        text: String::new(),
        reference: None,
        shared_index: Some(4),
    });
    wb.validate().unwrap();
    let p = projected(&wb);
    assert_eq!(
        p.iter().map(|f| f.text.as_str()).collect::<Vec<_>>(),
        vec![
            "Ñ café",
            "SUM(A2:A5)",
            "+01.2300E-9999",
            "true",
            "false",
            "#DIV/0!",
            "2026-10-15T12:30:00Z",
            "Ñ café",
            "+01.2300E-9999",
            "true",
            "false",
            "#DIV/0!",
            "2026-10-15T12:30:00Z"
        ]
    );
    for (i, f) in p.iter().enumerate() {
        assert_eq!(f.ordinal, i as u32);
    }
    assert!(matches!(
        &p[7].locator,
        AttachmentTextLocator::XlsxCellText {
            cell_ordinal: 8,
            sheet_id: 7,
            field: XlsxTextField::CachedText,
            ..
        }
    ));
    assert!(matches!(
        &p[1].locator,
        AttachmentTextLocator::XlsxCellText {
            cell_ordinal: 0,
            field: XlsxTextField::FormulaText,
            ..
        }
    ));
}

#[test]
fn projection_unicode_ranges_reconstruct_without_overlap_or_trim() {
    let mut wb = book(1);
    let text = format!("{}🚀 Ñ\n{}", "a".repeat(4095), "é".repeat(4096));
    wb.sheets[0].cells[0].cached = Some(CellValue::Text(text.clone()));
    let p = projected(&wb);
    assert_eq!(p[0].text.len(), 4095);
    assert_eq!(p.iter().map(|f| f.text.as_str()).collect::<String>(), text);
    let mut end = 0;
    for f in p {
        match f.locator {
            AttachmentTextLocator::XlsxCellText {
                field_bytes,
                text_byte_start,
                text_byte_end,
                ..
            } => {
                assert_eq!(text_byte_start, end);
                assert_eq!(field_bytes, text.len() as u32);
                assert_eq!(
                    &text[text_byte_start as usize..text_byte_end as usize],
                    f.text
                );
                assert!(f.text.len() <= 4096);
                end = text_byte_end;
            }
            _ => panic!(),
        }
    }
    assert_eq!(end, text.len() as u32);
}

#[test]
fn projection_512_513_and_deadline_boundaries_are_explicit() {
    let wb = book(512);
    assert_eq!(projected(&wb).len(), 512);
    let c = Connection::open_in_memory().unwrap();
    let b = Budget::new(&c).unwrap();
    assert_eq!(
        project(&book(513), &b).unwrap_err().details["reason"],
        "index_budget_exceeded"
    );
    drop(b);
    let b = Budget::with_duration(&c, Duration::ZERO).unwrap();
    assert!(matches!(
        project(&book(0), &b).unwrap_err().code,
        ErrorCode::WorkspaceBusy
    ));
}

#[test]
fn strict_xlsx_locator_enums_and_unknown_fields_are_rejected() {
    let good = json!({"kind":"xlsx_cell_text","projection":"xlsx_cell_fields_v1","cell_ordinal":4,"sheet_id":7,"reference":"XFD100","field":"cached_number","field_bytes":4,"text_byte_start":0,"text_byte_end":4});
    let loc: AttachmentTextLocator = dto(good.clone());
    assert_eq!(serde_json::to_value(loc).unwrap(), good);
    for (key, value) in [
        ("projection", json!("future")),
        ("field", json!("unknown")),
        ("unexpected", json!(true)),
        ("cell_ordinal", json!(-1)),
    ] {
        let mut bad = good.clone();
        bad[key] = value;
        assert!(serde_json::from_value::<AttachmentTextLocator>(bad).is_err());
    }
    assert_eq!(
        serde_json::to_value(IndexedExtractor::XlsxCells).unwrap(),
        "xlsx_cells"
    );
}

#[test]
fn explicit_selection_noop_provenance_and_no_cross_field_matching() {
    let (_t, mut w) = setup();
    let mut wb = book(3);
    wb.sheets[0].cells[0].cached = Some(CellValue::Text("evidencia uno".into()));
    wb.sheets[0].cells[0].formula = Some(Formula {
        kind: FormulaKind::Normal,
        text: "formula dos".into(),
        reference: None,
        shared_index: None,
    });
    wb.sheets[0].cells[1].cached = Some(CellValue::Text("tres".into()));
    let j = seed(&mut w, wb);
    let launches = crate::xlsx_supervisor::spawn_count();
    assert!(search(&w, "evidencia").items.is_empty());
    assert_eq!(get(&w)["coverage"]["unindexed_supported_generations"], 1);
    let original = raw(&w.conn);
    let p = put(&w, &j);
    assert_eq!(p["changed"], true);
    let confirmed = raw(&w.conn);
    let no = put(&w, &j);
    assert_eq!(no["changed"], false);
    assert_eq!(raw(&w.conn), confirmed);
    assert_eq!(p["entry"], no["entry"]);
    for q in ["uno dos", "dos tres"] {
        assert!(search(&w, q).items.is_empty());
    }
    let page = search(&w, "evidencia");
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.items[0].source.generation_id,
        j.generation.as_ref().unwrap().id
    );
    assert_eq!(page.items[0].text, "evidencia uno");
    assert!(matches!(
        page.items[0].locator,
        AttachmentTextLocator::XlsxCellText {
            cell_ordinal: 0,
            field: XlsxTextField::CachedText,
            ..
        }
    ));
    for (name, rows) in original {
        if !name.starts_with("extraction_search_") {
            assert_eq!(
                confirmed.iter().find(|(n, _)| n == &name).unwrap().1,
                rows,
                "{name}"
            );
        }
    }
    assert_eq!(crate::xlsx_supervisor::spawn_count(), launches);
}

#[test]
fn empty_projection_and_excess_selection_preserve_full_reader() {
    let (_t, mut w) = setup();
    let empty = seed(&mut w, book(0));
    let p = put(&w, &empty);
    assert_eq!(p["entry"]["fragment_count"], 0);
    assert_eq!(p["entry"]["text_bytes"], 0);
    let j = seed(&mut w, book(513));
    let before = raw(&w.conn);
    assert_eq!(
        api::index(&w.path, input(&w, &j)).unwrap_err().details["reason"],
        "index_budget_exceeded"
    );
    assert_eq!(raw(&w.conn), before);
    assert_eq!(get(&w)["coverage"]["unindexed_supported_generations"], 1);
    assert_eq!(
        w.extraction_xlsx_cells(ReadXlsxCells {
            id: j.id,
            limit: Some(64),
            cursor: None
        })
        .unwrap()
        .manifest
        .cell_count,
        513
    );
}

#[test]
fn literal_matching_does_not_join_slices_or_repair_cut_tokens() {
    let (_t, mut w) = setup();
    let mut wb = book(2);
    let text = format!("left {}right", " ".repeat(4091));
    assert_eq!(&text.as_bytes()[4096..], b"right");
    wb.sheets[0].cells[0].cached = Some(CellValue::Text(text));
    wb.sheets[0].cells[1].cached = Some(CellValue::Text(format!("{}splittoken", " ".repeat(4093))));
    let j = seed(&mut w, wb);
    put(&w, &j);
    assert_eq!(search(&w, "left").items.len(), 1);
    assert_eq!(search(&w, "right").items.len(), 1);
    assert!(search(&w, "left right").items.is_empty());
    assert!(search(&w, "splittoken").items.is_empty());
    assert_eq!(search(&w, "spl").items.len(), 1);
    assert_eq!(search(&w, "ittoken").items.len(), 1);
}

#[test]
fn canonical_representation_cap_dominates_xlsx_projected_text_cap() {
    let wb = crate::xlsx_tests::sized_book(1_048_576);
    let prepared = xs::prepare(wb.clone()).unwrap();
    assert_eq!(
        prepared.chunks.iter().map(|s| s.len()).sum::<usize>(),
        1_048_576
    );
    let projected = projected(&wb);
    assert!(projected.iter().map(|s| s.text.len()).sum::<usize>() < 1_048_576);
    assert!(projected.len() <= 512);
    // JSON aporta claves/metadatos/comillas y escapes no menores al string;
    // cada campo se usa una sola vez, incluidos true/false de igual longitud.
    // No fabricar un Workbook inválido >1 MiB para alcanzar el límite dominado.
}

#[test]
fn corruption_outside_filter_or_page_never_returns_empty_success() {
    for sql in [
        "UPDATE extraction_xlsx_chunks SET text=replace(text,'9007199254740993','8007199254740993')",
        "UPDATE extraction_xlsx_generations SET cell_count=cell_count+1",
        "UPDATE extraction_xlsx_chunks SET job_id='foreign'",
        "UPDATE extraction_search_fts SET text='forged'",
        "UPDATE extraction_search_entries SET content_hash=printf('%064d',0)",
    ] {
        let (_t, mut w) = setup();
        let j = seed(&mut w, book(40));
        put(&w, &j);
        w.conn.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
        w.conn.execute_batch(sql).unwrap();
        w.conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let before = raw(&w.conn);
        assert!(
            api::index(
                &w.path,
                dto(json!({"action":"get","extractor":"utf8_text","limit":1}))
            )
            .is_err(),
            "{sql}"
        );
        assert!(
            api::search(
                &w.path,
                dto(json!({"query":"notfound","extractor":"utf8_text","limit":1}))
            )
            .is_err(),
            "{sql}"
        );
        assert_eq!(raw(&w.conn), before);
    }
}

#[test]
fn same_text_but_changed_valid_metadata_invalidates_selection_hash() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, book(2));
    put(&w, &j);
    let mut wb = book(2);
    wb.sheets[0].name = "Renamed metadata only".into();
    let p = xs::prepare(wb).unwrap();
    let g = j.generation.as_ref().unwrap();
    w.conn
        .execute(
            "DELETE FROM extraction_xlsx_generations WHERE generation_id=?1",
            [&g.id],
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE extraction_generations SET fragment_count=?1 WHERE id=?2",
            params![p.chunks.len() as u32, g.id],
        )
        .unwrap();
    xs::publish(&w.conn, &g.id, &j.id, &p).unwrap();
    assert!(api::index(&w.path, dto(json!({"action":"get"}))).is_err());
    assert!(api::search(&w.path, dto(json!({"query":"missing"}))).is_err());
}

#[test]
fn original_validation_is_candidate_scoped_but_includes_lookahead() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, book(3));
    put(&w, &j);
    fs::write(w.path.join("attachments").join(j.attachment_id), b"altered").unwrap();
    assert_eq!(get(&w)["coverage"]["source_validation"], "not_checked");
    assert!(search(&w, "nonexistent").items.is_empty());
    assert!(api::search(&w.path, dto(json!({"query":"9007199254740993","limit":1}))).is_err());
}

#[test]
fn concurrent_reset_after_preparation_revalidates_revision() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, book(2));
    let expected = rev(&w);
    let mut reset = false;
    let err = api::index_with_hook(&w.path, input(&w, &j), &mut |stage| {
        if stage == "prepared" {
            api::index(
                &w.path,
                AttachmentIndexInput::Reset {
                    expected_revision: expected,
                },
            )?;
            reset = true;
        }
        Ok(())
    })
    .unwrap_err();
    assert!(reset);
    assert!(matches!(err.code, ErrorCode::RevisionConflict));
    assert_eq!(get(&w)["items"], json!([]));
}

#[test]
fn source_change_after_preparation_cannot_publish() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, book(2));
    let before = raw(&w.conn);
    let err = api::index_with_hook(&w.path, input(&w, &j), &mut |stage| {
        if stage == "prepared" {
            for p in fs::read_dir(w.path.join("attachments"))? {
                fs::write(p?.path(), b"changed")?;
            }
        }
        Ok(())
    })
    .unwrap_err();
    assert!(matches!(err.code, ErrorCode::StorageError));
    assert_eq!(raw(&w.conn), before);
}

#[test]
fn migration_rollback_before_and_after_version_update_keeps_v11() {
    for before_update in [false, true] {
        let (_t, mut w) = setup();
        let _j = seed(&mut w, book(2));
        w.conn
            .execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;UPDATE workspace_meta SET schema_version=11;PRAGMA user_version=11")
            .unwrap();
        if before_update {
            w.conn.execute_batch("CREATE TRIGGER fail_v12 BEFORE UPDATE OF schema_version ON workspace_meta WHEN NEW.schema_version=12 BEGIN SELECT RAISE(ABORT,'injected'); END").unwrap();
        }
        let before = raw(&w.conn);
        let e = w.migrate(!before_update).unwrap_err();
        assert_eq!(raw(&w.conn), before);
        assert_eq!(workspace::version(&w.conn).unwrap(), 11);
        assert!(
            std::path::Path::new(e.details["backup_path"].as_str().unwrap())
                .join("manifest.json")
                .exists()
        );
    }
}

#[test]
fn mixed_backup_restore_and_repaired_outer_hash_are_fully_validated() {
    for corruption in ["none", "representation", "postings", "original"] {
        let (t, mut w) = setup();
        let j = seed(&mut w, book(70));
        put(&w, &j);
        let text = crate::attachment_search_tests::add(&mut w, b"UTF8 evidence", "utf8_text");
        put(&w, &text);
        let pdf_source = crate::pdf_tests::source(&[b"PDF evidence".to_vec()]);
        let pdf = crate::attachment_search_tests::add(&mut w, &pdf_source, "pdf_text");
        put(&w, &pdf);
        let launches = crate::xlsx_supervisor::spawn_count();
        let pdf_launches = crate::pdf_supervisor::spawn_count();
        let expected = search(&w, "9007199254740993");
        let backup = t.path().join("backup");
        w.backup(&backup).unwrap();
        if corruption == "original" {
            fs::write(
                backup.join("attachments").join(&j.attachment_id),
                b"tampered",
            )
            .unwrap();
        } else if corruption != "none" {
            let c = Connection::open(backup.join(workspace::DB_NAME)).unwrap();
            let sql = if corruption == "representation" {
                "UPDATE extraction_xlsx_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_xlsx_chunks)"
            } else {
                // FTS5 conserva su copia de texto: sólo cambia su índice invertido.
                "UPDATE extraction_search_fts SET text='forged' WHERE text='9007199254740993.0001';UPDATE extraction_search_fts_content SET c3='9007199254740993.0001' WHERE c3='forged'"
            };
            c.execute_batch(sql).unwrap();
            drop(c);
            let path = backup.join("manifest.json");
            let mut manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            manifest["sha256"] = json!(workspace::hash(&backup.join(workspace::DB_NAME)).unwrap());
            fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        }
        let restored = t.path().join("restored");
        if corruption == "none" {
            Workspace::restore(&backup, &restored).unwrap();
            let opened = Workspace::open(&restored).unwrap();
            let mut actual = serde_json::to_value(search(&opened, "9007199254740993")).unwrap();
            let old_cursor = expected.next_cursor.clone().unwrap();
            let mut expected = serde_json::to_value(expected).unwrap();
            assert_ne!(actual["next_cursor"], expected["next_cursor"]); // ruta distinta
            actual.as_object_mut().unwrap().remove("next_cursor");
            expected.as_object_mut().unwrap().remove("next_cursor");
            assert_eq!(actual, expected);
            assert_eq!(
                api::search(
                    &opened.path,
                    dto(json!({"query":"9007199254740993","limit":20,"cursor":old_cursor}))
                )
                .unwrap_err()
                .details["reason"],
                "cursor_scope_mismatch"
            );
            assert_eq!(get(&opened), get(&w));
            assert!(Workspace::restore(&backup, &restored).is_err());
            assert_eq!(get(&opened), get(&w));
        } else {
            assert!(
                Workspace::restore(&backup, &restored).is_err(),
                "{corruption}"
            );
            assert!(!restored.exists());
        }
        assert_eq!(crate::xlsx_supervisor::spawn_count(), launches);
        assert_eq!(crate::pdf_supervisor::spawn_count(), pdf_launches);
    }
}

#[test]
fn xlsx_index_search_and_reader_invoke_match_typed_calls() {
    let (_t, mut w) = setup();
    let a = seed(&mut w, book(3));
    let b = seed(&mut w, book(2));
    let invoke = |op: &str, input: Value| {
        let r = crate::execute_json(
            json!({"workspace":w.path,"command":{"operation":op,"input":input}}),
            "test",
        );
        assert!(r.ok, "{:?}", r.error);
        r.data.unwrap()
    };
    let first = invoke(
        "extraction.index",
        serde_json::to_value(input(&w, &a)).unwrap(),
    );
    assert_eq!(first["changed"], true);
    put(&w, &b);
    assert_eq!(
        invoke(
            "extraction.index",
            serde_json::to_value(input(&w, &a)).unwrap()
        ),
        put(&w, &a)
    );
    for (op, payload) in [
        (
            "extraction.index",
            json!({"action":"get","extractor":"xlsx_cells","limit":1}),
        ),
        (
            "extraction.search",
            json!({"query":"9007199254740993","extractor":"xlsx_cells","limit":1}),
        ),
    ] {
        let typed = if op == "extraction.index" {
            serde_json::to_value(api::index(&w.path, dto(payload.clone())).unwrap()).unwrap()
        } else {
            serde_json::to_value(api::search(&w.path, dto(payload.clone())).unwrap()).unwrap()
        };
        assert_eq!(invoke(op, payload.clone()), typed);
        let mut next = payload;
        next["cursor"] = typed["next_cursor"].clone();
        assert!(next["cursor"].is_string());
        let typed = if op == "extraction.index" {
            serde_json::to_value(api::index(&w.path, dto(next.clone())).unwrap()).unwrap()
        } else {
            serde_json::to_value(api::search(&w.path, dto(next.clone())).unwrap()).unwrap()
        };
        assert_eq!(invoke(op, next), typed);
    }
    let before = raw(&w.conn);
    assert_eq!(
        invoke("extraction.xlsx_cells", json!({"id":a.id,"limit":1})),
        serde_json::to_value(
            w.extraction_xlsx_cells(ReadXlsxCells {
                id: a.id,
                limit: Some(1),
                cursor: None
            })
            .unwrap()
        )
        .unwrap()
    );
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn derived_xlsx_selection_between_preview_apply_preserves_original_hash_and_index() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, book(4));
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"batch"}}]}),
    );
    let base = crate::batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap();
    let preview = crate::batches::preview(&w.path, plan.clone(), "test", "preview").unwrap();
    put(&w, &j);
    assert_eq!(
        base,
        crate::batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap()
    );
    let derived = |w: &Workspace| {
        raw(&w.conn)
            .into_iter()
            .filter(|(n, _)| n.starts_with("extraction_search_"))
            .collect::<Vec<_>>()
    };
    let selected = derived(&w);
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
    assert_eq!(selected, derived(&w));
    assert_eq!(search(&w, "9007199254740993").items.len(), 4);
    for op in [
        "extraction.index",
        "extraction.search",
        "extraction.xlsx_cells",
    ] {
        for check in [false, true] {
            let mut plan = json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"must rollback"}}]});
            let item = json!({"key":"x","operation":op,"input":{}});
            if check {
                plan["checks"] = json!([item]);
            } else {
                plan["steps"].as_array_mut().unwrap().push(item);
            }
            let before = raw(&w.conn);
            let response = crate::execute_json(
                json!({"workspace":w.path,"command":{"operation":"changes.preview","input":plan}}),
                "test",
            );
            assert!(!response.ok);
            assert!(matches!(
                response.error.unwrap().code,
                ErrorCode::ValidationError
            ));
            assert_eq!(raw(&w.conn), before);
        }
    }
}

#[test]
fn exact_xlsx_global_quotas_reject_one_more_without_eviction() {
    use crate::attachment_search_tests::{clone_generation, fixture_select};
    for cap in ["entries", "fragments", "bytes"] {
        let (_t, mut w) = setup();
        let mut wb = book(match cap {
            "entries" => 0,
            "fragments" => 512,
            _ => 4,
        });
        if cap == "bytes" {
            for c in &mut wb.sheets[0].cells {
                c.cached = Some(CellValue::Text("x".repeat(131072)));
            }
        }
        let j = seed(&mut w, wb);
        let count = match cap {
            "entries" => 1024,
            "fragments" => 32,
            _ => 16,
        };
        // Población SQL explícita para evitar 1024 puts cuadráticos; la operación
        // probada es el siguiente put real, que revalida todo antes de rechazarlo.
        fixture_select(&w, &j, 1);
        for slot in 2..=count {
            let clone = clone_generation(&w, &j);
            fixture_select(&w, &clone, slot);
        }
        let mut tiny = book(1);
        tiny.sheets[0].cells[0].cached = Some(CellValue::Text("x".into()));
        let extra = seed(&mut w, tiny);
        let before = raw(&w.conn);
        let error = api::index(&w.path, input(&w, &extra)).unwrap_err();
        assert_eq!(
            error.details["reason"],
            match cap {
                "entries" => "index_entry_limit",
                "fragments" => "index_fragment_limit",
                _ => "index_text_size_limit",
            }
        );
        assert_eq!(before, raw(&w.conn));
        let coverage = get(&w);
        assert_eq!(coverage["coverage"]["indexed_generations"], count);
        assert_eq!(coverage["coverage"]["unindexed_supported_generations"], 1);
        if cap == "bytes" {
            assert_eq!(coverage["coverage"]["text_bytes"], 8_388_608);
        }
        if cap == "fragments" {
            assert_eq!(coverage["coverage"]["fragments"], 16_384);
        }
    }
}

#[test]
fn xlsx_snapshot_cap_and_future_index_are_closed_without_cleanup() {
    let (_t, mut w) = setup();
    let j = seed(&mut w, book(1));
    put(&w, &j);
    let extra = seed(&mut w, book(2));
    w.conn.execute_batch("CREATE TABLE future_original(data BLOB);INSERT INTO future_original VALUES(zeroblob(67108864))").unwrap();
    let error = api::index(&w.path, input(&w, &extra)).unwrap_err();
    assert_eq!(error.details["reason"], "snapshot_size_limit");
    assert_eq!(get(&w)["coverage"]["indexed_generations"], 1);
    assert_eq!(
        w.conn
            .query_row("SELECT length(data) FROM future_original", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        67_108_864
    );
    drop(w);
    let (_t, mut w) = setup();
    let j = seed(&mut w, book(1));
    put(&w, &j);
    let expected_revision = rev(&w);
    for sql in [
        "UPDATE extraction_search_control SET version=2",
        "UPDATE extraction_search_control SET version=1,profile='future_xlsx'",
    ] {
        w.conn.execute_batch(sql).unwrap();
        let before = raw(&w.conn);
        for action in [
            json!({"action":"get"}),
            json!({"action":"reset","expected_revision":expected_revision}),
        ] {
            assert!(api::index(&w.path, dto(action)).is_err());
            assert_eq!(before, raw(&w.conn));
        }
    }
}

#[test]
fn xlsx_lookahead_checks_a_different_original() {
    let (_t, mut w) = setup();
    let a = seed(&mut w, book(1));
    let b = seed(&mut w, book(1));
    put(&w, &a);
    put(&w, &b);
    let first = api::search(&w.path, dto(json!({"query":"9007199254740993","limit":1}))).unwrap();
    let next = api::search(
        &w.path,
        dto(json!({"query":"9007199254740993","limit":1,"cursor":first.next_cursor})),
    )
    .unwrap();
    assert_ne!(
        first.items[0].source.attachment_id,
        next.items[0].source.attachment_id
    );
    fs::remove_file(
        w.path
            .join("attachments")
            .join(&next.items[0].source.attachment_id),
    )
    .unwrap();
    assert!(api::search(&w.path, dto(json!({"query":"9007199254740993","limit":1}))).is_err());
    assert_eq!(get(&w)["coverage"]["source_validation"], "not_checked");
}

#[test]
fn prepared_xlsx_put_loses_to_other_put_or_published_generation() {
    for publication in [false, true] {
        let (_t, mut w) = setup();
        let j = seed(&mut w, book(2));
        let old = j.generation.as_ref().unwrap().id.clone();
        let error=api::index_with_hook(&w.path,input(&w,&j),&mut |stage|{
            if stage=="prepared" {
                if publication {
                    let tx=w.conn.unchecked_transaction()?;let id=workspace::id();
                    tx.execute("UPDATE extraction_jobs SET generation_id=NULL WHERE id=?1",[&j.id])?;
                    tx.execute("DELETE FROM extraction_generations WHERE id=?1",[&old])?;
                    let p=xs::prepare(book(3))?;
                    tx.execute("INSERT INTO extraction_generations SELECT ?1,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?2,created_at,1 FROM extraction_jobs WHERE id=?3",params![id,p.chunks.len() as u32,j.id])?;
                    xs::publish(&tx,&id,&j.id,&p)?;
                    tx.execute("UPDATE extraction_jobs SET generation_id=?1 WHERE id=?2",params![id,j.id])?;tx.commit()?;
                } else {api::index(&w.path,input(&w,&j))?;}
            }Ok(())
        }).unwrap_err();
        assert!(matches!(error.code, ErrorCode::RevisionConflict));
        assert_eq!(
            get(&w)["coverage"]["indexed_generations"],
            if publication { 0 } else { 1 }
        );
    }
}

#[test]
fn xlsx_publication_rollback_before_commit_keeps_all_rows() {
    for stage in ["locked", "before_commit"] {
        let (_t, mut w) = setup();
        let j = seed(&mut w, book(70));
        let before = raw(&w.conn);
        assert!(
            api::index_with_hook(&w.path, input(&w, &j), &mut |s| {
                if s == stage {
                    Err(CoreError::new(ErrorCode::StorageError, "injected"))
                } else {
                    Ok(())
                }
            })
            .is_err()
        );
        assert_eq!(before, raw(&w.conn));
    }
}

#[test]
fn xlsx_posting_fast_and_complete_paths_compare_the_same_fields() {
    for large_vocabulary in [false, true] {
        let mut wb = book(8);
        let text = if large_vocabulary {
            (0..300)
                .map(|i| format!("term{i:03}"))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            "CAFÉ café αββ quoted\" a a 😀 token".into()
        };
        for c in &mut wb.sheets[0].cells {
            c.cached = Some(CellValue::Text(text.clone()));
        }
        let fields = projected(&wb);
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE VIRTUAL TABLE extraction_search_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);CREATE VIRTUAL TABLE temp.attachment_actual_vocab USING fts5vocab('main','extraction_search_fts','instance');CREATE TEMP TABLE attachment_expected_vocab(term TEXT,doc INTEGER,col TEXT,offset INTEGER)").unwrap();
        for f in &fields {
            c.execute(
                "INSERT INTO extraction_search_fts(rowid,text) VALUES(?1,?2)",
                params![f.ordinal + 1, f.text],
            )
            .unwrap();
        }
        c.execute_batch("INSERT INTO attachment_expected_vocab SELECT term,doc,col,offset FROM attachment_actual_vocab").unwrap();
        let pairs = fields
            .iter()
            .map(|f| (f.ordinal + 1, f.text.as_str()))
            .collect::<Vec<_>>();
        let budget = Budget::new(&c).unwrap();
        assert_eq!(
            index::same_repeated_postings(&c, &budget, &pairs).unwrap(),
            Some(true)
        );
        let registered: u32 = c
            .query_row(
                "SELECT count(*) FROM pragma_function_list WHERE name='noteka_verify_postings_v1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(registered, 1);
        assert!(index::same_postings(&c, &budget).unwrap());
        c.execute_batch("UPDATE extraction_search_fts SET text='forged' WHERE rowid=1")
            .unwrap();
        c.execute(
            "UPDATE extraction_search_fts_content SET c3=?1 WHERE id=1",
            [&text],
        )
        .unwrap();
        assert!(!index::same_postings(&c, &budget).unwrap());
        assert_eq!(
            index::same_repeated_postings(&c, &budget, &pairs).unwrap(),
            Some(false)
        );
    }
}

// Sólo revierte las adiciones nombradas para conservar los fingerprints históricos,
// nunca modifica sus constantes. La comparación independiente completa vive en Python.
pub(crate) fn legacy_index_shapes(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            if let Some(defs) = obj.get_mut("$defs").and_then(Value::as_object_mut) {
                for name in ["OcrTextProjection", "OcrBoundingBox"] {
                    defs.remove(name);
                }
                defs.remove("XlsxTextProjection");
                defs.remove("XlsxTextField");
                if let Some(values) = defs
                    .get_mut("IndexedExtractor")
                    .and_then(|v| v.get_mut("enum"))
                    .and_then(Value::as_array_mut)
                {
                    values.retain(|v| v != "xlsx_cells" && v != "image_ocr");
                }
                if let Some(variants) = defs
                    .get_mut("AttachmentTextLocator")
                    .and_then(|v| v.get_mut("oneOf"))
                    .and_then(Value::as_array_mut)
                {
                    variants.retain(|v| {
                        v["properties"]["kind"]["const"] != "xlsx_cell_text"
                            && v["properties"]["kind"]["const"] != "image_ocr_region_text"
                    });
                }
            }
            for v in obj.values_mut() {
                legacy_index_shapes(v);
            }
        }
        Value::Array(values) => {
            for v in values {
                legacy_index_shapes(v);
            }
        }
        _ => {}
    }
}

#[test]
#[ignore = "Gate release mixto explícito: fixture CLI independiente, cinco warmups/veinte muestras"]
fn release_xlsx_search_resources() {
    use std::time::Instant;
    let path =
        std::path::PathBuf::from(std::env::var("NOTEKA_XLSX_SEARCH_BENCH_WORKSPACE").unwrap());
    let w = Workspace::open(&path).unwrap();
    assert_eq!(get(&w)["coverage"]["text_bytes"], 8_388_608);
    let jobs: Vec<String> = w
        .conn
        .prepare("SELECT job_id FROM extraction_search_entries ORDER BY slot")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    assert_eq!(jobs.len(), 12);
    let xlsx = w
        .extraction_get(IdInput {
            id: jobs[11].clone(),
        })
        .unwrap();
    let query_cases = vec![
        ("query-hit", json!({"query":"evidencia","limit":64})),
        ("query-zero", json!({"query":"ausente"})),
        (
            "query-filter-utf8",
            json!({"query":"café contrato","extractor":"utf8_text","limit":64}),
        ),
        (
            "query-filter-pdf",
            json!({"query":"evidencia","extractor":"pdf_text","limit":64}),
        ),
        (
            "query-filter-xlsx",
            json!({"query":"evidencia","extractor":"xlsx_cells","limit":64}),
        ),
        ("query-distinct", json!({"query":"term299","limit":64})),
        ("query-page", json!({"query":"evidencia","limit":1})),
    ];
    let mut results = Vec::new();
    for (name, input) in query_cases {
        for round in 0..25 {
            let start = Instant::now();
            let page = api::search(&path, dto(input.clone()));
            results.push(json!({"case":name,"round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"ok":page.is_ok(),"error":page.as_ref().err()}));
            if name == "query-page"
                && let Ok(page) = page
                && let Some(cursor) = page.next_cursor
            {
                let start = Instant::now();
                let next = api::search(
                    &path,
                    dto(json!({"query":"evidencia","limit":1,"cursor":cursor})),
                );
                results.push(json!({"case":"query-page-next","round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"ok":next.is_ok(),"error":next.err()}));
            }
        }
    }
    for round in 0..25 {
        let start = Instant::now();
        let page = api::index(&path, dto(json!({"action":"get","limit":64})));
        results.push(json!({"case":"get","round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"ok":page.is_ok(),"error":page.err()}));
    }
    for name in ["put", "no-op"] {
        for round in 0..25 {
            if name == "put" {
                w.conn
                    .execute(
                        "DELETE FROM extraction_search_entries WHERE job_id=?1",
                        [&xlsx.id],
                    )
                    .unwrap();
            }
            let mut locked = None;
            let mut lock_ms = None;
            let mut editor = None;
            let start = Instant::now();
            let result = api::index_with_hook(&path, input(&w, &xlsx), &mut |stage| {
                if stage == "locked" {
                    locked = Some(Instant::now());
                    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
                    let other = barrier.clone();
                    let db = path.join(workspace::DB_NAME);
                    editor = Some(std::thread::spawn(move || {
                        let conn = Connection::open(db).unwrap();
                        conn.busy_timeout(Duration::from_secs(2)).unwrap();
                        other.wait();
                        let start = Instant::now();
                        conn.execute_batch("BEGIN IMMEDIATE;ROLLBACK").unwrap();
                        start.elapsed().as_secs_f64() * 1000.
                    }));
                    barrier.wait();
                }
                if stage == "committed" {
                    lock_ms = locked.map(|s| s.elapsed().as_secs_f64() * 1000.);
                }
                Ok(())
            });
            let wall = start.elapsed().as_secs_f64() * 1000.;
            let wait = editor.map(|h| h.join().unwrap());
            results.push(json!({"case":name,"round":round,"warmup":round<5,"wall_ms":wall,"lock_ms":lock_ms,"editor_wait_ms":wait,"ok":result.is_ok(),"error":result.err()}));
        }
    }
    fn rss(who: i32) -> i64 {
        let mut value = std::mem::MaybeUninit::<libc::rusage>::zeroed();
        assert_eq!(unsafe { libc::getrusage(who, value.as_mut_ptr()) }, 0);
        unsafe { value.assume_init() }.ru_maxrss
    }
    let report = json!({"corpus":"noteka-mixed-fields-v1","text_bytes":8_388_608,"warmups":5,"samples":20,"results":results,"rss_self_native_units":rss(libc::RUSAGE_SELF),"rss_children_native_units":rss(libc::RUSAGE_CHILDREN),"rss_method":"separate getrusage high-water; bytes on macOS, not additive simultaneous RSS","sqlite_bytes":fs::metadata(path.join(workspace::DB_NAME)).unwrap().len()});
    fs::write(
        std::env::var("NOTEKA_XLSX_SEARCH_BENCH_REPORT").unwrap(),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "Diagnóstico de costo mixto; no gate ni sustituto de aceptación"]
fn mixed_index_cost_diagnostics() {
    use std::time::Instant;
    let path =
        std::path::PathBuf::from(std::env::var("NOTEKA_XLSX_SEARCH_BENCH_WORKSPACE").unwrap());
    for sample in 0..8 {
        let start = Instant::now();
        let (c, _) = index::open(&path, false).unwrap();
        let open = start.elapsed().as_secs_f64() * 1000.;
        let b = Budget::new(&c).unwrap();
        let tx = c.unchecked_transaction().unwrap();
        let jobs = tx
            .prepare("SELECT job_id FROM extraction_search_entries ORDER BY slot")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
        let start = Instant::now();
        for job in jobs {
            index::canonical(&tx, &job, &b).unwrap();
        }
        let canonical = start.elapsed().as_secs_f64() * 1000.;
        let start = Instant::now();
        let verified = index::verify(&tx, &b).unwrap();
        let verify = start.elapsed().as_secs_f64() * 1000.;
        let pairs = verified
            .entries
            .iter()
            .flat_map(|s| {
                s.data
                    .fragments
                    .iter()
                    .map(move |f| ((s.slot - 1) * 512 + f.ordinal + 1, f.text.as_str()))
            })
            .collect::<Vec<_>>();
        let start = Instant::now();
        assert_eq!(
            index::same_repeated_postings(&tx, &b, &pairs).unwrap(),
            Some(true)
        );
        let posting = start.elapsed().as_secs_f64() * 1000.;
        println!(
            "{}",
            json!({"sample":sample,"open_ms":open,"canonical_ms":canonical,"verify_ms":verify,"posting_ms":posting})
        );
    }
}
