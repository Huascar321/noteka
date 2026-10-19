use crate::{
    attachment_search as api, attachment_search_storage as index,
    batch_storage::{self, Budget},
    batches,
    contracts::*,
    error::*,
    extraction_tests::raw,
    workspace::{self, Workspace},
};
use rusqlite::Connection;
use rusqlite::params;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

fn dto<T: serde::de::DeserializeOwned>(v: Value) -> T {
    serde_json::from_value(v).unwrap()
}
fn setup() -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    (t, w)
}
fn enabled(w: &Workspace) {
    let c = w
        .extraction_control(ExtractionControlInput::Get {})
        .unwrap();
    w.extraction_control(ExtractionControlInput::Set {
        paused: false,
        expected_revision: c.revision,
    })
    .unwrap();
}
pub(crate) fn add(w: &mut Workspace, text: &[u8], ext: &str) -> ExtractionJob {
    let source = w.path.parent().unwrap().join(format!(
        "{}.{}",
        workspace::id(),
        if ext == "pdf_text" { "pdf" } else { "txt" }
    ));
    fs::write(&source, text).unwrap();
    let a = w
        .attachment_import(ImportAttachment {
            source_path: source.to_string_lossy().into(),
        })
        .unwrap();
    let j = w
        .extraction_enqueue(dto(json!({"attachment_id":a.id,"extractor":ext})))
        .unwrap()
        .job;
    enabled(w);
    w.extraction_run(RunExtraction { id: Some(j.id) })
        .unwrap()
        .job
        .unwrap()
}
fn revision(w: &Workspace) -> u32 {
    index::control(&w.conn).unwrap().revision
}
fn put(w: &Workspace, j: &ExtractionJob) -> Value {
    serde_json::to_value(api::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":revision(w)}))).unwrap()).unwrap()
}
fn get(w: &Workspace) -> Value {
    serde_json::to_value(api::index(&w.path, dto(json!({"action":"get"}))).unwrap()).unwrap()
}
fn query(w: &Workspace, query: &str) -> AttachmentSearchPage {
    api::search(&w.path, dto(json!({"query":query}))).unwrap()
}
fn reset(w: &Workspace) -> Value {
    serde_json::to_value(
        api::index(
            &w.path,
            AttachmentIndexInput::Reset {
                expected_revision: revision(w),
            },
        )
        .unwrap(),
    )
    .unwrap()
}
fn reason(e: &CoreError, r: &str) {
    assert_eq!(e.details["reason"], r, "{e:?}");
}

#[test]
fn schema_empty_registry_and_migration_rollback_preserve_v9() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"Original", "utf8_text");
    assert_eq!(
        workspace::version(&w.conn).unwrap(),
        workspace::SCHEMA_VERSION
    );
    batch_storage::structure(&w.conn).unwrap();
    assert_eq!(get(&w)["coverage"]["unindexed_supported_generations"], 1);
    assert!(query(&w, "Original").items.is_empty());
    w.conn
        .execute_batch("DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations")
        .unwrap();
    w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;UPDATE workspace_meta SET schema_version=9;PRAGMA user_version=9").unwrap();
    let before = raw(&w.conn);
    let err = w.migrate(true).unwrap_err();
    assert_eq!(before, raw(&w.conn));
    assert!(
        Path::new(err.details["backup_path"].as_str().unwrap())
            .join("manifest.json")
            .is_file()
    );
    let e = index::open(&w.path, false).err().unwrap();
    assert!(matches!(e.code, ErrorCode::UnsupportedWorkspaceVersion));
    w.migrate(false).unwrap();
    assert_eq!(revision(&w), 1);
    assert_eq!(get(&w)["items"], json!([]));
    let after = raw(&w.conn);
    for (name, rows) in before {
        if name != "workspace_meta" {
            assert_eq!(
                rows,
                after.iter().find(|(n, _)| *n == name).unwrap().1,
                "{name}"
            );
        }
    }
    assert_eq!(
        w.extraction_get(IdInput { id: j.id })
            .unwrap()
            .generation
            .unwrap()
            .fragment_count,
        1
    );
    batch_storage::structure(&w.conn).unwrap();
}

#[test]
fn strict_dtos_and_read_only_main_with_writable_temp() {
    let (_t, w) = setup();
    let before = raw(&w.conn);
    for input in [
        json!({"action":"put"}),
        json!({"action":"get","expected_revision":1}),
        json!({"action":"reset","expected_revision":null}),
        json!({"action":"get","provider":"remote"}),
        json!({"action":"get","limit":0}),
        json!({"action":"get","job_id":"../bad"}),
        json!({"action":"put","job_id":workspace::id(),"expected_generation_id":workspace::id(),"expected_revision":0}),
    ] {
        let r = crate::execute_json(
            json!({"workspace":w.path,"command":{"operation":"extraction.index","input":input}}),
            "test",
        );
        assert!(!r.ok);
        assert!(matches!(r.error.unwrap().code, ErrorCode::ValidationError));
    }
    let (c, _) = index::open(&w.path, false).unwrap();
    let tx = c.unchecked_transaction().unwrap();
    tx.execute_batch("CREATE TEMP TABLE proof(value);INSERT INTO temp.proof VALUES(1)")
        .unwrap();
    assert!(
        tx.execute("DELETE FROM main.extraction_search_control", [])
            .is_err()
    );
    let b = Budget::new(&c).unwrap();
    index::verify(&tx, &b).unwrap();
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn explicit_literal_index_noop_and_exact_multibyte_provenance() {
    let (_t, mut w) = setup();
    let text = format!("{}\r\n café OR NEAR fin", "é".repeat(2200));
    let j = add(&mut w, text.as_bytes(), "utf8_text");
    let original = raw(&w.conn);
    assert!(query(&w, "café").items.is_empty());
    assert_eq!(put(&w, &j)["changed"], true);
    assert_eq!(revision(&w), 2);
    let page = query(&w, "cafe OR");
    assert_eq!(page.items.len(), 1);
    let hit = &page.items[0];
    assert_eq!(hit.source.job_id, j.id);
    assert_eq!(hit.ordinal, 1);
    let literal = w
        .extraction_fragments(ReadFragments {
            id: j.id.clone(),
            limit: None,
            cursor: None,
        })
        .unwrap();
    assert_eq!(hit.text, literal.items[1].text);
    assert!(matches!(
        hit.locator,
        AttachmentTextLocator::Utf8Bytes {
            byte_start: 4096,
            ..
        }
    ));
    let indexed = raw(&w.conn);
    assert_eq!(put(&w, &j)["changed"], false);
    assert_eq!(revision(&w), 2);
    assert_eq!(indexed, raw(&w.conn));
    let e=api::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.unwrap().id,"expected_revision":1}))).unwrap_err();
    assert!(matches!(e.code, ErrorCode::RevisionConflict));
    for (name, rows) in original {
        if !name.starts_with("extraction_search_") {
            assert_eq!(
                rows,
                indexed.iter().find(|(n, _)| *n == name).unwrap().1,
                "{name}"
            );
        }
    }
}

#[test]
fn pdf_page_text_and_no_text_have_distinct_coverage_and_typed_locators() {
    let (_t, mut w) = setup();
    let bytes = crate::pdf_tests::source(&[
        b"Contrato cafe".to_vec(),
        Vec::new(),
        b"Otro contrato".to_vec(),
    ]);
    let j = add(&mut w, &bytes, "pdf_text");
    let empty = add(&mut w, &crate::pdf_tests::source(&[Vec::new()]), "pdf_text");
    put(&w, &j);
    put(&w, &empty);
    let page = query(&w, "contrato");
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.coverage.indexed_generations, 2);
    let literal = w
        .extraction_pdf_fragments(ReadPdfFragments {
            id: j.id.clone(),
            limit: None,
            cursor: None,
        })
        .unwrap();
    for h in &page.items {
        let f = &literal.items[h.ordinal as usize];
        assert_eq!(h.text, f.text);
        assert!(matches!(
            h.locator,
            AttachmentTextLocator::PdfPageText {
                page_number: 1 | 3,
                ..
            }
        ));
    }
    assert_eq!(get(&w)["coverage"]["fragments"], 2);
    assert_eq!(
        get(&w)["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["fragment_count"] == 0)
            .count(),
        1
    );
}

