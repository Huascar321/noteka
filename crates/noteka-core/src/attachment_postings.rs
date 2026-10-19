//! Exact posting traversal through the public FTS5 auxiliary API. No SQL writes
//! to main, tokenizer replacement, digest-only checks or cross-snapshot cache.
use crate::{batch_storage::Budget, error::*};
use rusqlite::{Connection, OptionalExtension, ffi};
use std::{collections::HashMap, ffi::c_void, marker::PhantomData, panic::AssertUnwindSafe, ptr};

fn sql_error(rc: i32) -> CoreError {
    rusqlite::Error::SqliteFailure(ffi::Error::new(rc), None).into()
}
struct Statement<'a>(*mut ffi::sqlite3_stmt, PhantomData<&'a Connection>);
impl Drop for Statement<'_> {
    fn drop(&mut self) {
        // SAFETY: exclusively owned prepared statement, finalized exactly once.
        unsafe {
            ffi::sqlite3_finalize(self.0);
        }
    }
}
fn prepare<'a>(c: &'a Connection, sql: &std::ffi::CStr) -> Result<Statement<'a>> {
    let mut stmt = ptr::null_mut();
    // SAFETY: the connection lives through the statement; SQL is nul terminated.
    let rc = unsafe {
        ffi::sqlite3_prepare_v2(c.handle(), sql.as_ptr(), -1, &mut stmt, ptr::null_mut())
    };
    if rc != ffi::SQLITE_OK {
        return Err(sql_error(rc));
    }
    Ok(Statement(stmt, PhantomData))
}
fn register(c: &Connection) -> Result<bool> {
    // Only registration metadata, not validation results, persists per connection.
    // xCreateFunction appends registrations, so do not register on every query.
    let present: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_function_list WHERE name='noteka_verify_postings_v1')",
        [],
        |r| r.get(0),
    )?;
    if present {
        return Ok(true);
    }
    let stmt = prepare(c, c"SELECT fts5(?1)")?;
    let mut api: *mut ffi::fts5_api = ptr::null_mut();
    // SAFETY: SQLite's documented fts5_api_ptr protocol writes only this local
    // pointer during step. No destructor owns it; the connection owns the API.
    unsafe {
        let rc = ffi::sqlite3_bind_pointer(
            stmt.0,
            1,
            (&mut api as *mut *mut ffi::fts5_api).cast(),
            c"fts5_api_ptr".as_ptr(),
            None,
        );
        if rc != ffi::SQLITE_OK {
            return Err(sql_error(rc));
        }
        let rc = ffi::sqlite3_step(stmt.0);
        if rc != ffi::SQLITE_ROW {
            return Err(sql_error(rc));
        }
        if api.is_null() {
            return Ok(false);
        }
        let Some(create) = (*api).xCreateFunction else {
            return Ok(false);
        };
        let rc = create(
            api,
            c"noteka_verify_postings_v1".as_ptr(),
            ptr::null_mut(),
            Some(callback),
            None,
        );
        if rc != ffi::SQLITE_OK {
            return Err(sql_error(rc));
        }
    }
    Ok(true)
}
struct State<'a, 'b> {
    positions: &'a [Vec<Option<u32>>],
    documents: &'a [Option<(usize, usize)>],
    seen: &'a mut [u64],
    budget: &'a Budget<'b>,
    term: u32,
    phrase_terms: Vec<u32>,
    phrases_checked: bool,
    count: u32,
    error: Option<CoreError>,
}
impl State<'_, '_> {
    fn instance(&mut self, doc: i64, offset: i32) -> bool {
        let (Ok(doc), Ok(offset)) = (usize::try_from(doc), usize::try_from(offset)) else {
            return false;
        };
        let Some(Some((class, base))) = self.documents.get(doc) else {
            return false;
        };
        if self.positions.get(*class).and_then(|p| p.get(offset)) != Some(&Some(self.term)) {
            return false;
        }
        let Some(bit) = base.checked_add(offset) else {
            return false;
        };
        let Some(word) = self.seen.get_mut(bit / 64) else {
            return false;
        };
        let mask = 1u64 << (bit % 64);
        if *word & mask != 0 {
            return false;
        }
        *word |= mask;
        self.count += 1;
        if self.count.is_multiple_of(1024)
            && let Err(e) = self.budget.check()
        {
            self.error = Some(e);
            return false;
        }
        true
    }
}
unsafe extern "C" fn callback(
    api: *const ffi::Fts5ExtensionApi,
    fts: *mut ffi::Fts5Context,
    ctx: *mut ffi::sqlite3_context,
    n: i32,
    args: *mut *mut ffi::sqlite3_value,
) {
    // Never unwind through SQLite's C ABI, including a future Rust change.
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: SQLite supplies these callback parameters. The tagged pointer
        // is bound only by verify below, with a State alive until finalization.
        unsafe {
            if api.is_null() || n != 1 || args.is_null() {
                return false;
            }
            let raw = ffi::sqlite3_value_pointer(*args, c"noteka_postings_v1".as_ptr());
            if raw.is_null() {
                return false;
            }
            let state = &mut *raw.cast::<State<'_, '_>>();
            let (Some(rowid), Some(count), Some(instance), Some(phrases), Some(size)) = (
                (*api).xRowid,
                (*api).xInstCount,
                (*api).xInst,
                (*api).xPhraseCount,
                (*api).xPhraseSize,
            ) else {
                return false;
            };
            if !state.phrases_checked {
                if phrases(fts) != state.phrase_terms.len() as i32
                    || state.phrase_terms.is_empty()
                    || (0..phrases(fts)).any(|i| size(fts, i) != 1)
                {
                    return false;
                }
                // Phrase structure is invariant for this one bound MATCH query,
                // not cached postings or trust across statements/snapshots.
                state.phrases_checked = true;
            }
            let doc = rowid(fts);
            if state.phrase_terms.len() > 1 {
                let (Some(first), Some(next)) = ((*api).xPhraseFirst, (*api).xPhraseNext) else {
                    return false;
                };
                let before = state.count;
                for phrase in 0..state.phrase_terms.len() {
                    state.term = state.phrase_terms[phrase];
                    let mut iter = ffi::Fts5PhraseIter {
                        a: ptr::null(),
                        b: ptr::null(),
                    };
                    let (mut column, mut offset) = (-1, -1);
                    let rc = first(fts, phrase as i32, &mut iter, &mut column, &mut offset);
                    if rc != ffi::SQLITE_OK {
                        state.error = Some(sql_error(rc));
                        return false;
                    }
                    // Same instances as xInst, without collating/sorting a merged
                    // array. Iterators are local to this row and phrase. Empty
                    // phrases in an OR batch are valid; an empty matching row is not.
                    while column >= 0 {
                        if column != 3 || !state.instance(doc, offset) {
                            return false;
                        }
                        next(fts, &mut iter, &mut column, &mut offset);
                    }
                }
                return state.count > before;
            }
            let mut len = 0;
            let rc = count(fts, &mut len);
            if rc != ffi::SQLITE_OK {
                state.error = Some(sql_error(rc));
                return false;
            }
            if len <= 0 {
                return false;
            }
            for i in 0..len {
                let (mut phrase, mut column, mut offset) = (0, 0, 0);
                let rc = instance(fts, i, &mut phrase, &mut column, &mut offset);
                if rc != ffi::SQLITE_OK {
                    state.error = Some(sql_error(rc));
                    return false;
                }
                let Some(term) = usize::try_from(phrase)
                    .ok()
                    .and_then(|i| state.phrase_terms.get(i))
                else {
                    return false;
                };
                state.term = *term;
                if column != 3 || !state.instance(doc, offset) {
                    return false;
                }
            }
            true
        }
    }));
    // SAFETY: ctx is valid for the duration of this SQLite callback.
    unsafe {
        ffi::sqlite3_result_int(ctx, i32::from(result.unwrap_or(false)));
    }
}

