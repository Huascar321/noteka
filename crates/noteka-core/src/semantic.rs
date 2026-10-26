//! Recuperación semántica local: perfiles, generaciones, búsqueda híbrida y
//! degradación explícita a FTS. No ejecuta LLM ni envía contenido remoto.
use crate::{
    attachment_search,
    attachment_search_contracts::{
        AttachmentSearchHit, AttachmentSearchInput, AttachmentTextLocator, IndexFilter,
        IndexedExtractor,
    },
    attachment_search_storage as index,
    batch_storage::Budget,
    contracts::*,
    error::*,
    extraction_storage as store, local_http, scoped, semantic_storage as semantic, workspace,
};
use rusqlite::params;
use serde_json::{Value, json};
use sha2::Digest;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    time::{Duration, Instant},
};

pub(crate) enum EmbeddingPurpose {
    Query,
    Document,
}

pub(crate) trait EmbeddingRuntime {
    fn embed(
        &self,
        purpose: EmbeddingPurpose,
        texts: &[String],
        timeout: Duration,
    ) -> Result<Vec<Vec<f32>>>;
}

#[derive(Debug)]
struct HttpEmbeddingRuntime {
    profile: EmbeddingProfile,
    endpoint: String,
}
impl HttpEmbeddingRuntime {
    fn new(profile: EmbeddingProfile, endpoint: String) -> Result<Self> {
        profile.validate()?;
        local_http::loopback_target(&endpoint)?;
        Ok(Self { profile, endpoint })
    }
}
impl EmbeddingRuntime for HttpEmbeddingRuntime {
    fn embed(
        &self,
        purpose: EmbeddingPurpose,
        texts: &[String],
        timeout: Duration,
    ) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() || texts.len() > MAX_EMBEDDING_BATCH {
            return Err(invalid("embedding_batch_limit"));
        }
        let input = texts
            .iter()
            .map(|text| match purpose {
                EmbeddingPurpose::Query => self.profile.query_text(text),
                EmbeddingPurpose::Document => self.profile.document_text(text),
            })
            .collect::<Vec<_>>();
        let body = json!({"model":self.profile.model,"input":input});
        let response = local_http::post_json(&self.endpoint, &body, 8 * 1024 * 1024, timeout)?;
        parse_embeddings(&response, texts.len(), self.profile.dimension)
    }
}
fn parse_embeddings(value: &Value, expected: usize, dimension: u32) -> Result<Vec<Vec<f32>>> {
    let mut out = vec![None; expected];
    if let Some(data) = value.get("data").and_then(|v| v.as_array()) {
        for (position, item) in data.iter().enumerate() {
            let index = item
                .get("index")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(position);
            let values = item
                .get("embedding")
                .and_then(|v| v.as_array())
                .ok_or_else(|| unavailable("provider_response_invalid"))?;
            fill_embedding(&mut out, index, values, dimension)?;
        }
    } else if let Some(data) = value.get("embeddings").and_then(|v| v.as_array()) {
        for (index, values) in data.iter().enumerate() {
            let values = values
                .as_array()
                .ok_or_else(|| unavailable("provider_response_invalid"))?;
            fill_embedding(&mut out, index, values, dimension)?;
        }
    } else {
        return Err(unavailable("provider_response_invalid"));
    }
    out.into_iter()
        .map(|v| v.ok_or_else(|| unavailable("provider_response_incomplete")))
        .collect()
}
fn fill_embedding(
    out: &mut [Option<Vec<f32>>],
    index: usize,
    values: &[Value],
    dimension: u32,
) -> Result<()> {
    if index >= out.len() || out[index].is_some() || values.len() != dimension as usize {
        return Err(unavailable("provider_dimension_mismatch"));
    }
    let mut vector = Vec::with_capacity(values.len());
    for value in values {
        let value = value
            .as_f64()
            .ok_or_else(|| unavailable("provider_vector_invalid"))?;
        let value = value as f32;
        if !value.is_finite() {
            return Err(unavailable("provider_vector_invalid"));
        }
        vector.push(value);
    }
    semantic::normalize(&mut vector).map_err(|_| unavailable("provider_vector_invalid"))?;
    out[index] = Some(vector);
    Ok(())
}

