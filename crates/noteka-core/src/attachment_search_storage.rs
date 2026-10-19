//! Índice derivado independiente. No claim, parser, scheduler ni escrituras de reparación.
use crate::{
    attachments,
    batch_storage::{self, Budget},
    contracts::*,
    error::*,
    extraction_storage as store, pdf_storage,
    workspace::{self, DB_NAME, SCHEMA_VERSION},
};
use chrono::DateTime;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Duration,
};

pub(crate) const MIGRATION: &str = include_str!("../migrations/010.sql");
pub(crate) fn reset_ddl() -> &'static str {
    MIGRATION
        .split_once("-- BEGIN RESETTABLE\n")
        .expect("DDL incorporada")
        .1
        .split_once("-- END RESETTABLE")
        .expect("DDL incorporada")
        .0
}
pub(crate) fn error(reason: &str) -> CoreError {
    store::storage(reason)
}
pub(crate) fn conflict(reason: &str, revision: u32) -> CoreError {
    let mut e = store::error(ErrorCode::RevisionConflict, reason);
    e.details["current_revision"] = json!(revision);
    e
}
pub(crate) fn open(path: &Path, write: bool) -> Result<(Connection, PathBuf)> {
    let v = workspace::preflight(path)?;
    if v != SCHEMA_VERSION {
        let mut e = store::error(ErrorCode::UnsupportedWorkspaceVersion, "migration_required");
        e.details = json!({"current_version":v,"supported_version":SCHEMA_VERSION,"migration_required":v<SCHEMA_VERSION});
        return Err(e);
    }
    let root = std::fs::canonicalize(path)?;
    let c = Connection::open_with_flags(
        root.join(DB_NAME),
        (if write {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        }) | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    c.busy_timeout(Duration::from_millis(workspace::BUSY_TIMEOUT_MS))?;
    if write {
        c.pragma_update(None, "foreign_keys", true)?;
        c.pragma_update(None, "synchronous", "FULL")?;
    }
    // Como el default de rusqlite: Connection no es Sync y esta conexión es local
    // a una operación. NO_MUTEX evita locks por acceso a columnas, no los locks
    // de escritor SQLite ni su busy_timeout. No query_only: TEMP necesita escribir;
    // main sigue READ_ONLY en get/search. No compartimos conexión entre threads.
    Ok((c, root))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Control {
    pub revision: u32,
}
pub(crate) fn control(c: &Connection) -> Result<Control> {
    let (n, version, revision, profile, at): (u32,u32,u32,String,String) = c.query_row(
        "SELECT (SELECT count(*) FROM extraction_search_control),version,revision,profile,updated_at FROM extraction_search_control WHERE id=1", [],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
    if version != 1 || profile != ATTACHMENT_INDEX_PROFILE {
        return Err(store::invalid("unsupported_index_version"));
    }
    if n != 1 || revision == 0 || DateTime::parse_from_rfc3339(&at).is_err() {
        return Err(error("invalid_attachment_index"));
    }
    Ok(Control { revision })
}

type SchemaRow = (String, String, String, Option<String>);
fn schema(c: &Connection) -> Result<Vec<SchemaRow>> {
    Ok(c.prepare("SELECT type,name,tbl_name,sql FROM main.sqlite_schema WHERE tbl_name GLOB 'extraction_search_*' ORDER BY type,name")?
        .query_map([], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?.collect::<std::result::Result<_,_>>()?)
}
pub(crate) fn structure(c: &Connection) -> Result<()> {
    static EXPECTED: OnceLock<Vec<SchemaRow>> = OnceLock::new();
    let expected = EXPECTED.get_or_init(|| {
        let c = Connection::open_in_memory().expect("SQLite en memoria");
        c.execute_batch("CREATE TABLE workspace_meta(schema_version INTEGER);")
            .expect("DDL incorporada");
        c.execute_batch(MIGRATION).expect("DDL incorporada");
        schema(&c).expect("Schema incorporado")
    });
    if &schema(c)? != expected {
        return Err(store::invalid("unsupported_index_schema"));
    }
    Ok(())
}

pub(crate) struct Canonical {
    pub generation: ExtractionGeneration,
    pub source: IndexedSource,
    pub fragments: Vec<AttachmentSearchFragment>,
    pub text_bytes: u32,
    pub hash: String,
    pub previous: bool,
}
impl Canonical {
    pub fn entry(&self, indexed_at: String) -> AttachmentIndexEntry {
        AttachmentIndexEntry {
            source: self.source.clone(),
            fragment_count: self.fragments.len() as u32,
            text_bytes: self.text_bytes,
            content_hash: self.hash.clone(),
            indexed_at,
            result_from_previous_attempt: self.previous,
        }
    }
}

// Vista JSON ordenada sin clonar los textos dentro de Value/canonical(Value).
// El orden y los prefijos de longitud son exactamente los del hash persistente v1.
struct SortedJson<'a>(&'a serde_json::Value);
impl Serialize for SortedJson<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::{SerializeMap, SerializeSeq};
        match self.0 {
            serde_json::Value::Object(object) => {
                let mut fields = object.iter().collect::<Vec<_>>();
                fields.sort_unstable_by(|a, b| a.0.cmp(b.0));
                let mut out = s.serialize_map(Some(fields.len()))?;
                for (key, value) in fields {
                    out.serialize_entry(key, &SortedJson(value))?;
                }
                out.end()
            }
            serde_json::Value::Array(values) => {
                let mut out = s.serialize_seq(Some(values.len()))?;
                for value in values {
                    out.serialize_element(&SortedJson(value))?;
                }
                out.end()
            }
            value => value.serialize(s),
        }
    }
}
struct HashFragments<'a>(&'a [AttachmentSearchFragment]);
impl Serialize for HashFragments<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        struct Fragment<'a>(&'a AttachmentSearchFragment);
        impl Serialize for Fragment<'_> {
            fn serialize<S: serde::Serializer>(
                &self,
                s: S,
            ) -> std::result::Result<S::Ok, S::Error> {
                use serde::ser::{Error, SerializeMap};
                let locator = serde_json::to_value(&self.0.locator).map_err(S::Error::custom)?;
                let mut out = s.serialize_map(Some(3))?;
                out.serialize_entry("locator", &SortedJson(&locator))?;
                out.serialize_entry("ordinal", &self.0.ordinal)?;
                out.serialize_entry("text", &self.0.text)?;
                out.end()
            }
        }
        let mut out = s.serialize_seq(Some(self.0.len()))?;
        for fragment in self.0 {
            out.serialize_element(&Fragment(fragment))?;
        }
        out.end()
    }
}
pub(crate) fn content_hash(
    source: &IndexedSource,
    generation: &ExtractionGeneration,
    manifest: Option<&PdfManifest>,
    fragments: &[AttachmentSearchFragment],
) -> Result<String> {
    content_hash_manifest(
        source,
        generation,
        serde_json::to_value(manifest)?,
        fragments,
    )
}
fn content_hash_manifest(
    source: &IndexedSource,
    generation: &ExtractionGeneration,
    manifest: serde_json::Value,
    fragments: &[AttachmentSearchFragment],
) -> Result<String> {
    // Campos en orden lexicográfico; no serializar los structs públicos en su
    // orden declarado. SortedJson cubre recursivamente sus metadatos/configs.
    #[derive(Serialize)]
    struct Content<'a> {
        fragments: HashFragments<'a>,
        generation: SortedJson<'a>,
        manifest: SortedJson<'a>,
        source: SortedJson<'a>,
    }
    let generation = serde_json::to_value(generation)?;
    let source = serde_json::to_value(source)?;
    let encoded = serde_json::to_vec(&Content {
        fragments: HashFragments(fragments),
        generation: SortedJson(&generation),
        manifest: SortedJson(&manifest),
        source: SortedJson(&source),
    })?;
    let mut hash = Sha256::new();
    for bytes in [
        b"noteka.extraction.index.content.v1".as_slice(),
        encoded.as_slice(),
    ] {
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(bytes);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub(crate) fn canonical(c: &Connection, job: &str, b: &Budget<'_>) -> Result<Canonical> {
    b.check()?;
    store::uuid(job)?;
    let (id, attachment, sha, bytes, extractor, version, config, config_hash, state, attempts):
        (Option<String>,String,String,u32,String,u32,String,String,String,u32) = c.query_row(
        "SELECT generation_id,attachment_id,source_sha256,source_bytes,extractor,extractor_version,config,config_hash,state,attempts FROM extraction_jobs WHERE id=?1",[job],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?))).optional()?.ok_or_else(||store::error(ErrorCode::NotFound,"job_not_found"))?;
    let id = id.ok_or_else(|| store::error(ErrorCode::NotFound, "generation_not_found"))?;
    let g = store::generation(c, &id)?;
    let owner: String = c.query_row(
        "SELECT job_id FROM extraction_generations WHERE id=?1",
        [&id],
        |r| r.get(0),
    )?;
    if owner != job
        || attachment != g.attachment_id
        || sha != g.source_sha256
        || bytes != g.source_bytes
        || extractor != g.extractor
        || version != g.extractor_version
        || config_hash != g.config_hash
        || config.len() > 8192
        || store::config_hash(&serde_json::from_str(&config)?)? != g.config_hash
        || attempts < g.attempt
        || !matches!(
            state.as_str(),
            "queued" | "running" | "succeeded" | "failed" | "cancelled" | "stale"
        )
    {
        return Err(error("invalid_job_generation"));
    }
    if !store::supported(&g.extractor, g.extractor_version, &g.config) {
        return Err(store::invalid("unsupported_generation_config"));
    }
    if !matches!(
        g.extractor.as_str(),
        "utf8_text" | "pdf_text" | "xlsx_cells" | "image_ocr"
    ) {
        return Err(store::invalid("unsupported_index_extractor"));
    }
    let a = attachments::metadata(c, &g.attachment_id)?;
    store::eligible_for(&a, &g.extractor)?;
    if a.sha256 != g.source_sha256 || a.bytes != u64::from(g.source_bytes) {
        return Err(error("input_stale"));
    }
    // Backup/preflight de formatos anteriores valida su índice antes de 013.
    // No consultar tablas nuevas mientras la versión efectiva siga siendo v12.
    let ocr_owner: bool = if crate::workspace::version(c)? >= 13 {
        c.query_row(
            "SELECT EXISTS(SELECT 1 FROM extraction_ocr_generations WHERE generation_id=?1)",
            [&g.id],
            |r| r.get(0),
        )?
    } else {
        false
    };
    if ocr_owner != (g.extractor == "image_ocr") {
        return Err(error("invalid_ocr_integrity"));
    }
    let mut manifest = None;
    let mut xlsx_manifest = None;
    let mut ocr_manifest = None;
    let fragments = if g.extractor == "utf8_text" {
        let (count, size):(u32,u32) = c.query_row("SELECT count(*),coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_fragments WHERE generation_id=?1",[&id],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if count != g.fragment_count
            || count > 512
            || size != g.source_bytes
            || size > MAX_EXTRACTION_BYTES
        {
            return Err(error("invalid_fragment"));
        }
        let raw:Vec<(ExtractionFragment,String)> = c.prepare("SELECT ordinal,text,byte_start,byte_end,line_start,line_end,job_id FROM extraction_fragments WHERE generation_id=?1 ORDER BY ordinal")?
            .query_map([&id],|r|Ok((ExtractionFragment {ordinal:r.get(0)?,text:r.get(1)?,byte_start:r.get(2)?,byte_end:r.get(3)?,line_start:r.get(4)?,line_end:r.get(5)?},r.get(6)?)))?.collect::<std::result::Result<_,_>>()?;
        let mut hasher = Sha256::new();
        let (mut offset, mut line) = (0u32, 1u32);
        let mut out = Vec::with_capacity(raw.len());
        for (f, owner) in raw {
            b.check()?;
            let text = f.text.as_bytes();
            if owner != job
                || f.ordinal != out.len() as u32
                || text.is_empty()
                || text.len() > 4096
                || text.contains(&0)
                || f.byte_start != offset
                || f.byte_end.checked_sub(f.byte_start) != Some(text.len() as u32)
                || f.line_start != line
                || f.line_end
                    != line
                        + text[..text.len() - 1]
                            .iter()
                            .filter(|&&x| x == b'\n')
                            .count() as u32
            {
                return Err(error("invalid_fragment"));
            }
            hasher.update(text);
            offset = f.byte_end;
            line += text.iter().filter(|&&x| x == b'\n').count() as u32;
            out.push(AttachmentSearchFragment {
                ordinal: f.ordinal,
                text: f.text,
                locator: AttachmentTextLocator::Utf8Bytes {
                    byte_start: f.byte_start,
                    byte_end: f.byte_end,
                    line_start: f.line_start,
                    line_end: f.line_end,
                },
            });
        }
        if offset != g.source_bytes || format!("{:x}", hasher.finalize()) != g.source_sha256 {
            return Err(error("invalid_fragment"));
        }
        if c.prepare("SELECT 1 FROM extraction_pdf_generations WHERE generation_id=?1")?
            .exists([&id])?
        {
            return Err(error("invalid_fragment"));
        }
        out
    } else if g.extractor == "pdf_text" {
        let pdf = pdf_storage::load(c, &g.id, job, g.fragment_count)?;
        manifest = Some(pdf.manifest);
        pdf.fragments
            .into_iter()
            .map(|f| AttachmentSearchFragment {
                ordinal: f.ordinal,
                text: f.text,
                locator: AttachmentTextLocator::PdfPageText {
                    page_number: f.locator.page_number,
                    text_byte_start: f.locator.text_byte_start,
                    text_byte_end: f.locator.text_byte_end,
                    scope: PdfTextScope::PageTextObjects,
                    limitations: PDF_LIMITATIONS.to_vec(),
                },
            })
            .collect()
    } else if g.extractor == "xlsx_cells" {
        let prepared = crate::xlsx_storage::load(c, &g.id, job, g.fragment_count)?;
        let projected = crate::xlsx_index_projection::project(&prepared.workbook, b)?;
        xlsx_manifest = Some(prepared.manifest);
        projected
    } else if g.extractor == "image_ocr" {
        let prepared = crate::ocr_storage::load(c, &g.id, job, g.fragment_count)?;
        crate::ocr_storage::source_format(&prepared.manifest, &a)?;
        let projected = crate::ocr_index_projection::project(&prepared.recognition, b)?;
        ocr_manifest = Some(prepared.manifest);
        projected
    } else {
        return Err(store::invalid("unsupported_index_extractor"));
    };
    b.check()?;
    let text_bytes = fragments.iter().map(|f| f.text.len() as u32).sum();
    let source = IndexedSource {
        job_id: job.into(),
        attachment_id: g.attachment_id.clone(),
        generation_id: g.id.clone(),
        source_sha256: g.source_sha256.clone(),
        extractor: g.extractor.clone(),
        extractor_version: g.extractor_version,
        config_hash: g.config_hash.clone(),
    };
    let hash = if let Some(manifest) = ocr_manifest {
        content_hash_manifest(&source, &g, serde_json::to_value(manifest)?, &fragments)?
    } else if let Some(manifest) = xlsx_manifest {
        content_hash_manifest(&source, &g, serde_json::to_value(manifest)?, &fragments)?
    } else {
        content_hash(&source, &g, manifest.as_ref(), &fragments)?
    };
    let previous = state != "succeeded" || attempts != g.attempt;
    Ok(Canonical {
        generation: g,
        source,
        fragments,
        text_bytes,
        hash,
        previous,
    })
}

