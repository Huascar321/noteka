//! Vista transitoria de regiones verificadas: nunca otra fuente de verdad.
use crate::{contracts::*, error::*, extraction_storage as store, workspace::Workspace};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadPdfOcrTable {
    pub id: String,
    #[schemars(range(min = 1, max = 20))]
    pub page_number: u32,
    #[schemars(length(min = 2, max = 33))]
    pub x_edges: Vec<String>,
    pub y_min: String,
    pub y_max: String,
    pub header_row: Option<u32>,
    #[schemars(range(min = 1, max = 64))]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
impl ReadPdfOcrTable {
    pub fn validate(&self) -> Result<u32> {
        store::uuid(&self.id)?;
        if !(1..=20).contains(&self.page_number)
            || !(2..=33).contains(&self.x_edges.len())
            || self.x_edges.iter().any(|s| s.len() > 512)
            || self.y_min.len() > 512
            || self.y_max.len() > 512
            || self.header_row.is_some_and(|n| !(1..=64).contains(&n))
        {
            return Err(store::invalid("invalid_pdf_table_input"));
        }
        if let Some(s) = &self.cursor {
            TableCursor::read(s)?;
        }
        store::page_limit(self.limit, 32, 64)
    }
    fn choice_hash(&self) -> Result<String> {
        let raw = serde_json::to_vec(&(
            "noteka.pdf_ocr_table.choice.v1",
            noteka_pdf_table::PROFILE,
            self.page_number,
            &self.x_edges,
            &self.y_min,
            &self.y_max,
            self.header_row,
        ))?;
        Ok(format!("{:x}", Sha256::digest(raw)))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfOcrTableCell {
    pub ordinal: u32,
    pub row: u32,
    pub column: u32,
    pub regions: Vec<PdfOcrRegion>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfOcrTableOutside {
    pub ordinal: u32,
    pub page_number: u32,
    pub region_ordinal: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PdfOcrTablePage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: PdfOcrManifest,
    pub page: PdfOcrPageDescriptor,
    pub profile: String,
    pub x_edges: Vec<String>,
    pub y_min: String,
    pub y_max: String,
    pub header_row: Option<u32>,
    pub row_count: u32,
    pub column_count: u32,
    pub cell_count: u32,
    pub outside: Vec<PdfOcrTableOutside>,
    pub limitations: Vec<String>,
    pub items: Vec<PdfOcrTableCell>,
    pub next_cursor: Option<String>,
    pub covers_table: bool,
    pub covers_generation: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TableCursor {
    base: store::Cursor,
    choice_sha256: String,
}
impl TableCursor {
    fn read(raw: &str) -> Result<Self> {
        if raw.len() > 2048 {
            return Err(store::invalid("cursor_size_limit"));
        }
        let c: Self = serde_json::from_str(raw).map_err(|_| store::invalid("invalid_cursor"))?;
        if c.base.version != 1 {
            return Err(store::invalid("unsupported_cursor_version"));
        }
        Ok(c)
    }
}
impl Workspace {
    pub fn extraction_pdf_ocr_table(&self, input: ReadPdfOcrTable) -> Result<PdfOcrTablePage> {
        let limit = input.validate()?;
        let tx = self.conn.unchecked_transaction()?;
        let job = store::job(&tx, &self.path, &input.id)?;
        let generation = job
            .generation
            .ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
        if generation.extractor != "pdf_ocr"
            || !store::supported(
                &generation.extractor,
                generation.extractor_version,
                &generation.config,
            )
        {
            return Err(store::invalid("unsupported_generation_config"));
        }
        let mut base = store::Cursor::new(self, "extraction.pdf_ocr_table", limit)?;
        base.job_id = Some(input.id.clone());
        base.generation_id = Some(generation.id.clone());
        let mut shape = TableCursor {
            base,
            choice_sha256: input.choice_hash()?,
        };
        let from = if let Some(raw) = &input.cursor {
            let mut c = TableCursor::read(raw)?;
            let ordinal = c.base.ordinal.take();
            if c != shape || ordinal.is_none_or(|n| n == 0) {
                return Err(store::invalid("cursor_scope_mismatch"));
            }
            ordinal.unwrap()
        } else {
            0
        };
        let a = crate::attachments::metadata(&tx, &generation.attachment_id)?;
        store::eligible_for(&a, "pdf_ocr")?;
        if a.sha256 != generation.source_sha256 || a.bytes != u64::from(generation.source_bytes) {
            return Err(store::storage("input_stale"));
        }
        store::source(&self.path, &a, || Ok(()))?;
        let all = crate::pdf_ocr_storage::load(
            &tx,
            &generation.id,
            &input.id,
            generation.fragment_count,
        )?;
        let index = (input.page_number - 1) as usize;
        let page = all
            .manifest
            .pages
            .get(index)
            .cloned()
            .ok_or_else(|| store::invalid("pdf_table_page_not_found"))?;
        let offset: u32 = all.manifest.pages[..index]
            .iter()
            .map(|p| p.region_count)
            .sum();
        let recognition = all
            .document
            .pages
            .into_iter()
            .nth(index)
            .unwrap()
            .recognition;
        let layout = noteka_pdf_table::project(noteka_pdf_table::Input {
            source_sha256: generation.source_sha256.clone(),
            page_number: input.page_number,
            recognition,
            x_edges: input.x_edges.clone(),
            y_min: input.y_min.clone(),
            y_max: input.y_max.clone(),
            header_row: input.header_row,
        })
        .map_err(|e| {
            store::invalid(match e {
                noteka_pdf_table::TableError::InvalidInput => "invalid_pdf_table_input",
                noteka_pdf_table::TableError::InvalidEvidence => "invalid_pdf_table_evidence",
                noteka_pdf_table::TableError::CrossingRegion => "pdf_table_crossing_region",
                noteka_pdf_table::TableError::AmbiguousRows => "pdf_table_ambiguous_rows",
                noteka_pdf_table::TableError::StructureLimit => "pdf_table_structure_limit",
                noteka_pdf_table::TableError::ResponseLimit => "response_size_limit",
            })
        })?;
        let row_count = layout.rows.len() as u32;
        let column_count = input.x_edges.len() as u32 - 1;
        let cell_count = row_count * column_count;
        if input.cursor.is_some() && from >= cell_count {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        let items = layout
            .rows
            .iter()
            .flat_map(|r| r.cells.iter().map(move |c| (r.number, c)))
            .enumerate()
            .skip(from as usize)
            .take(limit as usize)
            .map(|(ordinal, (row, c))| PdfOcrTableCell {
                ordinal: ordinal as u32,
                row,
                column: c.column,
                regions: c
                    .region_ordinals
                    .iter()
                    .map(|i| {
                        let r = &layout.evidence.recognition.regions[*i as usize];
                        PdfOcrRegion {
                            ordinal: offset + r.ordinal,
                            page_number: input.page_number,
                            region_ordinal: r.ordinal,
                            text: r.text.clone(),
                            confidence: r.confidence.clone(),
                            bbox: r.bbox.clone(),
                        }
                    })
                    .collect(),
            })
            .collect::<Vec<_>>();
        let outside = layout
            .outside
            .iter()
            .map(|n| PdfOcrTableOutside {
                ordinal: offset + n,
                page_number: input.page_number,
                region_ordinal: *n,
            })
            .collect();
        let after = from + items.len() as u32;
        let next_cursor = if after < cell_count {
            shape.base.ordinal = Some(after);
            let raw = serde_json::to_string(&shape)?;
            if raw.len() > 2048 {
                return Err(store::invalid("cursor_size_limit"));
            }
            Some(raw)
        } else {
            None
        };
        let covers_table = input.cursor.is_none() && next_cursor.is_none();
        // La biblioteca sólo declara el hash; aquí original/canónico sí fueron verificados.
        let limitations = layout
            .limitations
            .into_iter()
            .filter(|s| s != "declared_hash_not_file_or_workspace_verification")
            .chain(std::iter::once(
                "outside_regions_referenced_not_returned_as_table_cells".into(),
            ))
            .collect();
        let out = PdfOcrTablePage {
            job_id: input.id,
            generation,
            result_from_previous_attempt: job.result_from_previous_attempt,
            manifest: all.manifest,
            page,
            profile: layout.profile,
            x_edges: input.x_edges,
            y_min: input.y_min,
            y_max: input.y_max,
            header_row: input.header_row,
            row_count,
            column_count,
            cell_count,
            outside,
            limitations,
            items,
            next_cursor,
            covers_table,
            covers_generation: false,
        };
        store::response(&out)?;
        Ok(out)
    }
}
