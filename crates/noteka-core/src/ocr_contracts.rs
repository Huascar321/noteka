//! Regiones OCR inferidas; no son offsets del original ni lectura visual garantizada.
use crate::{contracts::ExtractionGeneration, error::*, extraction_storage as store};
pub use noteka_ocr_protocol::v2::{OcrBoundingBox, OcrEngine, OcrImage, OcrImageFormat, OcrRegion};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadOcrRegions {
    pub id: String,
    #[schemars(range(min = 1, max = 64))]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
impl ReadOcrRegions {
    pub fn validate(&self) -> Result<u32> {
        store::uuid(&self.id)?;
        let n = store::page_limit(self.limit, 32, 64)?;
        if let Some(c) = &self.cursor {
            store::Cursor::read(c)?;
        }
        Ok(n)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OcrManifest {
    pub version: u32,
    pub profile: String,
    pub representation_sha256: String,
    #[schemars(range(min = 1, max = 1048576))]
    pub representation_bytes: u32,
    pub image: OcrImage,
    pub engine: OcrEngine,
    pub coordinate_space: String,
    pub limitations: Vec<String>,
    #[schemars(range(max = 512))]
    pub region_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OcrRegionPage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: OcrManifest,
    pub items: Vec<OcrRegion>,
    pub next_cursor: Option<String>,
    pub covers_generation: bool,
}
