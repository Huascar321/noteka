use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum BatchOperation {
    #[serde(rename = "entity.create")]
    EntityCreate,
    #[serde(rename = "entity.update")]
    EntityUpdate,
    #[serde(rename = "collection.create")]
    CollectionCreate,
    #[serde(rename = "property.define")]
    PropertyDefine,
    #[serde(rename = "formula.set")]
    FormulaSet,
    #[serde(rename = "record.create")]
    RecordCreate,
    #[serde(rename = "record.attach")]
    RecordAttach,
    #[serde(rename = "record.update")]
    RecordUpdate,
    #[serde(rename = "view.save")]
    ViewSave,
}
impl BatchOperation {
    pub fn name(self) -> &'static str {
        match self {
            Self::EntityCreate => "entity.create",
            Self::EntityUpdate => "entity.update",
            Self::CollectionCreate => "collection.create",
            Self::PropertyDefine => "property.define",
            Self::FormulaSet => "formula.set",
            Self::RecordCreate => "record.create",
            Self::RecordAttach => "record.attach",
            Self::RecordUpdate => "record.update",
            Self::ViewSave => "view.save",
        }
    }
}
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BatchExport {
    EntityId,
    EntityRevision,
    CollectionId,
    CollectionRevision,
    TitlePropertyId,
    PropertyId,
    ViewId,
    ViewRevision,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ExportValue {
    Id(String),
    Revision(u32),
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BindingKind {
    Value,
    PropertyKey,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportSource {
    pub step: String,
    pub export: BatchExport,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub kind: BindingKind,
    pub target: String,
    pub source: ExportSource,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchStep {
    pub key: String,
    pub operation: BatchOperation,
    pub input: Value,
    #[serde(default)]
    pub bindings: Vec<Binding>,
}
fn yes() -> bool {
    true
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchCheck {
    pub key: String,
    pub input: Value,
    #[serde(default)]
    pub bindings: Vec<Binding>,
    #[serde(default = "yes")]
    pub require_complete: bool,
    #[serde(default = "yes")]
    pub require_no_calculation_errors: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchPlan {
    pub version: u32,
    pub batch_id: String,
    pub steps: Vec<BatchStep>,
    #[serde(default)]
    pub checks: Vec<BatchCheck>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchReceipt {
    pub version: u32,
    pub workspace_id: String,
    pub path_hash: String,
    pub plan_hash: String,
    pub base_hash: String,
    pub effect_hash: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApplyBatch {
    pub plan: BatchPlan,
    pub expected: BatchReceipt,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchStepResult {
    pub key: String,
    pub operation: BatchOperation,
    pub data: Value,
    pub exports: BTreeMap<BatchExport, ExportValue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchCheckResult {
    pub key: String,
    pub data: crate::database_contracts::DatabasePage,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchDiff {
    pub kind: String,
    pub key: Vec<String>,
    pub before: Option<Value>,
    pub after: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HistorySummary {
    pub entity_id: String,
    pub operation: String,
    pub previous_revision: u32,
    pub new_revision: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchPreview {
    pub applied: bool,
    pub batch_id: String,
    pub receipt: BatchReceipt,
    pub evaluated_at: String,
    pub steps: Vec<BatchStepResult>,
    pub checks: Vec<BatchCheckResult>,
    pub diff: Vec<BatchDiff>,
    pub history_summary: Vec<HistorySummary>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchApplied {
    pub applied: bool,
    pub batch_id: String,
    pub executed_at: String,
    pub steps: Vec<BatchStepResult>,
    pub checks: Vec<BatchCheckResult>,
    pub diff: Vec<BatchDiff>,
    pub history_summary: Vec<HistorySummary>,
}
