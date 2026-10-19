use super::*;
use serde_json::{Value, json};

fn region(ordinal: u32) -> Region {
    Region {
        ordinal,
        text: "café <script>\r\nenergía".into(),
        confidence: 0.5,
        bbox: BoundingBox {
            x: 0.1,
            y: 0.2,
            width: 0.8,
            height: 0.3,
        },
    }
}
fn recognition() -> Recognition {
    Recognition {
        image: Image {
            format: ImageFormat::Png,
            width: 1200,
            height: 300,
            orientation: 1,
            oriented_width: 1200,
            oriented_height: 300,
        },
        engine: Engine {
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
        regions: vec![region(0)],
    }
}
fn reply() -> Reply {
    Reply::new(Outcome::Success {
        source_sha256: "a".repeat(64),
        recognition: Box::new(recognition()),
    })
}

#[test]
fn header_round_trips_every_frontier_and_rejects_invalid_values() {
    for source_bytes in [1, MAX_SOURCE as u32] {
        for remaining_ms in [1, MAX_MS] {
            let h = Header {
                source_bytes,
                remaining_ms,
            };
            assert_eq!(Header::decode(&h.encode().unwrap()).unwrap(), h);
        }
    }
    for (source_bytes, remaining_ms) in
        [(0, 1), (MAX_SOURCE as u32 + 1, 1), (1, 0), (1, MAX_MS + 1)]
    {
        assert!(
            Header {
                source_bytes,
                remaining_ms
            }
            .encode()
            .is_err()
        );
    }
}

#[test]
fn header_rejects_reserved_magic_versions_short_and_extra_bytes() {
    let valid = Header {
        source_bytes: 10,
        remaining_ms: 200,
    }
    .encode()
    .unwrap();
    for at in (0..12).chain(20..24) {
        let mut bad = valid;
        bad[at] ^= 0xff;
        assert!(Header::decode(&bad).is_err(), "byte {at}");
    }
    for len in 0..HEADER_BYTES {
        assert!(Header::decode(&valid[..len]).is_err());
    }
    let mut extra = valid.to_vec();
    extra.push(0);
    assert!(Header::decode(&extra).is_err());
}

#[test]
fn valid_and_empty_recognition_preserve_exact_text_and_provenance() {
    let expected = reply();
    let raw = serde_json::to_vec(&expected).unwrap();
    assert_eq!(Reply::decode(&raw, &"a".repeat(64)).unwrap(), expected);
    let mut empty = recognition();
    empty.regions.clear();
    assert!(empty.validate().is_ok());
    assert_eq!(recognition().regions[0].text, "café <script>\r\nenergía");
}

#[test]
fn image_all_orientations_and_pixel_axis_boundaries() {
    for orientation in 1..=8 {
        let mut image = recognition().image;
        image.orientation = orientation;
        if orientation >= 5 {
            (image.oriented_width, image.oriented_height) = (image.height, image.width);
        }
        assert!(image.validate().is_ok());
        image.oriented_width += 1;
        assert!(image.validate().is_err());
    }
    for (w, h, valid) in [
        (2000, 2000, true),
        (2001, 2000, false),
        (4096, 1, true),
        (4097, 1, false),
        (0, 1, false),
        (1, 0, false),
    ] {
        let image = Image {
            width: w,
            height: h,
            oriented_width: w,
            oriented_height: h,
            ..recognition().image
        };
        assert_eq!(image.validate().is_ok(), valid, "{w}x{h}");
    }
}

#[test]
fn every_identity_profile_shape_and_geometry_mutation_is_rejected() {
    let original = serde_json::to_value(reply()).unwrap();
    let cases: Vec<(&str, Value)> = vec![
        ("/version", json!(2)),
        ("/profile", json!("other")),
        ("/outcome/source_sha256", json!("0".repeat(64))),
        ("/outcome/recognition/image/width", json!(0)),
        ("/outcome/recognition/image/width", json!(4097)),
        ("/outcome/recognition/image/width", json!(1.5)),
        ("/outcome/recognition/image/format", json!("gif")),
        ("/outcome/recognition/image/orientation", json!(0)),
        ("/outcome/recognition/image/orientation", json!(9)),
        ("/outcome/recognition/image/oriented_width", json!(300)),
        ("/outcome/recognition/engine/revision", json!(2)),
        (
            "/outcome/recognition/engine/recognition_level",
            json!("accurate"),
        ),
        (
            "/outcome/recognition/engine/languages",
            json!(["en-US", "es-ES"]),
        ),
        (
            "/outcome/recognition/engine/language_correction",
            json!(true),
        ),
        (
            "/outcome/recognition/engine/automatic_language_detection",
            json!(true),
        ),
        ("/outcome/recognition/engine/os_version", json!("12.0.0")),
        ("/outcome/recognition/engine/os_version", json!("13")),
        ("/outcome/recognition/engine/os_version", json!("13.a.0")),
        ("/outcome/recognition/engine/os_version", json!("")),
        (
            "/outcome/recognition/engine/os_version",
            json!("9".repeat(40)),
        ),
        (
            "/outcome/recognition/engine/os_version",
            json!("13.".repeat(1_000_000)),
        ),
        ("/outcome/recognition/engine/os_build", json!("<script>")),
        ("/outcome/recognition/engine/os_build", json!("")),
        (
            "/outcome/recognition/coordinate_space",
            json!("source_bytes"),
        ),
        ("/outcome/recognition/limitations", json!([])),
        ("/outcome/recognition/regions/0/ordinal", json!(1)),
        ("/outcome/recognition/regions/0/ordinal", json!(-1)),
        ("/outcome/recognition/regions/0/text", json!("")),
        ("/outcome/recognition/regions/0/text", json!("nul\u{0}text")),
        ("/outcome/recognition/regions/0/confidence", json!(-0.1)),
        ("/outcome/recognition/regions/0/confidence", json!(1.1)),
        ("/outcome/recognition/regions/0/confidence", json!("NaN")),
        ("/outcome/recognition/regions/0/bbox/x", json!(-0.1)),
        ("/outcome/recognition/regions/0/bbox/y", json!(0.9)),
        ("/outcome/recognition/regions/0/bbox/width", json!(0)),
        ("/outcome/recognition/regions/0/bbox/height", json!(2)),
    ];
    for (pointer, value) in cases {
        let mut bad = original.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            Reply::decode(&serde_json::to_vec(&bad).unwrap(), &"a".repeat(64)).is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn unknown_fields_at_every_level_are_rejected() {
    for at in [
        "",
        "/outcome",
        "/outcome/recognition",
        "/outcome/recognition/image",
        "/outcome/recognition/engine",
        "/outcome/recognition/regions/0",
        "/outcome/recognition/regions/0/bbox",
    ] {
        let mut bad = serde_json::to_value(reply()).unwrap();
        bad.pointer_mut(at)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!("never trust"));
        assert!(
            Reply::decode(&serde_json::to_vec(&bad).unwrap(), &"a".repeat(64)).is_err(),
            "{at}"
        );
    }
}

#[test]
fn confidence_and_geometry_reject_nonfinite_and_accept_range_endpoints() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut r = recognition();
        r.regions[0].confidence = value;
        assert!(r.validate().is_err());
        for axis in 0..4 {
            let mut r = recognition();
            let b = &mut r.regions[0].bbox;
            match axis {
                0 => b.x = value,
                1 => b.y = value,
                2 => b.width = value,
                _ => b.height = value,
            };
            assert!(r.validate().is_err());
        }
    }
    for value in [0.0, 1.0] {
        let mut r = recognition();
        r.regions[0].confidence = value;
        assert!(r.validate().is_ok());
    }
}

