//! OOXML Transitional cerrado. Los números y las fórmulas nunca se evalúan.
use crate::container;
use noteka_xlsx_protocol::{self as p, *};
use roxmltree::{Document, Node, ParsingOptions};
use std::collections::{BTreeMap, HashMap, HashSet};

type Result<T> = std::result::Result<T, WorkerError>;
use WorkerError as E;
const MAIN: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const PACKAGE: &str = "http://schemas.openxmlformats.org/package/2006/relationships";
const TYPES: &str = "http://schemas.openxmlformats.org/package/2006/content-types";
const MAIN_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml";
const SHEET_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml";

fn xml(bytes: &[u8]) -> Result<Document<'_>> {
    let s = std::str::from_utf8(bytes).map_err(|_| E::XlsxInvalidXml)?;
    let declaration = s.trim_start_matches('\u{feff}');
    if declaration.starts_with("<?xml")
        && matches!(
            declaration.as_bytes().get(5),
            Some(b' ' | b'\t' | b'\r' | b'\n')
        )
    {
        let end = declaration
            .find("?>")
            .filter(|&n| n <= 1024)
            .ok_or(E::XlsxInvalidXml)?;
        let d = &declaration[..end];
        if let Some(at) = d.find("encoding") {
            let value = d[at + 8..]
                .trim_start()
                .strip_prefix('=')
                .ok_or(E::XlsxInvalidXml)?
                .trim_start();
            let quote = value
                .chars()
                .next()
                .filter(|c| matches!(c, '\'' | '"'))
                .ok_or(E::XlsxInvalidXml)?;
            let encoding = value[1..].split(quote).next().ok_or(E::XlsxInvalidXml)?;
            if !encoding.eq_ignore_ascii_case("utf-8") {
                return Err(E::XlsxUnsupportedFormat);
            }
        }
    }
    let doc = Document::parse_with_options(
        s,
        ParsingOptions {
            allow_dtd: false,
            nodes_limit: MAX_NODES,
            entity_resolver: None,
        },
    )
    .map_err(|e| {
        if matches!(e, roxmltree::Error::NodesLimitReached) {
            E::XlsxXmlLimit
        } else {
            E::XlsxInvalidXml
        }
    })?;
    for n in doc.descendants().filter(Node::is_element) {
        if n.ancestors()
            .filter(Node::is_element)
            .take(MAX_DEPTH + 1)
            .count()
            > MAX_DEPTH
        {
            return Err(E::XlsxXmlLimit);
        }
        if n.tag_name().namespace()
            == Some("http://schemas.openxmlformats.org/markup-compatibility/2006")
            || n.tag_name()
                .namespace()
                .is_some_and(|s| s.starts_with("http://purl.oclc.org/ooxml/"))
        {
            return Err(E::XlsxUnsupportedFormat);
        }
    }
    Ok(doc)
}
fn root<'a, 'i>(d: &'a Document<'i>, name: &str, ns: &str) -> Result<Node<'a, 'i>> {
    let n = d.root_element();
    if n.has_tag_name((ns, name)) {
        Ok(n)
    } else {
        Err(E::XlsxUnsupportedFormat)
    }
}
fn children<'a, 'i>(n: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    n.children().filter(Node::is_element)
}
fn required<'a, 'i>(n: Node<'a, 'i>, key: &str) -> Result<&'a str> {
    n.attribute(key).ok_or(E::XlsxInvalidWorkbook)
}
fn unsigned(s: &str) -> Result<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(E::XlsxInvalidWorkbook);
    }
    s.parse().map_err(|_| E::XlsxInvalidWorkbook)
}
fn boolean(s: &str) -> Result<bool> {
    match s {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(E::XlsxInvalidWorkbook),
    }
}
fn one<'a, 'i>(n: Node<'a, 'i>, name: &str) -> Result<Option<Node<'a, 'i>>> {
    let mut nodes = children(n).filter(|n| n.has_tag_name((MAIN, name)));
    let first = nodes.next();
    if nodes.next().is_some() {
        return Err(E::XlsxInvalidWorkbook);
    }
    Ok(first)
}
fn plain(n: Node<'_, '_>) -> Result<String> {
    plain_bounded(n, MAX_TEXT)
}
fn plain_bounded(n: Node<'_, '_>, limit: usize) -> Result<String> {
    if children(n).next().is_some() {
        return Err(E::XlsxUnsupportedCell);
    }
    let mut s = String::new();
    for text in n.children().filter(Node::is_text).filter_map(|n| n.text()) {
        if s.len() + text.len() > limit {
            return Err(E::XlsxStructureLimit);
        }
        s.push_str(text);
    }
    Ok(s)
}
/// ST_Xstring tiene escapes UTF-16 además de las entidades XML. Una sola
/// pasada conserva _x005F_x0041_ como el texto literal _x0041_.
fn xstring(s: &str) -> Result<String> {
    fn unit(b: &[u8]) -> Option<u16> {
        if b.len() < 7 || &b[..2] != b"_x" || b[6] != b'_' {
            return None;
        }
        let hex = std::str::from_utf8(&b[2..6]).ok()?;
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        u16::from_str_radix(hex, 16).ok()
    }
    let mut out = String::new();
    let mut at = 0;
    while at < s.len() {
        if let Some(first) = unit(&s.as_bytes()[at..]) {
            at += 7;
            let scalar = if (0xd800..=0xdbff).contains(&first) {
                let second = unit(&s.as_bytes()[at..])
                    .filter(|n| (0xdc00..=0xdfff).contains(n))
                    .ok_or(E::XlsxUnsupportedCell)?;
                at += 7;
                0x10000 + (u32::from(first) - 0xd800) * 1024 + u32::from(second) - 0xdc00
            } else {
                u32::from(first)
            };
            let c = char::from_u32(scalar)
                .filter(|c| *c != '\0')
                .ok_or(E::XlsxUnsupportedCell)?;
            out.push(c);
        } else {
            let c = s[at..].chars().next().ok_or(E::XlsxInvalidCell)?;
            out.push(c);
            at += c.len_utf8();
        }
        if out.len() > MAX_TEXT {
            return Err(E::XlsxStructureLimit);
        }
    }
    Ok(out)
}
fn rich(n: Node<'_, '_>) -> Result<String> {
    let mut out = String::new();
    let mut simple = false;
    let mut runs = false;
    for part in children(n) {
        let text = if part.has_tag_name((MAIN, "t")) && !simple && !runs {
            simple = true;
            xstring(&plain_bounded(part, MAX_ENTRY)?)?
        } else if part.has_tag_name((MAIN, "r")) && !simple {
            runs = true;
            if children(part)
                .any(|n| !n.has_tag_name((MAIN, "t")) && !n.has_tag_name((MAIN, "rPr")))
            {
                return Err(E::XlsxUnsupportedCell);
            }
            xstring(&plain_bounded(
                one(part, "t")?.ok_or(E::XlsxInvalidCell)?,
                MAX_ENTRY,
            )?)?
        } else {
            return Err(E::XlsxUnsupportedCell);
        };
        if out.len() + text.len() > MAX_TEXT {
            return Err(E::XlsxStructureLimit);
        }
        out.push_str(&text);
    }
    // Una cadena vacía requiere <t/>: no se infiere texto omitido de un nodo desconocido.
    if !simple && !runs {
        return Err(E::XlsxInvalidCell);
    }
    Ok(out)
}

