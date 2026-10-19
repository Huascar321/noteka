//! JSON canónico versionado, persistido una vez; proyecciones públicas derivadas.
use crate::{contracts::*, error::*, extraction_storage as store};
use noteka_ocr_protocol::{PROFILE, v2::OcrRecognition};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

pub(crate) fn source_format(m: &OcrManifest, a: &Attachment) -> Result<()> {
    store::eligible_for(a, "image_ocr")?;
    let ext = std::path::Path::new(&a.name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if (ext == "png") != (m.image.format == OcrImageFormat::Png) {
        return Err(store::invalid("ocr_source_format_mismatch"));
    }
    Ok(())
}

pub(crate) struct Prepared {
    pub manifest: OcrManifest,
    pub chunks: Vec<String>,
    pub recognition: OcrRecognition,
}
pub(crate) fn prepare(recognition: OcrRecognition) -> Result<Prepared> {
    recognition
        .validate()
        .map_err(|_| store::storage("ocr_worker_protocol"))?;
    let raw = serde_json::to_string(&recognition)?;
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
        return Err(store::invalid("ocr_result_limit"));
    }
    let manifest = OcrManifest {
        version: 1,
        profile: PROFILE.into(),
        representation_sha256: format!("{:x}", Sha256::digest(raw.as_bytes())),
        representation_bytes: raw.len() as u32,
        image: recognition.image.clone(),
        engine: recognition.engine.clone(),
        coordinate_space: recognition.coordinate_space.clone(),
        limitations: recognition.limitations.clone(),
        region_count: recognition.regions.len() as u32,
    };
    Ok(Prepared {
        manifest,
        chunks,
        recognition,
    })
}
pub(crate) fn publish(c: &Connection, generation: &str, job: &str, p: &Prepared) -> Result<()> {
    let m = &p.manifest;
    c.execute(
        "INSERT INTO extraction_ocr_generations VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![
            generation,
            job,
            m.version,
            m.profile,
            m.representation_sha256,
            m.representation_bytes,
            m.region_count,
            p.chunks.len() as u32
        ],
    )?;
    let mut s = c.prepare("INSERT INTO extraction_ocr_chunks VALUES (?1,?2,?3,?4)")?;
    for (ordinal, text) in p.chunks.iter().enumerate() {
        s.execute(params![generation, job, ordinal as u32, text])?;
    }
    Ok(())
}
fn corrupt() -> CoreError {
    store::storage("invalid_ocr_integrity")
}
pub(crate) fn load(c: &Connection, generation: &str, job: &str, count: u32) -> Result<Prepared> {
    // No materializar texto antes de comprobar presupuestos y cruces de formato.
    let (chunks,bytes,oversized,literal,pdf,xlsx):(u32,u32,u32,u32,u32,u32)=c.query_row("SELECT (SELECT count(*) FROM extraction_ocr_chunks WHERE generation_id=?1),(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_ocr_chunks WHERE generation_id=?1),(SELECT count(*) FROM extraction_ocr_chunks WHERE generation_id=?1 AND length(CAST(text AS BLOB)) NOT BETWEEN 1 AND 4096),(SELECT count(*) FROM extraction_fragments WHERE generation_id=?1),(SELECT count(*) FROM extraction_pdf_generations WHERE generation_id=?1),(SELECT count(*) FROM extraction_xlsx_generations WHERE generation_id=?1)",[generation],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))?;
    if chunks != count
        || !(1..=512).contains(&chunks)
        || !(1..=1_048_576).contains(&bytes)
        || oversized != 0
        || literal != 0
        || pdf != 0
        || xlsx != 0
    {
        return Err(corrupt());
    }
    // Comparar propietario/parser dentro de SQL evita materializar metadatos
    // arbitrariamente grandes de una base alterada. El hash cargado queda <=64 B.
    let (owner,version,profile,hash,size,regions,declared):(bool,u32,bool,String,u32,u32,u32)=c.query_row("SELECT job_id=?2,version,profile=?3,CASE WHEN length(CAST(representation_sha256 AS BLOB))=64 THEN representation_sha256 ELSE '' END,representation_bytes,region_count,chunk_count FROM extraction_ocr_generations WHERE generation_id=?1",params![generation,job,PROFILE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)))?;
    if !owner
        || version != 1
        || !profile
        || hash.len() != 64
        || size != bytes
        || declared != chunks
        || regions > 512
    {
        return Err(corrupt());
    }
    let mut raw = String::with_capacity(size as usize);
    let mut stmt=c.prepare("SELECT job_id=?2,ordinal,text FROM extraction_ocr_chunks WHERE generation_id=?1 ORDER BY ordinal")?;
    let mut rows = stmt.query(params![generation, job])?;
    let mut expected = 0;
    while let Some(r) = rows.next()? {
        let owner: bool = r.get(0)?;
        let ordinal: u32 = r.get(1)?;
        let text: String = r.get(2)?;
        if !owner
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
    let recognition: OcrRecognition = serde_json::from_str(&raw).map_err(|_| corrupt())?;
    let prepared = prepare(recognition).map_err(|_| corrupt())?;
    // Impide codificaciones JSON no canónicas y particiones ambiguas sin cambiar versión.
    if prepared.chunks.concat() != raw
        || prepared.chunks.len() != chunks as usize
        || prepared.manifest.region_count != regions
    {
        return Err(corrupt());
    }
    let stored: Vec<String> = c
        .prepare("SELECT text FROM extraction_ocr_chunks WHERE generation_id=?1 ORDER BY ordinal")?
        .query_map([generation], |r| r.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    if stored != prepared.chunks {
        return Err(corrupt());
    }
    Ok(prepared)
}
pub(crate) fn shape(c: &Connection) -> Result<()> {
    if c.prepare("SELECT 1 FROM extraction_generations g WHERE (g.extractor='image_ocr') != EXISTS(SELECT 1 FROM extraction_ocr_generations x WHERE x.generation_id=g.id) UNION ALL SELECT 1 FROM extraction_ocr_generations x LEFT JOIN extraction_generations g ON g.id=x.generation_id WHERE g.id IS NULL OR x.job_id!=g.job_id OR x.chunk_count!=g.fragment_count OR x.chunk_count!=(SELECT count(*) FROM extraction_ocr_chunks WHERE generation_id=x.generation_id) OR x.representation_bytes!=(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_ocr_chunks WHERE generation_id=x.generation_id) UNION ALL SELECT 1 FROM extraction_ocr_chunks x LEFT JOIN extraction_ocr_generations g ON g.generation_id=x.generation_id WHERE g.generation_id IS NULL OR x.job_id!=g.job_id LIMIT 1")?.exists([])? { return Err(corrupt()); }
    Ok(())
}
pub(crate) fn integrity(c: &Connection) -> Result<()> {
    shape(c)?;
    let ids:Vec<(String,String,u32)>=c.prepare("SELECT id,job_id,fragment_count FROM extraction_generations WHERE extractor='image_ocr'")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<std::result::Result<_,_>>()?;
    for (id, job, count) in ids {
        let p = load(c, &id, &job, count)?;
        let g = store::generation(c, &id)?;
        let a = crate::attachments::metadata(c, &g.attachment_id)?;
        source_format(&p.manifest, &a)?;
    }
    Ok(())
}
pub(crate) fn retained(c: &Connection, except: &str) -> Result<u64> {
    let prior = crate::xlsx_storage::retained(c, except)?;
    let x:i64=c.query_row("SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_ocr_chunks WHERE job_id!=?1",[except],|r|r.get(0))?;
    prior
        .checked_add(u64::try_from(x).map_err(|_| store::storage("invalid_derived_budget"))?)
        .ok_or_else(|| store::storage("invalid_derived_budget"))
}

impl crate::workspace::Workspace {
    pub fn extraction_ocr_regions(&self, input: ReadOcrRegions) -> Result<OcrRegionPage> {
        let limit = input.validate()?;
        let tx = self.conn.unchecked_transaction()?;
        let j = store::job(&tx, &self.path, &input.id)?;
        let g = j
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if g.extractor != "image_ocr"
            || !store::supported(&g.extractor, g.extractor_version, &g.config)
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut shape = store::Cursor::new(self, "extraction.ocr_regions", limit)?;
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
        store::eligible_for(&a, "image_ocr")?;
        if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &a, || Ok(()))?;
        let all = load(&tx, &g.id, &input.id, g.fragment_count)?;
        source_format(&all.manifest, &a)?;
        if cursor.is_some() && from >= all.manifest.region_count {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        let items = all
            .recognition
            .regions
            .into_iter()
            .skip(from as usize)
            .take(limit as usize)
            .collect::<Vec<_>>();
        let after = from + items.len() as u32;
        let next_cursor = if after < all.manifest.region_count {
            shape.ordinal = Some(after);
            Some(shape.encode()?)
        } else {
            None
        };
        let out = OcrRegionPage {
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
