//! Adapter privado del helper descartable; nunca se enlaza al core/host.
//! Carga/árbol/recursos y texto se validan sin ejecutar acciones del documento.
use crate::{diagnostics, framing, streams, text, xref};
use lopdf::{
    Dictionary, Document, LoadOptions, Object, ObjectId, ObjectStream, Stream, xref::XrefEntry,
};
use noteka_pdf_protocol::{MAX_PAGES, MAX_SOURCE_BYTES, MAX_TEXT_BYTES, PageText, WorkerError};
use std::collections::{HashMap, HashSet};

pub(crate) type Result<T> = std::result::Result<T, WorkerError>;
pub(crate) fn invalid<T>(_: T) -> WorkerError {
    WorkerError::InvalidPdf
}
pub(crate) fn unsupported<T>() -> Result<T> {
    Err(WorkerError::PdfUnsupportedFeature)
}

pub(crate) struct Pdf<'a> {
    pub doc: &'a Document,
}
impl Pdf<'_> {
    pub fn resolve<'a>(&'a self, object: &'a Object) -> Result<&'a Object> {
        let mut object = object;
        let mut seen = HashSet::new();
        while let Object::Reference(id) = object {
            if seen.len() >= 64 || !seen.insert(*id) {
                return Err(WorkerError::InvalidPdf);
            }
            object = self.object(*id)?;
        }
        Ok(object)
    }
    pub fn required<'a>(&'a self, dict: &'a Dictionary, key: &[u8]) -> Result<&'a Object> {
        self.resolve(dict.get(key).map_err(invalid)?)
    }
    pub fn optional<'a>(&'a self, dict: &'a Dictionary, key: &[u8]) -> Result<Option<&'a Object>> {
        if dict.has(key) {
            self.required(dict, key).map(Some)
        } else {
            Ok(None)
        }
    }
    fn object(&self, id: ObjectId) -> Result<&Object> {
        // No aliases para nodos de árbol: se conserva identidad y Parent exacto.
        // lopdf también inserta miembros ObjStm ausentes del xref: presencia en
        // doc.objects no demuestra que una referencia esté viva ni su generación.
        match self.doc.reference_table.entries.get(&id.0) {
            Some(XrefEntry::Normal { generation, .. }) if *generation == id.1 => {}
            Some(XrefEntry::Compressed { .. }) if id.1 == 0 => {}
            _ => return Err(WorkerError::InvalidPdf),
        }
        self.doc.objects.get(&id).ok_or(WorkerError::InvalidPdf)
    }
    fn structure(&self, object: &Object, depth: usize) -> Result<()> {
        if depth > 64 {
            return unsupported();
        }
        match object {
            Object::Reference(_) => {
                self.resolve(object)?;
            }
            Object::Real(n) if !n.is_finite() => return Err(WorkerError::InvalidPdf),
            Object::Array(array) => {
                for object in array {
                    self.structure(object, depth + 1)?;
                }
            }
            Object::Dictionary(dict) => self.dictionary(dict, depth)?,
            Object::Stream(stream) => {
                self.dictionary(&stream.dict, depth)?;
                let length = self
                    .required(&stream.dict, b"Length")?
                    .as_i64()
                    .map_err(invalid)?;
                if usize::try_from(length).ok() != Some(stream.content.len()) {
                    return Err(WorkerError::InvalidPdf);
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn dictionary(&self, dict: &Dictionary, depth: usize) -> Result<()> {
        if dict.has(b"ActualText") || dict.has(b"Alt") {
            return unsupported();
        }
        for (_, value) in dict {
            self.structure(value, depth + 1)?;
        }
        Ok(())
    }
    fn contents(&self, page: &Dictionary) -> Result<Vec<u8>> {
        let Some(contents) = self.optional(page, b"Contents")? else {
            return Ok(Vec::new());
        };
        let objects: Vec<&Object> = match contents {
            Object::Stream(_) => vec![contents],
            Object::Array(array) => array
                .iter()
                .map(|o| self.resolve(o))
                .collect::<Result<_>>()?,
            _ => return Err(WorkerError::InvalidPdf),
        };
        let mut output = Vec::new();
        for object in objects {
            let stream = object.as_stream().map_err(invalid)?;
            let available = streams::PAGE_STREAM_BYTES.saturating_sub(output.len());
            let decoded = streams::decode(self, stream, available)?;
            output.extend_from_slice(&decoded);
        }
        if output.len() > streams::PAGE_STREAM_BYTES {
            return Err(WorkerError::PdfStreamLimit);
        }
        Ok(output)
    }
    fn annotations(&self, page: &Dictionary) -> Result<()> {
        if let Some(annotations) = self.optional(page, b"Annots")? {
            for annotation in annotations.as_array().map_err(invalid)? {
                let dict = self.resolve(annotation)?.as_dict().map_err(invalid)?;
                // Un enlace sin texto/appearance es dato pasivo. Resto excluido.
                if self
                    .required(dict, b"Subtype")?
                    .as_name()
                    .map_err(invalid)?
                    != b"Link"
                    || [b"Contents".as_slice(), b"RC", b"AP", b"AS"]
                        .iter()
                        .any(|key| dict.has(key))
                {
                    return unsupported();
                }
            }
        }
        Ok(())
    }
    fn pages<'a>(
        &'a self,
        id: ObjectId,
        parent: Option<ObjectId>,
        inherited: Option<&'a Dictionary>,
        seen: &mut HashSet<ObjectId>,
        pages: &mut Vec<(&'a Dictionary, Option<&'a Dictionary>)>,
        depth: usize,
    ) -> Result<usize> {
        if depth > 64 {
            return unsupported();
        }
        if !seen.insert(id) {
            return Err(WorkerError::InvalidPdf);
        }
        let dict = self.object(id)?.as_dict().map_err(invalid)?;
        match (parent, dict.get(b"Parent")) {
            (None, Err(_)) => {}
            (Some(expected), Ok(Object::Reference(actual))) if expected == *actual => {}
            _ => return Err(WorkerError::InvalidPdf),
        }
        let resources = match self.optional(dict, b"Resources")? {
            Some(resources) => Some(resources.as_dict().map_err(invalid)?),
            None => inherited,
        };
        match self.required(dict, b"Type")?.as_name().map_err(invalid)? {
            b"Page" if parent.is_some() => {
                if dict.has(b"Kids") || dict.has(b"Count") {
                    return Err(WorkerError::InvalidPdf);
                }
                if pages.len() >= MAX_PAGES {
                    return Err(WorkerError::PdfPageLimit);
                }
                self.annotations(dict)?;
                pages.push((dict, resources));
                Ok(1)
            }
            b"Pages" => {
                let count = self.required(dict, b"Count")?.as_i64().map_err(invalid)?;
                if count < 0 {
                    return Err(WorkerError::InvalidPdf);
                }
                if count > MAX_PAGES as i64 {
                    return Err(WorkerError::PdfPageLimit);
                }
                let kids = self.required(dict, b"Kids")?.as_array().map_err(invalid)?;
                let mut actual = 0usize;
                for kid in kids {
                    actual += self.pages(
                        kid.as_reference().map_err(invalid)?,
                        Some(id),
                        resources,
                        seen,
                        pages,
                        depth + 1,
                    )?;
                }
                if actual != count as usize {
                    return Err(WorkerError::InvalidPdf);
                }
                Ok(actual)
            }
            _ => Err(WorkerError::InvalidPdf),
        }
    }
}

fn load_error(error: lopdf::Error) -> WorkerError {
    match error {
        lopdf::Error::Decompress(lopdf::DecompressError::MemoryLimitExceeded { .. }) => {
            WorkerError::PdfStreamLimit
        }
        lopdf::Error::Decryption(_) | lopdf::Error::InvalidPassword => WorkerError::PdfEncrypted,
        _ => WorkerError::InvalidPdf,
    }
}

/// No contiene IO de archivos, red, SQL, shell ni ejecución de acciones PDF.
/// No sustituye el gate de corpus ni la validación/publicación futura del núcleo.
pub fn extract(source: &[u8]) -> Result<Vec<PageText>> {
    if source.len() > MAX_SOURCE_BYTES
        || source.len() < 9
        || &source[..7] != b"%PDF-1."
        || !(b'0'..=b'7').contains(&source[7])
        || !matches!(source[8], b'\r' | b'\n')
        || !source.trim_ascii_end().ends_with(b"%%EOF")
    {
        return Err(WorkerError::InvalidPdf);
    }
    if !diagnostics::begin() {
        return Err(WorkerError::PdfWorkerFailed);
    }
    let (indexed, index_id) = xref::indexed_source(source)?;
    let mut doc = Document::load_mem_with_options(
        &indexed,
        LoadOptions {
            strict: true,
            password: None,
            max_decompressed_size: Some(streams::LOAD_STREAM_BYTES),
            ..LoadOptions::default()
        },
    )
    .map_err(load_error)?;
    doc.reference_table.entries.remove(&index_id);
    doc.objects.remove(&(index_id, 0));
    if doc.was_encrypted() || doc.is_encrypted() || doc.trailer.has(b"Encrypt") {
        return Err(WorkerError::PdfEncrypted);
    }
    if diagnostics::degraded() {
        return Err(WorkerError::InvalidPdf);
    }
    let pdf = Pdf { doc: &doc };
    framing::document(&doc, source)?;
    // Segunda revisión de bytes originales: la carga de ObjStm/XRef de lopdf
    // tiene defaults/filter_map silenciosos que un logger no puede certificar.
    loading_streams(&pdf)?;
    for object in doc.objects.values() {
        pdf.structure(object, 0)?;
    }
    let catalog = pdf
        .required(&doc.trailer, b"Root")?
        .as_dict()
        .map_err(invalid)?;
    if pdf.required(catalog, b"Type")?.as_name().map_err(invalid)? != b"Catalog" {
        return Err(WorkerError::InvalidPdf);
    }
    if catalog.has(b"AcroForm") || catalog.has(b"Threads") {
        return unsupported();
    }
    let root = catalog
        .get(b"Pages")
        .map_err(invalid)?
        .as_reference()
        .map_err(invalid)?;
    let mut pages = Vec::new();
    pdf.pages(root, None, None, &mut HashSet::new(), &mut pages, 0)?;
    if pages.is_empty() {
        return Err(WorkerError::PdfPageLimit);
    }
    let mut output = Vec::new();
    let mut total = 0usize;
    let empty_resources = Dictionary::new();
    for (page, resources) in pages {
        let bytes = pdf.contents(page)?;
        let text = text::extract(
            &pdf,
            resources.unwrap_or(&empty_resources),
            &bytes,
            MAX_TEXT_BYTES - total,
        )?;
        total += text.len();
        output.push(PageText {
            page_number: output.len() as u32 + 1,
            text,
        });
    }
    if diagnostics::degraded() {
        return Err(WorkerError::PdfDecodeError);
    }
    Ok(output)
}

fn loading_streams(pdf: &Pdf<'_>) -> Result<()> {
    let mut headers = HashMap::new();
    for (&id, object) in &pdf.doc.objects {
        if let Object::Stream(stream) = object {
            match pdf.optional(&stream.dict, b"Type")? {
                Some(Object::Name(name)) if name == b"ObjStm" => {
                    let bytes = streams::decode(pdf, stream, streams::LOAD_STREAM_BYTES)?;
                    let first = usize::try_from(
                        pdf.required(&stream.dict, b"First")?
                            .as_i64()
                            .map_err(invalid)?,
                    )
                    .map_err(invalid)?;
                    let header = object_stream_header(pdf, stream, &bytes, first)?;
                    for (index, &(_, start)) in header.iter().enumerate() {
                        let end = header
                            .get(index + 1)
                            .map_or(bytes.len(), |&(_, offset)| offset);
                        framing::direct(&bytes[start..end])?;
                    }
                    // Miembros no activos pueden haber sido reemplazados/free.
                    // Se valida TODO su framing/AST crudo; sólo una entrada viva
                    // permite referenciarlos o comparar el AST que usará el texto.
                    // La carga inicial usó el decoder de la biblioteca. Volver a
                    // parsear sólo los bytes decodificados estrictamente y exigir
                    // igualdad del AST activo; un checksum correcto no certifica
                    // que ambos decoders hayan interpretado lo mismo.
                    let mut raw_dict = stream.dict.clone();
                    raw_dict.remove(b"Filter");
                    raw_dict.remove(b"DecodeParms");
                    raw_dict.set("First", first as i64);
                    raw_dict.set("N", header.len() as i64);
                    let raw = Stream::new(raw_dict, bytes);
                    let parsed =
                        ObjectStream::new_with_limit(&raw, Some(streams::LOAD_STREAM_BYTES))
                            .map_err(load_error)?;
                    if parsed.objects.len() != header.len() {
                        return Err(WorkerError::InvalidPdf);
                    }
                    for (index, &(member, _)) in header.iter().enumerate() {
                        if matches!(pdf.doc.reference_table.entries.get(&member),
                            Some(XrefEntry::Compressed { container, index: active_index })
                            if *container == id.0 && usize::from(*active_index) == index)
                            && parsed.objects.get(&(member, 0)) != pdf.doc.objects.get(&(member, 0))
                        {
                            return Err(WorkerError::PdfUnsupportedFeature);
                        }
                    }
                    headers.insert(id, header);
                }
                Some(Object::Name(name)) if name == b"XRef" => xref::validate_loaded(pdf, stream)?,
                _ => {}
            }
        }
    }
    for (&id, entry) in &pdf.doc.reference_table.entries {
        match entry {
            lopdf::xref::XrefEntry::Normal { generation, .. } => {
                if !pdf.doc.objects.contains_key(&(id, *generation)) {
                    return Err(WorkerError::InvalidPdf);
                }
            }
            lopdf::xref::XrefEntry::Compressed { container, index } => {
                if !pdf.doc.objects.contains_key(&(id, 0)) {
                    return Err(WorkerError::InvalidPdf);
                }
                let header = headers
                    .get(&(*container, 0))
                    .ok_or(WorkerError::InvalidPdf)?;
                if header.get(*index as usize).map(|pair| pair.0) != Some(id) {
                    return Err(WorkerError::InvalidPdf);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn object_stream_header(
    pdf: &Pdf<'_>,
    stream: &lopdf::Stream,
    bytes: &[u8],
    first: usize,
) -> Result<Vec<(u32, usize)>> {
    let header =
        std::str::from_utf8(bytes.get(..first).ok_or(WorkerError::InvalidPdf)?).map_err(invalid)?;
    let values = header
        .split_ascii_whitespace()
        .map(|n| n.parse::<u32>().map_err(invalid))
        .collect::<Result<Vec<_>>>()?;
    let n = usize::try_from(
        pdf.required(&stream.dict, b"N")?
            .as_i64()
            .map_err(invalid)?,
    )
    .map_err(invalid)?;
    if n.checked_mul(2) != Some(values.len()) {
        return Err(WorkerError::InvalidPdf);
    }
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for pair in values.as_chunks::<2>().0 {
        let offset = first
            .checked_add(pair[1] as usize)
            .ok_or(WorkerError::InvalidPdf)?;
        if pair[0] == 0
            || !seen.insert(pair[0])
            || offset >= bytes.len()
            || output
                .last()
                .is_some_and(|&(_, previous)| previous >= offset)
        {
            return Err(WorkerError::InvalidPdf);
        }
        output.push((pair[0], offset));
    }
    Ok(output)
}