// Each actual vocabulary term is checked, then every matching row/instance is
// checked against the canonical position map and its independent bitset. Thus
// A ⊆ E, no duplicate A and equal cardinality still prove exact equality.
pub(crate) fn verify(
    c: &Connection,
    b: &Budget<'_>,
    terms: &HashMap<Vec<u8>, u32>,
    positions: &[Vec<Option<u32>>],
    documents: &[Option<(usize, usize)>],
    seen: &mut [u64],
    expected_count: u32,
) -> Result<Option<bool>> {
    // Small vocabularies retain the original single-term traversal. Larger ones
    // use bounded OR batches, not one SQL result row per posting. Every actual
    // term and instance is still checked. 256 is an algorithm switch, not a cap;
    // unavailable APIs and independent/materialized streams keep verify_stream.
    let batch_size = if terms.len() > 256 { 64 } else { 1 };
    // The optimized traversal must examine the very same index as the input
    // vocabulary, not silently bypass an independently supplied/materialized
    // posting stream (which the full verifier also supports).
    let source: Option<String> = c.query_row("SELECT sql FROM temp.sqlite_schema WHERE type='table' AND name='attachment_actual_vocab'", [], |r| r.get(0)).optional()?;
    if source.as_deref()
        != Some(
            "CREATE VIRTUAL TABLE attachment_actual_vocab USING fts5vocab('main','extraction_search_fts','instance')",
        )
    {
        return Ok(None);
    }
    if !register(c)? {
        return Ok(None);
    }
    struct Scratch<'a>(&'a Connection);
    impl Drop for Scratch<'_> {
        fn drop(&mut self) {
            let _ = self
                .0
                .execute_batch("DROP TABLE IF EXISTS temp.attachment_actual_terms;");
        }
    }
    let _scratch = Scratch(c);
    c.execute_batch("CREATE VIRTUAL TABLE temp.attachment_actual_terms USING fts5vocab('main','extraction_search_fts','row');")?;
    let mut vocab = c.prepare("SELECT term FROM temp.attachment_actual_terms")?;
    let mut rows = vocab.query([])?;
    let mut state = State {
        positions,
        documents,
        seen,
        budget: b,
        term: 0,
        phrase_terms: Vec::with_capacity(batch_size),
        phrases_checked: false,
        count: 0,
        error: None,
    };
    let query = prepare(c, c"SELECT noteka_verify_postings_v1(extraction_search_fts,?2) FROM main.extraction_search_fts WHERE extraction_search_fts MATCH ?1")?;
    // The statement is finalized before state goes out of scope on every return.
    // SAFETY: the borrowed State stays at a stable address during synchronous
    // step/reset; no callback escapes, nested SQL or pointer destructor exists.
    unsafe {
        let rc = ffi::sqlite3_bind_pointer(
            query.0,
            2,
            (&mut state as *mut State<'_, '_>).cast::<c_void>(),
            c"noteka_postings_v1".as_ptr(),
            None,
        );
        if rc != ffi::SQLITE_OK {
            return Err(sql_error(rc));
        }
    }
    loop {
        let mut expression = String::new();
        state.phrase_terms.clear();
        state.phrases_checked = false;
        while state.phrase_terms.len() < batch_size {
            let Some(row) = rows.next()? else {
                break;
            };
            b.check()?;
            let rusqlite::types::ValueRef::Text(term) = row.get_ref(0)? else {
                return Ok(Some(false));
            };
            let Some(id) = terms.get(term) else {
                return Ok(Some(false));
            };
            let Ok(term) = std::str::from_utf8(term) else {
                return Ok(Some(false));
            };
            if !expression.is_empty() {
                expression.push_str(" OR ");
            }
            expression.push('"');
            expression.push_str(&term.replace('"', "\"\""));
            expression.push('"');
            state.phrase_terms.push(*id);
        }
        if state.phrase_terms.is_empty() {
            break;
        }
        // SAFETY: SQLITE_TRANSIENT copies this expression, no borrowed text is
        // held after bind; reset preserves the tagged State binding in slot 2.
        unsafe {
            let rc = ffi::sqlite3_bind_text(
                query.0,
                1,
                expression.as_ptr().cast(),
                expression.len() as i32,
                ffi::SQLITE_TRANSIENT(),
            );
            if rc != ffi::SQLITE_OK {
                return Err(sql_error(rc));
            }
            loop {
                match ffi::sqlite3_step(query.0) {
                    ffi::SQLITE_ROW => {
                        if let Some(error) = state.error.take() {
                            return Err(error);
                        }
                        if ffi::sqlite3_column_int(query.0, 0) != 1 {
                            return Ok(Some(false));
                        }
                    }
                    ffi::SQLITE_DONE => break,
                    rc => return Err(sql_error(rc)),
                }
            }
            let rc = ffi::sqlite3_reset(query.0);
            if rc != ffi::SQLITE_OK {
                return Err(sql_error(rc));
            }
        }
    }
    b.check()?;
    Ok(Some(state.count == expected_count))
}

