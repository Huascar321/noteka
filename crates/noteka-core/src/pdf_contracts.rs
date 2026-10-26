//! Contratos aditivos: los localizadores nunca son offsets del binario PDF.
use crate::{contracts::ExtractionGeneration, error::*, extraction_storage as store};
use noteka_pdf_protocol::{MAX_PAGES, MAX_TEXT_BYTES, PARSER_NAME, PARSER_VERSION, is_sha256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadPdfFragments {
    pub id: String,
    #[schemars(range(min = 1, max = 64))]
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
impl ReadPdfFragments {
    pub fn validate(&self) -> Result<u32> {
        store::uuid(&self.id)?;
        let limit = store::page_limit(self.limit, 32, 64)?;
        if let Some(cursor) = &self.cursor {
            store::Cursor::read(cursor)?;
        }
        Ok(limit)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PdfPageState {
    Text,
    NoText,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PdfTextScope {
    PageTextObjects,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PdfLimitation {
    NoOcr,
    NoVisualOrder,
    NoGeometry,
    NoStructuredTables,
}

pub const PDF_LIMITATIONS: [PdfLimitation; 4] = [
    PdfLimitation::NoOcr,
    PdfLimitation::NoVisualOrder,
    PdfLimitation::NoGeometry,
    PdfLimitation::NoStructuredTables,
];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfPageDescriptor {
    #[schemars(range(min = 1, max = 100))]
    pub page_number: u32,
    pub state: PdfPageState,
    #[schemars(range(max = 1_048_576))]
    pub text_bytes: u32,
    pub text_sha256: String,
    #[schemars(range(max = 512))]
    pub fragment_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfManifest {
    pub version: u32,
    pub parser_name: String,
    pub parser_version: String,
    pub scope: PdfTextScope,
    pub limitations: Vec<PdfLimitation>,
    #[schemars(range(min = 1, max = 100))]
    pub page_count: u32,
    #[schemars(range(max = 1_048_576))]
    pub extracted_bytes: u32,
    #[schemars(length(min = 1, max = 100))]
    pub pages: Vec<PdfPageDescriptor>,
}
impl PdfManifest {
    pub fn validate(&self) -> Result<()> {
        let parser_ok = (self.parser_name == PARSER_NAME && self.parser_version == PARSER_VERSION)
            || (self.parser_name == noteka_pdfium_protocol::PARSER_NAME
                && self.parser_version == noteka_pdfium_protocol::PARSER_VERSION)
            || (self.parser_name == "paddleocr_ppstructurev3" && self.parser_version == "3.7.0");
        if self.version != 1
            || !parser_ok
            || self.limitations != PDF_LIMITATIONS
            || !(1..=MAX_PAGES as u32).contains(&self.page_count)
            || self.pages.len() != self.page_count as usize
            || self.extracted_bytes as usize > MAX_TEXT_BYTES
        {
            return Err(store::storage("invalid_pdf_manifest"));
        }
        let mut total = 0u32;
        let mut count = 0u32;
        for (index, page) in self.pages.iter().enumerate() {
            if page.page_number as usize != index + 1
                || !is_sha256(&page.text_sha256)
                || page.text_bytes as usize > MAX_TEXT_BYTES
                || page.fragment_count > 512
                || match page.state {
                    PdfPageState::NoText => {
                        page.text_bytes != 0
                            || page.fragment_count != 0
                            || page.text_sha256 != text_hash("")
                    }
                    PdfPageState::Text => {
                        page.text_bytes == 0
                            || page.fragment_count == 0
                            || page.fragment_count > page.text_bytes
                    }
                }
            {
                return Err(store::storage("invalid_pdf_page"));
            }
            total = total
                .checked_add(page.text_bytes)
                .ok_or_else(|| store::storage("invalid_pdf_manifest"))?;
            count = count
                .checked_add(page.fragment_count)
                .ok_or_else(|| store::storage("invalid_pdf_manifest"))?;
        }
        if total != self.extracted_bytes || count > 512 {
            return Err(store::storage("invalid_pdf_manifest"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PdfLocatorKind {
    PdfPageText,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfTextLocator {
    pub kind: PdfLocatorKind,
    #[schemars(range(min = 1, max = 100))]
    pub page_number: u32,
    pub text_byte_start: u32,
    pub text_byte_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfFragment {
    #[schemars(range(max = 511))]
    pub ordinal: u32,
    pub text: String,
    pub locator: PdfTextLocator,
}
impl PdfFragment {
    pub fn validate(&self) -> Result<()> {
        if self.ordinal >= 512
            || self.text.is_empty()
            || self.text.len() > 4096
            || self.text.contains('\0')
            || !(1..=100).contains(&self.locator.page_number)
            || self.locator.text_byte_end as usize > MAX_TEXT_BYTES
            || self
                .locator
                .text_byte_end
                .checked_sub(self.locator.text_byte_start)
                != Some(self.text.len() as u32)
        {
            return Err(store::storage("invalid_pdf_fragment"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PdfFragmentPage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: PdfManifest,
    pub items: Vec<PdfFragment>,
    pub covers_generation: bool,
    pub next_cursor: Option<String>,
}

/// Validación del resultado completo antes de paginar; no lee ni parsea originales.
pub fn validate_pdf_fragments(manifest: &PdfManifest, fragments: &[PdfFragment]) -> Result<()> {
    manifest.validate()?;
    if fragments.len() > 512 {
        return Err(store::storage("invalid_pdf_fragments"));
    }
    let mut ordinal = 0usize;
    for page in &manifest.pages {
        let mut text = String::new();
        for _ in 0..page.fragment_count {
            let fragment = fragments
                .get(ordinal)
                .ok_or_else(|| store::storage("invalid_pdf_fragments"))?;
            fragment.validate()?;
            if fragment.ordinal as usize != ordinal
                || fragment.locator.page_number != page.page_number
                || fragment.locator.text_byte_start as usize != text.len()
                || fragment.locator.text_byte_end > page.text_bytes
            {
                return Err(store::storage("invalid_pdf_fragments"));
            }
            text.push_str(&fragment.text);
            ordinal += 1;
        }
        if text.len() != page.text_bytes as usize || text_hash(&text) != page.text_sha256 {
            return Err(store::storage("invalid_pdf_page_text"));
        }
    }
    if ordinal != fragments.len() {
        return Err(store::storage("invalid_pdf_fragments"));
    }
    Ok(())
}

fn text_hash(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn result() -> (PdfManifest, Vec<PdfFragment>) {
        let manifest = PdfManifest {
            version: 1,
            parser_name: PARSER_NAME.into(),
            parser_version: PARSER_VERSION.into(),
            scope: PdfTextScope::PageTextObjects,
            limitations: PDF_LIMITATIONS.to_vec(),
            page_count: 2,
            extracted_bytes: 3,
            pages: vec![
                PdfPageDescriptor {
                    page_number: 1,
                    state: PdfPageState::Text,
                    text_bytes: 3,
                    text_sha256: text_hash("á\n"),
                    fragment_count: 2,
                },
                PdfPageDescriptor {
                    page_number: 2,
                    state: PdfPageState::NoText,
                    text_bytes: 0,
                    text_sha256: text_hash(""),
                    fragment_count: 0,
                },
            ],
        };
        let fragments = vec![
            PdfFragment {
                ordinal: 0,
                text: "á".into(),
                locator: PdfTextLocator {
                    kind: PdfLocatorKind::PdfPageText,
                    page_number: 1,
                    text_byte_start: 0,
                    text_byte_end: 2,
                },
            },
            PdfFragment {
                ordinal: 1,
                text: "\n".into(),
                locator: PdfTextLocator {
                    kind: PdfLocatorKind::PdfPageText,
                    page_number: 1,
                    text_byte_start: 2,
                    text_byte_end: 3,
                },
            },
        ];
        (manifest, fragments)
    }

    #[test]
    fn pdf_input_has_strict_fields_and_limits() {
        let id = "00000000-0000-0000-0000-000000000001";
        for limit in [None, Some(1), Some(64)] {
            let input = ReadPdfFragments {
                id: id.into(),
                limit,
                cursor: None,
            };
            assert_eq!(input.validate().unwrap(), limit.unwrap_or(32));
        }
        for limit in [0, 65, u32::MAX] {
            assert!(
                ReadPdfFragments {
                    id: id.into(),
                    limit: Some(limit),
                    cursor: None
                }
                .validate()
                .is_err()
            );
        }
        for field in ["password", "sql", "helper_path", "command", "version"] {
            let mut bad = json!({"id":id});
            bad[field] = json!(1);
            assert!(serde_json::from_value::<ReadPdfFragments>(bad).is_err());
        }
        for cursor in ["".into(), "x".repeat(2049), "invalid".into()] {
            assert!(
                ReadPdfFragments {
                    id: id.into(),
                    limit: None,
                    cursor: Some(cursor)
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            ReadPdfFragments {
                id: "not-an-id".into(),
                limit: None,
                cursor: None
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn pdf_dtos_reject_unknown_fields_and_locator_kinds() {
        let (m, f) = result();
        for field in ["original_byte_start", "ocr", "extra"] {
            let mut v = serde_json::to_value(&m).unwrap();
            v[field] = json!(0);
            assert!(serde_json::from_value::<PdfManifest>(v).is_err());
            let mut v = serde_json::to_value(&f[0]).unwrap();
            v[field] = json!(0);
            assert!(serde_json::from_value::<PdfFragment>(v).is_err());
        }
        let mut v = serde_json::to_value(&m).unwrap();
        v["pages"][0]["line_start"] = json!(1);
        assert!(serde_json::from_value::<PdfManifest>(v).is_err());
        let mut v = serde_json::to_value(&f[0]).unwrap();
        v["locator"]["byte_start"] = json!(0);
        assert!(serde_json::from_value::<PdfFragment>(v).is_err());
        assert!(serde_json::from_value::<PdfTextLocator>(json!({"kind":"original_bytes","page_number":1,"text_byte_start":0,"text_byte_end":1})).is_err());
    }

    #[test]
    fn pdf_manifest_versions_counts_states_and_hashes_are_closed() {
        let (m, _) = result();
        m.validate().unwrap();
        for (field, value) in [
            ("version", json!(2)),
            ("parser_version", json!("other")),
            ("parser_name", json!("other")),
            ("page_count", json!(0)),
            ("page_count", json!(101)),
            ("extracted_bytes", json!(4)),
            ("limitations", json!([])),
        ] {
            let mut v = serde_json::to_value(&m).unwrap();
            v[field] = value;
            assert!(
                serde_json::from_value::<PdfManifest>(v)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
        for (field, value) in [
            ("page_number", json!(2)),
            ("state", json!("no_text")),
            ("text_bytes", json!(0)),
            ("text_sha256", json!("bad")),
            ("fragment_count", json!(0)),
            ("fragment_count", json!(u32::MAX)),
        ] {
            let mut v = serde_json::to_value(&m).unwrap();
            v["pages"][0][field] = value;
            assert!(
                serde_json::from_value::<PdfManifest>(v)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
        let mut wrong_empty = m;
        wrong_empty.pages[1].text_sha256 = "a".repeat(64);
        assert!(wrong_empty.validate().is_err());
    }

    #[test]
    fn pdf_ranges_hashes_and_page_local_reconstruction_are_verified() {
        let (m, f) = result();
        validate_pdf_fragments(&m, &f).unwrap();
        for (path, value) in [
            ("ordinal", json!(1)),
            ("text", json!("b")),
            ("text", json!("\0")),
        ] {
            let mut v = serde_json::to_value(&f).unwrap();
            v[0][path] = value;
            assert!(
                validate_pdf_fragments(&m, &serde_json::from_value::<Vec<PdfFragment>>(v).unwrap())
                    .is_err()
            );
        }
        for (field, value) in [
            ("page_number", 2),
            ("text_byte_start", 1),
            ("text_byte_end", 1),
            ("text_byte_end", u32::MAX),
        ] {
            let mut v = serde_json::to_value(&f).unwrap();
            v[0]["locator"][field] = json!(value);
            assert!(
                validate_pdf_fragments(&m, &serde_json::from_value::<Vec<PdfFragment>>(v).unwrap())
                    .is_err()
            );
        }
        assert!(validate_pdf_fragments(&m, &f[..1]).is_err());
        let mut extra = f.clone();
        extra.push(f[0].clone());
        assert!(validate_pdf_fragments(&m, &extra).is_err());
        let mut text = f[0].clone();
        text.text = "a".repeat(4096);
        text.locator.text_byte_end = 4096;
        text.validate().unwrap();
        text.text.push('a');
        text.locator.text_byte_end += 1;
        assert!(text.validate().is_err());
    }

    #[test]
    fn pdf_result_accepts_exact_text_fragment_and_page_frontiers() {
        let (mut m, _) = result();
        let text = "é".repeat(2048);
        let fragments: Vec<_> = (0..256)
            .map(|ordinal| PdfFragment {
                ordinal,
                text: text.clone(),
                locator: PdfTextLocator {
                    kind: PdfLocatorKind::PdfPageText,
                    page_number: 1,
                    text_byte_start: ordinal * 4096,
                    text_byte_end: (ordinal + 1) * 4096,
                },
            })
            .collect();
        m.page_count = 1;
        m.pages.truncate(1);
        m.extracted_bytes = 1_048_576;
        m.pages[0].text_bytes = m.extracted_bytes;
        m.pages[0].fragment_count = 256;
        m.pages[0].text_sha256 = text_hash(&text.repeat(256));
        validate_pdf_fragments(&m, &fragments).unwrap();
        m.extracted_bytes += 1;
        assert!(m.validate().is_err());

        let mut pages = Vec::new();
        let mut fragments = Vec::new();
        for number in 1..=100 {
            let count = if number == 1 { 413 } else { 1 };
            pages.push(PdfPageDescriptor {
                page_number: number,
                state: PdfPageState::Text,
                text_bytes: count,
                text_sha256: text_hash(&"a".repeat(count as usize)),
                fragment_count: count,
            });
            for offset in 0..count {
                fragments.push(PdfFragment {
                    ordinal: fragments.len() as u32,
                    text: "a".into(),
                    locator: PdfTextLocator {
                        kind: PdfLocatorKind::PdfPageText,
                        page_number: number,
                        text_byte_start: offset,
                        text_byte_end: offset + 1,
                    },
                });
            }
        }
        m.page_count = 100;
        m.extracted_bytes = 512;
        m.pages = pages;
        validate_pdf_fragments(&m, &fragments).unwrap();
        let mut extra = fragments[0].clone();
        extra.ordinal = 512;
        fragments.push(extra);
        assert!(validate_pdf_fragments(&m, &fragments).is_err());
    }

    fn fingerprint(value: &Value) -> String {
        format!("{:x}", Sha256::digest(serde_json::to_vec(value).unwrap()))
    }

    #[test]
    fn v8_schemas_and_root_references_stay_unchanged() {
        // Fingerprints independientes capturados del CLI antes de añadir DTOs PDF (13/10/2026).
        let mut c = crate::describe();
        for (key, hash) in [
            (
                "inputs",
                "2d83b98269d5896c7b60d2dd375f556096317cc9d3881bf46117abf44ff39ecd",
            ),
            (
                "outputs",
                "2c8a2e74228c217252344bd47a0fcbac15c77fbb95986f5da2e82fa8c2f2af1f",
            ),
            (
                "error_schema",
                "b913819780ad2efb726a35422f2653d4c4fa09f2b05848650d6706046a9fd0d5",
            ),
        ] {
            if key != "error_schema" {
                c[key]
                    .as_object_mut()
                    .unwrap()
                    .remove("extraction.pdf_fragments");
                for op in [
                    "external.link",
                    "external.unlink",
                    "external.list",
                    "agent.ask",
                    "agent.cancel",
                    "agent.trace",
                    "embedding.index",
                    "retrieval.search",
                    "extraction.index",
                    "extraction.search",
                    "extraction.xlsx_cells",
                    "extraction.ocr_regions",
                    "extraction.pdfkit_pages",
                    "extraction.pdf_ocr_regions",
                    "extraction.xlsx_range",
                    "extraction.pdf_ocr_table",
                    "extraction.pdf_structure",
                ] {
                    c[key].as_object_mut().unwrap().remove(op);
                }
            }
            assert_eq!(
                fingerprint(&c[key]),
                hash,
                "{key}: cambió un contrato previo"
            );
        }
        let schema = c["contract_schema"].as_object_mut().unwrap();
        let defs = schema.get_mut("$defs").unwrap().as_object_mut().unwrap();
        // Sólo DTOs aditivos del índice; los fingerprints históricos siguen fijados.
        defs.retain(|name, _| {
            !name.starts_with("AttachmentIndex")
                && !name.starts_with("AttachmentSearch")
                && !name.starts_with("Agent")
                && !name.starts_with("Embedding")
                && !name.starts_with("Semantic")
                && !name.starts_with("Retrieval")
                && !name.starts_with("Xlsx")
                && !name.starts_with("Ocr")
                && !name.starts_with("Pdfkit")
                && !name.starts_with("PdfOcr")
                && !name.starts_with("PdfStructure")
                && !name.starts_with("PdfTable")
                && !name.starts_with("External")
                && !matches!(
                    name.as_str(),
                    "LinkExternalResource" | "ListExternalResources" | "UnlinkExternalResource"
                )
                && !name.starts_with("ReadPdfStructure")
                && !matches!(
                    name.as_str(),
                    "ReadPdfOcrRegions"
                        | "ReadPdfOcrTable"
                        | "RasterIdentity"
                        | "ReadPdfkitPages"
                        | "Engine"
                        | "Page8"
                        | "PageStatus"
                        | "ReadOcrRegions"
                        | "ReadXlsxCells"
                        | "ReadXlsxRange"
                        | "Cell"
                        | "CellValue"
                        | "Formula"
                        | "FormulaKind"
                        | "NumberStyle"
                        | "Visibility"
                        | "AttachmentTextLocator"
                        | "IndexedSource"
                        | "IndexedExtractor"
                        | "IndexCoverageScope"
                        | "IndexValidation"
                        | "IndexSourceValidation"
                )
        });
        if let Some(one_of) = defs
            .get_mut("Operation")
            .and_then(|operation| operation.get_mut("oneOf"))
            .and_then(Value::as_array_mut)
        {
            one_of.retain(|variant| {
                !matches!(
                    variant["properties"]["operation"]["const"].as_str(),
                    Some("external.link" | "external.unlink" | "external.list")
                )
            });
        }
        for name in [
            "Operation",
            "ReadPdfFragments",
            "PdfFragmentPage",
            "PdfManifest",
            "PdfPageDescriptor",
            "PdfPageState",
            "PdfTextScope",
            "PdfLimitation",
            "PdfFragment",
            "PdfTextLocator",
            "PdfLocatorKind",
            "Page9",
        ] {
            defs.remove(name);
        }
        assert_eq!(defs.len(), 140);
        assert_eq!(
            fingerprint(&schema["$defs"]),
            "61632f9c67075e044de95b5606f915ef7d72316b9f795c73583b1016351890d6"
        );
        let props = schema
            .get_mut("properties")
            .unwrap()
            .as_object_mut()
            .unwrap();
        props.remove("pdf_fragments");
        props.remove("read_pdf_fragments");
        props.remove("attachment_index");
        props.remove("attachment_search");
        props.remove("read_xlsx_cells");
        props.remove("xlsx_cells");
        props.remove("read_ocr_regions");
        props.remove("ocr_regions");
        props.remove("pdfkit_pages");
        props.remove("read_pdfkit_pages");
        props.remove("read_pdf_ocr_regions");
        props.remove("pdf_ocr_regions");
        props.remove("read_xlsx_range");
        props.remove("xlsx_range");
        props.remove("read_pdf_ocr_table");
        props.remove("pdf_ocr_table");
        props.remove("pdf_structure");
        props.remove("external_resource");
        props.remove("external_resources");
        props.remove("external_resource_result");
        for name in [
            "embedding_index",
            "embedding_status",
            "retrieval",
            "agent_run",
            "agent_trace",
            "agent_traces",
        ] {
            props.remove(name);
        }
        assert_eq!(
            fingerprint(&schema["properties"]),
            "c9915e01031c7b52d6d9d29a6d25c784b699de516695eccfa8bc6a13af250186"
        );
        schema
            .get_mut("required")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .retain(|v| {
                v != "pdf_fragments"
                    && v != "read_pdf_fragments"
                    && v != "attachment_index"
                    && v != "attachment_search"
                    && v != "read_xlsx_cells"
                    && v != "xlsx_cells"
                    && v != "read_ocr_regions"
                    && v != "ocr_regions"
                    && v != "pdfkit_pages"
                    && v != "read_pdfkit_pages"
                    && v != "read_pdf_ocr_regions"
                    && v != "pdf_ocr_regions"
                    && v != "read_xlsx_range"
                    && v != "xlsx_range"
                    && v != "read_pdf_ocr_table"
                    && v != "pdf_ocr_table"
                    && v != "pdf_structure"
                    && v != "embedding_index"
                    && v != "embedding_status"
                    && v != "retrieval"
                    && v != "agent_run"
                    && v != "agent_trace"
                    && v != "agent_traces"
                    && v != "external_resource"
                    && v != "external_resources"
                    && v != "external_resource_result"
            });
        assert_eq!(
            fingerprint(&schema["required"]),
            "dcca9a26a774eae93d834a188ee32cd6921029d5be6ac6b5fbb7cb0ba39a69e7"
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfStructure {
    pub version: u32,
    pub pages: Vec<PdfStructurePage>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfStructurePage {
    pub page_number: u32,
    pub tables: Vec<PdfTable>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfTable {
    pub rows: u32,
    pub columns: u32,
    pub cells: Vec<PdfTableCell>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PdfTableCell {
    pub row: u32,
    pub column: u32,
    pub row_span: u32,
    pub col_span: u32,
    pub text: String,
}
impl PdfStructure {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.version != 1 || self.pages.is_empty() || self.pages.len() > 20 {
            return Err(store::invalid("pdf_structure_limit"));
        }
        let mut tables = 0u32;
        for (index, page) in self.pages.iter().enumerate() {
            if page.page_number as usize != index + 1 {
                return Err(store::invalid("pdf_structure_pages"));
            }
            for table in &page.tables {
                tables += 1;
                if tables > 512
                    || table.rows == 0
                    || table.columns == 0
                    || table.rows > 64
                    || table.columns > 64
                    || table.cells.is_empty()
                    || table.cells.len() > 4096
                {
                    return Err(store::invalid("pdf_structure_limit"));
                }
                let mut occupied = std::collections::HashSet::new();
                for cell in &table.cells {
                    if cell.row_span == 0
                        || cell.col_span == 0
                        || cell.row_span > 64
                        || cell.col_span > 64
                        || cell.row + cell.row_span > table.rows
                        || cell.column + cell.col_span > table.columns
                        || cell.text.is_empty()
                        || cell.text.len() > 512
                        || cell.text.contains('\0')
                    {
                        return Err(store::invalid("pdf_structure_cell"));
                    }
                    for row in cell.row..cell.row + cell.row_span {
                        for column in cell.column..cell.column + cell.col_span {
                            if !occupied.insert((row, column)) {
                                return Err(store::invalid("pdf_structure_overlap"));
                            }
                        }
                    }
                }
            }
        }
        if serde_json::to_vec(self)?.len() > 1_048_576 {
            return Err(store::invalid("pdf_structure_limit"));
        }
        Ok(())
    }
    pub(crate) fn table_count(&self) -> u32 {
        self.pages.iter().map(|p| p.tables.len() as u32).sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadPdfStructure {
    pub id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PdfStructureManifest {
    pub version: u32,
    pub format: String,
    pub source_sha256: String,
    pub structure_sha256: String,
    pub structure_bytes: u32,
    pub page_count: u32,
    pub table_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PdfStructureResult {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub manifest: PdfStructureManifest,
    pub structure: PdfStructure,
}
