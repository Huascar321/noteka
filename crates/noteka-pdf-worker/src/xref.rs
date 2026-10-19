//! Segunda lectura independiente de xref/trailers sobre el original.
//! Audita las revisiones originales antes de construir una vista de carga con
//! precedencia explícita. No reconstruye offsets ni busca objetos perdidos.
use crate::{
    framing::{self, Scan},
    parser::{Pdf, Result, invalid},
    streams,
};
use lopdf::{Dictionary, Object, Stream};
use noteka_pdf_protocol::WorkerError;
use std::collections::{BTreeMap, HashSet};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Entry {
    Free,
    Normal { offset: u32, generation: u16 },
    Compressed { container: u32, index: u16 },
}
struct Section {
    entries: BTreeMap<u32, Entry>,
    trailer: Dictionary,
    end: usize,
    stream: bool,
}

/// Valida el índice original antes de cargar objetos. La vista de carga sólo
/// añade un índice efectivo sin Prev: no reconstruye offsets ni recupera datos
/// inválidos. Evita que el loader pierda tombstones/suplementos ya validados.
pub fn indexed_source(source: &[u8]) -> Result<(Vec<u8>, u32)> {
    let (start, tail) = tail(source)?;
    let empty = lopdf::Document::new();
    let pdf = Pdf { doc: &empty };
    let mut seen = HashSet::new();
    let mut merged = BTreeMap::new();
    let mut next = Some(start);
    let mut latest = true;
    let mut latest_size = 0;
    let mut root = None;
    let mut revisions = Vec::new();
    let mut sections = Vec::new();
    let linearized = linearization(source)?;
    let mut terminal = false;
    while let Some(offset) = next {
        if !seen.insert(offset) {
            return Err(WorkerError::InvalidPdf);
        }
        let mut current = section(&pdf, source, offset)?;
        let size = size(&current.trailer)?;
        terminal |= current.end <= tail
            && source[current.end..tail]
                .iter()
                .all(|&b| framing::whitespace(b));
        if latest {
            latest_size = size;
            if linearized != Some(offset)
                && (current.end > tail
                    || source[current.end..tail]
                        .iter()
                        .any(|&b| !framing::whitespace(b)))
            {
                return Err(WorkerError::InvalidPdf);
            }
            root = Some(
                current
                    .trailer
                    .get(b"Root")
                    .map_err(invalid)?
                    .as_reference()
                    .map_err(invalid)?,
            );
            latest = false;
        }
        next = optional_offset(&current.trailer, b"Prev", source.len())?;
        let (suffix, suffix_end) = suffix(source, current.end)?;
        if suffix != offset
            && !(linearized.is_some()
                && ((linearized == Some(offset) && suffix == 0)
                    || (next.is_none() && linearized == Some(suffix))))
        {
            return Err(WorkerError::InvalidPdf);
        }
        if next.is_some_and(|n| n >= offset) && linearized != Some(offset) {
            return Err(WorkerError::InvalidPdf);
        }
        sections.push(offset..suffix_end);
        if let Some(supplement) = optional_offset(&current.trailer, b"XRefStm", source.len())? {
            if current.stream || !seen.insert(supplement) {
                return Err(WorkerError::InvalidPdf);
            }
            let supplemental = section(&pdf, source, supplement)?;
            sections.push(supplement..supplemental.end);
            if !supplemental.stream
                || size != self::size(&supplemental.trailer)?
                || supplemental.trailer.has(b"XRefStm")
            {
                return Err(WorkerError::InvalidPdf);
            }
            // ISO no define inequívocamente Prev de un suplemento hybrid; no
            // elegir una interpretación silenciosa para esa variante ambigua.
            if supplemental.trailer.has(b"Prev") {
                let prev = optional_offset(&supplemental.trailer, b"Prev", source.len())?
                    .ok_or(WorkerError::InvalidPdf)?;
                section(&pdf, source, prev)?;
                return Err(WorkerError::PdfUnsupportedFeature);
            }
            // En hybrid, el stream reemplaza entradas de su tabla, incluso free.
            current.entries.extend(supplemental.entries);
        }
        revisions.push(current.entries.clone());
        for (id, entry) in current.entries {
            if id >= latest_size {
                return Err(WorkerError::InvalidPdf);
            }
            merged.entry(id).or_insert(entry); // la revisión más reciente prevalece
        }
    }
    if !terminal {
        return Err(WorkerError::InvalidPdf);
    }
    // Todas las entradas normales de cada revisión, incluso reemplazadas/free,
    // deben ser objetos completos y no estar dentro de otros cuerpos o índices.
    let mut effective = BTreeMap::new();
    let mut ranges = BTreeMap::new();
    for revision in revisions.iter().rev() {
        effective.extend(revision.clone());
        for (&id, entry) in revision {
            if let Entry::Normal { offset, generation } = entry {
                let range =
                    framing::original_object(source, *offset as usize, (id, *generation), |raw| {
                        original_length(source, &effective, raw)
                    })?;
                if sections
                    .iter()
                    .any(|s| s.start != range.start && s.start < range.end && range.start < s.end)
                {
                    return Err(WorkerError::InvalidPdf);
                }
                ranges.insert(range.start, range.end);
            }
        }
    }
    let object_ranges = ranges.iter().map(|(&a, &b)| (a, b)).collect::<Vec<_>>();
    if object_ranges.windows(2).any(|p| p[0].1 > p[1].0) {
        return Err(WorkerError::InvalidPdf);
    }
    for span in sections {
        ranges
            .entry(span.start)
            .and_modify(|end| *end = (*end).max(span.end))
            .or_insert(span.end);
    }
    let ranges = ranges.into_iter().collect::<Vec<_>>();
    if ranges.windows(2).any(|p| p[0].1 > p[1].0) {
        return Err(WorkerError::InvalidPdf);
    }
    // Comprobar que los offsets no comiencen dentro de comments ni de objetos
    // huérfanos omitidos del índice. No se incorporan éstos al índice efectivo.
    let mut cursor = 0;
    for &(start, end) in &ranges {
        loop {
            let mut scan = Scan {
                bytes: source,
                pos: cursor,
            };
            scan.skip();
            if scan.pos == start {
                break;
            }
            if scan.pos > start {
                return Err(WorkerError::InvalidPdf);
            }
            let at = scan.pos;
            let id = u32::try_from(unsigned(scan.word()?)?).map_err(invalid)?;
            let generation = u16::try_from(unsigned(scan.word()?)?).map_err(invalid)?;
            let orphan = framing::original_object(source, at, (id, generation), |raw| {
                original_length(source, &merged, raw)
            })?;
            if orphan.end > start {
                return Err(WorkerError::InvalidPdf);
            }
            cursor = orphan.end;
        }
        cursor = end;
    }
    let mut ending = Scan {
        bytes: source,
        pos: cursor,
    };
    ending.skip();
    if ending.pos != source.len() {
        return Err(WorkerError::InvalidPdf);
    }
    let root = root.ok_or(WorkerError::InvalidPdf)?;
    let mut entries = Vec::new();
    let mut indices = String::new();
    use std::fmt::Write;
    for (&id, entry) in &merged {
        let fields = match entry {
            Entry::Free => continue,
            Entry::Normal { offset, generation } => {
                if (*offset as usize) >= source.len() {
                    return Err(WorkerError::InvalidPdf);
                }
                (1, *offset, *generation)
            }
            Entry::Compressed { container, index } => (2, *container, *index),
        };
        write!(&mut indices, "{id} 1 ").map_err(invalid)?;
        entries.push(fields.0);
        entries.extend_from_slice(&fields.1.to_be_bytes());
        entries.extend_from_slice(&fields.2.to_be_bytes());
        if entries.len() > streams::LOAD_STREAM_BYTES {
            return Err(WorkerError::PdfStreamLimit);
        }
    }
    let mut indexed = source.to_vec();
    let offset = indexed.len() + 1;
    write!(&mut indices, "{latest_size} 1 ").map_err(invalid)?;
    entries.push(1);
    entries.extend_from_slice(&(offset as u32).to_be_bytes());
    entries.extend_from_slice(&0u16.to_be_bytes());
    indexed.extend_from_slice(format!("\n{latest_size} 0 obj\n<< /Type /XRef /Size {} /Root {} {} R /W [1 4 2] /Index [{indices}] /Length {} >>\nstream\n", latest_size.checked_add(1).ok_or(WorkerError::InvalidPdf)?, root.0, root.1, entries.len()).as_bytes());
    indexed.extend(entries);
    indexed
        .extend_from_slice(format!("\nendstream\nendobj\nstartxref\n{offset}\n%%EOF\n").as_bytes());
    Ok((indexed, latest_size))
}