#[test]
fn bounded_literal_queries_filters_and_fragment_boundaries() {
    let (_t, mut w) = setup();
    let a = add(
        &mut w,
        format!("{}beta OR NEAR café", "alpha ".repeat(700)).as_bytes(),
        "utf8_text",
    );
    let b = add(&mut w, b"beta OR NEAR", "utf8_text");
    put(&w, &a);
    put(&w, &b);
    assert_eq!(query(&w, "OR NEAR").items.len(), 2);
    assert!(query(&w, "\"; DROP TABLE nope; --").items.is_empty());
    assert!(query(&w, "!!!").items.is_empty());
    for bad in ["".into(), "x".repeat(257), "x ".repeat(17), "a\0b".into()] {
        assert!(matches!(
            api::search(&w.path, dto(json!({"query":bad})))
                .unwrap_err()
                .code,
            ErrorCode::ValidationError
        ));
    }
    for input in [
        json!({"query":"beta","limit":0}),
        json!({"query":"beta","limit":65}),
        json!({"query":"beta","job_id":"bad"}),
    ] {
        assert!(api::search(&w.path, dto(input)).is_err());
    }
    let p = api::search(
        &w.path,
        dto(json!({"query":"beta","attachment_id":b.attachment_id,"extractor":"utf8_text"})),
    )
    .unwrap();
    assert_eq!(p.items.len(), 1);
    assert_eq!(p.items[0].source.job_id, b.id);
    let crossing = add(
        &mut w,
        format!("{} omega", "z".repeat(4096)).as_bytes(),
        "utf8_text",
    );
    put(&w, &crossing);
    let p = api::search(
        &w.path,
        dto(json!({"query":format!("{} omega","z".repeat(20)),"job_id":crossing.id})),
    )
    .unwrap();
    assert!(p.items.is_empty());
}

#[test]
fn cursors_paginate_ties_are_scoped_and_expire_only_on_index_changes() {
    let (_t, mut w) = setup();
    for _ in 0..3 {
        let j = add(&mut w, b"igual literal", "utf8_text");
        put(&w, &j);
    }
    let input = json!({"query":"igual","limit":1});
    let first = api::search(&w.path, dto(input.clone())).unwrap();
    let cursor = first.next_cursor.clone().unwrap();
    let second = api::search(
        &w.path,
        dto(json!({"query":"igual","limit":1,"cursor":cursor})),
    )
    .unwrap();
    let third = api::search(
        &w.path,
        dto(json!({"query":"igual","limit":1,"cursor":second.next_cursor})),
    )
    .unwrap();
    assert!(third.next_cursor.is_none());
    let ids = [
        &first.items[0].source.job_id,
        &second.items[0].source.job_id,
        &third.items[0].source.job_id,
    ];
    assert!(ids.windows(2).all(|a| a[0] < a[1]));
    for input in [
        json!({"query":"otro","limit":1,"cursor":cursor}),
        json!({"query":"igual","limit":2,"cursor":cursor}),
        json!({"action":"get","limit":1,"cursor":cursor}),
    ] {
        let e = if input.get("action").is_some() {
            api::index(&w.path, dto(input)).err().unwrap()
        } else {
            api::search(&w.path, dto(input)).err().unwrap()
        };
        assert!(matches!(e.code, ErrorCode::ValidationError));
    }
    reset(&w);
    reason(
        &api::search(
            &w.path,
            dto(json!({"query":"igual","limit":1,"cursor":cursor})),
        )
        .unwrap_err(),
        "index_revision_changed",
    );
    let r = revision(&w);
    reset(&w);
    assert_eq!(revision(&w), r + 1);
}

#[test]
fn logical_corruption_is_not_a_false_zero_and_reset_is_explicit() {
    for corruption in [
        "DELETE FROM extraction_search_fts",
        "UPDATE extraction_search_fts SET text='otro'",
        "UPDATE extraction_search_fts SET job_id='incorrecto'",
        "DELETE FROM extraction_search_fts_data WHERE id>10",
        "UPDATE extraction_search_entries SET content_hash=printf('%064d',0)",
    ] {
        let (_t, mut w) = setup();
        let j = add(&mut w, b"canonica evidencia", "utf8_text");
        put(&w, &j);
        w.conn.execute_batch(corruption).unwrap();
        let before = raw(&w.conn);
        let e = api::search(&w.path, dto(json!({"query":"noexiste"}))).unwrap_err();
        reason(&e, "invalid_attachment_index");
        assert_eq!(e.details["current_revision"], revision(&w));
        assert_eq!(before, raw(&w.conn));
        assert!(api::index(&w.path, dto(json!({"action":"get"}))).is_err());
        reset(&w);
        batch_storage::structure(&w.conn).unwrap();
        put(&w, &j);
        assert_eq!(query(&w, "evidencia").items.len(), 1);
    }
}

#[test]
fn put_and_reset_failures_roll_back_data_revision_and_ddl() {
    for stage in ["entry_inserted", "fragment_inserted", "before_commit"] {
        let (_t, mut w) = setup();
        let j = add(&mut w, b"safe", "utf8_text");
        let before = raw(&w.conn);
        let e=api::index_with_hook(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.unwrap().id,"expected_revision":revision(&w)})),&mut |s|if s==stage{Err(index::error("injected"))}else{Ok(())}).unwrap_err();
        reason(&e, "injected");
        assert_eq!(before, raw(&w.conn));
    }
    for stage in ["reset_dropped", "before_commit"] {
        let (_t, mut w) = setup();
        let j = add(&mut w, b"safe", "utf8_text");
        put(&w, &j);
        let before = raw(&w.conn);
        assert!(
            api::index_with_hook(
                &w.path,
                AttachmentIndexInput::Reset {
                    expected_revision: revision(&w)
                },
                &mut |s| if s == stage {
                    Err(index::error("injected"))
                } else {
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(before, raw(&w.conn));
        index::structure(&w.conn).unwrap();
        assert_eq!(query(&w, "safe").items.len(), 1);
    }
}

#[test]
fn changed_or_unsafe_sources_fail_without_mutation_or_refill() {
    for action in ["remove", "alter", "symlink"] {
        let (_t, mut w) = setup();
        let j = add(&mut w, b"evidencia", "utf8_text");
        put(&w, &j);
        let path = w.path.join("attachments").join(&j.attachment_id);
        match action {
            "remove" => fs::remove_file(&path).unwrap(),
            "alter" => fs::write(&path, b"alterado!").unwrap(),
            _ => {
                fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink(w.path.parent().unwrap().join("does-not-exist"), &path)
                    .unwrap();
            }
        }
        let before = raw(&w.conn);
        reason(
            &api::search(&w.path, dto(json!({"query":"evidencia"}))).unwrap_err(),
            "input_stale",
        );
        assert_eq!(before, raw(&w.conn));
        assert_eq!(get(&w)["coverage"]["source_validation"], "not_checked");
        assert!(query(&w, "noexiste").items.is_empty());
    }
}

#[test]
fn publication_invalidates_atomically_but_failed_attempt_keeps_prior_selection() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"conservada", "utf8_text");
    put(&w, &j);
    let rev = revision(&w);
    let base = batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap();
    let retry = w
        .extraction_retry(ExtractionRevision {
            id: j.id.clone(),
            expected_revision: j.revision,
        })
        .unwrap();
    assert!(query(&w, "conservada").items[0].result_from_previous_attempt);
    w.extraction_cancel(ExtractionRevision {
        id: j.id.clone(),
        expected_revision: retry.revision,
    })
    .unwrap();
    assert_eq!(revision(&w), rev);
    assert!(query(&w, "conservada").items[0].result_from_previous_attempt);
    let current = w.extraction_get(IdInput { id: j.id.clone() }).unwrap();
    let queued = w
        .extraction_retry(ExtractionRevision {
            id: j.id.clone(),
            expected_revision: current.revision,
        })
        .unwrap();
    let running = w
        .extraction_run(RunExtraction {
            id: Some(queued.id.clone()),
        })
        .unwrap()
        .job
        .unwrap();
    assert_ne!(running.generation.unwrap().id, j.generation.unwrap().id);
    assert_eq!(revision(&w), rev + 1);
    assert!(query(&w, "conservada").items.is_empty());
    assert_eq!(get(&w)["coverage"]["unindexed_supported_generations"], 1);
    assert_eq!(
        base,
        batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap()
    );
}