struct Relationship {
    id: String,
    kind: String,
    target: String,
}
fn rel_path(part: &str) -> String {
    match part.rsplit_once('/') {
        Some((dir, file)) => format!("{dir}/_rels/{file}.rels"),
        None => format!("_rels/{part}.rels"),
    }
}
fn owner(path: &str) -> Result<String> {
    if path == "_rels/.rels" {
        return Ok(String::new());
    }
    let (dir, file) = path
        .rsplit_once("/_rels/")
        .or_else(|| path.strip_prefix("_rels/").map(|f| ("", f)))
        .ok_or(E::XlsxInvalidWorkbook)?;
    let file = file.strip_suffix(".rels").ok_or(E::XlsxInvalidWorkbook)?;
    let part = if dir.is_empty() {
        file.into()
    } else {
        format!("{dir}/{file}")
    };
    if !container::canonical(&part) {
        return Err(E::XlsxInvalidWorkbook);
    }
    Ok(part)
}
fn target(part: &str, value: &str) -> Result<String> {
    let path = if let Some(value) = value.strip_prefix('/') {
        value.to_owned()
    } else if let Some((dir, _)) = part.rsplit_once('/') {
        format!("{dir}/{value}")
    } else {
        value.to_owned()
    };
    if !container::canonical(&path) {
        return Err(E::XlsxInvalidWorkbook);
    }
    Ok(path)
}
fn active(s: &str) -> bool {
    let s = s.to_ascii_lowercase();
    [
        "macroenabled",
        "vbaproject",
        "externallink",
        "connections",
        "querytable",
        "activex",
        "oleobject",
        "macrosheet",
        "dialogsheet",
        "sheet.binary",
    ]
    .iter()
    .any(|v| s.contains(v))
}
fn relationships(parts: &BTreeMap<String, Vec<u8>>) -> Result<HashMap<String, Vec<Relationship>>> {
    let mut out = HashMap::new();
    for (name, bytes) in parts {
        if active(name) {
            return Err(E::XlsxActiveContent);
        }
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let doc = xml(bytes)?;
            if name.ends_with(".rels") {
                let part = owner(name)?;
                if !part.is_empty() && !parts.contains_key(&part) {
                    return Err(E::XlsxInvalidWorkbook);
                }
                let n = root(&doc, "Relationships", PACKAGE)?;
                let mut ids = HashSet::new();
                let mut rels = Vec::new();
                for r in children(n) {
                    if !r.has_tag_name((PACKAGE, "Relationship")) {
                        return Err(E::XlsxInvalidWorkbook);
                    }
                    let id = required(r, "Id")?;
                    let kind = required(r, "Type")?;
                    if id.is_empty()
                        || id.len() > 512
                        || !ids.insert(id.to_owned())
                        || kind.len() > 512
                    {
                        return Err(E::XlsxInvalidWorkbook);
                    }
                    if let Some(mode) = r.attribute("TargetMode") {
                        if mode == "External" {
                            return Err(E::XlsxExternalRelationship);
                        }
                        if mode != "Internal" {
                            return Err(E::XlsxInvalidWorkbook);
                        }
                    }
                    if active(kind) {
                        return Err(E::XlsxActiveContent);
                    }
                    let target = target(&part, required(r, "Target")?)?;
                    if !parts.contains_key(&target) {
                        return Err(E::XlsxInvalidWorkbook);
                    }
                    rels.push(Relationship {
                        id: id.into(),
                        kind: kind.into(),
                        target,
                    });
                }
                out.insert(part, rels);
            }
        }
    }
    Ok(out)
}
struct ContentTypes {
    defaults: HashMap<String, String>,
    overrides: HashMap<String, String>,
}
impl ContentTypes {
    fn get(&self, path: &str) -> Result<&str> {
        self.overrides
            .get(path)
            .or_else(|| {
                path.rsplit_once('.')
                    .and_then(|(_, ext)| self.defaults.get(ext))
            })
            .map(String::as_str)
            .ok_or(E::XlsxInvalidWorkbook)
    }
    fn expect(&self, path: &str, kind: &str) -> Result<()> {
        if self.get(path)? == kind {
            Ok(())
        } else {
            Err(E::XlsxUnsupportedFormat)
        }
    }
}
fn content_types(parts: &BTreeMap<String, Vec<u8>>) -> Result<ContentTypes> {
    let d = xml(parts
        .get("[Content_Types].xml")
        .ok_or(E::XlsxInvalidWorkbook)?)?;
    let n = root(&d, "Types", TYPES)?;
    let mut out = ContentTypes {
        defaults: HashMap::new(),
        overrides: HashMap::new(),
    };
    for c in children(n) {
        let kind = required(c, "ContentType")?;
        if active(kind) {
            return Err(E::XlsxActiveContent);
        }
        if kind.is_empty() || kind.len() > 512 {
            return Err(E::XlsxInvalidWorkbook);
        }
        let old = if c.has_tag_name((TYPES, "Default")) {
            let ext = required(c, "Extension")?;
            if ext.is_empty() || !ext.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Err(E::XlsxInvalidWorkbook);
            }
            out.defaults.insert(ext.into(), kind.into())
        } else if c.has_tag_name((TYPES, "Override")) {
            let path = required(c, "PartName")?
                .strip_prefix('/')
                .ok_or(E::XlsxInvalidWorkbook)?;
            if !container::canonical(path) || !parts.contains_key(path) {
                return Err(E::XlsxInvalidWorkbook);
            }
            out.overrides.insert(path.into(), kind.into())
        } else {
            return Err(E::XlsxInvalidWorkbook);
        };
        if old.is_some() {
            return Err(E::XlsxInvalidWorkbook);
        }
    }
    for name in parts.keys().filter(|n| n.as_str() != "[Content_Types].xml") {
        out.get(name)?;
    }
    Ok(out)
}
fn unique_rel<'a>(rels: &'a [Relationship], suffix: &str) -> Result<Option<&'a Relationship>> {
    let kind = format!("{REL}/{suffix}");
    let mut found = rels.iter().filter(|r| r.kind == kind);
    let first = found.next();
    if found.next().is_some() {
        return Err(E::XlsxInvalidWorkbook);
    }
    Ok(first)
}
fn strings(bytes: &[u8]) -> Result<Vec<String>> {
    let doc = xml(bytes)?;
    let n = root(&doc, "sst", MAIN)?;
    let mut out = Vec::new();
    for s in children(n) {
        if !s.has_tag_name((MAIN, "si")) {
            return Err(E::XlsxUnsupportedCell);
        }
        if out.len() == MAX_STRINGS {
            return Err(E::XlsxStructureLimit);
        }
        out.push(rich(s)?);
    }
    if let Some(count) = n.attribute("uniqueCount")
        && unsigned(count)? as usize != out.len()
    {
        return Err(E::XlsxInvalidWorkbook);
    }
    if let Some(count) = n.attribute("count") {
        unsigned(count)?;
    }
    Ok(out)
}
fn styles(bytes: &[u8]) -> Result<Vec<NumberStyle>> {
    let doc = xml(bytes)?;
    let n = root(&doc, "styleSheet", MAIN)?;
    let mut formats = HashMap::new();
    if let Some(fmts) = one(n, "numFmts")? {
        for f in children(fmts) {
            if !f.has_tag_name((MAIN, "numFmt")) {
                return Err(E::XlsxInvalidWorkbook);
            }
            let id = unsigned(required(f, "numFmtId")?)?;
            let code = xstring(required(f, "formatCode")?)?;
            if code.is_empty() || code.len() > MAX_TEXT || formats.insert(id, code).is_some() {
                return Err(E::XlsxInvalidWorkbook);
            }
            if formats.len() > MAX_STYLES {
                return Err(E::XlsxStructureLimit);
            }
        }
        if let Some(count) = fmts.attribute("count")
            && unsigned(count)? as usize != formats.len()
        {
            return Err(E::XlsxInvalidWorkbook);
        }
    }
    let xfs = one(n, "cellXfs")?.ok_or(E::XlsxInvalidWorkbook)?;
    let mut out = Vec::new();
    for xf in children(xfs) {
        if !xf.has_tag_name((MAIN, "xf")) {
            return Err(E::XlsxInvalidWorkbook);
        }
        let id = unsigned(xf.attribute("numFmtId").unwrap_or("0"))?;
        let code = formats.get(&id).map(|s| (*s).to_owned());
        if id >= 164 && code.is_none() {
            return Err(E::XlsxInvalidWorkbook);
        }
        if out.len() == MAX_STYLES {
            return Err(E::XlsxStructureLimit);
        }
        out.push(NumberStyle {
            num_fmt_id: id,
            format_code: code,
        });
    }
    if out.is_empty() {
        return Err(E::XlsxInvalidWorkbook);
    }
    if let Some(count) = xfs.attribute("count")
        && unsigned(count)? as usize != out.len()
    {
        return Err(E::XlsxInvalidWorkbook);
    }
    Ok(out)
}
fn formula(n: Node<'_, '_>) -> Result<Formula> {
    if n.attributes()
        .any(|a| a.namespace().is_some() || !matches!(a.name(), "t" | "ref" | "si"))
    {
        return Err(E::XlsxUnsupportedFormula);
    }
    let kind = match n.attribute("t").unwrap_or("normal") {
        "normal" => FormulaKind::Normal,
        "shared" => FormulaKind::Shared,
        "array" => FormulaKind::Array,
        _ => return Err(E::XlsxUnsupportedFormula),
    };
    Ok(Formula {
        kind,
        text: plain(n)?,
        reference: n.attribute("ref").map(str::to_owned),
        shared_index: n.attribute("si").map(unsigned).transpose()?,
    })
}
fn cell(n: Node<'_, '_>, row: u32, shared: &[String], style_count: usize) -> Result<Cell> {
    if n.attributes()
        .any(|a| a.namespace().is_some() || !matches!(a.name(), "r" | "s" | "t"))
    {
        return Err(E::XlsxUnsupportedCell);
    }
    if children(n).any(|n| !["v", "f", "is"].iter().any(|s| n.has_tag_name((MAIN, *s)))) {
        return Err(E::XlsxUnsupportedCell);
    }
    let reference = required(n, "r")?;
    let (actual_row, column) = coordinate(reference).map_err(|_| E::XlsxInvalidCell)?;
    let style = n.attribute("s").map(unsigned).transpose()?.unwrap_or(0);
    if row != actual_row || style as usize >= style_count {
        return Err(E::XlsxInvalidCell);
    }
    let f = one(n, "f")?.map(formula).transpose()?;
    let kind = n.attribute("t").unwrap_or("n");
    let v = one(n, "v")?
        .map(|n| plain_bounded(n, if kind == "str" { MAX_ENTRY } else { MAX_TEXT }))
        .transpose()?;
    let inline = one(n, "is")?;
    if inline.is_some() && kind != "inlineStr" {
        return Err(E::XlsxInvalidCell);
    }
    let cached = if kind == "inlineStr" {
        if v.is_some() || f.is_some() {
            return Err(E::XlsxInvalidCell);
        }
        Some(CellValue::Text(rich(inline.ok_or(E::XlsxInvalidCell)?)?))
    } else if let Some(v) = v {
        Some(match kind {
            "n" if number(&v) => CellValue::Number(v),
            "n" => return Err(E::XlsxInvalidCell),
            "s" => CellValue::Text(
                shared
                    .get(unsigned(&v)? as usize)
                    .ok_or(E::XlsxInvalidCell)?
                    .clone(),
            ),
            "str" => CellValue::Text(xstring(&v)?),
            "b" => CellValue::Boolean(boolean(&v).map_err(|_| E::XlsxInvalidCell)?),
            "e" if !v.is_empty() => CellValue::Error(v),
            "d" if !v.is_empty() => CellValue::DateIso(v),
            "e" | "d" => return Err(E::XlsxInvalidCell),
            _ => return Err(E::XlsxUnsupportedCell),
        })
    } else {
        if !matches!(kind, "n" | "s" | "str" | "b" | "e" | "d") {
            return Err(E::XlsxUnsupportedCell);
        }
        None
    };
    Ok(Cell {
        reference: reference.into(),
        row,
        column,
        style,
        cached,
        formula: f,
    })
}
fn worksheet(
    bytes: &[u8],
    mut sheet: Sheet,
    shared: &[String],
    style_count: usize,
    count: &mut usize,
    size: &mut usize,
) -> Result<Sheet> {
    let d = xml(bytes)?;
    let n = root(&d, "worksheet", MAIN)?;
    for c in children(n) {
        if ![
            "sheetPr",
            "dimension",
            "sheetViews",
            "sheetFormatPr",
            "cols",
            "sheetData",
            "sheetCalcPr",
            "sheetProtection",
            "protectedRanges",
            "autoFilter",
            "sortState",
            "mergeCells",
            "conditionalFormatting",
            "dataValidations",
            "hyperlinks",
            "printOptions",
            "pageMargins",
            "pageSetup",
            "headerFooter",
            "rowBreaks",
            "colBreaks",
            "ignoredErrors",
            "drawing",
            "legacyDrawing",
            "legacyDrawingHF",
            "picture",
            "tableParts",
        ]
        .iter()
        .any(|s| c.has_tag_name((MAIN, *s)))
        {
            return Err(E::XlsxUnsupportedFormat);
        }
    }
    sheet.declared_range = one(n, "dimension")?
        .map(|n| required(n, "ref").map(str::to_owned))
        .transpose()?;
    if let Some(r) = &sheet.declared_range {
        range(r).map_err(|_| E::XlsxInvalidCell)?;
    }
    let data = one(n, "sheetData")?.ok_or(E::XlsxInvalidWorkbook)?;
    let mut prev_row = 0;
    for r in children(data) {
        if !r.has_tag_name((MAIN, "row")) {
            return Err(E::XlsxUnsupportedCell);
        }
        let row = unsigned(required(r, "r")?)?;
        if row <= prev_row || row > 1_048_576 {
            return Err(E::XlsxInvalidCell);
        }
        prev_row = row;
        let mut prev_col = 0;
        for c in children(r) {
            if !c.has_tag_name((MAIN, "c")) {
                return Err(E::XlsxUnsupportedCell);
            }
            if *count == MAX_CELLS {
                return Err(E::XlsxStructureLimit);
            }
            let c = cell(c, row, shared, style_count)?;
            if c.column <= prev_col {
                return Err(E::XlsxInvalidCell);
            }
            prev_col = c.column;
            *size = size
                .checked_add(json_size(&c, MAX_RESULT).map_err(|_| E::XlsxResultLimit)?)
                .filter(|s| *s <= MAX_RESULT)
                .ok_or(E::XlsxResultLimit)?;
            *count += 1;
            sheet.cells.push(c);
        }
    }
    if let Some(merges) = one(n, "mergeCells")? {
        for m in children(merges) {
            if !m.has_tag_name((MAIN, "mergeCell")) {
                return Err(E::XlsxInvalidWorkbook);
            }
            if sheet.merges.len() == MAX_MERGES {
                return Err(E::XlsxStructureLimit);
            }
            let r = required(m, "ref")?;
            range(r).map_err(|_| E::XlsxInvalidCell)?;
            sheet.merges.push(r.into());
        }
        if let Some(count) = merges.attribute("count")
            && unsigned(count)? as usize != sheet.merges.len()
        {
            return Err(E::XlsxInvalidWorkbook);
        }
    }
    sheet.observed_range = observed(&sheet.cells);
    Ok(sheet)
}