fn original_length(source: &[u8], entries: &BTreeMap<u32, Entry>, raw: &[u8]) -> Result<usize> {
    let object = parameter(raw, 0)?;
    let integer = match object {
        Object::Integer(n) => n,
        Object::Reference((id, generation)) => {
            let Some(Entry::Normal {
                offset,
                generation: active,
            }) = entries.get(&id)
            else {
                return Err(WorkerError::InvalidPdf);
            };
            if *active != generation {
                return Err(WorkerError::InvalidPdf);
            }
            let mut scan = Scan {
                bytes: source,
                pos: *offset as usize,
            };
            if unsigned(scan.word()?)? != id as usize
                || unsigned(scan.word()?)? != generation as usize
            {
                return Err(WorkerError::InvalidPdf);
            }
            scan.expect(b"obj")?;
            let n = unsigned(scan.word()?)?;
            scan.expect(b"endobj")?;
            return Ok(n);
        }
        _ => return Err(WorkerError::InvalidPdf),
    };
    usize::try_from(integer).map_err(invalid)
}

fn linearization(source: &[u8]) -> Result<Option<usize>> {
    let mut scan = Scan {
        bytes: source,
        pos: 0,
    };
    scan.skip();
    scan.word()?;
    scan.word()?;
    scan.expect(b"obj")?;
    scan.skip();
    if source.get(scan.pos..scan.pos + 2) != Some(b"<<") {
        return Ok(None);
    }
    let fields = scan.fields()?;
    let Some((_, range)) = fields.iter().find(|(key, _)| key == b"Linearized") else {
        return Ok(None);
    };
    let value = std::str::from_utf8(&source[range.clone()])
        .map_err(invalid)?
        .parse::<f64>()
        .map_err(invalid)?;
    if value != 1.0 {
        return Err(WorkerError::InvalidPdf);
    }
    // Las hints son datos no utilizados: la lectura completa no hace range IO.
    // El framing especial sólo se permite con el diccionario inicial explícito.
    scan.expect(b"endobj")?;
    scan.skip();
    Ok(Some(scan.pos))
}

