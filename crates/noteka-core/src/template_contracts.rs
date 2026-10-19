use crate::contracts::{Document, Values};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemplateDefinition {
    pub schema_version: u32,
    pub title: String,
    pub document: Document,
    #[serde(default)]
    pub values: Values,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Template {
    pub id: String,
    pub collection_id: String,
    pub name: String,
    pub revision: u32,
    pub definition: Value,
    pub editable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveTemplate {
    pub collection_id: String,
    pub expected_collection_revision: u32,
    pub id: Option<String>,
    pub expected_revision: Option<u32>,
    pub name: String,
    pub definition: TemplateDefinition,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListTemplates {
    pub collection_id: String,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
fn present_title<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InstantiateTemplate {
    pub id: String,
    pub expected_template_revision: u32,
    pub expected_collection_revision: u32,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_title"
    )]
    #[schemars(with = "String")]
    pub title: Option<String>,
    #[serde(default)]
    pub values: Values,
}
