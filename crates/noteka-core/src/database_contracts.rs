use crate::contracts::{Document, Entity, Page};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
pub type Values = BTreeMap<String, Value>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub revision: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateCollection {
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListCollections {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyKind {
    Formula,
    Rollup,
    Title,
    Text,
    Number,
    Decimal,
    Money,
    Checkbox,
    Date,
    DateRange,
    Select,
    MultiSelect,
    Url,
    Email,
    Files,
    Relation,
    CreatedAt,
    UpdatedAt,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PropertySpec {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: PropertyKind,
    #[serde(default)]
    pub options: Vec<String>,
    pub currency: Option<String>,
    pub target_collection_id: Option<String>,
    pub calculation_type: Option<crate::formula_contracts::CalculationType>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Property {
    pub id: String,
    pub collection_id: String,
    pub spec: PropertySpec,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DefineProperty {
    pub collection_id: String,
    pub expected_revision: u32,
    pub spec: PropertySpec,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordSeed {
    pub collection_id: String,
    #[serde(default)]
    pub values: Values,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRecord {
    pub collection_id: String,
    pub title: String,
    #[serde(default)]
    pub values: Values,
    pub document: Option<Document>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordInput {
    pub collection_id: String,
    pub entity_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateRecord {
    pub collection_id: String,
    pub entity_id: String,
    pub expected_revision: u32,
    #[serde(default)]
    pub values: Values,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecordData {
    pub entity: Entity,
    pub values: Values,
    pub collection_ids: Vec<String>,
    pub calculation_errors: BTreeMap<String, crate::formula_contracts::CalculationError>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InverseQuery {
    pub property_id: String,
    pub target_id: String,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewLayout {
    Table,
    List,
    Board,
    Calendar,
    Gallery,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Contains,
    IsEmpty,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PropertyFilter {
    pub property_id: String,
    pub op: FilterOp,
    #[serde(default)]
    pub value: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum FilterExpression {
    Condition { filter: PropertyFilter },
    All { children: Vec<FilterExpression> },
    Any { children: Vec<FilterExpression> },
    Not { child: Box<FilterExpression> },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CalendarConfig {
    pub property_id: String,
    pub month: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PropertySort {
    pub property_id: String,
    #[serde(default)]
    pub descending: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct ViewConfig {
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub filters: Vec<PropertyFilter>,
    #[serde(default)]
    pub sorts: Vec<PropertySort>,
    #[serde(default)]
    pub archived: bool,
    pub filter_tree: Option<FilterExpression>,
    pub group_by: Option<String>,
    pub calendar: Option<CalendarConfig>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SaveView {
    pub collection_id: String,
    pub id: Option<String>,
    pub expected_revision: Option<u32>,
    pub name: String,
    pub layout: ViewLayout,
    pub config: ViewConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollectionView {
    pub id: String,
    pub collection_id: String,
    pub revision: u32,
    pub name: String,
    pub layout: ViewLayout,
    pub config: ViewConfig,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DatabaseQuery {
    pub collection_id: String,
    pub view_id: Option<String>,
    pub config: Option<ViewConfig>,
    pub layout: Option<ViewLayout>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CollectionBundle {
    pub collection: Collection,
    pub properties: Vec<Property>,
    pub views: Vec<CollectionView>,
    pub calculations: BTreeMap<String, crate::formula_contracts::Expression>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabasePage {
    pub properties: Vec<Property>,
    pub config: ViewConfig,
    pub records: Page<RecordData>,
    pub layout: ViewLayout,
    pub matching_count: u32,
    pub groups: Vec<ViewGroup>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ViewGroup {
    pub key: Value,
    pub label: String,
    pub entity_ids: Vec<String>,
}
