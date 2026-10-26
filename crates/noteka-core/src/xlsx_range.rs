//! Proyección dispersa de una región elegida, sin otra representación persistida.
use crate::{contracts::*, error::*, extraction_storage as store, workspace::Workspace};
use noteka_xlsx_protocol::{a1, range};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadXlsxRange {
    pub id: String,
    #[schemars(range(min = 1))]
    pub sheet_id: u32,
    #[schemars(length(min = 2, max = 21))]
    pub range: String,
    #[schemars(range(min = 1, max = 1_048_576))]
    pub header_row: Option<u32>,
    #[schemars(range(min = 1, max = 64))]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
impl ReadXlsxRange {
    pub fn validate(&self) -> Result<u32> {
        store::uuid(&self.id)?;
        let ((first, left), (last, right)) = bounds(&self.range)?;
        if self.sheet_id == 0
            || right - left >= 64
            || self.header_row.is_some_and(|row| row < first || row > last)
        {
            return Err(store::invalid("invalid_xlsx_range"));
        }
        if let Some(raw) = &self.cursor {
            RangeCursor::read(raw)?;
        }
        store::page_limit(self.limit, 32, 64)
    }
}
fn bounds(raw: &str) -> Result<((u32, u32), (u32, u32))> {
    if raw.len() > 21 {
        return Err(store::invalid("invalid_xlsx_range"));
    }
    range(raw).map_err(|_| store::invalid("invalid_xlsx_range"))
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct XlsxRangeHeader {
    pub column: u32,
    pub reference: String,
    pub item: Option<XlsxCellItem>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct XlsxRangePage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: XlsxManifest,
    pub sheet: XlsxSheetDescriptor,
    pub range: String,
    pub header_row: Option<u32>,
    pub headers: Vec<XlsxRangeHeader>,
    pub matching_cell_count: u32,
    pub items: Vec<XlsxCellItem>,
    pub next_cursor: Option<String>,
    pub covers_range: bool,
    pub covers_generation: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RangeCursor {
    base: store::Cursor,
    sheet_id: u32,
    range: String,
    header_row: Option<u32>,
}
impl RangeCursor {
    fn read(raw: &str) -> Result<Self> {
        if raw.len() > 2048 {
            return Err(store::invalid("cursor_size_limit"));
        }
        let cursor: Self =
            serde_json::from_str(raw).map_err(|_| store::invalid("invalid_cursor"))?;
        if cursor.base.version != 1 {
            return Err(store::invalid("unsupported_cursor_version"));
        }
        Ok(cursor)
    }
}
impl Workspace {
    pub fn extraction_xlsx_range(&self, input: ReadXlsxRange) -> Result<XlsxRangePage> {
        let limit = input.validate()?;
        let ((first, left), (last, right)) = bounds(&input.range)?;
        let tx = self.conn.unchecked_transaction()?;
        let job = store::job(&tx, &self.path, &input.id)?;
        let generation = job
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if generation.extractor != "xlsx_cells"
            || !store::supported(
                &generation.extractor,
                generation.extractor_version,
                &generation.config,
            )
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut base = store::Cursor::new(self, "extraction.xlsx_range", limit)?;
        base.job_id = Some(input.id.clone());
        base.generation_id = Some(generation.id.clone());
        let mut shape = RangeCursor {
            base,
            sheet_id: input.sheet_id,
            range: input.range.clone(),
            header_row: input.header_row,
        };
        let from = if let Some(raw) = &input.cursor {
            let mut cursor = RangeCursor::read(raw)?;
            let ordinal = cursor.base.ordinal.take();
            if cursor != shape || ordinal.is_none_or(|n| n == 0) {
                return Err(store::invalid("cursor_scope_mismatch"));
            }
            ordinal.unwrap()
        } else {
            0
        };
        let attachment = crate::attachments::metadata(&tx, &generation.attachment_id)?;
        store::eligible_for(&attachment, "xlsx_cells")?;
        if attachment.sha256 != generation.source_sha256
            || attachment.bytes != u64::from(generation.source_bytes)
        {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &attachment, || Ok(()))?;
        let all =
            crate::xlsx_storage::load(&tx, &generation.id, &input.id, generation.fragment_count)?;
        let sheet = all
            .manifest
            .sheets
            .iter()
            .find(|s| s.id == input.sheet_id)
            .cloned()
            .ok_or_else(|| store::invalid("xlsx_sheet_not_found"))?;
        let matching: Vec<XlsxCellItem> = all
            .workbook
            .sheets
            .into_iter()
            .flat_map(|s| s.cells.into_iter().map(move |cell| (s.id, cell)))
            .enumerate()
            .filter(|(_, (id, c))| {
                *id == input.sheet_id
                    && (first..=last).contains(&c.row)
                    && (left..=right).contains(&c.column)
            })
            .map(|(ordinal, (sheet_id, cell))| XlsxCellItem {
                ordinal: ordinal as u32,
                locator: XlsxCellLocator {
                    kind: XlsxLocatorKind::XlsxCell,
                    sheet_id,
                    reference: cell.reference.clone(),
                },
                cell,
            })
            .collect();
        let total = matching.len() as u32;
        if input.cursor.is_some() && from >= total {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        let headers = input
            .header_row
            .map(|row| {
                (left..=right)
                    .map(|column| XlsxRangeHeader {
                        column,
                        reference: a1(row, column),
                        item: matching
                            .iter()
                            .find(|i| i.cell.row == row && i.cell.column == column)
                            .cloned(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let items: Vec<_> = matching
            .into_iter()
            .skip(from as usize)
            .take(limit as usize)
            .collect();
        let after = from + items.len() as u32;
        let next_cursor = if after < total {
            shape.base.ordinal = Some(after);
            let raw = serde_json::to_string(&shape)?;
            if raw.len() > 2048 {
                return Err(store::invalid("cursor_size_limit"));
            }
            Some(raw)
        } else {
            None
        };
        let covers_range = input.cursor.is_none() && next_cursor.is_none();
        let covers_generation = covers_range && total == all.manifest.cell_count;
        let out = XlsxRangePage {
            job_id: input.id,
            generation,
            result_from_previous_attempt: job.result_from_previous_attempt,
            manifest: all.manifest,
            sheet,
            range: input.range,
            header_row: input.header_row,
            headers,
            matching_cell_count: total,
            items,
            next_cursor,
            covers_range,
            covers_generation,
        };
        store::response(&out)?;
        Ok(out)
    }
}
