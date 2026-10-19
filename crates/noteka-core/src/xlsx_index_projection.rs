//! Sólo campos de un Workbook íntegro; nunca ZIP/XML, fórmulas evaluadas ni chunks JSON.
use crate::{batch_storage::Budget, contracts::*, error::*, extraction_storage as store};
use noteka_xlsx_protocol::{CellValue, Workbook};

pub(crate) fn project(w: &Workbook, b: &Budget<'_>) -> Result<Vec<AttachmentSearchFragment>> {
    let mut out = Vec::new();
    let mut bytes = 0usize;
    for (cell_ordinal, (sheet, cell)) in w
        .sheets
        .iter()
        .flat_map(|s| s.cells.iter().map(move |c| (s, c)))
        .enumerate()
    {
        b.check()?;
        let mut field = |text: &str, field: XlsxTextField| -> Result<()> {
            bytes = bytes
                .checked_add(text.len())
                .ok_or_else(|| store::invalid("index_budget_exceeded"))?;
            if bytes > MAX_EXTRACTION_BYTES as usize {
                return Err(store::invalid("index_budget_exceeded"));
            }
            let mut start = 0;
            while start < text.len() {
                b.check()?;
                if out.len() >= MAX_EXTRACTION_FRAGMENTS as usize {
                    return Err(store::invalid("index_budget_exceeded"));
                }
                let mut end = (start + EXTRACTION_CHUNK_BYTES as usize).min(text.len());
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                out.push(AttachmentSearchFragment {
                    ordinal: out.len() as u32,
                    text: text[start..end].to_owned(),
                    locator: AttachmentTextLocator::XlsxCellText {
                        projection: XlsxTextProjection::XlsxCellFieldsV1,
                        cell_ordinal: cell_ordinal as u32,
                        sheet_id: sheet.id,
                        reference: cell.reference.clone(),
                        field,
                        field_bytes: text.len() as u32,
                        text_byte_start: start as u32,
                        text_byte_end: end as u32,
                    },
                });
                start = end;
            }
            Ok(())
        };
        if let Some(value) = &cell.cached {
            match value {
                CellValue::Text(s) => field(s, XlsxTextField::CachedText)?,
                CellValue::Number(s) => field(s, XlsxTextField::CachedNumber)?,
                CellValue::Boolean(v) => field(
                    if *v { "true" } else { "false" },
                    XlsxTextField::CachedBoolean,
                )?,
                CellValue::Error(s) => field(s, XlsxTextField::CachedError)?,
                CellValue::DateIso(s) => field(s, XlsxTextField::CachedDateIso)?,
            }
        }
        if let Some(formula) = &cell.formula {
            field(&formula.text, XlsxTextField::FormulaText)?;
        }
    }
    b.check()?;
    Ok(out)
}