pub(crate) fn source(c: &Connection, root: &Path, data: &Canonical, b: &Budget<'_>) -> Result<()> {
    let a = attachments::metadata(c, &data.source.attachment_id)?;
    if a.sha256 != data.source.source_sha256 || a.bytes != u64::from(data.generation.source_bytes) {
        return Err(error("input_stale"));
    }
    store::source(root, &a, || b.check())?;
    Ok(())
}

pub(crate) struct Selected {
    pub slot: u32,
    pub data: Canonical,
    pub entry: AttachmentIndexEntry,
}
pub(crate) struct Verified<'a> {
    pub conn: &'a Connection,
    pub entries: Vec<Selected>,
}
impl Drop for Verified<'_> {
    fn drop(&mut self) {
        let _ = self.conn.execute_batch("DROP TABLE IF EXISTS temp.attachment_actual_vocab;DROP TABLE IF EXISTS temp.attachment_expected_vocab;DROP TABLE IF EXISTS temp.attachment_expected_fts;");
    }
}

// Fast path exacto con memoria constante: secuencias idénticas implican el mismo
// multiset. No asumimos orden público de fts5vocab: cualquier diferencia de
// orden usa después la comprobación completa por conjuntos, no un falso error.
pub(crate) fn same_posting_sequence(c: &Connection, b: &Budget<'_>) -> Result<bool> {
    let mut a = c.prepare("SELECT term,doc,col,offset FROM temp.attachment_actual_vocab")?;
    let mut e = c.prepare("SELECT term,doc,col,offset FROM temp.attachment_expected_vocab")?;
    let mut a = a.query([])?;
    let mut e = e.query([])?;
    let mut checked = 0u32;
    loop {
        match (a.next()?, e.next()?) {
            (None, None) => return Ok(true),
            (Some(a), Some(e)) => {
                for n in 0..4 {
                    if a.get_ref(n)? != e.get_ref(n)? {
                        return Ok(false);
                    }
                }
                checked += 1;
                if checked.is_multiple_of(1024) {
                    b.check()?;
                }
            }
            _ => return Ok(false),
        }
    }
}
pub(crate) fn same_postings(c: &Connection, b: &Budget<'_>) -> Result<bool> {
    if same_posting_sequence(c, b)? {
        return Ok(true);
    }
    // E es único (FTS unicode61 canónico): E ⊆ A y |E|=|A| implican A=E,
    // incluso si A tuviese duplicados. No basta comparar sólo conteos.
    let different: bool = c.query_row("SELECT ((SELECT count(*) FROM temp.attachment_actual_vocab)!=(SELECT count(*) FROM temp.attachment_expected_vocab)) OR EXISTS(SELECT term,doc,col,offset FROM temp.attachment_expected_vocab EXCEPT SELECT term,doc,col,offset FROM temp.attachment_actual_vocab)", [], |r|r.get(0))?;
    Ok(!different)
}

