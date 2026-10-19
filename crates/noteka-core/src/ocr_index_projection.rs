//! Proyectar sólo texto reconocido validado, sin metadata ni umbral de confianza.
use crate::{batch_storage::Budget, contracts::*, error::*, extraction_storage as store};
use noteka_ocr_protocol::v2::OcrRecognition;
pub(crate) fn project(r: &OcrRecognition, b: &Budget<'_>) -> Result<Vec<AttachmentSearchFragment>> {
    r.validate()
        .map_err(|_| store::storage("invalid_ocr_integrity"))?;
    let mut out = Vec::new();
    let mut bytes = 0usize;
    for region in &r.regions {
        b.check()?;
        bytes += region.text.len();
        if bytes > MAX_EXTRACTION_BYTES as usize {
            return Err(store::invalid("index_budget_exceeded"));
        }
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
                locator: AttachmentTextLocator::ImageOcrRegionText {
                    projection: OcrTextProjection::OcrRegionTextV1,
                    region_ordinal: region.ordinal,
                    bbox: region.bbox.clone(),
                    confidence: region.confidence.clone(),
                    coordinate_space: r.coordinate_space.clone(),
                    region_bytes: region.text.len() as u32,
                    text_byte_start: start as u32,
                    text_byte_end: end as u32,
                },
            });
            start = end;
        }
    }
    Ok(out)
}
