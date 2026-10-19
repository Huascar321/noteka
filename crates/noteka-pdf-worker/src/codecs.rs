//! Codecs PDF 1.x sin recuperación de EOD/bytes perdidos. Cada escritura lleva cap.
use crate::{framing::whitespace, parser::Result};
use noteka_pdf_protocol::WorkerError;

fn append(output: &mut Vec<u8>, bytes: &[u8], cap: usize) -> Result<()> {
    if output
        .len()
        .checked_add(bytes.len())
        .is_none_or(|n| n > cap)
    {
        return Err(WorkerError::PdfStreamLimit);
    }
    output.extend_from_slice(bytes);
    Ok(())
}
fn trailing(bytes: &[u8]) -> Result<()> {
    if bytes.iter().any(|&b| !whitespace(b)) {
        return Err(WorkerError::PdfDecodeError);
    }
    Ok(())
}

pub fn ascii_hex(input: &[u8], cap: usize) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut high = None;
    for (index, &byte) in input.iter().enumerate() {
        if byte == b'>' {
            if let Some(high) = high {
                append(&mut output, &[high << 4], cap)?;
            }
            trailing(&input[index + 1..])?;
            return Ok(output);
        }
        if whitespace(byte) {
            continue;
        }
        let nibble = (byte as char)
            .to_digit(16)
            .ok_or(WorkerError::PdfDecodeError)? as u8;
        if let Some(h) = high.take() {
            append(&mut output, &[(h << 4) | nibble], cap)?;
        } else {
            high = Some(nibble);
        }
    }
    Err(WorkerError::PdfDecodeError)
}

pub fn ascii85(input: &[u8], cap: usize) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut value = 0u64;
    let mut count = 0usize;
    let mut index = 0;
    while let Some(&byte) = input.get(index) {
        index += 1;
        if whitespace(byte) {
            continue;
        }
        if byte == b'~' {
            if input.get(index) != Some(&b'>') || count == 1 {
                return Err(WorkerError::PdfDecodeError);
            }
            if count > 1 {
                for _ in count..5 {
                    value = value * 85 + 84;
                }
                let bytes = u32::try_from(value)
                    .map_err(|_| WorkerError::PdfDecodeError)?
                    .to_be_bytes();
                append(&mut output, &bytes[..count - 1], cap)?;
            }
            trailing(&input[index + 1..])?;
            return Ok(output);
        }
        if byte == b'z' {
            if count != 0 {
                return Err(WorkerError::PdfDecodeError);
            }
            append(&mut output, &[0; 4], cap)?;
            continue;
        }
        if !(b'!'..=b'u').contains(&byte) {
            return Err(WorkerError::PdfDecodeError);
        }
        value = value * 85 + u64::from(byte - b'!');
        count += 1;
        if count == 5 {
            append(
                &mut output,
                &u32::try_from(value)
                    .map_err(|_| WorkerError::PdfDecodeError)?
                    .to_be_bytes(),
                cap,
            )?;
            value = 0;
            count = 0;
        }
    }
    Err(WorkerError::PdfDecodeError)
}

pub fn run_length(input: &[u8], cap: usize) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut index = 0;
    while let Some(&length) = input.get(index) {
        index += 1;
        match length {
            128 => {
                trailing(&input[index..])?;
                return Ok(output);
            }
            0..=127 => {
                let end = index + usize::from(length) + 1;
                append(
                    &mut output,
                    input.get(index..end).ok_or(WorkerError::PdfDecodeError)?,
                    cap,
                )?;
                index = end;
            }
            _ => {
                let byte = *input.get(index).ok_or(WorkerError::PdfDecodeError)?;
                index += 1;
                let length = 257 - usize::from(length);
                if output.len().checked_add(length).is_none_or(|n| n > cap) {
                    return Err(WorkerError::PdfStreamLimit);
                }
                output.resize(output.len() + length, byte);
            }
        }
    }
    Err(WorkerError::PdfDecodeError)
}

