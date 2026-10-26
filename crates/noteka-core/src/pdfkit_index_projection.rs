//! Proyección de texto nativo ya persistido, sin parsing, OCR o metadata indexable.
use crate::{batch_storage::Budget, contracts::*, error::*, extraction_storage as store};
use noteka_pdfkit_protocol::{Document, PROFILE};

pub(crate) fn project(
    document: &Document,
    b: &Budget<'_>,
) -> Result<Vec<AttachmentSearchFragment>> {
    document
        .validate()
        .map_err(|_| store::storage("invalid_pdfkit_integrity"))?;
    let mut out = Vec::new();
    for page in &document.pages {
        b.check()?;
        let mut start = 0;
        while start < page.text.len() {
            b.check()?;
            if out.len() >= MAX_EXTRACTION_FRAGMENTS as usize {
                return Err(store::invalid("index_budget_exceeded"));
            }
            let mut end = (start + EXTRACTION_CHUNK_BYTES as usize).min(page.text.len());
            while !page.text.is_char_boundary(end) {
                end -= 1;
            }
            out.push(AttachmentSearchFragment {
                ordinal: out.len() as u32,
                text: page.text[start..end].into(),
                locator: AttachmentTextLocator::PdfkitPageText {
                    projection: PdfkitTextProjection::PdfkitPageTextV1,
                    profile: PROFILE.into(),
                    page_number: page.number,
                    page_bytes: page.text.len() as u32,
                    text_byte_start: start as u32,
                    text_byte_end: end as u32,
                    engine: document.engine.clone(),
                    limitations: document.limitations.clone(),
                },
            });
            start = end;
        }
    }
    Ok(out)
}