pub fn extract(bytes: &[u8]) -> Result<Workbook> {
    let parts = container::read(bytes)?;
    let types = content_types(&parts)?;
    let rels = relationships(&parts)?;
    let root_rels = rels.get("").ok_or(E::XlsxInvalidWorkbook)?;
    let book = unique_rel(root_rels, "officeDocument")?.ok_or(E::XlsxInvalidWorkbook)?;
    types.expect(&book.target, MAIN_TYPE)?;
    let book_rels = rels.get(&book.target).ok_or(E::XlsxInvalidWorkbook)?;
    // También comprobar que el path de relaciones se corresponde con el workbook.
    if !parts.contains_key(&rel_path(&book.target)) {
        return Err(E::XlsxInvalidWorkbook);
    }
    let doc = xml(parts.get(&book.target).ok_or(E::XlsxInvalidWorkbook)?)?;
    let n = root(&doc, "workbook", MAIN)?;
    for c in children(n) {
        if ![
            "fileVersion",
            "fileSharing",
            "workbookPr",
            "workbookProtection",
            "bookViews",
            "sheets",
            "definedNames",
            "calcPr",
            "pivotCaches",
            "webPublishing",
            "fileRecoveryPr",
        ]
        .iter()
        .any(|s| c.has_tag_name((MAIN, *s)))
        {
            return Err(E::XlsxUnsupportedFormat);
        }
    }
    let date_1904 = one(n, "workbookPr")?
        .and_then(|n| n.attribute("date1904"))
        .map(boolean)
        .transpose()?
        .unwrap_or(false);
    let shared = if let Some(r) = unique_rel(book_rels, "sharedStrings")? {
        types.expect(
            &r.target,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml",
        )?;
        strings(parts.get(&r.target).ok_or(E::XlsxInvalidWorkbook)?)?
    } else {
        Vec::new()
    };
    let styles = if let Some(r) = unique_rel(book_rels, "styles")? {
        types.expect(
            &r.target,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml",
        )?;
        styles(parts.get(&r.target).ok_or(E::XlsxInvalidWorkbook)?)?
    } else {
        vec![NumberStyle {
            num_fmt_id: 0,
            format_code: None,
        }]
    };
    let sheets = one(n, "sheets")?.ok_or(E::XlsxInvalidWorkbook)?;
    let mut out = Workbook {
        date_1904,
        limitations: LIMITATIONS.iter().map(|s| (*s).into()).collect(),
        styles,
        sheets: Vec::new(),
    };
    let mut used = HashSet::new();
    let mut count = 0;
    let mut size = 0;
    for s in children(sheets) {
        if !s.has_tag_name((MAIN, "sheet")) {
            return Err(E::XlsxUnsupportedFormat);
        }
        if out.sheets.len() == MAX_SHEETS {
            return Err(E::XlsxStructureLimit);
        }
        let rid = s.attribute((REL, "id")).ok_or(E::XlsxInvalidWorkbook)?;
        let r = book_rels
            .iter()
            .find(|r| r.id == rid)
            .ok_or(E::XlsxInvalidWorkbook)?;
        if r.kind != format!("{REL}/worksheet") || !used.insert(&r.target) {
            return Err(E::XlsxUnsupportedFormat);
        }
        types.expect(&r.target, SHEET_TYPE)?;
        let sheet = Sheet {
            id: unsigned(required(s, "sheetId")?)?,
            name: xstring(required(s, "name")?)?,
            visibility: match s.attribute("state").unwrap_or("visible") {
                "visible" => Visibility::Visible,
                "hidden" => Visibility::Hidden,
                "veryHidden" => Visibility::VeryHidden,
                _ => return Err(E::XlsxInvalidWorkbook),
            },
            declared_range: None,
            observed_range: None,
            merges: Vec::new(),
            cells: Vec::new(),
        };
        out.sheets.push(worksheet(
            parts.get(&r.target).ok_or(E::XlsxInvalidWorkbook)?,
            sheet,
            &shared,
            out.styles.len(),
            &mut count,
            &mut size,
        )?);
    }
    // Diferenciar exceso de representación de una fórmula/grupo/celda inválida.
    if p::json_size(&out, MAX_RESULT).is_err() {
        return Err(E::XlsxResultLimit);
    }
    for sheet in &out.sheets {
        sheet
            .validate_formulas()
            .map_err(|_| E::XlsxInvalidFormula)?;
    }
    out.validate().map_err(|_| E::XlsxInvalidWorkbook)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xstring_is_utf16_aware_and_never_recursively_unescapes() {
        for (raw, expected) in [
            ("_x005F_x0041_", "_x0041_"),
            ("_xD83D__xDE80_", "🚀"),
            ("a_x000D_Ñ", "a\rÑ"),
            ("_xZZZZ_", "_xZZZZ_"),
        ] {
            assert_eq!(xstring(raw).unwrap(), expected);
        }
        for raw in ["_xD800_", "_xDC00_", "_xD800__x0041_", "_x0000_"] {
            assert_eq!(xstring(raw), Err(E::XlsxUnsupportedCell));
        }
    }
    #[test]
    fn escaped_text_frontier_counts_logical_utf8_not_encoded_lexeme() {
        for size in [MAX_TEXT, MAX_TEXT + 1] {
            let input = format!("<si xmlns='{MAIN}'><t>{}</t></si>", "_x0041_".repeat(size));
            let d = xml(input.as_bytes()).unwrap();
            let result = rich(root(&d, "si", MAIN).unwrap());
            if size == MAX_TEXT {
                assert_eq!(result.unwrap(), "A".repeat(MAX_TEXT));
            } else {
                assert_eq!(result, Err(E::XlsxStructureLimit));
            }
        }
    }
    #[test]
    fn xml_frontiers_and_namespace_resolution_are_not_string_matching() {
        let input = format!("<s:worksheet xmlns:s='{MAIN}'><s:sheetData/></s:worksheet>");
        let d = xml(input.as_bytes());
        assert!(d.is_ok());
        for depth in [64, 65] {
            for content in ["", "text", "<![CDATA[text]]><!--comment-->"] {
                let input = format!("{}{content}{}", "<x>".repeat(depth), "</x>".repeat(depth));
                assert_eq!(xml(input.as_bytes()).is_ok(), depth == 64);
            }
        }
        for separator in [" ", "\t", "\r", "\n", " \t"] {
            let good = format!("<?xml{separator}version='1.0' encoding='UTF-8'?><x/>");
            assert!(xml(good.as_bytes()).is_ok());
            let bad = good.replace("UTF-8", "ISO-8859-1");
            assert!(matches!(xml(bad.as_bytes()), Err(E::XlsxUnsupportedFormat)));
        }
        for input in ["<!DOCTYPE x><x/>", "<x>&custom;</x>", "<x a='1' a='2'/>"] {
            assert!(xml(input.as_bytes()).is_err());
        }
    }
    #[test]
    fn canonical_relationships_do_not_normalize_traversal_or_aliases() {
        assert_eq!(
            target("xl/workbook.xml", "worksheets/sheet1.xml").unwrap(),
            "xl/worksheets/sheet1.xml"
        );
        assert_eq!(
            target("xl/workbook.xml", "/xl/worksheets/sheet1.xml").unwrap(),
            "xl/worksheets/sheet1.xml"
        );
        for input in [
            "../sheet.xml",
            "//xl/sheet.xml",
            "file:///secret",
            "%73heet.xml",
            "a\\b.xml",
        ] {
            assert!(target("xl/workbook.xml", input).is_err());
        }
        assert_eq!(
            owner(&rel_path("xl/workbook.xml")).unwrap(),
            "xl/workbook.xml"
        );
    }
}
