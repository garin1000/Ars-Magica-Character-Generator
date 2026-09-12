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
    /// The native application menu could not be built or installed — a window
    /// -system failure, reported under its own kind rather than borrowed from
    /// [`AppError::Io`], because the frontend maps `kind` straight to
    /// `error-<kind>` and would otherwise blame a file.
    Menu { message: String },
}

impl AppError {
    /// Writes a ruleset failure's individual diagnostics to `out`, one per line
    /// under a header naming which check failed — and writes **nothing at all**
    /// for any other variant, so the one `?` chain that calls it
    /// (`commands.rs::load_ruleset`, which also raises [`AppError::Io`] when no
    /// rules directory exists) does not emit an empty header.
    ///
    /// Deliberately not [`std::fmt::Display`]: that joins the messages with
    /// `"; "` into a single line, which is unreadable once an integrity failure
    /// produces one message per violation. A reader looking for the offending id
    /// needs them stacked.
    ///
    /// **The text is English and stays English.** These are diagnostics for
    /// whoever is editing `rules/` — the same audience, and the same reasoning,
    /// as the `.md:NNNN` book references the engine spells out inside them (see
    /// `integrity.rs::validate_integrity`). They are not UI strings and have no
    /// Fluent keys; localizing them would mean translating a hundred-odd
    /// `format!` sites whose whole value is matching, verbatim, what a rules
    /// editor greps for in the JSON.
    pub fn write_ruleset_diagnostics(&self, out: &mut impl std::io::Write) -> std::io::Result<()> {
        let AppError::Ruleset {
            ruleset_kind,
            errors,
        } = self
        else {
            return Ok(());
        };
        let noun = if errors.len() == 1 {
            "problem"
        } else {
            "problems"
        };
        writeln!(
            out,
            "arm-char-gen: ruleset load failed ({ruleset_kind}): {} {noun}",
            errors.len()
        )?;
        for message in errors {
            writeln!(out, "  - {message}")?;
        }
        Ok(())
    }

    /// Prints this failure's ruleset diagnostics to stderr and hands it straight
    /// back, so a load can report itself in one link of its `?` chain:
    /// `load_ruleset_from_dir(..).map_err(AppError::reported)?`.
    ///
    /// Reporting never *consumes* the error: the frontend still receives the
    /// identical [`AppError`] and renders both its localized sentence and its
    /// technical-detail disclosure from it. The stderr copy is the second,
    /// terminal-only surface — a developer or power user who launched the binary
    /// from a shell sees the whole list without touching the UI.
    ///
    /// A failed write to stderr is ignored on purpose: a diagnostic that cannot
    /// be printed must not replace the failure it was describing.
    pub fn reported(self) -> Self {
        let _ = self.write_ruleset_diagnostics(&mut std::io::stderr().lock());
        self
    }
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
            AppError::Menu { message } => write!(f, "menu error: {message}"),
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
