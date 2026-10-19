use crate::{
    contracts::*,
    extraction_storage as store,
    workspace::{self, Workspace},
    xlsx_storage as xs,
};
use noteka_xlsx_protocol::{
    Cell, CellValue, LIMITATIONS, NumberStyle, Sheet, Visibility, Workbook,
};
use rusqlite::params;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;

fn dto<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> T {
    serde_json::from_value(v).unwrap()
}
pub(crate) fn book(n: u32) -> Workbook {
    let cells = (1..=n)
        .map(|row| Cell {
            reference: format!("A{row}"),
            row,
            column: 1,
            style: 0,
            cached: Some(CellValue::Number("9007199254740993.0001".into())),
            formula: None,
        })
        .collect::<Vec<_>>();
    Workbook {
        date_1904: false,
        limitations: LIMITATIONS.iter().map(|s| s.to_string()).collect(),
        styles: vec![NumberStyle {
            num_fmt_id: 0,
            format_code: None,
        }],
        sheets: vec![Sheet {
            id: 1,
            name: "Sheet1".into(),
            visibility: Visibility::Visible,
            declared_range: None,
            observed_range: noteka_xlsx_protocol::observed(&cells),
            merges: Vec::new(),
            cells,
        }],
    }
}
// ZIP Stored pequeño, sin dependencia de ZIP/XML en el core ni oracle del parser.
fn pack(parts: Vec<(&str, String)>) -> Vec<u8> {
    fn h(out: &mut Vec<u8>, x: u16) {
        out.extend(x.to_le_bytes());
    }
    fn d(out: &mut Vec<u8>, x: u32) {
        out.extend(x.to_le_bytes());
    }
    fn crc(b: &[u8]) -> u32 {
        let mut c = !0u32;
        for &b in b {
            c ^= u32::from(b);
            for _ in 0..8 {
                c = (c >> 1) ^ if c & 1 != 0 { 0xedb88320 } else { 0 };
            }
        }
        !c
    }
    let (mut out, mut central) = (Vec::new(), Vec::new());
    let count = parts.len() as u16;
    for (name, data) in parts {
        let offset = out.len() as u32;
        let len = data.len() as u32;
        let crc = crc(data.as_bytes());
        d(&mut out, 0x04034b50);
        for x in [20, 0, 0, 0, 0] {
            h(&mut out, x);
        }
        for x in [crc, len, len] {
            d(&mut out, x);
        }
        h(&mut out, name.len() as u16);
        h(&mut out, 0);
        out.extend(name.as_bytes());
        out.extend(data.as_bytes());
        d(&mut central, 0x02014b50);
        for x in [20, 20, 0, 0, 0, 0] {
            h(&mut central, x);
        }
        for x in [crc, len, len] {
            d(&mut central, x);
        }
        for x in [name.len() as u16, 0, 0, 0, 0] {
            h(&mut central, x);
        }
        d(&mut central, 0);
        d(&mut central, offset);
        central.extend(name.as_bytes());
    }
    let start = out.len() as u32;
    let size = central.len() as u32;
    out.extend(central);
    d(&mut out, 0x06054b50);
    for x in [0, 0, count, count] {
        h(&mut out, x);
    }
    d(&mut out, size);
    d(&mut out, start);
    h(&mut out, 0);
    out
}
pub(crate) fn source(n: u32) -> Vec<u8> {
    let m = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
    let r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let p = "http://schemas.openxmlformats.org/package/2006/relationships";
    let rows = (1..=n)
        .map(|i| format!("<row r=\"{i}\"><c r=\"A{i}\"><v>9007199254740993.0001</v></c></row>"))
        .collect::<String>();
    pack(vec![
        (
            "[Content_Types].xml",
            "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/><Override PartName=\"/xl/worksheets/sheet1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/></Types>".to_owned(),
        ),
        (
            "_rels/.rels",
            format!(
                "<Relationships xmlns=\"{p}\"><Relationship Id=\"root\" Type=\"{r}/officeDocument\" Target=\"xl/workbook.xml\"/></Relationships>"
            ),
        ),
        (
            "xl/workbook.xml",
            format!(
                "<workbook xmlns=\"{m}\" xmlns:r=\"{r}\"><sheets><sheet name=\"Sheet1\" sheetId=\"1\" r:id=\"r1\"/></sheets></workbook>"
            ),
        ),
        (
            "xl/_rels/workbook.xml.rels",
            format!(
                "<Relationships xmlns=\"{p}\"><Relationship Id=\"r1\" Type=\"{r}/worksheet\" Target=\"worksheets/sheet1.xml\"/></Relationships>"
            ),
        ),
        (
            "xl/worksheets/sheet1.xml",
            format!("<worksheet xmlns=\"{m}\"><sheetData>{rows}</sheetData></worksheet>"),
        ),
    ])
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
fn add(w: &mut Workspace, n: u32) -> ExtractionJob {
    let path = w.path.parent().unwrap().join(format!("book-{n}.xlsx"));
    fs::write(&path, source(n)).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: path.to_string_lossy().into(),
        })
        .unwrap();
    w.extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"xlsx_cells"})))
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
fn read(w: &Workspace, j: &ExtractionJob, limit: u32, cursor: Option<String>) -> XlsxCellPage {
    w.extraction_xlsx_cells(ReadXlsxCells {
        id: j.id.clone(),
        limit: Some(limit),
        cursor,
    })
    .unwrap()
}

