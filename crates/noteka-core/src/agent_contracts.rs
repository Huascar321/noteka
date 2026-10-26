//! Contratos del harness local. No incluyen credenciales ni acceso genérico a
//! SQL, shell o archivos.
use crate::attachment_search_contracts::AttachmentTextLocator;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const AGENT_VERSION: u32 = 1;
pub const MAX_QUESTION_BYTES: usize = 8192;
pub const MAX_ANSWER_BYTES: usize = 65_536;
pub const MAX_QUOTE_BYTES: usize = 1024;
pub const MAX_AGENT_STEPS: u32 = 12;
pub const MIN_CONTEXT_BYTES: u32 = 4 * 1024;
pub const MAX_CONTEXT_BYTES: u32 = 256 * 1024;
pub const MIN_TOOL_BYTES: u32 = 4 * 1024;
pub const MAX_TOOL_BYTES: u32 = 256 * 1024;
pub const MIN_DEADLINE_MS: u64 = 1_000;
pub const MAX_DEADLINE_MS: u64 = 120_000;
pub const MAX_PROVIDER_TIMEOUT_MS: u64 = 60_000;
pub const MAX_TRACE_BYTES: usize = 1024 * 1024;
pub const MAX_STEP_SUMMARY_BYTES: usize = 8 * 1024;
pub const AGENT_RETENTION: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentProviderConfig {
    pub runtime: String,
    pub endpoint: String,
    pub model: String,
    pub timeout_ms: u64,
    pub max_tokens: u32,
    pub temperature: f64,
}
impl AgentProviderConfig {
    pub fn validate(&self) -> crate::error::Result<()> {
        let text = |s: &str| !s.is_empty() && s == s.trim() && !s.chars().any(|c| c.is_control());
        if self.runtime != "openai_chat_completions_v1" {
            return Err(crate::agent_contracts::agent_invalid(
                "unsupported_agent_runtime",
            ));
        }
        if !text(&self.model) || self.model.len() > 160 {
            return Err(crate::agent_contracts::agent_invalid("invalid_agent_model"));
        }
        if self.endpoint.len() > 2048
            || self.endpoint.contains('@')
            || self.endpoint.contains('#')
            || self.endpoint.contains('?')
        {
            return Err(crate::agent_contracts::agent_invalid(
                "invalid_agent_endpoint",
            ));
        }
        if !(1_000..=MAX_PROVIDER_TIMEOUT_MS).contains(&self.timeout_ms) {
            return Err(crate::agent_contracts::agent_invalid("agent_timeout_limit"));
        }
        if !(1..=4096).contains(&self.max_tokens) {
            return Err(crate::agent_contracts::agent_invalid("agent_tokens_limit"));
        }
        if !self.temperature.is_finite() || !(0.0..=2.0).contains(&self.temperature) {
            return Err(crate::agent_contracts::agent_invalid(
                "agent_temperature_limit",
            ));
        }
        crate::local_http::loopback_target(&self.endpoint)
            .map_err(|_| crate::agent_contracts::agent_invalid("agent_endpoint_not_loopback"))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentLimits {
    pub max_steps: u32,
    pub max_context_bytes: u32,
    pub max_tool_bytes: u32,
    pub deadline_ms: u64,
}
impl Default for AgentLimits {
    fn default() -> Self {
        Self {
            max_steps: 6,
            max_context_bytes: 64 * 1024,
            max_tool_bytes: 64 * 1024,
            deadline_ms: 60_000,
        }
    }
}
impl AgentLimits {
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.max_steps == 0 || self.max_steps > MAX_AGENT_STEPS {
            return Err(crate::agent_contracts::agent_invalid("agent_steps_limit"));
        }
        if self.max_context_bytes < MIN_CONTEXT_BYTES || self.max_context_bytes > MAX_CONTEXT_BYTES
        {
            return Err(crate::agent_contracts::agent_invalid("agent_context_limit"));
        }
        if self.max_tool_bytes < MIN_TOOL_BYTES || self.max_tool_bytes > MAX_TOOL_BYTES {
            return Err(crate::agent_contracts::agent_invalid("agent_tool_limit"));
        }
        if self.deadline_ms < MIN_DEADLINE_MS || self.deadline_ms > MAX_DEADLINE_MS {
            return Err(crate::agent_contracts::agent_invalid(
                "agent_deadline_limit",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentAskInput {
    pub question: String,
    pub provider: AgentProviderConfig,
    #[serde(default)]
    pub limits: AgentLimits,
    #[serde(default)]
    pub allow_apply: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunState {
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Limited,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentCitation {
    pub job_id: String,
    pub attachment_id: String,
    pub generation_id: String,
    pub extractor: String,
    pub ordinal: u32,
    pub quote: String,
    pub locator: AttachmentTextLocator,
    pub text_sha256: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStepKind {
    Model,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentStep {
    pub index: u32,
    pub kind: AgentStepKind,
    pub tool: Option<String>,
    pub duration_ms: u64,
    pub input_bytes: u32,
    pub output_bytes: u32,
    pub status: String,
    pub error: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentRunSummary {
    pub id: String,
    pub state: AgentRunState,
    pub provider: String,
    pub model: String,
    pub endpoint: String,
    pub question: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub lease_expired: bool,
    pub cancel_requested: bool,
    pub step_count: u32,
    pub answer: Option<String>,
    pub citations: Vec<AgentCitation>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentStepTrace {
    pub steps: Vec<AgentStep>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentTrace {
    pub run: AgentRunSummary,
    pub steps: Vec<AgentStep>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentTracePage {
    pub items: Vec<AgentRunSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentRunResult {
    pub run: AgentRunSummary,
    pub steps: Vec<AgentStep>,
    pub citations: Vec<AgentCitation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentCancelInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentCancelResult {
    pub run: AgentRunSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentTraceInput {
    Get {
        id: String,
    },
    List {
        limit: Option<u32>,
        cursor: Option<String>,
    },
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentTraceResult {
    Get(AgentTrace),
    List(AgentTracePage),
}

pub(crate) fn agent_invalid(reason: &str) -> crate::error::CoreError {
    let mut e = crate::error::CoreError::new(
        crate::error::ErrorCode::ValidationError,
        "La solicitud del harness no es válida.",
    );
    e.details = serde_json::json!({"reason":reason});
    e
}
pub(crate) fn agent_error(reason: &str) -> crate::error::CoreError {
    let mut e = crate::error::CoreError::new(
        crate::error::ErrorCode::StorageError,
        "No se pudo completar la operación del harness.",
    );
    e.details = serde_json::json!({"reason":reason});
    e
}
