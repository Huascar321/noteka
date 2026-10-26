//! Contratos aditivos de recuperación semántica. No reutilizan cursores FTS ni
//! convierten el índice derivado en fuente de verdad.
use crate::{attachment_search_contracts::IndexedExtractor, error::*};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const SEMANTIC_PROFILE_VERSION: u32 = 1;
pub const MAX_SEMANTIC_SOURCES: u32 = 1024;
pub const MAX_SEMANTIC_CHUNKS: u32 = 8192;
pub const MAX_SEMANTIC_TEXT_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_EMBEDDING_DIMENSION: u32 = 1024;
pub const MAX_PROFILE_BYTES: usize = 8192;
pub const MAX_QUERY_BYTES: usize = 1024;
pub const MAX_RESULTS: u32 = 50;
pub const DEFAULT_RESULTS: u32 = 10;
pub const MAX_BUDGET_BYTES: u32 = 262_144;
pub const DEFAULT_BUDGET_BYTES: u32 = 16_384;
pub const MAX_ENDPOINT_BYTES: usize = 2048;
pub const MAX_EMBEDDING_BATCH: usize = 16;
pub const EMBEDDING_QUERY_TIMEOUT_MS: u64 = 30_000;
pub const SEMANTIC_BUILD_TIMEOUT_MS: u64 = 600_000;
pub const SEMANTIC_SEARCH_DEADLINE_MS: u64 = 5_000;
pub const SEMANTIC_CANDIDATES: u32 = 200;
pub const ABANDONED_BUILD_MS: u64 = 15 * 60 * 1000;