fn suffix(source: &[u8], end: usize) -> Result<(usize, usize)> {
    let mut pos = end;
    while source.get(pos).is_some_and(|&b| framing::whitespace(b)) {
        pos += 1;
    }
    if source.get(pos..pos + 9) != Some(b"startxref") {
        return Err(WorkerError::InvalidPdf);
    }
    let start = pos;
    pos += 9;
    while source.get(pos).is_some_and(|&b| framing::whitespace(b)) {
        pos += 1;
    }
    let number = pos;
    while source.get(pos).is_some_and(u8::is_ascii_digit) {
        pos += 1;
    }
    let offset = unsigned(&source[number..pos])?;
    let number_end = pos;
    while source.get(pos).is_some_and(|&b| framing::whitespace(b)) {
        pos += 1;
    }
    if source.get(pos..pos + 5) != Some(b"%%EOF")
        || !source[start + 9..number]
            .iter()
            .any(|b| b"\r\n".contains(b))
        || !source[number_end..pos].iter().any(|b| b"\r\n".contains(b))
        || source
            .get(pos + 5)
            .is_some_and(|&b| !framing::whitespace(b))
    {
        return Err(WorkerError::InvalidPdf);
    }
    Ok((offset, pos + 5))
}

fn tail(source: &[u8]) -> Result<(usize, usize)> {
    let trim = |bytes: &[u8]| {
        bytes
            .iter()
            .rposition(|&b| !framing::whitespace(b))
            .map_or(0, |n| n + 1)
    };
    let end = trim(source);
    let before_eof = source
        .get(..end.saturating_sub(5))
        .ok_or(WorkerError::InvalidPdf)?;
    if source.get(end.saturating_sub(5)..end) != Some(b"%%EOF") {
        return Err(WorkerError::InvalidPdf);
    }
    let number_end = trim(before_eof);
    if !before_eof[number_end..].iter().any(|b| b"\r\n".contains(b)) {
        return Err(WorkerError::InvalidPdf);
    }
    let number_start = before_eof[..number_end]
        .iter()
        .rposition(|&b| framing::whitespace(b))
        .map_or(0, |n| n + 1);
    let offset = unsigned(&before_eof[number_start..number_end])?;
    let keyword_end = trim(&before_eof[..number_start]);
    let keyword_start = keyword_end.checked_sub(9).ok_or(WorkerError::InvalidPdf)?;
    let line_start = source[..keyword_start]
        .iter()
        .rposition(|b| b"\r\n".contains(b))
        .map_or(0, |n| n + 1);
    if &source[keyword_start..keyword_end] != b"startxref"
        || keyword_start == 0
        || !framing::whitespace(source[keyword_start - 1])
        || source[line_start..keyword_start]
            .iter()
            .any(|&b| !framing::whitespace(b))
        || !source[keyword_end..number_start]
            .iter()
            .any(|b| b"\r\n".contains(b))
        || offset >= source.len()
    {
        return Err(WorkerError::InvalidPdf);
    }
    Ok((offset, keyword_start))
}

