//! Canónico PDF OCR completo; ningún PNG efímero se guarda como original.
use crate::{
    contracts::*,
    error::*,
    extraction_storage as store,
    pdf_ocr_compositor::{Document, PROFILE},
};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
pub(crate) struct Prepared {
    pub manifest: PdfOcrManifest,
    pub chunks: Vec<String>,
    pub document: Document,
}
pub(crate) fn prepare(document: Document) -> Result<Prepared> {
    document.validate()?;
    let raw = serde_json::to_string(&document)?;
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
    if chunks.is_empty() || chunks.len() > 512 || raw.len() > 1048576 {
        return Err(store::invalid("pdf_ocr_result_limit"));
    }
    let pages: Vec<_> = document
        .pages
        .iter()
        .map(|p| PdfOcrPageDescriptor {
            raster: p.raster.clone(),
            engine: p.recognition.engine.clone(),
            coordinate_space: p.recognition.coordinate_space.clone(),
            limitations: p.recognition.limitations.clone(),
            region_count: p.recognition.regions.len() as u32,
            text_bytes: p
                .recognition
                .regions
                .iter()
                .map(|r| r.text.len() as u32)
                .sum(),
        })
        .collect();
    let manifest = PdfOcrManifest {
        version: 1,
        profile: PROFILE.into(),
        raster_profile: document.raster_profile.clone(),
        source_sha256: document.source_sha256.clone(),
        representation_sha256: format!("{:x}", Sha256::digest(raw.as_bytes())),
        representation_bytes: raw.len() as u32,
        page_count: pages.len() as u32,
        region_count: pages.iter().map(|p| p.region_count).sum(),
        text_bytes: pages.iter().map(|p| p.text_bytes).sum(),
        pages,
    };
    Ok(Prepared {
        manifest,
        chunks,
        document,
    })
}
pub(crate) fn publish(c: &Connection, generation: &str, job: &str, p: &Prepared) -> Result<()> {
    let m = &p.manifest;
    let owner:bool=c.query_row("SELECT job_id=?2 AND extractor='pdf_ocr' AND source_sha256=?3 FROM extraction_generations WHERE id=?1",params![generation,job,m.source_sha256],|r|r.get(0))?;
    if !owner {
        return Err(corrupt());
    }
    c.execute(
        "INSERT INTO extraction_pdf_ocr_generations VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![
            generation,
            job,
            m.version,
            m.profile,
            m.representation_sha256,
            m.representation_bytes,
            m.page_count,
            m.region_count,
            m.text_bytes,
            p.chunks.len() as u32
        ],
    )?;
    let mut s = c.prepare("INSERT INTO extraction_pdf_ocr_chunks VALUES (?1,?2,?3,?4)")?;
    for (ordinal, text) in p.chunks.iter().enumerate() {
        s.execute(params![generation, job, ordinal as u32, text])?;
    }
    Ok(())
}
fn corrupt() -> CoreError {
    store::storage("invalid_pdf_ocr_integrity")
}
pub(crate) fn no_owner(c: &Connection, generation: &str) -> Result<()> {
    if crate::workspace::version(c)? >=16 && c.query_row("SELECT EXISTS(SELECT 1 FROM extraction_pdf_ocr_generations WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_pdf_ocr_chunks WHERE generation_id=?1)",[generation],|r|r.get::<_,bool>(0))? {return Err(corrupt());}
    Ok(())
}
pub(crate) fn load(c: &Connection, generation: &str, job: &str, count: u32) -> Result<Prepared> {
    let (chunks,bytes,bad,cross):(u32,u32,u32,bool)=c.query_row("SELECT (SELECT count(*) FROM extraction_pdf_ocr_chunks WHERE generation_id=?1),(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdf_ocr_chunks WHERE generation_id=?1),(SELECT count(*) FROM extraction_pdf_ocr_chunks WHERE generation_id=?1 AND (length(CAST(text AS BLOB)) NOT BETWEEN 1 AND 4096 OR job_id!=?2)),EXISTS(SELECT 1 FROM extraction_fragments WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_pdf_generations WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_xlsx_generations WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_ocr_generations WHERE generation_id=?1 UNION ALL SELECT 1 FROM extraction_pdfkit_generations WHERE generation_id=?1)",params![generation,job],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
    if chunks != count
        || !(1..=512).contains(&chunks)
        || !(1..=1048576).contains(&bytes)
        || bad != 0
        || cross
    {
        return Err(corrupt());
    }
    let (owner,version,profile,hash,size,pages,regions,text,declared,source):(bool,u16,bool,String,u32,u32,u32,u32,u32,String)=c.query_row("SELECT x.job_id=?2 AND g.job_id=?2 AND g.extractor='pdf_ocr',x.version,x.profile=?3,CASE WHEN length(CAST(x.representation_sha256 AS BLOB))=64 THEN x.representation_sha256 ELSE '' END,x.representation_bytes,x.page_count,x.region_count,x.text_bytes,x.chunk_count,CASE WHEN length(CAST(g.source_sha256 AS BLOB))=64 THEN g.source_sha256 ELSE '' END FROM extraction_pdf_ocr_generations x JOIN extraction_generations g ON g.id=x.generation_id WHERE x.generation_id=?1",params![generation,job,PROFILE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?)))?;
    if !owner
        || version != 1
        || !profile
        || size != bytes
        || declared != chunks
        || !(1..=20).contains(&pages)
        || regions > 10240
        || text > 1048576
    {
        return Err(corrupt());
    }
    let mut stored = Vec::with_capacity(chunks as usize);
    let mut s=c.prepare("SELECT ordinal,text FROM extraction_pdf_ocr_chunks WHERE generation_id=?1 ORDER BY ordinal")?;
    let mut rows = s.query([generation])?;
    while let Some(r) = rows.next()? {
        let ordinal: u32 = r.get(0)?;
        if ordinal as usize != stored.len() {
            return Err(corrupt());
        }
        stored.push(r.get::<_, String>(1)?);
    }
    let raw = stored.concat();
    if raw.len() != size as usize || format!("{:x}", Sha256::digest(raw.as_bytes())) != hash {
        return Err(corrupt());
    }
    let document: Document = serde_json::from_str(&raw).map_err(|_| corrupt())?;
    let p = prepare(document).map_err(|_| corrupt())?;
    if p.chunks != stored
        || p.manifest.source_sha256 != source
        || p.manifest.representation_sha256 != hash
        || p.manifest.page_count != pages
        || p.manifest.region_count != regions
        || p.manifest.text_bytes != text
    {
        return Err(corrupt());
    }
    Ok(p)
}
pub(crate) fn shape(c: &Connection) -> Result<()> {
    if c.prepare("SELECT 1 FROM extraction_generations g WHERE (g.extractor='pdf_ocr') != EXISTS(SELECT 1 FROM extraction_pdf_ocr_generations x WHERE x.generation_id=g.id) UNION ALL SELECT 1 FROM extraction_pdf_ocr_generations x LEFT JOIN extraction_generations g ON g.id=x.generation_id WHERE g.id IS NULL OR x.job_id!=g.job_id OR x.chunk_count!=g.fragment_count OR x.chunk_count!=(SELECT count(*) FROM extraction_pdf_ocr_chunks WHERE generation_id=x.generation_id) OR x.representation_bytes!=(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdf_ocr_chunks WHERE generation_id=x.generation_id) UNION ALL SELECT 1 FROM extraction_pdf_ocr_chunks x LEFT JOIN extraction_pdf_ocr_generations g ON g.generation_id=x.generation_id WHERE g.generation_id IS NULL OR x.job_id!=g.job_id LIMIT 1")?.exists([])? {return Err(corrupt());}
    Ok(())
}
pub(crate) fn integrity(c: &Connection) -> Result<()> {
    shape(c)?;
    let ids: Vec<(String, String, u32)> = c
        .prepare(
            "SELECT id,job_id,fragment_count FROM extraction_generations WHERE extractor='pdf_ocr'",
        )?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<std::result::Result<_, _>>()?;
    for (id, job, count) in ids {
        load(c, &id, &job, count)?;
        let g = store::generation(c, &id)?;
        let a = crate::attachments::metadata(c, &g.attachment_id)?;
        store::eligible_for(&a, "pdf_ocr")?;
    }
    Ok(())
}
pub(crate) fn retained(c: &Connection, except: &str) -> Result<u64> {
    let prior = crate::pdfkit_storage::retained(c, except)?;
    let x:i64=c.query_row("SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdf_ocr_chunks WHERE job_id!=?1",[except],|r|r.get(0))?;
    prior
        .checked_add(u64::try_from(x).map_err(|_| store::storage("invalid_derived_budget"))?)
        .ok_or_else(|| store::storage("invalid_derived_budget"))
}
impl crate::workspace::Workspace {
    pub fn extraction_pdf_ocr_regions(&self, input: ReadPdfOcrRegions) -> Result<PdfOcrRegionPage> {
        let limit = input.validate()?;
        let tx = self.conn.unchecked_transaction()?;
        let j = store::job(&tx, &self.path, &input.id)?;
        let g = j
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if g.extractor != "pdf_ocr"
            || !store::supported(&g.extractor, g.extractor_version, &g.config)
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut shape = store::Cursor::new(self, "extraction.pdf_ocr_regions", limit)?;
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
        store::eligible_for(&a, "pdf_ocr")?;
        if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &a, || Ok(()))?;
        let all = load(&tx, &g.id, &input.id, g.fragment_count)?;
        if cursor.is_some() && from >= all.manifest.region_count {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        let mut ordinal = 0;
        let mut items = Vec::new();
        for page in all.document.pages {
            for r in page.recognition.regions {
                if ordinal >= from && items.len() < limit as usize {
                    items.push(PdfOcrRegion {
                        ordinal,
                        page_number: page.raster.page_number,
                        region_ordinal: r.ordinal,
                        text: r.text,
                        confidence: r.confidence,
                        bbox: r.bbox,
                    });
                }
                ordinal += 1;
            }
        }
        let after = from + items.len() as u32;
        let next_cursor = if after < all.manifest.region_count {
            shape.ordinal = Some(after);
            Some(shape.encode()?)
        } else {
            None
        };
        let out = PdfOcrRegionPage {
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
