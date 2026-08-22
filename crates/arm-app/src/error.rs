//! Error type surfaced to the frontend. Serializes as a tagged object whose
//! outer `kind` discriminates the variant, so JS can map a stable key to a
//! Fluent message without parsing English prose. The `Ruleset` variant further
//! preserves the engine's own `kind` ("parse"/"integrity") plus the individual
//! integrity messages, rather than collapsing them into one English blob.

use arm_rules::RulesetError;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AppError {
    /// A filesystem operation failed (missing rules file, unreadable save, etc.).
    Io { message: String },
    /// A ruleset failed to parse or violated referential integrity.
    ///
    /// `ruleset_kind` carries the engine's stable discriminant
    /// ([`RulesetError::kind`]: `"parse"` or `"integrity"`) and `errors` carries
    /// one message per detected violation (a single element for parse failures).
    Ruleset {
        ruleset_kind: String,
        errors: Vec<String>,
    },
    /// A command needing a loaded ruleset ran before `load_ruleset` succeeded.
    NotLoaded,
    /// (De)serialization of an entity or save file failed.
    Serialize { message: String },
    /// A Markdown export could not resolve a document-chrome key or catalogue id
    /// to display text (see [`arm_rules::export::ExportError`]) — a stale locale
    /// bundle or a foreign/renamed id in the entity being exported. `missing`
    /// carries one message per offense, so the frontend can report every fix
    /// needed in one round trip rather than one failure at a time.
    Export { missing: Vec<String> },
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io { message } => write!(f, "io error: {message}"),
            AppError::Ruleset {
                ruleset_kind,
                errors,
            } => write!(f, "ruleset error ({ruleset_kind}): {}", errors.join("; ")),
            AppError::NotLoaded => f.write_str("no ruleset loaded"),
            AppError::Serialize { message } => write!(f, "serialize error: {message}"),
            AppError::Export { missing } => {
                write!(f, "export error: {}", missing.join("; "))
            }
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
        let ruleset_kind = e.kind().to_string();
        let errors = match e {
            RulesetError::Parse { source, message } => vec![format!("{source}: {message}")],
            RulesetError::Integrity(integrity) => integrity.errors().to_vec(),
        };
        AppError::Ruleset {
            ruleset_kind,
            errors,
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

impl From<arm_rules::export::ExportError> for AppError {
    fn from(e: arm_rules::export::ExportError) -> Self {
        AppError::Export {
            missing: e.missing.iter().map(|m| m.to_string()).collect(),
        }
    }
}