fn unsigned(bytes: &[u8]) -> Result<usize> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return Err(WorkerError::InvalidPdf);
    }
    std::str::from_utf8(bytes)
        .map_err(invalid)?
        .parse()
        .map_err(invalid)
}
fn size(dict: &Dictionary) -> Result<u32> {
    let size = dict
        .get(b"Size")
        .map_err(invalid)?
        .as_i64()
        .map_err(invalid)?;
    let size = u32::try_from(size).map_err(invalid)?;
    if size == 0 {
        return Err(WorkerError::InvalidPdf);
    }
    Ok(size)
}
fn optional_offset(dict: &Dictionary, key: &[u8], len: usize) -> Result<Option<usize>> {
    if !dict.has(key) {
        return Ok(None);
    }
    let offset = usize::try_from(dict.get(key).map_err(invalid)?.as_i64().map_err(invalid)?)
        .map_err(invalid)?;
    if offset >= len {
        return Err(WorkerError::InvalidPdf);
    }
    Ok(Some(offset))
}

fn section(pdf: &Pdf<'_>, source: &[u8], offset: usize) -> Result<Section> {
    let mut scan = Scan {
        bytes: source,
        pos: offset,
    };
    // El offset debe apuntar al token, no a whitespace que el parser corrija.
    if source.get(offset).is_none_or(|&b| framing::whitespace(b)) {
        return Err(WorkerError::InvalidPdf);
    }
    let first = scan.word()?;
    if first == b"xref" {
        table(scan)
    } else {
        xref_stream(pdf, scan, first)
    }
}