#[test]
fn region_count_text_bytes_and_ordinal_frontiers_are_exact() {
    let mut r = recognition();
    r.regions = (0..MAX_REGIONS as u32).map(region).collect();
    assert!(r.validate().is_ok());
    r.regions.push(region(MAX_REGIONS as u32));
    assert!(r.validate().is_err());
    r.regions = vec![region(0)];
    r.regions[0].text = "é".repeat(MAX_TEXT / 2);
    assert!(r.validate().is_ok());
    r.regions[0].text.push('x');
    assert!(r.validate().is_err());
    r.regions = vec![region(0), region(0)];
    assert!(r.validate().is_err());
}

#[test]
fn representation_frontier_is_measured_without_truncation() {
    let mut r = recognition();
    r.regions = (0..8).map(region).collect();
    for row in &mut r.regions[..7] {
        row.text = "x".repeat(131000);
    }
    r.regions[7].text = "x".into();
    let size = serde_json::to_vec(&r).unwrap().len();
    assert!(size < MAX_RESULT);
    r.regions[7].text.push_str(&"x".repeat(MAX_RESULT - size));
    assert!(r.regions[7].text.len() <= MAX_TEXT);
    assert_eq!(serde_json::to_vec(&r).unwrap().len(), MAX_RESULT);
    assert!(r.validate().is_ok());
    r.regions[7].text.push('x');
    assert_eq!(serde_json::to_vec(&r).unwrap().len(), MAX_RESULT + 1);
    assert!(r.validate().is_err());
}

#[test]
fn errors_are_closed_and_never_echo_content() {
    for reason in [
        WorkerError::OcrWorkerProtocol,
        WorkerError::OcrInvalidImage,
        WorkerError::OcrUnsupportedFormat,
        WorkerError::OcrMultipleImages,
        WorkerError::OcrImageLimit,
        WorkerError::OcrInvalidOrientation,
        WorkerError::OcrProfileUnavailable,
        WorkerError::OcrRecognitionFailed,
        WorkerError::OcrResultLimit,
    ] {
        let expected = Reply::new(Outcome::Error { reason });
        let raw = serde_json::to_vec(&expected).unwrap();
        assert_eq!(Reply::decode(&raw, &"a".repeat(64)).unwrap(), expected);
    }
    for bad in [
        json!({"status":"error","reason":"source secret"}),
        json!({"status":"error","reason":"ocr_invalid_image","text":"secret"}),
    ] {
        let mut v = serde_json::to_value(reply()).unwrap();
        v["outcome"] = bad;
        assert!(Reply::decode(&serde_json::to_vec(&v).unwrap(), &"a".repeat(64)).is_err());
    }
}

#[test]
fn malformed_json_extra_payload_hash_and_output_size_are_rejected() {
    let raw = serde_json::to_vec(&reply()).unwrap();
    for hash in ["A".repeat(64), "a".repeat(63), "not a hash".into()] {
        assert!(Reply::decode(&raw, &hash).is_err());
    }
    for len in [0, 1, raw.len() - 1] {
        assert!(Reply::decode(&raw[..len], &"a".repeat(64)).is_err());
    }
    let mut extra = raw;
    extra.extend_from_slice(b"{}");
    assert!(Reply::decode(&extra, &"a".repeat(64)).is_err());
    assert!(Reply::decode(&vec![b' '; MAX_STDOUT + 1], &"a".repeat(64)).is_err());
}
