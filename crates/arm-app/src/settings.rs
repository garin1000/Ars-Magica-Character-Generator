//! App settings: small, non-mechanical preferences the app itself owns.
//!
//! Four of them today — the **default saga year for new documents**
//! (guided-creation-review-2026-08 #25 / D3.1), and since C4 the **UI language**,
//! the **palette** and the **validation mode**. None is character state and none is
//! a rule. Engine purity is absolute: `arm-rules` owns the saga-year default and
//! arithmetic ([`arm_rules::DEFAULT_SAGA_YEAR`], [`arm_rules::age_in_saga_year`]) and
//! the [`arm_rules::ValidationMode`] taxonomy, and this module owns the file.
//!
//! **The saga year itself left this file in C8**, and that is the one thing to know
//! before editing here. It used to live here outright, which meant a storyguide
//! running a 1220 Rhine saga and a 1197 Iberia saga had one number that was wrong for
//! one of them — every derived age with it. A saga year is a property of the saga, so
//! it is now [`arm_rules::Entity::saga_year`], carried in each save. What remains
//! here is genuinely a preference: the year a *new* document is stamped with. It is
//! also what [`crate::ruleset_io::load_entity_from_path`] hands the engine, since a
//! pre-schema-17 save recorded no year of its own and the engine cannot read a file
//! to find one.
//!
//! **Where each default lives, and why none of them is here.** The default saga
//! year's own default is the engine's, because it is a rules value. The language's
//! and the validation mode's are the frontend store's existing initial state. The
//! theme's is `auto`, also the frontend's — a palette has nothing whatever to do with
//! Ars Magica, so `arm-rules` must never learn what one is, and putting a copy in
//! this crate would only create a second answer to drift from the first. This module
//! therefore reports an unchosen setting as *unset* rather than inventing a value for
//! it; the single exception is the default saga year, which resolves to the engine's
//! constant so the frontend never has to name a year of its own.
//!
//! **The file is read-modify-write, never rebuilt.** [`write_settings`] takes a
//! [`SettingsPatch`] — the fields the caller is changing — and leaves every other
//! key exactly as it found it. Rebuilding the document from the field being written
//! was harmless while the saga year was the only key and silently destroying the
//! moment a second one existed: choosing a saga year would have wiped the theme and
//! the language, with no error and no undo. It is also what carries C8's key rename
//! across: a file holding the old `saga_year` is read through its serde `alias`, and
//! the next write re-homes the value under `default_saga_year` with nothing lost.
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
//! A missing or unreadable file reads as the defaults **silently**: this runs at
//! launch, and a first launch has no settings file.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::AppError;

/// The settings file's name, inside whichever directory resolution picks.
pub const SETTINGS_FILE_NAME: &str = "settings.json";

/// The settings document as it sits on disk, and equally the shape of a partial
/// update: every field that is `Some` is written, every `None` leaves whatever the
/// file already holds untouched.
///
/// One type for both because they really are the same thing — the file is nothing
/// but the accumulated patches, and a second near-identical struct would only be a
/// place for the two to disagree.
///
/// Every field is optional so that a file written by an older or newer build — or
/// hand-edited down to `{}` — still parses, and each absent value falls back on its
/// own. The fields are declared in **alphabetical order**, which is also the order
/// they serialize in: whichever setting the user changes, the file's shape is the
/// same and the diff is only ever the line that moved (CLAUDE.md's canonical
/// serialization).
///
/// `lang` and `theme` are opaque `String`s on purpose. The validation mode is an
/// engine taxonomy, so it is typed as one and the file names the very value
/// `validate_entity` already takes over IPC; a language and a palette are UI-layer
/// taxonomies the engine knows nothing about, and their sole owner — the frontend —
/// is where they are checked. Carrying them as strings here is what stops this crate
/// from becoming a second, drifting definition of either set.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsPatch {
    /// The saga year a **new** document starts at.
    ///
    /// Called `saga_year` until C8, when the saga year itself moved onto the entity
    /// (a storyguide runs more than one saga, and one machine-global number was
    /// wrong for all but one of them). `alias` is the whole of the settings
    /// migration: a file written by an older build is read under the old name and
    /// re-serialized under the new one on the next write, so the year the user
    /// chose survives and no stale duplicate is left behind to disagree with it. A
    /// settings file is user data too, and silently resetting it to 1220 would be
    /// the same data-loss class as dropping a save field.
    #[serde(
        default,
        alias = "saga_year",
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub default_saga_year: Option<i32>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub lang: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub theme: Option<String>,
    #[serde(
        default,
        deserialize_with = "lenient",
        skip_serializing_if = "Option::is_none"
    )]
    pub validation_mode: Option<arm_rules::ValidationMode>,
}

