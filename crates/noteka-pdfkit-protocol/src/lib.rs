//! Transporte privado del lector PDFKit. Sin acceso al núcleo o al filesystem.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VERSION: u16 = 1;
pub const PROFILE: &str = "apple_pdfkit_text_v1";
pub const HEADER_BYTES: usize = 24;
pub const MAX_SOURCE: usize = 1_048_576;
pub const MAX_TEXT: usize = 1_048_576;
pub const MAX_PAGES: usize = 100;
pub const MAX_STDOUT: usize = 8 * 1_048_576;
pub const MAX_STDERR: usize = 8192;
pub const MAX_MS: u32 = 5000;
pub const LIMITATIONS: [&str; 5] = [
    "system_parser_not_complete_pdf_integrity_validator",
    "selection_order_not_visual_reading_order",
    "no_ocr_structured_tables_or_geometry",
    "no_text_not_proof_of_visual_absence",
    "system_framework_result_may_vary_by_os",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Invalid;
pub type Result<T> = std::result::Result<T, Invalid>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub source_bytes: u32,
    pub remaining_ms: u32,
}
impl Header {
    pub fn encode(self) -> Result<[u8; HEADER_BYTES]> {
        if self.source_bytes == 0
            || self.source_bytes as usize > MAX_SOURCE
            || !(1..=MAX_MS).contains(&self.remaining_ms)
        {
            return Err(Invalid);
        }
        let mut out = [0; HEADER_BYTES];
        out[..8].copy_from_slice(b"NKPDK001");
        out[8..10].copy_from_slice(&VERSION.to_le_bytes());
        out[12..16].copy_from_slice(&self.source_bytes.to_le_bytes());
        out[16..20].copy_from_slice(&self.remaining_ms.to_le_bytes());
        Ok(out)
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        if raw.len() != HEADER_BYTES
            || &raw[..8] != b"NKPDK001"
            || raw[8..12] != [1, 0, 0, 0]
            || raw[20..24] != [0; 4]
        {
            return Err(Invalid);
        }
        let value = Self {
            source_bytes: u32::from_le_bytes(raw[12..16].try_into().map_err(|_| Invalid)?),
            remaining_ms: u32::from_le_bytes(raw[16..20].try_into().map_err(|_| Invalid)?),
        };
        value.encode()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerError {
    PdfWorkerProtocol,
    PdfInvalidFraming,
    PdfInvalidDocument,
    PdfEncrypted,
    PdfPageLimit,
    PdfInvalidText,
    PdfTextLimit,
    PdfProfileUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Success {
        source_sha256: String,
        document: Box<Document>,
    },
    Error {
        reason: WorkerError,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub version: u16,
    pub profile: String,
    pub outcome: Outcome,
}
impl Reply {
    pub fn new(outcome: Outcome) -> Self {
        Self {
            version: VERSION,
            profile: PROFILE.into(),
            outcome,
        }
    }
    pub fn decode(raw: &[u8], expected_hash: &str) -> Result<Self> {
        if raw.len() > MAX_STDOUT
            || expected_hash.len() != 64
            || !expected_hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Invalid);
        }
        let reply: Self = serde_json::from_slice(raw).map_err(|_| Invalid)?;
        if reply.version != VERSION || reply.profile != PROFILE {
            return Err(Invalid);
        }
        if let Outcome::Success {
            source_sha256,
            document,
        } = &reply.outcome
        {
            if source_sha256 != expected_hash {
                return Err(Invalid);
            }
            document.validate()?;
        }
        Ok(reply)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Engine {
    pub api: String,
    pub os_version: String,
    pub os_build: String,
}
impl Engine {
    fn validate(&self) -> Result<()> {
        if self.api != "PDFKit.PDFPage.attributedString"
            || self.os_version.len() > 32
            || self.os_build.is_empty()
            || self.os_build.len() > 64
            || !self.os_build.bytes().all(|b| b.is_ascii_alphanumeric())
        {
            return Err(Invalid);
        }
        let parts: Vec<_> = self.os_version.split('.').collect();
        if parts.len() != 3
            || parts.iter().any(|s| {
                s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) || s.parse::<u32>().is_err()
            })
            || parts[0].parse::<u32>().is_ok_and(|v| v < 13)
        {
            return Err(Invalid);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PageStatus {
    Text,
    NoText,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Page {
    pub number: u32,
    pub status: PageStatus,
    pub text: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub version: u16,
    pub engine: Engine,
    pub limitations: Vec<String>,
    pub page_count: u32,
    pub text_bytes: u32,
    pub pages: Vec<Page>,
}
impl Document {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION
            || self.pages.is_empty()
            || self.pages.len() > MAX_PAGES
            || self.page_count as usize != self.pages.len()
            || self.limitations != LIMITATIONS
        {
            return Err(Invalid);
        }
        self.engine.validate()?;
        let mut total = 0usize;
        for (i, page) in self.pages.iter().enumerate() {
            if page.number as usize != i + 1
                || page.text.contains('\0')
                || (page.status == PageStatus::NoText) != page.text.is_empty()
            {
                return Err(Invalid);
            }
            total = total.checked_add(page.text.len()).ok_or(Invalid)?;
            if total > MAX_TEXT {
                return Err(Invalid);
            }
        }
        if total != self.text_bytes as usize {
            return Err(Invalid);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
