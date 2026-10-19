use crate::{
    attachment_search_storage as index,
    batch_storage::{self, Budget},
    contracts::*,
    error::*,
    extraction_storage as store,
};
use rusqlite::{Connection, params};
use std::path::Path;
#[cfg(test)]
#[path = "attachment_search_limits_tests.rs"]
mod limits_tests;

fn positive(n: u32) -> Result<()> {
    if n == 0 {
        Err(store::invalid("expected_revision"))
    } else {
        Ok(())
    }
}
fn filter(
    job_id: Option<String>,
    attachment_id: Option<String>,
    extractor: Option<IndexedExtractor>,
) -> Result<crate::attachment_search_contracts::IndexFilter> {
    let f = crate::attachment_search_contracts::IndexFilter {
        job_id,
        attachment_id,
        extractor,
    };
    f.validate()?;
    Ok(f)
}

pub(crate) fn index(path: &Path, input: AttachmentIndexInput) -> Result<AttachmentIndexResult> {
    index_with_hook(path, input, &mut |_| Ok(()))
}
pub(crate) fn index_scoped(
    path: &Path,
    input: AttachmentIndexInput,
    scope: Option<&crate::scoped::LocalScope>,
) -> Result<AttachmentIndexResult> {
    index_inner(path, input, scope, &mut |_| Ok(()))
}
pub(crate) fn index_with_hook(
    path: &Path,
    input: AttachmentIndexInput,
    hook: &mut dyn FnMut(&str) -> Result<()>,
) -> Result<AttachmentIndexResult> {
    index_inner(path, input, None, hook)
}
fn index_inner(
    path: &Path,
    input: AttachmentIndexInput,
    scope: Option<&crate::scoped::LocalScope>,
    hook: &mut dyn FnMut(&str) -> Result<()>,
) -> Result<AttachmentIndexResult> {
    let write = !matches!(input, AttachmentIndexInput::Get { .. });
    match &input {
        AttachmentIndexInput::Get {
            job_id,
            attachment_id,
            extractor,
            limit,
            cursor,
        } => {
            filter(job_id.clone(), attachment_id.clone(), *extractor)?;
            store::page_limit(*limit, 32, 64)?;
            if cursor.as_ref().is_some_and(|s| s.len() > 2048) {
                return Err(store::invalid("cursor_size_limit"));
            }
        }
        AttachmentIndexInput::Put {
            job_id,
            expected_generation_id,
            expected_revision,
        } => {
            store::uuid(job_id)?;
            store::uuid(expected_generation_id)?;
            positive(*expected_revision)?;
        }
        AttachmentIndexInput::Reset { expected_revision } => positive(*expected_revision)?,
    }
    let (c, root) = index::open(path, write)?;
    if let Some(scope) = scope {
        scope.check(&c)?;
    }
    let b = Budget::new(&c)?;
    let result = (|| {
        let out = match input {
            AttachmentIndexInput::Get {
                job_id,
                attachment_id,
                extractor,
                limit,
                cursor,
            } => {
                let f = filter(job_id, attachment_id, extractor)?;
                let limit = store::page_limit(limit, 32, 64)?;
                let tx = c.unchecked_transaction()?;
                if let Some(scope) = scope {
                    scope.check(&tx)?;
                }
                let revision = index::control(&tx)?.revision;
                let shape = index::Cursor::new(
                    &tx,
                    &root,
                    "extraction.index.get",
                    "",
                    f.clone(),
                    limit,
                    revision,
                )?;
                let offset = shape.offset(cursor.as_deref(), MAX_INDEX_ENTRIES)?;
                let verified = index::verify(&tx, &b)?;
                let coverage = index::coverage(
                    &tx,
                    &f,
                    &verified.entries,
                    IndexSourceValidation::NotChecked,
                    &b,
                )?;
                let matching = verified
                    .entries
                    .iter()
                    .filter(|e| index::matches_filter(&e.data.source, &f))
                    .collect::<Vec<_>>();
                if offset > 0 && offset as usize >= matching.len() {
                    return Err(store::invalid("cursor_scope_mismatch"));
                }
                let items = matching
                    .iter()
                    .skip(offset as usize)
                    .take(limit as usize)
                    .map(|e| e.entry.clone())
                    .collect::<Vec<_>>();
                let after = offset + items.len() as u32;
                let next_cursor = if after < (matching.len() as u32) {
                    Some(shape.encode(after)?)
                } else {
                    None
                };
                AttachmentIndexResult::Get(AttachmentIndexPage {
                    version: 1,
                    profile: ATTACHMENT_INDEX_PROFILE.into(),
                    revision,
                    coverage,
                    items,
                    next_cursor,
                })
            }
            AttachmentIndexInput::Put {
                job_id,
                expected_generation_id,
                expected_revision,
            } => put(
                &c,
                &root,
                Target {
                    job: &job_id,
                    generation: &expected_generation_id,
                    revision: expected_revision,
                },
                &b,
                hook,
                scope,
            )?,
            AttachmentIndexInput::Reset { expected_revision } => {
                reset(&c, expected_revision, &b, hook, scope)?
            }
        };
        if !write {
            store::response(&out)?;
            b.check()?;
        }
        Ok(out)
    })();
    result.map_err(|e| b.map(e))
}

