use crate::{
    batch_plan::invalid,
    error::*,
    workspace::{self, BUSY_TIMEOUT_MS, DB_NAME, SCHEMA_VERSION},
};
use rusqlite::{
    Connection, OpenFlags,
    backup::{Backup, StepResult},
    types::ValueRef,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
    time::{Duration, Instant},
};
pub(crate) const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;
// Contenido original. El registry de schema incluye TODAS las tablas de 001–011,
// también las operativas/derivadas, independientemente de esta huella.
pub(crate) const ORIGINAL_TABLES: [(&str, &str); 14] = [
    ("workspace_meta", "id"),
    ("entities", "id"),
    ("documents", "entity_id"),
    ("relations", "id"),
    ("changes", "id"),
    ("attachments", "id"),
    ("collections", "id"),
    ("properties", "id"),
    ("memberships", "collection_id,entity_id"),
    ("property_values", "entity_id,property_id"),
    ("views", "id"),
    ("calculations", "property_id"),
    ("templates", "id"),
    ("extraction_control", "id"),
];
type SchemaRow = (String, String, String, Option<String>);
fn schema(conn: &Connection) -> Result<Vec<SchemaRow>> {
    Ok(conn.prepare("SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type COLLATE BINARY,name COLLATE BINARY")?
        .query_map([],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?.collect::<std::result::Result<_,_>>()?)
}
pub(crate) fn current(conn: &Connection) -> Result<()> {
    let version = workspace::version(conn)?;
    if version != SCHEMA_VERSION {
        let mut e = CoreError::new(
            ErrorCode::UnsupportedWorkspaceVersion,
            "Migrá explícitamente el workspace antes de previsualizar cambios.",
        );
        e.details = json!({"current_version":version,"supported_version":SCHEMA_VERSION,"migration_required":true});
        return Err(e);
    }
    Ok(())
}
pub(crate) fn structure(conn: &Connection) -> Result<()> {
    current(conn)?;
    static EXPECTED: OnceLock<Vec<SchemaRow>> = OnceLock::new();
    let expected = EXPECTED.get_or_init(|| {
        let c = Connection::open_in_memory().expect("SQLite en memoria");
        for sql in [
            include_str!("../migrations/001.sql"),
            include_str!("../migrations/002.sql"),
            include_str!("../migrations/003.sql"),
            include_str!("../migrations/004.sql"),
            include_str!("../migrations/005.sql"),
            include_str!("../migrations/006.sql"),
            include_str!("../migrations/007.sql"),
            include_str!("../migrations/008.sql"),
            include_str!("../migrations/009.sql"),
            include_str!("../migrations/010.sql"),
            include_str!("../migrations/011.sql"),
            include_str!("../migrations/012.sql"),
            include_str!("../migrations/013.sql"),
        ] {
            c.execute_batch(sql)
                .expect("Migraciones incorporadas válidas");
        }
        schema(&c).expect("Esquema incorporado válido")
    });
    let actual = schema(conn)?;
    let equivalent = |a: &(String, String, String, Option<String>),
                      b: &(String, String, String, Option<String>)| {
        a.0 == b.0
            && a.1 == b.1
            && a.2 == b.2
            && a.3.as_ref().map(|s| sql_signature(s)) == b.3.as_ref().map(|s| sql_signature(s))
    };
    // Tablas, virtual/shadows y triggers exactos; índices ordinarios adicionales sobre
    // tablas originales son compatibles y quedan vinculados por base_hash.
    let allowed_index = |row: &(String, String, String, Option<String>)| {
        row.0 == "index" && ORIGINAL_TABLES.iter().any(|(t, _)| *t == row.2)
    };
    if actual
        .iter()
        .any(|row| !expected.iter().any(|e| equivalent(row, e)) && !allowed_index(row))
        || expected
            .iter()
            .filter(|r| r.0 != "index")
            .any(|r| !actual.iter().any(|a| equivalent(a, r)))
    {
        let mut e = CoreError::new(
            ErrorCode::UnsupportedWorkspaceVersion,
            "El esquema SQL no es compatible con lotes; no se modificó ni reparó.",
        );
        e.details = json!({"reason":"unsupported_batch_schema"});
        return Err(e);
    }
    Ok(())
}
fn sql_signature(sql: &str) -> String {
    // Sólo ignora whitespace fuera de literales/identificadores citados. No
    // normaliza texto persistente, CHECK, tokens ni mayúsculas de nombres.
    let mut out = vec![];
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_whitespace() {
            continue;
        }
        let mut token = c.to_string();
        if ['\'', '"', '`', '['].contains(&c) {
            let q = if c == '[' { ']' } else { c };
            while let Some(next) = chars.next() {
                token.push(next);
                if next == q {
                    if chars.peek() == Some(&q) {
                        token.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
            }
        } else if c.is_alphanumeric() || c == '_' {
            while chars
                .peek()
                .is_some_and(|c| c.is_alphanumeric() || *c == '_')
            {
                token.push(chars.next().unwrap());
            }
        }
        out.push(token);
    }
    out.join("\0")
}
pub(crate) fn open(path: &Path, write: bool) -> Result<(Connection, PathBuf)> {
    let version = workspace::preflight(path)?;
    if version != SCHEMA_VERSION {
        let mut e = CoreError::new(
            ErrorCode::UnsupportedWorkspaceVersion,
            "Se requiere migración explícita.",
        );
        e.details = json!({"current_version":version,"supported_version":SCHEMA_VERSION,"migration_required":true});
        return Err(e);
    }
    let root = std::fs::canonicalize(path)?;
    let c = Connection::open_with_flags(
        root.join(DB_NAME),
        if write {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        },
    )?;
    c.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
    if write {
        c.pragma_update(None, "foreign_keys", true)?;
        c.pragma_update(None, "synchronous", "FULL")?;
    } else {
        c.pragma_update(None, "query_only", true)?;
    }
    Ok((c, root))
}
pub(crate) fn size(conn: &Connection) -> Result<u64> {
    let pages: u32 = conn.pragma_query_value(None, "page_count", |r| r.get(0))?;
    let page_size: u32 = conn.pragma_query_value(None, "page_size", |r| r.get(0))?;
    let bytes = u64::from(pages)
        .checked_mul(u64::from(page_size))
        .ok_or_else(|| invalid("snapshot_size_overflow"))?;
    if bytes > MAX_SNAPSHOT_BYTES {
        return Err(invalid("snapshot_size_limit"));
    }
    Ok(u64::from(page_size))
}
pub(crate) fn snapshot(source: &Connection) -> Result<Connection> {
    let tx = source.unchecked_transaction()?;
    current(&tx)?; // Fija snapshot antes de tamaño/Backup; incluye commits en WAL.
    let page_size = size(&tx)?;
    let mut target = Connection::open_in_memory()?;
    target.pragma_update(None, "page_size", page_size as i64)?;
    copy(
        &tx,
        &mut target,
        Instant::now() + Duration::from_millis(BUSY_TIMEOUT_MS),
    )?;
    drop(tx);
    structure(&target)?;
    target.pragma_update(None, "foreign_keys", true)?;
    Ok(target)
}
pub(crate) fn copy(source: &Connection, target: &mut Connection, deadline: Instant) -> Result<()> {
    let backup = Backup::new(source, target)?;
    loop {
        if Instant::now() >= deadline {
            return Err(busy("backup_deadline"));
        }
        match backup.step(256)? {
            StepResult::Done => {
                if Instant::now() >= deadline {
                    return Err(busy("backup_deadline"));
                }
                return Ok(());
            }
            StepResult::More => {}
            StepResult::Busy | StepResult::Locked => std::thread::sleep(Duration::from_millis(10)),
            _ => {
                return Err(CoreError::new(
                    ErrorCode::StorageError,
                    "Estado inesperado de Backup.",
                ));
            }
        }
    }
}
pub(crate) fn busy(reason: &str) -> CoreError {
    let mut e = CoreError::new(
        ErrorCode::WorkspaceBusy,
        "Se agotó el tiempo acotado del lote.",
    );
    e.details = json!({"reason":reason});
    e
}
pub(crate) struct Budget<'a> {
    conn: &'a Connection,
    deadline: Instant,
}
impl<'a> Budget<'a> {
    pub fn new(conn: &'a Connection) -> Result<Self> {
        Self::with_duration(conn, Duration::from_secs(5))
    }
    pub fn with_duration(conn: &'a Connection, duration: Duration) -> Result<Self> {
        let deadline = Instant::now() + duration;
        conn.progress_handler(1000, Some(move || Instant::now() >= deadline))?;
        Ok(Self { conn, deadline })
    }
    pub fn check(&self) -> Result<()> {
        if Instant::now() >= self.deadline {
            Err(busy("execution_deadline"))
        } else {
            Ok(())
        }
    }
    pub fn remaining_ms(&self) -> Result<u32> {
        self.check()?;
        Ok(self
            .deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .clamp(1, 5000) as u32)
    }
    pub fn map(&self, mut e: CoreError) -> CoreError {
        if e.details["sqlite_interrupt"] == true && Instant::now() >= self.deadline {
            e.code = ErrorCode::WorkspaceBusy;
            e.details["reason"] = json!("execution_deadline");
        }
        e
    }
}
impl Drop for Budget<'_> {
    fn drop(&mut self) {
        let _ = self.conn.progress_handler(0, None::<fn() -> bool>);
    }
}
fn part(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_be_bytes());
    h.update(bytes);
}
fn cell(h: &mut Sha256, v: ValueRef<'_>) {
    match v {
        ValueRef::Null => part(h, b"null"),
        ValueRef::Integer(i) => {
            part(h, b"integer");
            part(h, &i.to_be_bytes());
        }
        ValueRef::Real(f) => {
            part(h, b"real");
            part(h, &f.to_bits().to_be_bytes());
        }
        ValueRef::Text(t) => {
            part(h, b"text");
            part(h, t);
        }
        ValueRef::Blob(b) => {
            part(h, b"blob");
            part(h, b);
        }
    }
}
pub(crate) fn base_hash(conn: &Connection, budget: &Budget<'_>) -> Result<String> {
    let mut h = Sha256::new();
    part(&mut h, b"noteka.batch.base.v2");
    for row in schema(conn)? {
        budget.check()?;
        part(&mut h, row.0.as_bytes());
        part(&mut h, row.1.as_bytes());
        part(&mut h, row.2.as_bytes());
        cell(
            &mut h,
            match &row.3 {
                Some(s) => ValueRef::Text(s.as_bytes()),
                None => ValueRef::Null,
            },
        );
    }
    for (table, keys) in ORIGINAL_TABLES {
        budget.check()?;
        part(&mut h, table.as_bytes());
        let order = keys
            .split(',')
            .map(|k| format!("{k} COLLATE BINARY"))
            .collect::<Vec<_>>()
            .join(",");
        let mut stmt = conn.prepare(&format!("SELECT * FROM {table} ORDER BY {order}"))?;
        let names = stmt
            .column_names()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        for name in &names {
            part(&mut h, name.as_bytes());
        }
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            budget.check()?;
            part(&mut h, b"row");
            for i in 0..names.len() {
                cell(&mut h, row.get_ref(i)?);
            }
        }
        part(&mut h, b"end-table");
    }
    Ok(format!("{:x}", h.finalize()))
}
fn canonical(v: &Value) -> Value {
    match v {
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, v)| (k.clone(), canonical(v)))
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(canonical).collect()),
        _ => v.clone(),
    }
}
pub(crate) fn json_hash(namespace: &str, v: &Value) -> Result<String> {
    let mut h = Sha256::new();
    part(&mut h, namespace.as_bytes());
    part(&mut h, &serde_json::to_vec(&canonical(v))?);
    Ok(format!("{:x}", h.finalize()))
}
pub(crate) fn path_hash(path: &Path) -> String {
    let mut h = Sha256::new();
    part(&mut h, b"noteka.batch.path.v1");
    part(&mut h, path.as_os_str().as_encoded_bytes());
    format!("{:x}", h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_sqlite_cells_keep_null_empty_and_types_distinct() {
        let conn = Connection::open_in_memory().unwrap();
        let hashes = conn
            .query_row(
                "SELECT NULL,'',0,'0',x'',CAST(x'ff' AS TEXT),x'ff',1.0",
                [],
                |r| {
                    (0..8)
                        .map(|i| {
                            let mut h = Sha256::new();
                            cell(&mut h, r.get_ref(i)?);
                            Ok(format!("{:x}", h.finalize()))
                        })
                        .collect::<rusqlite::Result<std::collections::BTreeSet<_>>>()
                },
            )
            .unwrap();
        assert_eq!(hashes.len(), 8);
    }
}
