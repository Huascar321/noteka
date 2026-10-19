//! JSON canónico versionado, persistido una vez; proyecciones públicas derivadas.
use crate::{contracts::*, error::*, extraction_storage as store};
use noteka_xlsx_protocol::{PARSER, Workbook};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

pub(crate) struct Prepared {
    pub manifest: XlsxManifest,
    pub chunks: Vec<String>,
    pub workbook: Workbook,
}
pub(crate) fn prepare(workbook: Workbook) -> Result<Prepared> {
    workbook
        .validate()
        .map_err(|_| store::storage("xlsx_worker_protocol"))?;
    let raw = serde_json::to_string(&workbook)?;
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < raw.len() {
        let mut end = (start + 4096).min(raw.len());
        while !raw.is_char_boundary(end) {
            end -= 1;
        }
        chunks.push(raw[start..end].to_owned());
        start = end;
    }
    if raw.is_empty() || raw.len() > 1_048_576 || chunks.len() > 512 {
        return Err(store::invalid("xlsx_result_limit"));
    }
    let sheets = workbook
        .sheets
        .iter()
        .map(|s| XlsxSheetDescriptor {
            id: s.id,
            name: s.name.clone(),
            visibility: s.visibility,
            declared_range: s.declared_range.clone(),
            observed_range: s.observed_range.clone(),
            merges: s.merges.clone(),
            cell_count: s.cells.len() as u32,
        })
        .collect::<Vec<_>>();
    let manifest = XlsxManifest {
        version: 1,
        parser: PARSER.into(),
        representation_sha256: format!("{:x}", Sha256::digest(raw.as_bytes())),
        representation_bytes: raw.len() as u32,
        date_1904: workbook.date_1904,
        limitations: workbook.limitations.clone(),
        styles: workbook.styles.clone(),
        cell_count: sheets.iter().map(|s| s.cell_count).sum(),
        sheets,
    };
    Ok(Prepared {
        manifest,
        chunks,
        workbook,
    })
}
pub(crate) fn publish(c: &Connection, generation: &str, job: &str, p: &Prepared) -> Result<()> {
    let m = &p.manifest;
    c.execute(
        "INSERT INTO extraction_xlsx_generations VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            generation,
            job,
            m.version,
            m.parser,
            m.representation_sha256,
            m.representation_bytes,
            m.sheets.len() as u32,
            m.cell_count,
            p.chunks.len() as u32
        ],
    )?;
    let mut s = c.prepare("INSERT INTO extraction_xlsx_chunks VALUES (?1,?2,?3,?4)")?;
    for (ordinal, text) in p.chunks.iter().enumerate() {
        s.execute(params![generation, job, ordinal as u32, text])?;
    }
    Ok(())
}
fn corrupt() -> CoreError {
    store::storage("invalid_xlsx_integrity")
}
pub(crate) fn load(c: &Connection, generation: &str, job: &str, count: u32) -> Result<Prepared> {
    // No materializar texto antes de comprobar presupuestos y cruces de formato.
    let (chunks,bytes,oversized,literal,pdf):(u32,u32,u32,u32,u32)=c.query_row("SELECT (SELECT count(*) FROM extraction_xlsx_chunks WHERE generation_id=?1),(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_xlsx_chunks WHERE generation_id=?1),(SELECT count(*) FROM extraction_xlsx_chunks WHERE generation_id=?1 AND length(CAST(text AS BLOB)) NOT BETWEEN 1 AND 4096),(SELECT count(*) FROM extraction_fragments WHERE generation_id=?1),(SELECT count(*) FROM extraction_pdf_generations WHERE generation_id=?1)",[generation],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
    if chunks != count
        || !(1..=512).contains(&chunks)
        || !(1..=1_048_576).contains(&bytes)
        || oversized != 0
        || literal != 0
        || pdf != 0
    {
        return Err(corrupt());
    }
    // Comparar propietario/parser dentro de SQL evita materializar metadatos
    // arbitrariamente grandes de una base alterada. El hash cargado queda <=64 B.
    let (owner,version,parser,hash,size,sheets,cells,declared):(bool,u32,bool,String,u32,u32,u32,u32)=c.query_row("SELECT job_id=?2,version,parser=?3,CASE WHEN length(CAST(representation_sha256 AS BLOB))=64 THEN representation_sha256 ELSE '' END,representation_bytes,sheet_count,cell_count,chunk_count FROM extraction_xlsx_generations WHERE generation_id=?1",params![generation,job,PARSER],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?)))?;
    if !owner
        || version != 1
        || !parser
        || hash.len() != 64
        || size != bytes
        || declared != chunks
        || !(1..=32).contains(&sheets)
        || cells > 10_000
    {
        return Err(corrupt());
    }
    let mut raw = String::with_capacity(size as usize);
    let mut stmt=c.prepare("SELECT job_id,ordinal,text FROM extraction_xlsx_chunks WHERE generation_id=?1 ORDER BY ordinal")?;
    let mut rows = stmt.query([generation])?;
    let mut expected = 0;
    while let Some(r) = rows.next()? {
        let owner: String = r.get(0)?;
        let ordinal: u32 = r.get(1)?;
        let text: String = r.get(2)?;
        if owner != job
            || ordinal != expected
            || text.is_empty()
            || text.len() > 4096
            || raw.len() + text.len() > size as usize
        {
            return Err(corrupt());
        }
        raw.push_str(&text);
        expected += 1;
    }
    if expected != chunks
        || raw.len() != size as usize
        || format!("{:x}", Sha256::digest(raw.as_bytes())) != hash
    {
        return Err(corrupt());
    }
    let workbook: Workbook = serde_json::from_str(&raw).map_err(|_| corrupt())?;
    let prepared = prepare(workbook).map_err(|_| corrupt())?;
    // Impide codificaciones JSON no canónicas y particiones ambiguas sin cambiar versión.
    if prepared.chunks.concat() != raw
        || prepared.chunks.len() != chunks as usize
        || prepared.manifest.sheets.len() != sheets as usize
        || prepared.manifest.cell_count != cells
    {
        return Err(corrupt());
    }
    let stored: Vec<String> = c
        .prepare("SELECT text FROM extraction_xlsx_chunks WHERE generation_id=?1 ORDER BY ordinal")?
        .query_map([generation], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    if stored != prepared.chunks {
        return Err(corrupt());
    }
    Ok(prepared)
}
pub(crate) fn shape(c: &Connection) -> Result<()> {
    if c.prepare("SELECT 1 FROM extraction_generations g WHERE (g.extractor='xlsx_cells') != EXISTS(SELECT 1 FROM extraction_xlsx_generations x WHERE x.generation_id=g.id) UNION ALL SELECT 1 FROM extraction_xlsx_generations x LEFT JOIN extraction_generations g ON g.id=x.generation_id WHERE g.id IS NULL OR x.job_id!=g.job_id OR x.chunk_count!=g.fragment_count OR x.chunk_count!=(SELECT count(*) FROM extraction_xlsx_chunks WHERE generation_id=x.generation_id) OR x.representation_bytes!=(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_xlsx_chunks WHERE generation_id=x.generation_id) UNION ALL SELECT 1 FROM extraction_xlsx_chunks x LEFT JOIN extraction_xlsx_generations g ON g.generation_id=x.generation_id WHERE g.generation_id IS NULL OR x.job_id!=g.job_id LIMIT 1")?.exists([])? { return Err(corrupt()); }
    Ok(())
}
pub(crate) fn integrity(c: &Connection) -> Result<()> {
    shape(c)?;
    let ids:Vec<(String,String,u32)>=c.prepare("SELECT id,job_id,fragment_count FROM extraction_generations WHERE extractor='xlsx_cells'")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<std::result::Result<_,_>>()?;
    for (id, job, count) in ids {
        load(c, &id, &job, count)?;
    }
    Ok(())
}
pub(crate) fn retained(c: &Connection, except: &str) -> Result<u64> {
    let prior = crate::pdf_storage::retained(c, except)?;
    let x:i64=c.query_row("SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_xlsx_chunks WHERE job_id!=?1",[except],|r|r.get(0))?;
    prior
        .checked_add(u64::try_from(x).map_err(|_| store::storage("invalid_derived_budget"))?)
        .ok_or_else(|| store::storage("invalid_derived_budget"))
}

