//! Protocolo privado de bytes y DTOs independientes del parser y del workspace.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub const VERSION: u16 = 1;
pub const PARSER: &str = "zip-9.0.3/roxmltree-0.21.1";
pub const HEADER_BYTES: usize = 24;
pub const MAX_SOURCE: usize = 1_048_576;
pub const MAX_ENTRY: usize = 4 * 1_048_576;
pub const MAX_EXPANDED: usize = 16 * 1_048_576;
pub const MAX_ENTRIES: usize = 256;
pub const MAX_NODES: u32 = 100_000;
pub const MAX_DEPTH: usize = 64;
pub const MAX_SHEETS: usize = 32;
pub const MAX_CELLS: usize = 10_000;
pub const MAX_STYLES: usize = 512;
pub const MAX_STRINGS: usize = 10_000;
pub const MAX_MERGES: usize = 1_000;
pub const MAX_TEXT: usize = 131_072;
pub const MAX_RESULT: usize = 1_048_576;
pub const MAX_STDOUT: usize = 8 * 1_048_576;
pub const MAX_STDERR: usize = 8 * 1024;
pub const MAX_MS: u32 = 5_000;
pub const LIMITATIONS: [&str; 6] = [
    "cached_values_not_recalculated",
    "shared_formulas_not_expanded",
    "no_visual_formatting_or_date_conversion",
    "no_header_or_table_inference",
    "no_images_charts_comments_or_shapes_transcription",
    "bounded_transitional_xlsx_only",
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
        out[..8].copy_from_slice(b"NKXLS001");
        out[8..10].copy_from_slice(&VERSION.to_le_bytes());
        out[12..16].copy_from_slice(&self.source_bytes.to_le_bytes());
        out[16..20].copy_from_slice(&self.remaining_ms.to_le_bytes());
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != HEADER_BYTES
            || &bytes[..8] != b"NKXLS001"
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerError {
    XlsxWorkerProtocol,
    XlsxInvalidZip,
    XlsxUnsupportedZip,
    XlsxExpansionLimit,
    XlsxInvalidXml,
    XlsxXmlLimit,
    XlsxUnsupportedFormat,
    XlsxActiveContent,
    XlsxExternalRelationship,
    XlsxInvalidWorkbook,
    XlsxInvalidCell,
    XlsxUnsupportedCell,
    XlsxInvalidFormula,
    XlsxUnsupportedFormula,
    XlsxStructureLimit,
    XlsxResultLimit,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Success {
        source_sha256: String,
        workbook: Workbook,
    },
    Error {
        reason: WorkerError,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub version: u16,
    pub parser: String,
    pub outcome: Outcome,
}
impl Reply {
    pub fn new(outcome: Outcome) -> Self {
        Self {
            version: VERSION,
            parser: PARSER.into(),
            outcome,
        }
    }
    pub fn decode(bytes: &[u8], expected_hash: &str) -> Result<Self> {
        if bytes.len() > MAX_STDOUT || !sha256(expected_hash) {
            return Err(Invalid);
        }
        let reply: Self = serde_json::from_slice(bytes).map_err(|_| Invalid)?;
        if reply.version != VERSION || reply.parser != PARSER {
            return Err(Invalid);
        }
        if let Outcome::Success {
            source_sha256,
            workbook,
        } = &reply.outcome
        {
            if source_sha256 != expected_hash {
                return Err(Invalid);
            }
            workbook.validate()?;
        }
        Ok(reply)
    }
}
fn sha256(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Workbook {
    pub date_1904: bool,
    pub limitations: Vec<String>,
    pub styles: Vec<NumberStyle>,
    pub sheets: Vec<Sheet>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NumberStyle {
    pub num_fmt_id: u32,
    pub format_code: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Visible,
    Hidden,
    VeryHidden,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Sheet {
    pub id: u32,
    pub name: String,
    pub visibility: Visibility,
    pub declared_range: Option<String>,
    pub observed_range: Option<String>,
    pub merges: Vec<String>,
    pub cells: Vec<Cell>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub reference: String,
    pub row: u32,
    pub column: u32,
    pub style: u32,
    pub cached: Option<CellValue>,
    pub formula: Option<Formula>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CellValue {
    Number(String),
    Text(String),
    Boolean(bool),
    Error(String),
    DateIso(String),
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FormulaKind {
    Normal,
    Shared,
    Array,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Formula {
    pub kind: FormulaKind,
    pub text: String,
    pub reference: Option<String>,
    pub shared_index: Option<u32>,
}

/// No f64, trim, locale, round-trip ni límite artificial de precisión decimal.
pub fn number(s: &str) -> bool {
    let b = s.as_bytes();
    let mut p = usize::from(b.first().is_some_and(|b| matches!(b, b'+' | b'-')));
    let mut digits = 0;
    while b.get(p).is_some_and(u8::is_ascii_digit) {
        p += 1;
        digits += 1;
    }
    if b.get(p) == Some(&b'.') {
        p += 1;
        while b.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return false;
    }
    if b.get(p).is_some_and(|b| matches!(b, b'e' | b'E')) {
        p += 1;
        if b.get(p).is_some_and(|b| matches!(b, b'+' | b'-')) {
            p += 1;
        }
        let start = p;
        while b.get(p).is_some_and(u8::is_ascii_digit) {
            p += 1;
        }
        if p == start {
            return false;
        }
    }
    p == b.len()
}
pub fn coordinate(s: &str) -> Result<(u32, u32)> {
    let n = s.bytes().take_while(u8::is_ascii_uppercase).count();
    if !(1..=3).contains(&n) || n == s.len() || s.as_bytes()[n] == b'0' {
        return Err(Invalid);
    }
    let mut col = 0u32;
    for b in s.bytes().take(n) {
        col = col * 26 + u32::from(b - b'A' + 1);
    }
    let row_str = &s[n..];
    if !row_str.bytes().all(|b| b.is_ascii_digit()) {
        return Err(Invalid);
    }
    let row = row_str.parse::<u32>().map_err(|_| Invalid)?;
    if !(1..=1_048_576).contains(&row) || !(1..=16_384).contains(&col) {
        return Err(Invalid);
    }
    Ok((row, col))
}
pub fn a1(row: u32, mut col: u32) -> String {
    let mut letters = Vec::new();
    while col > 0 {
        col -= 1;
        letters.push(b'A' + (col % 26) as u8);
        col /= 26;
    }
    letters.reverse();
    format!("{}{row}", String::from_utf8(letters).expect("ASCII"))
}
pub fn range(s: &str) -> Result<((u32, u32), (u32, u32))> {
    let (a, b) = s.split_once(':').unwrap_or((s, s));
    let (a, b) = (coordinate(a)?, coordinate(b)?);
    if a.0 > b.0 || a.1 > b.1 {
        return Err(Invalid);
    }
    Ok((a, b))
}
pub fn observed(cells: &[Cell]) -> Option<String> {
    let first = cells.first()?;
    let mut lo = (first.row, first.column);
    let mut hi = lo;
    for c in cells {
        lo = (lo.0.min(c.row), lo.1.min(c.column));
        hi = (hi.0.max(c.row), hi.1.max(c.column));
    }
    let a = a1(lo.0, lo.1);
    Some(if lo == hi {
        a
    } else {
        format!("{a}:{}", a1(hi.0, hi.1))
    })
}
fn text(s: &str) -> bool {
    s.len() <= MAX_TEXT && !s.contains('\0')
}
fn inside(pos: (u32, u32), r: ((u32, u32), (u32, u32))) -> bool {
    pos.0 >= r.0.0 && pos.0 <= r.1.0 && pos.1 >= r.0.1 && pos.1 <= r.1.1
}
/// Cuenta JSON escapado sin asignarlo; útil antes de acumular strings repetidos.
pub fn json_size(value: &impl Serialize, max: usize) -> Result<usize> {
    struct Count {
        n: usize,
        max: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.max)
                .ok_or_else(|| std::io::Error::other("size_limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Count { n: 0, max };
    serde_json::to_writer(&mut counter, value).map_err(|_| Invalid)?;
    Ok(counter.n)
}
impl Workbook {
    pub fn validate(&self) -> Result<()> {
        if self.limitations != LIMITATIONS
            || self.sheets.is_empty()
            || self.sheets.len() > MAX_SHEETS
            || self.styles.is_empty()
            || self.styles.len() > MAX_STYLES
        {
            return Err(Invalid);
        }
        let mut formats = HashMap::new();
        for style in &self.styles {
            if style
                .format_code
                .as_ref()
                .is_some_and(|s| !text(s) || s.is_empty())
                || (style.num_fmt_id >= 164 && style.format_code.is_none())
            {
                return Err(Invalid);
            }
            if let Some(old) = formats.insert(style.num_fmt_id, &style.format_code)
                && old != &style.format_code
            {
                return Err(Invalid);
            }
        }
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        let mut count = 0usize;
        for sheet in &self.sheets {
            count += sheet.cells.len();
            if count > MAX_CELLS
                || sheet.id == 0
                || !ids.insert(sheet.id)
                || sheet.name.is_empty()
                || sheet.name.chars().count() > 31
                || sheet
                    .name
                    .chars()
                    .any(|c| c.is_control() || "[]:*?/\\".contains(c))
                || !names.insert(sheet.name.to_lowercase())
                || sheet.merges.len() > MAX_MERGES
                || sheet.observed_range != observed(&sheet.cells)
            {
                return Err(Invalid);
            }
            if let Some(s) = &sheet.declared_range {
                range(s)?;
            }
            let mut merges = HashSet::new();
            for r in &sheet.merges {
                range(r)?;
                if !merges.insert(r) {
                    return Err(Invalid);
                }
            }
            let mut previous = (0, 0);
            for cell in &sheet.cells {
                let pos = coordinate(&cell.reference)?;
                if pos != (cell.row, cell.column)
                    || pos <= previous
                    || cell.style as usize >= self.styles.len()
                {
                    return Err(Invalid);
                }
                previous = pos;
                if let Some(value) = &cell.cached {
                    let valid = match value {
                        CellValue::Number(s) => text(s) && number(s),
                        CellValue::Text(s) => text(s),
                        CellValue::Error(s) | CellValue::DateIso(s) => text(s) && !s.is_empty(),
                        CellValue::Boolean(_) => true,
                    };
                    if !valid {
                        return Err(Invalid);
                    }
                }
            }
            sheet.validate_formulas()?;
        }
        json_size(self, MAX_RESULT)?;
        Ok(())
    }
}
impl Sheet {
    pub fn validate_formulas(&self) -> Result<()> {
        let mut anchors = HashMap::new();
        for cell in &self.cells {
            let pos = (cell.row, cell.column);
            if let Some(f) = &cell.formula {
                if !text(&f.text) {
                    return Err(Invalid);
                }
                match f.kind {
                    FormulaKind::Normal
                        if f.text.is_empty()
                            || f.shared_index.is_some()
                            || f.reference.is_some() =>
                    {
                        return Err(Invalid);
                    }
                    FormulaKind::Array => {
                        let r = range(f.reference.as_deref().ok_or(Invalid)?)?;
                        if f.text.is_empty() || f.shared_index.is_some() || pos != r.0 {
                            return Err(Invalid);
                        }
                    }
                    FormulaKind::Shared => {
                        let index = f.shared_index.ok_or(Invalid)?;
                        if !f.text.is_empty() {
                            let r = range(f.reference.as_deref().ok_or(Invalid)?)?;
                            if r.0 != pos || anchors.insert(index, r).is_some() {
                                return Err(Invalid);
                            }
                        } else if f.reference.is_some() {
                            return Err(Invalid);
                        }
                    }
                    _ => {}
                }
            }
        }
        for cell in &self.cells {
            if let Some(f) = &cell.formula
                && f.kind == FormulaKind::Shared
            {
                let r = *anchors
                    .get(&f.shared_index.ok_or(Invalid)?)
                    .ok_or(Invalid)?;
                if !inside((cell.row, cell.column), r) {
                    return Err(Invalid);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Workbook {
        Workbook {
            date_1904: false,
            limitations: LIMITATIONS.iter().map(|s| (*s).into()).collect(),
            styles: vec![NumberStyle {
                num_fmt_id: 0,
                format_code: None,
            }],
            sheets: vec![Sheet {
                id: 1,
                name: "Sheet1".into(),
                visibility: Visibility::Visible,
                declared_range: Some("A1:XFD1048576".into()),
                observed_range: Some("A1:B1".into()),
                merges: vec!["A1:B1".into()],
                cells: vec![
                    Cell {
                        reference: "A1".into(),
                        row: 1,
                        column: 1,
                        style: 0,
                        cached: Some(CellValue::Number("9007199254740993.00".into())),
                        formula: Some(Formula {
                            kind: FormulaKind::Shared,
                            text: "B2+1".into(),
                            reference: Some("A1:B1".into()),
                            shared_index: Some(3),
                        }),
                    },
                    Cell {
                        reference: "B1".into(),
                        row: 1,
                        column: 2,
                        style: 0,
                        cached: None,
                        formula: Some(Formula {
                            kind: FormulaKind::Shared,
                            text: String::new(),
                            reference: None,
                            shared_index: Some(3),
                        }),
                    },
                ],
            }],
        }
    }
    #[test]
    fn independent_validator_catches_every_locator_and_shared_formula_invariant() {
        let good = sample();
        assert!(good.validate().is_ok());
        let mut bads = Vec::new();
        let mut b = good.clone();
        b.sheets[0].cells[1].formula.as_mut().unwrap().shared_index = Some(4);
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].cells[0].formula.as_mut().unwrap().reference = Some("A1".into());
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].cells[0].formula.as_mut().unwrap().text = String::new();
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].cells.swap(0, 1);
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].observed_range = Some("A1".into());
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].cells[0].row = 2;
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].cells[0].style = 1;
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].cells[0].cached = Some(CellValue::Number("NaN".into()));
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].cells[0].cached = Some(CellValue::Text("a\0b".into()));
        bads.push(b);
        let mut b = good.clone();
        b.sheets.push(b.sheets[0].clone());
        bads.push(b);
        let mut b = good.clone();
        b.styles[0].num_fmt_id = 164;
        bads.push(b);
        let mut b = good.clone();
        b.sheets[0].merges.push("B1:A1".into());
        bads.push(b);
        for (i, bad) in bads.iter().enumerate() {
            assert!(bad.validate().is_err(), "case {i}");
        }
        let reply = Reply::new(Outcome::Success {
            source_sha256: "a".repeat(64),
            workbook: good,
        });
        let bytes = serde_json::to_vec(&reply).unwrap();
        assert_eq!(Reply::decode(&bytes, &"a".repeat(64)), Ok(reply));
        assert!(Reply::decode(&bytes, &"b".repeat(64)).is_err());
    }
    #[test]
    fn normal_array_and_absent_cache_are_distinct() {
        let mut b = sample();
        b.sheets[0].cells[0].formula = Some(Formula {
            kind: FormulaKind::Normal,
            text: "B1".into(),
            reference: None,
            shared_index: None,
        });
        b.sheets[0].cells[1].formula = None;
        assert!(b.validate().is_ok());
        b.sheets[0].cells[0].formula.as_mut().unwrap().kind = FormulaKind::Array;
        assert!(b.validate().is_err());
        b.sheets[0].cells[0].formula.as_mut().unwrap().reference = Some("A1:B1".into());
        assert!(b.validate().is_ok());
        b.sheets[0].cells[0].formula.as_mut().unwrap().reference = Some("B1:B2".into());
        assert!(b.validate().is_err());
    }
    #[test]
    fn header_roundtrip_and_every_reserved_byte() {
        let h = Header {
            source_bytes: MAX_SOURCE as u32,
            remaining_ms: MAX_MS,
        };
        let bytes = h.encode().unwrap();
        assert_eq!(Header::decode(&bytes), Ok(h));
        for i in [0, 8, 10, 11, 20, 21, 22, 23] {
            let mut bad = bytes;
            bad[i] ^= 1;
            assert!(Header::decode(&bad).is_err());
        }
        for n in [0, MAX_SOURCE as u32 + 1] {
            assert!(
                Header {
                    source_bytes: n,
                    ..h
                }
                .encode()
                .is_err()
            );
        }
        for n in [0, MAX_MS + 1] {
            assert!(
                Header {
                    remaining_ms: n,
                    ..h
                }
                .encode()
                .is_err()
            );
        }
        assert!(Header::decode(&bytes[..23]).is_err());
    }
    #[test]
    fn coordinates_and_exact_numeric_grammar() {
        for s in ["A1", "Z9", "AA10", "XFD1048576"] {
            let (r, c) = coordinate(s).unwrap();
            assert_eq!(a1(r, c), s);
        }
        for s in [
            "A0",
            "A01",
            "a1",
            "$A$1",
            "XFE1",
            "A1048577",
            "A1:B2",
            "🦀1",
            "ABC999999999999",
        ] {
            assert!(coordinate(s).is_err(), "{s}");
        }
        for s in [
            "0",
            "-0",
            "+01.2300E-9999",
            ".5",
            "1.",
            "9007199254740993",
            "145.6900000000000000000001",
        ] {
            assert!(number(s), "{s}");
        }
        for s in [
            "", "+", ".", "1e", "NaN", "INF", "1,23", " 2", "2 ", "1e+-2", "1e2e3",
        ] {
            assert!(!number(s), "{s}");
        }
    }
    #[test]
    fn json_budget_includes_escaping_and_accepts_frontier() {
        assert_eq!(json_size(&"\n", 4), Ok(4));
        assert!(json_size(&"\n", 3).is_err());
    }
    #[test]
    fn reply_rejects_malformed_hash_unknown_fields_and_versions() {
        let r = Reply::new(Outcome::Error {
            reason: WorkerError::XlsxInvalidZip,
        });
        let bytes = serde_json::to_vec(&r).unwrap();
        let hash = "a".repeat(64);
        assert_eq!(Reply::decode(&bytes, &hash), Ok(r));
        assert!(Reply::decode(&bytes, "BAD").is_err());
        for bad in [
            br#"{"version":2,"parser":"x","outcome":{}}"#.as_slice(),
            br#"{"version":1,"parser":"x","outcome":{},"extra":true}"#,
        ] {
            assert!(Reply::decode(bad, &hash).is_err());
        }
    }
}