/// The settings as the app reads them: the default saga year resolved to the
/// engine's own constant when unset, and the other three reported honestly as unset
/// so their defaults stay in the one place each already lives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Settings {
    pub default_saga_year: i32,
    pub lang: Option<String>,
    pub theme: Option<String>,
    pub validation_mode: Option<arm_rules::ValidationMode>,
}

/// Parse one field, and read a value this build cannot use as simply absent.
///
/// The settings file IS the trust boundary (CLAUDE.md): it is hand-editable, and a
/// key whose value does not fit would otherwise fail the *whole* document — every
/// other setting read as unset, and then overwritten with that reading by the next
/// read-modify-write. Parsing each field on its own means an unusable value only
/// ever costs its own key.
fn lenient<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(serde_json::from_value(value).ok())
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

/// The document currently at `path`, or an empty one when there is nothing usable
/// to read. Never fails, for [`read_settings`]'s reasons.
fn read_patch(path: &Path) -> SettingsPatch {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<SettingsPatch>(&text).ok())
        .unwrap_or_default()
}

/// The stored settings, with the default saga year resolved to
/// [`arm_rules::DEFAULT_SAGA_YEAR`] when there is none to read.
///
/// Never fails. A missing file, an unreadable file, malformed JSON and a document
/// without the keys are all the same answer, because all four mean "the user has not
/// chosen" and none of them may stop the app from starting.
pub fn read_settings(path: Option<&Path>) -> Settings {
    let stored = path.map(read_patch).unwrap_or_default();
    Settings {
        default_saga_year: stored
            .default_saga_year
            .unwrap_or(arm_rules::DEFAULT_SAGA_YEAR),
        lang: stored.lang,
        theme: stored.theme,
        validation_mode: stored.validation_mode,
    }
}

/// Applies `patch` to exactly `path`, creating its directory if needed.
///
/// **Read-modify-write**: only the fields the patch names are touched, so changing
/// one setting can never destroy another. See the module header.
pub fn write_settings(path: &Path, patch: &SettingsPatch) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut settings = read_patch(path);
    if patch.default_saga_year.is_some() {
        settings.default_saga_year = patch.default_saga_year;
    }
    if patch.lang.is_some() {
        settings.lang.clone_from(&patch.lang);
    }
    if patch.theme.is_some() {
        settings.theme.clone_from(&patch.theme);
    }
    if patch.validation_mode.is_some() {
        settings.validation_mode = patch.validation_mode;
    }
    // Pretty-printed with a trailing newline: a settings file is small enough to be
    // read and hand-edited, and the newline keeps it diff- and editor-friendly.
    let mut json = serde_json::to_string_pretty(&settings)?;
    json.push('\n');
    fs::write(path, json)?;
    Ok(())
}

/// Applies `patch` to the best available candidate and returns the path used.
///
/// A candidate that already exists wins, so settings are never stranded and a
/// second file cannot start up beside the first and disagree with it. Otherwise the
/// candidates are tried in order and the first successful write wins — an installed
/// build's executable directory is not writable by the user, and falling through is
/// how that stays a non-event.
///
/// Unlike [`read_settings`], a total failure IS an error: this is a user action, and
/// silently dropping it would leave the setting looking saved when it was not.
pub fn store_settings(candidates: &[PathBuf], patch: &SettingsPatch) -> Result<PathBuf, AppError> {
    let existing = candidates.iter().filter(|path| path.is_file());
    let fresh = candidates.iter().filter(|path| !path.is_file());
    let mut refusals = Vec::new();
    for path in existing.chain(fresh) {
        match write_settings(path, patch) {
            Ok(()) => return Ok(path.clone()),
            Err(error) => refusals.push(format!("{}: {error}", path.display())),
        }
    }
    Err(AppError::Io {
        message: format!("no writable settings file; tried: {}", refusals.join(", ")),
    })
}
