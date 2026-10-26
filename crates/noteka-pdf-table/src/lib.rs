//! Proyección independiente: el caller debe verificar el PDF/generación vigentes.
//! No I/O, detección de columnas, división de texto ni descarte de evidencia.
use noteka_ocr_protocol::v2::{OcrImageFormat, OcrRecognition};
use serde::{Deserialize, Serialize};

pub const PROFILE: &str = "pdf_ocr_columns_rows_v1";
pub const MAX_WIRE: usize = 2_097_152;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub source_sha256: String,
    pub page_number: u32,
    pub recognition: OcrRecognition,
    pub x_edges: Vec<String>,
    pub y_min: String,
    pub y_max: String,
    pub header_row: Option<u32>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TableError {
    InvalidInput,
    InvalidEvidence,
    CrossingRegion,
    AmbiguousRows,
    StructureLimit,
    ResponseLimit,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub column: u32,
    pub region_ordinals: Vec<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub number: u32,
    pub cells: Vec<Cell>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub profile: String,
    pub evidence: Input,
    pub rows: Vec<Row>,
    pub outside: Vec<u32>,
    pub limitations: Vec<String>,
}
fn unit(s: &str) -> Result<f64, TableError> {
    if s.is_empty() || s.len() > 512 {
        return Err(TableError::InvalidInput);
    }
    let f = s.parse::<f64>().map_err(|_| TableError::InvalidInput)?;
    if !f.is_finite() || !(0.0..=1.0).contains(&f) || f.to_string() != s {
        return Err(TableError::InvalidInput);
    }
    Ok(f)
}
struct Placed {
    ordinal: u32,
    column: usize,
    x: f64,
    low: f64,
    high: f64,
}
struct Band {
    low: f64,
    high: f64,
    envelope_low: f64,
    envelope_high: f64,
    regions: Vec<Placed>,
}
fn overlaps(a: f64, b: f64, c: f64, d: f64) -> bool {
    b.min(d) > a.max(c)
}

pub fn project(input: Input) -> Result<Layout, TableError> {
    if input.source_sha256.len() != 64
        || !input
            .source_sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || !(1..=20).contains(&input.page_number)
        || !(2..=33).contains(&input.x_edges.len())
        || input.header_row == Some(0)
    {
        return Err(TableError::InvalidInput);
    }
    if input.recognition.regions.len() > 512
        || input
            .recognition
            .regions
            .iter()
            .map(|r| r.text.len())
            .sum::<usize>()
            > 1_048_576
    {
        return Err(TableError::InvalidEvidence);
    }
    input
        .recognition
        .validate()
        .map_err(|_| TableError::InvalidEvidence)?;
    if input.recognition.image.format != OcrImageFormat::Png
        || input.recognition.image.orientation != 1
    {
        return Err(TableError::InvalidEvidence);
    }
    let edges: Vec<_> = input
        .x_edges
        .iter()
        .map(|s| unit(s))
        .collect::<Result<_, _>>()?;
    if edges.windows(2).any(|e| e[0] >= e[1]) {
        return Err(TableError::InvalidInput);
    }
    let low = unit(&input.y_min)?;
    let high = unit(&input.y_max)?;
    if low >= high {
        return Err(TableError::InvalidInput);
    }
    let mut inside = Vec::new();
    let mut outside = Vec::new();
    for r in &input.recognition.regions {
        let x = unit(&r.bbox.x)?;
        let y = unit(&r.bbox.y)?;
        let right = x + unit(&r.bbox.width)?;
        let top = y + unit(&r.bbox.height)?;
        if right <= x || top <= y {
            return Err(TableError::InvalidEvidence);
        }
        if right <= edges[0] || x >= edges[edges.len() - 1] || top <= low || y >= high {
            outside.push(r.ordinal);
            continue;
        }
        if x < edges[0] || right > edges[edges.len() - 1] || y < low || top > high {
            return Err(TableError::CrossingRegion);
        }
        let column = edges
            .windows(2)
            .position(|e| x >= e[0] && right <= e[1])
            .ok_or(TableError::CrossingRegion)?;
        inside.push(Placed {
            ordinal: r.ordinal,
            column,
            x,
            low: y,
            high: top,
        });
    }
    inside.sort_by(|a, b| {
        ((b.low + b.high) / 2.0)
            .total_cmp(&((a.low + a.high) / 2.0))
            .then_with(|| a.x.total_cmp(&b.x))
            .then(a.ordinal.cmp(&b.ordinal))
    });
    let mut bands: Vec<Band> = Vec::new();
    for r in inside {
        let mut matching = None;
        for (i, b) in bands.iter().enumerate() {
            let common = overlaps(r.low, r.high, b.low, b.high);
            if (overlaps(r.low, r.high, b.envelope_low, b.envelope_high) && !common)
                || (common && matching.is_some())
            {
                return Err(TableError::AmbiguousRows);
            }
            if common {
                matching = Some(i);
            }
        }
        if let Some(i) = matching {
            let b = &mut bands[i];
            b.low = b.low.max(r.low);
            b.high = b.high.min(r.high);
            b.envelope_low = b.envelope_low.min(r.low);
            b.envelope_high = b.envelope_high.max(r.high);
            b.regions.push(r);
        } else {
            if bands.len() == 64 {
                return Err(TableError::StructureLimit);
            }
            bands.push(Band {
                low: r.low,
                high: r.high,
                envelope_low: r.low,
                envelope_high: r.high,
                regions: vec![r],
            });
        }
    }
    let columns = edges.len() - 1;
    if bands.len() * columns > 512 {
        return Err(TableError::StructureLimit);
    }
    if input.header_row.is_some_and(|n| n as usize > bands.len()) {
        return Err(TableError::InvalidInput);
    }
    bands.sort_by(|a, b| b.high.total_cmp(&a.high));
    let rows = bands
        .into_iter()
        .enumerate()
        .map(|(n, b)| {
            let mut cells: Vec<_> = (0..columns)
                .map(|c| Cell {
                    column: c as u32 + 1,
                    region_ordinals: Vec::new(),
                })
                .collect();
            for r in b.regions {
                cells[r.column].region_ordinals.push(r.ordinal);
            }
            for c in &mut cells {
                c.region_ordinals.sort_unstable();
            }
            Row {
                number: n as u32 + 1,
                cells,
            }
        })
        .collect();
    let layout = Layout {
        profile: PROFILE.into(),
        evidence: input,
        rows,
        outside,
        limitations: vec![
            "inferred_rows_explicit_columns_not_generic_pdf_tables".into(),
            "ocr_text_may_be_incorrect_or_incomplete".into(),
            "rendered_page_normalized_bottom_left_not_pdf_coordinates".into(),
            "no_spans_merges_text_splitting_or_header_inference".into(),
            "declared_hash_not_file_or_workspace_verification".into(),
        ],
    };
    if serde_json::to_vec(&layout)
        .map_err(|_| TableError::ResponseLimit)?
        .len()
        > MAX_WIRE
    {
        return Err(TableError::ResponseLimit);
    }
    Ok(layout)
}

#[cfg(test)]
mod tests;