struct Target<'a> {
    job: &'a str,
    generation: &'a str,
    revision: u32,
}
fn expected(c: &Connection, t: &Target<'_>) -> Result<()> {
    let revision = index::control(c)?.revision;
    if revision != t.revision {
        return Err(index::conflict("expected_revision_mismatch", revision));
    }
    let generation: Option<String> = c
        .query_row(
            "SELECT generation_id FROM extraction_jobs WHERE id=?1",
            [t.job],
            |r| r.get(0),
        )
        .optional()?
        .ok_or_else(|| store::error(ErrorCode::NotFound, "job_not_found"))?;
    match generation {
        None => Err(store::error(ErrorCode::NotFound, "generation_not_found")),
        Some(id) if id != t.generation => Err(index::conflict("generation_changed", revision)),
        _ => Ok(()),
    }
}

struct PageLimit<'a> {
    c: &'a Connection,
    old: u32,
}
impl<'a> PageLimit<'a> {
    fn new(c: &'a Connection) -> Result<Self> {
        let old: u32 = c.pragma_query_value(None, "max_page_count", |r| r.get(0))?;
        let page_size = batch_storage::size(c)?;
        let limit = old.min((batch_storage::MAX_SNAPSHOT_BYTES / page_size) as u32);
        c.pragma_update(None, "max_page_count", limit)?;
        let actual: u32 = c.pragma_query_value(None, "max_page_count", |r| r.get(0))?;
        if actual != limit {
            c.pragma_update(None, "max_page_count", old)?;
            return Err(store::invalid("snapshot_size_limit"));
        }
        Ok(Self { c, old })
    }
}
impl Drop for PageLimit<'_> {
    fn drop(&mut self) {
        let _ = self.c.pragma_update(None, "max_page_count", self.old);
    }
}
fn write_error(e: rusqlite::Error) -> CoreError {
    if matches!(&e,rusqlite::Error::SqliteFailure(s,_) if s.code==rusqlite::ErrorCode::DiskFull) {
        return index::error("storage_full_or_index_size_limit");
    }
    e.into()
}
fn put(
    c: &Connection,
    root: &Path,
    t: Target<'_>,
    b: &Budget<'_>,
    hook: &mut dyn FnMut(&str) -> Result<()>,
    scope: Option<&crate::scoped::LocalScope>,
) -> Result<AttachmentIndexResult> {
    let prepared = {
        let tx = c.unchecked_transaction()?;
        if let Some(scope) = scope {
            scope.check(&tx)?;
        }
        expected(&tx, &t)?;
        let data = index::canonical(&tx, t.job, b)?;
        index::source(&tx, root, &data, b)?;
        let _verified = index::verify(&tx, b)?;
        data
    };
    hook("prepared")?;
    b.check()?;
    // Declarado antes de tx: también en error/unwind el rollback debe ocurrir
    // ANTES de restaurar max_page_count. Inicializado sólo al escribir filas.
    let _pages;
    let tx = store::immediate(c)?;
    hook("locked")?;
    if let Some(scope) = scope {
        scope.check(&tx)?;
    }
    expected(&tx, &t)?;
    index::structure(&tx)?;
    let current = index::canonical(&tx, t.job, b)?;
    if current.hash != prepared.hash {
        return Err(index::conflict("generation_changed", t.revision));
    }
    index::source(&tx, root, &current, b)?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT indexed_at FROM extraction_search_entries WHERE job_id=?1",
            [t.job],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(at) = existing {
        let out = AttachmentIndexResult::Put {
            revision: t.revision,
            entry: current.entry(at),
            changed: false,
        };
        store::response(&out)?;
        b.check()?;
        hook("before_commit")?;
        b.check()?;
        tx.commit().map_err(write_error)?;
        hook("committed")?;
        return Ok(out);
    }
    _pages = PageLimit::new(c)?;
    let (entries,fragments,bytes):(u32,u32,u32)=tx.query_row("SELECT (SELECT count(*) FROM extraction_search_entries),count(*),coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_search_fts",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
    if entries >= MAX_INDEX_ENTRIES {
        return Err(store::invalid("index_entry_limit"));
    }
    if fragments + current.fragments.len() as u32 > MAX_INDEX_FRAGMENTS {
        return Err(store::invalid("index_fragment_limit"));
    }
    if u64::from(bytes) + u64::from(current.text_bytes) > MAX_INDEX_TEXT_BYTES {
        return Err(store::invalid("index_text_size_limit"));
    }
    let used = tx
        .prepare("SELECT slot FROM extraction_search_entries")?
        .query_map([], |r| r.get::<_, u32>(0))?
        .collect::<std::result::Result<std::collections::HashSet<_>, _>>()?;
    let slot = (1..=MAX_INDEX_ENTRIES)
        .find(|s| !used.contains(s))
        .ok_or_else(|| store::invalid("index_entry_limit"))?;
    let at = crate::workspace::now();
    tx.execute("INSERT INTO extraction_search_entries(slot,job_id,generation_id,fragment_count,text_bytes,content_hash,indexed_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![slot,t.job,t.generation,current.fragments.len() as u32,current.text_bytes,current.hash,at]).map_err(write_error)?;
    hook("entry_inserted")?;
    {
        let mut insert=tx.prepare("INSERT INTO extraction_search_fts(rowid,job_id,generation_id,ordinal,text) VALUES (?1,?2,?3,?4,?5)")?;
        for f in &current.fragments {
            b.check()?;
            insert
                .execute(params![
                    (slot - 1) * 512 + f.ordinal + 1,
                    t.job,
                    t.generation,
                    f.ordinal,
                    f.text
                ])
                .map_err(write_error)?;
            hook("fragment_inserted")?;
        }
    }
    let revision = store::bump(t.revision)?;
    tx.execute(
        "UPDATE extraction_search_control SET revision=?1,updated_at=?2 WHERE id=1",
        params![revision, at],
    )?;
    let out = AttachmentIndexResult::Put {
        revision,
        entry: current.entry(at),
        changed: true,
    };
    store::response(&out)?;
    batch_storage::size(&tx)?;
    b.check()?;
    hook("before_commit")?;
    b.check()?;
    tx.commit().map_err(write_error)?;
    hook("committed")?;
    Ok(out)
}

fn reset(
    c: &Connection,
    expected_revision: u32,
    b: &Budget<'_>,
    hook: &mut dyn FnMut(&str) -> Result<()>,
    scope: Option<&crate::scoped::LocalScope>,
) -> Result<AttachmentIndexResult> {
    let tx = store::immediate(c)?;
    hook("locked")?;
    if let Some(scope) = scope {
        scope.check(&tx)?;
    }
    let current = index::control(&tx)?.revision;
    if current != expected_revision {
        return Err(index::conflict("expected_revision_mismatch", current));
    }
    index::structure(&tx)?;
    let revision = store::bump(current)?;
    let (generations,fragments):(u32,u32)=tx.query_row("SELECT (SELECT count(*) FROM extraction_search_entries),(SELECT count(*) FROM extraction_search_fts)",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
    tx.execute_batch("DROP TRIGGER extraction_search_delete;DELETE FROM extraction_search_entries;DROP TABLE extraction_search_fts;")?;
    hook("reset_dropped")?;
    b.check()?;
    tx.execute_batch(index::reset_ddl())?;
    tx.execute(
        "UPDATE extraction_search_control SET revision=?1,updated_at=?2 WHERE id=1",
        params![revision, crate::workspace::now()],
    )?;
    index::structure(&tx)?;
    let out = AttachmentIndexResult::Reset {
        revision,
        removed_generations: generations,
        removed_fragments: fragments,
    };
    store::response(&out)?;
    b.check()?;
    hook("before_commit")?;
    b.check()?;
    tx.commit().map_err(write_error)?;
    hook("committed")?;
    Ok(out)
}

use rusqlite::OptionalExtension;

#[derive(Debug, PartialEq)]
struct Candidate {
    rowid: u32,
    job: String,
    generation: String,
    ordinal: u32,
    score: f64,
}
fn candidates(
    c: &Connection,
    expected: bool,
    expression: &str,
    f: &crate::attachment_search_contracts::IndexFilter,
    limit: u32,
    offset: u32,
) -> Result<Vec<Candidate>> {
    let (table, qualified) = if expected {
        ("attachment_expected_fts", "temp.attachment_expected_fts")
    } else {
        ("extraction_search_fts", "main.extraction_search_fts")
    };
    // Identificadores fijos internos; sólo valores del solicitante son parámetros.
    let sql = format!(
        "SELECT {table}.rowid,{table}.job_id,{table}.generation_id,{table}.ordinal,bm25({table}) FROM {qualified} JOIN extraction_generations g ON g.id={table}.generation_id WHERE {table} MATCH ?1 AND (?2 IS NULL OR {table}.job_id=?2) AND (?3 IS NULL OR g.attachment_id=?3) AND (?4 IS NULL OR g.extractor=?4) ORDER BY bm25({table}),{table}.job_id COLLATE BINARY,{table}.generation_id COLLATE BINARY,CAST({table}.ordinal AS INTEGER) LIMIT ?5 OFFSET ?6"
    );
    Ok(c.prepare(&sql)?
        .query_map(
            params![
                expression,
                f.job_id,
                f.attachment_id,
                f.extractor.map(IndexedExtractor::as_str),
                limit,
                offset
            ],
            |r| {
                Ok(Candidate {
                    rowid: r.get(0)?,
                    job: r.get(1)?,
                    generation: r.get(2)?,
                    ordinal: r.get(3)?,
                    score: r.get(4)?,
                })
            },
        )?
        .collect::<std::result::Result<_, _>>()?)
}
pub(crate) fn search(path: &Path, input: AttachmentSearchInput) -> Result<AttachmentSearchPage> {
    search_scoped(path, input, None)
}
pub(crate) fn search_scoped(
    path: &Path,
    input: AttachmentSearchInput,
    scope: Option<&crate::scoped::LocalScope>,
) -> Result<AttachmentSearchPage> {
    if input.query.len() > 256 {
        return Err(store::invalid("search_query_limit"));
    }
    let terms = input.query.split_whitespace().collect::<Vec<_>>();
    if terms.is_empty()
        || terms.len() > 16
        || input.query.len() > 256
        || input
            .query
            .chars()
            .any(|c| c.is_control() && !c.is_whitespace())
    {
        return Err(store::invalid("search_query_limit"));
    }
    let expression = terms
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ");
    let f = filter(input.job_id, input.attachment_id, input.extractor)?;
    let limit = store::page_limit(input.limit, 20, 64)?;
    if input.cursor.as_ref().is_some_and(|s| s.len() > 2048) {
        return Err(store::invalid("cursor_size_limit"));
    }
    let (c, root) = index::open(path, false)?;
    if let Some(scope) = scope {
        scope.check(&c)?;
    }
    let b = Budget::new(&c)?;
    let result = (|| {
        let tx = c.unchecked_transaction()?;
        if let Some(scope) = scope {
            scope.check(&tx)?;
        }
        let revision = index::control(&tx)?.revision;
        let shape = index::Cursor::new(
            &tx,
            &root,
            "extraction.search",
            &input.query,
            f.clone(),
            limit,
            revision,
        )?;
        let offset = shape.offset(input.cursor.as_deref(), MAX_INDEX_FRAGMENTS)?;
        let verified = index::verify(&tx, &b)?;
        let coverage = index::coverage(
            &tx,
            &f,
            &verified.entries,
            IndexSourceValidation::ReturnedCandidates,
            &b,
        )?;
        let query = |expected| {
            candidates(&tx, expected, &expression, &f, limit + 1, offset).map_err(|e| {
                let e = b.map(e);
                if matches!(e.code, ErrorCode::WorkspaceBusy) {
                    return e;
                }
                let mut e = index::error("invalid_attachment_index");
                e.details["current_revision"] = serde_json::json!(revision);
                e
            })
        };
        let mut raw = query(false)?;
        let expected = query(true)?;
        if raw != expected || raw.iter().any(|r| !r.score.is_finite()) {
            let mut e = index::error("invalid_attachment_index");
            e.details["current_revision"] = serde_json::json!(revision);
            return Err(e);
        }
        if offset > 0 && raw.is_empty() {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        let mut checked = std::collections::HashSet::new();
        for r in &raw {
            b.check()?;
            let selected = verified
                .entries
                .iter()
                .find(|s| {
                    s.data.source.generation_id == r.generation && s.data.source.job_id == r.job
                })
                .ok_or_else(|| index::error("invalid_attachment_index"))?;
            if r.rowid != (selected.slot - 1) * 512 + r.ordinal + 1 {
                return Err(index::error("invalid_attachment_index"));
            }
            if checked.insert(&r.generation) {
                index::source(&tx, &root, &selected.data, &b)?;
            }
        }
        let next_cursor = if raw.len() > limit as usize {
            raw.pop();
            Some(shape.encode(offset + limit)?)
        } else {
            None
        };
        let mut items = Vec::with_capacity(raw.len());
        for r in raw {
            let selected = verified
                .entries
                .iter()
                .find(|s| s.data.source.generation_id == r.generation)
                .ok_or_else(|| index::error("invalid_attachment_index"))?;
            let part = selected
                .data
                .fragments
                .get(r.ordinal as usize)
                .ok_or_else(|| index::error("invalid_attachment_index"))?;
            items.push(AttachmentSearchHit {
                source: selected.data.source.clone(),
                ordinal: part.ordinal,
                text: part.text.clone(),
                locator: part.locator.clone(),
                score: r.score,
                result_from_previous_attempt: selected.data.previous,
            });
        }
        let out = AttachmentSearchPage {
            version: 1,
            profile: ATTACHMENT_INDEX_PROFILE.into(),
            revision,
            coverage,
            items,
            next_cursor,
        };
        store::response(&out)?;
        b.check()?;
        Ok(out)
    })();
    result.map_err(|e| b.map(e))
}
