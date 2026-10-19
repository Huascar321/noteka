//! Texto PDF derivado: persistencia/lectura sin dependencia del parser.
use crate::{contracts::*, error::*, extraction_storage as store, workspace::Workspace};
use noteka_pdf_protocol::{PARSER_NAME, PARSER_VERSION, PageText};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

pub(crate) struct Prepared {
    pub manifest: PdfManifest,
    pub fragments: Vec<PdfFragment>,
}
pub(crate) fn prepare(pages: Vec<PageText>) -> Result<Prepared> {
    if pages.is_empty() || pages.len() > 100 {
        return Err(store::invalid("pdf_page_limit"));
    }
    let mut fragments = Vec::new();
    let mut descriptors = Vec::new();
    let mut total = 0usize;
    for (index, page) in pages.into_iter().enumerate() {
        if page.page_number as usize != index + 1 || page.text.contains('\0') {
            return Err(store::storage("pdf_worker_protocol"));
        }
        total = total
            .checked_add(page.text.len())
            .ok_or_else(|| store::invalid("pdf_text_limit"))?;
        if total > 1_048_576 {
            return Err(store::invalid("pdf_text_limit"));
        }
        let mut start = 0;
        let before = fragments.len();
        while start < page.text.len() {
            let mut end = (start + 4096).min(page.text.len());
            while !page.text.is_char_boundary(end) {
                end -= 1;
            }
            if fragments.len() >= 512 {
                return Err(store::invalid("fragment_count_limit"));
            }
            fragments.push(PdfFragment {
                ordinal: fragments.len() as u32,
                text: page.text[start..end].into(),
                locator: PdfTextLocator {
                    kind: PdfLocatorKind::PdfPageText,
                    page_number: page.page_number,
                    text_byte_start: start as u32,
                    text_byte_end: end as u32,
                },
            });
            start = end;
        }
        descriptors.push(PdfPageDescriptor {
            page_number: page.page_number,
            state: if page.text.is_empty() {
                PdfPageState::NoText
            } else {
                PdfPageState::Text
            },
            text_bytes: page.text.len() as u32,
            text_sha256: format!("{:x}", Sha256::digest(page.text.as_bytes())),
            fragment_count: (fragments.len() - before) as u32,
        });
    }
    let manifest = PdfManifest {
        version: 1,
        parser_name: PARSER_NAME.into(),
        parser_version: PARSER_VERSION.into(),
        scope: PdfTextScope::PageTextObjects,
        limitations: PDF_LIMITATIONS.to_vec(),
        page_count: descriptors.len() as u32,
        extracted_bytes: total as u32,
        pages: descriptors,
    };
    validate_pdf_fragments(&manifest, &fragments)?;
    Ok(Prepared {
        manifest,
        fragments,
    })
}
pub(crate) fn publish(
    conn: &Connection,
    generation: &str,
    job: &str,
    prepared: &Prepared,
) -> Result<()> {
    validate_pdf_fragments(&prepared.manifest, &prepared.fragments)?;
    let m = &prepared.manifest;
    conn.execute(
        "INSERT INTO extraction_pdf_generations VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            generation,
            job,
            m.version,
            m.parser_name,
            m.parser_version,
            m.page_count,
            m.extracted_bytes
        ],
    )?;
    let mut stmt = conn.prepare("INSERT INTO extraction_pdf_pages VALUES (?1,?2,?3,?4,?5,?6)")?;
    for p in &m.pages {
        stmt.execute(params![
            generation,
            p.page_number,
            if p.state == PdfPageState::Text {
                "text"
            } else {
                "no_text"
            },
            p.text_bytes,
            p.text_sha256,
            p.fragment_count
        ])?;
    }
    let mut stmt =
        conn.prepare("INSERT INTO extraction_pdf_fragments VALUES (?1,?2,?3,?4,?5,?6,?7)")?;
    for f in &prepared.fragments {
        stmt.execute(params![
            generation,
            job,
            f.ordinal,
            f.text,
            f.locator.page_number,
            f.locator.text_byte_start,
            f.locator.text_byte_end
        ])?;
    }
    Ok(())
}
pub(crate) fn load(conn: &Connection, generation: &str, job: &str, count: u32) -> Result<Prepared> {
    let (pages,fragments,bytes,literal):(u32,u32,u32,u32) = conn.query_row("SELECT (SELECT count(*) FROM extraction_pdf_pages WHERE generation_id=?1),(SELECT count(*) FROM extraction_pdf_fragments WHERE generation_id=?1),(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdf_fragments WHERE generation_id=?1),(SELECT count(*) FROM extraction_fragments WHERE generation_id=?1)",[generation],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
    if !(1..=100).contains(&pages)
        || fragments != count
        || fragments > 512
        || bytes > 1_048_576
        || literal != 0
    {
        return Err(store::storage("invalid_pdf_integrity"));
    }
    let (mut manifest,owner): (PdfManifest,String) = conn.query_row("SELECT version,parser_name,parser_version,page_count,extracted_bytes,job_id FROM extraction_pdf_generations WHERE generation_id=?1",[generation],|r| Ok((PdfManifest { version:r.get(0)?,parser_name:r.get(1)?,parser_version:r.get(2)?,scope:PdfTextScope::PageTextObjects,limitations:PDF_LIMITATIONS.to_vec(),page_count:r.get(3)?,extracted_bytes:r.get(4)?,pages:Vec::new() },r.get(5)?)))?;
    if owner != job || manifest.page_count != pages || manifest.extracted_bytes != bytes {
        return Err(store::storage("invalid_pdf_integrity"));
    }
    manifest.pages = conn.prepare("SELECT page_number,state,text_bytes,text_sha256,fragment_count FROM extraction_pdf_pages WHERE generation_id=?1 ORDER BY page_number")?.query_map([generation],|r| {
        let state:String=r.get(1)?;
        Ok(PdfPageDescriptor { page_number:r.get(0)?,state:match state.as_str() { "text"=>PdfPageState::Text,"no_text"=>PdfPageState::NoText,_=>return Err(rusqlite::Error::InvalidQuery) },text_bytes:r.get(2)?,text_sha256:r.get(3)?,fragment_count:r.get(4)? })
    })?.collect::<std::result::Result<_,_>>()?;
    let rows:Vec<(PdfFragment,String)> = conn.prepare("SELECT ordinal,text,page_number,text_byte_start,text_byte_end,job_id FROM extraction_pdf_fragments WHERE generation_id=?1 ORDER BY ordinal")?.query_map([generation],|r| Ok((PdfFragment { ordinal:r.get(0)?,text:r.get(1)?,locator:PdfTextLocator { kind:PdfLocatorKind::PdfPageText,page_number:r.get(2)?,text_byte_start:r.get(3)?,text_byte_end:r.get(4)? } },r.get(5)?)))?.collect::<std::result::Result<_,_>>()?;
    if rows.iter().any(|(_, owner)| owner != job) {
        return Err(store::storage("invalid_pdf_integrity"));
    }
    let fragments = rows.into_iter().map(|(f, _)| f).collect::<Vec<_>>();
    validate_pdf_fragments(&manifest, &fragments)?;
    Ok(Prepared {
        manifest,
        fragments,
    })
}
pub(crate) fn shape(conn: &Connection) -> Result<()> {
    let invalid = conn.prepare("SELECT 1 FROM extraction_generations g WHERE (g.extractor='pdf_text') != EXISTS(SELECT 1 FROM extraction_pdf_generations p WHERE p.generation_id=g.id) UNION ALL SELECT 1 FROM extraction_pdf_generations p LEFT JOIN extraction_generations g ON g.id=p.generation_id WHERE g.id IS NULL OR p.job_id!=g.job_id UNION ALL SELECT 1 FROM extraction_pdf_pages p LEFT JOIN extraction_pdf_generations g ON g.generation_id=p.generation_id WHERE g.generation_id IS NULL UNION ALL SELECT 1 FROM extraction_pdf_fragments f LEFT JOIN extraction_pdf_pages p ON p.generation_id=f.generation_id AND p.page_number=f.page_number WHERE p.generation_id IS NULL LIMIT 1")?.exists([])?;
    if invalid {
        return Err(store::storage("invalid_pdf_integrity"));
    }
    let counts=conn.prepare("SELECT 1 FROM extraction_pdf_generations p JOIN extraction_generations g ON g.id=p.generation_id WHERE p.page_count!=(SELECT count(*) FROM extraction_pdf_pages WHERE generation_id=p.generation_id) OR p.extracted_bytes!=(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdf_fragments WHERE generation_id=p.generation_id) OR g.fragment_count!=(SELECT count(*) FROM extraction_pdf_fragments WHERE generation_id=p.generation_id) LIMIT 1")?.exists([])?;
    if counts {
        return Err(store::storage("invalid_pdf_integrity"));
    }
    Ok(())
}
pub(crate) fn integrity(conn: &Connection) -> Result<()> {
    shape(conn)?;
    let ids:Vec<(String,String,u32)>=conn.prepare("SELECT id,job_id,fragment_count FROM extraction_generations WHERE extractor='pdf_text'")?.query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<std::result::Result<_,_>>()?;
    for (id, job, count) in ids {
        load(conn, &id, &job, count)?;
    }
    Ok(())
}
pub(crate) fn retained(conn: &Connection, except: &str) -> Result<u64> {
    let n:i64=conn.query_row("SELECT (SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_fragments WHERE job_id!=?1)+(SELECT coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_pdf_fragments WHERE job_id!=?1)",[except],|r|r.get(0))?;
    u64::try_from(n).map_err(|_| store::storage("invalid_derived_budget"))
}
impl Workspace {
    pub fn extraction_pdf_fragments(&self, input: ReadPdfFragments) -> Result<PdfFragmentPage> {
        let limit = input.validate()?;
        let tx = self.conn.unchecked_transaction()?;
        let j = store::job(&tx, &self.path, &input.id)?;
        let g = j
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if g.extractor != "pdf_text"
            || !store::supported(&g.extractor, g.extractor_version, &g.config)
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut shape = store::Cursor::new(self, "extraction.pdf_fragments", limit)?;
        shape.job_id = Some(input.id.clone());
        shape.generation_id = Some(g.id.clone());
        let from = if let Some(raw) = &input.cursor {
            let c = store::Cursor::read(raw)?;
            let mut blank = c.clone();
            blank.ordinal = None;
            if blank != shape || c.ordinal.is_none_or(|n| n == 0 || n >= g.fragment_count) {
                return Err(store::invalid("cursor_scope_mismatch"));
            }
            c.ordinal.unwrap()
        } else {
            0
        };
        let a = crate::attachments::metadata(&tx, &g.attachment_id)?;
        store::eligible_for(&a, "pdf_text")?;
        if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &a, || Ok(()))?;
        let all = load(&tx, &g.id, &input.id, g.fragment_count)?;
        let items = all
            .fragments
            .into_iter()
            .skip(from as usize)
            .take(limit as usize)
            .collect::<Vec<_>>();
        let after = from + items.len() as u32;
        let next_cursor = if after < g.fragment_count {
            shape.ordinal = Some(after);
            Some(shape.encode()?)
        } else {
            None
        };
        let out = PdfFragmentPage {
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
