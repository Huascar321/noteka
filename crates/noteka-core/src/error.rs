use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ValidationError,
    NotFound,
    RevisionConflict,
    WorkspaceBusy,
    StorageError,
    UnsupportedWorkspaceVersion,
    UnsupportedDocumentFormat,
    UnsupportedTemplateFormat,
    InvalidBackup,
    DestinationNotEmpty,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoreError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Value,
}

impl CoreError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: json!({}),
        }
    }
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationError, message)
    }
    pub fn exit_code(&self) -> i32 {
        match self.code {
            ErrorCode::ValidationError
            | ErrorCode::DestinationNotEmpty
            | ErrorCode::InvalidBackup => 2,
            ErrorCode::NotFound => 3,
            ErrorCode::RevisionConflict | ErrorCode::WorkspaceBusy => 4,
            ErrorCode::StorageError => 5,
            ErrorCode::UnsupportedWorkspaceVersion
            | ErrorCode::UnsupportedDocumentFormat
            | ErrorCode::UnsupportedTemplateFormat => 6,
        }
    }
}
impl From<rusqlite::Error> for CoreError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(ref e, _)
                if e.code == rusqlite::ErrorCode::OperationInterrupted =>
            {
                let mut error = Self::new(
                    ErrorCode::StorageError,
                    "La operación SQLite fue interrumpida.",
                );
                error.details = json!({"sqlite_interrupt":true});
                error
            }
            rusqlite::Error::SqliteFailure(ref e, _)
                if matches!(
                    e.code,
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                ) =>
            {
                Self::new(
                    ErrorCode::WorkspaceBusy,
                    "Workspace ocupado; reintentá la operación.",
                )
            }
            rusqlite::Error::QueryReturnedNoRows => Self::new(
                ErrorCode::NotFound,
                "No se encontró el recurso en este workspace.",
            ),
            _ => Self::new(
                ErrorCode::StorageError,
                "No se pudo completar la operación SQLite.",
            ),
        }
    }
}
impl From<std::io::Error> for CoreError {
    fn from(_: std::io::Error) -> Self {
        Self::new(
            ErrorCode::StorageError,
            "No se pudo acceder al almacenamiento local.",
        )
    }
}
impl From<serde_json::Error> for CoreError {
    fn from(_: serde_json::Error) -> Self {
        Self::validation("JSON inválido o campos no admitidos.")
    }
}
