//! Celdas completas y localizadores del workbook; chunks internos no son offsets fuente.
use crate::{contracts::ExtractionGeneration, error::*, extraction_storage as store};
use noteka_xlsx_protocol::{Cell, NumberStyle, Visibility};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadXlsxCells {
    pub id: String,
    #[schemars(range(min = 1, max = 64))]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
impl ReadXlsxCells {
    pub fn validate(&self) -> Result<u32> {
        store::uuid(&self.id)?;
        let limit = store::page_limit(self.limit, 32, 64)?;
        if let Some(cursor) = &self.cursor {
            store::Cursor::read(cursor)?;
        }
        Ok(limit)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct XlsxSheetDescriptor {
    pub id: u32,
    pub name: String,
    pub visibility: Visibility,
    pub declared_range: Option<String>,
    pub observed_range: Option<String>,
    pub merges: Vec<String>,
    #[schemars(range(max = 10_000))]
    pub cell_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct XlsxManifest {
    pub version: u32,
    pub parser: String,
    pub representation_sha256: String,
    #[schemars(range(min = 1, max = 1_048_576))]
    pub representation_bytes: u32,
    pub date_1904: bool,
    pub limitations: Vec<String>,
    pub styles: Vec<NumberStyle>,
    #[schemars(length(min = 1, max = 32))]
    pub sheets: Vec<XlsxSheetDescriptor>,
    #[schemars(range(max = 10_000))]
    pub cell_count: u32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum XlsxLocatorKind {
    XlsxCell,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct XlsxCellLocator {
    pub kind: XlsxLocatorKind,
    pub sheet_id: u32,
    pub reference: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct XlsxCellItem {
    #[schemars(range(max = 9_999))]
    pub ordinal: u32,
    pub locator: XlsxCellLocator,
    pub cell: Cell,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct XlsxCellPage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: XlsxManifest,
    pub items: Vec<XlsxCellItem>,
    pub next_cursor: Option<String>,
    pub covers_generation: bool,
}
