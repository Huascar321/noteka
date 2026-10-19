//! Transporte privado y acotado. No contiene parser, rutas, ownership ni SQLite.
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u16 = 1;
pub const EXTRACTOR_VERSION: u32 = 1;
pub const PARSER_NAME: &str = "lopdf";
pub const PARSER_VERSION: &str = "0.45.0";
pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
pub const MAX_PAGES: usize = 100;
pub const MAX_STDOUT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_STDERR_BYTES: usize = 8 * 1024;
pub const MAX_REMAINING_MS: u32 = 5_000;
pub const HEADER_BYTES: usize = 24;
const MAGIC: &[u8; 8] = b"NKPDF001";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidProtocol;

/// Enteros little-endian. Los bytes reservados deben ser cero; no hay extensiones implícitas.
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
    PdfStreamLimit,
    PdfMemoryLimit,
    PdfWorkerProtocol,
    PdfWorkerFailed,
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

    /// El padre debe comparar además el hash contra los bytes que envió.
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

    fn success() -> Reply {
        Reply::new(Outcome::Success {
            source_sha256: "a".repeat(64),
            pages: vec![PageText {
                page_number: 1,
                text: "á\n".into(),
            }],
        })
    }

    #[test]
    fn header_is_fixed_strict_and_bounded() {
        for source_bytes in [1, MAX_SOURCE_BYTES as u32] {
            for remaining_ms in [1, MAX_REMAINING_MS] {
                let header = Header {
                    source_bytes,
                    remaining_ms,
                };
                let bytes = header.encode().unwrap();
                assert_eq!(Header::decode(&bytes).unwrap(), header);
                for index in [0, 8, 10, 20] {
                    let mut corrupt = bytes;
                    corrupt[index] ^= 0xff;
                    assert_eq!(Header::decode(&corrupt), Err(InvalidProtocol));
                }
                assert_eq!(Header::decode(&bytes[..23]), Err(InvalidProtocol));
                let mut extra = bytes.to_vec();
                extra.push(0);
                assert_eq!(Header::decode(&extra), Err(InvalidProtocol));
            }
        }
        for header in [
            Header {
                source_bytes: 0,
                remaining_ms: 1,
            },
            Header {
                source_bytes: MAX_SOURCE_BYTES as u32 + 1,
                remaining_ms: 1,
            },
            Header {
                source_bytes: 1,
                remaining_ms: 0,
            },
            Header {
                source_bytes: 1,
                remaining_ms: MAX_REMAINING_MS + 1,
            },
        ] {
            assert_eq!(header.encode(), Err(InvalidProtocol));
        }
    }

    #[test]
    fn reply_rejects_unknown_fields_and_unknown_errors() {
        let value = serde_json::to_value(success()).unwrap();
        for field in ["password", "helper_path", "workspace", "token", "sql"] {
            let mut bad = value.clone();
            bad[field] = "private".into();
            assert!(serde_json::from_value::<Reply>(bad).is_err());
        }
        for path in ["outcome", "page"] {
            let mut bad = value.clone();
            if path == "page" {
                bad["outcome"]["pages"][0]["byte_start"] = 0.into();
            } else {
                bad["outcome"]["extra"] = true.into();
            }
            assert!(serde_json::from_value::<Reply>(bad).is_err());
        }
        assert!(serde_json::from_str::<WorkerError>("\"arbitrary_message\"").is_err());
    }

    #[test]
    fn reply_checks_versions_hash_order_and_limits() {
        let good = success();
        good.validate().unwrap();
        for field in [
            "protocol_version",
            "extractor_version",
            "parser_name",
            "parser_version",
        ] {
            let mut bad = serde_json::to_value(&good).unwrap();
            bad[field] = if field.ends_with("version") && field != "parser_version" {
                2.into()
            } else {
                "other".into()
            };
            assert!(
                serde_json::from_value::<Reply>(bad)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
        let mut reply = good;
        if let Outcome::Success {
            pages,
            source_sha256,
        } = &mut reply.outcome
        {
            pages[0].text = "é".repeat(MAX_TEXT_BYTES / 2);
            assert_eq!(pages[0].text.len(), MAX_TEXT_BYTES);
            source_sha256.replace_range(0..1, "F");
        }
        assert!(reply.validate().is_err());
        if let Outcome::Success {
            pages,
            source_sha256,
        } = &mut reply.outcome
        {
            *source_sha256 = "b".repeat(64);
            pages[0].text.truncate(MAX_TEXT_BYTES);
        }
        reply.validate().unwrap();
        if let Outcome::Success { pages, .. } = &mut reply.outcome {
            pages[0].text.push('a');
        }
        assert!(reply.validate().is_err());
        for count in [0, MAX_PAGES, MAX_PAGES + 1] {
            let reply = Reply::new(Outcome::Success {
                source_sha256: "a".repeat(64),
                pages: (1..=count)
                    .map(|n| PageText {
                        page_number: n as u32,
                        text: String::new(),
                    })
                    .collect(),
            });
            assert_eq!(reply.validate().is_ok(), count == MAX_PAGES);
        }
        for (page_number, text) in [(2, "ok"), (1, "bad\0")] {
            let reply = Reply::new(Outcome::Success {
                source_sha256: "a".repeat(64),
                pages: vec![PageText {
                    page_number,
                    text: text.into(),
                }],
            });
            assert!(reply.validate().is_err());
        }
    }
}