fn table(mut scan: Scan<'_>) -> Result<Section> {
    eol(&mut scan, true)?;
    let mut entries = BTreeMap::new();
    loop {
        let first = scan.word()?;
        if first == b"trailer" {
            break;
        }
        let start = u32::try_from(unsigned(first)?).map_err(invalid)?;
        let count = unsigned(scan.word()?)?;
        // Comprobar antes de iterar/asignar desde el count no confiable.
        if count == 0
            || count > scan.bytes.len().saturating_sub(scan.pos) / 20
            || u32::try_from(count)
                .ok()
                .and_then(|n| start.checked_add(n))
                .is_none()
        {
            return Err(WorkerError::InvalidPdf);
        }
        eol(&mut scan, true)?;
        for index in 0..count {
            let row = scan
                .bytes
                .get(scan.pos..scan.pos + 20)
                .ok_or(WorkerError::InvalidPdf)?;
            if row[10] != b' '
                || row[16] != b' '
                || ![b" \n".as_slice(), b" \r", b"\r\n"].contains(&&row[18..])
            {
                return Err(WorkerError::InvalidPdf);
            }
            let offset = u32::try_from(unsigned(&row[..10])?).map_err(invalid)?;
            let generation = u16::try_from(unsigned(&row[11..16])?).map_err(invalid)?;
            let entry = match row[17] {
                b'n' => Entry::Normal { offset, generation },
                b'f' => Entry::Free,
                _ => return Err(WorkerError::InvalidPdf),
            };
            let id = start + index as u32;
            if (id == 0 && !matches!(entry, Entry::Free)) || entries.insert(id, entry).is_some() {
                return Err(WorkerError::InvalidPdf);
            }
            scan.pos += 20;
        }
    }
    if entries.is_empty() {
        return Err(WorkerError::InvalidPdf);
    }
    let trailer = dictionary(&mut scan)?;
    let size = size(&trailer)?;
    if entries.keys().any(|&id| id >= size) {
        return Err(WorkerError::InvalidPdf);
    }
    Ok(Section {
        entries,
        trailer,
        end: scan.pos,
        stream: false,
    })
}

fn eol(scan: &mut Scan<'_>, optional_space: bool) -> Result<()> {
    if optional_space && scan.bytes.get(scan.pos) == Some(&b' ') {
        scan.pos += 1;
    }
    match scan.bytes.get(scan.pos) {
        Some(b'\r') => {
            scan.pos += 1;
            if scan.bytes.get(scan.pos) == Some(&b'\n') {
                scan.pos += 1;
            }
        }
        Some(b'\n') => scan.pos += 1,
        _ => return Err(WorkerError::InvalidPdf),
    }
    Ok(())
}

fn dictionary(scan: &mut Scan<'_>) -> Result<Dictionary> {
    let mut dictionary = Dictionary::new();
    for (key, range) in scan.fields()? {
        if key == b"Encrypt" {
            return Err(WorkerError::PdfEncrypted);
        }
        if [
            b"Size".as_slice(),
            b"Prev",
            b"XRefStm",
            b"Root",
            b"Type",
            b"Length",
            b"Filter",
            b"DecodeParms",
            b"W",
            b"Index",
            b"F",
            b"FFilter",
            b"FDecodeParms",
        ]
        .contains(&key.as_slice())
        {
            let value = parameter(&scan.bytes[range], 0)?;
            if [b"Size".as_slice(), b"Prev", b"XRefStm", b"Length"].contains(&key.as_slice())
                && !matches!(value, Object::Integer(_))
            {
                return Err(WorkerError::InvalidPdf);
            }
            dictionary.set(key, value);
        }
    }
    Ok(dictionary)
}

fn parameter(bytes: &[u8], depth: usize) -> Result<Object> {
    if depth > 64 {
        return Err(WorkerError::InvalidPdf);
    }
    let mut scan = Scan { bytes, pos: 0 };
    scan.skip();
    match bytes.get(scan.pos) {
        Some(b'/') => Ok(Object::Name(scan.name()?)),
        Some(b'<') if bytes.get(scan.pos + 1) == Some(&b'<') => {
            let mut dict = Dictionary::new();
            for (key, range) in scan.fields()? {
                dict.set(key, parameter(&bytes[range], depth + 1)?);
            }
            Ok(Object::Dictionary(dict))
        }
        Some(b'[') => {
            scan.pos += 1;
            let mut array = Vec::new();
            loop {
                scan.skip();
                if bytes.get(scan.pos) == Some(&b']') {
                    break;
                }
                let start = scan.pos;
                scan.value(0)?;
                array.push(parameter(&bytes[start..scan.pos], depth + 1)?);
            }
            Ok(Object::Array(array))
        }
        _ => {
            let word = scan.word()?;
            if word == b"null" {
                return Ok(Object::Null);
            }
            let first = std::str::from_utf8(word)
                .map_err(invalid)?
                .parse::<i64>()
                .map_err(invalid)?;
            scan.skip();
            if scan.pos == bytes.len() {
                return Ok(Object::Integer(first));
            }
            let generation = u16::try_from(unsigned(scan.word()?)?).map_err(invalid)?;
            scan.expect(b"R")?;
            Ok(Object::Reference((
                u32::try_from(first).map_err(invalid)?,
                generation,
            )))
        }
    }
}

