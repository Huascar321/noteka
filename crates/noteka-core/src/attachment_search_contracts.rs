//! DTOs aditivos: no reutilizan hits ni cursores de la búsqueda de entidades.
use crate::{
    contracts::{PdfLimitation, PdfTextScope},
    error::*,
    extraction_storage as store,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MAX_INDEX_ENTRIES: u32 = 1024;
pub const MAX_INDEX_FRAGMENTS: u32 = 16_384;
pub const MAX_INDEX_TEXT_BYTES: u64 = 8 * 1024 * 1024;
pub const ATTACHMENT_INDEX_PROFILE: &str = "literal_unicode61_v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum AttachmentIndexInput {
    Get {
        job_id: Option<String>,
        attachment_id: Option<String>,
        extractor: Option<IndexedExtractor>,
        limit: Option<u32>,
        cursor: Option<String>,
    },
    Put {
        job_id: String,
        expected_generation_id: String,
        expected_revision: u32,
    },
    Reset {
        expected_revision: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentSearchInput {
    pub query: String,
    pub job_id: Option<String>,
    pub attachment_id: Option<String>,
    pub extractor: Option<IndexedExtractor>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IndexedExtractor {
    Utf8Text,
    PdfText,
    XlsxCells,
    ImageOcr,
}
impl IndexedExtractor {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Utf8Text => "utf8_text",
            Self::PdfText => "pdf_text",
            Self::XlsxCells => "xlsx_cells",
            Self::ImageOcr => "image_ocr",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexFilter {
    pub job_id: Option<String>,
    pub attachment_id: Option<String>,
    pub extractor: Option<IndexedExtractor>,
}
impl IndexFilter {
    pub fn validate(&self) -> Result<()> {
        for id in [&self.job_id, &self.attachment_id].into_iter().flatten() {
            store::uuid(id)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IndexedSource {
    pub job_id: String,
    pub attachment_id: String,
    pub generation_id: String,
    pub source_sha256: String,
    pub extractor: String,
    pub extractor_version: u32,
    pub config_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AttachmentTextLocator {
    ImageOcrRegionText {
        projection: OcrTextProjection,
        region_ordinal: u32,
        bbox: crate::contracts::OcrBoundingBox,
        confidence: String,
        coordinate_space: String,
        region_bytes: u32,
        text_byte_start: u32,
        text_byte_end: u32,
    },
    Utf8Bytes {
        byte_start: u32,
        byte_end: u32,
        line_start: u32,
        line_end: u32,
    },
    PdfPageText {
        page_number: u32,
        text_byte_start: u32,
        text_byte_end: u32,
        scope: PdfTextScope,
        limitations: Vec<PdfLimitation>,
    },
    XlsxCellText {
        projection: XlsxTextProjection,
        cell_ordinal: u32,
        sheet_id: u32,
        reference: String,
        field: XlsxTextField,
        field_bytes: u32,
        text_byte_start: u32,
        text_byte_end: u32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OcrTextProjection {
    OcrRegionTextV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum XlsxTextProjection {
    XlsxCellFieldsV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum XlsxTextField {
    CachedText,
    CachedNumber,
    CachedBoolean,
    CachedError,
    CachedDateIso,
    FormulaText,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AttachmentSearchFragment {
    pub ordinal: u32,
    pub text: String,
    pub locator: AttachmentTextLocator,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentIndexEntry {
    pub source: IndexedSource,
    pub fragment_count: u32,
    pub text_bytes: u32,
    pub content_hash: String,
    pub indexed_at: String,
    pub result_from_previous_attempt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexCoverageScope {
    SelectedGenerations,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexValidation {
    CanonicalTextAndPostings,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IndexSourceValidation {
    NotChecked,
    ReturnedCandidates,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentIndexCoverage {
    pub scope: IndexCoverageScope,
    pub index_validation: IndexValidation,
    pub source_validation: IndexSourceValidation,
    pub indexed_generations: u32,
    pub unindexed_supported_generations: u32,
    pub unsupported_generations: u32,
    pub jobs_without_generation: u32,
    pub fragments: u32,
    pub text_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentIndexPage {
    pub version: u32,
    pub profile: String,
    pub revision: u32,
    pub coverage: AttachmentIndexCoverage,
    pub items: Vec<AttachmentIndexEntry>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum AttachmentIndexResult {
    Get(AttachmentIndexPage),
    Put {
        revision: u32,
        entry: AttachmentIndexEntry,
        changed: bool,
    },
    Reset {
        revision: u32,
        removed_generations: u32,
        removed_fragments: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentSearchHit {
    pub source: IndexedSource,
    pub ordinal: u32,
    pub text: String,
    pub locator: AttachmentTextLocator,
    pub score: f64,
    pub result_from_previous_attempt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttachmentSearchPage {
    pub version: u32,
    pub profile: String,
    pub revision: u32,
    pub coverage: AttachmentIndexCoverage,
    pub items: Vec<AttachmentSearchHit>,
    pub next_cursor: Option<String>,
}
