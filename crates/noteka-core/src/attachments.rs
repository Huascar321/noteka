use crate::{
    contracts::*,
    error::*,
    workspace::{Workspace, hash, id, now},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub const MAX_ATTACHMENT_BYTES: u64 = 64 * 1024 * 1024;

pub(crate) fn metadata(conn: &Connection, id: &str) -> Result<Attachment> {
    let a = conn.query_row(
        "SELECT id,name,mime,bytes,sha256 FROM attachments WHERE id=?1",
        [id],
        |r| {
            Ok(Attachment {
                id: r.get(0)?,
                name: r.get(1)?,
                mime: r.get(2)?,
                bytes: u64::from(r.get::<_, u32>(3)?),
                sha256: r.get(4)?,
            })
        },
    )?;
    validate_metadata(&a)?;
    Ok(a)
}
fn validate_metadata(a: &Attachment) -> Result<()> {
    if uuid::Uuid::parse_str(&a.id).is_err()
        || a.id.contains(['/', '\\'])
        || a.bytes > MAX_ATTACHMENT_BYTES
        || a.sha256.len() != 64
        || !a.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(CoreError::new(
            ErrorCode::StorageError,
            "Metadatos de adjunto inválidos.",
        ));
    }
    Ok(())
}
pub(crate) fn all(conn: &Connection) -> Result<Vec<Attachment>> {
    let mut stmt = conn.prepare("SELECT id FROM attachments ORDER BY id")?;
    let ids = stmt
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ids.iter().map(|id| metadata(conn, id)).collect()
}
pub(crate) fn verify(root: &Path, a: &Attachment) -> Result<()> {
    validate_metadata(a)?;
    let dir = root.join("attachments");
    if !fs::symlink_metadata(&dir)?.file_type().is_dir() {
        return Err(CoreError::new(
            ErrorCode::StorageError,
            "El directorio de originales no es regular.",
        ));
    }
    let p = dir.join(&a.id);
    let m = fs::symlink_metadata(&p)?;
    if !m.file_type().is_file() || m.len() != a.bytes || hash(&p)? != a.sha256 {
        return Err(CoreError::new(
            ErrorCode::StorageError,
            "Original ausente o alterado; no se modificaron referencias.",
        ));
    }
    Ok(())
}
pub(crate) fn copy_original(source: &Path, destination: &Path, a: &Attachment) -> Result<()> {
    verify(source, a)?;
    fs::create_dir_all(destination.join("attachments"))?;
    let p = destination.join("attachments").join(&a.id);
    let mut from = fs::File::open(source.join("attachments").join(&a.id))?;
    let mut to = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&p)?;
    std::io::copy(
        &mut Read::by_ref(&mut from).take(MAX_ATTACHMENT_BYTES + 1),
        &mut to,
    )?;
    to.sync_all()?;
    verify(destination, a)
}
fn mime(header: &[u8]) -> &'static str {
    if header.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if header.starts_with(b"\xff\xd8\xff") {
        "image/jpeg"
    } else if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
        "image/gif"
    } else if header.starts_with(b"RIFF") && header.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else if header.starts_with(b"%PDF-") {
        "application/pdf"
    } else {
        "application/octet-stream"
    }
}
impl Workspace {
    pub fn attachment_import(&mut self, input: ImportAttachment) -> Result<Attachment> {
        let path = Path::new(&input.source_path);
        if !fs::metadata(path)?.is_file() {
            return Err(CoreError::validation("La fuente no es un archivo regular."));
        }
        let mut source = fs::File::open(path)?;
        if !source.metadata()?.is_file() || source.metadata()?.len() > MAX_ATTACHMENT_BYTES {
            return Err(CoreError::validation(
                "Importá un archivo regular de hasta 64 MiB.",
            ));
        }
        let dir = self.path.join("attachments");
        fs::create_dir_all(&dir)?;
        if !fs::symlink_metadata(&dir)?.file_type().is_dir() {
            return Err(CoreError::validation("Directorio de adjuntos inseguro."));
        }
        let mut stage = tempfile::NamedTempFile::new_in(&dir)?;
        let mut digest = Sha256::new();
        let mut bytes = 0u64;
        let mut header = vec![];
        let mut buffer = [0; 65536];
        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            bytes += n as u64;
            if bytes > MAX_ATTACHMENT_BYTES {
                return Err(CoreError::validation(
                    "El archivo creció por encima de 64 MiB.",
                ));
            }
            if header.is_empty() {
                header.extend_from_slice(&buffer[..n.min(16)]);
            }
            digest.update(&buffer[..n]);
            stage.write_all(&buffer[..n])?;
        }
        stage.as_file().sync_all()?;
        let a = Attachment {
            id: id(),
            name: path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "archivo".into()),
            mime: mime(&header).into(),
            bytes,
            sha256: format!("{:x}", digest.finalize()),
        };
        let managed = dir.join(&a.id);
        stage
            .persist_noclobber(&managed)
            .map_err(|e| CoreError::from(e.error))?;
        let result = (|| -> Result<()> {
            fs::File::open(&dir)?.sync_all()?;
            verify(&self.path, &a)?;
            self.conn.execute(
                "INSERT INTO attachments VALUES (?1,?2,?3,?4,?5,?6)",
                params![a.id, a.name, a.mime, a.bytes as i64, a.sha256, now()],
            )?;
            Ok(())
        })();
        if let Err(e) = result {
            let _ = fs::remove_file(managed);
            return Err(e);
        }
        Ok(a)
    }
    pub fn attachment_read(&self, input: IdInput) -> Result<AttachmentData> {
        let a = metadata(&self.conn, &input.id)?;
        verify(&self.path, &a)?;
        let bytes = fs::read(self.path.join("attachments").join(&a.id))?;
        if bytes.len() as u64 != a.bytes || format!("{:x}", Sha256::digest(&bytes)) != a.sha256 {
            return Err(CoreError::new(
                ErrorCode::StorageError,
                "Original cambió durante la lectura.",
            ));
        }
        Ok(AttachmentData {
            attachment: a,
            base64: STANDARD.encode(bytes),
        })
    }
}