#[test]
fn all_51_prior_operation_schemas_and_errors_keep_independent_baseline_hashes() {
    let mut c = crate::describe();
    crate::xlsx_search_tests::legacy_index_shapes(&mut c);
    for (key, hash) in [
        (
            "inputs",
            "731e5788f1d2dee3d9b1f8581c37b9fca1ff1422deab63d39ef0d61366c41964",
        ),
        (
            "outputs",
            "6908f102edc4ec35c2819539b3b1ca64496e734ea41befde3ba05ec36f4337db",
        ),
        (
            "error_schema",
            "b913819780ad2efb726a35422f2653d4c4fa09f2b05848650d6706046a9fd0d5",
        ),
    ] {
        if key != "error_schema" {
            let group = c[key].as_object_mut().unwrap();
            assert_eq!(group.len(), 53);
            assert!(group.remove("extraction.ocr_regions").is_some());
            assert_eq!(group.len(), 52);
            assert!(group.remove("extraction.xlsx_cells").is_some());
            assert_eq!(group.len(), 51);
        }
        assert_eq!(
            format!("{:x}", Sha256::digest(serde_json::to_vec(&c[key]).unwrap())),
            hash,
            "{key}"
        );
    }
}

#[test]
fn explicit_worker_pagination_lexemes_and_cursors() {
    let (_t, mut w) = setup();
    let launches = crate::xlsx_supervisor::spawn_count();
    let j = add(&mut w, 70);
    assert!(
        w.extraction_enqueue(dto(
            json!({"attachment_id":j.attachment_id,"extractor":"xlsx_cells"})
        ))
        .unwrap()
        .deduplicated
    );
    w.extraction_run(RunExtraction::default()).unwrap();
    assert_eq!(crate::xlsx_supervisor::spawn_count(), launches);
    enable(&w);
    let j = run(&w, &j);
    assert_eq!(j.state, ExtractionState::Succeeded);
    assert_eq!(crate::xlsx_supervisor::spawn_count(), launches + 1);
    let p = read(&w, &j, 32, None);
    assert_eq!(p.manifest.cell_count, 70);
    assert_eq!(p.items.len(), 32);
    assert!(!p.covers_generation);
    assert_eq!(p.items[0].cell.cached, book(1).sheets[0].cells[0].cached);
    // Mismo request que entrega invoke del host, sin abrir ventana ni simular IPC.
    let invoked = crate::execute_json(
        json!({"workspace":w.path,"command":{"operation":"extraction.xlsx_cells","input":{"id":j.id,"limit":32}}}),
        "desktop",
    );
    assert!(invoked.ok, "{:?}", invoked.error);
    assert_eq!(invoked.data.unwrap(), serde_json::to_value(&p).unwrap());
    let mut items = p.items;
    let mut cursor = p.next_cursor.clone();
    while let Some(c) = cursor {
        let q = read(&w, &j, 32, Some(c));
        cursor = q.next_cursor;
        items.extend(q.items);
    }
    assert_eq!(
        items.into_iter().map(|x| x.cell).collect::<Vec<_>>(),
        book(70).sheets.remove(0).cells
    );
    assert!(
        w.extraction_xlsx_cells(ReadXlsxCells {
            id: j.id.clone(),
            limit: Some(64),
            cursor: p.next_cursor
        })
        .is_err()
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
    let rev = crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision;
    let put=crate::attachment_search::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":rev}))).unwrap();
    assert!(matches!(
        put,
        AttachmentIndexResult::Put {
            changed: true,
            entry: AttachmentIndexEntry {
                fragment_count: 70,
                ..
            },
            ..
        }
    ));
}

#[test]
fn canonical_chunks_long_unicode_and_corruption_outside_page() {
    let (_t, mut w) = setup();
    let j = add(&mut w, 70);
    enable(&w);
    let j = run(&w, &j);
    let g = j.generation.as_ref().unwrap();
    let mut wb = book(70);
    wb.sheets[0].cells[69].cached = Some(CellValue::Text("Ñ🚀".repeat(12000)));
    let p = xs::prepare(wb.clone()).unwrap();
    assert!(p.chunks.len() > 10);
    assert!(p.chunks.iter().all(|s| s.len() <= 4096));
    // Sustitución transaccional de una generación válida para probar almacenamiento, sin oracle parser.
    w.conn
        .execute(
            "DELETE FROM extraction_xlsx_generations WHERE generation_id=?1",
            [&g.id],
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE extraction_generations SET fragment_count=?2 WHERE id=?1",
            params![g.id, p.chunks.len() as u32],
        )
        .unwrap();
    xs::publish(&w.conn, &g.id, &j.id, &p).unwrap();
    assert_eq!(
        xs::load(&w.conn, &g.id, &j.id, p.chunks.len() as u32)
            .unwrap()
            .workbook,
        wb
    );
    assert_eq!(read(&w, &j, 1, None).manifest.cell_count, 70);
    w.conn.execute("UPDATE extraction_xlsx_chunks SET text=replace(text,'Ñ','X') WHERE generation_id=?1 AND ordinal=(SELECT max(ordinal) FROM extraction_xlsx_chunks WHERE generation_id=?1)",[&g.id]).unwrap();
    assert!(
        w.extraction_xlsx_cells(ReadXlsxCells {
            id: j.id,
            limit: Some(1),
            cursor: None
        })
        .is_err()
    );
    assert!(
        w.backup(&w.path.parent().unwrap().join("bad-backup"))
            .is_err()
    );
}

#[test]
fn v10_migration_rollback_and_restore_keep_existing_rows() {
    let (t, mut w) = setup();
    let j = crate::extraction_tests::raw(&w.conn);
    assert!(!j.is_empty());
    w.conn.execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations;UPDATE workspace_meta SET schema_version=10;PRAGMA user_version=10").unwrap();
    let before = crate::extraction_tests::raw(&w.conn);
    let e = w.migrate(true).unwrap_err();
    assert_eq!(workspace::version(&w.conn).unwrap(), 10);
    assert_eq!(crate::extraction_tests::raw(&w.conn), before);
    let backup = std::path::Path::new(e.details["backup_path"].as_str().unwrap());
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(backup.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], 10);
    w.migrate(false).unwrap();
    assert_eq!(
        workspace::version(&w.conn).unwrap(),
        workspace::SCHEMA_VERSION
    );
    for (name, rows) in before {
        if name != "workspace_meta" {
            assert_eq!(
                rows,
                crate::extraction_tests::raw(&w.conn)
                    .into_iter()
                    .find(|(n, _)| n == &name)
                    .unwrap()
                    .1,
                "{name}"
            );
        }
    }
    let j = add(&mut w, 0);
    enable(&w);
    let j = run(&w, &j);
    let p = read(&w, &j, 32, None);
    assert!(p.items.is_empty() && p.covers_generation && p.next_cursor.is_none());
    let path = t.path().join("backup");
    w.backup(&path).unwrap();
    let restored = t.path().join("restored");
    Workspace::restore(&path, &restored).unwrap();
    let r = Workspace::open(&restored).unwrap();
    assert_eq!(read(&r, &j, 32, None).manifest, p.manifest);
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
fn xlsx_fences_preserve_prior_generation_and_replacement_invalidates_cursor() {
    for stage in ["source_read", "extracted", "before_publish"] {
        for mode in ["pause", "cancel", "source", "config", "owner", "lease"] {
            let (_t, mut w) = setup();
            let j = add(&mut w, 70);
            enable(&w);
            let j = run(&w, &j);
            let old = read(&w, &j, 1, None);
            retry(&w, &j);
            let bytes = fs::read(w.path.join("attachments").join(&j.attachment_id)).unwrap();
            let mut hook = |at: &str| {
                if at == stage {
                    let other = Workspace::open(&w.path).unwrap();
                    match mode {
                        "pause" => {
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
                        "cancel" => {
                            let c = other.extraction_get(IdInput { id: j.id.clone() }).unwrap();
                            other
                                .extraction_cancel(ExtractionRevision {
                                    id: c.id,
                                    expected_revision: c.revision,
                                })
                                .unwrap();
                        }
                        "source" => {
                            let mut bad = bytes.clone();
                            bad[0] ^= 1;
                            fs::write(w.path.join("attachments").join(&j.attachment_id), bad)
                                .unwrap();
                        }
                        "config" => {
                            let config = json!({"version":2,"chunk_bytes":4096});
                            let hash = store::config_hash(&config).unwrap();
                            other.conn.execute("UPDATE extraction_jobs SET config=?1,config_hash=?2 WHERE id=?3",params![config.to_string(),hash,j.id]).unwrap();
                        }
                        "owner" => {
                            other
                                .conn
                                .execute(
                                    "UPDATE extraction_runtime SET token=?1 WHERE id=1",
                                    [workspace::id()],
                                )
                                .unwrap();
                        }
                        "lease" => {
                            other.conn.execute("UPDATE extraction_runtime SET lease_until='2020-01-01T00:00:00Z' WHERE id=1",[]).unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
            };
            let result = w.extraction_run_impl(
                RunExtraction {
                    id: Some(j.id.clone()),
                },
                Some(&mut hook),
                None,
            );
            assert!(
                result.is_err()
                    || matches!(result.unwrap().outcome, ExtractionOutcome::Interrupted),
                "{stage}/{mode}"
            );
            fs::write(w.path.join("attachments").join(&j.attachment_id), &bytes).unwrap();
            if mode == "config" {
                let config = json!(ExtractionConfig::default());
                w.conn
                    .execute(
                        "UPDATE extraction_jobs SET config=?1,config_hash=?2 WHERE id=?3",
                        params![
                            config.to_string(),
                            store::config_hash(&config).unwrap(),
                            j.id
                        ],
                    )
                    .unwrap();
            }
            assert_eq!(
                read(&w, &j, 1, None).generation.id,
                old.generation.id,
                "{stage}/{mode}"
            );
            let n: u32 = w
                .conn
                .query_row(
                    "SELECT count(*) FROM extraction_xlsx_generations",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(n, 1);
        }
    }
    let (_t, mut w) = setup();
    let j = add(&mut w, 70);
    enable(&w);
    let j = run(&w, &j);
    let p = read(&w, &j, 1, None);
    retry(&w, &j);
    let new = run(&w, &j);
    assert_ne!(new.generation.unwrap().id, p.generation.id);
    assert_eq!(
        w.extraction_xlsx_cells(ReadXlsxCells {
            id: j.id,
            limit: Some(1),
            cursor: p.next_cursor
        })
        .unwrap_err()
        .details["reason"],
        "cursor_scope_mismatch"
    );
}

#[test]
fn stored_manifest_chunk_owner_counts_and_canonicality_fail_closed() {
    for sql in [
        "UPDATE extraction_xlsx_generations SET representation_sha256=printf('%064d',0)",
        "UPDATE extraction_xlsx_generations SET version=2",
        "UPDATE extraction_xlsx_generations SET parser='future'",
        "UPDATE extraction_xlsx_generations SET parser=printf('%131072s','future')",
        "UPDATE extraction_xlsx_generations SET representation_bytes=representation_bytes-1",
        "UPDATE extraction_xlsx_generations SET sheet_count=2",
        "UPDATE extraction_xlsx_generations SET cell_count=1",
        "UPDATE extraction_xlsx_generations SET chunk_count=chunk_count+1",
        "UPDATE extraction_xlsx_chunks SET ordinal=ordinal+100",
        "DELETE FROM extraction_xlsx_chunks WHERE ordinal=1",
        "UPDATE extraction_xlsx_chunks SET text='x'||substr(text,2) WHERE ordinal=1",
        "PRAGMA foreign_keys=OFF;UPDATE extraction_xlsx_chunks SET job_id='other';PRAGMA foreign_keys=ON",
        "INSERT INTO extraction_fragments SELECT generation_id,job_id,0,'x',0,1,1,1 FROM extraction_xlsx_generations",
    ] {
        let (_t, mut w) = setup();
        let j = add(&mut w, 70);
        enable(&w);
        let j = run(&w, &j);
        let queued = add(&mut w, 71);
        w.conn.execute_batch(sql).unwrap();
        let damaged = crate::extraction_tests::raw(&w.conn);
        assert!(
            w.extraction_xlsx_cells(ReadXlsxCells {
                id: j.id,
                limit: Some(1),
                cursor: None
            })
            .is_err(),
            "{sql}"
        );
        assert!(store::integrity(&w.conn).is_err(), "{sql}");
        assert!(
            w.extraction_run(RunExtraction {
                id: Some(queued.id.clone())
            })
            .is_err(),
            "{sql}"
        );
        assert_eq!(
            w.extraction_get(IdInput { id: queued.id })
                .unwrap()
                .attempts,
            0
        );
        assert_eq!(crate::extraction_tests::raw(&w.conn), damaged);
    }
    // JSON semánticamente válido pero no canónico, con hash/conteos reparados, sigue rechazado.
    let (_t, mut w) = setup();
    let j = add(&mut w, 1);
    enable(&w);
    let j = run(&w, &j);
    let id = j.generation.as_ref().unwrap().id.clone();
    let mut raw = serde_json::to_string(&book(1)).unwrap();
    raw.push(' ');
    w.conn
        .execute(
            "UPDATE extraction_xlsx_chunks SET text=?1 WHERE generation_id=?2",
            params![raw, id],
        )
        .unwrap();
    w.conn.execute("UPDATE extraction_xlsx_generations SET representation_bytes=?1,representation_sha256=?2 WHERE generation_id=?3",params![raw.len() as u32,format!("{:x}",Sha256::digest(raw.as_bytes())),id]).unwrap();
    assert!(read_result(&w, &j, None).is_err());
}

fn read_result(
    w: &Workspace,
    j: &ExtractionJob,
    cursor: Option<String>,
) -> crate::error::Result<XlsxCellPage> {
    w.extraction_xlsx_cells(ReadXlsxCells {
        id: j.id.clone(),
        limit: Some(1),
        cursor,
    })
}

#[test]
fn open_current_xlsx_validates_all_chunks_without_helper_or_repair() {
    let (_t, mut w) = setup();
    let j = add(&mut w, 70);
    enable(&w);
    let _ = run(&w, &j);
    let launches = crate::xlsx_supervisor::spawn_count();
    let opened = Workspace::open(&w.path).unwrap();
    assert_eq!(read(&opened, &j, 1, None).manifest.cell_count, 70);
    drop(opened);
    w.conn.execute("UPDATE extraction_xlsx_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_xlsx_chunks)", []).unwrap();
    let before = crate::extraction_tests::raw(&w.conn);
    assert_eq!(
        Workspace::open(&w.path).err().unwrap().details["reason"],
        "invalid_xlsx_integrity"
    );
    assert_eq!(before, crate::extraction_tests::raw(&w.conn));
    assert_eq!(crate::xlsx_supervisor::spawn_count(), launches);
}

#[test]
fn restore_rejects_tampered_xlsx_backup_even_with_repaired_outer_hash() {
    for original in [false, true] {
        let (t, mut w) = setup();
        let j = add(&mut w, 70);
        enable(&w);
        let j = run(&w, &j);
        let backup = t.path().join("backup");
        w.backup(&backup).unwrap();
        if original {
            let path = backup.join("attachments").join(&j.attachment_id);
            let mut bytes = fs::read(&path).unwrap();
            bytes[0] ^= 1;
            fs::write(path, bytes).unwrap();
        } else {
            let db = rusqlite::Connection::open(backup.join(workspace::DB_NAME)).unwrap();
            db.execute("UPDATE extraction_xlsx_chunks SET text='x'||substr(text,2) WHERE ordinal=(SELECT max(ordinal) FROM extraction_xlsx_chunks)", []).unwrap();
            drop(db);
            // Supera la primera frontera del hash de SQLite; debe fallar por
            // la integridad del derivado, no sólo por el manifiesto externo.
            let path = backup.join("manifest.json");
            let mut manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            manifest["sha256"] = json!(workspace::hash(&backup.join(workspace::DB_NAME)).unwrap());
            fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        }
        let launches = crate::xlsx_supervisor::spawn_count();
        let destination = t.path().join("rejected");
        assert!(matches!(
            Workspace::restore(&backup, &destination).unwrap_err().code,
            crate::error::ErrorCode::InvalidBackup
        ));
        assert!(!destination.exists());
        let occupied = t.path().join("occupied");
        fs::create_dir(&occupied).unwrap();
        fs::write(occupied.join("sentinel"), b"keep").unwrap();
        assert!(Workspace::restore(&backup, &occupied).is_err());
        assert_eq!(fs::read(occupied.join("sentinel")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(occupied).unwrap().count(), 1);
        assert_eq!(read(&w, &j, 1, None).manifest.cell_count, 70);
        assert_eq!(crate::xlsx_supervisor::spawn_count(), launches);
    }
}

#[test]
fn cursor_scope_operation_job_and_limit_are_not_interchangeable() {
    let (t, mut w) = setup();
    let j = add(&mut w, 70);
    enable(&w);
    let j = run(&w, &j);
    let p = read(&w, &j, 1, None);
    let other = add(&mut w, 71);
    let other = run(&w, &other);
    let c = p.next_cursor.unwrap();
    assert!(read_result(&w, &other, Some(c.clone())).is_err());
    for field in ["scope", "operation", "generation_id", "job_id"] {
        let mut value: serde_json::Value = serde_json::from_str(&c).unwrap();
        value[field] = json!("other");
        assert!(
            read_result(&w, &j, Some(value.to_string())).is_err(),
            "{field}"
        );
    }
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let restored = t.path().join("restored");
    Workspace::restore(&backup, &restored).unwrap();
    let r = Workspace::open(&restored).unwrap();
    assert!(read_result(&r, &j, Some(c)).is_err());
}

#[test]
fn xlsx_admission_future_configs_and_mime_do_not_spawn() {
    let (_t, mut w) = setup();
    let before = crate::xlsx_supervisor::spawn_count();
    let j = add(&mut w, 1);
    let mut a = crate::attachments::metadata(&w.conn, &j.attachment_id).unwrap();
    for mime in [
        "application/octet-stream",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ] {
        a.mime = mime.into();
        store::eligible_for(&a, "xlsx_cells").unwrap();
    }
    for (name, mime) in [
        ("a.xls", "application/octet-stream"),
        ("a.xlsx.exe", "application/octet-stream"),
        ("a.xlsx", "application/pdf"),
        ("a.xlsm", "application/octet-stream"),
    ] {
        a.name = name.into();
        a.mime = mime.into();
        assert_eq!(
            store::eligible_for(&a, "xlsx_cells").unwrap_err().details["reason"],
            "unsupported_source_format"
        );
    }
    let config = json!({"version":2,"chunk_bytes":4096});
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?1,config_hash=?2 WHERE id=?3",
            params![
                config.to_string(),
                store::config_hash(&config).unwrap(),
                j.id
            ],
        )
        .unwrap();
    let before_rows = crate::extraction_tests::raw(&w.conn);
    let future = w.extraction_get(IdInput { id: j.id.clone() }).unwrap();
    assert!(!future.supported);
    assert_eq!(
        w.extraction_run(RunExtraction { id: Some(j.id) })
            .unwrap_err()
            .details["reason"],
        "unsupported_extractor_config"
    );
    assert_eq!(crate::extraction_tests::raw(&w.conn), before_rows);
    assert_eq!(crate::xlsx_supervisor::spawn_count(), before);
}

pub(crate) fn sized_book(size: usize) -> Workbook {
    let mut b = book(8);
    for c in &mut b.sheets[0].cells {
        c.cached = Some(CellValue::Text(String::new()));
    }
    let overhead = serde_json::to_vec(&b).unwrap().len();
    let mut left = size.checked_sub(overhead).unwrap();
    for c in &mut b.sheets[0].cells {
        let n = left.min(131_072);
        c.cached = Some(CellValue::Text("x".repeat(n)));
        left -= n;
    }
    assert_eq!(left, 0);
    b.validate().unwrap();
    assert_eq!(serde_json::to_vec(&b).unwrap().len(), size);
    b
}
fn seed_generation(w: &Workspace, j: &ExtractionJob, count: u32) -> String {
    let id = workspace::id();
    w.conn.execute("INSERT INTO extraction_generations SELECT ?1,id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,source_bytes,?2,created_at,1 FROM extraction_jobs WHERE id=?3",params![id,count,j.id]).unwrap();
    w.conn
        .execute(
            "UPDATE extraction_jobs SET state='succeeded',generation_id=?1,attempts=1 WHERE id=?2",
            params![id, j.id],
        )
        .unwrap();
    id
}

#[test]
fn exact_mixed_budget_keeps_32_mib_and_replacement_subtracts_only_its_job() {
    let (_t, mut w) = setup();
    enable(&w);
    // Literal real de 1 MiB; PDF sintético con derivado preparado de tamaño conocido.
    let text = w.path.parent().unwrap().join("budget.txt");
    fs::write(&text, vec![b't'; 1_048_576]).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: text.to_string_lossy().into(),
        })
        .unwrap();
    let literal = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"utf8_text"})))
        .unwrap()
        .job;
    run(&w, &literal);
    let target_bytes = xs::prepare(book(0)).unwrap().manifest.representation_bytes as usize;
    let path = w.path.parent().unwrap().join("budget.pdf");
    fs::write(&path, crate::pdf_tests::source(&[b"synthetic".to_vec()])).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: path.to_string_lossy().into(),
        })
        .unwrap();
    let pdf = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"pdf_text"})))
        .unwrap()
        .job;
    let prepared = crate::pdf_storage::prepare(vec![noteka_pdf_protocol::PageText {
        page_number: 1,
        text: "p".repeat(1_048_576 - target_bytes),
    }])
    .unwrap();
    let id = seed_generation(&w, &pdf, prepared.fragments.len() as u32);
    crate::pdf_storage::publish(&w.conn, &id, &pdf.id, &prepared).unwrap();
    let large = xs::prepare(sized_book(1_048_576)).unwrap();
    for n in 1..=30 {
        let j = add(&mut w, n);
        let id = seed_generation(&w, &j, large.chunks.len() as u32);
        xs::publish(&w.conn, &id, &j.id, &large).unwrap();
    }
    let target = add(&mut w, 0);
    assert_eq!(
        xs::retained(&w.conn, &target.id).unwrap(),
        MAX_DERIVED_BYTES - target_bytes as u64
    );
    let done = run(&w, &target);
    assert_eq!(xs::retained(&w.conn, "").unwrap(), MAX_DERIVED_BYTES);
    retry(&w, &done);
    let done = run(&w, &done);
    assert_eq!(xs::retained(&w.conn, "").unwrap(), MAX_DERIVED_BYTES);
    let extra = add(&mut w, 31);
    let extra_bytes = xs::prepare(book(31)).unwrap().manifest.representation_bytes as usize;
    let smaller = crate::pdf_storage::prepare(vec![noteka_pdf_protocol::PageText {
        page_number: 1,
        text: "p".repeat(1_048_576 - target_bytes - extra_bytes + 1),
    }])
    .unwrap();
    w.conn
        .execute(
            "DELETE FROM extraction_pdf_generations WHERE generation_id=?1",
            [&id],
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE extraction_generations SET fragment_count=?1 WHERE id=?2",
            params![smaller.fragments.len() as u32, id],
        )
        .unwrap();
    crate::pdf_storage::publish(&w.conn, &id, &pdf.id, &smaller).unwrap();
    let before = xs::retained(&w.conn, "").unwrap();
    assert_eq!(before + extra_bytes as u64, MAX_DERIVED_BYTES + 1);
    let e = w
        .extraction_run(RunExtraction { id: Some(extra.id) })
        .unwrap_err();
    assert_eq!(e.details["reason"], "derived_size_limit");
    assert_eq!(xs::retained(&w.conn, "").unwrap(), before);
    assert_eq!(
        read(&w, &done, 1, None).generation.id,
        done.generation.unwrap().id
    );
    // Canonical result exact/+1; las cadenas individuales conservan su cap de 128 KiB.
    let b = sized_book(1_048_576);
    assert_eq!(xs::prepare(b.clone()).unwrap().chunks.len(), 256);
    let mut excess = b;
    let last = excess.sheets[0].cells.last_mut().unwrap();
    if let Some(CellValue::Text(s)) = &mut last.cached {
        s.push('x');
    }
    assert!(xs::prepare(excess).is_err());
}

