use super::*;
use serde_json::{Value, json};

fn document() -> Document {
    Document {
        version: 1,
        engine: Engine {
            api: "PDFKit.PDFPage.attributedString".into(),
            os_version: "27.0.0".into(),
            os_build: "26A123".into(),
        },
        limitations: LIMITATIONS.iter().map(|s| (*s).into()).collect(),
        page_count: 1,
        text_bytes: 5,
        pages: vec![Page {
            number: 1,
            status: PageStatus::Text,
            text: "café".into(),
        }],
    }
}
fn reply() -> Value {
    serde_json::to_value(Reply::new(Outcome::Success {
        source_sha256: "a".repeat(64),
        document: Box::new(document()),
    }))
    .unwrap()
}
fn decode(value: &Value) -> Result<Reply> {
    Reply::decode(&serde_json::to_vec(value).unwrap(), &"a".repeat(64))
}

#[test]
fn header_boundaries_and_all_fixed_bytes() {
    for size in [1, MAX_SOURCE as u32] {
        for ms in [1, MAX_MS] {
            let h = Header {
                source_bytes: size,
                remaining_ms: ms,
            };
            let raw = h.encode().unwrap();
            assert_eq!(Header::decode(&raw), Ok(h));
            for i in (0..12).chain(20..24) {
                let mut changed = raw;
                changed[i] ^= 1;
                assert!(Header::decode(&changed).is_err());
            }
            assert!(Header::decode(&raw[..23]).is_err());
        }
    }
    for (size, ms) in [(0, 1), (MAX_SOURCE as u32 + 1, 1), (1, 0), (1, MAX_MS + 1)] {
        assert!(
            Header {
                source_bytes: size,
                remaining_ms: ms
            }
            .encode()
            .is_err()
        );
    }
}
#[test]
fn exact_unicode_roundtrip_and_no_text() {
    assert!(decode(&reply()).is_ok());
    let mut d = document();
    d.pages.push(Page {
        number: 2,
        status: PageStatus::NoText,
        text: String::new(),
    });
    d.page_count = 2;
    assert!(d.validate().is_ok());
    d.pages[0].text = "e\u{301}".into();
    d.text_bytes = 3;
    assert!(d.validate().is_ok());
}
#[test]
fn unknown_fields_all_envelopes_and_duplicates() {
    for path in [
        "",
        "/outcome",
        "/outcome/document",
        "/outcome/document/engine",
        "/outcome/document/pages/0",
    ] {
        let mut r = reply();
        let target = if path.is_empty() {
            &mut r
        } else {
            r.pointer_mut(path).unwrap()
        };
        target
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(1));
        assert!(decode(&r).is_err());
    }
    let raw = serde_json::to_string(&reply()).unwrap().replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(Reply::decode(raw.as_bytes(), &"a".repeat(64)).is_err());
}
#[test]
fn corrupt_every_semantic_field() {
    for (path, value) in [
        ("/version", json!(2)),
        ("/profile", json!("other")),
        ("/outcome/source_sha256", json!("b".repeat(64))),
        ("/outcome/document/version", json!(2)),
        ("/outcome/document/page_count", json!(2)),
        ("/outcome/document/text_bytes", json!(4)),
        ("/outcome/document/limitations", json!([])),
        ("/outcome/document/pages/0/number", json!(0)),
        ("/outcome/document/pages/0/status", json!("no_text")),
        ("/outcome/document/pages/0/text", json!("\0")),
        ("/outcome/document/engine/api", json!("PDFPage.string")),
        ("/outcome/document/engine/os_version", json!("12.0.0")),
        ("/outcome/document/engine/os_build", json!("a-b")),
    ] {
        let mut r = reply();
        *r.pointer_mut(path).unwrap() = value;
        assert!(decode(&r).is_err(), "{path}");
    }
}
#[test]
fn metadata_bounded_before_parsing() {
    for version in [
        "",
        "27",
        "27.0",
        "27.0.0.0",
        "27.-1.0",
        "27.4294967296.0",
        &".".repeat(4096),
    ] {
        let mut d = document();
        d.engine.os_version = version.into();
        assert!(d.validate().is_err());
    }
    let mut d = document();
    d.engine.os_build = "x".repeat(65);
    assert!(d.validate().is_err());
}
#[test]
fn page_and_text_limits_complete_rejection() {
    let mut d = document();
    d.pages = (1..=100)
        .map(|number| Page {
            number,
            status: PageStatus::NoText,
            text: String::new(),
        })
        .collect();
    d.page_count = 100;
    d.text_bytes = 0;
    assert!(d.validate().is_ok());
    d.pages.push(Page {
        number: 101,
        status: PageStatus::NoText,
        text: String::new(),
    });
    d.page_count = 101;
    assert!(d.validate().is_err());
    let mut d = document();
    d.pages[0].text = "x".repeat(MAX_TEXT);
    d.text_bytes = MAX_TEXT as u32;
    assert!(d.validate().is_ok());
    d.pages[0].text.push('x');
    d.text_bytes += 1;
    assert!(d.validate().is_err());
    d.pages.clear();
    d.page_count = 0;
    assert!(d.validate().is_err());
}
#[test]
fn json_pipes_errors_and_expected_hash() {
    for raw in [b"".as_slice(), b"{}", b"null", b"[]", b"{}{}", b"\xff"] {
        assert!(Reply::decode(raw, &"a".repeat(64)).is_err());
    }
    assert!(Reply::decode(&vec![32; MAX_STDOUT + 1], &"a".repeat(64)).is_err());
    let raw = serde_json::to_vec(&reply()).unwrap();
    for h in ["", &"A".repeat(64), &"a".repeat(63)] {
        assert!(Reply::decode(&raw, h).is_err());
    }
    let err = Reply::new(Outcome::Error {
        reason: WorkerError::PdfEncrypted,
    });
    assert_eq!(
        Reply::decode(&serde_json::to_vec(&err).unwrap(), &"a".repeat(64)).unwrap(),
        err
    );
    let mut e = serde_json::to_value(err).unwrap();
    e["outcome"]["reason"] = json!("unknown");
    assert!(decode(&e).is_err());
}
