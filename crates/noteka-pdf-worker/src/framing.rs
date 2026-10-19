//! Preflight léxico sobre bytes originales. lopdf admite `endobj` ausente y
//! sobreescribe claves duplicadas sin diagnóstico: no basta con revisar el AST.
use crate::parser::{Result, invalid};
use lopdf::{Document, Object, ObjectId, Stream, xref::XrefEntry};
use noteka_pdf_protocol::WorkerError;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(crate) struct Scan<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
}
pub(crate) fn whitespace(byte: u8) -> bool {
    matches!(byte, 0 | 9 | 10 | 12 | 13 | 32)
}
fn delimiter(byte: u8) -> bool {
    whitespace(byte) || b"()<>[]{}/%".contains(&byte)
}
fn hex(byte: u8) -> Option<u8> {
    (byte as char).to_digit(16).map(|n| n as u8)
}

impl<'a> Scan<'a> {
    pub fn skip(&mut self) {
        loop {
            while self.bytes.get(self.pos).is_some_and(|&b| whitespace(b)) {
                self.pos += 1;
            }
            if self.bytes.get(self.pos) != Some(&b'%') {
                break;
            }
            while self
                .bytes
                .get(self.pos)
                .is_some_and(|b| !b"\r\n".contains(b))
            {
                self.pos += 1;
            }
        }
    }
    pub fn word(&mut self) -> Result<&'a [u8]> {
        self.skip();
        let start = self.pos;
        while self.bytes.get(self.pos).is_some_and(|&b| !delimiter(b)) {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(WorkerError::InvalidPdf);
        }
        Ok(&self.bytes[start..self.pos])
    }
    pub fn expect(&mut self, word: &[u8]) -> Result<()> {
        if self.word()? != word {
            return Err(WorkerError::InvalidPdf);
        }
        Ok(())
    }
    pub fn name(&mut self) -> Result<Vec<u8>> {
        self.skip();
        if self.bytes.get(self.pos) != Some(&b'/') {
            return Err(WorkerError::InvalidPdf);
        }
        self.pos += 1;
        let mut output = Vec::new();
        while let Some(&byte) = self.bytes.get(self.pos) {
            if delimiter(byte) {
                break;
            }
            self.pos += 1;
            if byte == b'#' {
                let high = self
                    .bytes
                    .get(self.pos)
                    .copied()
                    .and_then(hex)
                    .ok_or(WorkerError::InvalidPdf)?;
                let low = self
                    .bytes
                    .get(self.pos + 1)
                    .copied()
                    .and_then(hex)
                    .ok_or(WorkerError::InvalidPdf)?;
                self.pos += 2;
                output.push(high * 16 + low);
            } else {
                output.push(byte);
            }
        }
        Ok(output)
    }
    fn literal(&mut self) -> Result<()> {
        self.pos += 1;
        let mut nested = 1usize;
        while let Some(&byte) = self.bytes.get(self.pos) {
            self.pos += 1;
            match byte {
                b'\\' => {
                    // Se admite escape de cualquier carácter según PDF; el scanner
                    // sólo evita confundir paréntesis escapados con delimitadores.
                    if self.pos >= self.bytes.len() {
                        return Err(WorkerError::InvalidPdf);
                    }
                    if self.bytes[self.pos] == b'\r' && self.bytes.get(self.pos + 1) == Some(&b'\n')
                    {
                        self.pos += 1;
                    }
                    self.pos += 1;
                }
                b'(' => nested += 1,
                b')' => {
                    nested -= 1;
                    if nested == 0 {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        Err(WorkerError::InvalidPdf)
    }
    fn dictionary(&mut self, depth: usize) -> Result<()> {
        self.pos += 2;
        let mut keys = HashSet::new();
        loop {
            self.skip();
            if self.bytes.get(self.pos..self.pos + 2) == Some(b">>") {
                self.pos += 2;
                return Ok(());
            }
            if !keys.insert(self.name()?) {
                return Err(WorkerError::InvalidPdf);
            }
            self.value(depth + 1)?;
        }
    }
    pub fn value(&mut self, depth: usize) -> Result<()> {
        if depth > 64 {
            return Err(WorkerError::InvalidPdf);
        }
        self.skip();
        match self.bytes.get(self.pos).copied() {
            Some(b'/') => {
                self.name()?;
            }
            Some(b'(') => self.literal()?,
            Some(b'[') => {
                self.pos += 1;
                loop {
                    self.skip();
                    if self.bytes.get(self.pos) == Some(&b']') {
                        self.pos += 1;
                        break;
                    }
                    self.value(depth + 1)?;
                }
            }
            Some(b'<') if self.bytes.get(self.pos + 1) == Some(&b'<') => self.dictionary(depth)?,
            Some(b'<') => {
                self.pos += 1;
                loop {
                    let byte = *self.bytes.get(self.pos).ok_or(WorkerError::InvalidPdf)?;
                    self.pos += 1;
                    if byte == b'>' {
                        break;
                    }
                    if !whitespace(byte) && hex(byte).is_none() {
                        return Err(WorkerError::InvalidPdf);
                    }
                }
            }
            Some(_) => {
                let word = self.word()?;
                if [b"true".as_slice(), b"false", b"null"].contains(&word) {
                    return Ok(());
                }
                if !number(word) {
                    return Err(WorkerError::InvalidPdf);
                }
                // Un entero puede ser el primer token de una referencia indirecta.
                if word.iter().all(u8::is_ascii_digit) {
                    let mut lookahead = *self;
                    if let Ok(generation) = lookahead.word()
                        && generation.iter().all(u8::is_ascii_digit)
                        && lookahead.expect(b"R").is_ok()
                    {
                        *self = lookahead;
                    }
                }
            }
            None => return Err(WorkerError::InvalidPdf),
        }
        Ok(())
    }

    /// Rangos originales, no un AST que pueda borrar duplicados. Los valores
    /// completos (también los desconocidos) pasan por el scanner acotado.
    pub fn fields(&mut self) -> Result<Vec<(Vec<u8>, std::ops::Range<usize>)>> {
        self.skip();
        if self.bytes.get(self.pos..self.pos + 2) != Some(b"<<") {
            return Err(WorkerError::InvalidPdf);
        }
        self.pos += 2;
        let mut keys = HashSet::new();
        let mut fields = Vec::new();
        loop {
            self.skip();
            if self.bytes.get(self.pos..self.pos + 2) == Some(b">>") {
                self.pos += 2;
                return Ok(fields);
            }
            let key = self.name()?;
            if !keys.insert(key.clone()) {
                return Err(WorkerError::InvalidPdf);
            }
            self.skip();
            let start = self.pos;
            self.value(1)?;
            fields.push((key, start..self.pos));
        }
    }
}

fn number(word: &[u8]) -> bool {
    let word = if matches!(word.first(), Some(b'+' | b'-')) {
        &word[1..]
    } else {
        word
    };
    let mut digits = 0;
    let mut points = 0;
    for &byte in word {
        if byte.is_ascii_digit() {
            digits += 1;
        } else if byte == b'.' {
            points += 1;
        } else {
            return false;
        }
    }
    digits > 0 && points <= 1
}

pub fn document(doc: &Document, source: &[u8]) -> Result<()> {
    let mut ranges = Vec::new();
    for (&id, entry) in &doc.reference_table.entries {
        if let XrefEntry::Normal { offset, generation } = entry {
            let object = doc
                .objects
                .get(&(id, *generation))
                .ok_or(WorkerError::InvalidPdf)?;
            let start = *offset as usize;
            let length = indirect(source, start, (id, *generation), object)?;
            ranges.push(start..start + length);
        }
    }
    ranges.sort_unstable_by_key(|range| range.start);
    if ranges.windows(2).any(|pair| pair[0].end > pair[1].start) {
        // Ningún objeto activo puede comenzar dentro del string/stream/cuerpo
        // de otro. Tener un token "N 0 obj" allí no lo vuelve un objeto externo.
        return Err(WorkerError::InvalidPdf);
    }
    Ok(())
}

fn indirect(source: &[u8], offset: usize, id: ObjectId, object: &Object) -> Result<usize> {
    let mut scan = Scan {
        bytes: source.get(offset..).ok_or(WorkerError::InvalidPdf)?,
        pos: 0,
    };
    if scan.bytes.first().is_none_or(|byte| !byte.is_ascii_digit()) {
        return Err(WorkerError::InvalidPdf);
    }
    let parsed_id = std::str::from_utf8(scan.word()?)
        .map_err(invalid)?
        .parse::<u32>()
        .map_err(invalid)?;
    let generation = std::str::from_utf8(scan.word()?)
        .map_err(invalid)?
        .parse::<u16>()
        .map_err(invalid)?;
    if (parsed_id, generation) != id {
        return Err(WorkerError::InvalidPdf);
    }
    scan.expect(b"obj")?;
    scan.value(0)?;
    if let Object::Stream(stream) = object {
        stream_body(&mut scan, stream)?;
    }
    scan.expect(b"endobj")?;
    Ok(scan.pos)
}

/// Framing de un objeto de cualquier revisión, antes del loader. La longitud
/// indirecta se resuelve con el índice efectivo de ESA revisión, no la última.
pub(crate) fn original_object(
    source: &[u8],
    offset: usize,
    id: ObjectId,
    length: impl Fn(&[u8]) -> Result<usize>,
) -> Result<std::ops::Range<usize>> {
    let mut scan = Scan {
        bytes: source,
        pos: offset,
    };
    if source.get(offset).is_none_or(|b| !b.is_ascii_digit()) {
        return Err(WorkerError::InvalidPdf);
    }
    let number = |b: &[u8]| {
        std::str::from_utf8(b)
            .map_err(invalid)?
            .parse::<u32>()
            .map_err(invalid)
    };
    if (number(scan.word()?)?, number(scan.word()?)?) != (id.0, u32::from(id.1)) {
        return Err(WorkerError::InvalidPdf);
    }
    scan.expect(b"obj")?;
    scan.skip();
    let fields = if source.get(scan.pos..scan.pos + 2) == Some(b"<<") {
        Some(scan.fields()?)
    } else {
        scan.value(0)?;
        None
    };
    let mut look = scan;
    if look.word().ok() == Some(b"stream".as_slice()) {
        let fields = fields.ok_or(WorkerError::InvalidPdf)?;
        let raw = fields
            .iter()
            .find(|(key, _)| key == b"Length")
            .ok_or(WorkerError::InvalidPdf)?;
        let len = length(&source[raw.1.clone()])?;
        scan = look;
        if source.get(scan.pos) == Some(&b'\r') {
            scan.pos += 1;
            if source.get(scan.pos) == Some(&b'\n') {
                scan.pos += 1;
            }
        } else if source.get(scan.pos) == Some(&b'\n') {
            scan.pos += 1;
        } else {
            return Err(WorkerError::InvalidPdf);
        }
        scan.pos = scan
            .pos
            .checked_add(len)
            .filter(|&n| n <= source.len())
            .ok_or(WorkerError::InvalidPdf)?;
        scan.expect(b"endstream")?;
    }
    scan.expect(b"endobj")?;
    Ok(offset..scan.pos)
}

pub(crate) fn stream_body(scan: &mut Scan<'_>, stream: &Stream) -> Result<()> {
    scan.expect(b"stream")?;
    if scan.bytes.get(scan.pos) == Some(&b'\r') {
        scan.pos += 1;
        if scan.bytes.get(scan.pos) == Some(&b'\n') {
            scan.pos += 1;
        }
    } else if scan.bytes.get(scan.pos) == Some(&b'\n') {
        scan.pos += 1;
    } else {
        return Err(WorkerError::InvalidPdf);
    }
    let end = scan
        .pos
        .checked_add(stream.content.len())
        .ok_or(WorkerError::InvalidPdf)?;
    if scan.bytes.get(scan.pos..end) != Some(stream.content.as_slice()) {
        return Err(WorkerError::InvalidPdf);
    }
    scan.pos = end;
    scan.expect(b"endstream")
}

pub fn direct(bytes: &[u8]) -> Result<()> {
    let mut scan = Scan { bytes, pos: 0 };
    scan.value(0)?;
    scan.skip();
    if scan.pos != bytes.len() {
        return Err(WorkerError::InvalidPdf);
    }
    Ok(())
}

pub fn content(bytes: &[u8]) -> Result<()> {
    let mut scan = Scan { bytes, pos: 0 };
    loop {
        scan.skip();
        let Some(&first) = bytes.get(scan.pos) else {
            return Ok(());
        };
        if b"/([<+-.0123456789".contains(&first) {
            scan.value(0)?;
        } else {
            // Operadores/keywords se validan semánticamente en text. Aquí no
            // reinterpretar strings, comments o diccionarios como operadores.
            scan.word()?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_duplicates_strings_comments_and_reference_boundaries() {
        assert!(content(b"/Span << /MCID 0 >> BDC BT (literal endobj) Tj ET EMC").is_ok());
        assert_eq!(
            content(b"/Span << /MCID 0 /#4dCID 1 >> BDC"),
            Err(WorkerError::InvalidPdf)
        );
        for good in [
            b"<< /A (a\\(b\\) endobj) /B [1 0 R 1.2 +3 -4 null true <A>] >>".as_slice(),
            b"<< /A << /A 1 >> >>",
            b"(literal (nested) string)",
            b"[ /A /A ]",
            b"1 %comment\n",
        ] {
            assert!(direct(good).is_ok());
        }
        for bad in [
            b"<< /A 1 /A 2 >>".as_slice(),
            b"<< /A 1 /#41 2 >>",
            b"<< /A >>",
            b"(bad\\)",
            b"<AG>",
            b"[1.2.3]",
            b"[2e10]",
            b"null trailing",
            b"/bad#G1",
            b"[1 2R]",
        ] {
            assert_eq!(direct(bad), Err(WorkerError::InvalidPdf));
        }
    }

    #[test]
    fn indirect_requires_actual_marker_not_a_string_or_comment() {
        let object = Object::Integer(1);
        assert!(indirect(b"5 0 obj 1 endobj", 0, (5, 0), &object).is_ok());
        for bad in [
            b"5 0 obj 1".as_slice(),
            b"5 0 obj 1 %endobj",
            b"5 0 obj 1 endobject",
            b"6 0 obj 1 endobj",
        ] {
            assert_eq!(
                indirect(bad, 0, (5, 0), &object),
                Err(WorkerError::InvalidPdf)
            );
        }
    }
}