fn filter(
    job_id: Option<String>,
    attachment_id: Option<String>,
    extractor: Option<IndexedExtractor>,
) -> Result<IndexFilter> {
    let filter = IndexFilter {
        job_id,
        attachment_id,
        extractor,
    };
    filter.validate()?;
    Ok(filter)
}

pub(crate) fn index(
    path: &Path,
    input: EmbeddingIndexInput,
    scope: Option<&scoped::LocalScope>,
) -> Result<EmbeddingIndexResult> {
    match input {
        EmbeddingIndexInput::Status {} => {
            let (c, _) = semantic::open(path, false)?;
            if let Some(scope) = scope {
                scope.check(&c)?;
            }
            Ok(EmbeddingIndexResult::Status(semantic::status(&c)?))
        }
        EmbeddingIndexInput::Reset { expected_revision } => {
            if expected_revision == 0 {
                return Err(invalid("expected_revision"));
            }
            let (c, _) = semantic::open(path, true)?;
            if let Some(scope) = scope {
                scope.check(&c)?;
            }
            let (revision, generations, chunks) = semantic::reset(&c, expected_revision)?;
            Ok(EmbeddingIndexResult::Reset {
                revision,
                removed_generations: generations,
                removed_chunks: chunks,
            })
        }
        EmbeddingIndexInput::Build {
            profile,
            endpoint,
            job_id,
            attachment_id,
            extractor,
        } => {
            let filter = filter(job_id, attachment_id, extractor)?;
            let runtime = HttpEmbeddingRuntime::new(profile.clone(), endpoint.clone())?;
            Ok(EmbeddingIndexResult::Built(build_with_runtime(
                path, profile, endpoint, filter, scope, &runtime,
            )?))
        }
    }
}

