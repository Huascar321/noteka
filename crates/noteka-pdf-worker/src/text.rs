//! Orden de operaciones, no orden visual. No usa el extractor permisivo de lopdf.
use crate::parser::{Pdf, Result, invalid, unsupported};
use lopdf::{Dictionary, Encoding, Object, content::Content};
use noteka_pdf_protocol::{MAX_TEXT_BYTES, WorkerError};

#[derive(Clone, Copy)]
struct Font([Option<char>; 256]);

fn font(pdf: &Pdf<'_>, resources: &Dictionary, name: &[u8]) -> Result<Font> {
    let fonts = pdf
        .required(resources, b"Font")?
        .as_dict()
        .map_err(invalid)?;
    let dict = pdf.required(fonts, name)?.as_dict().map_err(invalid)?;
    if pdf.required(dict, b"Type")?.as_name().map_err(invalid)? != b"Font"
        || pdf.required(dict, b"Subtype")?.as_name().map_err(invalid)? != b"Type1"
        || dict.has(b"ToUnicode")
    {
        return unsupported();
    }
    let base = pdf
        .required(dict, b"BaseFont")?
        .as_name()
        .map_err(invalid)?;
    if ![
        b"Courier".as_slice(),
        b"Courier-Bold",
        b"Courier-Oblique",
        b"Courier-BoldOblique",
        b"Helvetica",
        b"Helvetica-Bold",
        b"Helvetica-Oblique",
        b"Helvetica-BoldOblique",
        b"Times-Roman",
        b"Times-Bold",
        b"Times-Italic",
        b"Times-BoldItalic",
    ]
    .contains(&base)
    {
        return unsupported();
    }
    if let Some(descriptor) = pdf.optional(dict, b"FontDescriptor")? {
        let descriptor = descriptor.as_dict().map_err(invalid)?;
        if [b"FontFile".as_slice(), b"FontFile2", b"FontFile3"]
            .iter()
            .any(|key| descriptor.has(key))
        {
            return unsupported();
        }
    }
    if let Some(encoding) = pdf.optional(dict, b"Encoding")?
        && !matches!(encoding, Object::Name(n) if n == b"StandardEncoding" || n == b"WinAnsiEncoding")
    {
        return unsupported();
    }
    // Sólo se llega a esta API después de comprobar las alternativas que harían
    // fallback. Acceso al mapping tipado, no bytes_to_string que omite glyphs.
    let Encoding::OneByteEncoding(mapping) = dict
        .get_font_encoding_with_limit(pdf.doc, 1024 * 1024)
        .map_err(invalid)?
    else {
        return unsupported();
    };
    let mut chars = [None; 256];
    for (index, glyph) in mapping.iter().enumerate() {
        chars[index] = glyph
            .and_then(|glyph| char::from_u32(u32::from(glyph.utf16_code_unit())))
            .filter(|&ch| ch != '\0' && ch != '\u{fffd}');
    }
    Ok(Font(chars))
}

struct Text {
    output: String,
    cap: usize,
}
impl Text {
    fn push(&mut self, ch: char) -> Result<()> {
        if self
            .output
            .len()
            .checked_add(ch.len_utf8())
            .is_none_or(|n| n > self.cap)
        {
            return Err(WorkerError::PdfTextLimit);
        }
        self.output.push(ch);
        Ok(())
    }
    fn newline(&mut self) -> Result<()> {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.push('\n')?;
        }
        Ok(())
    }
    fn show(&mut self, selected: Option<Font>, object: &Object) -> Result<()> {
        let selected = selected.ok_or(WorkerError::InvalidPdf)?;
        for byte in object.as_str().map_err(invalid)? {
            self.push(selected.0[*byte as usize].ok_or(WorkerError::PdfDecodeError)?)?;
        }
        Ok(())
    }
}

fn numbers(operands: &[Object], count: usize) -> Result<()> {
    if operands.len() != count
        || operands
            .iter()
            .any(|o| !matches!(o, Object::Integer(_) | Object::Real(_)))
    {
        return Err(WorkerError::InvalidPdf);
    }
    if operands
        .iter()
        .any(|o| matches!(o, Object::Real(n) if !n.is_finite()))
    {
        return Err(WorkerError::InvalidPdf);
    }
    Ok(())
}
fn arity(operands: &[Object], count: usize) -> Result<()> {
    if operands.len() != count {
        return Err(WorkerError::InvalidPdf);
    }
    Ok(())
}
fn name(object: &Object) -> Result<&[u8]> {
    object.as_name().map_err(invalid)
}

