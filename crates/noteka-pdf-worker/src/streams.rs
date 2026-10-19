//! Decodificación sin fallback de checksum, truncado o filtros mal tipados.
use crate::parser::{Pdf, Result, invalid, unsupported};
use crate::{codecs, predictors};
use flate2::{Decompress, FlushDecompress, Status};
use lopdf::{Object, Stream};
use noteka_pdf_protocol::WorkerError;

pub const LOAD_STREAM_BYTES: usize = 1024 * 1024;
pub const PAGE_STREAM_BYTES: usize = 4 * 1024 * 1024;

pub fn decode(pdf: &Pdf<'_>, stream: &Stream, cap: usize) -> Result<Vec<u8>> {
    // Los streams externos no autorizan acceso al filesystem o a la red.
    if stream.dict.has(b"F") || stream.dict.has(b"FFilter") || stream.dict.has(b"FDecodeParms") {
        return unsupported();
    }
    let length = pdf
        .required(&stream.dict, b"Length")?
        .as_i64()
        .map_err(invalid)?;
    if usize::try_from(length).ok() != Some(stream.content.len()) {
        return Err(WorkerError::InvalidPdf);
    }
    let filters = match pdf.optional(&stream.dict, b"Filter")? {
        None => Vec::new(),
        Some(Object::Name(name)) => vec![name.as_slice()],
        Some(Object::Array(array)) => array
            .iter()
            .map(|item| item.as_name().map_err(invalid))
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(WorkerError::InvalidPdf),
    };
    let params = match pdf.optional(&stream.dict, b"DecodeParms")? {
        None | Some(Object::Null) => vec![None; filters.len()],
        Some(Object::Dictionary(dict)) if filters.len() == 1 => vec![Some(dict)],
        Some(Object::Array(array)) if array.len() == filters.len() => array
            .iter()
            .map(|item| match pdf.resolve(item)? {
                Object::Null => Ok(None),
                Object::Dictionary(dict) => Ok(Some(dict)),
                _ => Err(WorkerError::InvalidPdf),
            })
            .collect::<Result<Vec<_>>>()?,
        _ => return Err(WorkerError::InvalidPdf),
    };
    let mut bytes = stream.content.clone();
    for (filter, params) in filters.into_iter().zip(params) {
        let predicting = matches!(filter, b"FlateDecode" | b"LZWDecode");
        if let Some(params) = params {
            for (key, value) in params {
                let known = predicting
                    && [
                        b"Predictor".as_slice(),
                        b"Colors",
                        b"Columns",
                        b"BitsPerComponent",
                    ]
                    .contains(&key.as_slice())
                    || filter == b"LZWDecode" && key == b"EarlyChange";
                if !known {
                    return unsupported();
                }
                pdf.resolve(value)?.as_i64().map_err(invalid)?;
            }
        }
        bytes = match filter {
            b"FlateDecode" => inflate(&bytes, cap)?,
            b"ASCIIHexDecode" => codecs::ascii_hex(&bytes, cap)?,
            b"ASCII85Decode" => codecs::ascii85(&bytes, cap)?,
            b"RunLengthDecode" => codecs::run_length(&bytes, cap)?,
            b"LZWDecode" => codecs::lzw(
                &bytes,
                cap,
                predictors::integer(pdf, params, b"EarlyChange", 1)?,
            )?,
            // DCT/JPX/CCITT/JBIG2 son codecs de imagen; no convertirlos en texto.
            // Crypt/codificaciones ajenas tampoco habilitan un fallback raw.
            _ => return unsupported(),
        };
        if predicting {
            bytes = predictors::apply(pdf, params, bytes, cap)?;
        }
    }
    if bytes.len() > cap {
        return Err(WorkerError::PdfStreamLimit);
    }
    Ok(bytes)
}

fn inflate(input: &[u8], cap: usize) -> Result<Vec<u8>> {
    let mut decoder = Decompress::new(true);
    let mut output = Vec::new();
    loop {
        let mut buffer = [0; 8192];
        let before_in = decoder.total_in();
        let before_out = decoder.total_out();
        let status = decoder
            .decompress(
                &input[before_in as usize..],
                &mut buffer,
                FlushDecompress::None,
            )
            .map_err(|_| WorkerError::PdfDecodeError)?;
        let count = (decoder.total_out() - before_out) as usize;
        if output.len().checked_add(count).is_none_or(|len| len > cap) {
            return Err(WorkerError::PdfStreamLimit);
        }
        output.extend_from_slice(&buffer[..count]);
        if status == Status::StreamEnd {
            if decoder.total_in() as usize != input.len() {
                return Err(WorkerError::PdfDecodeError);
            }
            return Ok(output);
        }
        if before_in == decoder.total_in() && count == 0 {
            return Err(WorkerError::PdfDecodeError);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn flate_requires_end_checksum_exact_input_and_cap() {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&vec![b'x'; 20_000]).unwrap();
        let encoded = encoder.finish().unwrap();
        assert_eq!(inflate(&encoded, 20_000).unwrap(), vec![b'x'; 20_000]);
        assert_eq!(inflate(&encoded, 19_999), Err(WorkerError::PdfStreamLimit));
        for len in [0, 1, 2, encoded.len() - 1, encoded.len() - 4] {
            assert_eq!(
                inflate(&encoded[..len], 20_000),
                Err(WorkerError::PdfDecodeError)
            );
        }
        let mut corrupt = encoded.clone();
        *corrupt.last_mut().unwrap() ^= 1;
        assert_eq!(inflate(&corrupt, 20_000), Err(WorkerError::PdfDecodeError));
        let mut trailing = encoded;
        trailing.push(0);
        assert_eq!(inflate(&trailing, 20_000), Err(WorkerError::PdfDecodeError));
    }
}
