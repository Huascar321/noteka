//! Contrato privado OCR; no abre workspaces, originales o proveedores.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VERSION: u16 = 1;
pub const PROFILE: &str = "apple_vision_r3_fast_es_en_v1";
pub const HEADER_BYTES: usize = 24;
pub const MAX_SOURCE: usize = 1_048_576;
pub const MAX_AXIS: u32 = 4096;
pub const MAX_PIXELS: u64 = 4_000_000;
pub const MAX_REGIONS: usize = 512;
pub const MAX_TEXT: usize = 131_072;
pub const MAX_RESULT: usize = 1_048_576;
pub const MAX_STDOUT: usize = 8 * 1_048_576;
pub const MAX_STDERR: usize = 8192;
pub const MAX_MS: u32 = 5000;
pub const RSS_OBSERVED_BYTES: u64 = 128 * 1_048_576;
pub const COORDINATE_SPACE: &str = "oriented_normalized_bottom_left";
pub const LIMITATIONS: [&str; 5] = [
    "inferred_text_may_be_incorrect_or_missing",
    "observation_order_not_guaranteed_reading_order",
    "no_layout_tables_or_visual_description",
    "confidence_not_probability_of_correctness",
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
        out[..8].copy_from_slice(b"NKOCR001");
        out[8..10].copy_from_slice(&VERSION.to_le_bytes());
        out[12..16].copy_from_slice(&self.source_bytes.to_le_bytes());
        out[16..20].copy_from_slice(&self.remaining_ms.to_le_bytes());
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != HEADER_BYTES
            || &bytes[..8] != b"NKOCR001"
            || u16::from_le_bytes(bytes[8..10].try_into().map_err(|_| Invalid)?) != VERSION
            || bytes[10..12] != [0; 2]
            || bytes[20..24] != [0; 4]
        {
            return Err(Invalid);
        }
        let out = Self {
            source_bytes: u32::from_le_bytes(bytes[12..16].try_into().map_err(|_| Invalid)?),
            remaining_ms: u32::from_le_bytes(bytes[16..20].try_into().map_err(|_| Invalid)?),
        };
        out.encode()?;
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerError {
    OcrWorkerProtocol,
    OcrInvalidImage,
    OcrUnsupportedFormat,
    OcrMultipleImages,
    OcrImageLimit,
    OcrInvalidOrientation,
    OcrProfileUnavailable,
    OcrRecognitionFailed,
    OcrResultLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Success {
        source_sha256: String,
        recognition: Box<Recognition>,
    },
    Error {
        reason: WorkerError,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub fn decode(bytes: &[u8], expected_hash: &str) -> Result<Self> {
        if bytes.len() > MAX_STDOUT || !sha256(expected_hash) {
            return Err(Invalid);
        }
        let reply: Self = serde_json::from_slice(bytes).map_err(|_| Invalid)?;
        if reply.version != VERSION || reply.profile != PROFILE {
            return Err(Invalid);
        }
        if let Outcome::Success {
            source_sha256,
            recognition,
        } = &reply.outcome
        {
            if source_sha256 != expected_hash {
                return Err(Invalid);
            }
            recognition.validate()?;
        }
        Ok(reply)
    }
}
fn sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
    Jpeg,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub orientation: u8,
    pub oriented_width: u32,
    pub oriented_height: u32,
}
impl Image {
    pub fn validate(&self) -> Result<()> {
        if self.width == 0
            || self.height == 0
            || self.width > MAX_AXIS
            || self.height > MAX_AXIS
            || u64::from(self.width) * u64::from(self.height) > MAX_PIXELS
            || !(1..=8).contains(&self.orientation)
        {
            return Err(Invalid);
        }
        let (width, height) = if self.orientation >= 5 {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        if (self.oriented_width, self.oriented_height) != (width, height) {
            return Err(Invalid);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Engine {
    pub revision: u32,
    pub recognition_level: String,
    pub languages: Vec<String>,
    pub language_correction: bool,
    pub automatic_language_detection: bool,
    pub os_version: String,
    pub os_build: String,
}
impl Engine {
    pub fn validate(&self) -> Result<()> {
        // Acotar antes de split/collect evita expandir metadatos hostiles.
        if self.os_version.len() > 32 {
            return Err(Invalid);
        }
        let version: Vec<_> = self.os_version.split('.').collect();
        if self.revision != 3
            || self.recognition_level != "fast"
            || self.languages != ["es-ES", "en-US"]
            || self.language_correction
            || self.automatic_language_detection
            || version.len() != 3
            || version.iter().any(|s| {
                s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) || s.parse::<u32>().is_err()
            })
            || version[0].parse::<u32>().is_ok_and(|major| major < 13)
            || self.os_build.is_empty()
            || self.os_build.len() > 64
            || !self.os_build.bytes().all(|b| b.is_ascii_alphanumeric())
        {
            return Err(Invalid);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl BoundingBox {
    pub fn validate(&self) -> Result<()> {
        if [self.x, self.y, self.width, self.height]
            .iter()
            .any(|v| !v.is_finite())
            || self.x < 0.0
            || self.y < 0.0
            || self.width <= 0.0
            || self.height <= 0.0
            || self.x + self.width > 1.0
            || self.y + self.height > 1.0
        {
            return Err(Invalid);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub ordinal: u32,
    pub text: String,
    pub confidence: f64,
    pub bbox: BoundingBox,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Recognition {
    pub image: Image,
    pub engine: Engine,
    pub coordinate_space: String,
    pub limitations: Vec<String>,
    pub regions: Vec<Region>,
}
impl Recognition {
    pub fn validate(&self) -> Result<()> {
        self.image.validate()?;
        self.engine.validate()?;
        if self.coordinate_space != COORDINATE_SPACE
            || self.limitations != LIMITATIONS
            || self.regions.len() > MAX_REGIONS
        {
            return Err(Invalid);
        }
        for (ordinal, region) in self.regions.iter().enumerate() {
            if region.ordinal as usize != ordinal
                || region.text.is_empty()
                || region.text.len() > MAX_TEXT
                || region.text.contains('\0')
                || !region.confidence.is_finite()
                || !(0.0..=1.0).contains(&region.confidence)
            {
                return Err(Invalid);
            }
            region.bbox.validate()?;
        }
        if serde_json::to_vec(self).map_err(|_| Invalid)?.len() > MAX_RESULT {
            return Err(Invalid);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

/// Transporte decimal estable; v1 permanece independiente.
pub mod v2;
