//! Diferencial contra el cuerpo previo; fixtures sintéticas, no mediciones OCR.
use crate::{
    attachment_search_storage as index,
    attachment_search_tests::{add, fixture_select},
    batch_storage::Budget,
    error::{CoreError, ErrorCode},
    extraction_tests::raw,
    workspace::Workspace,
};
use rusqlite::params;
use serde_json::{Value, json};
use std::time::Duration;

fn fixture(mode: &str) -> (tempfile::TempDir, Workspace) {
    let t = tempfile::tempdir().unwrap();
    let mut w = Workspace::create(&t.path().join("w"), Some("UTC".into())).unwrap();
    let count = match mode {
        "empty" => 0,
        "unique" => 3,
        "mixed" => 16,
        _ => 8,
    };
    for n in 0..count {
        let text = if mode == "zero_tokens" {
            " \n\t".into()
        } else if mode == "unique" || mode == "mixed" && n >= count - 2 {
            format!("distinto{n} café Ñ 🚀 texto propio\n")
        } else {
            "canonica evidencia café Ñ 🚀\n".into()
        };
        let j = add(&mut w, text.as_bytes(), "utf8_text");
        fixture_select(&w, &j, n * 3 + 1);
    }
    (t, w)
}
fn verify(w: &Workspace, optimized: bool) -> Result<(Value, bool), CoreError> {
    let (c, _) = index::open(&w.path, false)?;
    let b = Budget::new(&c)?;
    let tx = c.unchecked_transaction()?;
    let v = if optimized {
        index::verify(&tx, &b)?
    } else {
        index::verify_reference(&tx, &b)?
    };
    let rows = v.entries.iter().map(|s| json!({"slot":s.slot,"entry":s.entry,"fragments":s.data.fragments,"hash":s.data.hash,"previous":s.data.previous})).collect::<Vec<_>>();
    let materialized = c
        .prepare("SELECT 1 FROM temp.sqlite_schema WHERE name='attachment_expected_fts'")?
        .exists([])?;
    Ok((json!(rows), materialized))
}
fn same_rejection(w: &Workspace) {
    let before = raw(&w.conn);
    let old = verify(w, false).unwrap_err();
    let new = verify(w, true).unwrap_err();
    assert_eq!(json!(old.code), json!(new.code));
    assert_eq!(old.details["reason"], new.details["reason"]);
    assert_eq!(
        old.details["current_revision"],
        new.details["current_revision"]
    );
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn exact_validation_differential_valid_corpus_and_lazy_materialization() {
    for mode in ["empty", "unique", "repeated", "mixed", "zero_tokens"] {
        let (_t, w) = fixture(mode);
        let before = raw(&w.conn);
        let old = verify(&w, false).unwrap();
        let new = verify(&w, true).unwrap();
        assert_eq!(old.0, new.0, "{mode}");
        assert!(old.1);
        assert_eq!(new.1, matches!(mode, "empty" | "unique"), "{mode}");
        assert_eq!(before, raw(&w.conn));
    }
}

#[test]
fn exact_validation_differential_all_copy_types_and_metadata() {
    for mode in ["unique", "repeated"] {
        for sql in [
            "DELETE FROM extraction_search_fts WHERE rowid=(SELECT min(rowid) FROM extraction_search_fts)",
            "UPDATE extraction_search_fts_content SET id=id+2 WHERE id=(SELECT min(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c0='otro' WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c1='otro' WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c2='0' WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c2=0.0 WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c2=X'30' WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c3=CAST(c3 AS BLOB) WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c0=CAST(c0 AS BLOB) WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "UPDATE extraction_search_fts_content SET c3='otro texto' WHERE id=(SELECT max(id) FROM extraction_search_fts_content)",
            "INSERT INTO extraction_search_fts(rowid,job_id,generation_id,ordinal,text) SELECT rowid+512,job_id,generation_id,ordinal,text FROM extraction_search_fts LIMIT 1",
            "UPDATE extraction_search_entries SET content_hash=printf('%064d',0)",
            "UPDATE extraction_search_entries SET indexed_at='fecha-invalida'",
            "UPDATE extraction_fragments SET line_end=line_end+1",
        ] {
            let (_t, w) = fixture(mode);
            assert!(verify(&w, true).is_ok());
            w.conn.execute_batch(sql).unwrap();
            same_rejection(&w);
        }
    }
}

#[test]
fn exact_validation_differential_hidden_postings_and_statistics() {
    for mode in ["unique", "repeated"] {
        for corruption in 0..4 {
            let (_t, w) = fixture(mode);
            let (rowid, text): (i64, String) = w
                .conn
                .query_row(
                    "SELECT rowid,text FROM extraction_search_fts ORDER BY rowid DESC LIMIT 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            match corruption {
                0 => {
                    w.conn.execute("UPDATE extraction_search_fts SET text='otro posting oculto' WHERE rowid=?1", [rowid]).unwrap();
                    w.conn
                        .execute(
                            "UPDATE extraction_search_fts_content SET c3=?1 WHERE id=?2",
                            params![text, rowid],
                        )
                        .unwrap();
                }
                1 => {
                    w.conn.execute_batch("UPDATE extraction_search_fts_data SET block=X'010101010101' WHERE id=1").unwrap();
                }
                2 => {
                    w.conn
                        .execute(
                            "UPDATE extraction_search_fts_docsize SET sz=X'00000000' WHERE id=?1",
                            [rowid],
                        )
                        .unwrap();
                }
                _ => {
                    w.conn
                        .execute_batch("DELETE FROM extraction_search_fts_data WHERE id>10")
                        .unwrap();
                }
            }
            if matches!(corruption, 1 | 2) {
                let before = raw(&w.conn);
                // El oráculo previo podía admitir estadísticas corruptas en
                // verify; la API search las confrontaba al calcular candidatos.
                let _ = verify(&w, false);
                assert!(verify(&w, true).is_err());
                let input = serde_json::from_value(json!({"query":"café","limit":64})).unwrap();
                assert!(crate::attachment_search::search(&w.path, input).is_err());
                assert_eq!(before, raw(&w.conn));
            } else {
                same_rejection(&w);
            }
        }
    }
}

#[test]
fn exact_validation_preserves_expired_budget() {
    let (_t, w) = fixture("repeated");
    let before = raw(&w.conn);
    for optimized in [false, true] {
        let (c, _) = index::open(&w.path, false).unwrap();
        let b = Budget::with_duration(&c, Duration::ZERO).unwrap();
        let result = if optimized {
            index::verify(&c, &b)
        } else {
            index::verify_reference(&c, &b)
        };
        assert!(matches!(
            result.err().unwrap().code,
            ErrorCode::WorkspaceBusy
        ));
    }
    assert_eq!(before, raw(&w.conn));
}

#[test]
fn exact_validation_canonical_absence_matches_full_query_and_expires_with_snapshot() {
    let (_t, w) = fixture("mixed");
    let (c, _) = index::open(&w.path, false).unwrap();
    let b = Budget::new(&c).unwrap();
    let tx = c.unchecked_transaction().unwrap();
    let v = index::verify(&tx, &b).unwrap();
    assert_eq!(
        index::canonical_has_match(&tx, "\"ausente\"").unwrap(),
        Some(false)
    );
    assert_eq!(
        index::canonical_has_match(&tx, "\"evidencia\"").unwrap(),
        Some(true)
    );
    index::ensure_expected(&tx, &b, &v.entries).unwrap();
    for query in [
        "evidencia",
        "ausente",
        "café Ñ",
        "OR",
        "\"",
        "café-canonica",
        "🚀",
        "distinto14",
    ] {
        let expression = query
            .split_whitespace()
            .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");
        let full = tx.prepare("SELECT 1 FROM temp.attachment_expected_fts WHERE attachment_expected_fts MATCH ?1 LIMIT 1").unwrap().exists([&expression]).unwrap();
        assert_eq!(
            index::canonical_has_match(&tx, &expression).unwrap(),
            Some(full),
            "{query}"
        );
    }
    drop(v);
    assert_eq!(
        index::canonical_has_match(&tx, "\"evidencia\"").unwrap(),
        None
    );
    let v = index::verify(&tx, &b).unwrap();
    assert_eq!(
        index::canonical_has_match(&tx, "\"evidencia\"").unwrap(),
        Some(true)
    );
    drop(v);
    assert_eq!(
        index::canonical_has_match(&tx, "\"evidencia\"").unwrap(),
        None
    );
}