// Fast path exacto para copias repetidas, no un digest ni una muestra. Unicode61
// depende sólo del texto y del perfil fijo; UNINDEXED no añade postings. Textos
// byte a byte iguales comparten el mismo mapa completo posición -> término.
// Cada fila real se verifica contra ese mapa y marca su (rowid,posición) propio.
// A ⊆ E, ausencia de duplicados en A y |A|=|E| prueban A=E sin asumir orden.
// El bitset separa documentos repetidos: un duplicado no tapa un faltante.
pub(crate) fn same_repeated_postings(
    c: &Connection,
    b: &Budget<'_>,
    fragments: &[(u32, &str)],
) -> Result<Option<bool>> {
    let mut classes = HashMap::new();
    let mut texts = Vec::new();
    let mut bytes = 0usize;
    let mut unique_bytes = 0usize;
    let mut max_rowid = 0u32;
    for &(rowid, text) in fragments {
        b.check()?;
        bytes += text.len();
        max_rowid = max_rowid.max(rowid);
        if let std::collections::hash_map::Entry::Vacant(e) = classes.entry(text) {
            e.insert(texts.len());
            texts.push(text);
            unique_bytes += text.len();
        }
    }
    // No duplicar tokenización/memoria cuando no hay suficiente reutilización.
    // Esto elige algoritmo, no limita ni acepta parcialmente el índice.
    if fragments.is_empty() || texts.len() == fragments.len() || unique_bytes * 4 > bytes {
        return Ok(None);
    }
    if max_rowid > MAX_INDEX_ENTRIES * 512 || bytes as u64 > MAX_INDEX_TEXT_BYTES {
        return Err(error("invalid_attachment_index"));
    }
    struct Scratch<'a>(&'a Connection);
    impl Drop for Scratch<'_> {
        fn drop(&mut self) {
            let _ = self.0.execute_batch("DROP TABLE IF EXISTS temp.attachment_repeated_vocab;DROP TABLE IF EXISTS temp.attachment_repeated_fts;");
        }
    }
    let _scratch = Scratch(c);
    c.execute_batch("CREATE VIRTUAL TABLE temp.attachment_repeated_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);CREATE VIRTUAL TABLE temp.attachment_repeated_vocab USING fts5vocab('temp','attachment_repeated_fts','instance');")?;
    let mut insert =
        c.prepare("INSERT INTO temp.attachment_repeated_fts(rowid,text) VALUES(?1,?2)")?;
    for (class, text) in texts.iter().enumerate() {
        b.check()?;
        insert.execute(params![class as u32 + 1, text])?;
    }
    drop(insert);
    let mut terms: HashMap<Vec<u8>, u32> = HashMap::new();
    let mut positions = vec![Vec::<Option<u32>>::new(); texts.len()];
    let mut counts = vec![0u32; texts.len()];
    let mut stmt = c.prepare("SELECT term,doc,col,offset FROM temp.attachment_repeated_vocab")?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        b.check()?;
        let rusqlite::types::ValueRef::Text(term) = r.get_ref(0)? else {
            return Err(error("invalid_attachment_index"));
        };
        let class = r
            .get::<_, u32>(1)?
            .checked_sub(1)
            .ok_or_else(|| error("invalid_attachment_index"))? as usize;
        let offset = r.get::<_, u32>(3)? as usize;
        if class >= texts.len()
            || offset >= texts[class].len()
            || r.get_ref(2)? != rusqlite::types::ValueRef::Text(b"text")
        {
            return Err(error("invalid_attachment_index"));
        }
        let next = terms.len() as u32;
        let term = *terms.entry(term.to_vec()).or_insert(next);
        let size = positions[class].len().max(offset + 1);
        positions[class].resize(size, None);
        if positions[class][offset].replace(term).is_some() {
            return Err(error("invalid_attachment_index"));
        }
        counts[class] += 1;
    }
    drop(rows);
    drop(stmt);
    let mut documents = vec![None; max_rowid as usize + 1];
    let mut bit_count = 0usize;
    let mut expected_count = 0u32;
    for &(rowid, text) in fragments {
        let class = classes[text];
        if rowid == 0
            || documents[rowid as usize]
                .replace((class, bit_count))
                .is_some()
        {
            return Err(error("invalid_attachment_index"));
        }
        bit_count += positions[class].len();
        expected_count += counts[class];
    }
    let mut seen = vec![0u64; bit_count.div_ceil(64)];
    if let Some(result) = crate::attachment_postings::verify(
        c,
        b,
        &terms,
        &positions,
        &documents,
        &mut seen,
        expected_count,
    )? {
        return Ok(Some(result));
    }
    // Full instance-stream fallback if the auxiliary API is unavailable.
    seen.fill(0);
    crate::attachment_postings::verify_stream(
        c,
        b,
        &terms,
        &positions,
        &documents,
        &mut seen,
        expected_count,
    )
    .map(Some)
}
pub(crate) fn same_copies(c: &Connection, b: &Budget<'_>) -> Result<bool> {
    // rowid es único en cada FTS. ORDER BY explícito permite comparar todas las
    // filas/celdas (incluidos sus tipos) sin construir dos árboles EXCEPT de texto.
    let mut a = c.prepare("SELECT rowid,job_id,generation_id,ordinal,text FROM main.extraction_search_fts ORDER BY rowid")?;
    let mut e = c.prepare("SELECT rowid,job_id,generation_id,ordinal,text FROM temp.attachment_expected_fts ORDER BY rowid")?;
    let mut a = a.query([])?;
    let mut e = e.query([])?;
    let mut checked = 0u32;
    loop {
        match (a.next()?, e.next()?) {
            (None, None) => return Ok(true),
            (Some(a), Some(e)) => {
                for n in 0..5 {
                    if a.get_ref(n)? != e.get_ref(n)? {
                        return Ok(false);
                    }
                }
                checked += 1;
                if checked.is_multiple_of(32) {
                    b.check()?;
                }
            }
            _ => return Ok(false),
        }
    }
}
pub(crate) fn verify<'a>(c: &'a Connection, b: &Budget<'_>) -> Result<Verified<'a>> {
    let revision = control(c)?.revision;
    structure(c)?;
    let result = (|| {
        let count: u32 =
            c.query_row("SELECT count(*) FROM extraction_search_entries", [], |r| {
                r.get(0)
            })?;
        let (fragments,bytes):(u32,u32) = c.query_row("SELECT count(*),coalesce(sum(length(CAST(text AS BLOB))),0) FROM extraction_search_fts",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if count > MAX_INDEX_ENTRIES
            || fragments > MAX_INDEX_FRAGMENTS
            || u64::from(bytes) > MAX_INDEX_TEXT_BYTES
        {
            return Err(error("invalid_attachment_index"));
        }
        let raw:Vec<(u32,String,String,u32,u32,String,String)> = c.prepare("SELECT slot,job_id,generation_id,fragment_count,text_bytes,content_hash,indexed_at FROM extraction_search_entries ORDER BY job_id,generation_id,slot")?
            .query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?)))?.collect::<std::result::Result<_,_>>()?;
        c.execute_batch("DROP TABLE IF EXISTS temp.attachment_actual_vocab;DROP TABLE IF EXISTS temp.attachment_expected_vocab;DROP TABLE IF EXISTS temp.attachment_expected_fts;CREATE VIRTUAL TABLE temp.attachment_expected_fts USING fts5(job_id UNINDEXED,generation_id UNINDEXED,ordinal UNINDEXED,text,tokenize='unicode61',detail=full);")?;
        let mut verified = Verified {
            conn: c,
            entries: Vec::with_capacity(raw.len()),
        };
        let mut insert = c.prepare("INSERT INTO temp.attachment_expected_fts(rowid,job_id,generation_id,ordinal,text) VALUES (?1,?2,?3,?4,?5)")?;
        for (slot, job, generation, count, bytes, hash, at) in raw {
            b.check()?;
            if !(1..=MAX_INDEX_ENTRIES).contains(&slot)
                || DateTime::parse_from_rfc3339(&at).is_err()
            {
                return Err(error("invalid_attachment_index"));
            }
            let data = canonical(c, &job, b)?;
            if data.generation.id != generation
                || data.fragments.len() as u32 != count
                || data.text_bytes != bytes
                || data.hash != hash
            {
                return Err(error("invalid_attachment_index"));
            }
            for f in &data.fragments {
                b.check()?;
                insert.execute(params![
                    (slot - 1) * 512 + f.ordinal + 1,
                    job,
                    generation,
                    f.ordinal,
                    f.text
                ])?;
            }
            let entry = data.entry(at);
            verified.entries.push(Selected { slot, data, entry });
        }
        drop(insert);
        if !same_copies(c, b)? {
            return Err(error("invalid_attachment_index"));
        }
        c.execute_batch("CREATE VIRTUAL TABLE temp.attachment_actual_vocab USING fts5vocab('main','extraction_search_fts','instance');CREATE VIRTUAL TABLE temp.attachment_expected_vocab USING fts5vocab('temp','attachment_expected_fts','instance');")?;
        let fragments = verified
            .entries
            .iter()
            .flat_map(|s| {
                s.data
                    .fragments
                    .iter()
                    .map(move |f| ((s.slot - 1) * 512 + f.ordinal + 1, f.text.as_str()))
            })
            .collect::<Vec<_>>();
        if same_repeated_postings(c, b, &fragments)? != Some(true) && !same_postings(c, b)? {
            return Err(error("invalid_attachment_index"));
        }
        b.check()?;
        Ok(verified)
    })();
    result.map_err(|e| {
        let e = b.map(e);
        if matches!(e.code, ErrorCode::WorkspaceBusy) {
            return e;
        }
        let mut e = error("invalid_attachment_index");
        e.details["current_revision"] = json!(revision);
        e
    })
}