#[test]
fn generation_fence_rejects_replacement_even_without_index_revision_change() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"fence", "utf8_text");
    let retry = w
        .extraction_retry(ExtractionRevision {
            id: j.id.clone(),
            expected_revision: j.revision,
        })
        .unwrap();
    let e=api::index_with_hook(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.unwrap().id,"expected_revision":revision(&w)})),&mut |stage|{
        if stage=="prepared" {w.extraction_run(RunExtraction{id:Some(retry.id.clone())})?;}Ok(())
    }).unwrap_err();
    reason(&e, "generation_changed");
    assert_eq!(revision(&w), 1);
    assert!(query(&w, "fence").items.is_empty());
}

#[test]
fn backup_roundtrip_and_batches_keep_current_index_without_original_hash_changes() {
    let (t, mut w) = setup();
    let j = add(&mut w, b"conservada", "utf8_text");
    let before = batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap();
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"batch"}}]}),
    );
    let preview = batches::preview(&w.path, plan.clone(), "test", "preview").unwrap();
    put(&w, &j);
    assert_eq!(
        before,
        batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap()
    );
    let indexed = raw(&w.conn)
        .into_iter()
        .filter(|(n, _)| n.starts_with("extraction_search_"))
        .collect::<Vec<_>>();
    batches::apply(
        &w.path,
        ApplyBatch {
            plan,
            expected: preview.receipt,
        },
        "test",
        "apply",
    )
    .unwrap();
    assert_eq!(
        indexed,
        raw(&w.conn)
            .into_iter()
            .filter(|(n, _)| n.starts_with("extraction_search_"))
            .collect::<Vec<_>>()
    );
    let backup = t.path().join("backup");
    w.backup(&backup).unwrap();
    let restored = t.path().join("restored");
    Workspace::restore(&backup, &restored).unwrap();
    let w2 = Workspace::open(&restored).unwrap();
    assert_eq!(
        serde_json::to_value(query(&w, "conservada")).unwrap(),
        serde_json::to_value(query(&w2, "conservada")).unwrap()
    );
    let c = Connection::open(backup.join(workspace::DB_NAME)).unwrap();
    c.execute("DELETE FROM extraction_search_fts", []).unwrap();
    drop(c);
    let manifest_path = backup.join("manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["sha256"] = json!(workspace::hash(&backup.join(workspace::DB_NAME)).unwrap());
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    assert!(Workspace::restore(&backup, &t.path().join("bad")).is_err());
    assert!(!t.path().join("bad").exists());
}

#[test]
fn future_index_format_is_never_reset_or_reserialized() {
    let (_t, w) = setup();
    w.conn
        .execute_batch("UPDATE extraction_search_control SET version=2,profile='future'")
        .unwrap();
    let before = raw(&w.conn);
    for input in [
        json!({"action":"get"}),
        json!({"action":"reset","expected_revision":1}),
    ] {
        reason(
            &api::index(&w.path, dto(input)).unwrap_err(),
            "unsupported_index_version",
        );
        assert_eq!(before, raw(&w.conn));
    }
}

