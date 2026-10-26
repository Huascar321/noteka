//! Protocolo privado del worker PDFium. Sin parser, rutas, workspace ni SQLite.
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;
pub const EXTRACTOR_VERSION: u32 = 1;
pub const PARSER_NAME: &str = "pdfium";
pub const PARSER_VERSION: &str = "0.9.4/pdfium-7881";
pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
pub const MAX_PAGES: usize = 100;
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_STDERR_BYTES: usize = 8 * 1024;
pub const MAX_REMAINING_MS: u32 = 5_000;
pub const HEADER_BYTES: usize = 24;
const MAGIC: &[u8; 8] = b"NKPFM001";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidProtocol;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub source_bytes: u32,
    pub remaining_ms: u32,
}
impl Header {
    pub fn encode(self) -> Result<[u8; HEADER_BYTES], InvalidProtocol> {
        self.validate()?;
        let mut bytes = [0; HEADER_BYTES];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..10].copy_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.source_bytes.to_le_bytes());
        bytes[16..20].copy_from_slice(&self.remaining_ms.to_le_bytes());
        Ok(bytes)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, InvalidProtocol> {
        if bytes.len() != HEADER_BYTES
            || &bytes[..8] != MAGIC
            || bytes[8..10] != PROTOCOL_VERSION.to_le_bytes()
            || bytes[10..12] != [0; 2]
            || bytes[20..24] != [0; 4]
        {
            return Err(InvalidProtocol);
        }
        let header = Self {
            source_bytes: u32::from_le_bytes(
                bytes[12..16].try_into().map_err(|_| InvalidProtocol)?,
            ),
            remaining_ms: u32::from_le_bytes(
                bytes[16..20].try_into().map_err(|_| InvalidProtocol)?,
            ),
        };
        header.validate()?;
        Ok(header)
    }
    fn validate(self) -> Result<(), InvalidProtocol> {
        if self.source_bytes == 0
            || self.source_bytes as usize > MAX_SOURCE_BYTES
            || !(1..=MAX_REMAINING_MS).contains(&self.remaining_ms)
        {
            return Err(InvalidProtocol);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PageText {
    pub page_number: u32,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerError {
    InvalidPdf,
    PdfEncrypted,
    PdfUnsupportedFeature,
    PdfDecodeError,
    PdfPageLimit,
    PdfTextLimit,
    PdfWorkerProtocol,
    PdfWorkerFailed,
    PdfiumUnavailable,
    ExecutionDeadline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Success {
        source_sha256: String,
        pages: Vec<PageText>,
    },
    Error {
        reason: WorkerError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub protocol_version: u16,
    pub extractor_version: u32,
    pub parser_name: String,
    pub parser_version: String,
    pub outcome: Outcome,
}
impl Reply {
    pub fn new(outcome: Outcome) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            extractor_version: EXTRACTOR_VERSION,
            parser_name: PARSER_NAME.into(),
            parser_version: PARSER_VERSION.into(),
            outcome,
        }
    }
    pub fn validate(&self) -> Result<(), InvalidProtocol> {
        if self.protocol_version != PROTOCOL_VERSION
            || self.extractor_version != EXTRACTOR_VERSION
            || self.parser_name != PARSER_NAME
            || self.parser_version != PARSER_VERSION
        {
            return Err(InvalidProtocol);
        }
        if let Outcome::Success {
            source_sha256,
            pages,
        } = &self.outcome
        {
            if !is_sha256(source_sha256) || pages.is_empty() || pages.len() > MAX_PAGES {
                return Err(InvalidProtocol);
            }
            let mut total = 0usize;
            for (index, page) in pages.iter().enumerate() {
                if page.page_number as usize != index + 1 || page.text.contains('\0') {
                    return Err(InvalidProtocol);
                }
                total = total.checked_add(page.text.len()).ok_or(InvalidProtocol)?;
                if total > MAX_TEXT_BYTES {
                    return Err(InvalidProtocol);
                }
            }
        }
        Ok(())
    }
}
pub fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn header_and_reply_are_strict() {
        let header = Header {
            source_bytes: 10,
            remaining_ms: 100,
        };
        let bytes = header.encode().unwrap();
        assert_eq!(Header::decode(&bytes).unwrap(), header);
        assert_eq!(
            Reply::new(Outcome::Success {
                source_sha256: "a".repeat(64),
                pages: vec![PageText {
                    page_number: 1,
                    text: "hola".into()
                }]
            })
            .validate(),
            Ok(())
        );
    }
}
