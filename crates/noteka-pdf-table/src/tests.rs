use super::*;
use serde_json::json;
fn region(n: u32, x: &str, y: &str, w: &str, h: &str, text: &str) -> serde_json::Value {
    json!({"ordinal":n,"text":text,"confidence":"0","bbox":{"x":x,"y":y,"width":w,"height":h}})
}
fn fixture(regions: Vec<serde_json::Value>) -> Input {
    serde_json::from_value(json!({"source_sha256":"a".repeat(64),"page_number":1,"x_edges":["0","0.5","1"],"y_min":"0.2","y_max":"1","header_row":1,"recognition":{
        "version":1,"image":{"format":"png","width":1000,"height":1000,"orientation":1,"oriented_width":1000,"oriented_height":1000},
        "engine":{"revision":3,"recognition_level":"fast","languages":["es-ES","en-US"],"language_correction":false,"automatic_language_detection":false,"os_version":"13.0.0","os_build":"Synthetic"},
        "coordinate_space":noteka_ocr_protocol::COORDINATE_SPACE,"limitations":noteka_ocr_protocol::LIMITATIONS,"regions":regions
    }})).unwrap()
}
#[test]
fn independent_golden_keeps_every_ordinal_values_and_empty_cells() {
    // Orden OCR no es orden de filas ni columnas. Cero confianza no se filtra.
    let input = fixture(vec![
        region(0, "0.6", "0.5", "0.2", "0.1", "145.6900"),
        region(1, "0.1", "0.8", "0.2", "0.1", "<script>dato</script>"),
        region(2, "0.6", "0.8", "0.2", "0.1", "<script>dato</script>"),
        region(3, "0.1", "0.05", "0.2", "0.05", "fuera"),
        region(4, "0.1", "0.3", "0.2", "0.1", ""),
    ]);
    let mut input = input;
    input.recognition.regions[4].text = "sin par".into();
    let original = input.clone();
    let layout = project(input).unwrap();
    assert_eq!(layout.evidence, original);
    assert_eq!(
        layout.rows,
        vec![
            Row {
                number: 1,
                cells: vec![
                    Cell {
                        column: 1,
                        region_ordinals: vec![1]
                    },
                    Cell {
                        column: 2,
                        region_ordinals: vec![2]
                    }
                ]
            },
            Row {
                number: 2,
                cells: vec![
                    Cell {
                        column: 1,
                        region_ordinals: vec![]
                    },
                    Cell {
                        column: 2,
                        region_ordinals: vec![0]
                    }
                ]
            },
            Row {
                number: 3,
                cells: vec![
                    Cell {
                        column: 1,
                        region_ordinals: vec![4]
                    },
                    Cell {
                        column: 2,
                        region_ordinals: vec![]
                    }
                ]
            },
        ]
    );
    assert_eq!(layout.outside, vec![3]);
    let json = serde_json::to_vec(&layout).unwrap();
    assert_eq!(serde_json::from_slice::<Layout>(&json).unwrap(), layout);
    let mut refs = layout.outside.clone();
    refs.extend(layout.rows.iter().flat_map(|r| {
        r.cells
            .iter()
            .flat_map(|c| c.region_ordinals.iter().copied())
    }));
    refs.sort_unstable();
    assert_eq!(refs, (0..5).collect::<Vec<_>>());
}
#[test]
fn crossings_and_ambiguous_rows_fail_whole_without_splitting() {
    for (x, y, w, h) in [("0.4", "0.5", "0.2", "0.1"), ("0.1", "0.1", "0.2", "0.2")] {
        assert_eq!(
            project(fixture(vec![region(0, x, y, w, h, "unsplit")])).unwrap_err(),
            TableError::CrossingRegion
        );
    }
    // Cadena vertical: A∩bridge y bridge∩B, pero no A∩bridge∩B.
    let input = fixture(vec![
        region(0, "0.1", "0.8", "0.1", "0.1", "a"),
        region(1, "0.6", "0.6", "0.1", "0.3", "bridge"),
        region(2, "0.1", "0.6", "0.1", "0.1", "b"),
    ]);
    assert_eq!(project(input).unwrap_err(), TableError::AmbiguousRows);
}
#[test]
fn input_and_evidence_are_strict_canonical_and_not_authority() {
    let clean = fixture(vec![region(0, "0.1", "0.5", "0.1", "0.1", "text")]);
    for edges in [
        vec!["0", "0.5", "0.5", "1"],
        vec!["0", "1", "0.5"],
        vec!["0", "NaN"],
        vec!["0", "1.0"],
        vec!["0", "1.1"],
        vec!["0", " 1"],
    ] {
        let mut input = clean.clone();
        input.x_edges = edges.iter().map(|s| s.to_string()).collect();
        assert_eq!(project(input).unwrap_err(), TableError::InvalidInput);
    }
    for header in [0, 2] {
        let mut i = clean.clone();
        i.header_row = Some(header);
        assert_eq!(project(i).unwrap_err(), TableError::InvalidInput);
    }
    let mut i = clean.clone();
    i.recognition.regions[0].ordinal = 1;
    assert_eq!(project(i).unwrap_err(), TableError::InvalidEvidence);
    let mut i = clean.clone();
    i.recognition.regions[0].bbox.x = "0.10".into();
    assert_eq!(project(i).unwrap_err(), TableError::InvalidEvidence);
    let mut i = clean.clone();
    i.source_sha256 = "A".repeat(64);
    assert_eq!(project(i).unwrap_err(), TableError::InvalidInput);
    let mut j = serde_json::to_value(&clean).unwrap();
    j["workspace"] = json!("/private");
    assert!(serde_json::from_value::<Input>(j).is_err());
    let mut i = clean;
    i.header_row = None;
    let p = project(i).unwrap();
    assert_eq!(p.evidence.header_row, None);
    assert!(p.limitations.iter().any(|s| s.contains("not_file")));
}
#[test]
fn empty_and_edge_touching_regions_are_not_discarded() {
    let mut input = fixture(vec![]);
    input.header_row = None;
    let l = project(input).unwrap();
    assert!(l.rows.is_empty() && l.outside.is_empty());
    let input = fixture(vec![
        region(0, "0.5", "0.5", "0.1", "0.1", "right"),
        region(1, "0.1", "0.1", "0.1", "0.1", "touching bottom"),
    ]);
    let l = project(input).unwrap();
    assert_eq!(l.rows[0].cells[1].region_ordinals, vec![0]);
    assert_eq!(l.outside, vec![1]);
}
fn full(rows: u32, columns: u32) -> Input {
    let mut regions = Vec::new();
    for r in 0..rows {
        for c in 0..columns {
            let y = 0.99 - f64::from(r) * 0.014;
            let x = (f64::from(c) + 0.1) / f64::from(columns);
            regions.push(region(
                r * columns + c,
                &x.to_string(),
                &y.to_string(),
                &(0.5 / f64::from(columns)).to_string(),
                "0.005",
                "cell",
            ));
        }
    }
    let mut i = fixture(regions);
    i.y_min = "0".into();
    i.x_edges = (0..=columns)
        .map(|c| (f64::from(c) / f64::from(columns)).to_string())
        .collect();
    i
}
#[test]
fn exact_row_column_cell_and_region_limits_do_not_expand() {
    let i = full(64, 8);
    assert_eq!(i.recognition.regions.len(), 512);
    let l = project(i).unwrap();
    assert_eq!(l.rows.len(), 64);
    assert_eq!(l.rows.iter().map(|r| r.cells.len()).sum::<usize>(), 512);
    assert_eq!(project(full(16, 32)).unwrap().rows.len(), 16);
    assert_eq!(
        project(full(65, 1)).unwrap_err(),
        TableError::StructureLimit
    );
    assert_eq!(project(full(16, 33)).unwrap_err(), TableError::InvalidInput);
    let mut i = full(64, 8);
    i.x_edges.insert(1, "0.01".into());
    assert_eq!(project(i).unwrap_err(), TableError::StructureLimit);
    let mut i = full(64, 8);
    let mut r = i.recognition.regions[0].clone();
    r.ordinal = 512;
    i.recognition.regions.push(r);
    assert_eq!(project(i).unwrap_err(), TableError::InvalidEvidence);
}
#[test]
fn degenerate_effective_geometry_is_rejected() {
    let mut input = fixture(vec![region(0, "0.5", "0.5", "0.1", "0.1", "tiny")]);
    input.recognition.regions[0].bbox.width =
        "0.00000000000000000000000000000000000000000000000001".into();
    input.recognition.validate().unwrap();
    assert_eq!(project(input).unwrap_err(), TableError::InvalidEvidence);
}
