//! App settings: small, non-mechanical preferences the app itself owns.
//!
//! Today that is one value, the **saga year**
//! (guided-creation-review-2026-08 #25 / D3.1). It is saga state, not character
//! state and not a rule, so it belongs neither on the entity — per-character copies
//! of one saga-wide fact would immediately disagree — nor in the ruleset. Engine
//! purity is absolute: `arm-rules` owns the default and the arithmetic
//! ([`arm_rules::DEFAULT_SAGA_YEAR`], [`arm_rules::age_in_saga_year`]) and this
//! module owns the file.
//!
//! **Path resolution follows `pick_rules_dir`'s precedent**: the caller offers a
//! list of candidates and whichever is really on disk wins, so a layout whose
//! preferred directory does not exist falls through instead of failing. The list
//! itself is deliberately different, though, and the difference is the point.
//! `load_ruleset` has to try the executable's own directory because a portable
//! Linux build's `BaseDirectory::Resource` resolves to a system path that was never
//! installed. The per-user config directory has no such failure mode — it is
//! computed from the environment, not from where the bundle was installed, so it
//! resolves to a real, writable path in a dev run, an installed bundle and a
//! portable copy alike. It is therefore first, and the executable's directory is
//! only the fallback for a machine with no resolvable config directory at all.
//!
//! A missing or unreadable file reads as the default **silently**: this runs at
//! launch, and a first launch has no settings file.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// The settings file's name, inside whichever directory resolution picks.
pub const SETTINGS_FILE_NAME: &str = "settings.json";

/// The whole settings document. Every field is optional so that a file written by
/// an older or newer build — or hand-edited down to `{}` — still parses, and each
/// absent value simply falls back to its own default.
#[derive(Debug, Default, Serialize, Deserialize)]
struct AppSettings {
    /// The calendar year the saga stands in; `None` means "never chosen", which
    /// reads as [`arm_rules::DEFAULT_SAGA_YEAR`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    saga_year: Option<i32>,
}

/// The settings files to look for, in preference order.
///
/// Pure: it touches no disk, which is what makes every layout — including the
/// portable one — testable without a Tauri runtime.
pub fn settings_candidates(config_dir: Option<&Path>, exe_dir: Option<&Path>) -> Vec<PathBuf> {
    [config_dir, exe_dir]
        .into_iter()
        .flatten()
        .map(|dir| dir.join(SETTINGS_FILE_NAME))
        .collect()
}

/// The first candidate that is really a file, or `None` when none is.
///
/// Mirrors [`crate::ruleset_io::pick_rules_dir`]: offer the candidates and let the
/// filesystem decide.
pub fn pick_settings_file(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|path| path.is_file()).cloned()
}

/// The stored saga year, or [`arm_rules::DEFAULT_SAGA_YEAR`] when there is none to
/// read.
///
/// Never fails. A missing file, an unreadable file, malformed JSON and a document
/// without the key are all the same answer, because all four mean "the user has not
/// chosen a saga year" and none of them may stop the app from starting.
pub fn read_saga_year(path: Option<&Path>) -> i32 {
    path.and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<AppSettings>(&text).ok())
        .and_then(|settings| settings.saga_year)
        .unwrap_or(arm_rules::DEFAULT_SAGA_YEAR)
}

/// Writes the saga year to exactly `path`, creating its directory if needed.
pub fn write_saga_year(path: &Path, year: i32) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let settings = AppSettings {
        saga_year: Some(year),
    };
    // Pretty-printed with a trailing newline: a settings file is small enough to be
    // read and hand-edited, and the newline keeps it diff- and editor-friendly.
    let mut json = serde_json::to_string_pretty(&settings)?;
    json.push('\n');
    fs::write(path, json)?;
    Ok(())
}

/// Writes the saga year to the best available candidate and returns the path used.
///
/// A candidate that already exists wins, so settings are never stranded and a
/// second file cannot start up beside the first and disagree with it. Otherwise the
/// candidates are tried in order and the first successful write wins — an installed
/// build's executable directory is not writable by the user, and falling through is
/// how that stays a non-event.
///
/// Unlike [`read_saga_year`], a total failure IS an error: this is a user action, and
/// silently dropping it would leave the saga year looking saved when it was not.
pub fn store_saga_year(candidates: &[PathBuf], year: i32) -> Result<PathBuf, AppError> {
    let existing = candidates.iter().filter(|path| path.is_file());
    let fresh = candidates.iter().filter(|path| !path.is_file());
    let mut refusals = Vec::new();
    for path in existing.chain(fresh) {
        match write_saga_year(path, year) {
            Ok(()) => return Ok(path.clone()),
            Err(error) => refusals.push(format!("{}: {error}", path.display())),
        }
    }
    Err(AppError::Io {
        message: format!("no writable settings file; tried: {}", refusals.join(", ")),
    })
}
