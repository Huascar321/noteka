//! TIFF/PNG por filas, con aritmética comprobada y sin padding/filas perdidas.
use crate::parser::{Pdf, Result, invalid};
use lopdf::Dictionary;
use noteka_pdf_protocol::WorkerError;

pub fn integer(
    pdf: &Pdf<'_>,
    params: Option<&Dictionary>,
    key: &[u8],
    default: usize,
) -> Result<usize> {
    match params
        .map(|dict| pdf.optional(dict, key))
        .transpose()?
        .flatten()
    {
        None => Ok(default),
        Some(value) => usize::try_from(value.as_i64().map_err(invalid)?).map_err(invalid),
    }
}

pub fn apply(
    pdf: &Pdf<'_>,
    params: Option<&Dictionary>,
    data: Vec<u8>,
    cap: usize,
) -> Result<Vec<u8>> {
    let predictor = integer(pdf, params, b"Predictor", 1)?;
    if predictor == 1 {
        return Ok(data);
    }
    if predictor != 2 && !(10..=15).contains(&predictor) {
        return Err(WorkerError::PdfUnsupportedFeature);
    }
    let colors = integer(pdf, params, b"Colors", 1)?;
    let columns = integer(pdf, params, b"Columns", 1)?;
    let bits = integer(pdf, params, b"BitsPerComponent", 8)?;
    if colors == 0 || columns == 0 || ![1, 2, 4, 8, 16].contains(&bits) {
        return Err(WorkerError::InvalidPdf);
    }
    let samples = colors
        .checked_mul(columns)
        .ok_or(WorkerError::PdfStreamLimit)?;
    let row_bits = samples
        .checked_mul(bits)
        .ok_or(WorkerError::PdfStreamLimit)?;
    let row_bytes = row_bits.checked_add(7).ok_or(WorkerError::PdfStreamLimit)? / 8;
    if row_bytes > cap {
        return Err(WorkerError::PdfStreamLimit);
    }
    if predictor == 2 {
        return tiff(data, colors, samples, bits, row_bytes);
    }
    let pixel = colors
        .checked_mul(bits)
        .ok_or(WorkerError::PdfStreamLimit)?
        .div_ceil(8);
    // ISO 32000-1 §7.4.4.4: 10–15 sólo señalan PNG al decoder.
    // El tag de cada fila es autoritativo, no tiene que coincidir con Predictor.
    png(&data, row_bytes, pixel)
}

fn tiff(
    mut bytes: Vec<u8>,
    colors: usize,
    samples: usize,
    bits: usize,
    row_bytes: usize,
) -> Result<Vec<u8>> {
    if row_bytes == 0 || !bytes.len().is_multiple_of(row_bytes) {
        return Err(WorkerError::PdfDecodeError);
    }
    for row in bytes.chunks_exact_mut(row_bytes) {
        for sample in colors..samples {
            let mut value = 0u32;
            let mut left = 0u32;
            for bit in 0..bits {
                let a = sample * bits + bit;
                let b = (sample - colors) * bits + bit;
                value = (value << 1) | u32::from((row[a / 8] >> (7 - a % 8)) & 1);
                left = (left << 1) | u32::from((row[b / 8] >> (7 - b % 8)) & 1);
            }
            let value = (value + left) & ((1 << bits) - 1);
            for bit in 0..bits {
                let a = sample * bits + bit;
                let mask = 1 << (7 - a % 8);
                let set = (value >> (bits - bit - 1)) & 1 != 0;
                row[a / 8] = (row[a / 8] & !mask) | if set { mask } else { 0 };
            }
        }
    }
    Ok(bytes)
}

fn png(data: &[u8], columns: usize, pixel: usize) -> Result<Vec<u8>> {
    let row_bytes = columns.checked_add(1).ok_or(WorkerError::PdfStreamLimit)?;
    if columns == 0 || !data.len().is_multiple_of(row_bytes) {
        return Err(WorkerError::PdfDecodeError);
    }
    let mut output: Vec<u8> = Vec::new();
    for encoded in data.chunks_exact(row_bytes) {
        let filter = encoded[0];
        if filter > 4 {
            return Err(WorkerError::PdfDecodeError);
        }
        let start = output.len();
        for (index, &byte) in encoded[1..].iter().enumerate() {
            let left = if index >= pixel {
                output[start + index - pixel]
            } else {
                0
            };
            let up = if start >= columns {
                output[start - columns + index]
            } else {
                0
            };
            let upper_left = if start >= columns && index >= pixel {
                output[start - columns + index - pixel]
            } else {
                0
            };
            let prediction = match filter {
                0 => 0,
                1 => left,
                2 => up,
                3 => ((u16::from(left) + u16::from(up)) / 2) as u8,
                4 => paeth(left, up, upper_left),
                _ => unreachable!(),
            };
            output.push(byte.wrapping_add(prediction));
        }
    }
    Ok(output)
}
fn paeth(left: u8, up: u8, upper_left: u8) -> u8 {
    let p = i32::from(left) + i32::from(up) - i32::from(upper_left);
    let dl = (p - i32::from(left)).abs();
    let du = (p - i32::from(up)).abs();
    let dul = (p - i32::from(upper_left)).abs();
    if dl <= du && dl <= dul {
        left
    } else if du <= dul {
        up
    } else {
        upper_left
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tiff_packed_samples_color_stride_and_wrap_are_exact() {
        assert_eq!(tiff(vec![0b11011010], 1, 8, 1, 1).unwrap(), [0b10010011]);
        assert_eq!(tiff(vec![0x12, 0x34], 1, 4, 4, 2).unwrap(), [0x13, 0x6a]);
        assert_eq!(
            tiff(vec![255, 2, 3, 2, 255, 254], 3, 6, 8, 6).unwrap(),
            [255, 2, 3, 1, 1, 1]
        );
        assert_eq!(
            tiff(vec![255, 255, 0, 2], 1, 2, 16, 4).unwrap(),
            [255, 255, 0, 1]
        );
        assert_eq!(
            tiff(vec![1, 2, 3], 1, 2, 8, 2),
            Err(WorkerError::PdfDecodeError)
        );
    }
    #[test]
    fn png_rows_require_known_tags_and_complete_rows() {
        let data = [0, 10, 20, 30, 1, 10, 10, 10, 2, 0, 0, 0];
        assert_eq!(
            png(&data, 3, 1).unwrap(),
            [10, 20, 30, 10, 20, 30, 10, 20, 30]
        );
        assert_eq!(
            png(&[3, 10, 15, 20, 4, 0, 0, 0], 3, 1).unwrap(),
            [10, 20, 30, 10, 20, 30]
        );
        assert_eq!(png(&[0, 1, 2, 3], 3, 1).unwrap(), [1, 2, 3]);
        assert_eq!(png(&[5, 0, 0, 0], 3, 1), Err(WorkerError::PdfDecodeError));
        assert_eq!(png(&[0, 1], 3, 1), Err(WorkerError::PdfDecodeError));
    }
}
