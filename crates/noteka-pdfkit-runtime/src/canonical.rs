//! Formato candidato para persistencia; no ejecuta DDL ni administra cuotas globales.
use noteka_pdfkit_protocol::Document;
use sha2::{Digest, Sha256};

pub const MAX_BYTES: usize = 1_048_576;
pub const CHUNK_BYTES: usize = 4096;
pub const MAX_CHUNKS: usize = 512;

#[derive(Debug, PartialEq, Eq)]
pub enum Invalid {
    Document,
    RepresentationLimit,
    Integrity,
}
#[derive(Debug)]
pub struct Prepared {
    pub document: Document,
    pub chunks: Vec<String>,
    pub representation_sha256: String,
    pub representation_bytes: u32,
}

pub fn prepare(document: Document) -> Result<Prepared, Invalid> {
    document.validate().map_err(|_| Invalid::Document)?;
    let raw = serde_json::to_string(&document).map_err(|_| Invalid::Document)?;
    if raw.is_empty() || raw.len() > MAX_BYTES {
        return Err(Invalid::RepresentationLimit);
    }
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < raw.len() {
        let mut end = (start + CHUNK_BYTES).min(raw.len());
        while !raw.is_char_boundary(end) {
            end -= 1;
        }
        chunks.push(raw[start..end].to_owned());
        start = end;
    }
    if chunks.len() > MAX_CHUNKS {
        return Err(Invalid::RepresentationLimit);
    }
    Ok(Prepared {
        document,
        chunks,
        representation_sha256: format!("{:x}", Sha256::digest(raw.as_bytes())),
        representation_bytes: raw.len() as u32,
    })
}

/// El loader SQL futuro debe acotar cantidades/tamaños antes de materializar chunks.
pub fn load(
    chunks: &[String],
    expected_sha256: &str,
    expected_bytes: u32,
) -> Result<Prepared, Invalid> {
    if chunks.is_empty()
        || chunks.len() > MAX_CHUNKS
        || expected_sha256.len() != 64
        || !expected_sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !(1..=MAX_BYTES as u32).contains(&expected_bytes)
    {
        return Err(Invalid::Integrity);
    }
    let mut total = 0usize;
    for chunk in chunks {
        if chunk.is_empty() || chunk.len() > CHUNK_BYTES {
            return Err(Invalid::Integrity);
        }
        total = total.checked_add(chunk.len()).ok_or(Invalid::Integrity)?;
        if total > MAX_BYTES {
            return Err(Invalid::Integrity);
        }
    }
    if total != expected_bytes as usize {
        return Err(Invalid::Integrity);
    }
    let raw = chunks.concat();
    if format!("{:x}", Sha256::digest(raw.as_bytes())) != expected_sha256 {
        return Err(Invalid::Integrity);
    }
    let document: Document = serde_json::from_str(&raw).map_err(|_| Invalid::Integrity)?;
    let prepared = prepare(document).map_err(|_| Invalid::Integrity)?;
    if prepared.chunks != chunks
        || prepared.representation_bytes != expected_bytes
        || prepared.representation_sha256 != expected_sha256
    {
        return Err(Invalid::Integrity);
    }
    Ok(prepared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use noteka_pdfkit_protocol::{Engine, LIMITATIONS, Page, PageStatus};
    fn document(text: &str) -> Document {
        Document {
            version: 1,
            engine: Engine {
                api: "PDFKit.PDFPage.attributedString".into(),
                os_version: "27.0.1".into(),
                os_build: "26A434".into(),
            },
            limitations: LIMITATIONS.iter().map(|s| (*s).into()).collect(),
            page_count: 1,
            text_bytes: text.len() as u32,
            pages: vec![Page {
                number: 1,
                status: if text.is_empty() {
                    PageStatus::NoText
                } else {
                    PageStatus::Text
                },
                text: text.into(),
            }],
        }
    }
    #[test]
    fn exact_order_unicode_and_reload_are_stable() {
        let p = prepare(document("café")).unwrap();
        let expected = r#"{"version":1,"engine":{"api":"PDFKit.PDFPage.attributedString","os_version":"27.0.1","os_build":"26A434"},"limitations":["system_parser_not_complete_pdf_integrity_validator","selection_order_not_visual_reading_order","no_ocr_structured_tables_or_geometry","no_text_not_proof_of_visual_absence","system_framework_result_may_vary_by_os"],"page_count":1,"text_bytes":5,"pages":[{"number":1,"status":"text","text":"café"}]}"#;
        assert_eq!(p.chunks.concat(), expected);
        assert_eq!(
            load(&p.chunks, &p.representation_sha256, p.representation_bytes)
                .unwrap()
                .document,
            p.document
        );
        assert!(prepare(document("")).is_ok());
    }
    #[test]
    fn representation_boundary_and_escaping_are_not_truncated() {
        let mut count = MAX_BYTES - 1000;
        loop {
            let d = document(&"x".repeat(count));
            let size = serde_json::to_vec(&d).unwrap().len();
            if size == MAX_BYTES {
                break;
            }
            if size < MAX_BYTES {
                count += MAX_BYTES - size;
            } else {
                count -= size - MAX_BYTES;
            }
        }
        let p = prepare(document(&"x".repeat(count))).unwrap();
        assert_eq!(p.representation_bytes, MAX_BYTES as u32);
        assert_eq!(
            load(&p.chunks, &p.representation_sha256, p.representation_bytes)
                .unwrap()
                .document,
            p.document
        );
        assert_eq!(
            prepare(document(&"x".repeat(count + 1))).unwrap_err(),
            Invalid::RepresentationLimit
        );
        assert_eq!(
            prepare(document(&"\n".repeat(MAX_BYTES))).unwrap_err(),
            Invalid::RepresentationLimit
        );
        let p = prepare(document(&"漢é".repeat(5000))).unwrap();
        assert!(p.chunks.iter().all(|s| s.len() <= 4096));
        assert_eq!(
            load(&p.chunks, &p.representation_sha256, p.representation_bytes)
                .unwrap()
                .document,
            p.document
        );
    }
    #[test]
    fn hostile_chunks_hash_metadata_and_noncanonical_json_fail_closed() {
        let p = prepare(document("source")).unwrap();
        for chunks in [
            vec![],
            vec![String::new()],
            vec!["x".repeat(4097)],
            vec!["x".into(); 513],
        ] {
            assert!(load(&chunks, &p.representation_sha256, p.representation_bytes).is_err());
        }
        for hash in ["", &"A".repeat(64), &"0".repeat(64)] {
            assert!(load(&p.chunks, hash, p.representation_bytes).is_err());
        }
        assert!(
            load(
                &p.chunks,
                &p.representation_sha256,
                p.representation_bytes + 1
            )
            .is_err()
        );
        let raw = " ".to_owned() + &p.chunks.concat();
        let hash = format!("{:x}", Sha256::digest(raw.as_bytes()));
        assert!(load(std::slice::from_ref(&raw), &hash, raw.len() as u32).is_err());
        let mut bad = p.document.clone();
        bad.page_count = 2;
        assert_eq!(prepare(bad).unwrap_err(), Invalid::Document);
    }
}
