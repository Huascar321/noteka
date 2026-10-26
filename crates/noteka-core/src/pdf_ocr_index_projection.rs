//! Texto OCR persistido, separado por página/región; jamás indexa JSON/metadata.
use crate::{
    batch_storage::Budget,
    contracts::*,
    error::*,
    extraction_storage as store,
    pdf_ocr_compositor::{Document, PROFILE},
};
pub(crate) fn project(
    document: &Document,
    b: &Budget<'_>,
) -> Result<Vec<AttachmentSearchFragment>> {
    document
        .validate()
        .map_err(|_| store::storage("invalid_pdf_ocr_integrity"))?;
    let mut out = Vec::new();
    let mut global = 0;
    for page in &document.pages {
        for region in &page.recognition.regions {
            b.check()?;
            let mut start = 0;
            while start < region.text.len() {
                b.check()?;
                if out.len() >= MAX_EXTRACTION_FRAGMENTS as usize {
                    return Err(store::invalid("index_budget_exceeded"));
                }
                let mut end = (start + EXTRACTION_CHUNK_BYTES as usize).min(region.text.len());
                while !region.text.is_char_boundary(end) {
                    end -= 1;
                }
                out.push(AttachmentSearchFragment {
                    ordinal: out.len() as u32,
                    text: region.text[start..end].into(),
                    locator: AttachmentTextLocator::PdfOcrRegionText {
                        projection: PdfOcrTextProjection::PdfOcrRegionTextV1,
                        profile: PROFILE.into(),
                        page_number: page.raster.page_number,
                        region_ordinal: region.ordinal,
                        global_region_ordinal: global,
                        raster: Box::new(page.raster.clone()),
                        engine: Box::new(page.recognition.engine.clone()),
                        bbox: region.bbox.clone(),
                        confidence: region.confidence.clone(),
                        coordinate_space: "rendered_page_normalized_bottom_left".into(),
                        limitations: page.recognition.limitations.clone(),
                        region_bytes: region.text.len() as u32,
                        text_byte_start: start as u32,
                        text_byte_end: end as u32,
                    },
                });
                start = end;
            }
            global += 1;
        }
    }
    Ok(out)
}