pub(crate) fn invalid(reason: &str) -> CoreError {
    let mut e = CoreError::new(
        ErrorCode::ValidationError,
        "La operación de recuperación semántica no es válida.",
    );
    e.details = json!({"reason":reason});
    e
}
pub(crate) fn storage(reason: &str) -> CoreError {
    let mut e = CoreError::new(
        ErrorCode::StorageError,
        "No se pudo completar la operación de recuperación semántica.",
    );
    e.details = json!({"reason":reason});
    e
}
pub(crate) fn unavailable(reason: &str) -> CoreError {
    let mut e = CoreError::new(
        ErrorCode::StorageError,
        "El runtime local de embeddings no está disponible.",
    );
    e.details = json!({"reason":reason});
    e
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingNormalization {
    L2,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingTask {
    Retrieval,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingChunking {
    ExtractionFragmentV1,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingQuantization {
    F32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingProfile {
    pub version: u32,
    pub provider: String,
    pub runtime: String,
    pub model: String,
    pub revision: String,
    pub dimension: u32,
    pub normalization: EmbeddingNormalization,
    pub task: EmbeddingTask,
    pub query_prefix: String,
    pub document_prefix: String,
    pub tokenizer: String,
    pub chunking: EmbeddingChunking,
    pub quantization: EmbeddingQuantization,
}
impl EmbeddingProfile {
    pub fn validate(&self) -> Result<()> {
        let text = |s: &str| {
            !s.is_empty()
                && !s.trim().is_empty()
                && s == s.trim()
                && !s.chars().any(|c| c.is_control())
        };
        if self.version != SEMANTIC_PROFILE_VERSION {
            return Err(invalid("unsupported_embedding_profile_version"));
        }
        if self.provider != "local_http" || self.runtime != "openai_embeddings_v1" {
            return Err(invalid("unsupported_embedding_provider"));
        }
        if !text(&self.model) || self.model.len() > 160 {
            return Err(invalid("invalid_embedding_model"));
        }
        if !text(&self.revision)
            || self.revision.len() > 160
            || matches!(self.revision.as_str(), "latest" | "unknown")
        {
            return Err(invalid("invalid_embedding_revision"));
        }
        if self.dimension == 0 || self.dimension > MAX_EMBEDDING_DIMENSION {
            return Err(invalid("invalid_embedding_dimension"));
        }
        if self.normalization != EmbeddingNormalization::L2
            || self.task != EmbeddingTask::Retrieval
            || self.chunking != EmbeddingChunking::ExtractionFragmentV1
            || self.quantization != EmbeddingQuantization::F32
        {
            return Err(invalid("unsupported_embedding_space"));
        }
        if !text(&self.tokenizer) || self.tokenizer.len() > 160 {
            return Err(invalid("invalid_embedding_tokenizer"));
        }
        for (name, value) in [
            ("query_prefix", &self.query_prefix),
            ("document_prefix", &self.document_prefix),
        ] {
            if value.len() > 512 || value.chars().any(|c| c.is_control() && !c.is_whitespace()) {
                let mut e = invalid("embedding_prompt_limit");
                e.details["field"] = json!(name);
                return Err(e);
            }
        }
        if self.canonical().len() > MAX_PROFILE_BYTES {
            return Err(invalid("embedding_profile_size_limit"));
        }
        Ok(())
    }
    pub fn canonical(&self) -> String {
        canonical_json(&serde_json::to_value(self).expect("perfil serializable"))
    }
    pub fn fingerprint(&self) -> String {
        let mut h = Sha256::new();
        h.update(b"noteka.embedding.profile.v1\0");
        h.update(self.canonical().as_bytes());
        format!("{:x}", h.finalize())
    }
    pub fn query_text(&self, query: &str) -> String {
        format!("{}{}", self.query_prefix, query)
    }
    pub fn document_text(&self, text: &str) -> String {
        format!("{}{}", self.document_prefix, text)
    }
}
pub(crate) fn canonical_json(value: &Value) -> String {
    fn write(value: &Value, out: &mut String) {
        match value {
            Value::Object(map) => {
                let mut keys = map.keys().collect::<Vec<_>>();
                keys.sort_unstable();
                out.push('{');
                for (i, key) in keys.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push_str(&serde_json::to_string(key).expect("clave JSON"));
                    out.push(':');
                    write(&map[key], out);
                }
                out.push('}');
            }
            Value::Array(items) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write(item, out);
                }
                out.push(']');
            }
            other => out.push_str(&serde_json::to_string(other).expect("valor JSON")),
        }
    }
    let mut out = String::new();
    write(value, &mut out);
    out
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum EmbeddingIndexInput {
    Status {},
    Build {
        profile: EmbeddingProfile,
        endpoint: String,
        job_id: Option<String>,
        attachment_id: Option<String>,
        extractor: Option<IndexedExtractor>,
    },
    Reset {
        expected_revision: u32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticGenerationState {
    Building,
    Active,
    Retired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SemanticGenerationInfo {
    pub generation_id: String,
    pub fingerprint: String,
    pub profile: EmbeddingProfile,
    pub state: SemanticGenerationState,
    pub dimension: u32,
    pub endpoint: String,
    pub source_digest: String,
    pub source_count: u32,
    pub chunk_count: u32,
    pub text_bytes: u64,
    pub created_at: String,
    pub updated_at: String,
    pub activated_at: Option<String>,
    pub retired_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SemanticCoverage {
    pub active_sources: u32,
    pub active_chunks: u32,
    pub active_text_bytes: u64,
    pub stale_sources: u32,
    pub stale_chunks: u32,
    pub unindexed_supported_generations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingStatus {
    pub version: u32,
    pub revision: u32,
    pub active: Option<SemanticGenerationInfo>,
    pub building: Option<SemanticGenerationInfo>,
    pub retired: Vec<SemanticGenerationInfo>,
    pub coverage: SemanticCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SemanticBuildResult {
    pub version: u32,
    pub revision: u32,
    pub generation: SemanticGenerationInfo,
    pub previous_generation_id: Option<String>,
    pub reused: bool,
    pub embedded_chunks: u32,
    pub duration_ms: u64,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum EmbeddingIndexResult {
    Status(EmbeddingStatus),
    Built(SemanticBuildResult),
    Reset {
        revision: u32,
        removed_generations: u32,
        removed_chunks: u32,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMode {
    Hybrid,
    Semantic,
    Lexical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalInput {
    pub query: String,
    pub mode: Option<RetrievalMode>,
    pub job_id: Option<String>,
    pub attachment_id: Option<String>,
    pub extractor: Option<IndexedExtractor>,
    pub limit: Option<u32>,
    pub budget_bytes: Option<u32>,
    #[serde(default)]
    pub include_stale: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalSourceState {
    Current,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalHit {
    pub job_id: String,
    pub attachment_id: String,
    pub generation_id: String,
    pub extractor: String,
    pub extractor_version: u32,
    pub fragment_ordinal: u32,
    pub text: String,
    pub text_sha256: String,
    pub locator: crate::attachment_search_contracts::AttachmentTextLocator,
    pub source_state: RetrievalSourceState,
    pub semantic_score: Option<f32>,
    pub semantic_rank: Option<u32>,
    pub lexical_score: Option<f64>,
    pub lexical_rank: Option<u32>,
    pub fused_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalCoverage {
    pub semantic_generation: Option<String>,
    pub semantic_chunks: u32,
    pub semantic_stale_chunks: u32,
    pub lexical_indexed_generations: u32,
    pub stale_excluded: u32,
    pub deduplicated: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetrievalPage {
    pub version: u32,
    pub query: String,
    pub mode_requested: RetrievalMode,
    pub mode_used: RetrievalMode,
    pub degraded: Vec<String>,
    pub profile_fingerprint: Option<String>,
    pub coverage: RetrievalCoverage,
    pub items: Vec<RetrievalHit>,
    pub budget_bytes: u32,
    pub used_bytes: u32,
    pub budget_exhausted: bool,
}
