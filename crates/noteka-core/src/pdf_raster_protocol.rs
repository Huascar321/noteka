//! Contrato privado del rasterizador; no publica ni abre originales.
use base64::{Engine, engine::general_purpose::STANDARD};
use noteka_ocr_protocol::v2::OcrEngine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub(crate) const PROFILE: &str = "cgpdf_media_gray_scale2_png_v1";
pub(crate) const MAX_SOURCE: usize = 1_048_576;
pub(crate) const MAX_STDOUT: usize = 8 * MAX_SOURCE;
pub(crate) const MAX_STDERR: usize = 8192;
pub(crate) const MAX_DOCUMENT_PIXELS: u64 = 16_000_000;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Invalid;
type Result<T> = std::result::Result<T, Invalid>;

pub(crate) fn header(size: usize, ms: u32, page: u32) -> Result<[u8; 28]> {
    if size == 0 || size > MAX_SOURCE || !(1..=5000).contains(&ms) || !(1..=20).contains(&page) {
        return Err(Invalid);
    }
    let mut h = [0; 28];
    h[..8].copy_from_slice(b"NKPDFR01");
    h[8..10].copy_from_slice(&1u16.to_le_bytes());
    h[12..16].copy_from_slice(&(size as u32).to_le_bytes());
    h[16..20].copy_from_slice(&ms.to_le_bytes());
    h[20..24].copy_from_slice(&page.to_le_bytes());
    Ok(h)
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RasterIdentity {
    pub page_count: u32,
    pub page_number: u32,
    pub total_pixels: u64,
    pub box_rect: [String; 4],
    pub rotation: u16,
    pub width: u32,
    pub height: u32,
    pub png_sha256: String,
    pub png_bytes: u32,
    pub os_version: String,
    pub os_build: String,
}
#[derive(Debug)]
pub(crate) struct Raster {
    pub identity: RasterIdentity,
    pub png: Vec<u8>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorkerError {
    #[serde(rename = "pdf_ocr_raster_protocol")]
    RasterProtocol,
    #[serde(rename = "pdf_invalid_framing")]
    InvalidFraming,
    #[serde(rename = "pdf_invalid_document")]
    InvalidDocument,
    #[serde(rename = "pdf_encrypted")]
    Encrypted,
    #[serde(rename = "pdf_ocr_page_limit")]
    PageLimit,
    #[serde(rename = "pdf_ocr_geometry_limit")]
    GeometryLimit,
    #[serde(rename = "pdf_ocr_pixel_limit")]
    PixelLimit,
    #[serde(rename = "pdf_ocr_raster_limit")]
    RasterLimit,
    #[serde(rename = "pdf_ocr_render_failed")]
    RenderFailed,
    #[serde(rename = "pdf_ocr_profile_unavailable")]
    ProfileUnavailable,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRaster {
    page_count: u32,
    page_number: u32,
    total_pixels: u64,
    #[serde(rename = "box")]
    box_rect: [String; 4],
    rotation: u16,
    width: u32,
    height: u32,
    png: String,
    os_version: String,
    os_build: String,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum Outcome {
    Success {
        source_sha256: String,
        raster: Box<WireRaster>,
    },
    Error {
        reason: WorkerError,
    },
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    version: u16,
    profile: String,
    outcome: Outcome,
}
pub(crate) enum Decoded {
    Raster(Box<Raster>),
    Rejected(WorkerError),
}
pub(crate) fn valid_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn png_framing(png: &[u8], width: u32, height: u32) -> Result<()> {
    if png.len() < 57 || png.len() > MAX_SOURCE || &png[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(Invalid);
    }
    let (mut offset, mut image_data, mut after_data) = (8usize, false, false);
    while offset < png.len() {
        let head = png.get(offset..offset + 8).ok_or(Invalid)?;
        let len = u32::from_be_bytes(head[..4].try_into().map_err(|_| Invalid)?) as usize;
        let end = offset
            .checked_add(12)
            .and_then(|n| n.checked_add(len))
            .ok_or(Invalid)?;
        let chunk = png.get(offset..end).ok_or(Invalid)?;
        let kind = &head[4..8];
        if !kind.iter().all(u8::is_ascii_alphabetic)
            || kind[2].is_ascii_lowercase()
            || crc32fast::hash(&chunk[4..8 + len]).to_be_bytes() != chunk[8 + len..]
        {
            return Err(Invalid);
        }
        match kind {
            b"IHDR" if offset == 8 && len == 13 => {
                if chunk[8..12] != width.to_be_bytes()
                    || chunk[12..16] != height.to_be_bytes()
                    || chunk[16..21] != [8, 0, 0, 0, 0]
                {
                    return Err(Invalid);
                }
            }
            b"IDAT" if offset > 8 && !after_data && len > 0 => image_data = true,
            b"IEND" if image_data && len == 0 && end == png.len() => return Ok(()),
            _ if offset > 8 && kind[0].is_ascii_lowercase() => after_data |= image_data,
            _ => return Err(Invalid),
        }
        offset = end;
    }
    Err(Invalid)
}
impl RasterIdentity {
    pub(crate) fn normalize(mut self) -> Result<Self> {
        if !(1..=20).contains(&self.page_count)
            || !(1..=self.page_count).contains(&self.page_number)
            || ![0, 90, 180, 270].contains(&self.rotation)
            || self.width == 0
            || self.height == 0
            || self.width > 4096
            || self.height > 4096
            || u64::from(self.width) * u64::from(self.height) > 4_000_000
            || self.total_pixels < u64::from(self.width) * u64::from(self.height)
            || self.total_pixels > MAX_DOCUMENT_PIXELS
            || !valid_hash(&self.png_sha256)
            || self.png_bytes == 0
            || self.png_bytes as usize > MAX_SOURCE
        {
            return Err(Invalid);
        }
        OcrEngine {
            revision: 3,
            recognition_level: "fast".into(),
            languages: vec!["es-ES".into(), "en-US".into()],
            language_correction: false,
            automatic_language_detection: false,
            os_version: self.os_version.clone(),
            os_build: self.os_build.clone(),
        }
        .validate()
        .map_err(|_| Invalid)?;
        let mut rect = [0.0; 4];
        for (i, s) in self.box_rect.iter_mut().enumerate() {
            if s.is_empty() || s.len() > 512 || s.trim() != s {
                return Err(Invalid);
            }
            let n = s.parse::<f64>().map_err(|_| Invalid)?;
            if !n.is_finite() || n.abs() > 1_000_000.0 || (i >= 2 && n <= 0.0) {
                return Err(Invalid);
            }
            rect[i] = n;
            *s = n.to_string();
        }
        let (w, h) = if self.rotation == 90 || self.rotation == 270 {
            (rect[3], rect[2])
        } else {
            (rect[2], rect[3])
        };
        let scale = 2.0f64
            .min(4096.0 / w.max(h))
            .min((4_000_000.0 / (w * h)).sqrt());
        if (self.width, self.height)
            != (
                (w * scale).floor().max(1.0) as u32,
                (h * scale).floor().max(1.0) as u32,
            )
        {
            return Err(Invalid);
        }
        Ok(self)
    }
}
pub(crate) fn decode(bytes: &[u8], expected_hash: &str, page: u32) -> Result<Decoded> {
    if bytes.len() > MAX_STDOUT || !valid_hash(expected_hash) {
        return Err(Invalid);
    }
    let reply: Reply = serde_json::from_slice(bytes).map_err(|_| Invalid)?;
    if reply.version != 1 || reply.profile != PROFILE {
        return Err(Invalid);
    }
    match reply.outcome {
        Outcome::Error { reason } => Ok(Decoded::Rejected(reason)),
        Outcome::Success {
            source_sha256,
            raster: r,
        } => {
            let r = *r;
            if source_sha256 != expected_hash
                || r.page_number != page
                || r.png.len() > MAX_SOURCE.div_ceil(3) * 4
            {
                return Err(Invalid);
            }
            let png = STANDARD.decode(r.png).map_err(|_| Invalid)?;
            png_framing(&png, r.width, r.height)?;
            let identity = RasterIdentity {
                page_count: r.page_count,
                page_number: r.page_number,
                total_pixels: r.total_pixels,
                box_rect: r.box_rect,
                rotation: r.rotation,
                width: r.width,
                height: r.height,
                png_sha256: format!("{:x}", Sha256::digest(&png)),
                png_bytes: png.len() as u32,
                os_version: r.os_version,
                os_build: r.os_build,
            }
            .normalize()?;
            Ok(Decoded::Raster(Box::new(Raster { identity, png })))
        }
    }
}
