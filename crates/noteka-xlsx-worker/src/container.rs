//! ZIP cerrado: validar directorio/encabezados ANTES de construir ZipArchive.
use noteka_xlsx_protocol::{MAX_ENTRIES, MAX_ENTRY, MAX_EXPANDED, MAX_SOURCE, WorkerError as E};
use std::{
    collections::{BTreeMap, HashSet},
    io::{Cursor, Read},
};

type Result<T> = std::result::Result<T, E>;
fn bytes(b: &[u8], p: usize, n: usize) -> Result<&[u8]> {
    b.get(p..p.checked_add(n).ok_or(E::XlsxInvalidZip)?)
        .ok_or(E::XlsxInvalidZip)
}
fn u16at(b: &[u8], p: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        bytes(b, p, 2)?.try_into().map_err(|_| E::XlsxInvalidZip)?,
    ))
}
fn u32at(b: &[u8], p: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes(b, p, 4)?.try_into().map_err(|_| E::XlsxInvalidZip)?,
    ))
}
pub(crate) fn canonical(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 512
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.[]/".contains(&b))
        && s.split('/').all(|s| !matches!(s, "" | "." | ".."))
}
fn extras(mut b: &[u8]) -> Result<()> {
    while !b.is_empty() {
        let kind = u16at(b, 0)?;
        let len = u16at(b, 2)? as usize;
        if matches!(kind, 0x0001 | 0x7075) {
            return Err(E::XlsxUnsupportedZip);
        }
        b = b.get(4 + len..).ok_or(E::XlsxInvalidZip)?;
    }
    Ok(())
}
struct Entry {
    name: String,
    offset: usize,
    end: usize,
    size: usize,
    data: usize,
    compressed: usize,
    method: u16,
}
fn preflight(b: &[u8]) -> Result<Vec<Entry>> {
    if b.is_empty() || b.len() > MAX_SOURCE {
        return Err(E::XlsxStructureLimit);
    }
    let eocd = (b.len().saturating_sub(65_557)..b.len().saturating_sub(21))
        .rev()
        .find(|&p| {
            bytes(b, p, 4).is_ok_and(|v| v == b"PK\x05\x06")
                && u16at(b, p + 20).is_ok_and(|n| p + 22 + usize::from(n) == b.len())
        })
        .ok_or(E::XlsxInvalidZip)?;
    let count = u16at(b, eocd + 10)? as usize;
    if count == 0 || count > MAX_ENTRIES {
        return Err(E::XlsxStructureLimit);
    }
    if u16at(b, eocd + 4)? != 0
        || u16at(b, eocd + 6)? != 0
        || usize::from(u16at(b, eocd + 8)?) != count
    {
        return Err(E::XlsxUnsupportedZip);
    }
    let central = u32at(b, eocd + 16)? as usize;
    if central + u32at(b, eocd + 12)? as usize != eocd {
        return Err(E::XlsxInvalidZip);
    }
    let mut p = central;
    let mut names = HashSet::new();
    let mut entries = Vec::new();
    let mut total = 0usize;
    for _ in 0..count {
        if bytes(b, p, 4)? != b"PK\x01\x02" {
            return Err(E::XlsxInvalidZip);
        }
        let flags = u16at(b, p + 8)?;
        let method = u16at(b, p + 10)?;
        let version = u16at(b, p + 6)?;
        let crc = u32at(b, p + 16)?;
        let compressed = u32at(b, p + 20)? as usize;
        let size = u32at(b, p + 24)? as usize;
        let n = u16at(b, p + 28)? as usize;
        let x = u16at(b, p + 30)? as usize;
        let comment = u16at(b, p + 32)? as usize;
        let offset = u32at(b, p + 42)? as usize;
        if u16at(b, p + 34)? != 0
            || flags & !0x080e != 0
            || !matches!(method, 0 | 8)
            || !matches!(version, 10 | 20)
            || (method == 8 && version < 20)
            || (method == 0 && flags & 6 != 0)
        {
            return Err(E::XlsxUnsupportedZip);
        }
        if method == 0 && compressed != size {
            return Err(E::XlsxInvalidZip);
        }
        if size > MAX_ENTRY {
            return Err(E::XlsxExpansionLimit);
        }
        total = total
            .checked_add(size)
            .filter(|&v| v <= MAX_EXPANDED)
            .ok_or(E::XlsxExpansionLimit)?;
        let raw_name = bytes(b, p + 46, n)?;
        let name = std::str::from_utf8(raw_name).map_err(|_| E::XlsxUnsupportedZip)?;
        let is_dir = name.ends_with('/');
        if !canonical(name.strip_suffix('/').unwrap_or(name)) || !names.insert(name.to_owned()) {
            return Err(E::XlsxInvalidZip);
        }
        let mode = u32at(b, p + 38)? >> 16 & 0o170000;
        if !matches!(mode, 0 | 0o100000 | 0o040000)
            || (mode == 0o040000 && !is_dir)
            || (is_dir && size != 0)
        {
            return Err(E::XlsxUnsupportedZip);
        }
        extras(bytes(b, p + 46 + n, x)?)?;
        p += 46 + n + x + comment;
        if p > eocd || bytes(b, offset, 4)? != b"PK\x03\x04" || offset >= central {
            return Err(E::XlsxInvalidZip);
        }
        let ln = u16at(b, offset + 26)? as usize;
        let lx = u16at(b, offset + 28)? as usize;
        if u16at(b, offset + 4)? != version
            || u16at(b, offset + 6)? != flags
            || u16at(b, offset + 8)? != method
            || bytes(b, offset + 30, ln)? != raw_name
        {
            return Err(E::XlsxInvalidZip);
        }
        extras(bytes(b, offset + 30 + ln, lx)?)?;
        if flags & 8 == 0
            && (u32at(b, offset + 14)? != crc
                || u32at(b, offset + 18)? as usize != compressed
                || u32at(b, offset + 22)? as usize != size)
        {
            return Err(E::XlsxInvalidZip);
        }
        if flags & 8 != 0
            && (![0, crc].contains(&u32at(b, offset + 14)?)
                || ![0, compressed as u32].contains(&u32at(b, offset + 18)?)
                || ![0, size as u32].contains(&u32at(b, offset + 22)?))
        {
            return Err(E::XlsxInvalidZip);
        }
        let mut end = offset + 30 + ln + lx + compressed;
        bytes(b, offset + 30 + ln + lx, compressed)?;
        if flags & 8 != 0 {
            // Descriptor firmado o sin firma; tamaño/CRC deben coincidir exactamente.
            if bytes(b, end, 4)? == b"PK\x07\x08" {
                end += 4;
            }
            if u32at(b, end)? != crc
                || u32at(b, end + 4)? as usize != compressed
                || u32at(b, end + 8)? as usize != size
            {
                return Err(E::XlsxInvalidZip);
            }
            end += 12;
        }
        if end > central {
            return Err(E::XlsxInvalidZip);
        }
        entries.push(Entry {
            name: name.into(),
            offset,
            end,
            size,
            data: offset + 30 + ln + lx,
            compressed,
            method,
        });
    }
    if p != eocd {
        return Err(E::XlsxInvalidZip);
    }
    let mut physical: Vec<_> = entries.iter().collect();
    physical.sort_unstable_by_key(|e| e.offset);
    let mut end = 0;
    for entry in physical {
        if entry.offset != end {
            return Err(E::XlsxInvalidZip);
        }
        end = entry.end;
    }
    if end != central {
        return Err(E::XlsxInvalidZip);
    }
    Ok(entries)
}
pub(crate) fn read(b: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    let entries = preflight(b)?;
    let mut archive = zip::ZipArchive::new(Cursor::new(b)).map_err(|_| E::XlsxInvalidZip)?;
    if archive.len() != entries.len() {
        return Err(E::XlsxInvalidZip);
    }
    let mut out = BTreeMap::new();
    for (index, e) in entries.into_iter().enumerate() {
        let f = archive.by_index(index).map_err(|_| E::XlsxInvalidZip)?;
        if f.name_raw() != e.name.as_bytes()
            || f.name().map_err(|_| E::XlsxInvalidZip)? != e.name
            || f.encrypted()
            || f.is_symlink()
            || f.size() != e.size as u64
            || f.header_start() != e.offset as u64
        {
            return Err(E::XlsxInvalidZip);
        }
        let mut part = Vec::new();
        f.take(MAX_ENTRY as u64 + 1)
            .read_to_end(&mut part)
            .map_err(|_| E::XlsxInvalidZip)?;
        if part.len() > MAX_ENTRY {
            return Err(E::XlsxExpansionLimit);
        }
        if part.len() != e.size {
            return Err(E::XlsxInvalidZip);
        }
        if e.method == 8 {
            // ZipFile verifica CRC/salida pero un decoder puede aceptar bytes
            // tras el fin de Deflate. Verificar también el consumo exacto del
            // stream comprimido; no aceptar padding oculto como parte del ZIP.
            let stream = bytes(b, e.data, e.compressed)?;
            let mut decoder = flate2::bufread::DeflateDecoder::new(stream);
            let mut scratch = [0u8; 16 * 1024];
            let mut actual = 0;
            loop {
                let n = decoder.read(&mut scratch).map_err(|_| E::XlsxInvalidZip)?;
                actual += n;
                if actual > e.size {
                    return Err(E::XlsxInvalidZip);
                }
                if n == 0 {
                    break;
                }
            }
            if actual != e.size || decoder.total_in() != e.compressed as u64 {
                return Err(E::XlsxInvalidZip);
            }
        }
        if !e.name.ends_with('/') {
            out.insert(e.name, part);
        }
    }
    Ok(out)
}