// Fixtures pobladas por SQL, no evidencia de ejecución del extractor ni del put:
// permiten saturar cuotas independientes sin miles de preparaciones cuadráticas.
pub(crate) fn clone_generation(w: &Workspace, seed: &ExtractionJob) -> ExtractionJob {
    let (job, generation, attachment) = (workspace::id(), workspace::id(), workspace::id());
    let old_generation = &seed.generation.as_ref().unwrap().id;
    let tx = w.conn.unchecked_transaction().unwrap();
    for (table, key, old) in [
        ("attachments", "id", &seed.attachment_id),
        ("extraction_jobs", "id", &seed.id),
        ("extraction_generations", "id", old_generation),
        ("extraction_fragments", "generation_id", old_generation),
        (
            "extraction_pdf_generations",
            "generation_id",
            old_generation,
        ),
        ("extraction_pdf_pages", "generation_id", old_generation),
        ("extraction_pdf_fragments", "generation_id", old_generation),
        (
            "extraction_xlsx_generations",
            "generation_id",
            old_generation,
        ),
        ("extraction_xlsx_chunks", "generation_id", old_generation),
    ] {
        let columns = tx
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
        let select = columns
            .iter()
            .map(|name| match name.as_str() {
                "id" if table == "attachments" => format!("'{attachment}'"),
                "id" if table == "extraction_jobs" => format!("'{job}'"),
                "id" | "generation_id" => format!("'{generation}'"),
                "job_id" => format!("'{job}'"),
                "attachment_id" => format!("'{attachment}'"),
                _ => format!("\"{name}\""),
            })
            .collect::<Vec<_>>()
            .join(",");
        tx.execute(
            &format!("INSERT INTO {table} SELECT {select} FROM {table} WHERE {key}=?1"),
            [old],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    fs::copy(
        w.path.join("attachments").join(&seed.attachment_id),
        w.path.join("attachments").join(attachment),
    )
    .unwrap();
    w.extraction_get(IdInput { id: job }).unwrap()
}

pub(crate) fn fixture_select(w: &Workspace, j: &ExtractionJob, slot: u32) {
    let b = Budget::new(&w.conn).unwrap();
    let data = index::canonical(&w.conn, &j.id, &b).unwrap();
    let tx = w.conn.unchecked_transaction().unwrap();
    tx.execute(
        "INSERT INTO extraction_search_entries VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![
            slot,
            j.id,
            data.generation.id,
            data.fragments.len() as u32,
            data.text_bytes,
            data.hash,
            workspace::now()
        ],
    )
    .unwrap();
    for f in data.fragments {
        tx.execute("INSERT INTO extraction_search_fts(rowid,job_id,generation_id,ordinal,text) VALUES(?1,?2,?3,?4,?5)",params![(slot-1)*512+f.ordinal+1,j.id,data.generation.id,f.ordinal,f.text]).unwrap();
    }
    tx.commit().unwrap();
}

#[test]
fn exact_entry_fragment_and_utf8_byte_caps_are_independent_without_eviction() {
    for cap in ["entries", "fragments", "bytes"] {
        let (_t, mut w) = setup();
        let seed = match cap {
            "entries" => add(&mut w, b"", "utf8_text"),
            "fragments" => {
                let j = add(
                    &mut w,
                    &crate::pdf_tests::source(&[b"a".to_vec()]),
                    "pdf_text",
                );
                let id = &j.generation.as_ref().unwrap().id;
                let text = "é".repeat(512);
                let hash = format!("{:x}", Sha256::digest(text.as_bytes()));
                w.conn
                    .execute(
                        "DELETE FROM extraction_pdf_fragments WHERE generation_id=?1",
                        [id],
                    )
                    .unwrap();
                w.conn
                    .execute(
                        "UPDATE extraction_generations SET fragment_count=512 WHERE id=?1",
                        [id],
                    )
                    .unwrap();
                w.conn.execute("UPDATE extraction_pdf_generations SET extracted_bytes=1024 WHERE generation_id=?1",[id]).unwrap();
                w.conn.execute("UPDATE extraction_pdf_pages SET text_bytes=1024,text_sha256=?2,fragment_count=512 WHERE generation_id=?1",params![id,hash]).unwrap();
                for n in 0..512 {
                    w.conn
                        .execute(
                            "INSERT INTO extraction_pdf_fragments VALUES(?1,?2,?3,'é',1,?4,?5)",
                            params![id, j.id, n, n * 2, n * 2 + 2],
                        )
                        .unwrap();
                }
                w.extraction_get(IdInput { id: j.id }).unwrap()
            }
            _ => add(&mut w, "é".repeat(524288).as_bytes(), "utf8_text"),
        };
        let count = match cap {
            "entries" => 1024,
            "fragments" => 32,
            _ => 8,
        };
        fixture_select(&w, &seed, 1);
        for slot in 2..=count {
            let j = clone_generation(&w, &seed);
            fixture_select(&w, &j, slot);
        }
        let coverage = get(&w);
        assert_eq!(coverage["coverage"]["indexed_generations"], count);
        let extra = add(&mut w, b"x", "utf8_text");
        let before = raw(&w.conn);
        let e=api::index(&w.path,dto(json!({"action":"put","job_id":extra.id,"expected_generation_id":extra.generation.unwrap().id,"expected_revision":revision(&w)}))).unwrap_err();
        reason(
            &e,
            match cap {
                "entries" => "index_entry_limit",
                "fragments" => "index_fragment_limit",
                _ => "index_text_size_limit",
            },
        );
        assert_eq!(before, raw(&w.conn));
        reset(&w);
        assert_eq!(get(&w)["coverage"]["indexed_generations"], 0);
    }
}

#[test]
fn get_coverage_future_raw_configs_and_expired_running_are_pure_and_filtered() {
    let (_t, mut w) = setup();
    let indexed = add(&mut w, b"indexed", "utf8_text");
    put(&w, &indexed);
    let future = add(&mut w, b"future", "utf8_text");
    let config = "{ \"version\":99, \"opaque\": [true, null] }";
    let hash =
        crate::extraction_storage::config_hash(&serde_json::from_str(config).unwrap()).unwrap();
    w.conn
        .execute(
            "UPDATE extraction_jobs SET config=?1,config_hash=?2 WHERE id=?3",
            params![config, hash, future.id],
        )
        .unwrap();
    w.conn
        .execute(
            "UPDATE extraction_generations SET config=?1,config_hash=?2 WHERE id=?3",
            params![config, hash, future.generation.unwrap().id],
        )
        .unwrap();
    let unindexed = add(&mut w, b"unindexed", "utf8_text");
    w.conn
        .execute(
            "UPDATE extraction_jobs SET state='running' WHERE id=?1",
            [&unindexed.id],
        )
        .unwrap();
    w.conn.execute("UPDATE extraction_runtime SET job_id=?1,token=?2,scope=?3,control_revision=?4,lease_until='2000-01-01T00:00:00Z'",params![unindexed.id,workspace::id(),crate::extraction_storage::scope(&w.conn,&w.path).unwrap(),2]).unwrap();
    let no_generation = clone_generation(&w, &indexed);
    w.conn
        .execute(
            "UPDATE extraction_jobs SET generation_id=NULL,state='queued' WHERE id=?1",
            [&no_generation.id],
        )
        .unwrap();
    w.conn
        .execute(
            "DELETE FROM extraction_generations WHERE job_id=?1",
            [&no_generation.id],
        )
        .unwrap();
    let before = raw(&w.conn);
    let page = get(&w);
    let c = &page["coverage"];
    for key in [
        "indexed_generations",
        "unindexed_supported_generations",
        "unsupported_generations",
        "jobs_without_generation",
    ] {
        assert_eq!(c[key], 1, "{key}");
    }
    let filtered = api::index(&w.path, dto(json!({"action":"get","job_id":future.id}))).unwrap();
    let filtered = serde_json::to_value(filtered).unwrap();
    assert_eq!(filtered["coverage"]["unsupported_generations"], 1);
    assert_eq!(filtered["coverage"]["indexed_generations"], 0);
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn corruption_of_canonical_locators_postings_extras_and_bm25_stats_fails_closed() {
    for sql in [
        "UPDATE extraction_fragments SET line_start=2,line_end=2",
        "INSERT INTO extraction_search_fts(rowid,job_id,generation_id,ordinal,text) SELECT rowid+512,job_id,generation_id,ordinal,'extra' FROM extraction_search_fts LIMIT 1",
        "INSERT INTO extraction_search_fts(extraction_search_fts,rowid,job_id,generation_id,ordinal,text) SELECT 'delete',rowid,job_id,generation_id,ordinal,text FROM extraction_search_fts LIMIT 1",
        "UPDATE extraction_search_fts_data SET block=X'010101010101' WHERE id=1",
        "UPDATE extraction_search_fts SET text='extra evidencia';UPDATE extraction_search_fts_content SET c3='canonica evidencia'",
    ] {
        let (_t, mut w) = setup();
        let j = add(&mut w, b"canonica evidencia", "utf8_text");
        put(&w, &j);
        // contentless delete no está permitido en FTS normal: adulterar los
        // postings con su comando 'delete' no es una fixture válida en ese modo.
        if w.conn.execute_batch(sql).is_err() {
            assert!(sql.contains("'delete'"));
            continue;
        }
        let before = raw(&w.conn);
        assert!(
            api::search(&w.path, dto(json!({"query":"evidencia"}))).is_err(),
            "{sql}"
        );
        assert_eq!(before, raw(&w.conn));
    }
}

#[test]
fn revision_overflow_and_unsupported_schema_never_discard_or_wrap() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"overflow", "utf8_text");
    put(&w, &j);
    w.conn
        .execute_batch("UPDATE extraction_search_control SET revision=4294967295")
        .unwrap();
    let before = raw(&w.conn);
    assert!(
        api::index(
            &w.path,
            AttachmentIndexInput::Reset {
                expected_revision: u32::MAX
            }
        )
        .is_err()
    );
    assert_eq!(before, raw(&w.conn));
    w.conn
        .execute_batch("DROP TRIGGER extraction_search_delete")
        .unwrap();
    let before = raw(&w.conn);
    reason(
        &api::index(
            &w.path,
            AttachmentIndexInput::Reset {
                expected_revision: u32::MAX,
            },
        )
        .unwrap_err(),
        "unsupported_index_schema",
    );
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn all_49_v9_input_output_contracts_remain_byte_identical() {
    let mut c = crate::describe();
    for (group, hash) in [
        (
            "inputs",
            "76e36fce2d961998157a6e317bdd8598d7b1d3b7c88a5cad1c8bb60b07a762ec",
        ),
        (
            "outputs",
            "d4a894fd4932450fe26736d2b90a82eb59f7642223875040afe11042d172e09d",
        ),
    ] {
        let values = c[group].as_object_mut().unwrap();
        assert_eq!(values.len(), 53);
        assert!(values.remove("extraction.ocr_regions").is_some());
        assert_eq!(values.len(), 52);
        for op in [
            "extraction.index",
            "extraction.search",
            "extraction.xlsx_cells",
        ] {
            values.remove(op);
        }
        assert_eq!(values.len(), 49);
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(serde_json::to_vec(&c[group]).unwrap())
            ),
            hash
        );
    }
}