fn xref_stream(pdf: &Pdf<'_>, mut scan: Scan<'_>, first: &[u8]) -> Result<Section> {
    let id = u32::try_from(unsigned(first)?).map_err(invalid)?;
    let generation = u16::try_from(unsigned(scan.word()?)?).map_err(invalid)?;
    if id == 0 || generation != 0 {
        return Err(WorkerError::InvalidPdf);
    }
    scan.expect(b"obj")?;
    let dict = dictionary(&mut scan)?;
    if dict
        .get(b"Type")
        .map_err(invalid)?
        .as_name()
        .map_err(invalid)?
        != b"XRef"
    {
        return Err(WorkerError::InvalidPdf);
    }
    let length = usize::try_from(
        dict.get(b"Length")
            .map_err(invalid)?
            .as_i64()
            .map_err(invalid)?,
    )
    .map_err(invalid)?;
    let mut body_scan = scan;
    body_scan.expect(b"stream")?;
    eol(&mut body_scan, false)?;
    let end = body_scan
        .pos
        .checked_add(length)
        .ok_or(WorkerError::InvalidPdf)?;
    let content = scan
        .bytes
        .get(body_scan.pos..end)
        .ok_or(WorkerError::InvalidPdf)?
        .to_vec();
    let stream = Stream::new(dict.clone(), content);
    framing::stream_body(&mut scan, &stream)?;
    scan.expect(b"endobj")?;
    let bytes = streams::decode(pdf, &stream, streams::LOAD_STREAM_BYTES)?;
    let entries = stream_entries(&dict, &bytes)?;
    Ok(Section {
        entries,
        trailer: dict,
        end: scan.pos,
        stream: true,
    })
}

pub fn validate_loaded(pdf: &Pdf<'_>, stream: &Stream) -> Result<()> {
    let bytes = streams::decode(pdf, stream, streams::LOAD_STREAM_BYTES)?;
    stream_entries(&stream.dict, &bytes).map(|_| ())
}