pub fn extract(pdf: &Pdf<'_>, resources: &Dictionary, bytes: &[u8], cap: usize) -> Result<String> {
    crate::framing::content(bytes).map_err(|_| WorkerError::PdfDecodeError)?;
    let content = Content::decode_strict(bytes).map_err(|_| WorkerError::PdfDecodeError)?;
    let mut text = Text {
        output: String::new(),
        cap: cap.min(MAX_TEXT_BYTES),
    };
    let mut in_text = false;
    let mut text_start = 0;
    let mut selected = None;
    let mut stack = Vec::new();
    let mut marked = 0usize;
    for op in content.operations {
        let args = &op.operands;
        let textual = matches!(
            op.operator.as_str(),
            "Tf" | "Tj"
                | "TJ"
                | "'"
                | "\""
                | "Tc"
                | "Tw"
                | "Tz"
                | "TL"
                | "Tr"
                | "Ts"
                | "Td"
                | "TD"
                | "Tm"
                | "T*"
        );
        if textual && !in_text {
            return Err(WorkerError::InvalidPdf);
        }
        match op.operator.as_str() {
            "BT" => {
                arity(args, 0)?;
                if in_text {
                    return Err(WorkerError::InvalidPdf);
                }
                in_text = true;
                text_start = text.output.len();
            }
            "ET" => {
                arity(args, 0)?;
                if !in_text {
                    return Err(WorkerError::InvalidPdf);
                }
                in_text = false;
                if text.output.len() > text_start {
                    text.newline()?;
                }
            }
            "Tf" => {
                arity(args, 2)?;
                numbers(&args[1..], 1)?;
                selected = Some(font(pdf, resources, name(&args[0])?)?);
            }
            "Tj" | "'" => {
                arity(args, 1)?;
                if op.operator == "'" {
                    text.newline()?;
                }
                text.show(selected, &args[0])?;
            }
            "\"" => {
                arity(args, 3)?;
                numbers(&args[..2], 2)?;
                text.newline()?;
                text.show(selected, &args[2])?;
            }
            "TJ" => {
                arity(args, 1)?;
                for item in args[0].as_array().map_err(invalid)? {
                    match item {
                        Object::String(..) => text.show(selected, item)?,
                        Object::Integer(_) | Object::Real(_) => {
                            numbers(std::slice::from_ref(item), 1)?
                        }
                        _ => return Err(WorkerError::InvalidPdf),
                    }
                }
            }
            "T*" => {
                arity(args, 0)?;
                text.newline()?;
            }
            "Td" | "TD" => {
                numbers(args, 2)?;
                if args[1].as_float().map_err(invalid)? != 0.0 {
                    text.newline()?;
                }
            }
            "Tm" | "cm" => numbers(args, 6)?,
            "Tc" | "Tw" | "Tz" | "TL" | "Ts" => numbers(args, 1)?,
            "Tr" => {
                arity(args, 1)?;
                if !(0..=7).contains(&args[0].as_i64().map_err(invalid)?) {
                    return Err(WorkerError::InvalidPdf);
                }
            }
            "q" => {
                arity(args, 0)?;
                if stack.len() >= 64 {
                    return unsupported();
                }
                stack.push(selected);
            }
            "Q" => {
                arity(args, 0)?;
                selected = stack.pop().ok_or(WorkerError::InvalidPdf)?;
            }
            "Do" => {
                arity(args, 1)?;
                let objects = pdf
                    .required(resources, b"XObject")?
                    .as_dict()
                    .map_err(invalid)?;
                let stream = pdf
                    .required(objects, name(&args[0])?)?
                    .as_stream()
                    .map_err(invalid)?;
                if pdf
                    .required(&stream.dict, b"Subtype")?
                    .as_name()
                    .map_err(invalid)?
                    != b"Image"
                {
                    return unsupported();
                }
                // No decode, render ni OCR del stream de imagen.
            }
            "gs" => {
                arity(args, 1)?;
                let states = pdf
                    .required(resources, b"ExtGState")?
                    .as_dict()
                    .map_err(invalid)?;
                let state = pdf
                    .required(states, name(&args[0])?)?
                    .as_dict()
                    .map_err(invalid)?;
                if state.has(b"Font") || state.has(b"SMask") {
                    return unsupported();
                }
            }
            "BMC" | "BDC" => {
                arity(args, if op.operator == "BMC" { 1 } else { 2 })?;
                name(&args[0])?;
                if op.operator == "BDC" {
                    let property = match &args[1] {
                        Object::Dictionary(dict) => dict,
                        Object::Name(key) => {
                            let properties = pdf
                                .required(resources, b"Properties")?
                                .as_dict()
                                .map_err(invalid)?;
                            pdf.required(properties, key)?.as_dict().map_err(invalid)?
                        }
                        _ => return Err(WorkerError::InvalidPdf),
                    };
                    if property.has(b"ActualText") || property.has(b"Alt") {
                        return unsupported();
                    }
                }
                marked += 1;
                if marked > 64 {
                    return unsupported();
                }
            }
            "EMC" => {
                arity(args, 0)?;
                marked = marked.checked_sub(1).ok_or(WorkerError::InvalidPdf)?;
            }
            "MP" => {
                arity(args, 1)?;
                name(&args[0])?;
            }
            "w" | "M" | "i" | "G" | "g" => numbers(args, 1)?,
            "J" | "j" => {
                arity(args, 1)?;
                if !(0..=2).contains(&args[0].as_i64().map_err(invalid)?) {
                    return Err(WorkerError::InvalidPdf);
                }
            }
            "m" | "l" => numbers(args, 2)?,
            "c" => numbers(args, 6)?,
            "v" | "y" | "re" | "K" | "k" => numbers(args, 4)?,
            "RG" | "rg" => numbers(args, 3)?,
            "h" | "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" | "n" | "W" | "W*" => {
                arity(args, 0)?
            }
            "ri" => {
                arity(args, 1)?;
                name(&args[0])?;
            }
            "d" => {
                arity(args, 2)?;
                let array = args[0].as_array().map_err(invalid)?;
                numbers(array, array.len())?;
                numbers(&args[1..], 1)?;
            }
            // Incluye BI, Form, BX/EX y operadores desconocidos: nunca omitirlos.
            _ => return unsupported(),
        }
    }
    if in_text || !stack.is_empty() || marked != 0 {
        return Err(WorkerError::InvalidPdf);
    }
    Ok(text.output)
}