fn build_with_runtime(
    path: &Path,
    profile: EmbeddingProfile,
    endpoint: String,
    filter: IndexFilter,
    scope: Option<&scoped::LocalScope>,
    runtime: &dyn EmbeddingRuntime,
) -> Result<SemanticBuildResult> {
    profile.validate()?;
    local_http::loopback_target(&endpoint)?;
    let started = Instant::now();
    let (c, root) = semantic::open(path, true)?;
    if let Some(scope) = scope {
        scope.check(&c)?;
    }
    semantic::prune_abandoned_builds(&c)?;
    let building: Option<String> = c
        .query_row(
            "SELECT generation_id FROM semantic_generations WHERE state='building' LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();
    if building.is_some() {
        return Err(invalid("semantic_build_in_progress"));
    }
    let filter_string = filter.extractor.map(|e| e.as_str().to_owned());
    let budget = Budget::with_duration(&c, Duration::from_secs(120))?;
    let mut stmt = c.prepare(
        "SELECT id FROM extraction_jobs WHERE generation_id IS NOT NULL AND extractor IN ('utf8_text','pdf_text','xlsx_cells','image_ocr','pdfkit_text','pdf_ocr') AND (?1 IS NULL OR id=?1) AND (?2 IS NULL OR attachment_id=?2) AND (?3 IS NULL OR extractor=?3) ORDER BY id",
    )?;
    let jobs = stmt
        .query_map(
            params![
                filter.job_id.as_deref(),
                filter.attachment_id.as_deref(),
                filter_string.as_deref()
            ],
            |r| r.get::<_, String>(0),
        )?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut selected = Vec::new();
    let mut total_chunks = 0u32;
    let mut text_bytes = 0u64;
    let mut digest_rows = Vec::new();
    for job in &jobs {
        budget.check()?;
        let canonical = index::canonical(&c, job, &budget)?;
        index::source(&c, &root, &canonical, &budget)?;
        if canonical.fragments.is_empty() {
            continue;
        }
        total_chunks = total_chunks.saturating_add(canonical.fragments.len() as u32);
        text_bytes = text_bytes.saturating_add(u64::from(canonical.text_bytes));
        if total_chunks > MAX_SEMANTIC_CHUNKS
            || text_bytes > MAX_SEMANTIC_TEXT_BYTES
            || selected.len() as u32 >= MAX_SEMANTIC_SOURCES
        {
            return Err(invalid("semantic_capacity_limit"));
        }
        if canonical
            .fragments
            .iter()
            .any(|f| f.text.is_empty() || f.text.len() > 4096)
        {
            return Err(invalid("semantic_chunk_limit"));
        }
        digest_rows.push((
            canonical.source.job_id.clone(),
            canonical.source.generation_id.clone(),
            canonical.hash.clone(),
        ));
        selected.push(canonical);
    }
    drop(stmt);
    drop(budget);
    if selected.is_empty() {
        return Err(store::error(ErrorCode::NotFound, "no_semantic_sources"));
    }
    let digest = semantic::source_digest(&digest_rows);
    let control = semantic::control(&c)?;
    if let Some(active_id) = &control.active_generation_id {
        let active = semantic::generation_info(&c, active_id)?;
        if active.fingerprint == profile.fingerprint()
            && active.source_digest == digest
            && active.dimension == profile.dimension
        {
            return Ok(SemanticBuildResult {
                version: 1,
                revision: control.revision,
                generation: active,
                previous_generation_id: None,
                reused: true,
                embedded_chunks: 0,
                duration_ms: started.elapsed().as_millis() as u64,
            });
        }
    }
    let generation_id = workspace::id();
    {
        let tx = c.unchecked_transaction()?;
        if let Some(scope) = scope {
            scope.check(&tx)?;
        }
        semantic::insert_building(
            &tx,
            &generation_id,
            &profile,
            &endpoint,
            &digest,
            selected.len() as u32,
            total_chunks,
            text_bytes,
        )?;
        tx.commit()?;
    }
    let result = (|| -> Result<semantic::ActivateResult> {
        let deadline = Instant::now() + Duration::from_millis(SEMANTIC_BUILD_TIMEOUT_MS);
        let mut pending = Vec::new();
        for (source_index, source) in selected.iter().enumerate() {
            for (fragment_index, _) in source.fragments.iter().enumerate() {
                pending.push((source_index, fragment_index));
            }
        }
        let mut ordinal = 0u32;
        for batch in pending.chunks(MAX_EMBEDDING_BATCH) {
            if Instant::now() >= deadline {
                return Err(invalid("semantic_build_deadline"));
            }
            scoped::check_shutdown()?;
            let texts = batch
                .iter()
                .map(|(s, f)| selected[*s].fragments[*f].text.clone())
                .collect::<Vec<_>>();
            let remaining = deadline.saturating_duration_since(Instant::now());
            let timeout = remaining
                .min(Duration::from_secs(60))
                .max(Duration::from_millis(1));
            let vectors = runtime.embed(EmbeddingPurpose::Document, &texts, timeout)?;
            if vectors.len() != texts.len() {
                return Err(unavailable("provider_response_incomplete"));
            }
            let mut chunks = Vec::with_capacity(vectors.len());
            for ((source_index, fragment_index), vector) in batch.iter().zip(vectors) {
                let source = &selected[*source_index];
                let fragment = &source.fragments[*fragment_index];
                if vector.len() != profile.dimension as usize {
                    return Err(invalid("embedding_dimension_mismatch"));
                }
                let mut vector = vector;
                semantic::normalize(&mut vector)?;
                chunks.push(semantic::NewChunk {
                    ordinal,
                    job_id: source.source.job_id.clone(),
                    attachment_id: source.source.attachment_id.clone(),
                    source_generation_id: source.source.generation_id.clone(),
                    source_content_hash: source.hash.clone(),
                    extractor: source.source.extractor.clone(),
                    extractor_version: source.source.extractor_version,
                    fragment_ordinal: fragment.ordinal,
                    text: fragment.text.clone(),
                    text_sha256: format!("{:x}", sha2::Sha256::digest(fragment.text.as_bytes())),
                    locator: fragment.locator.clone(),
                    vector,
                });
                ordinal += 1;
            }
            let tx = c.unchecked_transaction()?;
            semantic::insert_chunks(&tx, &generation_id, &chunks)?;
            tx.commit()?;
        }
        if ordinal != total_chunks {
            return Err(storage("semantic_coverage_mismatch"));
        }
        semantic::activate(&c, &generation_id)
    })();
    let activated = match result {
        Ok(activated) => activated,
        Err(error) => {
            let reason = error
                .details
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("semantic_build_failed")
                .to_owned();
            if let Err(cleanup) = semantic::fail_building(&c, &generation_id, &reason) {
                let _ = cleanup;
            }
            return Err(error);
        }
    };
    let info = semantic::generation_info(&c, &generation_id)?;
    Ok(SemanticBuildResult {
        version: 1,
        revision: activated.revision,
        generation: info,
        previous_generation_id: activated.previous_generation_id,
        reused: false,
        embedded_chunks: total_chunks,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

struct Candidate {
    job_id: String,
    attachment_id: String,
    source_generation_id: String,
    extractor: String,
    extractor_version: u32,
    fragment_ordinal: u32,
    text: String,
    text_sha256: String,
    locator: AttachmentTextLocator,
    source_state: RetrievalSourceState,
    semantic_score: Option<f32>,
    semantic_rank: Option<u32>,
    lexical_score: Option<f64>,
    lexical_rank: Option<u32>,
    fused: f64,
}
impl Candidate {
    fn key(&self) -> (String, String, u32) {
        (
            self.job_id.clone(),
            self.source_generation_id.clone(),
            self.fragment_ordinal,
        )
    }
}

pub(crate) fn search(
    path: &Path,
    input: RetrievalInput,
    scope: Option<&scoped::LocalScope>,
) -> Result<RetrievalPage> {
    search_inner(path, input, scope, None)
}

fn search_inner(
    path: &Path,
    input: RetrievalInput,
    scope: Option<&scoped::LocalScope>,
    runtime_override: Option<&dyn EmbeddingRuntime>,
) -> Result<RetrievalPage> {
    let query = input.query.as_str();
    if query.is_empty()
        || query.len() > MAX_QUERY_BYTES
        || query.chars().any(|c| c.is_control() && !c.is_whitespace())
    {
        return Err(invalid("retrieval_query_limit"));
    }
    let mode = input.mode.unwrap_or(RetrievalMode::Hybrid);
    let limit = input.limit.unwrap_or(DEFAULT_RESULTS);
    if limit == 0 || limit > MAX_RESULTS {
        return Err(invalid("retrieval_result_limit"));
    }
    let budget_bytes = input.budget_bytes.unwrap_or(DEFAULT_BUDGET_BYTES);
    if budget_bytes == 0 || budget_bytes > MAX_BUDGET_BYTES {
        return Err(invalid("retrieval_budget_limit"));
    }
    let filter = filter(input.job_id, input.attachment_id, input.extractor)?;
    let (c, _) = semantic::open(path, false)?;
    if let Some(scope) = scope {
        scope.check(&c)?;
    }
    let control = semantic::control(&c)?;
    let active = control
        .active_generation_id
        .as_ref()
        .map(|id| semantic::generation_info(&c, id))
        .transpose()?;
    let lexical_count: u32 =
        c.query_row("SELECT count(*) FROM extraction_search_entries", [], |r| {
            r.get(0)
        })?;
    let mut degraded = Vec::<String>::new();
    let mut semantic_items = Vec::<Candidate>::new();
    let mut lexical_candidates = Vec::<Candidate>::new();
    let mut stale_excluded = 0u32;
    let mut semantic_failed = false;

    let wants_semantic = matches!(mode, RetrievalMode::Hybrid | RetrievalMode::Semantic);
    if wants_semantic {
        match &active {
            None => {
                semantic_failed = true;
                degraded.push("no_active_generation".into());
            }
            Some(active) => {
                match semantic_candidates(
                    &c,
                    active,
                    query,
                    &filter,
                    input.include_stale,
                    limit,
                    runtime_override,
                ) {
                    Ok((items, excluded)) => {
                        stale_excluded = excluded;
                        semantic_items = items;
                    }
                    Err(error) if degradable(&error) => {
                        semantic_failed = true;
                        degraded.push("query_embedding_failed".into());
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }

    let wants_lexical = match mode {
        RetrievalMode::Hybrid | RetrievalMode::Lexical => true,
        RetrievalMode::Semantic => semantic_failed,
    };
    if wants_lexical {
        if lexical_count == 0 {
            degraded.push("no_lexical_index".into());
        } else if !lexical_query_supported(query) {
            degraded.push("query_not_lexical".into());
        } else {
            let page = attachment_search::search_scoped(
                path,
                AttachmentSearchInput {
                    query: query.to_owned(),
                    job_id: filter.job_id.clone(),
                    attachment_id: filter.attachment_id.clone(),
                    extractor: filter.extractor,
                    limit: Some((limit * 4).min(64)),
                    cursor: None,
                },
                scope,
            )?;
            lexical_candidates = page
                .items
                .into_iter()
                .enumerate()
                .map(|(i, hit)| lexical_candidate(hit, i as u32 + 1))
                .collect::<Result<Vec<_>>>()?;
        }
    }

    let mode_used = match (!semantic_items.is_empty(), !lexical_candidates.is_empty()) {
        (true, true) => RetrievalMode::Hybrid,
        (true, false) => RetrievalMode::Semantic,
        (false, true) => RetrievalMode::Lexical,
        (false, false) => mode,
    };
    let mut fused = fuse(semantic_items, lexical_candidates);
    let mut deduped = 0u32;
    let mut seen_text = HashSet::new();
    fused.retain(|item| {
        if seen_text.insert((item.attachment_id.clone(), item.text_sha256.clone())) {
            true
        } else {
            deduped += 1;
            false
        }
    });
    let mut items = Vec::new();
    let mut used_bytes = 0u32;
    let mut budget_exhausted = false;
    for candidate in fused {
        if items.len() >= limit as usize {
            break;
        }
        let bytes = candidate.text.len() as u32;
        if used_bytes.saturating_add(bytes) > budget_bytes {
            budget_exhausted = true;
            break;
        }
        used_bytes += bytes;
        items.push(RetrievalHit {
            job_id: candidate.job_id,
            attachment_id: candidate.attachment_id,
            generation_id: candidate.source_generation_id,
            extractor: candidate.extractor,
            extractor_version: candidate.extractor_version,
            fragment_ordinal: candidate.fragment_ordinal,
            text: candidate.text,
            text_sha256: candidate.text_sha256,
            locator: candidate.locator,
            source_state: candidate.source_state,
            semantic_score: candidate.semantic_score,
            semantic_rank: candidate.semantic_rank,
            lexical_score: candidate.lexical_score,
            lexical_rank: candidate.lexical_rank,
            fused_score: candidate.fused,
        });
    }
    let coverage = semantic::coverage(&c, &control)?;
    Ok(RetrievalPage {
        version: 1,
        query: query.to_owned(),
        mode_requested: mode,
        mode_used,
        degraded,
        profile_fingerprint: active.as_ref().map(|g| g.fingerprint.clone()),
        coverage: RetrievalCoverage {
            semantic_generation: active.as_ref().map(|g| g.generation_id.clone()),
            semantic_chunks: coverage.active_chunks,
            semantic_stale_chunks: coverage.stale_chunks,
            lexical_indexed_generations: lexical_count,
            stale_excluded,
            deduplicated: deduped,
        },
        items,
        budget_bytes,
        used_bytes,
        budget_exhausted,
    })
}

fn degradable(error: &CoreError) -> bool {
    if matches!(error.code, ErrorCode::WorkspaceBusy) {
        return false;
    }
    error
        .details
        .get("reason")
        .and_then(|v| v.as_str())
        .is_some_and(|reason| {
            reason.starts_with("provider_") || reason == "semantic_search_deadline"
        })
}

fn lexical_query_supported(query: &str) -> bool {
    let terms = query.split_whitespace().collect::<Vec<_>>();
    !terms.is_empty() && terms.len() <= 16 && query.len() <= 256
}

fn lexical_candidate(hit: AttachmentSearchHit, rank: u32) -> Result<Candidate> {
    Ok(Candidate {
        job_id: hit.source.job_id,
        attachment_id: hit.source.attachment_id,
        source_generation_id: hit.source.generation_id,
        extractor: hit.source.extractor,
        extractor_version: hit.source.extractor_version,
        fragment_ordinal: hit.ordinal,
        text_sha256: format!("{:x}", sha2::Sha256::digest(hit.text.as_bytes())),
        text: hit.text,
        locator: hit.locator,
        source_state: RetrievalSourceState::Current,
        semantic_score: None,
        semantic_rank: None,
        lexical_score: Some(hit.score),
        lexical_rank: Some(rank),
        fused: 0.0,
    })
}

fn semantic_candidates(
    c: &rusqlite::Connection,
    active: &SemanticGenerationInfo,
    query: &str,
    filter: &IndexFilter,
    include_stale: bool,
    limit: u32,
    runtime_override: Option<&dyn EmbeddingRuntime>,
) -> Result<(Vec<Candidate>, u32)> {
    let http_runtime;
    let runtime: &dyn EmbeddingRuntime = match runtime_override {
        Some(runtime) => runtime,
        None => {
            http_runtime =
                HttpEmbeddingRuntime::new(active.profile.clone(), active.endpoint.clone())?;
            &http_runtime
        }
    };
    let mut query_vector = runtime
        .embed(
            EmbeddingPurpose::Query,
            &[query.to_owned()],
            Duration::from_millis(EMBEDDING_QUERY_TIMEOUT_MS),
        )?
        .into_iter()
        .next()
        .ok_or_else(|| unavailable("provider_response_incomplete"))?;
    if query_vector.len() != active.dimension as usize {
        return Err(unavailable("provider_dimension_mismatch"));
    }
    semantic::normalize(&mut query_vector).map_err(|_| unavailable("provider_vector_invalid"))?;
    let semantic_filter = semantic::SemanticFilter {
        job_id: filter.job_id.clone(),
        attachment_id: filter.attachment_id.clone(),
        extractor: filter.extractor.map(|e| e.as_str().to_owned()),
    };
    let chunks = semantic::load_chunks(c, &active.generation_id, &semantic_filter)?;
    let (states, excluded) = semantic::stale_filter(c, &chunks, include_stale)?;
    let deadline = Instant::now() + Duration::from_millis(SEMANTIC_SEARCH_DEADLINE_MS);
    let mut scored = Vec::new();
    for chunk in &chunks {
        let Some(state) = states.get(&chunk.ordinal) else {
            continue;
        };
        if Instant::now() >= deadline {
            return Err(storage("semantic_search_deadline"));
        }
        scoped::check_shutdown()?;
        let vector = semantic::decode_vector(&chunk.vector, active.dimension)?;
        let score = semantic::dot(&query_vector, &vector).clamp(-1.0, 1.0);
        scored.push((
            score,
            chunk.job_id.clone(),
            chunk.source_generation_id.clone(),
            chunk.fragment_ordinal,
            *state,
            chunk,
        ));
    }
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
    });
    let take = ((limit * 4) as usize).min(SEMANTIC_CANDIDATES as usize);
    let mut out = Vec::new();
    for (rank, (score, _, _, _, state, chunk)) in scored.into_iter().take(take).enumerate() {
        out.push(Candidate {
            job_id: chunk.job_id.clone(),
            attachment_id: chunk.attachment_id.clone(),
            source_generation_id: chunk.source_generation_id.clone(),
            extractor: chunk.extractor.clone(),
            extractor_version: chunk.extractor_version,
            fragment_ordinal: chunk.fragment_ordinal,
            text: chunk.text.clone(),
            text_sha256: chunk.text_sha256.clone(),
            locator: chunk.locator.clone(),
            source_state: state,
            semantic_score: Some(score),
            semantic_rank: Some(rank as u32 + 1),
            lexical_score: None,
            lexical_rank: None,
            fused: 0.0,
        });
    }
    Ok((out, excluded))
}

fn fuse(mut semantic: Vec<Candidate>, lexical: Vec<Candidate>) -> Vec<Candidate> {
    const K: f64 = 60.0;
    let mut by_key = HashMap::<(String, String, u32), usize>::new();
    let mut out: Vec<Candidate> = Vec::new();
    for candidate in semantic.drain(..) {
        let key = candidate.key();
        let rank = candidate.semantic_rank.unwrap_or(1) as f64;
        let mut candidate = candidate;
        candidate.fused = 1.0 / (K + rank);
        by_key.insert(key, out.len());
        out.push(candidate);
    }
    for candidate in lexical {
        let key = candidate.key();
        let rank = candidate.lexical_rank.unwrap_or(1) as f64;
        if let Some(index) = by_key.get(&key) {
            let existing = &mut out[*index];
            existing.lexical_score = candidate.lexical_score;
            existing.lexical_rank = candidate.lexical_rank;
            existing.fused += 1.0 / (K + rank);
        } else {
            let mut candidate = candidate;
            candidate.fused = 1.0 / (K + rank);
            by_key.insert(key, out.len());
            out.push(candidate);
        }
    }
    out.sort_by(|a, b| {
        b.fused
            .partial_cmp(&a.fused)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.job_id.cmp(&b.job_id))
            .then_with(|| a.source_generation_id.cmp(&b.source_generation_id))
            .then_with(|| a.fragment_ordinal.cmp(&b.fragment_ordinal))
    });
    out
}

#[cfg(test)]
#[path = "semantic_tests.rs"]
mod tests;
