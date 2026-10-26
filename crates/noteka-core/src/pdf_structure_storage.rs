//! Estructura tabular Paddle derivada. No es fuente de verdad ni ejecuta OCR.
use crate::{contracts::*, error::*, extraction_storage as store, workspace::Workspace};
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};

pub(crate) fn publish(
    c: &Connection,
    generation: &str,
    job: &str,
    source_sha256: &str,
    structure: &PdfStructure,
) -> Result<()> {
    structure.validate()?;
    let owner: bool = c.query_row(
        "SELECT job_id=?2 AND extractor='paddle_ocr' AND source_sha256=?3 FROM extraction_generations WHERE id=?1",
        params![generation, job, source_sha256],
        |r| r.get(0),
    )?;
    if !owner {
        return Err(store::storage("invalid_pdf_structure_integrity"));
    }
    let raw = serde_json::to_string(structure)?;
    let bytes = raw.len() as u32;
    if bytes > 1_048_576 {
        return Err(store::invalid("pdf_structure_limit"));
    }
    c.execute(
        "INSERT INTO extraction_pdf_structures(generation_id,job_id,version,format,source_sha256,structure_sha256,structure_bytes,page_count,table_count,structure_json) VALUES (?1,?2,1,'paddle_structure_v1',?3,?4,?5,?6,?7,?8)",
        params![
            generation,
            job,
            source_sha256,
            format!("{:x}", Sha256::digest(raw.as_bytes())),
            bytes,
            structure.pages.len() as u32,
            structure.table_count(),
            raw
        ],
    )?;
    Ok(())
}

pub(crate) fn integrity(c: &Connection) -> Result<()> {
    let rows = c
        .prepare("SELECT x.generation_id,x.job_id,x.source_sha256,x.structure_sha256,x.structure_bytes,x.page_count,x.table_count,x.structure_json,g.extractor,g.source_sha256,g.job_id FROM extraction_pdf_structures x JOIN extraction_generations g ON g.id=x.generation_id")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, u32>(4)?,
                r.get::<_, u32>(5)?,
                r.get::<_, u32>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, String>(8)?,
                r.get::<_, String>(9)?,
                r.get::<_, String>(10)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for row in rows {
        let (generation, job, source, hash, bytes, pages, tables, raw, extractor, gsource, gjob) =
            row;
        if extractor != "paddle_ocr"
            || gjob != job
            || gsource != source
            || !is_sha256(&source)
            || !is_sha256(&hash)
            || raw.len() != bytes as usize
            || format!("{:x}", Sha256::digest(raw.as_bytes())) != hash
        {
            return Err(store::storage("invalid_pdf_structure_integrity"));
        }
        let structure: PdfStructure = serde_json::from_str(&raw)
            .map_err(|_| store::storage("invalid_pdf_structure_integrity"))?;
        structure
            .validate()
            .map_err(|_| store::storage("invalid_pdf_structure_integrity"))?;
        if structure.pages.len() as u32 != pages || structure.table_count() != tables {
            return Err(store::storage("invalid_pdf_structure_integrity"));
        }
        let _ = generation;
    }
    Ok(())
}
fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

impl Workspace {
    pub fn extraction_pdf_structure(&self, input: ReadPdfStructure) -> Result<PdfStructureResult> {
        store::uuid(&input.id)?;
        let tx = self.conn.unchecked_transaction()?;
        let j = store::job(&tx, &self.path, &input.id)?;
        let g = j
            .generation
            .clone()
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if g.extractor != "paddle_ocr"
            || !store::supported(&g.extractor, g.extractor_version, &g.config)
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let a = crate::attachments::metadata(&tx, &g.attachment_id)?;
        store::eligible_for(&a, "paddle_ocr")?;
        if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &a, || Ok(()))?;
        let row: Option<(String, String, u32, u32, u32, String)> = tx
            .query_row(
                "SELECT source_sha256,structure_sha256,structure_bytes,page_count,table_count,structure_json FROM extraction_pdf_structures WHERE generation_id=?1 AND job_id=?2",
                params![g.id, input.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .optional()?;
        let (source, hash, bytes, pages, tables, raw) =
            row.ok_or_else(|| store::error(ErrorCode::NotFound, "pdf_structure_not_found"))?;
        if source != g.source_sha256
            || raw.len() != bytes as usize
            || format!("{:x}", Sha256::digest(raw.as_bytes())) != hash
        {
            return Err(store::storage("invalid_pdf_structure_integrity"));
        }
        let structure: PdfStructure = serde_json::from_str(&raw)
            .map_err(|_| store::storage("invalid_pdf_structure_integrity"))?;
        structure.validate()?;
        if structure.pages.len() as u32 != pages || structure.table_count() != tables {
            return Err(store::storage("invalid_pdf_structure_integrity"));
        }
        let out = PdfStructureResult {
            job_id: input.id,
            generation: g,
            result_from_previous_attempt: j.result_from_previous_attempt,
            manifest: PdfStructureManifest {
                version: 1,
                format: "paddle_structure_v1".into(),
                source_sha256: source,
                structure_sha256: hash,
                structure_bytes: bytes,
                page_count: pages,
                table_count: tables,
            },
            structure,
        };
        store::response(&out)?;
        Ok(out)
    }
}
