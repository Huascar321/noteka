use super::*;

#[test]
fn repeated_postings_are_exact_order_independent_and_detect_duplicate_missing_pairs() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("CREATE VIRTUAL TABLE actual USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);CREATE VIRTUAL TABLE temp.source_vocab USING fts5vocab('main','actual','instance');CREATE TEMP TABLE attachment_actual_vocab(term TEXT,doc INTEGER,col TEXT,offset INTEGER);CREATE TEMP TABLE attachment_expected_vocab(term TEXT,doc INTEGER,col TEXT,offset INTEGER);").unwrap();
    let texts = [
        "CAFÉ café αββ quoted\" a a 😀 token",
        "different 123 élève --  end",
    ];
    let fragments = (1..=16)
        .map(|n| (if n == 16 { 524288 } else { n }, texts[n as usize % 2]))
        .collect::<Vec<_>>();
    for &(rowid, text) in &fragments {
        c.execute(
            "INSERT INTO actual(rowid,text) VALUES(?1,?2)",
            params![rowid, text],
        )
        .unwrap();
    }
    // No se supone el orden público de vocabularios. Tampoco se confunden
    // documentos separados con texto idéntico ni términos repetidos/diacríticos.
    c.execute_batch("INSERT INTO attachment_expected_vocab SELECT term,doc,col,offset FROM source_vocab;INSERT INTO attachment_actual_vocab SELECT * FROM attachment_expected_vocab ORDER BY offset DESC,doc DESC,term DESC").unwrap();
    let b = Budget::new(&c).unwrap();
    assert_eq!(
        index::same_repeated_postings(&c, &b, &fragments).unwrap(),
        Some(true)
    );
    assert!(index::same_postings(&c, &b).unwrap());
    let n: u32 = c
        .query_row("SELECT count(*) FROM attachment_expected_vocab", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(n, 88);
    for sql in [
        "DELETE FROM attachment_actual_vocab WHERE rowid=1",
        "INSERT INTO attachment_actual_vocab SELECT term,doc,col,offset FROM attachment_actual_vocab LIMIT 1",
        "UPDATE attachment_actual_vocab SET term='absent' WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET col='ordinal' WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET doc=524289 WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET doc=0 WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET offset=-1 WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET offset=2147483647 WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET offset=NULL WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET term=x'63616665' WHERE rowid=1",
        "UPDATE attachment_actual_vocab SET col=NULL WHERE rowid=1",
        // Misma cardinalidad, reemplaza una tupla por otra ya presente.
        "DELETE FROM attachment_actual_vocab WHERE rowid=1;INSERT INTO attachment_actual_vocab SELECT term,doc,col,offset FROM attachment_actual_vocab LIMIT 1",
    ] {
        c.execute_batch("DELETE FROM attachment_actual_vocab;INSERT INTO attachment_actual_vocab SELECT * FROM attachment_expected_vocab").unwrap();
        c.execute_batch(sql).unwrap();
        assert_eq!(
            index::same_repeated_postings(&c, &b, &fragments).unwrap(),
            Some(false),
            "{sql}"
        );
        assert!(!index::same_postings(&c, &b).unwrap(), "{sql}");
        let leaked: u32 = c
            .query_row(
                "SELECT count(*) FROM temp.sqlite_schema WHERE name LIKE 'attachment_repeated_%'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(leaked, 0);
    }
    // Sin reutilización suficiente conserva el camino completo anterior.
    assert_eq!(
        index::same_repeated_postings(&c, &b, &[(1, "one"), (2, "two")]).unwrap(),
        None
    );
    assert_eq!(index::same_repeated_postings(&c, &b, &[]).unwrap(), None);
    c.execute_batch("DELETE FROM attachment_actual_vocab")
        .unwrap();
    let empty = [(1, "--"), (2, "--"), (3, "--"), (4, "--")];
    assert_eq!(
        index::same_repeated_postings(&c, &b, &empty).unwrap(),
        Some(true)
    );
    c.execute_batch("INSERT INTO attachment_actual_vocab VALUES('unexpected',1,'text',0)")
        .unwrap();
    assert_eq!(
        index::same_repeated_postings(&c, &b, &empty).unwrap(),
        Some(false)
    );
}

#[test]
fn copy_stream_checks_complete_rows_types_and_rowids() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("CREATE VIRTUAL TABLE extraction_search_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text);CREATE VIRTUAL TABLE temp.attachment_expected_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text);INSERT INTO extraction_search_fts(rowid,job_id,generation_id,ordinal,text) VALUES(2,'j','g',1,'two'),(1,'j','g',0,'one');INSERT INTO attachment_expected_fts(rowid,job_id,generation_id,ordinal,text) VALUES(1,'j','g',0,'one'),(2,'j','g',1,'two');").unwrap();
    let b = Budget::new(&c).unwrap();
    assert!(index::same_copies(&c, &b).unwrap());
    for sql in [
        "UPDATE extraction_search_fts SET ordinal='0' WHERE rowid=1",
        "UPDATE extraction_search_fts SET ordinal=0,job_id='other' WHERE rowid=1",
        "UPDATE extraction_search_fts SET job_id='j',generation_id='other' WHERE rowid=1",
        "UPDATE extraction_search_fts SET generation_id='g',text='changed' WHERE rowid=1",
        "UPDATE extraction_search_fts SET text='one',rowid=3 WHERE rowid=1",
        "DELETE FROM extraction_search_fts WHERE rowid=3",
    ] {
        c.execute_batch(sql).unwrap();
        assert!(!index::same_copies(&c, &b).unwrap(), "{sql}");
    }
    c.execute_batch("INSERT INTO extraction_search_fts(rowid,job_id,generation_id,ordinal,text) VALUES(1,'j','g',0,'one'),(3,'j','g',2,'extra')").unwrap();
    assert!(!index::same_copies(&c, &b).unwrap());
}