#[test]
fn get_and_search_cursors_reject_tampering_alias_restore_and_lookahead_corruption() {
    let (t, mut w) = setup();
    for _ in 0..3 {
        let j = add(&mut w, b"igual", "utf8_text");
        put(&w, &j);
    }
    let page = api::index(&w.path, dto(json!({"action":"get","limit":1}))).unwrap();
    let page = serde_json::to_value(page).unwrap();
    let cursor = page["next_cursor"].as_str().unwrap();
    for (key, value) in [
        ("offset", json!(0)),
        ("offset", json!(1024)),
        ("version", json!(2)),
        ("extra", json!(true)),
        ("revision", json!(0)),
    ] {
        let mut c: Value = serde_json::from_str(cursor).unwrap();
        c[key] = value;
        assert!(
            api::index(
                &w.path,
                dto(json!({"action":"get","limit":1,"cursor":c.to_string()}))
            )
            .is_err()
        );
    }
    let backup = t.path().join("backup-cursors");
    w.backup(&backup).unwrap();
    let restored = t.path().join("restored-cursors");
    Workspace::restore(&backup, &restored).unwrap();
    reason(
        &api::index(
            &restored,
            dto(json!({"action":"get","limit":1,"cursor":cursor})),
        )
        .unwrap_err(),
        "cursor_scope_mismatch",
    );
    let page = api::search(&w.path, dto(json!({"query":"igual","limit":1}))).unwrap();
    let second = api::search(
        &w.path,
        dto(json!({"query":"igual","limit":1,"cursor":page.next_cursor})),
    )
    .unwrap();
    fs::remove_file(
        w.path
            .join("attachments")
            .join(&second.items[0].source.attachment_id),
    )
    .unwrap();
    reason(
        &api::search(&w.path, dto(json!({"query":"igual","limit":1}))).unwrap_err(),
        "input_stale",
    );
}

#[test]
#[ignore = "Benchmark release explícito, corpus sintético externo, 5 warmups/20 muestras"]
fn release_attachment_index_resources() {
    use std::time::Instant;
    let (_t, mut w) = setup();
    let text = "canonica evidencia cafe contrato alpha beta gamma delta\n".repeat(25000);
    let bytes = &text.as_bytes()[..1_048_576];
    let seed = add(&mut w, bytes, "utf8_text");
    fixture_select(&w, &seed, 1);
    let mut jobs = vec![seed];
    for slot in 2..=8 {
        let j = clone_generation(&w, &jobs[0]);
        fixture_select(&w, &j, slot);
        jobs.push(j);
    }
    let mut results = Vec::new();
    let cases = vec![
        ("query-hit", json!({"query":"evidencia","limit":64})),
        ("query-zero", json!({"query":"ausente"})),
        (
            "query-filter",
            json!({"query":"café contrato","job_id":jobs[0].id,"limit":64}),
        ),
        ("query-page", json!({"query":"evidencia","limit":1})),
    ];
    for (name, input) in cases {
        for round in 0..25 {
            let start = Instant::now();
            let page = api::search(&w.path, dto(input.clone()));
            let row = json!({"case":name,"round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"ok":page.is_ok(),"error":page.as_ref().err().map(|e|serde_json::to_value(e).unwrap())});
            println!("ATTACHMENT_BENCH {row}");
            results.push(row);
            if let Ok(page) = page
                && name == "query-page"
                && let Some(cursor) = page.next_cursor
            {
                let start = Instant::now();
                let page = api::search(
                    &w.path,
                    dto(json!({"query":"evidencia","limit":1,"cursor":cursor})),
                );
                let row = json!({"case":"query-page-next","round":round,"warmup":round<5,"wall_ms":start.elapsed().as_secs_f64()*1000.,"ok":page.is_ok()});
                println!("ATTACHMENT_BENCH {row}");
                results.push(row);
            }
        }
    }
    for name in ["put", "no-op", "reset", "invalidation"] {
        for round in 0..25 {
            if name == "put" {
                w.conn
                    .execute(
                        "DELETE FROM extraction_search_entries WHERE job_id=?1",
                        [&jobs[7].id],
                    )
                    .unwrap();
            }
            let start = Instant::now();
            let mut locked = None;
            let mut lock_ms = None;
            let mut editor = None;
            let root = w.path.clone();
            let mut hook = |stage: &str| {
                if matches!(stage, "locked" | "publish_locked") {
                    locked = Some(Instant::now());
                    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
                    let other = barrier.clone();
                    let db = root.join(workspace::DB_NAME);
                    editor = Some(std::thread::spawn(move || {
                        let c = Connection::open(db).unwrap();
                        c.busy_timeout(std::time::Duration::from_secs(2)).unwrap();
                        let start = Instant::now();
                        other.wait();
                        c.execute_batch("BEGIN IMMEDIATE;ROLLBACK").unwrap();
                        start.elapsed().as_secs_f64() * 1000.
                    }));
                    barrier.wait();
                }
                if stage == "committed" {
                    lock_ms = locked.map(|s| s.elapsed().as_secs_f64() * 1000.);
                }
            };
            let ok = if name == "invalidation" {
                let current = w
                    .extraction_get(IdInput {
                        id: jobs[0].id.clone(),
                    })
                    .unwrap();
                w.extraction_retry(ExtractionRevision {
                    id: current.id.clone(),
                    expected_revision: current.revision,
                })
                .unwrap();
                let out = w.extraction_run_impl(
                    RunExtraction {
                        id: Some(current.id),
                    },
                    Some(&mut hook),
                    None,
                );
                if let Ok(out) = &out {
                    jobs[0] = out.job.clone().unwrap();
                }
                out.is_ok()
            } else {
                let input = if name == "reset" {
                    AttachmentIndexInput::Reset {
                        expected_revision: revision(&w),
                    }
                } else {
                    dto(
                        json!({"action":"put","job_id":jobs[7].id,"expected_generation_id":jobs[7].generation.as_ref().unwrap().id,"expected_revision":revision(&w)}),
                    )
                };
                api::index_with_hook(&w.path, input, &mut |s| {
                    hook(s);
                    Ok(())
                })
                .is_ok()
            };
            let wall_ms = start.elapsed().as_secs_f64() * 1000.;
            let editor_wait_ms = editor.map(|h| h.join().unwrap());
            let row = json!({"case":name,"round":round,"warmup":round<5,"wall_ms":wall_ms,"lock_ms":lock_ms,"editor_wait_ms":editor_wait_ms,"ok":ok});
            println!("ATTACHMENT_BENCH {row}");
            results.push(row);
            if name == "reset" {
                for (n, j) in jobs.iter().enumerate() {
                    fixture_select(&w, j, n as u32 + 1);
                }
            }
            if name == "invalidation" {
                fixture_select(&w, &jobs[0], 1);
            }
        }
    }
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) },
        0
    );
    let rss = unsafe { usage.assume_init() }.ru_maxrss;
    let b = Budget::new(&w.conn).unwrap();
    let verified = index::verify(&w.conn, &b).unwrap();
    let temp_pages: u32 = w
        .conn
        .query_row("PRAGMA temp.page_count", [], |r| r.get(0))
        .unwrap();
    let temp_page_size: u32 = w
        .conn
        .query_row("PRAGMA temp.page_size", [], |r| r.get(0))
        .unwrap();
    drop(verified);
    drop(b);
    let report = json!({"corpus":"8x1MiB UTF-8 repeated fixed line v1","source_sha256":format!("{:x}",Sha256::digest(bytes)),"text_bytes":8388608,"fragments":2048,"warmups":5,"samples":20,"results":results,"rss_self_high_water_native_units":rss,"temp_logical_bytes":u64::from(temp_pages)*u64::from(temp_page_size),"sqlite_bytes":fs::metadata(w.path.join(workspace::DB_NAME)).unwrap().len(),"wal_bytes":fs::metadata(w.path.join(format!("{}-wal",workspace::DB_NAME))).map(|m|m.len()).unwrap_or(0)});
    if let Ok(path) = std::env::var("NOTEKA_ATTACHMENT_BENCH_REPORT") {
        fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
}

