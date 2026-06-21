//! Error type surfaced to the frontend. Serializes as a tagged object
//! `{ "kind": "...", "message": "..." }` so JS can map `kind` to a Fluent key
//! without parsing English prose.

use arm_rules::RulesetError;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppError {
    /// A filesystem operation failed (missing rules file, unreadable save, etc.).
    Io { message: String },
    /// A ruleset failed to parse or violated referential integrity.
    Ruleset { message: String },
    /// A command needing a loaded ruleset ran before `load_ruleset` succeeded.
    NotLoaded,
    /// (De)serialization of an entity or save file failed.
    Serialize { message: String },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io { message } => write!(f, "io error: {message}"),
            AppError::Ruleset { message } => write!(f, "ruleset error: {message}"),
            AppError::NotLoaded => f.write_str("no ruleset loaded"),
            AppError::Serialize { message } => write!(f, "serialize error: {message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io {
            message: e.to_string(),
        }
    }
}

impl From<RulesetError> for AppError {
    fn from(e: RulesetError) -> Self {
        AppError::Ruleset {
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialize {
            message: e.to_string(),
        }
    }
}
