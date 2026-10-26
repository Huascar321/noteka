//! Páginas físicas del derivado PDFKit; sin geometría ni OCR implícito.
use crate::{contracts::ExtractionGeneration, error::*, extraction_storage as store};
pub use noteka_pdfkit_protocol::{Engine as PdfkitEngine, Page as PdfkitPage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadPdfkitPages {
    pub id: String,
    #[schemars(range(min = 1, max = 64))]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
impl ReadPdfkitPages {
    pub fn validate(&self) -> Result<u32> {
        store::uuid(&self.id)?;
        let limit = store::page_limit(self.limit, 32, 64)?;
        if let Some(c) = &self.cursor {
            store::Cursor::read(c)?;
        }
        Ok(limit)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfkitManifest {
    pub version: u32,
    pub profile: String,
    pub representation_sha256: String,
    #[schemars(range(min = 1, max = 1048576))]
    pub representation_bytes: u32,
    pub engine: PdfkitEngine,
    pub limitations: Vec<String>,
    #[schemars(range(min = 1, max = 100))]
    pub page_count: u32,
    pub text_bytes: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PdfkitPagePage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: PdfkitManifest,
    pub items: Vec<PdfkitPage>,
    pub next_cursor: Option<String>,
    pub covers_generation: bool,
}
