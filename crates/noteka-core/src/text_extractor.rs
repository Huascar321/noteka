use crate::{contracts::*, error::*, extraction_storage::invalid};

/// Puro: el caller es dueño de IO, deadline, ownership y publicación.
pub(crate) fn extract(
    bytes: &[u8],
    mut check: impl FnMut() -> Result<()>,
) -> Result<Vec<ExtractionFragment>> {
    if bytes.len() > MAX_EXTRACTION_BYTES as usize {
        return Err(invalid("source_size_limit"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("invalid_utf8"))?;
    if bytes.contains(&0) {
        return Err(invalid("nul_in_text"));
    }
    let mut out = Vec::new();
    let mut start = 0;
    let mut line = 1u32;
    while start < bytes.len() {
        check()?;
        let mut end = (start + EXTRACTION_CHUNK_BYTES as usize).min(bytes.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let slice = &bytes[start..end];
        let line_end = line
            + slice[..slice.len() - 1]
                .iter()
                .filter(|&&b| b == b'\n')
                .count() as u32;
        out.push(ExtractionFragment {
            ordinal: out.len() as u32,
            text: text[start..end].into(),
            byte_start: start as u32,
            byte_end: end as u32,
            line_start: line,
            line_end,
        });
        line += slice.iter().filter(|&&b| b == b'\n').count() as u32;
        start = end;
        if out.len() > MAX_EXTRACTION_FRAGMENTS as usize {
            return Err(invalid("fragment_count_limit"));
        }
    }
    check()?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf8_literal_ranges_reconstruct_bom_crlf_and_long_lines() {
        for text in [
            String::new(),
            "\u{feff}a\r\n\n<script>fetch('https://invalid')</script>".into(),
            "é世😀\n".repeat(30_000),
            "x".repeat(MAX_EXTRACTION_BYTES as usize),
            format!("{}\r\n😀end", "a".repeat(4095)),
        ] {
            let parts = extract(text.as_bytes(), || Ok(())).unwrap();
            assert_eq!(
                parts.iter().map(|p| p.text.as_str()).collect::<String>(),
                text
            );
            let mut offset = 0;
            for p in parts {
                assert_eq!(p.byte_start, offset);
                assert!(p.text.len() <= 4096);
                assert_eq!(&text[p.byte_start as usize..p.byte_end as usize], p.text);
                assert_eq!(
                    p.line_start,
                    1 + text.as_bytes()[..p.byte_start as usize]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count() as u32
                );
                assert_eq!(
                    p.line_end,
                    1 + text.as_bytes()[..p.byte_end as usize - 1]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count() as u32
                );
                offset = p.byte_end;
            }
            assert_eq!(offset as usize, text.len());
        }
        for bytes in [
            vec![0],
            vec![0xff],
            vec![b'x'; MAX_EXTRACTION_BYTES as usize + 1],
        ] {
            assert!(extract(&bytes, || Ok(())).is_err());
        }
    }
}