#[test]
fn xlsx_explicit_index_preserves_existing_text_evidence_and_batch_exclusion() {
    let (_t, mut w) = setup();
    enable(&w);
    let path = w.path.parent().unwrap().join("indexed.txt");
    fs::write(&path, "Synthetic indexed original").unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: path.to_string_lossy().into(),
        })
        .unwrap();
    let text = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"utf8_text"})))
        .unwrap()
        .job;
    let text = run(&w, &text);
    let rev = crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision;
    crate::attachment_search::index(&w.path,dto(json!({"action":"put","job_id":text.id,"expected_generation_id":text.generation.unwrap().id,"expected_revision":rev}))).unwrap();
    let j = add(&mut w, 3);
    let j = run(&w, &j);
    let before = crate::extraction_tests::raw(&w.conn);
    let query = serde_json::to_value(
        crate::attachment_search::search(&w.path, dto(json!({"query":"Synthetic"}))).unwrap(),
    )
    .unwrap();
    let rev = crate::attachment_search_storage::control(&w.conn)
        .unwrap()
        .revision;
    let put=crate::attachment_search::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":rev}))).unwrap();
    assert!(matches!(
        put,
        AttachmentIndexResult::Put { changed: true, .. }
    ));
    let after = crate::extraction_tests::raw(&w.conn);
    for (name, rows) in &before {
        if !name.starts_with("extraction_search_") {
            assert_eq!(
                &after.iter().find(|(n, _)| n == name).unwrap().1,
                rows,
                "{name}"
            );
        }
    }
    let now = serde_json::to_value(
        crate::attachment_search::search(&w.path, dto(json!({"query":"Synthetic"}))).unwrap(),
    )
    .unwrap();
    assert_eq!(now["items"][0]["source"], query["items"][0]["source"]);
    assert_eq!(now["items"][0]["text"], query["items"][0]["text"]);
    assert_eq!(now["items"][0]["locator"], query["items"][0]["locator"]);
    assert_eq!(now["items"].as_array().unwrap().len(), 1);
    assert_eq!(now["coverage"]["indexed_generations"], 2);
    assert_eq!(now["coverage"]["unsupported_generations"], 0);
    assert_eq!(crate::attachment_search::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":rev}))).unwrap_err().details["reason"],"expected_revision_mismatch");
    let plan = json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"no","operation":"extraction.xlsx_cells","input":{"id":j.id}}]});
    assert!(serde_json::from_value::<BatchPlan>(plan).is_err());
    assert_eq!(after, crate::extraction_tests::raw(&w.conn));
}

