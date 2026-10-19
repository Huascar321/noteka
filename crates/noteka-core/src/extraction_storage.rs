use crate::{
    attachments, batch_storage,
    contracts::*,
    error::*,
    workspace::{self, Workspace},
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Transaction, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

pub(crate) fn error(code: ErrorCode, reason: &str) -> CoreError {
    let mut e = CoreError::new(
        code,
        "No se pudo completar la operación de extracción local.",
    );
    e.details = json!({"reason":reason});
    e
}
pub(crate) fn invalid(reason: &str) -> CoreError {
    error(ErrorCode::ValidationError, reason)
}
pub(crate) fn storage(reason: &str) -> CoreError {
    error(ErrorCode::StorageError, reason)
}
pub(crate) fn uuid(value: &str) -> Result<()> {
    if uuid::Uuid::parse_str(value).is_err()
        || uuid::Uuid::parse_str(value).unwrap().to_string() != value
    {
        return Err(invalid("invalid_id"));
    }
    Ok(())
}
fn digest_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub(crate) fn bump(value: u32) -> Result<u32> {
    value
        .checked_add(1)
        .ok_or_else(|| invalid("counter_overflow"))
}
pub(crate) fn expected(actual: u32, revision: u32) -> Result<()> {
    if actual != revision {
        let mut e = error(ErrorCode::RevisionConflict, "expected_revision_mismatch");
        e.details["current_revision"] = json!(actual);
        return Err(e);
    }
    Ok(())
}
pub(crate) fn immediate(conn: &Connection) -> Result<Transaction<'_>> {
    Ok(Transaction::new_unchecked(
        conn,
        TransactionBehavior::Immediate,
    )?)
}
pub(crate) fn response<T: Serialize>(data: &T) -> Result<()> {
    let bytes = serde_json::to_vec(
        &json!({"api_version":1,"request_id":"00000000-0000-0000-0000-000000000000","ok":true,"data":data}),
    )?;
    if bytes.len() > EXTRACTION_RESPONSE_BYTES {
        return Err(invalid("response_size_limit"));
    }
    Ok(())
}
pub(crate) fn config_hash(value: &Value) -> Result<String> {
    batch_storage::json_hash("noteka.extraction.config.v1", value)
}
pub(crate) fn supported(extractor: &str, version: u32, config: &Value) -> bool {
    matches!(
        extractor,
        "utf8_text" | "pdf_text" | "xlsx_cells" | "image_ocr"
    ) && version == 1
        && serde_json::from_value::<ExtractionConfig>(config.clone())
            .is_ok_and(|c| c == ExtractionConfig::default())
}
pub(crate) fn scope(conn: &Connection, root: &Path) -> Result<String> {
    let id: String = conn.query_row("SELECT id FROM workspace_meta", [], |r| r.get(0))?;
    let mut h = Sha256::new();
    for bytes in [
        b"noteka.extraction.scope.v1".as_slice(),
        id.as_bytes(),
        fs::canonicalize(root)?.as_os_str().as_encoded_bytes(),
    ] {
        h.update((bytes.len() as u64).to_be_bytes());
        h.update(bytes);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub(crate) fn control(conn: &Connection) -> Result<ExtractionControl> {
    let c = conn.query_row(
        "SELECT version,paused,revision,updated_at FROM extraction_control WHERE id=1",
        [],
        |r| {
            Ok(ExtractionControl {
                version: r.get(0)?,
                paused: r.get(1)?,
                revision: r.get(2)?,
                updated_at: r.get(3)?,
            })
        },
    )?;
    if c.version != 1 {
        return Err(invalid("unsupported_control_version"));
    }
    if c.revision == 0 || DateTime::parse_from_rfc3339(&c.updated_at).is_err() {
        return Err(storage("invalid_control"));
    }
    Ok(c)
}
#[derive(Debug, Clone)]
pub(crate) struct Owner {
    pub job_id: String,
    pub token: String,
    pub scope: String,
    pub control_revision: u32,
    pub lease_until: String,
}
fn lease_now() -> DateTime<Utc> {
    #[cfg(test)]
    if let Some(now) = TEST_TIME.with(|v| *v.borrow()) {
        return now;
    }
    Utc::now()
}
#[cfg(test)]
thread_local! {static TEST_TIME: std::cell::RefCell<Option<DateTime<Utc>>>=const {std::cell::RefCell::new(None)};}
#[cfg(test)]
pub(crate) struct TestTime(Option<DateTime<Utc>>);
#[cfg(test)]
pub(crate) fn test_time(now: DateTime<Utc>) -> TestTime {
    TestTime(TEST_TIME.with(|v| v.replace(Some(now))))
}
#[cfg(test)]
impl Drop for TestTime {
    fn drop(&mut self) {
        TEST_TIME.with(|v| v.replace(self.0));
    }
}
impl Owner {
    pub fn live(&self, current_scope: &str) -> Result<bool> {
        Ok(self.scope == current_scope
            && DateTime::parse_from_rfc3339(&self.lease_until)
                .map_err(|_| storage("invalid_lease"))?
                .with_timezone(&Utc)
                > lease_now())
    }
}
pub(crate) fn owner(conn: &Connection) -> Result<Option<Owner>> {
    let o = conn.query_row(
        "SELECT job_id,token,scope,control_revision,lease_until FROM extraction_runtime WHERE id=1",
        [],
        |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<u32>>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        },
    )?;
    match o {
        (None, None, None, None, None) => Ok(None),
        (Some(job_id), Some(token), Some(scope), Some(control_revision), Some(lease_until))
            if control_revision > 0 =>
        {
            uuid(&job_id).map_err(|_| storage("invalid_owner"))?;
            uuid(&token).map_err(|_| storage("invalid_owner"))?;
            if !digest_valid(&scope) || DateTime::parse_from_rfc3339(&lease_until).is_err() {
                return Err(storage("invalid_owner"));
            }
            Ok(Some(Owner {
                job_id,
                token,
                scope,
                control_revision,
                lease_until,
            }))
        }
        _ => Err(storage("invalid_owner")),
    }
}
pub(crate) fn release(conn: &Connection) -> Result<()> {
    conn.execute("UPDATE extraction_runtime SET job_id=NULL,token=NULL,scope=NULL,control_revision=NULL,lease_until=NULL WHERE id=1",[])?;
    Ok(())
}
pub(crate) fn generation(conn: &Connection, id: &str) -> Result<ExtractionGeneration> {
    let (mut g,raw): (ExtractionGeneration,String) = conn.query_row("SELECT id,attachment_id,source_sha256,extractor,extractor_version,config,config_hash,source_bytes,covered_bytes,fragment_count,published_at,attempt FROM extraction_generations WHERE id=?1",[id],|r| {
        Ok((ExtractionGeneration {id:r.get(0)?,attachment_id:r.get(1)?,source_sha256:r.get(2)?,extractor:r.get(3)?,extractor_version:r.get(4)?,config:Value::Null,config_hash:r.get(6)?,source_bytes:r.get(7)?,covered_bytes:r.get(8)?,fragment_count:r.get(9)?,complete:true,published_at:r.get(10)?,attempt:r.get(11)?},r.get(5)?))
    })?;
    g.config = serde_json::from_str(&raw).map_err(|_| storage("invalid_generation_config"))?;
    if raw.len() > 8192
        || config_hash(&g.config)? != g.config_hash
        || g.source_bytes > MAX_EXTRACTION_BYTES
        || g.covered_bytes != g.source_bytes
        || g.fragment_count > MAX_EXTRACTION_FRAGMENTS
        || g.extractor_version == 0
        || g.attempt == 0
        || uuid(&g.id).is_err()
        || uuid(&g.attachment_id).is_err()
        || !digest_valid(&g.source_sha256)
        || DateTime::parse_from_rfc3339(&g.published_at).is_err()
    {
        return Err(storage("invalid_generation"));
    }
    Ok(g)
}
pub(crate) fn job(conn: &Connection, root: &Path, id: &str) -> Result<ExtractionJob> {
    let (mut j, raw, err, gen_id): (ExtractionJob,String,Option<String>,Option<String>)=conn.query_row("SELECT id,attachment_id,source_sha256,source_bytes,extractor,extractor_version,config,config_hash,state,revision,attempts,created_at,updated_at,started_at,finished_at,error,generation_id FROM extraction_jobs WHERE id=?1",[id],|r| {
        let state:String=r.get(8)?;
        let state=match state.as_str() { "queued"=>ExtractionState::Queued,"running"=>ExtractionState::Running,"succeeded"=>ExtractionState::Succeeded,"failed"=>ExtractionState::Failed,"cancelled"=>ExtractionState::Cancelled,"stale"=>ExtractionState::Stale,_=>return Err(rusqlite::Error::InvalidQuery) };
        Ok((ExtractionJob{id:r.get(0)?,attachment_id:r.get(1)?,source_sha256:r.get(2)?,source_bytes:r.get(3)?,extractor:r.get(4)?,extractor_version:r.get(5)?,config:Value::Null,config_hash:r.get(7)?,supported:false,unsupported_reason:None,state,revision:r.get(9)?,attempts:r.get(10)?,created_at:r.get(11)?,updated_at:r.get(12)?,started_at:r.get(13)?,finished_at:r.get(14)?,error:None,ownership_active:false,recoverable:false,generation:None,result_from_previous_attempt:false},r.get(6)?,r.get(15)?,r.get(16)?))
    })?;
    j.config = serde_json::from_str(&raw).map_err(|_| storage("invalid_job_config"))?;
    if raw.len() > 8192
        || config_hash(&j.config)? != j.config_hash
        || j.revision == 0
        || j.extractor_version == 0
        || !digest_valid(&j.source_sha256)
        || j.source_bytes > MAX_EXTRACTION_BYTES
        || DateTime::parse_from_rfc3339(&j.created_at).is_err()
        || DateTime::parse_from_rfc3339(&j.updated_at).is_err()
        || [&j.started_at, &j.finished_at].iter().any(|v| {
            v.as_ref()
                .is_some_and(|d| DateTime::parse_from_rfc3339(d).is_err())
        })
    {
        return Err(storage("invalid_job"));
    }
    uuid(&j.id).map_err(|_| storage("invalid_job_id"))?;
    uuid(&j.attachment_id).map_err(|_| storage("invalid_job_source"))?;
    j.supported = supported(&j.extractor, j.extractor_version, &j.config);
    j.unsupported_reason = (!j.supported).then(|| "unsupported_extractor_config".into());
    if let Some(raw) = err {
        if raw.len() > 4096 {
            return Err(storage("invalid_job_error"));
        }
        let e: CoreError = serde_json::from_str(&raw).map_err(|_| storage("invalid_job_error"))?;
        if e.message.chars().count() > 512 {
            return Err(storage("invalid_job_error"));
        }
        j.error = Some(e);
    }
    if let Some(id) = gen_id {
        j.generation = Some(generation(conn, &id)?);
    }
    if (j.state == ExtractionState::Succeeded && j.generation.is_none())
        || j.generation.as_ref().is_some_and(|g| {
            g.attachment_id != j.attachment_id
                || g.source_sha256 != j.source_sha256
                || g.source_bytes != j.source_bytes
                || g.config_hash != j.config_hash
                || g.extractor != j.extractor
                || g.extractor_version != j.extractor_version
        })
    {
        return Err(storage("invalid_job_generation"));
    }
    j.result_from_previous_attempt = j
        .generation
        .as_ref()
        .is_some_and(|g| j.state != ExtractionState::Succeeded || g.attempt != j.attempts);
    if j.state == ExtractionState::Running {
        let own = owner(conn)?.ok_or_else(|| storage("running_without_owner"))?;
        if own.job_id != j.id {
            return Err(storage("running_without_owner"));
        }
        let c = control(conn)?;
        j.ownership_active =
            own.live(&scope(conn, root)?)? && own.control_revision == c.revision && !c.paused;
        j.recoverable = !j.ownership_active;
    }
    Ok(j)
}
pub(crate) fn admit(conn: &Connection, new_job: bool) -> Result<()> {
    let (total, pending): (u32, u32) = conn.query_row(
        "SELECT count(*),coalesce(sum(state IN ('queued','running')),0) FROM extraction_jobs",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if new_job && total >= MAX_EXTRACTION_JOBS {
        return Err(invalid("job_count_limit"));
    }
    if pending >= MAX_EXTRACTION_PENDING {
        return Err(invalid("pending_count_limit"));
    }
    Ok(())
}
pub(crate) fn eligible(a: &Attachment) -> Result<()> {
    eligible_for(a, "utf8_text")
}
pub(crate) fn eligible_for(a: &Attachment, extractor: &str) -> Result<()> {
    if a.bytes > u64::from(MAX_EXTRACTION_BYTES) {
        return Err(invalid("source_size_limit"));
    }
    let extension = Path::new(&a.name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let valid = match extractor {
        "utf8_text" => {
            matches!(extension.as_str(), "txt" | "md")
                && matches!(
                    a.mime.as_str(),
                    "application/octet-stream" | "text/plain" | "text/markdown"
                )
        }
        "pdf_text" => {
            extension == "pdf"
                && matches!(
                    a.mime.as_str(),
                    "application/octet-stream" | "application/pdf"
                )
        }
        "xlsx_cells" => {
            extension == "xlsx"
                && matches!(
                    a.mime.as_str(),
                    "application/octet-stream"
                        | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                )
        }
        "image_ocr" => {
            (extension == "png"
                && matches!(a.mime.as_str(), "application/octet-stream" | "image/png"))
                || (matches!(extension.as_str(), "jpg" | "jpeg")
                    && matches!(a.mime.as_str(), "application/octet-stream" | "image/jpeg"))
        }
        _ => false,
    };
    if !valid {
        return Err(invalid("unsupported_source_format"));
    }
    Ok(())
}
pub(crate) fn source(
    root: &Path,
    a: &Attachment,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Vec<u8>> {
    if a.bytes > u64::from(MAX_EXTRACTION_BYTES) {
        return Err(invalid("source_size_limit"));
    }
    let dir = root.join("attachments");
    let path = dir.join(&a.id);
    uuid(&a.id)?;
    let safe = fs::symlink_metadata(&dir)
        .map(|m| m.file_type().is_dir())
        .and_then(|regular| {
            fs::symlink_metadata(&path)
                .map(|m| regular && m.file_type().is_file() && m.len() == a.bytes)
        });
    match safe {
        Ok(true) => {}
        Ok(false) => return Err(storage("input_stale")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(storage("input_stale")),
        Err(e) => return Err(e.into()),
    }
    let f = fs::File::open(&path)?;
    if !f.metadata()?.is_file() || f.metadata()?.len() != a.bytes {
        return Err(storage("input_stale"));
    }
    // Incluso si el archivo crece después de metadata, leer como máximo límite+1.
    let mut f = f.take(u64::from(MAX_EXTRACTION_BYTES) + 1);
    let mut bytes = Vec::with_capacity(a.bytes as usize);
    let mut buffer = [0; 65536];
    loop {
        check()?;
        let n = f.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        if bytes.len() + n > MAX_EXTRACTION_BYTES as usize {
            return Err(storage("input_stale"));
        }
        bytes.extend_from_slice(&buffer[..n]);
    }
    check()?;
    if bytes.len() as u64 != a.bytes || format!("{:x}", Sha256::digest(&bytes)) != a.sha256 {
        return Err(storage("input_stale"));
    }
    // Rechazar sustitución estática/de ruta durante IO, no prometer sandbox contra otro proceso privilegiado.
    if !fs::symlink_metadata(&dir)?.file_type().is_dir()
        || !fs::symlink_metadata(&path)?.file_type().is_file()
    {
        return Err(storage("input_stale"));
    }
    Ok(bytes)
}
pub(crate) fn job_source(conn: &Connection, j: &ExtractionJob) -> Result<Attachment> {
    let a = attachments::metadata(conn, &j.attachment_id)?;
    if a.sha256 != j.source_sha256 || a.bytes != u64::from(j.source_bytes) {
        return Err(storage("input_stale"));
    }
    Ok(a)
}
pub(crate) fn fail(
    conn: &Connection,
    id: &str,
    state: ExtractionState,
    e: &CoreError,
) -> Result<()> {
    let rev: u32 = conn.query_row(
        "SELECT revision FROM extraction_jobs WHERE id=?1",
        [id],
        |r| r.get(0),
    )?;
    let raw = serde_json::to_string(e)?;
    if raw.len() > 4096 || e.message.chars().count() > 512 {
        return Err(storage("error_size_limit"));
    }
    conn.execute("UPDATE extraction_jobs SET state=?2,revision=?3,updated_at=?4,finished_at=?4,error=?5 WHERE id=?1",
        params![id,state.as_str(),bump(rev)?,workspace::now(),raw])?;
    release(conn)
}
pub(crate) fn recover(conn: &Connection, root: &Path) -> Result<Option<String>> {
    if let Some(o) = owner(conn)? {
        let current = scope(conn, root)?;
        let c = control(conn)?;
        if !o.live(&current)? || o.control_revision != c.revision || c.paused {
            let reason = if o.scope != current {
                "scope_mismatch"
            } else {
                "worker_interrupted"
            };
            fail(
                conn,
                &o.job_id,
                ExtractionState::Failed,
                &error(ErrorCode::WorkspaceBusy, reason),
            )?;
            return Ok(Some(o.job_id));
        }
    }
    Ok(None)
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cursor {
    pub version: u32,
    pub scope: String,
    pub operation: String,
    pub attachment_id: Option<String>,
    pub state: Option<ExtractionState>,
    pub limit: u32,
    pub after_created: Option<String>,
    pub after_id: Option<String>,
    pub job_id: Option<String>,
    pub generation_id: Option<String>,
    pub ordinal: Option<u32>,
}
impl Cursor {
    pub fn new(w: &Workspace, operation: &str, limit: u32) -> Result<Self> {
        Ok(Self {
            version: 1,
            scope: scope(&w.conn, &w.path)?,
            operation: operation.into(),
            attachment_id: None,
            state: None,
            limit,
            after_created: None,
            after_id: None,
            job_id: None,
            generation_id: None,
            ordinal: None,
        })
    }
    pub fn read(raw: &str) -> Result<Self> {
        if raw.len() > 2048 {
            return Err(invalid("cursor_size_limit"));
        }
        let c: Self = serde_json::from_str(raw).map_err(|_| invalid("invalid_cursor"))?;
        if c.version != 1 {
            return Err(invalid("unsupported_cursor_version"));
        }
        Ok(c)
    }
    pub fn encode(&self) -> Result<String> {
        let s = serde_json::to_string(self)?;
        if s.len() > 2048 {
            return Err(invalid("cursor_size_limit"));
        }
        Ok(s)
    }
}
pub(crate) fn page_limit(n: Option<u32>, default: u32, max: u32) -> Result<u32> {
    let n = n.unwrap_or(default);
    if n == 0 || n > max {
        return Err(invalid("page_limit"));
    }
    Ok(n)
}
/// Sólo verificación de integridad: nunca recupera ni descarta jobs futuros.
pub(crate) fn integrity(conn: &Connection) -> Result<()> {
    queue_integrity(conn)?;
    if crate::workspace::version(conn)? >= 9 {
        crate::pdf_storage::integrity(conn)?;
    }
    if crate::workspace::version(conn)? >= 11 {
        crate::xlsx_storage::integrity(conn)?;
    }
    if crate::workspace::version(conn)? >= 13 {
        crate::ocr_storage::integrity(conn)?;
    }
    Ok(())
}
/// Estado/ownership/configuración actuales bajo claim. Reconstruir/hash de
/// todo texto PDF se valida en snapshot read-only antes de tomar el escritor.
pub(crate) fn queue_integrity(conn: &Connection) -> Result<()> {
    let counts:(u32,u32)=conn.query_row("SELECT (SELECT count(*) FROM extraction_control),(SELECT count(*) FROM extraction_runtime)",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
    if counts != (1, 1) {
        return Err(storage("invalid_extraction_singletons"));
    }
    let invalid=conn.prepare("SELECT 1 FROM extraction_runtime r LEFT JOIN extraction_jobs j ON j.id=r.job_id WHERE r.job_id IS NOT NULL AND (j.id IS NULL OR j.state!='running') UNION ALL SELECT 1 FROM extraction_jobs j LEFT JOIN extraction_runtime r ON r.job_id=j.id WHERE j.state='running' AND r.job_id IS NULL UNION ALL SELECT 1 FROM extraction_generations g LEFT JOIN extraction_jobs j ON j.id=g.job_id WHERE j.generation_id IS NOT g.id OR j.attachment_id!=g.attachment_id OR j.source_sha256!=g.source_sha256 OR j.config_hash!=g.config_hash OR j.source_bytes!=g.source_bytes OR (g.extractor NOT IN ('pdf_text','xlsx_cells','image_ocr') AND (g.fragment_count!=(SELECT count(*) FROM extraction_fragments f WHERE f.generation_id=g.id) OR g.covered_bytes!=coalesce((SELECT sum(byte_end-byte_start) FROM extraction_fragments f WHERE f.generation_id=g.id),0))) OR (g.extractor IN ('pdf_text','xlsx_cells','image_ocr') AND EXISTS(SELECT 1 FROM extraction_fragments f WHERE f.generation_id=g.id)) UNION ALL SELECT 1 FROM (SELECT ordinal,byte_start,coalesce(lag(byte_end) OVER(PARTITION BY generation_id ORDER BY ordinal),0) AS previous_end, row_number() OVER(PARTITION BY generation_id ORDER BY ordinal)-1 AS expected_ordinal FROM extraction_fragments) WHERE byte_start!=previous_end OR ordinal!=expected_ordinal LIMIT 1")?.exists([])?;
    if invalid {
        return Err(storage("invalid_extraction_integrity"));
    }
    let inconsistent=conn.prepare("SELECT 1 FROM extraction_jobs j LEFT JOIN extraction_generations g ON g.id=j.generation_id WHERE (j.state='succeeded' AND j.generation_id IS NULL) OR (j.generation_id IS NOT NULL AND (g.id IS NULL OR g.job_id!=j.id OR g.extractor!=j.extractor OR g.extractor_version!=j.extractor_version OR g.config_hash!=j.config_hash OR g.attachment_id!=j.attachment_id OR g.source_sha256!=j.source_sha256 OR g.source_bytes!=j.source_bytes OR g.attempt>j.attempts OR (j.state='succeeded' AND g.attempt!=j.attempts))) LIMIT 1")?.exists([])?;
    if inconsistent {
        return Err(storage("invalid_job_generation"));
    }
    owner(conn)?;
    let mut stmt=conn.prepare("SELECT id,attachment_id,source_sha256,source_bytes,extractor_version,config,config_hash,revision,attempts,created_at,updated_at,started_at,finished_at,error FROM extraction_jobs")?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let id: String = r.get(0)?;
        let attachment: String = r.get(1)?;
        let sha: String = r.get(2)?;
        let bytes: u32 = r.get(3)?;
        let version: u32 = r.get(4)?;
        let config: String = r.get(5)?;
        let hash: String = r.get(6)?;
        let revision: u32 = r.get(7)?;
        let _: u32 = r.get(8)?;
        if uuid(&id).is_err()
            || uuid(&attachment).is_err()
            || !digest_valid(&sha)
            || bytes > MAX_EXTRACTION_BYTES
            || version == 0
            || revision == 0
            || config.len() > 8192
        {
            return Err(storage("invalid_job"));
        }
        let value: Value =
            serde_json::from_str(&config).map_err(|_| storage("invalid_job_config"))?;
        if config_hash(&value)? != hash {
            return Err(storage("invalid_job_config"));
        }
        for i in 9..=12 {
            let time: Option<String> = r.get(i)?;
            if (i <= 10 && time.is_none())
                || time
                    .as_ref()
                    .is_some_and(|v| DateTime::parse_from_rfc3339(v).is_err())
            {
                return Err(storage("invalid_job_timestamp"));
            }
        }
        if let Some(raw) = r.get::<_, Option<String>>(13)? {
            let e: CoreError =
                serde_json::from_str(&raw).map_err(|_| storage("invalid_job_error"))?;
            if raw.len() > 4096 || e.message.chars().count() > 512 {
                return Err(storage("invalid_job_error"));
            }
        }
    }
    let ids = conn
        .prepare("SELECT id FROM extraction_generations")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for id in ids {
        generation(conn, &id)?;
    }
    Ok(())
}
