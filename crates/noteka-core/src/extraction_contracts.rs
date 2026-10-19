use crate::error::CoreError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_EXTRACTION_BYTES: u32 = 1_048_576;
pub const EXTRACTION_CHUNK_BYTES: u32 = 4096;
pub const MAX_EXTRACTION_FRAGMENTS: u32 = 512;
pub const MAX_EXTRACTION_PENDING: u32 = 1000;
pub const MAX_EXTRACTION_JOBS: u32 = 10_000;
pub const MAX_DERIVED_BYTES: u64 = 32 * 1024 * 1024;
pub const EXTRACTION_RESPONSE_BYTES: usize = 2_097_152;
pub const EXTRACTION_EXECUTION_MS: u64 = 5000;
pub const EXTRACTION_LEASE_SECONDS: i64 = 15;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExtractionConfig {
    pub version: u32,
    pub chunk_bytes: u32,
}
impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            version: 1,
            chunk_bytes: EXTRACTION_CHUNK_BYTES,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Stale,
}
impl ExtractionState {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Stale => "stale",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnqueueExtraction {
    pub attachment_id: String,
    pub extractor: String,
    pub config: Option<ExtractionConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListExtraction {
    pub attachment_id: Option<String>,
    pub state: Option<ExtractionState>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunExtraction {
    pub id: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExtractionRevision {
    pub id: String,
    pub expected_revision: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExtractionControlInput {
    Get {},
    Set {
        paused: bool,
        expected_revision: u32,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReadFragments {
    pub id: String,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractionControl {
    pub version: u32,
    pub paused: bool,
    pub revision: u32,
    pub updated_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractionGeneration {
    pub id: String,
    pub attachment_id: String,
    pub source_sha256: String,
    pub extractor: String,
    pub extractor_version: u32,
    pub config: Value,
    pub config_hash: String,
    pub source_bytes: u32,
    pub covered_bytes: u32,
    pub fragment_count: u32,
    pub complete: bool,
    pub published_at: String,
    pub attempt: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractionJob {
    pub id: String,
    pub attachment_id: String,
    pub source_sha256: String,
    pub source_bytes: u32,
    pub extractor: String,
    pub extractor_version: u32,
    /// Crudo: una configuración futura no se reserializa en persistencia al leer.
    pub config: Value,
    pub config_hash: String,
    pub supported: bool,
    pub unsupported_reason: Option<String>,
    pub state: ExtractionState,
    pub revision: u32,
    pub attempts: u32,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<CoreError>,
    pub ownership_active: bool,
    pub recoverable: bool,
    pub generation: Option<ExtractionGeneration>,
    pub result_from_previous_attempt: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnqueuedExtraction {
    pub job: ExtractionJob,
    pub deduplicated: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionOutcome {
    Idle,
    Succeeded,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractionRun {
    pub outcome: ExtractionOutcome,
    pub reason: Option<String>,
    pub job: Option<ExtractionJob>,
    pub recovered_count: u32,
    pub incompatible_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractionFragment {
    pub ordinal: u32,
    pub text: String,
    pub byte_start: u32,
    pub byte_end: u32,
    pub line_start: u32,
    pub line_end: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FragmentPage {
    pub job_id: String,
    pub generation: ExtractionGeneration,
    pub result_from_previous_attempt: bool,
    pub items: Vec<ExtractionFragment>,
    pub covers_generation: bool,
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn strict_extraction_shapes_and_defaults() {
        assert_eq!(ExtractionConfig::default().version, 1);
        assert_eq!(ExtractionConfig::default().chunk_bytes, 4096);
        for v in [
            json!({"action":"get","paused":true}),
            json!({"action":"set","paused":false}),
            json!({"action":"set","paused":true,"expected_revision":-1}),
            json!({"action":"set","paused":true,"expected_revision":4294967296u64}),
            json!({"action":"get","sql":"SELECT 1"}),
        ] {
            assert!(
                serde_json::from_value::<ExtractionControlInput>(v.clone()).is_err(),
                "{v}"
            );
        }
        assert!(serde_json::from_value::<EnqueueExtraction>(json!({"attachment_id":"a","extractor":"utf8_text","config":{"version":1,"chunk_bytes":4096,"unknown":1}})).is_err());
        assert!(serde_json::from_value::<RunExtraction>(json!({"token":"secret"})).is_err());
        assert!(serde_json::from_value::<ExtractionRevision>(json!({"id":"a"})).is_err());
        assert!(serde_json::from_value::<ReadFragments>(json!({"id":"a","cursor":1})).is_err());
        for state in [
            ExtractionState::Queued,
            ExtractionState::Running,
            ExtractionState::Succeeded,
            ExtractionState::Failed,
            ExtractionState::Cancelled,
            ExtractionState::Stale,
        ] {
            assert_eq!(serde_json::to_value(&state).unwrap(), state.as_str());
        }
    }
}