impl crate::workspace::Workspace {
    pub fn extraction_xlsx_cells(&self, input: ReadXlsxCells) -> Result<XlsxCellPage> {
        let limit = input.validate()?;
        let tx = self.conn.unchecked_transaction()?;
        let j = store::job(&tx, &self.path, &input.id)?;
        let g = j
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if g.extractor != "xlsx_cells"
            || !store::supported(&g.extractor, g.extractor_version, &g.config)
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut shape = store::Cursor::new(self, "extraction.xlsx_cells", limit)?;
        shape.job_id = Some(input.id.clone());
        shape.generation_id = Some(g.id.clone());
        let cursor = input
            .cursor
            .as_ref()
            .map(|s| store::Cursor::read(s))
            .transpose()?;
        let from = if let Some(c) = &cursor {
            let mut blank = c.clone();
            blank.ordinal = None;
            if blank != shape || c.ordinal.is_none_or(|n| n == 0) {
                return Err(store::invalid("cursor_scope_mismatch"));
            }
            c.ordinal.unwrap()
        } else {
            0
        };
        let a = crate::attachments::metadata(&tx, &g.attachment_id)?;
        store::eligible_for(&a, "xlsx_cells")?;
        if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &a, || Ok(()))?;
        let all = load(&tx, &g.id, &input.id, g.fragment_count)?;
        if cursor.is_some() && from >= all.manifest.cell_count {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        let items = all
            .workbook
            .sheets
            .into_iter()
            .flat_map(|s| s.cells.into_iter().map(move |cell| (s.id, cell)))
            .enumerate()
            .skip(from as usize)
            .take(limit as usize)
            .map(|(i, (sheet_id, cell))| XlsxCellItem {
                ordinal: i as u32,
                locator: XlsxCellLocator {
                    kind: XlsxLocatorKind::XlsxCell,
                    sheet_id,
                    reference: cell.reference.clone(),
                },
                cell,
            })
            .collect::<Vec<_>>();
        let after = from + items.len() as u32;
        let next_cursor = if after < all.manifest.cell_count {
            shape.ordinal = Some(after);
            Some(shape.encode()?)
        } else {
            None
        };
        let out = XlsxCellPage {
            job_id: input.id,
            generation: g,
            result_from_previous_attempt: j.result_from_previous_attempt,
            manifest: all.manifest,
            items,
            covers_generation: input.cursor.is_none() && next_cursor.is_none(),
            next_cursor,
        };
        store::response(&out)?;
        Ok(out)
    }
}