#[test]
#[ignore = "Diagnóstico release comparativo de algoritmos; no es benchmark de aceptación"]
fn attachment_posting_diagnostics() {
    use std::time::Instant;
    let (_t, mut w) = setup();
    let text = "canonica evidencia cafe contrato alpha beta gamma delta\n".repeat(25000);
    let seed = add(&mut w, &text.as_bytes()[..1_048_576], "utf8_text");
    fixture_select(&w, &seed, 1);
    for slot in 2..=8 {
        let j = clone_generation(&w, &seed);
        fixture_select(&w, &j, slot);
    }
    let b = Budget::new(&w.conn).unwrap();
    let start = Instant::now();
    let verified = index::verify(&w.conn, &b).unwrap();
    println!(
        "POSTING_DIAGNOSTIC verify_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    drop(b);
    let b = Budget::new(&w.conn).unwrap();
    let start = Instant::now();
    let same = index::same_posting_sequence(&w.conn, &b).unwrap();
    println!(
        "POSTING_DIAGNOSTIC sequence_ms={} same={same}",
        start.elapsed().as_secs_f64() * 1000.
    );
    let start = Instant::now();
    for entry in &verified.entries {
        index::canonical(&w.conn, &entry.data.source.job_id, &b).unwrap();
    }
    println!(
        "POSTING_DIAGNOSTIC canonical_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    drop(b);
    let start = Instant::now();
    let copies=w.conn.prepare("SELECT 1 FROM (SELECT * FROM (SELECT rowid,job_id,generation_id,ordinal,text FROM main.extraction_search_fts EXCEPT SELECT rowid,job_id,generation_id,ordinal,text FROM temp.attachment_expected_fts) UNION ALL SELECT * FROM (SELECT rowid,job_id,generation_id,ordinal,text FROM temp.attachment_expected_fts EXCEPT SELECT rowid,job_id,generation_id,ordinal,text FROM main.extraction_search_fts)) LIMIT 1").unwrap().exists([]).unwrap();
    assert!(!copies);
    println!(
        "POSTING_DIAGNOSTIC copies_except_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    let start = Instant::now();
    w.conn.execute_batch("CREATE VIRTUAL TABLE temp.profile_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);INSERT INTO temp.profile_fts(rowid,job_id,generation_id,ordinal,text) SELECT rowid,job_id,generation_id,ordinal,text FROM temp.attachment_expected_fts").unwrap();
    println!(
        "POSTING_DIAGNOSTIC build_fts_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    w.conn.execute_batch("DROP TABLE temp.profile_fts").unwrap();
    for (name, sql) in [
        (
            "count_one_except",
            "SELECT ((SELECT count(*) FROM temp.attachment_actual_vocab)!=(SELECT count(*) FROM temp.attachment_expected_vocab)) OR EXISTS(SELECT term,doc,col,offset FROM temp.attachment_expected_vocab EXCEPT SELECT term,doc,col,offset FROM temp.attachment_actual_vocab)",
        ),
        (
            "group_union",
            "SELECT EXISTS(SELECT term,doc,col,offset FROM (SELECT term,doc,col,offset,1 AS side FROM temp.attachment_actual_vocab UNION ALL SELECT term,doc,col,offset,-1 FROM temp.attachment_expected_vocab) GROUP BY term,doc,col,offset HAVING sum(side)!=0)",
        ),
    ] {
        let start = Instant::now();
        let diff: bool = w.conn.query_row(sql, [], |r| r.get(0)).unwrap();
        assert!(!diff);
        println!(
            "POSTING_DIAGNOSTIC {name}_ms={}",
            start.elapsed().as_secs_f64() * 1000.
        );
    }
    let start = Instant::now();
    let mut a=w.conn.prepare("SELECT term,doc,col,offset FROM temp.attachment_actual_vocab ORDER BY term,doc,col,offset").unwrap();
    let mut e=w.conn.prepare("SELECT term,doc,col,offset FROM temp.attachment_expected_vocab ORDER BY term,doc,col,offset").unwrap();
    let mut a = a.query([]).unwrap();
    let mut e = e.query([]).unwrap();
    let mut count = 0u32;
    loop {
        match (a.next().unwrap(), e.next().unwrap()) {
            (None, None) => break,
            (Some(a), Some(e)) => {
                for n in 0..4 {
                    assert_eq!(a.get_ref(n).unwrap(), e.get_ref(n).unwrap());
                }
                count += 1;
            }
            _ => panic!("postings diferentes"),
        }
    }
    println!(
        "POSTING_DIAGNOSTIC ordered_stream_ms={} instances={count}",
        start.elapsed().as_secs_f64() * 1000.
    );
    drop(verified);
    let (c, _) = index::open(&w.path, false).unwrap();
    let b = Budget::new(&c).unwrap();
    let tx = c.unchecked_transaction().unwrap();
    let start = Instant::now();
    let verified = index::verify(&tx, &b).unwrap();
    println!(
        "POSTING_DIAGNOSTIC operation_connection_verify_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    let start = Instant::now();
    assert!(index::same_posting_sequence(&tx, &b).unwrap());
    println!(
        "POSTING_DIAGNOSTIC operation_connection_sequence_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    let fragments = verified
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
        index::same_repeated_postings(&tx, &b, &fragments).unwrap(),
        Some(true)
    );
    println!(
        "POSTING_DIAGNOSTIC operation_repeated_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    let start = Instant::now();
    for e in &verified.entries {
        assert_eq!(
            index::content_hash(&e.data.source, &e.data.generation, None, &e.data.fragments)
                .unwrap(),
            e.data.hash
        );
    }
    println!(
        "POSTING_DIAGNOSTIC content_hash_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    let start = Instant::now();
    for e in &verified.entries {
        let mut hash = Sha256::new();
        for f in &e.data.fragments {
            hash.update(f.text.as_bytes());
        }
        assert_eq!(
            format!("{:x}", hash.finalize()),
            e.data.source.source_sha256
        );
    }
    println!(
        "POSTING_DIAGNOSTIC source_sha_ms={}",
        start.elapsed().as_secs_f64() * 1000.
    );
    drop(b);
    drop(verified);
}

#[test]
fn failed_utf8_and_pdf_publications_roll_back_selection_trigger_and_postings() {
    for extractor in ["utf8_text", "pdf_text"] {
        for table in [
            "extraction_generations",
            if extractor == "pdf_text" {
                "extraction_pdf_fragments"
            } else {
                "extraction_fragments"
            },
        ] {
            let (_t, mut w) = setup();
            let source = if extractor == "pdf_text" {
                crate::pdf_tests::source(&[b"prior evidencia".to_vec()])
            } else {
                b"prior evidencia".to_vec()
            };
            let j = add(&mut w, &source, extractor);
            put(&w, &j);
            let rev = revision(&w);
            let prior = raw(&w.conn)
                .into_iter()
                .filter(|(n, _)| n.starts_with("extraction_search_"))
                .collect::<Vec<_>>();
            w.conn.execute_batch(&format!("CREATE TRIGGER fail_publication BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT,'fixture failure');END")).unwrap();
            w.extraction_retry(ExtractionRevision {
                id: j.id.clone(),
                expected_revision: j.revision,
            })
            .unwrap();
            assert!(w.extraction_run(RunExtraction { id: Some(j.id) }).is_err());
            assert_eq!(revision(&w), rev);
            assert_eq!(
                prior,
                raw(&w.conn)
                    .into_iter()
                    .filter(|(n, _)| n.starts_with("extraction_search_"))
                    .collect::<Vec<_>>()
            );
            let page = query(&w, "evidencia");
            assert_eq!(page.items.len(), 1);
            assert!(page.items[0].result_from_previous_attempt);
        }
    }
}

#[test]
fn borrowed_content_hash_preserves_v1_for_both_locators_escapes_and_nested_metadata() {
    for (extractor, source) in [
        ("utf8_text", b"a \"quote\"\r\ncafe caf\xc3\xa9".to_vec()),
        (
            "pdf_text",
            crate::pdf_tests::source(&[b"PDF evidencia".to_vec()]),
        ),
        ("pdf_text", crate::pdf_tests::source(&[vec![]])),
    ] {
        let (_t, mut w) = setup();
        let job = add(&mut w, &source, extractor);
        let b = Budget::new(&w.conn).unwrap();
        let mut data = index::canonical(&w.conn, &job.id, &b).unwrap();
        let manifest = if extractor == "pdf_text" {
            Some(
                crate::pdf_storage::load(
                    &w.conn,
                    &data.generation.id,
                    &job.id,
                    data.generation.fragment_count,
                )
                .unwrap()
                .manifest,
            )
        } else {
            None
        };
        for variant in 0..3 {
            if variant == 1 {
                data.generation.config = json!({"z":[null,true,false,3,-7,0.25,{"z":"last","a":"first"}],"a":{"future":"value"}});
            }
            if variant == 2 {
                for f in &mut data.fragments {
                    f.text = "\u{0}\u{1}\t\r\n\\\" caf\u{e9} 😀".repeat(100);
                }
            }
            let legacy=batch_storage::json_hash("noteka.extraction.index.content.v1",&json!({"source":data.source,"generation":data.generation,"manifest":manifest,"fragments":data.fragments})).unwrap();
            let borrowed = index::content_hash(
                &data.source,
                &data.generation,
                manifest.as_ref(),
                &data.fragments,
            )
            .unwrap();
            assert_eq!(borrowed, legacy, "{extractor} / {variant}");
        }
    }
}

#[test]
fn repeated_fragment_verification_rejects_hidden_postings_and_keeps_source_checks() {
    for corrupt in [false, true] {
        let (_t, mut w) = setup();
        let seed = add(&mut w, b"cafe evidencia a a", "utf8_text");
        let mut jobs = vec![seed];
        for _ in 1..8 {
            jobs.push(clone_generation(&w, &jobs[0]));
        }
        for (n, j) in jobs.iter().enumerate() {
            fixture_select(&w, j, n as u32 + 1);
        }
        assert_eq!(query(&w, "evidencia").items.len(), 8);
        if corrupt {
            let original: String = w
                .conn
                .query_row(
                    "SELECT text FROM extraction_search_fts WHERE rowid=1",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            // Texto copiado sigue idéntico al canónico; sólo postings distintos.
            w.conn
                .execute(
                    "UPDATE extraction_search_fts SET text='different' WHERE rowid=1",
                    [],
                )
                .unwrap();
            w.conn
                .execute(
                    "UPDATE extraction_search_fts_content SET c3=?1 WHERE id=1",
                    [original],
                )
                .unwrap();
            let e = api::search(&w.path, dto(json!({"query":"absentword"}))).unwrap_err();
            reason(&e, "invalid_attachment_index");
        } else {
            fs::write(
                w.path.join("attachments").join(&jobs[0].attachment_id),
                b"fake evidencia a a",
            )
            .unwrap();
            let e = api::search(&w.path, dto(json!({"query":"evidencia"}))).unwrap_err();
            reason(&e, "input_stale");
        }
    }
}

#[test]
fn put_deadline_and_writer_contention_do_not_confirm_or_retry() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"deadline", "utf8_text");
    let before = raw(&w.conn);
    let request = json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.as_ref().unwrap().id,"expected_revision":1});
    let e = api::index_with_hook(&w.path, dto(request.clone()), &mut |s| {
        if s == "before_commit" {
            std::thread::sleep(std::time::Duration::from_millis(5050));
        }
        Ok(())
    })
    .unwrap_err();
    reason(&e, "execution_deadline");
    assert_eq!(before, raw(&w.conn));
    let other = Connection::open(w.path.join(workspace::DB_NAME)).unwrap();
    other.execute_batch("BEGIN IMMEDIATE").unwrap();
    let start = std::time::Instant::now();
    let e = api::index(&w.path, dto(request)).unwrap_err();
    assert!(matches!(e.code, ErrorCode::WorkspaceBusy));
    assert!(start.elapsed() < std::time::Duration::from_secs(4));
    other.execute_batch("ROLLBACK").unwrap();
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn full_snapshot_cap_denies_put_but_reset_preserves_originals_without_vacuum() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"size", "utf8_text");
    put(&w, &j);
    w.conn.execute_batch("CREATE TABLE original_extension(data BLOB);INSERT INTO original_extension VALUES(zeroblob(67108864));").unwrap();
    let other = clone_generation(&w, &j);
    let revision = revision(&w);
    let count: u32 = w
        .conn
        .pragma_query_value(None, "page_count", |r| r.get(0))
        .unwrap();
    let e=api::index(&w.path,dto(json!({"action":"put","job_id":other.id,"expected_generation_id":other.generation.unwrap().id,"expected_revision":revision}))).unwrap_err();
    reason(&e, "snapshot_size_limit");
    assert_eq!(get(&w)["coverage"]["indexed_generations"], 1);
    reset(&w);
    assert_eq!(get(&w)["coverage"]["indexed_generations"], 0);
    assert!(
        w.conn
            .pragma_query_value(None, "page_count", |r| r.get::<_, u32>(0))
            .unwrap()
            >= count
    );
    assert_eq!(
        w.conn
            .query_row("SELECT length(data) FROM original_extension", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        67108864
    );
}

#[test]
fn oversized_cursor_response_and_reset_between_preview_apply_are_explicit() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"receipt", "utf8_text");
    put(&w, &j);
    for operation in ["extraction.index", "extraction.search"] {
        let input = if operation == "extraction.index" {
            json!({"action":"get","cursor":"x".repeat(2049)})
        } else {
            json!({"query":"x","cursor":"x".repeat(2049)})
        };
        let r = crate::execute_json(
            json!({"workspace":w.path,"command":{"operation":operation,"input":input}}),
            "test",
        );
        assert!(!r.ok);
        reason(&r.error.unwrap(), "cursor_size_limit");
    }
    reason(
        &crate::extraction_storage::response(&"x".repeat(2 * 1024 * 1024)).unwrap_err(),
        "response_size_limit",
    );
    let plan: BatchPlan = dto(
        json!({"version":1,"batch_id":workspace::id(),"steps":[{"key":"n","operation":"entity.create","input":{"kind":"note","title":"unchanged receipt"}}]}),
    );
    let preview = batches::preview(&w.path, plan.clone(), "test", "preview").unwrap();
    let base = batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap();
    reset(&w);
    assert_eq!(
        base,
        batch_storage::base_hash(&w.conn, &Budget::new(&w.conn).unwrap()).unwrap()
    );
    batches::apply(
        &w.path,
        ApplyBatch {
            plan,
            expected: preview.receipt,
        },
        "test",
        "apply",
    )
    .unwrap();
    assert_eq!(get(&w)["coverage"]["indexed_generations"], 0);
}

#[test]
fn shared_archived_and_removed_references_never_become_index_owners_or_filters() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"evidencia compartida", "utf8_text");
    put(&w, &j);
    let rev = revision(&w);
    let mut notes = Vec::new();
    for n in 0..2 {
        let note=w.entity_create(dto(json!({"kind":"note","title":format!("reference {n}"),"document":{"schema_version":2,"blocks":[{"id":"file","type":"file","content":[],"attrs":{"attachment_id":j.attachment_id,"name":"source.txt"}}]}})),"test","note").unwrap();
        notes.push(note);
    }
    w.entity_archive(
        dto(json!({"id":notes[0].id,"expected_revision":1})),
        true,
        "test",
        "archive",
    )
    .unwrap();
    assert_eq!(query(&w, "compartida").items.len(), 1);
    assert_eq!(revision(&w), rev);
    let p = serde_json::to_value(query(&w, "compartida")).unwrap();
    assert!(p["items"][0].get("entity_id").is_none());
    assert!(p["items"][0]["source"].get("entity_id").is_none());
    let before = query(&w, "compartida").items[0].score;
    w.search_rebuild().unwrap();
    assert_eq!(query(&w, "compartida").items[0].score, before);
    for note in notes {
        let current = w.entity_get(&IdInput { id: note.id }).unwrap();
        w.entity_update(dto(json!({"id":current.id,"expected_revision":current.revision,"document":{"schema_version":2,"blocks":[]}})),"test","remove-ref").unwrap();
    }
    assert_eq!(query(&w, "compartida").items.len(), 1);
    assert_eq!(revision(&w), rev);
    for (key, value) in [
        ("archived", json!(false)),
        ("kind", json!("note")),
        ("related_id", json!(workspace::id())),
    ] {
        let mut input = json!({"query":"evidencia"});
        input[key] = value;
        assert!(serde_json::from_value::<AttachmentSearchInput>(input).is_err());
    }
}

