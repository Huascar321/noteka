//! Persistencia derivada del índice semántico. No es fuente de verdad ni
//! repara fuentes: sólo publica generaciones verificadas.
use crate::{
    attachment_search_contracts::AttachmentTextLocator,
    error::*,
    extraction_storage as store,
    semantic_contracts::*,
    workspace::{self, DB_NAME, SCHEMA_VERSION},
};
use chrono::DateTime;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde_json::json;
use sha2::Digest;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Duration,
};

pub(crate) struct Control {
    pub revision: u32,
    pub active_generation_id: Option<String>,
}

pub(crate) fn open(path: &Path, write: bool) -> Result<(Connection, PathBuf)> {
    let v = workspace::preflight(path)?;
    if v != SCHEMA_VERSION {
        let mut e = storage("migration_required");
        e.code = ErrorCode::UnsupportedWorkspaceVersion;
        e.details = json!({"current_version":v,"supported_version":SCHEMA_VERSION,"migration_required":v<SCHEMA_VERSION});
        return Err(e);
    }
    let root = std::fs::canonicalize(path)?;
    let c = Connection::open_with_flags(
        root.join(DB_NAME),
        (if write {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        }) | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    c.busy_timeout(Duration::from_millis(workspace::BUSY_TIMEOUT_MS))?;
    if write {
        c.pragma_update(None, "foreign_keys", true)?;
        c.pragma_update(None, "synchronous", "FULL")?;
    }
    Ok((c, root))
}

pub(crate) fn control(c: &Connection) -> Result<Control> {
    let (n, version, revision, active, at): (u32, u32, u32, Option<String>, String) = c.query_row(
        "SELECT (SELECT count(*) FROM semantic_control),version,revision,active_generation_id,updated_at FROM semantic_control WHERE id=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
    )?;
    if n != 1 || version != 1 || revision == 0 || DateTime::parse_from_rfc3339(&at).is_err() {
        return Err(storage("invalid_semantic_control"));
    }
    if let Some(id) = &active {
        let state: Option<String> = c
            .query_row(
                "SELECT state FROM semantic_generations WHERE generation_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        if state.as_deref() != Some("active") {
            return Err(storage("invalid_semantic_active_generation"));
        }
    }
    Ok(Control {
        revision,
        active_generation_id: active,
    })
}

pub(crate) fn insert_profile(c: &Connection, profile: &EmbeddingProfile) -> Result<()> {
    profile.validate()?;
    let fingerprint = profile.fingerprint();
    let canonical = profile.canonical();
    if let Some(stored) = c
        .query_row(
            "SELECT profile_json FROM semantic_profiles WHERE fingerprint=?1",
            [&fingerprint],
            |r| r.get::<_, String>(0),
        )
        .optional()?
    {
        if stored != canonical {
            return Err(storage("embedding_fingerprint_collision"));
        }
        return Ok(());
    }
    c.execute(
        "INSERT INTO semantic_profiles(fingerprint,version,profile_json,dimension,created_at) VALUES (?1,?2,?3,?4,?5)",
        params![
            fingerprint,
            SEMANTIC_PROFILE_VERSION,
            canonical,
            profile.dimension,
            workspace::now()
        ],
    )?;
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct NewChunk {
    pub ordinal: u32,
    pub job_id: String,
    pub attachment_id: String,
    pub source_generation_id: String,
    pub source_content_hash: String,
    pub extractor: String,
    pub extractor_version: u32,
    pub fragment_ordinal: u32,
    pub text: String,
    pub text_sha256: String,
    pub locator: AttachmentTextLocator,
    pub vector: Vec<f32>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn insert_building(
    c: &Connection,
    generation_id: &str,
    profile: &EmbeddingProfile,
    endpoint: &str,
    source_digest: &str,
    source_count: u32,
    chunk_count: u32,
    text_bytes: u64,
) -> Result<()> {
    if source_digest.len() != 64
        || source_count > MAX_SEMANTIC_SOURCES
        || chunk_count > MAX_SEMANTIC_CHUNKS
        || text_bytes > MAX_SEMANTIC_TEXT_BYTES
    {
        return Err(invalid("semantic_capacity_limit"));
    }
    insert_profile(c, profile)?;
    let at = workspace::now();
    c.execute(
        "INSERT INTO semantic_generations(generation_id,version,fingerprint,state,endpoint,source_digest,source_count,chunk_count,text_bytes,created_at,updated_at) VALUES (?1,1,?2,'building',?3,?4,?5,?6,?7,?8,?8)",
        params![
            generation_id,
            profile.fingerprint(),
            endpoint,
            source_digest,
            source_count,
            chunk_count,
            text_bytes as i64,
            at
        ],
    )?;
    Ok(())
}

pub(crate) fn insert_chunks(
    c: &Connection,
    generation_id: &str,
    chunks: &[NewChunk],
) -> Result<()> {
    let mut stmt = c.prepare(
        "INSERT INTO semantic_chunks(generation_id,ordinal,job_id,attachment_id,source_generation_id,source_content_hash,extractor,extractor_version,fragment_ordinal,text,text_sha256,locator,vector) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
    )?;
    for chunk in chunks {
        if chunk.text.is_empty()
            || chunk.text.len() > 4096
            || chunk.vector.is_empty()
            || chunk.vector.len() > MAX_EMBEDDING_DIMENSION as usize
        {
            return Err(invalid("semantic_chunk_limit"));
        }
        let bytes = encode_vector(&chunk.vector)?;
        stmt.execute(params![
            generation_id,
            chunk.ordinal,
            chunk.job_id,
            chunk.attachment_id,
            chunk.source_generation_id,
            chunk.source_content_hash,
            chunk.extractor,
            chunk.extractor_version,
            chunk.fragment_ordinal,
            chunk.text,
            chunk.text_sha256,
            serde_json::to_string(&chunk.locator)?,
            bytes
        ])?;
    }
    Ok(())
}

pub(crate) fn encode_vector(vector: &[f32]) -> Result<Vec<u8>> {
    if vector.is_empty() || vector.len() > MAX_EMBEDDING_DIMENSION as usize {
        return Err(invalid("invalid_embedding_dimension"));
    }
    let mut out = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        if !value.is_finite() {
            return Err(invalid("non_finite_embedding"));
        }
        out.extend_from_slice(&value.to_le_bytes());
    }
    Ok(out)
}
pub(crate) fn decode_vector(bytes: &[u8], dimension: u32) -> Result<Vec<f32>> {
    if dimension == 0
        || dimension > MAX_EMBEDDING_DIMENSION
        || bytes.len() != dimension as usize * 4
    {
        return Err(storage("invalid_semantic_vector"));
    }
    let mut out = Vec::with_capacity(dimension as usize);
    let (chunks, _) = bytes.as_chunks::<4>();
    for chunk in chunks {
        let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if !value.is_finite() {
            return Err(storage("invalid_semantic_vector"));
        }
        out.push(value);
    }
    Ok(out)
}
pub(crate) fn normalize(vector: &mut [f32]) -> Result<()> {
    if vector.is_empty() || vector.len() > MAX_EMBEDDING_DIMENSION as usize {
        return Err(invalid("invalid_embedding_dimension"));
    }
    let mut sum = 0.0f64;
    for value in vector.iter() {
        if !value.is_finite() {
            return Err(invalid("non_finite_embedding"));
        }
        sum += f64::from(*value) * f64::from(*value);
    }
    let norm = sum.sqrt();
    if !norm.is_finite() || norm <= 1e-12 {
        return Err(invalid("zero_embedding_norm"));
    }
    for value in vector.iter_mut() {
        *value = (f64::from(*value) / norm) as f32;
    }
    Ok(())
}
pub(crate) fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

pub(crate) fn prune_abandoned_builds(c: &Connection) -> Result<u32> {
    let cutoff = chrono::Utc::now() - chrono::Duration::milliseconds(ABANDONED_BUILD_MS as i64);
    let cutoff = cutoff.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    let ids = c
        .prepare("SELECT generation_id FROM semantic_generations WHERE state='building' AND updated_at<?1")?
        .query_map([cutoff], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for id in &ids {
        c.execute("DELETE FROM semantic_chunks WHERE generation_id=?1", [id])?;
        c.execute(
            "UPDATE semantic_generations SET state='failed',error='abandoned_build',updated_at=?2 WHERE generation_id=?1 AND state='building'",
            params![id, workspace::now()],
        )?;
    }
    Ok(ids.len() as u32)
}

type GenerationRow = (
    String,
    String,
    String,
    String,
    u32,
    u32,
    i64,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    u32,
);
pub(crate) fn generation_info(c: &Connection, id: &str) -> Result<SemanticGenerationInfo> {
    let (fingerprint, state, endpoint, digest, sources, chunks, bytes, created, updated, activated, retired, error, profile_json, dimension): GenerationRow = c.query_row(
        "SELECT g.fingerprint,g.state,g.endpoint,g.source_digest,g.source_count,g.chunk_count,g.text_bytes,g.created_at,g.updated_at,g.activated_at,g.retired_at,g.error,p.profile_json,p.dimension FROM semantic_generations g JOIN semantic_profiles p ON p.fingerprint=g.fingerprint WHERE g.generation_id=?1",
        [id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?,r.get(10)?,r.get(11)?,r.get(12)?,r.get(13)?)),
    ).optional()?.ok_or_else(|| store::error(ErrorCode::NotFound,"semantic_generation_not_found"))?;
    let profile: EmbeddingProfile = serde_json::from_str(&profile_json)?;
    if profile.fingerprint() != fingerprint || profile.dimension != dimension {
        return Err(storage("invalid_semantic_generation"));
    }
    let state = match state.as_str() {
        "building" => SemanticGenerationState::Building,
        "active" => SemanticGenerationState::Active,
        "retired" => SemanticGenerationState::Retired,
        "failed" => SemanticGenerationState::Failed,
        _ => return Err(storage("invalid_semantic_generation")),
    };
    Ok(SemanticGenerationInfo {
        generation_id: id.into(),
        fingerprint,
        profile,
        state,
        dimension,
        endpoint,
        source_digest: digest,
        source_count: sources,
        chunk_count: chunks,
        text_bytes: bytes as u64,
        created_at: created,
        updated_at: updated,
        activated_at: activated,
        retired_at: retired,
        error,
    })
}

pub(crate) fn status(c: &Connection) -> Result<EmbeddingStatus> {
    let control = control(c)?;
    let active = control
        .active_generation_id
        .as_ref()
        .map(|id| generation_info(c, id))
        .transpose()?;
    let building_id: Option<String> = c
        .query_row(
            "SELECT generation_id FROM semantic_generations WHERE state='building' ORDER BY created_at DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let building = building_id
        .as_ref()
        .map(|id| generation_info(c, id))
        .transpose()?;
    let retired = c
        .prepare("SELECT generation_id FROM semantic_generations WHERE state='retired' ORDER BY retired_at DESC,generation_id LIMIT 8")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .map(|id| generation_info(c, id))
        .collect::<Result<Vec<_>>>()?;
    let coverage = coverage(c, &control)?;
    Ok(EmbeddingStatus {
        version: 1,
        revision: control.revision,
        active,
        building,
        retired,
        coverage,
    })
}

impl std::fmt::Debug for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Control")
            .field("revision", &self.revision)
            .field("active_generation_id", &self.active_generation_id)
            .finish()
    }
}

pub(crate) fn coverage(c: &Connection, control: &Control) -> Result<SemanticCoverage> {
    let Some(active) = &control.active_generation_id else {
        let unindexed: u32 = c.query_row(
            "SELECT count(*) FROM extraction_jobs WHERE generation_id IS NOT NULL",
            [],
            |r| r.get(0),
        )?;
        return Ok(SemanticCoverage {
            active_sources: 0,
            active_chunks: 0,
            active_text_bytes: 0,
            stale_sources: 0,
            stale_chunks: 0,
            unindexed_supported_generations: unindexed,
        });
    };
    let (active_sources, active_chunks, active_text_bytes): (u32, u32, i64) = c.query_row(
        "SELECT count(DISTINCT job_id),count(*),coalesce(sum(length(CAST(text AS BLOB))),0) FROM semantic_chunks WHERE generation_id=?1",
        [active],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let sources = c
        .prepare("SELECT DISTINCT job_id,source_generation_id,attachment_id,source_content_hash FROM semantic_chunks WHERE generation_id=?1")?
        .query_map([active], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut stale_sources = 0;
    let mut stale_chunks = 0;
    for (job, generation, attachment, hash) in &sources {
        if source_state(c, job, generation, attachment, hash)? != RetrievalSourceState::Current {
            stale_sources += 1;
            stale_chunks += c.query_row(
                "SELECT count(*) FROM semantic_chunks WHERE generation_id=?1 AND job_id=?2",
                params![active, job],
                |r| r.get::<_, u32>(0),
            )?;
        }
    }
    let total: u32 = c.query_row(
        "SELECT count(*) FROM extraction_jobs WHERE generation_id IS NOT NULL",
        [],
        |r| r.get(0),
    )?;
    Ok(SemanticCoverage {
        active_sources,
        active_chunks,
        active_text_bytes: active_text_bytes as u64,
        stale_sources,
        stale_chunks,
        unindexed_supported_generations: total.saturating_sub(active_sources),
    })
}

pub(crate) fn source_state(
    c: &Connection,
    job_id: &str,
    source_generation_id: &str,
    attachment_id: &str,
    source_content_hash: &str,
) -> Result<RetrievalSourceState> {
    let row: Option<(Option<String>, String, u32, Option<String>)> = c
        .query_row(
            "SELECT j.generation_id,a.sha256,a.bytes,(SELECT content_hash FROM extraction_search_entries e WHERE e.job_id=j.id) FROM extraction_jobs j JOIN attachments a ON a.id=j.attachment_id WHERE j.id=?1 AND j.attachment_id=?2",
            params![job_id, attachment_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let Some((generation, sha, bytes, indexed_hash)) = row else {
        return Ok(RetrievalSourceState::Stale);
    };
    if generation.as_deref() != Some(source_generation_id)
        || indexed_hash
            .as_deref()
            .is_some_and(|h| h != source_content_hash)
    {
        return Ok(RetrievalSourceState::Stale);
    }
    let source_ok: bool = c.query_row(
        "SELECT EXISTS(SELECT 1 FROM extraction_generations g WHERE g.id=?1 AND g.job_id=?2 AND g.source_sha256=?3 AND g.source_bytes=?4)",
        params![source_generation_id, job_id, sha, bytes],
        |r| r.get(0),
    )?;
    if !source_ok {
        return Ok(RetrievalSourceState::Stale);
    }
    Ok(RetrievalSourceState::Current)
}

pub(crate) struct ActivateResult {
    pub revision: u32,
    pub previous_generation_id: Option<String>,
}
pub(crate) fn activate(c: &Connection, generation_id: &str) -> Result<ActivateResult> {
    let tx = c.unchecked_transaction()?;
    let (state, expected_sources, expected_chunks, expected_bytes, expected_digest): (
        String,
        u32,
        u32,
        i64,
        String,
    ) = tx
        .query_row(
            "SELECT state,source_count,chunk_count,text_bytes,source_digest FROM semantic_generations WHERE generation_id=?1",
            [generation_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?
        .ok_or_else(|| store::error(ErrorCode::NotFound, "semantic_generation_not_found"))?;
    if state != "building" {
        return Err(storage("semantic_generation_not_building"));
    }
    let (actual_chunks, actual_bytes, actual_sources): (u32, i64, u32) = tx.query_row(
        "SELECT count(*),coalesce(sum(length(CAST(text AS BLOB))),0),count(DISTINCT job_id) FROM semantic_chunks WHERE generation_id=?1",
        [generation_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    if actual_chunks != expected_chunks
        || actual_bytes != expected_bytes
        || actual_sources != expected_sources
    {
        return Err(storage("semantic_coverage_mismatch"));
    }
    let digest_rows = tx
        .prepare("SELECT DISTINCT job_id,source_generation_id,source_content_hash FROM semantic_chunks WHERE generation_id=?1")?
        .query_map([generation_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if source_digest(&digest_rows) != expected_digest {
        return Err(storage("semantic_source_digest_mismatch"));
    }
    let bad_vector: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM semantic_chunks c JOIN semantic_generations g ON g.generation_id=c.generation_id JOIN semantic_profiles p ON p.fingerprint=g.fingerprint WHERE c.generation_id=?1 AND length(c.vector)<>p.dimension*4)",
        [generation_id],
        |r| r.get(0),
    )?;
    if bad_vector {
        return Err(storage("semantic_vector_dimension_mismatch"));
    }
    let control = control(&tx)?;
    let previous = control.active_generation_id.clone();
    let at = workspace::now();
    if let Some(previous) = &previous {
        tx.execute(
            "UPDATE semantic_generations SET state='retired',retired_at=?2,updated_at=?2 WHERE generation_id=?1 AND state='active'",
            params![previous, at],
        )?;
    }
    tx.execute(
        "UPDATE semantic_generations SET state='active',activated_at=?2,updated_at=?2,error=NULL WHERE generation_id=?1",
        params![generation_id, at],
    )?;
    let revision = store::bump(control.revision)?;
    tx.execute(
        "UPDATE semantic_control SET revision=?2,active_generation_id=?3,updated_at=?4 WHERE id=1",
        params![1, revision, generation_id, at],
    )?;
    // Sólo se conserva la generación retirada inmediatamente anterior.
    let stale: Vec<String> = tx
        .prepare("SELECT generation_id FROM semantic_generations WHERE state='retired' AND generation_id<>?1")?
        .query_map([previous.as_deref().unwrap_or("")], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for id in stale {
        tx.execute("DELETE FROM semantic_chunks WHERE generation_id=?1", [&id])?;
        tx.execute(
            "DELETE FROM semantic_generations WHERE generation_id=?1",
            [&id],
        )?;
    }
    tx.execute(
        "DELETE FROM semantic_chunks WHERE generation_id IN (SELECT generation_id FROM semantic_generations WHERE state='failed')",
        [],
    )?;
    tx.execute("DELETE FROM semantic_generations WHERE state='failed'", [])?;
    tx.execute(
        "DELETE FROM semantic_profiles WHERE fingerprint NOT IN (SELECT fingerprint FROM semantic_generations)",
        [],
    )?;
    tx.commit()?;
    Ok(ActivateResult {
        revision,
        previous_generation_id: previous,
    })
}

pub(crate) fn fail_building(c: &Connection, generation_id: &str, reason: &str) -> Result<()> {
    let tx = c.unchecked_transaction()?;
    tx.execute(
        "DELETE FROM semantic_chunks WHERE generation_id=?1",
        [generation_id],
    )?;
    tx.execute(
        "UPDATE semantic_generations SET state='failed',error=?2,updated_at=?3 WHERE generation_id=?1 AND state='building'",
        params![generation_id, reason.chars().take(512).collect::<String>(), workspace::now()],
    )?;
    tx.commit()?;
    Ok(())
}

pub(crate) fn reset(c: &Connection, expected_revision: u32) -> Result<(u32, u32, u32)> {
    let tx = c.unchecked_transaction()?;
    let control = control(&tx)?;
    if control.revision != expected_revision {
        let mut e = storage("expected_revision_mismatch");
        e.code = ErrorCode::RevisionConflict;
        e.details["current_revision"] = json!(control.revision);
        return Err(e);
    }
    let (generations, chunks): (u32, u32) = tx.query_row(
        "SELECT (SELECT count(*) FROM semantic_generations),(SELECT count(*) FROM semantic_chunks)",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let revision = store::bump(control.revision)?;
    tx.execute(
        "UPDATE semantic_control SET revision=?2,active_generation_id=NULL,updated_at=?3 WHERE id=1",
        params![1, revision, workspace::now()],
    )?;
    tx.execute("DELETE FROM semantic_chunks", [])?;
    tx.execute("DELETE FROM semantic_generations", [])?;
    tx.execute("DELETE FROM semantic_profiles", [])?;
    tx.commit()?;
    Ok((revision, generations, chunks))
}

pub(crate) fn source_digest(entries: &[(String, String, String)]) -> String {
    let mut sorted = entries.to_vec();
    sorted.sort_unstable();
    let mut h = sha2::Sha256::new();
    h.update(b"noteka.semantic.sources.v1\0");
    for (job, generation, hash) in sorted {
        for value in [job, generation, hash] {
            h.update((value.len() as u64).to_be_bytes());
            h.update(value.as_bytes());
        }
    }
    format!("{:x}", h.finalize())
}

pub(crate) fn load_chunks(
    c: &Connection,
    generation_id: &str,
    filter: &SemanticFilter,
) -> Result<Vec<LoadedChunk>> {
    let mut stmt = c.prepare(
        "SELECT ordinal,job_id,attachment_id,source_generation_id,source_content_hash,extractor,extractor_version,fragment_ordinal,text,text_sha256,locator,vector FROM semantic_chunks WHERE generation_id=?1 ORDER BY ordinal",
    )?;
    let rows = stmt.query_map([generation_id], |r| {
        Ok((
            r.get::<_, u32>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, String>(5)?,
            r.get::<_, u32>(6)?,
            r.get::<_, u32>(7)?,
            r.get::<_, String>(8)?,
            r.get::<_, String>(9)?,
            r.get::<_, String>(10)?,
            r.get::<_, Vec<u8>>(11)?,
        ))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let row = row?;
        if !filter.matches(&row.1, &row.2, &row.5) {
            continue;
        }
        out.push(LoadedChunk {
            ordinal: row.0,
            job_id: row.1,
            attachment_id: row.2,
            source_generation_id: row.3,
            source_content_hash: row.4,
            extractor: row.5,
            extractor_version: row.6,
            fragment_ordinal: row.7,
            text: row.8,
            text_sha256: row.9,
            locator: serde_json::from_str(&row.10)?,
            vector: row.11,
        });
    }
    Ok(out)
}

pub(crate) struct LoadedChunk {
    pub ordinal: u32,
    pub job_id: String,
    pub attachment_id: String,
    pub source_generation_id: String,
    pub source_content_hash: String,
    pub extractor: String,
    pub extractor_version: u32,
    pub fragment_ordinal: u32,
    pub text: String,
    pub text_sha256: String,
    pub locator: AttachmentTextLocator,
    pub vector: Vec<u8>,
}

pub(crate) struct SemanticFilter {
    pub job_id: Option<String>,
    pub attachment_id: Option<String>,
    pub extractor: Option<String>,
}
impl SemanticFilter {
    pub fn matches(&self, job: &str, attachment: &str, extractor: &str) -> bool {
        self.job_id.as_deref().is_none_or(|v| v == job)
            && self
                .attachment_id
                .as_deref()
                .is_none_or(|v| v == attachment)
            && self.extractor.as_deref().is_none_or(|v| v == extractor)
    }
}

pub(crate) fn stale_filter(
    c: &Connection,
    chunks: &[LoadedChunk],
    include_stale: bool,
) -> Result<(HashMap<u32, RetrievalSourceState>, u32)> {
    let mut cache = HashMap::<(String, String, String, String), RetrievalSourceState>::new();
    let mut states = HashMap::new();
    let mut excluded = 0u32;
    for chunk in chunks {
        let key = (
            chunk.job_id.clone(),
            chunk.source_generation_id.clone(),
            chunk.attachment_id.clone(),
            chunk.source_content_hash.clone(),
        );
        let state = if let Some(v) = cache.get(&key) {
            *v
        } else {
            let v = source_state(
                c,
                &chunk.job_id,
                &chunk.source_generation_id,
                &chunk.attachment_id,
                &chunk.source_content_hash,
            )?;
            cache.insert(key, v);
            v
        };
        if state == RetrievalSourceState::Current
            || (include_stale && state != RetrievalSourceState::Unknown)
        {
            states.insert(chunk.ordinal, state);
        } else {
            excluded += 1;
        }
    }
    Ok((states, excluded))
}

/// Integridad de las tablas derivadas. No repara ni descarta fuentes.
pub(crate) fn integrity(c: &Connection) -> Result<()> {
    let control = control(c)?;
    let active_count: u32 = c.query_row(
        "SELECT count(*) FROM semantic_generations WHERE state='active'",
        [],
        |r| r.get(0),
    )?;
    let building_count: u32 = c.query_row(
        "SELECT count(*) FROM semantic_generations WHERE state='building'",
        [],
        |r| r.get(0),
    )?;
    if active_count > 1 || building_count > 1 {
        return Err(storage("invalid_semantic_generation_state"));
    }
    match (&control.active_generation_id, active_count) {
        (Some(_), 1) | (None, 0) => {}
        _ => return Err(storage("invalid_semantic_active_generation")),
    }
    let rows = c
        .prepare("SELECT g.generation_id,g.fingerprint,g.state,p.dimension,g.source_count,g.chunk_count,g.text_bytes,g.source_digest FROM semantic_generations g JOIN semantic_profiles p ON p.fingerprint=g.fingerprint")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, u32>(3)?,
                r.get::<_, u32>(4)?,
                r.get::<_, u32>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, String>(7)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for (id, fingerprint, state, dimension, sources, chunks, bytes, digest) in rows {
        let profile_json: String = c
            .query_row(
                "SELECT profile_json FROM semantic_profiles WHERE fingerprint=?1",
                [&fingerprint],
                |r| r.get(0),
            )
            .optional()?
            .ok_or_else(|| storage("missing_semantic_profile"))?;
        let profile: EmbeddingProfile = serde_json::from_str(&profile_json)?;
        profile.validate()?;
        if profile.fingerprint() != fingerprint || profile.dimension != dimension {
            return Err(storage("invalid_semantic_profile"));
        }
        if digest.len() != 64
            || sources > MAX_SEMANTIC_SOURCES
            || chunks > MAX_SEMANTIC_CHUNKS
            || bytes < 0
            || bytes as u64 > MAX_SEMANTIC_TEXT_BYTES
        {
            return Err(storage("invalid_semantic_generation"));
        }
        let (actual_chunks, actual_bytes, actual_sources): (u32, i64, u32) = c.query_row(
            "SELECT count(*),coalesce(sum(length(CAST(text AS BLOB))),0),count(DISTINCT job_id) FROM semantic_chunks WHERE generation_id=?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let exact = state == "active" || state == "retired";
        if (exact
            && (actual_chunks != chunks || actual_bytes != bytes || actual_sources != sources))
            || (!exact
                && (actual_chunks > chunks || actual_bytes > bytes || actual_sources > sources))
        {
            return Err(storage("semantic_coverage_mismatch"));
        }
        let mut stmt = c.prepare(
            "SELECT ordinal,text,text_sha256,locator,vector FROM semantic_chunks WHERE generation_id=?1 ORDER BY ordinal",
        )?;
        let rows = stmt.query_map([&id], |r| {
            Ok((
                r.get::<_, u32>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Vec<u8>>(4)?,
            ))
        })?;
        let mut expected_ordinal = 0;
        for row in rows {
            let (ordinal, text, text_hash, locator, vector) = row?;
            if ordinal != expected_ordinal
                || text.is_empty()
                || text.len() > 4096
                || text_hash != format!("{:x}", sha2::Sha256::digest(text.as_bytes()))
                || !locator.starts_with('{')
            {
                return Err(storage("invalid_semantic_chunk"));
            }
            let decoded = decode_vector(&vector, dimension)?;
            let norm = decoded.iter().map(|v| v * v).sum::<f32>().sqrt();
            if (norm - 1.0).abs() > 0.01 {
                return Err(storage("invalid_semantic_vector_norm"));
            }
            let _: AttachmentTextLocator = serde_json::from_str(&locator)?;
            expected_ordinal += 1;
        }
        if expected_ordinal != actual_chunks {
            return Err(storage("invalid_semantic_chunk"));
        }
    }
    Ok(())
}
