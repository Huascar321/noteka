//! Regiones inferidas de un PDF administrado, localizadas en su raster derivado.
pub use crate::pdf_raster_protocol::RasterIdentity as PdfOcrRaster;
use crate::{
    contracts::{ExtractionGeneration, OcrBoundingBox, OcrEngine},
    error::*,
    extraction_storage as store,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadPdfOcrRegions {
    pub id: String,
    #[schemars(range(min = 1, max = 64))]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
impl ReadPdfOcrRegions {
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
pub struct PdfOcrPageDescriptor {
    pub raster: PdfOcrRaster,
    pub engine: OcrEngine,
    pub coordinate_space: String,
    pub limitations: Vec<String>,
    pub region_count: u32,
    pub text_bytes: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfOcrManifest {
    pub version: u16,
    pub profile: String,
    pub raster_profile: String,
    pub source_sha256: String,
    pub representation_sha256: String,
    pub representation_bytes: u32,
    pub page_count: u32,
    pub region_count: u32,
    pub text_bytes: u32,
    pub pages: Vec<PdfOcrPageDescriptor>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfOcrRegion {
    pub ordinal: u32,
    pub page_number: u32,
    pub region_ordinal: u32,
    pub text: String,
    pub confidence: String,
    pub bbox: OcrBoundingBox,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PdfOcrRegionPage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: PdfOcrManifest,
    pub items: Vec<PdfOcrRegion>,
    pub next_cursor: Option<String>,
    pub covers_generation: bool,
}