#[test]
fn get_and_search_cursors_follow_put_invalidation_noop_and_dynamic_job_state() {
    for action in ["noop", "put", "invalidation", "retry", "pause"] {
        let (_t, mut w) = setup();
        let a = add(&mut w, b"igual literal", "utf8_text");
        let b = add(&mut w, b"igual literal", "utf8_text");
        put(&w, &a);
        put(&w, &b);
        let search = api::search(&w.path, dto(json!({"query":"igual","limit":1}))).unwrap();
        let listing = serde_json::to_value(
            api::index(&w.path, dto(json!({"action":"get","limit":1}))).unwrap(),
        )
        .unwrap();
        let before = revision(&w);
        match action {
            "noop" => {
                assert_eq!(put(&w, &a)["changed"], false);
            }
            "put" => {
                let c = add(&mut w, b"igual literal", "utf8_text");
                put(&w, &c);
            }
            "invalidation" | "retry" => {
                w.extraction_retry(ExtractionRevision {
                    id: a.id.clone(),
                    expected_revision: a.revision,
                })
                .unwrap();
                if action == "invalidation" {
                    w.extraction_run(RunExtraction {
                        id: Some(a.id.clone()),
                    })
                    .unwrap();
                }
            }
            _ => {
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
        let search = api::search(
            &w.path,
            dto(json!({"query":"igual","limit":1,"cursor":search.next_cursor})),
        );
        let listing = api::index(
            &w.path,
            dto(json!({"action":"get","limit":1,"cursor":listing["next_cursor"]})),
        );
        if matches!(action, "put" | "invalidation") {
            assert_eq!(revision(&w), before + 1);
            reason(&search.unwrap_err(), "index_revision_changed");
            reason(&listing.unwrap_err(), "index_revision_changed");
        } else {
            assert_eq!(revision(&w), before);
            assert!(search.is_ok() && listing.is_ok());
        }
    }
}

#[test]
fn terms_on_separate_utf8_fragments_and_pdf_pages_never_form_a_hit() {
    let (_t, mut w) = setup();
    let text = format!("alpha {} beta", "x ".repeat(2045));
    let a = add(&mut w, text.as_bytes(), "utf8_text");
    let pdf = add(
        &mut w,
        &crate::pdf_tests::source(&[b"alpha".to_vec(), b"beta".to_vec()]),
        "pdf_text",
    );
    for j in [a, pdf] {
        put(&w, &j);
        for term in ["alpha", "beta"] {
            assert_eq!(
                api::search(&w.path, dto(json!({"query":term,"job_id":j.id})))
                    .unwrap()
                    .items
                    .len(),
                1
            );
        }
        assert!(
            api::search(&w.path, dto(json!({"query":"alpha beta","job_id":j.id})))
                .unwrap()
                .items
                .is_empty()
        );
    }
}

#[test]
fn changed_metadata_and_future_generation_configs_fail_without_repair_or_rewrite() {
    for sql in [
        "UPDATE attachments SET sha256=printf('%064d',0)",
        "UPDATE attachments SET bytes=bytes+1",
        "UPDATE attachments SET name='unknown.exe'",
    ] {
        let (_t, mut w) = setup();
        let j = add(&mut w, b"evidencia", "utf8_text");
        put(&w, &j);
        w.conn.execute_batch(sql).unwrap();
        let before = raw(&w.conn);
        reason(
            &api::search(&w.path, dto(json!({"query":"ausente"}))).unwrap_err(),
            "invalid_attachment_index",
        );
        assert!(api::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.unwrap().id,"expected_revision":revision(&w)}))).is_err());
        assert_eq!(before, raw(&w.conn));
    }
    let (_t, mut w) = setup();
    let j = add(&mut w, b"future", "utf8_text");
    let raw_config = "{ \"version\":99, \"opaque\":true }";
    let hash =
        crate::extraction_storage::config_hash(&serde_json::from_str(raw_config).unwrap()).unwrap();
    for table in ["extraction_jobs", "extraction_generations"] {
        w.conn
            .execute(
                &format!("UPDATE {table} SET config=?1,config_hash=?2"),
                params![raw_config, hash],
            )
            .unwrap();
    }
    let before = raw(&w.conn);
    reason(&api::index(&w.path,dto(json!({"action":"put","job_id":j.id,"expected_generation_id":j.generation.unwrap().id,"expected_revision":1}))).unwrap_err(),"unsupported_generation_config");
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn additive_output_dtos_and_schemas_reject_unknown_fields_too() {
    let (_t, mut w) = setup();
    let j = add(&mut w, b"strict", "utf8_text");
    let published = put(&w, &j);
    serde_json::from_value::<AttachmentIndexResult>(published.clone()).unwrap();
    let mut bad = published;
    bad["entry"]["source"]["provider"] = json!("unexpected");
    assert!(serde_json::from_value::<AttachmentIndexResult>(bad).is_err());
    let page = serde_json::to_value(query(&w, "strict")).unwrap();
    serde_json::from_value::<AttachmentSearchPage>(page.clone()).unwrap();
    for pointer in [
        "",
        "/coverage",
        "/items/0",
        "/items/0/source",
        "/items/0/locator",
    ] {
        let mut bad = page.clone();
        bad.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), json!(true));
        assert!(
            serde_json::from_value::<AttachmentSearchPage>(bad).is_err(),
            "{pointer}"
        );
    }
    serde_json::from_value::<AttachmentIndexResult>(get(&w)).unwrap();
    let schema = schemars::schema_for!(AttachmentSearchPage);
    let schema = serde_json::to_value(schema).unwrap();
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        serde_json::to_value(schemars::schema_for!(AttachmentSearchFragment)).unwrap()["additionalProperties"],
        false
    );
    assert_eq!(
        serde_json::to_value(schemars::schema_for!(AttachmentIndexPage)).unwrap()["additionalProperties"],
        false
    );
    for name in [
        "AttachmentIndexEntry",
        "AttachmentIndexCoverage",
        "AttachmentSearchHit",
        "IndexedSource",
    ] {
        let catalog = crate::describe();
        assert_eq!(
            catalog["contract_schema"]["$defs"][name]["additionalProperties"], false,
            "{name}"
        );
    }
}
