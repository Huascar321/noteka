use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScalarKind {
    Number,
    Decimal,
    Boolean,
    Text,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CalculationType {
    pub kind: ScalarKind,
    pub currency: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    And,
    Or,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Aggregate {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expression {
    Number {
        value: f64,
    },
    Decimal {
        value: String,
        currency: Option<String>,
    },
    Boolean {
        value: bool,
    },
    Text {
        value: String,
    },
    Property {
        property_id: String,
    },
    Binary {
        operator: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Not {
        value: Box<Expression>,
    },
    If {
        condition: Box<Expression>,
        then: Box<Expression>,
        otherwise: Box<Expression>,
    },
    Rollup {
        relation_id: String,
        property_id: Option<String>,
        aggregate: Aggregate,
        #[serde(default)]
        inverse: bool,
    },
    Allocate {
        relation_id: String,
        cost_id: String,
        weight_id: String,
        scale: u32,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FormulaSet {
    pub collection_id: String,
    pub expected_revision: u32,
    pub property_id: Option<String>,
    pub name: String,
    pub expression: Expression,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CalculationError {
    pub code: String,
    pub message: String,
}
