use crate::{
    batch_storage::Budget, extraction_storage as store, pdf_ocr_compositor as compositor,
    pdf_raster_protocol as protocol,
};
use std::time::Duration;
#[test]
fn pdf_ocr_headers_enforce_caps() {
    let h = protocol::header(1, 5000, 20).unwrap();
    assert_eq!(&h[..8], b"NKPDFR01");
    assert_eq!(&h[20..24], &20u32.to_le_bytes());
    for (size, ms, page) in [
        (0, 1, 1),
        (1048577, 1, 1),
        (1, 0, 1),
        (1, 5001, 1),
        (1, 1, 0),
        (1, 1, 21),
    ] {
        assert!(protocol::header(size, ms, page).is_err());
    }
}
#[test]
fn pdf_ocr_source_and_fences_precede_any_native_call() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    let b = Budget::new(&conn).unwrap();
    let spawned = crate::pdf_raster_supervisor::spawn_count();
    let e = compositor::run(b"", &b, || Ok(())).unwrap_err();
    assert_eq!(e.details["reason"], "source_size_limit");
    let e = compositor::run(b"test", &b, || Err(store::invalid("ownership_lost"))).unwrap_err();
    assert_eq!(e.details["reason"], "ownership_lost");
    assert_eq!(crate::pdf_raster_supervisor::spawn_count(), spawned);
    let b = Budget::with_duration(&conn, Duration::ZERO).unwrap();
    let e = compositor::run(b"test", &b, || Ok(())).unwrap_err();
    assert_eq!(e.details["reason"], "execution_deadline");
}
#[test]
#[ignore = "requires own external real scanned PDF corpus and both native helpers"]
fn pdf_ocr_real_corpus() {
    let root = std::path::PathBuf::from(
        std::env::var_os("NOTEKA_PDF_OCR_CORPUS").expect("external corpus"),
    );
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let mut results = Vec::new();
    for case in manifest["cases"].as_array().unwrap() {
        let bytes = std::fs::read(root.join(case["name"].as_str().unwrap())).unwrap();
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let b = Budget::new(&conn).unwrap();
        let start = std::time::Instant::now();
        let result = compositor::run(&bytes, &b, || Ok(()));
        if let Some(reason) = case["error"].as_str() {
            assert_eq!(
                result.unwrap_err().details["reason"],
                reason,
                "{}",
                case["name"]
            );
            results.push(serde_json::json!({"name":case["name"],"error":reason,"ms":start.elapsed().as_millis()}));
            continue;
        }
        let doc = result.unwrap_or_else(|e| panic!("{}: {:?}", case["name"], e));
        doc.validate().unwrap();
        assert_eq!(doc.pages.len() as u64, case["pages"].as_u64().unwrap());
        for p in &doc.pages {
            assert_eq!(p.raster.width as u64, case["width"].as_u64().unwrap());
            assert_eq!(p.raster.height as u64, case["height"].as_u64().unwrap());
            assert_eq!(p.raster.rotation as u64, case["rotation"].as_u64().unwrap());
        }
        results.push(serde_json::json!({"name":case["name"],"document":doc,"ms":start.elapsed().as_millis()}));
    }
    let output = std::path::PathBuf::from(
        std::env::var_os("NOTEKA_PDF_OCR_OUTPUT").expect("external report"),
    );
    assert!(!output.exists());
    std::fs::write(output, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
}

// Fixtures de contrato sintéticas; no representan salida de Vision.
pub(crate) fn document(n: u32, regions: u32) -> compositor::Document {
    use sha2::{Digest, Sha256};
    let pages = (1..=n)
        .map(|page| {
            let recognition = crate::ocr_tests::recognition(regions);
            compositor::Page {
                raster: protocol::RasterIdentity {
                    page_count: n,
                    page_number: page,
                    total_pixels: u64::from(n) * 1200 * 300,
                    box_rect: ["0".into(), "0".into(), "600".into(), "150".into()],
                    rotation: 0,
                    width: 1200,
                    height: 300,
                    png_sha256: format!(
                        "{:x}",
                        Sha256::digest(b"synthetic PNG fixture; no inference")
                    ),
                    png_bytes: b"synthetic PNG fixture; no inference".len() as u32,
                    os_version: "13.0.0".into(),
                    os_build: "22A380".into(),
                },
                recognition,
            }
        })
        .collect();
    compositor::Document {
        version: 1,
        profile: compositor::PROFILE.into(),
        raster_profile: protocol::PROFILE.into(),
        source_sha256: format!("{:x}", Sha256::digest(b"source")),
        pages,
    }
}
#[test]
fn pdf_ocr_documents_validate_every_page_and_exact_result_cap() {
    let doc = document(2, 4);
    doc.validate().unwrap();
    let value = serde_json::to_value(&doc).unwrap();
    for (pointer, bad) in [
        ("/version", serde_json::json!(2)),
        ("/profile", serde_json::json!("other")),
        ("/source_sha256", serde_json::json!("?".repeat(64))),
        ("/pages/1/raster/page_number", serde_json::json!(1)),
        ("/pages/1/raster/total_pixels", serde_json::json!(1)),
        ("/pages/1/raster/os_build", serde_json::json!("22A381")),
        ("/pages/1/raster/box_rect/0", serde_json::json!("0.0")),
        (
            "/pages/1/recognition/image/format",
            serde_json::json!("jpeg"),
        ),
        (
            "/pages/1/recognition/image/orientation",
            serde_json::json!(2),
        ),
        (
            "/pages/1/recognition/engine/os_build",
            serde_json::json!("22A381"),
        ),
        (
            "/pages/1/recognition/regions/0/confidence",
            serde_json::json!("0.50"),
        ),
    ] {
        let mut bad_doc = value.clone();
        *bad_doc.pointer_mut(pointer).unwrap() = bad;
        assert!(
            serde_json::from_value::<compositor::Document>(bad_doc)
                .unwrap()
                .validate()
                .is_err(),
            "{pointer}"
        );
    }
    let mut capped = doc;
    for page in &mut capped.pages {
        for r in &mut page.recognition.regions {
            r.text = "x".repeat(120000);
        }
    }
    let mut remaining = protocol::MAX_SOURCE - serde_json::to_vec(&capped).unwrap().len();
    for page in &mut capped.pages {
        for r in &mut page.recognition.regions {
            let add = (131072 - r.text.len()).min(remaining);
            r.text.push_str(&"x".repeat(add));
            remaining -= add;
        }
    }
    assert_eq!(remaining, 0);
    assert_eq!(
        serde_json::to_vec(&capped).unwrap().len(),
        protocol::MAX_SOURCE
    );
    capped.validate().unwrap();
    capped.pages[1].recognition.regions[3].text.push('x');
    assert_eq!(
        capped.validate().unwrap_err().details["reason"],
        "pdf_ocr_result_limit"
    );
}
#[test]
fn pdf_ocr_composition_never_returns_partial_pages_and_shares_guards() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    let b = Budget::new(&conn).unwrap();
    let mut render_count = 0;
    let mut ocr_count = 0;
    let doc = document(2, 1);
    let e = compositor::compose(
        b"source",
        &b,
        &mut || Ok(()),
        |page, _, _| {
            render_count += 1;
            if page == 2 {
                return Err(store::invalid("pdf_ocr_render_failed"));
            }
            Ok(protocol::Raster {
                identity: doc.pages[0].raster.clone(),
                png: b"synthetic PNG fixture; no inference".to_vec(),
            })
        },
        |_, _, _| {
            ocr_count += 1;
            Ok(doc.pages[0].recognition.clone())
        },
    )
    .unwrap_err();
    assert_eq!(e.details["reason"], "pdf_ocr_render_failed");
    assert_eq!((render_count, ocr_count), (2, 1));
    let b = Budget::with_duration(&conn, Duration::from_millis(20)).unwrap();
    let mut ocr_count = 0;
    let e = compositor::compose(
        b"source",
        &b,
        &mut || Ok(()),
        |_, budget, _| {
            std::thread::sleep(Duration::from_millis(25));
            budget.check()?;
            unreachable!()
        },
        |_, _, _| {
            ocr_count += 1;
            unreachable!()
        },
    )
    .unwrap_err();
    assert_eq!(e.details["reason"], "execution_deadline");
    assert_eq!(ocr_count, 0);
}
#[test]
fn pdf_ocr_raster_reply_rejects_wrong_source_geometry_and_png_framing() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use sha2::{Digest, Sha256};
    let png=STANDARD.decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAAAAAA6fptVAAAACklEQVR4nGP4DwABAQEAsTj2FAAAAABJRU5ErkJggg==").unwrap();
    let hash = format!("{:x}", Sha256::digest(b"source"));
    let value = serde_json::json!({"version":1,"profile":protocol::PROFILE,"outcome":{"status":"success","source_sha256":hash,
        "raster":{"page_count":1,"page_number":1,"total_pixels":1,"box":["0.0","-0.0","0.5","0.5"],"rotation":0,"width":1,"height":1,"png":STANDARD.encode(&png),"os_version":"13.0.0","os_build":"22A380"}}});
    let decoded = protocol::decode(&serde_json::to_vec(&value).unwrap(), &hash, 1).unwrap();
    let protocol::Decoded::Raster(r) = decoded else {
        panic!("raster")
    };
    assert_eq!(r.identity.box_rect, ["0", "-0", "0.5", "0.5"]);
    assert_eq!(r.png, png);
    for (pointer, bad) in [
        ("/version", serde_json::json!(2)),
        ("/profile", serde_json::json!("other")),
        ("/outcome/source_sha256", serde_json::json!("0".repeat(64))),
        ("/outcome/raster/page_number", serde_json::json!(2)),
        ("/outcome/raster/total_pixels", serde_json::json!(16000001)),
        ("/outcome/raster/width", serde_json::json!(2)),
        ("/outcome/raster/box/2", serde_json::json!("NaN")),
        ("/outcome/raster/rotation", serde_json::json!(45)),
        ("/outcome/raster/os_build", serde_json::json!("bad build")),
        ("/outcome/raster/png", serde_json::json!("?")),
    ] {
        let mut v = value.clone();
        *v.pointer_mut(pointer).unwrap() = bad;
        assert!(
            protocol::decode(&serde_json::to_vec(&v).unwrap(), &hash, 1).is_err(),
            "{pointer}"
        );
    }
    for bytes in [
        png[..png.len() - 1].to_vec(),
        [png.as_slice(), b"X"].concat(),
        {
            let mut p = png.clone();
            p[45] ^= 1;
            p
        },
    ] {
        let mut v = value.clone();
        v["outcome"]["raster"]["png"] = serde_json::json!(STANDARD.encode(bytes));
        assert!(protocol::decode(&serde_json::to_vec(&v).unwrap(), &hash, 1).is_err());
    }
    let mut unknown = value;
    unknown["outcome"]["raster"]["future"] = serde_json::json!(true);
    assert!(protocol::decode(&serde_json::to_vec(&unknown).unwrap(), &hash, 1).is_err());
}
