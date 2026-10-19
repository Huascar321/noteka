use crate::{contracts::*, entities::get, error::*, workspace::Workspace};
use rusqlite::{Connection, TransactionBehavior, params};
use serde_json::json;

pub(crate) fn reindex(conn: &Connection, e: &Entity) -> Result<u32> {
    conn.execute("DELETE FROM search_fts WHERE entity_id=?1", [&e.id])?;
    conn.execute(
        "INSERT INTO search_fts(entity_id,block_id,revision,title,text) VALUES (?1,NULL,?2,?3,'')",
        params![e.id, e.revision, e.title],
    )?;
    let mut count = 1;
    fn walk(conn: &Connection, e: &Entity, b: &Block, count: &mut u32) -> Result<()> {
        let text = b
            .content
            .iter()
            .map(|t| {
                if matches!(t.r#type, TextKind::HardBreak) {
                    "\n"
                } else {
                    t.text.as_str()
                }
            })
            .collect::<String>();
        if !text.trim().is_empty() {
            conn.execute("INSERT INTO search_fts(entity_id,block_id,revision,title,text) VALUES (?1,?2,?3,'',?4)",params![e.id,b.id,e.revision,text])?;
            *count += 1;
        }
        for child in &b.children {
            walk(conn, e, child, count)?;
        }
        Ok(())
    }
    if e.document_editable
        && let Some(ref d) = e.document
    {
        let d: Document = serde_json::from_value(d.clone())?;
        for b in &d.blocks {
            walk(conn, e, b, &mut count)?;
        }
    }
    Ok(count)
}
pub(crate) fn rebuild(conn: &Connection) -> Result<SearchIndexInfo> {
    conn.execute("DELETE FROM search_fts", [])?;
    let mut stmt = conn.prepare("SELECT id FROM entities ORDER BY id")?;
    let ids = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut count = 0;
    for id in &ids {
        count += reindex(conn, &get(conn, id)?)?;
    }
    Ok(SearchIndexInfo {
        entities: ids.len() as u32,
        chunks: count,
    })
}
impl Workspace {
    pub fn search_rebuild(&mut self) -> Result<SearchIndexInfo> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = rebuild(&tx)?;
        tx.commit()?;
        Ok(result)
    }
    pub fn search_query(&self, input: SearchQuery) -> Result<SearchPage> {
        let terms = input.query.split_whitespace().collect::<Vec<_>>();
        if terms.is_empty()
            || terms.len() > 16
            || input.query.len() > 256
            || input
                .query
                .chars()
                .any(|c| c.is_control() && !c.is_whitespace())
        {
            return Err(CoreError::validation(
                "Búsqueda: 1–16 términos y hasta 256 bytes.",
            ));
        }
        let expression = terms
            .iter()
            .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" AND ");
        let n = input.limit.unwrap_or(30);
        if !(1..=100).contains(&n) {
            return Err(CoreError::validation("limit de búsqueda debe ser 1–100."));
        }
        let offset: u32 = input
            .cursor
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?
            .unwrap_or(0);
        if offset > 100_000 {
            return Err(CoreError::validation("Cursor de búsqueda supera 100.000."));
        }
        let tx = self.conn.unchecked_transaction()?;
        if let Some(ref id) = input.related_id {
            let e = get(&tx, id)?;
            if !matches!(
                e.kind,
                EntityKind::Project | EntityKind::Tag | EntityKind::Person
            ) {
                return Err(CoreError::validation(
                    "Filtro contextual debe ser proyecto, tag o persona.",
                ));
            }
        }
        let kind = input.kind.as_ref().map(|k| {
            serde_json::to_value(k)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        });
        let mut stmt=tx.prepare("SELECT e.id,e.title,e.kind,e.revision,search_fts.block_id,CASE WHEN search_fts.block_id IS NULL THEN e.title ELSE snippet(search_fts,4,'','',' … ',24) END,bm25(search_fts) FROM search_fts JOIN entities e ON e.id=search_fts.entity_id WHERE search_fts MATCH ?1 AND CAST(search_fts.revision AS INTEGER)=e.revision AND (e.archived_at IS NOT NULL)=?2 AND (?3 IS NULL OR e.kind=?3) AND (?4 IS NULL OR EXISTS(SELECT 1 FROM relations r WHERE (r.source_id=e.id AND r.target_id=?4) OR (r.target_id=e.id AND r.source_id=?4))) ORDER BY bm25(search_fts),e.id,search_fts.block_id LIMIT ?5 OFFSET ?6")?;
        let raw = stmt
            .query_map(
                params![
                    expression,
                    input.archived,
                    kind,
                    input.related_id,
                    n + 1,
                    offset
                ],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, u32>(3)?,
                        r.get::<_, Option<String>>(4)?,
                        r.get::<_, String>(5)?,
                        r.get::<_, f64>(6)?,
                    ))
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut items = raw
            .into_iter()
            .map(
                |(entity_id, title, kind, revision, block_id, excerpt, score)| {
                    Ok(SearchHit {
                        entity_id,
                        title,
                        kind: serde_json::from_value(json!(kind))?,
                        revision,
                        block_id,
                        excerpt,
                        score,
                    })
                },
            )
            .collect::<Result<Vec<_>>>()?;
        let next_cursor = if items.len() > n as usize {
            items.pop();
            Some(json!(offset + n).to_string())
        } else {
            None
        };
        Ok(SearchPage { items, next_cursor })
    }
}