#[test]
fn sequence_fast_path_does_not_assume_iteration_order_or_ignore_duplicates() {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch("CREATE TEMP TABLE attachment_actual_vocab(term TEXT,doc INTEGER,col TEXT,offset INTEGER);CREATE TEMP TABLE attachment_expected_vocab(term TEXT,doc INTEGER,col TEXT,offset INTEGER);INSERT INTO attachment_actual_vocab VALUES('a',1,'text',0),('b',2,'text',0);INSERT INTO attachment_expected_vocab VALUES('b',2,'text',0),('a',1,'text',0);").unwrap();
    let b = Budget::new(&c).unwrap();
    assert!(!index::same_posting_sequence(&c, &b).unwrap());
    assert!(
        index::same_postings(&c, &b).unwrap(),
        "El fallback acepta exactamente el mismo conjunto con otro orden"
    );
    c.execute_batch("DELETE FROM attachment_actual_vocab;INSERT INTO attachment_actual_vocab VALUES('a',1,'text',0),('a',1,'text',0)").unwrap();
    assert!(!index::same_posting_sequence(&c, &b).unwrap());
    assert!(
        !index::same_postings(&c, &b).unwrap(),
        "Mismo conteo pero posting repetido y otro faltante no es igualdad"
    );
}

#[test]
fn page_guard_lives_through_commit_restores_prior_and_catches_late_fts_allocation() {
    let c = Connection::open_in_memory().unwrap();
    let old: u32 = c
        .pragma_query_value(None, "max_page_count", |r| r.get(0))
        .unwrap();
    let guard = PageLimit::new(&c).unwrap();
    assert!(
        c.pragma_query_value(None, "max_page_count", |r| r.get::<_, u32>(0))
            .unwrap()
            < old
    );
    drop(guard);
    assert_eq!(
        c.pragma_query_value(None, "max_page_count", |r| r.get::<_, u32>(0))
            .unwrap(),
        old
    );
    let mut late = false;
    for pages in 8..80 {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE VIRTUAL TABLE f USING fts5(text)")
            .unwrap();
        c.pragma_update(None, "max_page_count", pages).unwrap();
        let old: u32 = c
            .pragma_query_value(None, "max_page_count", |r| r.get(0))
            .unwrap();
        let guard = PageLimit::new(&c).unwrap();
        assert_eq!(
            c.pragma_query_value(None, "max_page_count", |r| r.get::<_, u32>(0))
                .unwrap(),
            old
        );
        let tx = c.unchecked_transaction().unwrap();
        let text = (0..4000).map(|n| format!("term{n} ")).collect::<String>();
        if tx.execute("INSERT INTO f VALUES(?1)", [text]).is_ok()
            && let Err(e) = tx.commit()
        {
            assert!(
                matches!(e,rusqlite::Error::SqliteFailure(s,_) if s.code==rusqlite::ErrorCode::DiskFull)
            );
            assert_eq!(
                c.query_row("SELECT count(*) FROM f", [], |r| r.get::<_, u32>(0))
                    .unwrap(),
                0
            );
            late = true;
        }
        drop(guard);
        assert_eq!(
            c.pragma_query_value(None, "max_page_count", |r| r.get::<_, u32>(0))
                .unwrap(),
            old
        );
        if late {
            break;
        }
    }
    assert!(
        late,
        "Debe probar asignación FTS tardía durante COMMIT, no sólo INSERT"
    );
}

#[test]
fn budget_expiry_and_sqlite_interrupt_leave_no_persistent_or_temp_state() {
    let t = tempfile::tempdir().unwrap();
    let w = crate::workspace::Workspace::create(&t.path().join("w"), None).unwrap();
    let before = crate::extraction_tests::raw(&w.conn);
    let b = Budget::with_duration(&w.conn, std::time::Duration::ZERO).unwrap();
    let e = index::verify(&w.conn, &b).err().unwrap();
    assert!(matches!(b.map(e).code, ErrorCode::WorkspaceBusy));
    drop(b);
    assert_eq!(before, crate::extraction_tests::raw(&w.conn));
    let b = Budget::new(&w.conn).unwrap();
    drop(index::verify(&w.conn, &b).unwrap());
    assert!(
        !w.conn
            .prepare("SELECT 1 FROM sqlite_temp_schema WHERE name LIKE 'attachment_%'")
            .unwrap()
            .exists([])
            .unwrap()
    );
}