/// LZW MSB de PDF, Clear=256, EOD=257, 9–12 bits, EarlyChange 0/1.
/// Diccionario de como máximo 4096 entradas; nunca depende del count de un PDF.
pub fn lzw(input: &[u8], cap: usize, early_change: usize) -> Result<Vec<u8>> {
    if early_change > 1 {
        return Err(WorkerError::PdfDecodeError);
    }
    let mut dictionary: Vec<Vec<u8>> = (0..256).map(|n| vec![n as u8]).collect();
    dictionary.extend([Vec::new(), Vec::new()]);
    let mut previous: Option<Vec<u8>> = None;
    let mut width = 9;
    let mut bit = 0usize;
    let mut cleared = false;
    let mut output = Vec::new();
    loop {
        if bit
            .checked_add(width)
            .is_none_or(|end| end > input.len() * 8)
        {
            return Err(WorkerError::PdfDecodeError);
        }
        let mut code = 0usize;
        for _ in 0..width {
            code = (code << 1) | usize::from((input[bit / 8] >> (7 - bit % 8)) & 1);
            bit += 1;
        }
        if code == 256 {
            dictionary.truncate(258);
            previous = None;
            width = 9;
            cleared = true;
            continue;
        }
        if !cleared {
            return Err(WorkerError::PdfDecodeError);
        }
        if code == 257 {
            // Sólo padding de bits del último byte, no bytes extra ocultos.
            if input.len() * 8 - bit >= 8 {
                return Err(WorkerError::PdfDecodeError);
            }
            return Ok(output);
        }
        let entry = if code < dictionary.len() {
            dictionary[code].clone()
        } else if code == dictionary.len()
            && let Some(previous) = &previous
        {
            let mut entry = previous.clone();
            entry.push(previous[0]);
            entry
        } else {
            return Err(WorkerError::PdfDecodeError);
        };
        if entry.is_empty() {
            return Err(WorkerError::PdfDecodeError);
        }
        append(&mut output, &entry, cap)?;
        if let Some(mut previous) = previous
            && dictionary.len() < 4096
        {
            previous.push(entry[0]);
            dictionary.push(previous);
            if width < 12 && dictionary.len() + early_change == 1 << width {
                width += 1;
            }
        }
        previous = Some(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_eod_glyph_bytes_caps_and_corruption() {
        assert_eq!(ascii_hex(b"61 62 6>\n", 3).unwrap(), b"ab`");
        assert_eq!(ascii85(b"z~>", 4).unwrap(), [0; 4]);
        assert_eq!(ascii85(b"@:E_W~>", 4).unwrap(), b"abcd");
        assert_eq!(ascii85(b"@:~>", 1).unwrap(), b"a");
        assert_eq!(ascii85(b"@:B~>", 2).unwrap(), b"ab");
        assert_eq!(ascii85(b"@:E^~>", 3).unwrap(), b"abc");
        assert_eq!(run_length(b"\x02abc\xfeX\x80", 6).unwrap(), b"abcXXX");
        assert_eq!(ascii_hex(b"6162>", 1), Err(WorkerError::PdfStreamLimit));
        assert_eq!(ascii85(b"z~>", 3), Err(WorkerError::PdfStreamLimit));
        assert_eq!(
            run_length(b"\x81X\x80", 127),
            Err(WorkerError::PdfStreamLimit)
        );
        for bad in [b"61".as_slice(), b"61>junk", b"6G>"] {
            assert_eq!(ascii_hex(bad, 10), Err(WorkerError::PdfDecodeError));
        }
        for bad in [b"z".as_slice(), b"!~>", b"!z~>", b"uuuuu~>", b"z~>junk"] {
            assert_eq!(ascii85(bad, 10), Err(WorkerError::PdfDecodeError));
        }
        for bad in [b"\x02ab".as_slice(), b"\xff", b"\x80x", b"\x00a"] {
            assert_eq!(run_length(bad, 10), Err(WorkerError::PdfDecodeError));
        }
        // Codes explicitos 256,65,66,258,257 a nueve bits: golden ABAB.
        let vector = [0x80, 0x10, 0x48, 0x50, 0x28, 0x08];
        assert_eq!(lzw(&vector, 4, 1).unwrap(), b"ABAB");
        assert_eq!(lzw(&vector, 3, 1), Err(WorkerError::PdfStreamLimit));
        assert_eq!(lzw(&vector[..5], 4, 1), Err(WorkerError::PdfDecodeError));
    }

    #[test]
    fn lzw_width_transitions_match_independent_weezl_decoder_and_literal_golden() {
        use lopdf::{Object, Stream, dictionary};
        for early in [0, 1] {
            let golden: Vec<u8> = (0..12_800).map(|n| n as u8).collect();
            let mut packed = Vec::new();
            let mut bit = 0usize;
            let mut width = 9;
            let mut next = 258;
            let mut previous = false;
            for code in std::iter::once(256)
                .chain(golden.iter().map(|&b| usize::from(b)))
                .chain(std::iter::once(257))
            {
                for i in (0..width).rev() {
                    if bit.is_multiple_of(8) {
                        packed.push(0u8);
                    }
                    packed[bit / 8] |= (((code >> i) & 1) as u8) << (7 - bit % 8);
                    bit += 1;
                }
                if code == 256 {
                    width = 9;
                    next = 258;
                    previous = false;
                } else if code != 257 {
                    if previous && next < 4096 {
                        next += 1;
                        if width < 12 && next + early == 1 << width {
                            width += 1;
                        }
                    }
                    previous = true;
                }
            }
            let stream = Stream::new(
                dictionary! {
                    "Filter" => "LZWDecode",
                    "DecodeParms" => dictionary! { "EarlyChange" => Object::Integer(early as i64) },
                },
                packed.clone(),
            );
            assert_eq!(
                stream.get_plain_content_with_limit(golden.len()).unwrap(),
                golden
            );
            assert_eq!(lzw(&packed, golden.len(), early).unwrap(), golden);
            assert_eq!(
                lzw(&packed, golden.len() - 1, early),
                Err(WorkerError::PdfStreamLimit)
            );
        }
    }
}