pub(crate) fn integrity(c: &Connection) -> Result<()> {
    let b = Budget::new(c)?;
    verify(c, &b).map(|_| ()).map_err(|e| b.map(e))
}

pub(crate) fn coverage(
    c: &Connection,
    f: &IndexFilter,
    selected: &[Selected],
    sources: IndexSourceValidation,
    b: &Budget<'_>,
) -> Result<AttachmentIndexCoverage> {
    let total: u32 = c.query_row("SELECT count(*) FROM extraction_jobs", [], |r| r.get(0))?;
    if total > MAX_EXTRACTION_JOBS {
        return Err(error("invalid_attachment_index"));
    }
    let indexed: HashSet<_> = selected
        .iter()
        .map(|e| e.entry.source.job_id.as_str())
        .collect();
    let mut out = AttachmentIndexCoverage {
        scope: IndexCoverageScope::SelectedGenerations,
        index_validation: IndexValidation::CanonicalTextAndPostings,
        source_validation: sources,
        indexed_generations: 0,
        unindexed_supported_generations: 0,
        unsupported_generations: 0,
        jobs_without_generation: 0,
        fragments: 0,
        text_bytes: 0,
    };
    let extractor = f.extractor.map(IndexedExtractor::as_str);
    let mut stmt=c.prepare("SELECT j.id,j.generation_id,g.extractor,g.extractor_version,g.config FROM extraction_jobs j LEFT JOIN extraction_generations g ON g.id=j.generation_id WHERE (?1 IS NULL OR j.id=?1) AND (?2 IS NULL OR j.attachment_id=?2) AND (?3 IS NULL OR j.extractor=?3)")?;
    let mut rows = stmt.query(params![f.job_id, f.attachment_id, extractor])?;
    while let Some(r) = rows.next()? {
        b.check()?;
        let id: String = r.get(0)?;
        let generation: Option<String> = r.get(1)?;
        if generation.is_none() {
            out.jobs_without_generation += 1;
        } else if indexed.contains(id.as_str()) {
            out.indexed_generations += 1;
        } else {
            let raw: String = r.get(4)?;
            if raw.len() > 8192 {
                return Err(error("invalid_generation"));
            }
            let extractor: String = r.get(2)?;
            if matches!(
                extractor.as_str(),
                "utf8_text" | "pdf_text" | "xlsx_cells" | "image_ocr"
            ) && store::supported(&extractor, r.get(3)?, &serde_json::from_str(&raw)?)
            {
                out.unindexed_supported_generations += 1;
            } else {
                out.unsupported_generations += 1;
            }
        }
    }
    for s in selected
        .iter()
        .filter(|s| matches_filter(&s.data.source, f))
    {
        out.fragments += s.entry.fragment_count;
        out.text_bytes += u64::from(s.entry.text_bytes);
    }
    Ok(out)
}
pub(crate) fn matches_filter(s: &IndexedSource, f: &IndexFilter) -> bool {
    f.job_id.as_ref().is_none_or(|id| id == &s.job_id)
        && f.attachment_id
            .as_ref()
            .is_none_or(|id| id == &s.attachment_id)
        && f.extractor.is_none_or(|x| x.as_str() == s.extractor)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cursor {
    version: u32,
    scope: String,
    operation: String,
    query_hash: String,
    filter: IndexFilter,
    limit: u32,
    profile: String,
    revision: u32,
    offset: u32,
}
impl Cursor {
    pub fn new(
        c: &Connection,
        root: &Path,
        op: &str,
        query: &str,
        filter: IndexFilter,
        limit: u32,
        revision: u32,
    ) -> Result<Self> {
        Ok(Self {
            version: 1,
            scope: store::scope(c, root)?,
            operation: op.into(),
            query_hash: batch_storage::json_hash(
                "noteka.extraction.search.query.v1",
                &json!(query),
            )?,
            filter,
            limit,
            profile: ATTACHMENT_INDEX_PROFILE.into(),
            revision,
            offset: 0,
        })
    }
    pub fn offset(&self, raw: Option<&str>, max: u32) -> Result<u32> {
        let Some(raw) = raw else { return Ok(0) };
        if raw.len() > 2048 {
            return Err(store::invalid("cursor_size_limit"));
        }
        let c: Self = serde_json::from_str(raw).map_err(|_| store::invalid("invalid_cursor"))?;
        if c.version != 1 {
            return Err(store::invalid("unsupported_cursor_version"));
        }
        if c.revision == 0 {
            return Err(store::invalid("invalid_cursor"));
        }
        let mut shape = c.clone();
        shape.offset = 0;
        shape.revision = self.revision;
        if shape != *self || c.offset == 0 || c.offset >= max {
            return Err(store::invalid("cursor_scope_mismatch"));
        }
        if c.revision != self.revision {
            return Err(conflict("index_revision_changed", self.revision));
        }
        Ok(c.offset)
    }
    pub fn encode(mut self, offset: u32) -> Result<String> {
        self.offset = offset;
        let raw = serde_json::to_string(&self)?;
        if raw.len() > 2048 {
            return Err(store::invalid("cursor_size_limit"));
        }
        Ok(raw)
    }
}
