use crate::{
    contracts::*,
    entities::{MAX_BLOCKS, get},
    error::*,
    workspace::{Workspace, publish},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::{collections::HashSet, fs, path::Path};

pub(crate) fn validate(d: &Document) -> Result<()> {
    if ![1, 2, 3].contains(&d.schema_version) {
        return Err(CoreError::validation(
            "Versión de documento editable no admitida.",
        ));
    }
    let mut ids = HashSet::new();
    fn walk(
        b: &Block,
        v: u32,
        ids: &mut HashSet<String>,
        parent: Option<&BlockKind>,
    ) -> Result<()> {
        if b.id.trim().is_empty() || !ids.insert(b.id.clone()) || ids.len() > MAX_BLOCKS {
            return Err(CoreError::validation(
                "IDs vacíos/duplicados o más de 10.000 bloques.",
            ));
        }
        let attrs = b.attrs.as_ref().and_then(Value::as_object);
        if b.attrs.is_some() && attrs.is_none() {
            return Err(CoreError::validation("attrs debe ser un objeto."));
        }
        if v == 1
            && (!matches!(b.r#type, BlockKind::Paragraph)
                || b.attrs.is_some()
                || !b.children.is_empty()
                || b.content
                    .iter()
                    .any(|t| !t.marks.is_empty() || !matches!(t.r#type, TextKind::Text)))
        {
            return Err(CoreError::validation(
                "La versión 1 sólo admite párrafos de texto.",
            ));
        }
        let allowed: &[&str] = match b.r#type {
            BlockKind::Heading => &["level"],
            BlockKind::OrderedList => &["start"],
            BlockKind::TaskItem => &["checked"],
            BlockKind::CodeBlock => &["language"],
            BlockKind::Image => &["attachment_id", "alt"],
            BlockKind::File => &["attachment_id", "name"],
            BlockKind::CollectionView => &["collection_id", "view_id"],
            _ => &[],
        };
        if attrs.is_some_and(|a| a.keys().any(|k| !allowed.contains(&k.as_str()))) {
            return Err(CoreError::validation("Atributo de bloque no soportado."));
        }
        let a = b.attrs.clone().unwrap_or(json!({}));
        if matches!(b.r#type, BlockKind::CollectionView)
            && (v != 3
                || ["collection_id", "view_id"].iter().any(|k| {
                    !a[k]
                        .as_str()
                        .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
                }))
        {
            return Err(CoreError::validation(
                "Vista incrustada requiere documento v3 e IDs locales válidos.",
            ));
        }
        match b.r#type {
            BlockKind::Heading if !a["level"].as_u64().is_some_and(|n| (1..=3).contains(&n)) => {
                return Err(CoreError::validation("Encabezado requiere level 1–3."));
            }
            BlockKind::OrderedList
                if !a["start"]
                    .as_u64()
                    .is_some_and(|n| (1..=1_000_000).contains(&n)) =>
            {
                return Err(CoreError::validation("Lista requiere start positivo."));
            }
            BlockKind::TaskItem if !a["checked"].is_boolean() => {
                return Err(CoreError::validation(
                    "Checklist requiere checked booleano.",
                ));
            }
            BlockKind::Image | BlockKind::File
                if !a["attachment_id"]
                    .as_str()
                    .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok()) =>
            {
                return Err(CoreError::validation("Adjunto requiere ID válido."));
            }
            _ => {}
        }
        for k in ["language", "alt", "name"] {
            if let Some(x) = a.get(k)
                && !x.is_null()
                && !x.is_string()
            {
                return Err(CoreError::validation("Atributo textual inválido."));
            }
        }
        let container = matches!(
            b.r#type,
            BlockKind::BulletList
                | BlockKind::OrderedList
                | BlockKind::TaskList
                | BlockKind::ListItem
                | BlockKind::TaskItem
                | BlockKind::Blockquote
        );
        if container && (!b.content.is_empty() || b.children.is_empty()) {
            return Err(CoreError::validation(
                "Contenedor requiere hijos y no texto inline.",
            ));
        }
        if !container && !b.children.is_empty() {
            return Err(CoreError::validation("Bloque hoja no admite hijos."));
        }
        if matches!(
            b.r#type,
            BlockKind::HorizontalRule
                | BlockKind::Image
                | BlockKind::File
                | BlockKind::CollectionView
        ) && !b.content.is_empty()
        {
            return Err(CoreError::validation(
                "Bloque atómico no admite texto inline.",
            ));
        }
        if matches!(b.r#type, BlockKind::ListItem)
            && !matches!(parent, Some(BlockKind::BulletList | BlockKind::OrderedList))
        {
            return Err(CoreError::validation("list_item fuera de lista."));
        }
        if matches!(b.r#type, BlockKind::TaskItem) && !matches!(parent, Some(BlockKind::TaskList)) {
            return Err(CoreError::validation("task_item fuera de checklist."));
        }
        if matches!(b.r#type, BlockKind::ListItem | BlockKind::TaskItem)
            && !b
                .children
                .first()
                .is_some_and(|c| matches!(c.r#type, BlockKind::Paragraph))
        {
            return Err(CoreError::validation(
                "Item de lista requiere un primer párrafo.",
            ));
        }
        for t in &b.content {
            if matches!(t.r#type, TextKind::HardBreak)
                && (!t.text.is_empty() || !t.marks.is_empty())
            {
                return Err(CoreError::validation("Salto de línea inválido."));
            }
            let mut marks = HashSet::new();
            for m in &t.marks {
                if !marks.insert(&m.r#type) {
                    return Err(CoreError::validation("Marca duplicada."));
                }
                match m.r#type {
                    MarkKind::Link if !m.href.as_ref().is_some_and(|s| safe_link(s)) => {
                        return Err(CoreError::validation("Enlace no seguro."));
                    }
                    MarkKind::Link => {}
                    _ if m.href.is_some() => {
                        return Err(CoreError::validation("href sólo corresponde a link."));
                    }
                    _ => {}
                }
            }
            if matches!(b.r#type, BlockKind::CodeBlock)
                && (!t.marks.is_empty() || !matches!(t.r#type, TextKind::Text))
            {
                return Err(CoreError::validation(
                    "Código sólo admite texto sin marcas.",
                ));
            }
            if t.marks.iter().any(|m| matches!(m.r#type, MarkKind::Code)) && t.marks.len() > 1 {
                return Err(CoreError::validation(
                    "La marca code no se combina con otras marcas.",
                ));
            }
        }
        for child in &b.children {
            if matches!(b.r#type, BlockKind::BulletList | BlockKind::OrderedList)
                && !matches!(child.r#type, BlockKind::ListItem)
                || matches!(b.r#type, BlockKind::TaskList)
                    && !matches!(child.r#type, BlockKind::TaskItem)
            {
                return Err(CoreError::validation("Hijo incompatible con lista."));
            }
            walk(child, v, ids, Some(&b.r#type))?;
        }
        Ok(())
    }
    for b in &d.blocks {
        walk(b, d.schema_version, &mut ids, None)?;
    }
    Ok(())
}
pub(crate) fn safe_link(s: &str) -> bool {
    !s.chars().any(char::is_control)
        && ["https://", "http://", "mailto:"]
            .iter()
            .any(|p| s.starts_with(p))
}
pub(crate) fn attachment_ids(value: &Value) -> HashSet<String> {
    fn walk(v: &Value, out: &mut HashSet<String>) {
        match v {
            Value::Object(o) => {
                if let Some(s) = o.get("attachment_id").and_then(Value::as_str) {
                    out.insert(s.into());
                }
                for x in o.values() {
                    walk(x, out);
                }
            }
            Value::Array(a) => {
                for x in a {
                    walk(x, out);
                }
            }
            _ => {}
        }
    }
    let mut out = HashSet::new();
    walk(value, &mut out);
    out
}
pub(crate) fn validate_references(conn: &Connection, root: &Path, d: &Value) -> Result<()> {
    for id in attachment_ids(d) {
        let a = crate::attachments::metadata(conn, &id)?;
        crate::attachments::verify(root, &a)?;
    }
    fn walk(v: &Value, conn: &Connection) -> Result<()> {
        if v["type"] == "collection_view" {
            let c = v["attrs"]["collection_id"].as_str().unwrap_or("");
            let view = v["attrs"]["view_id"].as_str().unwrap_or("");
            crate::database::collection(conn, c)?;
            let owner: String =
                conn.query_row("SELECT collection_id FROM views WHERE id=?1", [view], |r| {
                    r.get(0)
                })?;
            if owner != c {
                return Err(CoreError::validation(
                    "Vista incrustada pertenece a otra colección.",
                ));
            }
        }
        if v["type"] == "image" {
            let id = v["attrs"]["attachment_id"].as_str().unwrap_or("");
            let a = crate::attachments::metadata(conn, id)?;
            if !["image/png", "image/jpeg", "image/gif", "image/webp"].contains(&a.mime.as_str()) {
                return Err(CoreError::validation(
                    "El bloque imagen requiere un original de imagen admitida.",
                ));
            }
        }
        if let Some(o) = v.as_object() {
            for child in o.values() {
                walk(child, conn)?;
            }
        }
        if let Some(a) = v.as_array() {
            for child in a {
                walk(child, conn)?;
            }
        }
        Ok(())
    }
    walk(d, conn)
}
fn markdown(b: &Block) -> String {
    let inline = b
        .content
        .iter()
        .map(|t| {
            let mut s = t
                .text
                .replace('\\', "\\\\")
                .replace('*', "\\*")
                .replace('_', "\\_");
            for m in &t.marks {
                s = match m.r#type {
                    MarkKind::Bold => format!("**{s}**"),
                    MarkKind::Italic => format!("*{s}*"),
                    MarkKind::Strike => format!("~~{s}~~"),
                    MarkKind::Code => format!("`{s}`"),
                    MarkKind::Link => format!("[{s}]({})", m.href.as_deref().unwrap_or("")),
                };
            }
            if matches!(t.r#type, TextKind::HardBreak) {
                "  \n".into()
            } else {
                s
            }
        })
        .collect::<String>();
    let children = b.children.iter().map(markdown).collect::<Vec<_>>();
    let a = b.attrs.clone().unwrap_or(json!({}));
    match b.r#type {
        BlockKind::Heading => format!(
            "{} {inline}",
            "#".repeat(a["level"].as_u64().unwrap_or(1) as usize)
        ),
        BlockKind::BulletList => children
            .iter()
            .map(|s| format!("- {}", s.replace('\n', "\n  ")))
            .collect::<Vec<_>>()
            .join("\n"),
        BlockKind::OrderedList => children
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "{}. {}",
                    a["start"].as_u64().unwrap_or(1) + i as u64,
                    s.replace('\n', "\n   ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        BlockKind::TaskList => children.join("\n"),
        BlockKind::TaskItem => format!(
            "- [{}] {}",
            if a["checked"] == true { "x" } else { " " },
            children.join("\n")
        ),
        BlockKind::ListItem => children.join("\n"),
        BlockKind::Blockquote => children
            .join("\n")
            .lines()
            .map(|s| format!("> {s}"))
            .collect::<Vec<_>>()
            .join("\n"),
        BlockKind::HorizontalRule => "---".into(),
        BlockKind::CodeBlock => {
            let raw = b
                .content
                .iter()
                .map(|t| t.text.as_str())
                .collect::<String>();
            let fence = "`".repeat(
                raw.split(|c| c != '`')
                    .map(str::len)
                    .max()
                    .unwrap_or(0)
                    .max(2)
                    + 1,
            );
            format!(
                "{fence}{}\n{raw}\n{fence}",
                a["language"].as_str().unwrap_or("")
            )
        }
        BlockKind::Image => format!(
            "![{}](attachments/{})",
            a["alt"].as_str().unwrap_or(""),
            a["attachment_id"].as_str().unwrap_or("")
        ),
        BlockKind::File => format!(
            "[{}](attachments/{})",
            a["name"].as_str().unwrap_or("archivo"),
            a["attachment_id"].as_str().unwrap_or("")
        ),
        BlockKind::CollectionView => format!(
            "[Vista local: colección {} · vista {} — registros no incluidos; usá backup del workspace]",
            a["collection_id"].as_str().unwrap_or(""),
            a["view_id"].as_str().unwrap_or("")
        ),
        _ => inline,
    }
}
impl Workspace {
    pub fn document_export(&self, input: ExportDocument) -> Result<ExportResult> {
        let tx = self.conn.unchecked_transaction()?;
        let entity = get(&tx, &input.id)?;
        let d = entity
            .document
            .as_ref()
            .ok_or_else(|| CoreError::validation("La entidad no tiene documento."))?;
        let mut stmt=tx.prepare("SELECT id,source_id,target_id,kind FROM relations WHERE source_id=?1 OR target_id=?1 ORDER BY id")?;
        let relations=stmt.query_map([&input.id],|r|Ok(json!({"id":r.get::<_,String>(0)?,"source_id":r.get::<_,String>(1)?,"target_id":r.get::<_,String>(2)?,"kind":r.get::<_,String>(3)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
        let mut warnings = vec![];
        fn has_view(v: &Value) -> bool {
            v["type"] == "collection_view"
                || v.as_object().is_some_and(|o| o.values().any(has_view))
                || v.as_array().is_some_and(|a| a.iter().any(has_view))
        }
        if has_view(d) {
            warnings.push("Referencias a vistas conservadas, pero sus colecciones/esquemas/registros no se incluyen transitivamente. Usá un backup completo del workspace para transportarlos.".into());
        }
        let body = match input.format {
            ExportFormat::Json => serde_json::to_string_pretty(d)?,
            ExportFormat::Markdown => {
                if !entity.document_editable {
                    return Err(CoreError::new(
                        ErrorCode::UnsupportedDocumentFormat,
                        "Exportá JSON para recuperar este formato.",
                    ));
                }
                warnings.push("Markdown no conserva IDs, marcas/metadatos completos ni referencias de relaciones; usá JSON para una exportación fiel.".into());
                let doc: Document = serde_json::from_value(d.clone())?;
                format!(
                    "# {}\n\n{}\n",
                    entity.title,
                    doc.blocks
                        .iter()
                        .map(markdown)
                        .collect::<Vec<_>>()
                        .join("\n\n")
                )
            }
        };
        let destination = Path::new(&input.destination);
        publish(destination, |stage| {
            fs::write(
                stage.join(match input.format {
                    ExportFormat::Json => "document.json",
                    ExportFormat::Markdown => "document.md",
                }),
                &body,
            )?;
            fs::write(
                stage.join("entity.json"),
                serde_json::to_vec_pretty(&entity)?,
            )?;
            fs::write(
                stage.join("relations.json"),
                serde_json::to_vec_pretty(&relations)?,
            )?;
            let mut originals = vec![];
            let mut ids = attachment_ids(d).into_iter().collect::<Vec<_>>();
            ids.sort();
            for id in ids {
                let a = crate::attachments::metadata(&tx, &id)?;
                crate::attachments::copy_original(&self.path, stage, &a)?;
                originals.push(a);
            }
            fs::write(
                stage.join("manifest.json"),
                serde_json::to_vec_pretty(
                    &json!({"format_version":1,"entity_id":entity.id,"attachments":originals,"warnings":warnings}),
                )?,
            )?;
            Ok(())
        })?;
        Ok(ExportResult {
            path: fs::canonicalize(destination)?.to_string_lossy().into(),
            warnings,
        })
    }
}