// Complete instance-stream fallback, including independently materialized
// vocabularies. The fixed four-column projection avoids a dynamic Row/index
// conversion for every value of every posting; it does NOT skip any instance,
// type, column, offset, term, duplicate, cardinality or cooperative budget check.
pub(crate) fn verify_stream(
    c: &Connection,
    b: &Budget<'_>,
    terms: &HashMap<Vec<u8>, u32>,
    positions: &[Vec<Option<u32>>],
    documents: &[Option<(usize, usize)>],
    seen: &mut [u64],
    expected_count: u32,
) -> Result<bool> {
    let mut state = State {
        positions,
        documents,
        seen,
        budget: b,
        term: 0,
        phrase_terms: Vec::new(),
        phrases_checked: false,
        count: 0,
        error: None,
    };
    let stmt = prepare(
        c,
        c"SELECT term,doc,col,offset FROM temp.attachment_actual_vocab",
    )?;
    let mut last_term = Vec::new();
    let mut term_id = None;
    loop {
        // SAFETY: stmt owns a live prepared statement on this private connection.
        // Only ROW permits column access. The fixed SELECT has four columns. Check
        // original SQLite types BEFORE conversions. Borrowed UTF-8 buffers remain
        // inside this iteration and never cross step/reset/finalize or conversion
        // of the same value. text() precedes bytes(); NULL/OOM is never dereferenced.
        unsafe {
            match ffi::sqlite3_step(stmt.0) {
                ffi::SQLITE_DONE => break,
                ffi::SQLITE_ROW => {}
                rc => return Err(sql_error(rc)),
            }
            if ffi::sqlite3_column_type(stmt.0, 0) != ffi::SQLITE_TEXT
                || ffi::sqlite3_column_type(stmt.0, 1) != ffi::SQLITE_INTEGER
                || ffi::sqlite3_column_type(stmt.0, 2) != ffi::SQLITE_TEXT
                || ffi::sqlite3_column_type(stmt.0, 3) != ffi::SQLITE_INTEGER
            {
                return Ok(false);
            }
            let column = ffi::sqlite3_column_text(stmt.0, 2);
            let column_bytes = ffi::sqlite3_column_bytes(stmt.0, 2);
            if column.is_null() {
                return Err(sql_error(ffi::SQLITE_NOMEM));
            }
            if column_bytes != 4 || std::slice::from_raw_parts(column, 4) != b"text" {
                return Ok(false);
            }
            let term = ffi::sqlite3_column_text(stmt.0, 0);
            let term_bytes = ffi::sqlite3_column_bytes(stmt.0, 0);
            if term.is_null() {
                return Err(sql_error(ffi::SQLITE_NOMEM));
            }
            let Ok(term_bytes) = usize::try_from(term_bytes) else {
                return Ok(false);
            };
            let term = std::slice::from_raw_parts(term, term_bytes);
            if term_id.is_none() || last_term.as_slice() != term {
                term_id = terms.get(term).copied();
                let Some(id) = term_id else {
                    return Ok(false);
                };
                state.term = id;
                last_term.clear();
                last_term.extend_from_slice(term);
            }
            let doc = ffi::sqlite3_column_int64(stmt.0, 1);
            let Ok(offset) = i32::try_from(ffi::sqlite3_column_int64(stmt.0, 3)) else {
                return Ok(false);
            };
            if !state.instance(doc, offset) {
                if let Some(error) = state.error.take() {
                    return Err(error);
                }
                return Ok(false);
            }
        }
    }
    b.check()?;
    Ok(state.count == expected_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachment_search_storage as index;

    fn fixture(texts: &[&str]) -> (Connection, Vec<(u32, String)>) {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch("CREATE VIRTUAL TABLE main.extraction_search_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);CREATE VIRTUAL TABLE temp.attachment_expected_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);CREATE VIRTUAL TABLE temp.attachment_actual_vocab USING fts5vocab('main','extraction_search_fts','instance');CREATE VIRTUAL TABLE temp.attachment_expected_vocab USING fts5vocab('temp','attachment_expected_fts','instance');").unwrap();
        let mut fragments = Vec::new();
        for i in 0..16 {
            let id = (i * 512 + 1) as u32;
            let text = texts[i % texts.len()];
            for table in ["main.extraction_search_fts", "temp.attachment_expected_fts"] {
                c.execute(&format!("INSERT INTO {table}(rowid,job_id,generation_id,ordinal,text) VALUES(?1,'job','generation',0,?2)"),rusqlite::params![id,text]).unwrap();
            }
            fragments.push((id, text.to_owned()));
        }
        (c, fragments)
    }
    fn compare(c: &Connection, fragments: &[(u32, String)], expected: bool) {
        let b = Budget::new(c).unwrap();
        let input = fragments
            .iter()
            .map(|(id, s)| (*id, s.as_str()))
            .collect::<Vec<_>>();
        // The old oracle reconstructs/tokenizes every expected document and
        // compares the complete independent fts5vocab instance stream/set.
        assert_eq!(index::same_postings(c, &b).unwrap(), expected);
        assert_eq!(
            index::same_repeated_postings(c, &b, &input).unwrap(),
            Some(expected)
        );
        let scratch: u32 = c
            .query_row(
                "SELECT count(*) FROM sqlite_temp_master WHERE name='attachment_actual_terms'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(scratch, 0);
    }
    #[test]
    fn api_matches_full_instance_oracle_for_unicode_empty_and_repeated_positions() {
        for texts in [
            vec!["alpha beta alpha"],
            vec!["Café CAFÉ café", "漢字 հայերեն Ελληνικά"],
            vec!["", "---"],
            vec!["a a a a a b", "c c d c c d"],
        ] {
            let (c, fragments) = fixture(&texts);
            compare(&c, &fragments, true);
            // No pointer survives statement finalization. A call without our
            // tagged binding rejects safely even though registration persists.
            let rejected = c.prepare("SELECT noteka_verify_postings_v1(extraction_search_fts,NULL) FROM extraction_search_fts WHERE extraction_search_fts MATCH 'alpha'").unwrap().query_map([],|r|r.get::<_,i32>(0)).unwrap().collect::<std::result::Result<Vec<_>,_>>().unwrap();
            assert!(rejected.iter().all(|value| *value == 0));
        }
    }
    #[test]
    fn api_rejects_hidden_missing_extra_shifted_and_foreign_document_postings() {
        for sql in [
            "UPDATE extraction_search_fts SET text='beta alpha alpha' WHERE rowid=1",
            "UPDATE extraction_search_fts SET text='alpha alpha alpha' WHERE rowid=1",
            "UPDATE extraction_search_fts SET text='alpha beta alpha extra' WHERE rowid=1",
            "UPDATE extraction_search_fts SET text='alpha beta' WHERE rowid=1",
            "UPDATE extraction_search_fts SET text='' WHERE rowid=1",
            "DELETE FROM extraction_search_fts WHERE rowid=1",
            "INSERT INTO extraction_search_fts(rowid,text) VALUES(2,'alpha beta alpha')",
        ] {
            let (c, fragments) = fixture(&["alpha beta alpha"]);
            c.execute_batch(sql).unwrap();
            // Restore copied content without repairing the deliberately corrupt
            // index, so copied-text equality cannot conceal the posting check.
            c.execute(
                "UPDATE extraction_search_fts_content SET c3='alpha beta alpha' WHERE id=1",
                [],
            )
            .unwrap();
            compare(&c, &fragments, false);
        }
    }
    #[test]
    fn api_repeated_calls_have_no_validation_cache_or_live_bound_pointer() {
        let (c, fragments) = fixture(&["alpha beta alpha"]);
        for _ in 0..32 {
            compare(&c, &fragments, true);
        }
        let registrations: u32 = c
            .query_row(
                "SELECT count(*) FROM pragma_function_list WHERE name='noteka_verify_postings_v1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(registrations, 1);
        c.execute(
            "UPDATE extraction_search_fts SET text='alpha' WHERE rowid=1",
            [],
        )
        .unwrap();
        c.execute(
            "UPDATE extraction_search_fts_content SET c3='alpha beta alpha' WHERE id=1",
            [],
        )
        .unwrap();
        compare(&c, &fragments, false);
    }

    #[test]
    fn high_vocabulary_batches_match_full_stream_and_detect_corruption() {
        let text = (0..300)
            .map(|i| format!("term{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let (c, fragments) = fixture(&[&text]);
        compare(&c, &fragments, true);
        let registrations: u32 = c
            .query_row(
                "SELECT count(*) FROM pragma_function_list WHERE name='noteka_verify_postings_v1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(registrations, 1);
        c.execute(
            "UPDATE extraction_search_fts SET text='term0' WHERE rowid=1",
            [],
        )
        .unwrap();
        c.execute(
            "UPDATE extraction_search_fts_content SET c3=?1 WHERE id=1",
            [&text],
        )
        .unwrap();
        compare(&c, &fragments, false);
    }

    #[test]
    fn batch_switch_boundaries_and_partial_batches_match_full_stream() {
        for count in [0, 1, 255, 256, 257, 300, 512, 513, 700] {
            let text = (0..count)
                .map(|i| format!("t{i:03}"))
                .collect::<Vec<_>>()
                .join(" ");
            let (c, fragments) = fixture(&[&text]);
            compare(&c, &fragments, true);
            if count > 0 {
                c.execute(
                    "UPDATE extraction_search_fts SET text='foreignterm' WHERE rowid=1",
                    [],
                )
                .unwrap();
                c.execute(
                    "UPDATE extraction_search_fts_content SET c3=?1 WHERE id=1",
                    [&text],
                )
                .unwrap();
                compare(&c, &fragments, false);
            }
        }
    }

    #[test]
    fn batched_phrases_cover_sparse_rows_unicode_repetition_and_corruption() {
        let a = (0..300)
            .map(|i| format!("term{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let z = format!(
            "{} Café CAFÉ café 漢字",
            (300..701)
                .map(|i| format!("term{i} term{i}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        for changed in [
            None,
            Some("term0"),
            Some("foreign"),
            Some("Café 漢字"),
            Some(""),
        ] {
            let (c, fragments) = fixture(&[&a, &z, "", "term700 term0"]);
            if let Some(text) = changed {
                c.execute(
                    "UPDATE extraction_search_fts SET text=?1 WHERE rowid=513",
                    [text],
                )
                .unwrap();
                c.execute(
                    "UPDATE extraction_search_fts_content SET c3=?1 WHERE id=513",
                    [&z],
                )
                .unwrap();
            }
            compare(&c, &fragments, changed.is_none());
        }
    }

    #[test]
    fn stream_checks_exact_materialized_types_values_duplicates_and_budget() {
        for mutation in [
            "",
            "UPDATE attachment_actual_vocab SET term=NULL WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET term=42 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET term=X'616C706861' WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET term='unknown' WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET doc='1' WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET doc=-1 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET doc=0 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET doc=2 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET col='TEXT' WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET col=X'74657874' WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET offset=0.0 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET offset=-1 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET offset=2147483648 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET offset=9223372036854775807 WHERE rowid=1",
            "UPDATE attachment_actual_vocab SET offset=1 WHERE rowid=1",
            "DELETE FROM attachment_actual_vocab WHERE rowid=1",
            "INSERT INTO attachment_actual_vocab SELECT * FROM attachment_actual_vocab LIMIT 1",
        ] {
            let c = Connection::open_in_memory().unwrap();
            // Untyped columns ensure SQLite cannot silently coerce a hostile value
            // into the type expected by the verifier. Order deliberately revisits
            // earlier terms; no reliance on term/doc/offset sort order.
            c.execute_batch("CREATE TEMP TABLE attachment_actual_vocab(term,doc,col,offset);INSERT INTO attachment_actual_vocab VALUES('alpha',1,'text',0),('beta',513,'text',1),('alpha',513,'text',2),('beta',1,'text',1),('alpha',513,'text',0),('alpha',1,'text',2)").unwrap();
            c.execute_batch(mutation).unwrap();
            let terms = HashMap::from([(b"alpha".to_vec(), 0), (b"beta".to_vec(), 1)]);
            let positions = [vec![Some(0), Some(1), Some(0)]];
            let mut docs = vec![None; 514];
            docs[1] = Some((0, 0));
            docs[513] = Some((0, 3));
            let b = Budget::new(&c).unwrap();
            assert_eq!(
                verify_stream(&c, &b, &terms, &positions, &docs, &mut [0], 6).unwrap(),
                mutation.is_empty(),
                "{mutation}"
            );
            if mutation.is_empty() {
                let deadline = Budget::with_duration(&c, std::time::Duration::ZERO).unwrap();
                assert_eq!(
                    verify_stream(&c, &deadline, &terms, &positions, &docs, &mut [0], 6)
                        .unwrap_err()
                        .details["reason"],
                    "execution_deadline"
                );
            }
        }
    }

    #[test]
    fn callback_position_checks_duplicates_unknown_rows_and_deadline() {
        let c = Connection::open_in_memory().unwrap();
        let b = Budget::with_duration(&c, std::time::Duration::ZERO).unwrap();
        let mut seen = [0];
        let mut state = State {
            positions: &[vec![Some(0), Some(0)]],
            documents: &[None, Some((0, 0))],
            seen: &mut seen,
            budget: &b,
            term: 0,
            phrase_terms: Vec::new(),
            phrases_checked: false,
            count: 0,
            error: None,
        };
        assert!(!state.instance(-1, 0));
        assert!(!state.instance(0, 0));
        assert!(!state.instance(2, 0));
        assert!(!state.instance(1, -1));
        assert!(!state.instance(1, 2));
        assert!(state.instance(1, 0));
        assert!(!state.instance(1, 0));
        state.count = 1023;
        assert!(!state.instance(1, 1));
        assert_eq!(state.error.unwrap().details["reason"], "execution_deadline");
    }
}
