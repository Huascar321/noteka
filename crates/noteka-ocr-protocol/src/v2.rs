//! V2 transporta los escalares como cadenas; no cambia el codec JSON global.
use super::*;
pub const VERSION: u16 = 2;
pub const MAX_SCALAR_BYTES: usize = 512;
impl Header {
    pub fn encode_v2(self) -> Result<[u8; HEADER_BYTES]> {
        let mut h = self.encode()?;
        h[..8].copy_from_slice(b"NKOCR002");
        h[8..10].copy_from_slice(&VERSION.to_le_bytes());
        Ok(h)
    }
    pub fn decode_v2(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != HEADER_BYTES
            || &bytes[..8] != b"NKOCR002"
            || bytes[8..10] != VERSION.to_le_bytes()
        {
            return Err(Invalid);
        }
        let mut h = [0; HEADER_BYTES];
        h.copy_from_slice(bytes);
        h[..8].copy_from_slice(b"NKOCR001");
        h[8..10].copy_from_slice(&super::VERSION.to_le_bytes());
        Self::decode(&h)
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OcrImageFormat {
    Png,
    Jpeg,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OcrImage {
    pub format: OcrImageFormat,
    pub width: u32,
    pub height: u32,
    pub orientation: u8,
    pub oriented_width: u32,
    pub oriented_height: u32,
}
impl OcrImage {
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
pub struct OcrEngine {
    pub revision: u32,
    pub recognition_level: String,
    pub languages: Vec<String>,
    pub language_correction: bool,
    pub automatic_language_detection: bool,
    pub os_version: String,
    pub os_build: String,
}
impl OcrEngine {
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

fn scalar(s: &str) -> Result<f64> {
    if s.is_empty() || s.len() > MAX_SCALAR_BYTES || s.trim() != s {
        return Err(Invalid);
    }
    let v = s.parse::<f64>().map_err(|_| Invalid)?;
    if !v.is_finite() || !(0.0..=1.0).contains(&v) {
        return Err(Invalid);
    }
    Ok(v)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OcrBoundingBox {
    pub x: String,
    pub y: String,
    pub width: String,
    pub height: String,
}
impl OcrBoundingBox {
    fn validate(&self) -> Result<()> {
        let (x, y, w, h) = (
            scalar(&self.x)?,
            scalar(&self.y)?,
            scalar(&self.width)?,
            scalar(&self.height)?,
        );
        if w <= 0.0 || h <= 0.0 || x + w > 1.0 || y + h > 1.0 {
            return Err(Invalid);
        }
        Ok(())
    }
    fn normalize(&mut self) -> Result<()> {
        self.validate()?;
        for s in [&mut self.x, &mut self.y, &mut self.width, &mut self.height] {
            *s = scalar(s)?.to_string();
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OcrRegion {
    pub ordinal: u32,
    pub text: String,
    pub confidence: String,
    pub bbox: OcrBoundingBox,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OcrRecognition {
    pub version: u32,
    pub image: OcrImage,
    pub engine: OcrEngine,
    pub coordinate_space: String,
    pub limitations: Vec<String>,
    pub regions: Vec<OcrRegion>,
}
impl OcrRecognition {
    fn shape(&self) -> Result<()> {
        self.image.validate()?;
        self.engine.validate()?;
        if self.version != 1
            || self.coordinate_space != COORDINATE_SPACE
            || self.limitations != LIMITATIONS
            || self.regions.len() > MAX_REGIONS
        {
            return Err(Invalid);
        }
        for (i, r) in self.regions.iter().enumerate() {
            if r.ordinal as usize != i
                || r.text.is_empty()
                || r.text.len() > MAX_TEXT
                || r.text.contains('\0')
            {
                return Err(Invalid);
            }
            scalar(&r.confidence)?;
            r.bbox.validate()?;
        }
        Ok(())
    }
    pub fn normalize(mut self) -> Result<Self> {
        self.shape()?;
        for r in &mut self.regions {
            r.confidence = scalar(&r.confidence)?.to_string();
            r.bbox.normalize()?;
        }
        self.validate()?;
        Ok(self)
    }
    pub fn validate(&self) -> Result<()> {
        self.shape()?;
        for r in &self.regions {
            for s in [
                &r.confidence,
                &r.bbox.x,
                &r.bbox.y,
                &r.bbox.width,
                &r.bbox.height,
            ] {
                if scalar(s)?.to_string() != *s {
                    return Err(Invalid);
                }
            }
        }
        if serde_json::to_vec(self).map_err(|_| Invalid)?.len() > MAX_RESULT {
            return Err(Invalid);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Success {
        source_sha256: String,
        recognition: Box<OcrRecognition>,
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
    pub fn decode(bytes: &[u8], expected_hash: &str) -> Result<Self> {
        if bytes.len() > MAX_STDOUT || !super::sha256(expected_hash) {
            return Err(Invalid);
        }
        let mut r: Self = serde_json::from_slice(bytes).map_err(|_| Invalid)?;
        if r.version != VERSION || r.profile != PROFILE {
            return Err(Invalid);
        }
        if let Outcome::Success {
            source_sha256,
            recognition,
        } = &mut r.outcome
        {
            if source_sha256 != expected_hash {
                return Err(Invalid);
            }
            **recognition = (**recognition).clone().normalize()?;
        }
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> OcrRecognition {
        OcrRecognition {
            version: 1,
            image: OcrImage {
                format: OcrImageFormat::Png,
                width: 1200,
                height: 300,
                orientation: 1,
                oriented_width: 1200,
                oriented_height: 300,
            },
            engine: OcrEngine {
                revision: 3,
                recognition_level: "fast".into(),
                languages: vec!["es-ES".into(), "en-US".into()],
                language_correction: false,
                automatic_language_detection: false,
                os_version: "13.0.0".into(),
                os_build: "22A380".into(),
            },
            coordinate_space: COORDINATE_SPACE.into(),
            limitations: LIMITATIONS.iter().map(|s| s.to_string()).collect(),
            regions: vec![OcrRegion {
                ordinal: 0,
                text: "café 🚀\r\n<script>".into(),
                confidence: "0.5".into(),
                bbox: OcrBoundingBox {
                    x: "0.1".into(),
                    y: "0.2".into(),
                    width: "0.8".into(),
                    height: "0.3".into(),
                },
            }],
        }
    }
    #[test]
    fn header_versions_are_strict_and_independent() {
        let h = Header {
            source_bytes: 1,
            remaining_ms: 5000,
        };
        let v1 = h.encode().unwrap();
        let v2 = h.encode_v2().unwrap();
        assert_eq!(Header::decode_v2(&v2).unwrap(), h);
        assert!(Header::decode_v2(&v1).is_err());
        assert!(Header::decode(&v2).is_err());
        for i in (0..12).chain(20..24) {
            let mut bad = v2;
            bad[i] ^= 255;
            assert!(Header::decode_v2(&bad).is_err());
        }
    }
    #[test]
    fn decimal_transport_round_trips_bits_including_subnormals_and_negative_zero() {
        let mut bits = 0x1234567890abcdefu64;
        let fixed = [
            -0.0,
            0.0,
            1.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            0.10000000000000002,
            0.9999999999999999,
        ];
        for value in fixed.into_iter().chain((0..20000).map(|_| {
            bits = bits.wrapping_mul(6364136223846793005).wrapping_add(1);
            f64::from_bits(bits % 0x3ff0000000000001)
        })) {
            let mut r = sample();
            r.regions[0].confidence = format!("{value:e}");
            let r = r.normalize().unwrap();
            let actual = scalar(&r.regions[0].confidence).unwrap();
            assert_eq!(value.to_bits(), actual.to_bits());
            let raw = serde_json::to_vec(&r).unwrap();
            let decoded: OcrRecognition = serde_json::from_slice(&raw).unwrap();
            decoded.validate().unwrap();
            assert_eq!(decoded, r);
        }
    }
    #[test]
    fn rejects_unbounded_noncanonical_and_invalid_geometry_without_truncating() {
        for bad in [
            "NaN",
            "inf",
            "-0.1",
            "1.1",
            " 0.5",
            "0.5 ",
            "",
            &"0".repeat(513),
        ] {
            let mut r = sample();
            r.regions[0].confidence = bad.into();
            assert!(r.normalize().is_err());
        }
        let mut r = sample();
        r.regions[0].confidence = "5e-1".into();
        assert!(r.validate().is_err());
        assert_eq!(r.normalize().unwrap().regions[0].confidence, "0.5");
        for value in ["0", "-0", "0.99"] {
            let mut r = sample();
            r.regions[0].bbox.width = value.into();
            assert!(r.normalize().is_err());
        }
        let mut r = sample();
        r.regions = vec![r.regions[0].clone(); 513];
        assert!(r.normalize().is_err());
        let mut r = sample();
        r.regions[0].text = "a".repeat(MAX_TEXT + 1);
        assert!(r.normalize().is_err());
        let mut r = sample();
        r.engine.os_version = ".".repeat(MAX_RESULT);
        assert!(r.normalize().is_err());
    }
    #[test]
    fn reply_is_bound_to_version_profile_and_source() {
        let hash = "a".repeat(64);
        let r = Reply {
            version: 2,
            profile: PROFILE.into(),
            outcome: Outcome::Success {
                source_sha256: hash.clone(),
                recognition: Box::new(sample()),
            },
        };
        let raw = serde_json::to_vec(&r).unwrap();
        assert_eq!(Reply::decode(&raw, &hash).unwrap(), r);
        assert!(super::super::Reply::decode(&raw, &hash).is_err());
        for (key, value) in [
            ("version", serde_json::json!(1)),
            ("profile", serde_json::json!("other")),
            ("unknown", serde_json::json!(0)),
        ] {
            let mut v = serde_json::to_value(&r).unwrap();
            v[key] = value;
            assert!(Reply::decode(&serde_json::to_vec(&v).unwrap(), &hash).is_err());
        }
        assert!(Reply::decode(&raw, &"b".repeat(64)).is_err());
    }
}
