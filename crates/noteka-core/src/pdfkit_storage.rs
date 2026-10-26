//! Persistencia canónica de páginas; nunca interpreta los chunks JSON como texto FTS.
use crate::{contracts::*, error::*, extraction_storage as store};
use noteka_pdfkit_protocol::{Document, PROFILE};
use noteka_pdfkit_runtime::canonical;
use rusqlite::{Connection, params};

pub(crate) struct Prepared {
    pub manifest: PdfkitManifest,
    pub chunks: Vec<String>,
    pub document: Document,
}
pub(crate) fn prepare(document: Document) -> Result<Prepared> {
    let p = canonical::prepare(document).map_err(|e| match e {
        canonical::Invalid::RepresentationLimit => store::invalid("pdfkit_result_limit"),
        _ => store::storage("pdfkit_worker_protocol"),
    })?;
    Ok(Prepared {
        manifest: PdfkitManifest {
            version: 1,
            profile: PROFILE.into(),
            representation_sha256: p.representation_sha256,
            representation_bytes: p.representation_bytes,
            engine: p.document.engine.clone(),
            limitations: p.document.limitations.clone(),
            page_count: p.document.page_count,
            text_bytes: p.document.text_bytes,
        },
        chunks: p.chunks,
        document: p.document,
    })
}
pub(crate) fn publish(c: &Connection, generation: &str, job: &str, p: &Prepared) -> Result<()> {
    let m = &p.manifest;
    c.execute(
        "INSERT INTO extraction_pdfkit_generations VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            generation,
            job,
            m.version,
            m.profile,
            m.representation_sha256,
            m.representation_bytes,
            m.page_count,
            m.text_bytes,
            p.chunks.len() as u32
        ],
    )?;
    let mut s = c.prepare("INSERT INTO extraction_pdfkit_chunks VALUES (?1,?2,?3,?4)")?;
    for (ordinal, text) in p.chunks.iter().enumerate() {
        s.execute(params![generation, job, ordinal as u32, text])?;
    }
    Ok(())
}
fn corrupt() -> CoreError {
    store::storage("invalid_pdfkit_integrity")
}
pub(crate) fn load(c: &Connection, generation: &str, job: &str, count: u32) -> Result<Prepared> {
    crate::pdf_ocr_storage::no_owner(c, generation)?;
    // Acotar antes de materializar y rechazar propietarios o formatos cruzados.
    let (chunks,bytes,bad,cross):(u32,u32,u32,bool)=c.query_row("SELECT (SELECT count(*) FROM extraction_pdfkit_chunks WHERE generation_id=?1),(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdfkit_chunks WHERE generation_id=?1),(SELECT count(*) FROM extraction_pdfkit_chunks WHERE generation_id=?1 AND (length(CAST(text AS BLOB)) NOT BETWEEN 1 AND 4096 OR job_id!=?2)),EXISTS(SELECT 1 FROM extraction_fragments WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_pdf_generations WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_xlsx_generations WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_ocr_generations WHERE generation_id=?1)",params![generation,job],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
    if chunks != count
        || !(1..=512).contains(&chunks)
        || !(1..=1048576).contains(&bytes)
        || bad != 0
        || cross
    {
        return Err(corrupt());
    }
    let (owner,version,profile,hash,size,pages,text,declared):(bool,u32,bool,String,u32,u32,u32,u32)=c.query_row("SELECT job_id=?2,version,profile=?3,CASE WHEN length(CAST(representation_sha256 AS BLOB))=64 THEN representation_sha256 ELSE '' END,representation_bytes,page_count,text_bytes,chunk_count FROM extraction_pdfkit_generations WHERE generation_id=?1",params![generation,job,PROFILE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?)))?;
    if !owner
        || version != 1
        || !profile
        || size != bytes
        || declared != chunks
        || !(1..=100).contains(&pages)
        || text > 1048576
    {
        return Err(corrupt());
    }
    let mut stored = Vec::with_capacity(chunks as usize);
    let mut s = c.prepare(
        "SELECT ordinal,text FROM extraction_pdfkit_chunks WHERE generation_id=?1 ORDER BY ordinal",
    )?;
    let mut rows = s.query([generation])?;
    while let Some(r) = rows.next()? {
        let ordinal: u32 = r.get(0)?;
        if ordinal as usize != stored.len() {
            return Err(corrupt());
        }
        stored.push(r.get::<_, String>(1)?);
    }
    let p = canonical::load(&stored, &hash, size).map_err(|_| corrupt())?;
    if p.document.page_count != pages || p.document.text_bytes != text {
        return Err(corrupt());
    }
    prepare(p.document).map_err(|_| corrupt())
}
pub(crate) fn shape(c: &Connection) -> Result<()> {
    if c.prepare("SELECT 1 FROM extraction_generations g WHERE (g.extractor='pdfkit_text') != EXISTS(SELECT 1 FROM extraction_pdfkit_generations x WHERE x.generation_id=g.id) UNION ALL SELECT 1 FROM extraction_pdfkit_generations x LEFT JOIN extraction_generations g ON g.id=x.generation_id WHERE g.id IS NULL OR x.job_id!=g.job_id OR x.chunk_count!=g.fragment_count OR x.chunk_count!=(SELECT count(*) FROM extraction_pdfkit_chunks WHERE generation_id=x.generation_id) OR x.representation_bytes!=(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdfkit_chunks WHERE generation_id=x.generation_id) UNION ALL SELECT 1 FROM extraction_pdfkit_chunks x LEFT JOIN extraction_pdfkit_generations g ON g.generation_id=x.generation_id WHERE g.generation_id IS NULL OR x.job_id!=g.job_id LIMIT 1")?.exists([])? { return Err(corrupt()); }
    Ok(())
}
pub(crate) fn integrity(c: &Connection) -> Result<()> {
    shape(c)?;
    let ids:Vec<(String,String,u32)>=c.prepare("SELECT id,job_id,fragment_count FROM extraction_generations WHERE extractor='pdfkit_text'")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<std::result::Result<_,_>>()?;
    for (id, job, count) in ids {
        load(c, &id, &job, count)?;
        let g = store::generation(c, &id)?;
        let a = crate::attachments::metadata(c, &g.attachment_id)?;
        store::eligible_for(&a, "pdfkit_text")?;
    }
    Ok(())
}
pub(crate) fn retained(c: &Connection, except: &str) -> Result<u64> {
    let prior = crate::ocr_storage::retained(c, except)?;
    let x:i64=c.query_row("SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdfkit_chunks WHERE job_id!=?1",[except],|r|r.get(0))?;
    prior
        .checked_add(u64::try_from(x).map_err(|_| store::storage("invalid_derived_budget"))?)
        .ok_or_else(|| store::storage("invalid_derived_budget"))
}

impl crate::workspace::Workspace {
    pub fn extraction_pdfkit_pages(&self, input: ReadPdfkitPages) -> Result<PdfkitPagePage> {
        let limit = input.validate()?;
        let tx = self.conn.unchecked_transaction()?;
        let j = store::job(&tx, &self.path, &input.id)?;
        let g = j
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if g.extractor != "pdfkit_text"
            || !store::supported(&g.extractor, g.extractor_version, &g.config)
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut shape = store::Cursor::new(self, "extraction.pdfkit_pages", limit)?;
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
        store::eligible_for(&a, "pdfkit_text")?;
        if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &a, || Ok(()))?;
        let all = load(&tx, &g.id, &input.id, g.fragment_count)?;
        store::eligible_for(&a, "pdfkit_text")?;
        if cursor.is_some() && from >= all.manifest.page_count {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        let items = all
            .document
            .pages
            .into_iter()
            .skip(from as usize)
            .take(limit as usize)
            .collect::<Vec<_>>();
        let after = from + items.len() as u32;
        let next_cursor = if after < all.manifest.page_count {
            shape.ordinal = Some(after);
            Some(shape.encode()?)
        } else {
            None
        };
        let out = PdfkitPagePage {
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