fn stream_entries(dict: &Dictionary, bytes: &[u8]) -> Result<BTreeMap<u32, Entry>> {
    let integers = |key: &[u8]| -> Result<Vec<usize>> {
        dict.get(key)
            .map_err(invalid)?
            .as_array()
            .map_err(invalid)?
            .iter()
            .map(|o| usize::try_from(o.as_i64().map_err(invalid)?).map_err(invalid))
            .collect()
    };
    let size = size(dict)?;
    let widths = integers(b"W")?;
    if widths.len() != 3 || widths.iter().any(|&w| w > 8) {
        return Err(WorkerError::InvalidPdf);
    }
    let width: usize = widths.iter().sum();
    if width == 0 {
        return Err(WorkerError::InvalidPdf);
    }
    let indices = if dict.has(b"Index") {
        integers(b"Index")?
    } else {
        vec![0, size as usize]
    };
    if indices.is_empty() || indices.len() % 2 != 0 {
        return Err(WorkerError::InvalidPdf);
    }
    let mut count = 0usize;
    let mut previous_end = 0;
    for pair in indices.as_chunks::<2>().0 {
        let end = pair[0]
            .checked_add(pair[1])
            .ok_or(WorkerError::InvalidPdf)?;
        if pair[0] < previous_end || end > size as usize {
            return Err(WorkerError::InvalidPdf);
        }
        previous_end = end;
        count = count.checked_add(pair[1]).ok_or(WorkerError::InvalidPdf)?;
    }
    if count.checked_mul(width) != Some(bytes.len()) {
        return Err(WorkerError::InvalidPdf);
    }
    let mut rows = bytes.chunks_exact(width);
    let mut entries = BTreeMap::new();
    for pair in indices.as_chunks::<2>().0 {
        for id in pair[0]..pair[0] + pair[1] {
            let row = rows.next().ok_or(WorkerError::InvalidPdf)?;
            let mut fields = [0u64; 3];
            let mut start = 0;
            for (index, &width) in widths.iter().enumerate() {
                for &byte in &row[start..start + width] {
                    fields[index] = (fields[index] << 8) | u64::from(byte);
                }
                start += width;
            }
            if widths[0] == 0 {
                fields[0] = 1;
            }
            let entry = match fields[0] {
                0 => Entry::Free,
                1 => Entry::Normal {
                    offset: u32::try_from(fields[1]).map_err(invalid)?,
                    generation: u16::try_from(fields[2]).map_err(invalid)?,
                },
                2 => Entry::Compressed {
                    container: u32::try_from(fields[1]).map_err(invalid)?,
                    index: u16::try_from(fields[2]).map_err(invalid)?,
                },
                _ => return Err(WorkerError::PdfUnsupportedFeature),
            };
            if (id == 0 && !matches!(entry, Entry::Free))
                || entries.insert(id as u32, entry).is_some()
            {
                return Err(WorkerError::InvalidPdf);
            }
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tail_is_exact_bounded_and_not_a_comment() {
        assert_eq!(tail(b"%PDF-1.7\nxref\nstartxref\n9\n%%EOF\n"), Ok((9, 14)));
        for bad in [
            b"startxref\n0\n%%EOF".as_slice(),
            b"x\nstartxref\n-1\n%%EOF",
            b"x\nstartxref\n999\n%%EOF",
            b"x\nstartxref\n1 %%EOF",
            b"x\nstartxref\n1\n%%EOF junk",
            b"x\nstartxref\n1%comment\n%%EOF",
            b"x\n%comment startxref\n1\n%%EOF",
        ] {
            assert_eq!(tail(bad), Err(WorkerError::InvalidPdf));
        }
    }

    #[test]
    fn prev_cycle_is_rejected_even_if_library_silently_accepts_it() {
        let mut bytes = b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_vec();
        let page_offset = bytes.len();
        bytes.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Count 0 /Kids [] >>\nendobj\n");
        let xref_offset = bytes.len();
        bytes.extend_from_slice(format!(
            "xref\n0 3\n0000000000 65535 f \n0000000009 00000 n \n{page_offset:010} 00000 n \ntrailer\n<< /Size 3 /Root 1 0 R /Prev {xref_offset} >>\nstartxref\n{xref_offset}\n%%EOF\n"
        ).as_bytes());
        let original = bytes.clone();
        let doc = lopdf::Document::load_mem_with_options(
            &bytes,
            lopdf::LoadOptions {
                strict: true,
                max_decompressed_size: Some(streams::LOAD_STREAM_BYTES),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!doc.trailer.has(b"Prev"));
        assert_eq!(
            indexed_source(&bytes).map(|_| ()),
            Err(WorkerError::InvalidPdf)
        );
        assert_eq!(bytes, original);
    }
    #[test]
    fn metadata_types_and_duplicate_escaped_keys_are_not_coerced() {
        for bad in [
            b"<< /Size 1 /#53ize 2 >>".as_slice(),
            b"<< /Size 1.0 >>",
            b"<< /Prev /Invalid >>",
        ] {
            let mut scan = Scan { bytes: bad, pos: 0 };
            assert!(dictionary(&mut scan).is_err());
        }
        let mut scan = Scan {
            bytes: b"<< /Size 3 /Root 1 0 R /ID [<ABC> <DEF>] /Unknown (preserved in original) >>",
            pos: 0,
        };
        let dict = dictionary(&mut scan).unwrap();
        assert_eq!(size(&dict), Ok(3));
        assert_eq!(dict.get(b"Root").unwrap(), &Object::Reference((1, 0)));
    }
}