#[test]
fn xlsx_deadline_immediately_before_commit_rolls_back_the_whole_replacement() {
    let (_t, mut w) = setup();
    let j = add(&mut w, 3);
    enable(&w);
    let j = run(&w, &j);
    retry(&w, &j);
    let mut reached = false;
    let mut hook = |stage: &str| {
        if stage == "before_commit" {
            reached = true;
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }
    };
    let e = w
        .extraction_run_impl(
            RunExtraction {
                id: Some(j.id.clone()),
            },
            Some(&mut hook),
            Some(std::time::Duration::from_millis(1000)),
        )
        .unwrap_err();
    assert!(reached);
    assert_eq!(e.details["reason"], "execution_deadline");
    let p = read(&w, &j, 64, None);
    assert_eq!(p.generation.id, j.generation.unwrap().id);
    assert!(p.result_from_previous_attempt);
    assert_eq!(
        w.conn
            .query_row(
                "SELECT count(*) FROM extraction_xlsx_generations",
                [],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
        1
    );
}

#[test]
#[ignore = "medición release explícita con corpus externo, no test unitario ordinario"]
fn release_xlsx_integration_measurements() {
    if cfg!(debug_assertions) {
        panic!("medir sólo release");
    }
    use std::{collections::HashMap, path::Path, time::Instant};
    let root = std::env::var("NOTEKA_XLSX_BENCH_ROOT").expect("directorio externo requerido");
    let root = Path::new(&root);
    let names: Vec<String> =
        serde_json::from_slice(&fs::read(root.join("cases.json")).unwrap()).unwrap();
    assert_eq!(names.len(), 77);
    let (_t, mut w) = setup();
    enable(&w);
    let mut records = Vec::new();
    fn p95(v: &[f64]) -> f64 {
        let mut v = v.to_vec();
        v.sort_by(f64::total_cmp);
        v[((v.len() * 95).div_ceil(100) - 1).min(v.len() - 1)]
    }
    for name in names {
        let a = w
            .attachment_import(ImportAttachment {
                source_path: root.join(format!("{name}.xlsx")).to_string_lossy().into(),
            })
            .unwrap();
        let mut job = w
            .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":"xlsx_cells"})))
            .unwrap()
            .job;
        let (mut runs, mut reads, mut claims, mut publishes) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for n in 0..25 {
            if n > 0 {
                job = retry(&w, &job);
            }
            let start = Instant::now();
            let mut stages = HashMap::new();
            let mut hook = |stage: &str| {
                stages.insert(stage.to_owned(), start.elapsed().as_secs_f64() * 1000.0);
            };
            job = w
                .extraction_run_impl(
                    RunExtraction {
                        id: Some(job.id.clone()),
                    },
                    Some(&mut hook),
                    None,
                )
                .unwrap()
                .job
                .unwrap();
            let run_ms = start.elapsed().as_secs_f64() * 1000.0;
            let start = Instant::now();
            let page = read(&w, &job, 64, None);
            let read_ms = start.elapsed().as_secs_f64() * 1000.0;
            assert_eq!(page.generation.id, job.generation.as_ref().unwrap().id);
            if n >= 5 {
                runs.push(run_ms);
                reads.push(read_ms);
                claims.push(stages["claim_committed"] - stages["claim_locked"]);
                publishes.push(stages["committed"] - stages["publish_locked"]);
            }
        }
        records.push(json!({"name":name,"source_bytes":job.source_bytes,"representation_bytes":read(&w,&job,64,None).manifest.representation_bytes,
            "run_samples_ms":runs,"read_64_samples_ms":reads,"claim_lock_samples_ms":claims,"publish_lock_samples_ms":publishes,
            "run_p95_ms":p95(&runs),"read_64_p95_ms":p95(&reads),"claim_lock_p95_ms":p95(&claims),"publish_lock_p95_ms":p95(&publishes)}));
    }
    let max_lock = records
        .iter()
        .flat_map(|r| {
            [
                r["claim_lock_p95_ms"].as_f64().unwrap(),
                r["publish_lock_p95_ms"].as_f64().unwrap(),
            ]
        })
        .fold(0.0, f64::max);
    #[cfg(unix)]
    let memory = {
        let mut own = unsafe { std::mem::zeroed::<libc::rusage>() };
        let mut children = unsafe { std::mem::zeroed::<libc::rusage>() };
        assert_eq!(unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut own) }, 0);
        assert_eq!(
            unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, &mut children) },
            0
        );
        let scale = if cfg!(target_os = "macos") { 1 } else { 1024 };
        json!({"parent_rss_high_water_bytes":own.ru_maxrss*scale,"waited_children_rss_high_water_bytes":children.ru_maxrss*scale,"method":"getrusage SELF/CHILDREN high-water, not live heap nor simultaneous summed RSS"})
    };
    #[cfg(not(unix))]
    let memory = json!(null);
    println!(
        "NOTEKA_XLSX_INTEGRATION_BENCH {}",
        json!({"warmups":5,"repetitions":20,"cases":records,"maximum_lock_p95_ms":max_lock,"memory":memory})
    );
    assert!(max_lock <= 150.0, "lock p95 {max_lock} ms >150 ms");
}
