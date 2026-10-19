use crate::{contracts::*, error::*};
use chrono::Utc;
use rusqlite::{
    Connection, OpenFlags, TransactionBehavior,
    backup::{Backup, StepResult},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 13;
pub const BUSY_TIMEOUT_MS: u64 = 2_000;
pub const DB_NAME: &str = "noteka.sqlite";

pub struct Workspace {
    pub(crate) conn: Connection,
    pub path: PathBuf,
}

pub fn default_path() -> Result<PathBuf> {
    dirs::data_local_dir()
        .map(|p| p.join("Noteka").join("workspace"))
        .ok_or_else(|| {
            CoreError::new(
                ErrorCode::StorageError,
                "No hay directorio de datos local disponible.",
            )
        })
}
pub(crate) fn now() -> String {
    Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
}
pub(crate) fn id() -> String {
    Uuid::new_v4().to_string()
}

fn configure(conn: &Connection) -> Result<()> {
    conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "FULL")?;
    Ok(())
}
pub(crate) fn version(conn: &Connection) -> Result<u32> {
    let v: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let (meta, count, workspace_id, timezone): (u32, u32, String, String) = conn.query_row(
        "SELECT schema_version, (SELECT count(*) FROM workspace_meta),id,timezone FROM workspace_meta",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    if v != meta || count != 1 {
        return Err(CoreError::new(
            ErrorCode::StorageError,
            "Identidad o versión de workspace inconsistente.",
        ));
    }
    supported(v)?;
    if Uuid::parse_str(&workspace_id).is_err() || timezone.parse::<chrono_tz::Tz>().is_err() {
        return Err(CoreError::new(
            ErrorCode::StorageError,
            "Identidad o zona horaria de workspace inválida.",
        ));
    }
    Ok(v)
}
fn schema_check(conn: &Connection) -> Result<()> {
    let v = version(conn)?;
    // Comprobar columnas requeridas sin rechazar tablas/campos de extensiones.
    for sql in [
        "SELECT id,kind,title,revision,archived_at,created_at,updated_at,payload FROM entities LIMIT 0",
        "SELECT entity_id,schema_version,json FROM documents LIMIT 0",
        "SELECT id,source_id,target_id,kind FROM relations LIMIT 0",
        "SELECT id,schema_version,timezone,created_at FROM workspace_meta LIMIT 0",
    ] {
        conn.prepare(sql)?;
    }
    if v >= 1 {
        conn.prepare("SELECT id,request_id,entity_id,operation,origin,previous_revision,new_revision,created_at FROM changes LIMIT 0")?;
    }
    if v >= 2 {
        conn.prepare("SELECT id,name,mime,bytes,sha256,created_at FROM attachments LIMIT 0")?;
    }
    if v >= 3 {
        conn.prepare("SELECT entity_id,block_id,revision,title,text FROM search_fts LIMIT 0")?;
    }
    if v >= 4 {
        for sql in [
            "SELECT id,name,revision FROM collections LIMIT 0",
            "SELECT id,collection_id,name,spec FROM properties LIMIT 0",
            "SELECT collection_id,entity_id FROM memberships LIMIT 0",
            "SELECT entity_id,property_id,json FROM property_values LIMIT 0",
            "SELECT id,collection_id,revision,json FROM views LIMIT 0",
        ] {
            conn.prepare(sql)?;
        }
    }
    if v >= 5 {
        conn.prepare("SELECT property_id,expression FROM calculations LIMIT 0")?;
    }
    if v >= 7 {
        conn.prepare("SELECT id,collection_id,name,revision,definition FROM templates LIMIT 0")?;
    }
    if v >= 8 {
        for sql in [
            "SELECT id,version,paused,revision,updated_at FROM extraction_control LIMIT 0",
            "SELECT id,job_id,token,scope,control_revision,lease_until FROM extraction_runtime LIMIT 0",
            "SELECT id,attachment_id,source_sha256,source_bytes,extractor,extractor_version,config,config_hash,state,revision,attempts,created_at,updated_at,started_at,finished_at,error,generation_id FROM extraction_jobs LIMIT 0",
            "SELECT id,job_id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,covered_bytes,fragment_count,published_at,attempt FROM extraction_generations LIMIT 0",
            "SELECT generation_id,job_id,ordinal,text,byte_start,byte_end,line_start,line_end FROM extraction_fragments LIMIT 0",
        ] {
            conn.prepare(sql)?;
        }
    }
    if v >= 9 {
        for sql in [
            "SELECT generation_id,job_id,version,parser_name,parser_version,page_count,extracted_bytes FROM extraction_pdf_generations LIMIT 0",
            "SELECT generation_id,page_number,state,text_bytes,text_sha256,fragment_count FROM extraction_pdf_pages LIMIT 0",
            "SELECT generation_id,job_id,ordinal,text,page_number,text_byte_start,text_byte_end FROM extraction_pdf_fragments LIMIT 0",
        ] {
            conn.prepare(sql)?;
        }
    }
    if v >= 10 {
        for sql in [
            "SELECT id,version,revision,profile,updated_at FROM extraction_search_control LIMIT 0",
            "SELECT slot,job_id,generation_id,fragment_count,text_bytes,content_hash,indexed_at FROM extraction_search_entries LIMIT 0",
            "SELECT job_id,generation_id,ordinal,text FROM extraction_search_fts LIMIT 0",
        ] {
            conn.prepare(sql)?;
        }
    }
    if v >= 11 {
        for sql in [
            "SELECT generation_id,job_id,version,parser,representation_sha256,representation_bytes,sheet_count,cell_count,chunk_count FROM extraction_xlsx_generations LIMIT 0",
            "SELECT generation_id,job_id,ordinal,text FROM extraction_xlsx_chunks LIMIT 0",
        ] {
            conn.prepare(sql)?;
        }
    }
    if v >= 13 {
        conn.prepare("SELECT generation_id,job_id,version,profile,representation_sha256,representation_bytes,region_count,chunk_count FROM extraction_ocr_generations LIMIT 0")?;
        conn.prepare(
            "SELECT generation_id,job_id,ordinal,text FROM extraction_ocr_chunks LIMIT 0",
        )?;
    }
    Ok(())
}
fn supported(v: u32) -> Result<()> {
    if v > SCHEMA_VERSION {
        let mut e = CoreError::new(
            ErrorCode::UnsupportedWorkspaceVersion,
            "Este workspace requiere una versión más nueva de Noteka.",
        );
        e.details = json!({"current_version":v,"supported_version":SCHEMA_VERSION});
        return Err(e);
    }
    Ok(())
}
fn integrity(conn: &Connection) -> Result<()> {
    let check: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    let fk_error = conn.prepare("PRAGMA foreign_key_check")?.exists([])?;
    if check != "ok" || fk_error {
        return Err(CoreError::new(
            ErrorCode::StorageError,
            "La base no supera la validación de integridad.",
        ));
    }
    schema_check(conn)?;
    if version(conn)? >= 8 {
        crate::extraction_storage::integrity(conn)?;
    }
    if version(conn)? >= 10 {
        crate::attachment_search_storage::integrity(conn)?;
    }
    Ok(())
}

// La prevalidación no abre el original: incluso SQLite de solo lectura puede
// crear/modificar un SHM. Esta copia privada NO es un backup ni se publica como tal.
pub(crate) fn preflight(path: &Path) -> Result<u32> {
    preflight_impl(
        path,
        #[cfg(test)]
        None,
    )
}
fn preflight_impl(
    path: &Path,
    #[cfg(test)] before_wal_copy: Option<&mut dyn FnMut()>,
) -> Result<u32> {
    Ok(preflight_probe(
        path,
        #[cfg(test)]
        before_wal_copy,
    )?
    .0)
}
pub(crate) fn preflight_identity(path: &Path) -> Result<(u32, String)> {
    preflight_probe(
        path,
        #[cfg(test)]
        None,
    )
}
fn preflight_probe(
    path: &Path,
    #[cfg(test)] before_wal_copy: Option<&mut dyn FnMut()>,
) -> Result<(u32, String)> {
    if !path.join(DB_NAME).is_file() {
        return Err(CoreError::new(
            ErrorCode::NotFound,
            "La ruta no contiene un workspace Noteka.",
        ));
    }
    let stage = tempfile::tempdir()?;
    fs::copy(path.join(DB_NAME), stage.path().join(DB_NAME))?;
    let wal = path.join(format!("{DB_NAME}-wal"));
    if wal.exists() {
        #[cfg(test)]
        if let Some(hook) = before_wal_copy {
            hook();
        }
        let private_wal = stage.path().join(format!("{DB_NAME}-wal"));
        match fs::copy(&wal, &private_wal) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                // La última conexión puede hacer checkpoint y eliminar WAL
                // entre exists/copy. No es una falla del workspace. Leer de
                // nuevo la base evita validar la copia anterior al checkpoint.
                match fs::remove_file(&private_wal) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
                fs::copy(path.join(DB_NAME), stage.path().join(DB_NAME))?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let probe =
        Connection::open_with_flags(stage.path().join(DB_NAME), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let v = version(&probe)?;
    supported(v)?;
    schema_check(&probe)?;
    let id = probe.query_row("SELECT id FROM workspace_meta", [], |r| r.get(0))?;
    Ok((v, id))
}

impl Workspace {
    pub fn create(path: &Path, timezone: Option<String>) -> Result<Self> {
        let timezone = timezone
            .or_else(|| iana_time_zone::get_timezone().ok())
            .unwrap_or_else(|| "UTC".into());
        if timezone.parse::<chrono_tz::Tz>().is_err() {
            return Err(CoreError::validation("Zona horaria IANA inválida."));
        }
        publish(path, |stage| {
            let mut conn = Connection::open(stage.join(DB_NAME))?;
            conn.pragma_update(None, "foreign_keys", true)?;
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            tx.execute_batch(include_str!("../migrations/001.sql"))?;
            tx.execute(
                "INSERT INTO workspace_meta VALUES (?1,1,?2,?3)",
                rusqlite::params![id(), timezone, now()],
            )?;
            tx.execute_batch(include_str!("../migrations/002.sql"))?;
            tx.execute_batch(include_str!("../migrations/003.sql"))?;
            tx.execute_batch(include_str!("../migrations/004.sql"))?;
            tx.execute_batch(include_str!("../migrations/005.sql"))?;
            tx.execute_batch(include_str!("../migrations/006.sql"))?;
            tx.execute_batch(include_str!("../migrations/007.sql"))?;
            tx.execute_batch(include_str!("../migrations/008.sql"))?;
            tx.execute_batch(include_str!("../migrations/009.sql"))?;
            tx.execute_batch(include_str!("../migrations/010.sql"))?;
            tx.execute_batch(include_str!("../migrations/011.sql"))?;
            tx.execute_batch(include_str!("../migrations/012.sql"))?;
            tx.execute_batch(include_str!("../migrations/013.sql"))?;
            tx.commit()?;
            integrity(&conn)?;
            Ok(())
        })?;
        Self::open(path)
    }
    pub fn open(path: &Path) -> Result<Self> {
        preflight(path)?;
        let path = fs::canonicalize(path)?;
        let conn =
            Connection::open_with_flags(path.join(DB_NAME), OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
        let mut workspace = Self { conn, path };
        supported(version(&workspace.conn)?)?;
        if version(&workspace.conn)? < SCHEMA_VERSION {
            workspace.migrate(false)?;
        }
        // Formato XLSX nuevo: validar íntegramente en el snapshot efectivo antes
        // de configurar escritura. No parsea ZIP/XML ni inicia helpers; los
        // verificadores UTF-8/PDF/FTS mantienen sus fronteras anteriores.
        {
            let tx = workspace.conn.unchecked_transaction()?;
            crate::xlsx_storage::integrity(&tx)?;
            crate::ocr_storage::integrity(&tx)?;
            tx.commit()?;
        }
        configure(&workspace.conn)?;
        Ok(workspace)
    }
    pub fn info(&self) -> Result<WorkspaceInfo> {
        Ok(self.conn.query_row(
            "SELECT id,schema_version,timezone FROM workspace_meta",
            [],
            |r| {
                Ok(WorkspaceInfo {
                    id: r.get(0)?,
                    path: self.path.to_string_lossy().into(),
                    schema_version: r.get(1)?,
                    timezone: r.get(2)?,
                })
            },
        )?)
    }
    pub(crate) fn migrate(&mut self, inject_failure: bool) -> Result<()> {
        let backup_path = self
            .path
            .parent()
            .unwrap_or(&self.path)
            .join(format!("noteka-pre-migration-{}", id()));
        self.backup(&backup_path)?;
        let result = (|| -> Result<()> {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            // v0 es la fixture previa soportada: entidades/documentos/relaciones,
            // todavía sin historial. El chequeo dentro del lock evita doble migración.
            if version(&tx)? == 0 {
                tx.execute_batch("CREATE TABLE changes (id TEXT PRIMARY KEY, request_id TEXT NOT NULL, entity_id TEXT NOT NULL REFERENCES entities(id), operation TEXT NOT NULL, origin TEXT NOT NULL, previous_revision INTEGER NOT NULL, new_revision INTEGER NOT NULL, created_at TEXT NOT NULL); CREATE INDEX changes_listing ON changes(created_at,id); UPDATE workspace_meta SET schema_version=1; PRAGMA user_version=1;")?;
            }
            if version(&tx)? == 1 {
                tx.execute_batch(include_str!("../migrations/002.sql"))?;
            }
            if version(&tx)? == 2 {
                tx.execute_batch(include_str!("../migrations/003.sql"))?;
                crate::search::rebuild(&tx)?;
            }
            if version(&tx)? == 3 {
                tx.execute_batch(include_str!("../migrations/004.sql"))?;
            }
            if version(&tx)? == 4 {
                tx.execute_batch(include_str!("../migrations/005.sql"))?;
            }
            if version(&tx)? == 5 {
                tx.execute_batch(include_str!("../migrations/006.sql"))?;
            }
            if version(&tx)? == 6 {
                tx.execute_batch(include_str!("../migrations/007.sql"))?;
            }
            if version(&tx)? == 7 {
                tx.execute_batch(include_str!("../migrations/008.sql"))?;
            }
            if version(&tx)? == 8 {
                tx.execute_batch(include_str!("../migrations/009.sql"))?;
            }
            if version(&tx)? == 9 {
                tx.execute_batch(include_str!("../migrations/010.sql"))?;
            }
            if version(&tx)? == 10 {
                tx.execute_batch(include_str!("../migrations/011.sql"))?;
            }
            if version(&tx)? == 11 {
                tx.execute_batch(include_str!("../migrations/012.sql"))?;
            }
            if version(&tx)? == 12 {
                tx.execute_batch(include_str!("../migrations/013.sql"))?;
            }
            if inject_failure {
                tx.execute_batch("SELECT * FROM injected_missing_table;")?;
            }
            integrity(&tx)?;
            tx.commit()?;
            Ok(())
        })();
        result.map_err(|mut e| {
            e.details = json!({"backup_path":backup_path});
            e
        })
    }
    pub fn backup(&self, destination: &Path) -> Result<BackupInfo> {
        publish(destination, |stage| {
            let mut target = Connection::open(stage.join(DB_NAME))?;
            target.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
            {
                let backup = Backup::new(&self.conn, &mut target)?;
                let deadline = Instant::now() + Duration::from_millis(BUSY_TIMEOUT_MS);
                loop {
                    if Instant::now() >= deadline {
                        return Err(CoreError::new(
                            ErrorCode::WorkspaceBusy,
                            "El backup excedió el tiempo acotado; reintentá sin tanta contención.",
                        ));
                    }
                    match backup.step(256)? {
                        StepResult::Done => break,
                        StepResult::More => {}
                        StepResult::Busy | StepResult::Locked => {
                            if Instant::now() >= deadline {
                                return Err(CoreError::new(
                                    ErrorCode::WorkspaceBusy,
                                    "No se pudo completar el backup por contención.",
                                ));
                            }
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        _ => {
                            return Err(CoreError::new(
                                ErrorCode::StorageError,
                                "Estado inesperado de backup.",
                            ));
                        }
                    }
                }
            }
            // Publicar una base autosuficiente, sin WAL lateral pendiente.
            target.pragma_update(None, "journal_mode", "DELETE")?;
            integrity(&target)?;
            let (workspace_id, schema_version): (String, u32) =
                target.query_row("SELECT id,schema_version FROM workspace_meta", [], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })?;
            let attachments = if schema_version >= 2 {
                crate::attachments::all(&target)?
            } else {
                vec![]
            };
            for a in &attachments {
                crate::attachments::copy_original(&self.path, stage, a)?;
            }
            drop(target);
            let manifest = Manifest {
                format_version: if schema_version >= 2 { 2 } else { 1 },
                workspace_id,
                schema_version,
                sha256: hash(&stage.join(DB_NAME))?,
                attachments,
            };
            write_sync(
                &stage.join("manifest.json"),
                &serde_json::to_vec_pretty(&manifest)?,
            )?;
            fs::File::open(stage.join(DB_NAME))?.sync_all()?;
            Ok(())
        })?;
        let manifest = validate_backup(destination)?;
        Ok(BackupInfo {
            path: fs::canonicalize(destination)?.to_string_lossy().into(),
            workspace_id: manifest.workspace_id,
            schema_version: manifest.schema_version,
            sha256: manifest.sha256,
        })
    }
    pub fn restore(backup: &Path, destination: &Path) -> Result<WorkspaceInfo> {
        // Validar lo que realmente se va a publicar, no una fuente que podría cambiar.
        publish(destination, |stage| {
            fs::copy(backup.join(DB_NAME), stage.join(DB_NAME)).map_err(|_| {
                CoreError::new(
                    ErrorCode::InvalidBackup,
                    "No se pudo leer la base del backup.",
                )
            })?;
            fs::copy(backup.join("manifest.json"), stage.join("manifest.json")).map_err(|_| {
                CoreError::new(ErrorCode::InvalidBackup, "Falta el manifiesto del backup.")
            })?;
            let manifest: Manifest =
                serde_json::from_slice(&fs::read(stage.join("manifest.json"))?).map_err(|_| {
                    CoreError::new(ErrorCode::InvalidBackup, "Manifiesto inválido.")
                })?;
            for a in &manifest.attachments {
                crate::attachments::copy_original(backup, stage, a).map_err(|_| {
                    CoreError::new(
                        ErrorCode::InvalidBackup,
                        "Original ausente, alterado o inseguro.",
                    )
                })?;
            }
            validate_backup(stage)?;
            fs::remove_file(stage.join("manifest.json"))?;
            fs::File::open(stage.join(DB_NAME))?.sync_all()?;
            Ok(())
        })?;
        Self::open(destination)?.info()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    format_version: u32,
    workspace_id: String,
    schema_version: u32,
    sha256: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    attachments: Vec<Attachment>,
}
fn validate_backup(path: &Path) -> Result<Manifest> {
    let result = (|| -> Result<Manifest> {
        let manifest: Manifest = serde_json::from_slice(&fs::read(path.join("manifest.json"))?)?;
        supported(manifest.schema_version)?;
        if ![1, 2].contains(&manifest.format_version)
            || (manifest.schema_version >= 2 && manifest.format_version != 2)
            || hash(&path.join(DB_NAME))? != manifest.sha256
        {
            return Err(CoreError::validation("Manifiesto o hash inválido."));
        }
        let conn =
            Connection::open_with_flags(path.join(DB_NAME), OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        integrity(&conn)?;
        let workspace_id: String =
            conn.query_row("SELECT id FROM workspace_meta", [], |r| r.get(0))?;
        if version(&conn)? != manifest.schema_version || workspace_id != manifest.workspace_id {
            return Err(CoreError::validation("Identidad inconsistente."));
        }
        let expected = if manifest.schema_version >= 2 {
            crate::attachments::all(&conn)?
        } else {
            vec![]
        };
        if manifest.attachments != expected {
            return Err(CoreError::validation("Lista de originales inconsistente."));
        }
        for a in &expected {
            crate::attachments::verify(path, a)?;
        }
        Ok(manifest)
    })();
    result.map_err(|e| {
        if matches!(e.code, ErrorCode::UnsupportedWorkspaceVersion) {
            e
        } else {
            CoreError::new(
                ErrorCode::InvalidBackup,
                "Backup inválido: integridad, versión o manifiesto no coinciden.",
            )
        }
    })
}
pub(crate) fn hash(path: &Path) -> Result<String> {
    let mut input = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn write_sync(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

// Reserva explícita; sólo se limpian archivos propios. rename publica el
// directorio completo, no una colección de archivos parcialmente copiados.
pub(crate) fn publish(destination: &Path, build: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let created = match fs::create_dir(destination) {
        Ok(()) => true,
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if !destination.is_dir() || fs::read_dir(destination)?.next().is_some() {
                return Err(CoreError::new(
                    ErrorCode::DestinationNotEmpty,
                    "El destino contiene datos; no se sobrescribió.",
                ));
            }
            false
        }
        Err(e) => return Err(e.into()),
    };
    let marker = destination.join(".noteka-reservation");
    if let Err(e) = write_sync(&marker, b"reserved") {
        if created {
            let _ = fs::remove_dir(destination);
        }
        return Err(e);
    }
    let result = (|| -> Result<()> {
        let stage = tempfile::Builder::new()
            .prefix(".noteka-stage-")
            .tempdir_in(parent)?;
        build(stage.path())?;
        fn sync_tree(path: &Path) -> Result<()> {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let p = entry.path();
                if entry.file_type()?.is_dir() {
                    sync_tree(&p)?;
                } else {
                    fs::File::open(&p)?.sync_all()?;
                }
            }
            fs::File::open(path)?.sync_all()?;
            Ok(())
        }
        sync_tree(stage.path())?;
        if fs::read_dir(destination)?.count() != 1 {
            return Err(CoreError::new(
                ErrorCode::DestinationNotEmpty,
                "El destino cambió durante la operación.",
            ));
        }
        fs::remove_file(&marker)?;
        // macOS rename de directorio sobre destino vacío es atómico.
        fs::rename(stage.path(), destination)?;
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&marker);
        if created {
            let _ = fs::remove_dir(destination);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extraction_migration_failure_v7_and_success_preserve_all_unknown_originals() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("v7");
        let mut w = Workspace::create(&path, Some("UTC".into())).unwrap();
        for v in [1, 2, 3, 77] {
            let e = w
                .entity_create(
                    serde_json::from_value(json!({"kind":"note","title":format!("v{v}")})).unwrap(),
                    "test",
                    "seed",
                )
                .unwrap();
            let doc = format!(
                "{{ \"schema_version\":{v}, \"blocks\":[{{\"id\":\"unknown\",\"type\":\"extension\",\"raw\":{{\"quoted\":\"0.00\"}}}}] }}"
            );
            w.conn
                .execute(
                    "UPDATE documents SET schema_version=?2,json=?3 WHERE entity_id=?1",
                    rusqlite::params![e.id, v, doc],
                )
                .unwrap();
            w.conn.execute("UPDATE entities SET payload=' {\"future\":{\"decimal\":\"1.00\"}} ' WHERE id=?1",[&e.id]).unwrap();
        }
        let c = w
            .collection_create(CreateCollection {
                name: "preserve".into(),
            })
            .unwrap()
            .collection
            .id;
        let property=w.property_define(serde_json::from_value(json!({"collection_id":c,"expected_revision":1,"spec":{"name":"Calculated","type":"text"}})).unwrap()).unwrap();
        w.conn.execute("INSERT INTO calculations(property_id,expression) VALUES (?1,' {\"future_ast\":{\"raw\":\"1.00\"}} ')",[&property.id]).unwrap();
        let source = t.path().join("source.txt");
        fs::write(&source, b"unchanged attachment").unwrap();
        let a = w
            .attachment_import(ImportAttachment {
                source_path: source.to_string_lossy().into(),
            })
            .unwrap();
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;UPDATE workspace_meta SET schema_version=7;PRAGMA user_version=7;").unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        let before = crate::extraction_tests::raw(&w.conn);
        let source_hash = hash(&w.path.join("attachments").join(&a.id)).unwrap();
        let e = w.migrate(true).unwrap_err();
        assert_eq!(version(&w.conn).unwrap(), 7);
        assert_eq!(crate::extraction_tests::raw(&w.conn), before);
        let backup = Path::new(e.details["backup_path"].as_str().unwrap());
        assert_eq!(validate_backup(backup).unwrap().schema_version, 7);
        let destination = t.path().join("restored");
        assert_eq!(
            Workspace::restore(backup, &destination)
                .unwrap()
                .schema_version,
            SCHEMA_VERSION
        );
        w.migrate(false).unwrap();
        assert_eq!(version(&w.conn).unwrap(), SCHEMA_VERSION);
        assert_eq!(
            hash(&w.path.join("attachments").join(&a.id)).unwrap(),
            source_hash
        );
        let after = crate::extraction_tests::raw(&w.conn);
        assert_eq!(
            before
                .iter()
                .filter(|(n, _)| n != "workspace_meta")
                .collect::<Vec<_>>(),
            after
                .iter()
                .filter(|(n, _)| n != "workspace_meta" && !n.starts_with("extraction_"))
                .collect::<Vec<_>>()
        );
        assert!(crate::extraction_storage::control(&w.conn).unwrap().paused);
        assert_eq!(
            w.conn
                .query_row("SELECT count(*) FROM extraction_jobs", [], |r| r
                    .get::<_, u32>(0))
                .unwrap(),
            0
        );
        w.conn.execute_batch(&format!("UPDATE workspace_meta SET schema_version={0};PRAGMA user_version={0};PRAGMA wal_checkpoint(TRUNCATE)",SCHEMA_VERSION+1)).unwrap();
        drop(w);
        let bytes = fs::read(path.join(DB_NAME)).unwrap();
        assert!(matches!(
            Workspace::open(&path).err().unwrap().code,
            ErrorCode::UnsupportedWorkspaceVersion
        ));
        assert_eq!(fs::read(path.join(DB_NAME)).unwrap(), bytes);
    }
    #[test]
    fn template_migration_failure_preserves_v6_json_and_originals() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let mut w = Workspace::create(&path, Some("UTC".into())).unwrap();
        let c = w
            .collection_create(CreateCollection {
                name: "Legacy".into(),
            })
            .unwrap()
            .collection
            .id;
        let p = w.property_define(serde_json::from_value(json!({"collection_id":c,"expected_revision":1,"spec":{"name":"Legacy value","type":"text"}})).unwrap()).unwrap();
        for version in [1, 2] {
            w.record_create(serde_json::from_value(json!({"collection_id":c,"title":format!("v{version}"),"values":{p.id.clone():"preserve"},"document":{"schema_version":version,"blocks":[]}})).unwrap(),"test","migration").unwrap();
        }
        w.view_save(
            serde_json::from_value(
                json!({"collection_id":c,"name":"Legacy view","layout":"table","config":{}}),
            )
            .unwrap(),
        )
        .unwrap();
        w.formula_set(serde_json::from_value(json!({"collection_id":c,"expected_revision":2,"name":"Calculation","expression":{"op":"number","value":2}})).unwrap()).unwrap();
        let source = temp.path().join("a.txt");
        fs::write(&source, "immutable").unwrap();
        let original = w
            .attachment_import(ImportAttachment {
                source_path: source.to_string_lossy().into(),
            })
            .unwrap();
        fn raw(conn: &Connection) -> Vec<String> {
            let mut out = vec![];
            for sql in [
                "SELECT json FROM documents ORDER BY entity_id",
                "SELECT json FROM views ORDER BY id",
                "SELECT spec FROM properties ORDER BY id",
                "SELECT expression FROM calculations ORDER BY property_id",
                "SELECT json FROM property_values ORDER BY entity_id,property_id",
                "SELECT text FROM search_fts ORDER BY entity_id,block_id",
            ] {
                out.extend(
                    conn.prepare(sql)
                        .unwrap()
                        .query_map([], |r| r.get::<_, String>(0))
                        .unwrap()
                        .collect::<std::result::Result<Vec<_>, _>>()
                        .unwrap(),
                );
            }
            out
        }
        let before = raw(&w.conn);
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
        w.conn.execute_batch("DROP TABLE templates;UPDATE workspace_meta SET schema_version=6;PRAGMA user_version=6;").unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        let error = w.migrate(true).unwrap_err();
        assert_eq!(version(&w.conn).unwrap(), 6);
        assert_eq!(raw(&w.conn), before);
        assert!(
            !w.conn
                .prepare("SELECT 1 FROM sqlite_master WHERE name='templates'")
                .unwrap()
                .exists([])
                .unwrap()
        );
        let backup = Path::new(error.details["backup_path"].as_str().unwrap());
        assert_eq!(validate_backup(backup).unwrap().schema_version, 6);
        let destination = temp.path().join("restored");
        Workspace::restore(backup, &destination).unwrap();
        let restored = Workspace::open(&destination).unwrap();
        assert_eq!(version(&restored.conn).unwrap(), SCHEMA_VERSION);
        assert_eq!(raw(&restored.conn), before);
        crate::attachments::verify(&destination, &original).unwrap();
        w.migrate(false).unwrap();
        assert_eq!(version(&w.conn).unwrap(), SCHEMA_VERSION);
        assert_eq!(raw(&w.conn), before);
    }
    #[test]
    fn migration_failure_is_atomic_and_has_backup() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let mut w = Workspace::create(&path, None).unwrap();
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
        w.conn.execute_batch("DROP TABLE templates; DROP TABLE calculations; DROP TABLE views; DROP TABLE property_values; DROP TABLE memberships; DROP TABLE properties; DROP TABLE collections; DROP TABLE changes; DROP TABLE attachments; DROP TABLE search_fts; UPDATE workspace_meta SET schema_version=0; PRAGMA user_version=0;").unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        let error = w.migrate(true).unwrap_err();
        assert_eq!(version(&w.conn).unwrap(), 0);
        assert!(
            Path::new(error.details["backup_path"].as_str().unwrap())
                .join("manifest.json")
                .exists()
        );
        w.migrate(false).unwrap();
        assert_eq!(version(&w.conn).unwrap(), SCHEMA_VERSION);
    }
    #[test]
    fn preflight_survives_wal_removed_by_last_connection_checkpoint() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let w = Workspace::create(&path, Some("UTC".into())).unwrap();
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
        w.conn.execute_batch("PRAGMA wal_autocheckpoint=0;DROP TABLE templates;UPDATE workspace_meta SET schema_version=6;PRAGMA user_version=6").unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        assert!(path.join(format!("{DB_NAME}-wal")).exists());
        let mut writer = Some(w);
        let v = preflight_impl(
            &path,
            Some(&mut || {
                drop(writer.take());
            }),
        )
        .unwrap();
        assert_eq!(v, 6);
        assert!(!path.join(format!("{DB_NAME}-wal")).exists());
        assert_eq!(
            Connection::open_with_flags(path.join(DB_NAME), OpenFlags::SQLITE_OPEN_READ_ONLY)
                .unwrap()
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            6
        );
    }
    #[test]
    fn fts_migration_failure_preserves_v2_and_backup() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let mut w = Workspace::create(&path, Some("UTC".into())).unwrap();
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
        w.conn.execute_batch("DROP TABLE templates; DROP TABLE calculations; DROP TABLE views; DROP TABLE property_values; DROP TABLE memberships; DROP TABLE properties; DROP TABLE collections; DROP TABLE search_fts; UPDATE workspace_meta SET schema_version=2; PRAGMA user_version=2;").unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        let error = w.migrate(true).unwrap_err();
        assert_eq!(version(&w.conn).unwrap(), 2);
        assert!(
            !w.conn
                .prepare("SELECT name FROM sqlite_master WHERE name='search_fts'")
                .unwrap()
                .exists([])
                .unwrap()
        );
        let backup = Path::new(error.details["backup_path"].as_str().unwrap());
        assert_eq!(validate_backup(backup).unwrap().schema_version, 2);
        w.migrate(false).unwrap();
        assert_eq!(version(&w.conn).unwrap(), SCHEMA_VERSION);
        assert_eq!(w.search_rebuild().unwrap().entities, 0);
    }
    #[test]
    fn collection_migration_failure_preserves_v3() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let mut w = Workspace::create(&path, Some("UTC".into())).unwrap();
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
        w.conn.execute_batch("DROP TABLE templates;DROP TABLE calculations;DROP TABLE views;DROP TABLE property_values;DROP TABLE memberships;DROP TABLE properties;DROP TABLE collections;UPDATE workspace_meta SET schema_version=3;PRAGMA user_version=3;").unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        let error = w.migrate(true).unwrap_err();
        assert_eq!(version(&w.conn).unwrap(), 3);
        assert_eq!(
            validate_backup(Path::new(error.details["backup_path"].as_str().unwrap()))
                .unwrap()
                .schema_version,
            3
        );
        assert!(
            !w.conn
                .prepare("SELECT name FROM sqlite_master WHERE name='collections'")
                .unwrap()
                .exists([])
                .unwrap()
        );
        w.migrate(false).unwrap();
        assert_eq!(version(&w.conn).unwrap(), SCHEMA_VERSION);
    }
    #[test]
    fn calculation_migration_failure_preserves_v4() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let mut w = Workspace::create(&path, Some("UTC".into())).unwrap();
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
        w.conn.execute_batch("DROP TABLE templates;DROP TABLE calculations;UPDATE workspace_meta SET schema_version=4;PRAGMA user_version=4;").unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        let error = w.migrate(true).unwrap_err();
        assert_eq!(version(&w.conn).unwrap(), 4);
        assert_eq!(
            validate_backup(Path::new(error.details["backup_path"].as_str().unwrap()))
                .unwrap()
                .schema_version,
            4
        );
        assert!(
            !w.conn
                .prepare("SELECT name FROM sqlite_master WHERE name='calculations'")
                .unwrap()
                .exists([])
                .unwrap()
        );
        w.migrate(false).unwrap();
        assert_eq!(version(&w.conn).unwrap(), SCHEMA_VERSION);
    }
    #[test]
    fn view_format_migration_failure_preserves_v5_and_json() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("w");
        let mut w = Workspace::create(&path, Some("UTC".into())).unwrap();
        let c = w
            .collection_create(CreateCollection {
                name: "Legacy".into(),
            })
            .unwrap();
        let raw=serde_json::json!({"id":"legacy","collection_id":c.collection.id,"revision":1,"name":"Table","layout":"table","config":{"columns":[],"filters":[],"sorts":[],"archived":false}}).to_string();
        w.conn
            .execute(
                "INSERT INTO views VALUES (?1,?2,1,?3)",
                rusqlite::params!["legacy", c.collection.id, raw],
            )
            .unwrap();
        w.conn.execute_batch("DROP TABLE extraction_search_entries;DROP TABLE extraction_search_fts;DROP TABLE extraction_search_control;DROP TABLE extraction_pdf_fragments;DROP TABLE extraction_pdf_pages;DROP TABLE extraction_pdf_generations;DROP TABLE extraction_fragments;DROP TABLE extraction_runtime;DROP TABLE extraction_jobs;DROP TABLE extraction_generations;DROP TABLE extraction_control;").unwrap();
        w.conn
            .execute_batch("DROP TABLE templates;UPDATE workspace_meta SET schema_version=5;PRAGMA user_version=5;")
            .unwrap();
        w.conn
            .execute_batch(
                "DROP TABLE extraction_ocr_chunks;DROP TABLE extraction_ocr_generations;DROP TABLE extraction_xlsx_chunks;DROP TABLE extraction_xlsx_generations",
            )
            .unwrap();
        let error = w.migrate(true).unwrap_err();
        assert_eq!(version(&w.conn).unwrap(), 5);
        let backup = Path::new(error.details["backup_path"].as_str().unwrap());
        assert_eq!(validate_backup(backup).unwrap().schema_version, 5);
        let restored_path = temp.path().join("restored");
        Workspace::restore(backup, &restored_path).unwrap();
        let restored = Workspace::open(&restored_path).unwrap();
        assert_eq!(version(&restored.conn).unwrap(), SCHEMA_VERSION);
        assert_eq!(
            restored
                .conn
                .query_row("SELECT json FROM views WHERE id='legacy'", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            raw
        );
        w.migrate(false).unwrap();
        assert_eq!(version(&w.conn).unwrap(), SCHEMA_VERSION);
        assert_eq!(
            w.conn
                .query_row("SELECT json FROM views WHERE id='legacy'", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            raw
        );
        assert_eq!(w.collection_get(&c.collection.id).unwrap().views.len(), 1);
    }
}
