//! Thin `#[tauri::command]` shims. They resolve the rules directory and managed
//! state, then delegate to the webview-free logic in [`crate::ruleset_io`].

use std::sync::{Mutex, RwLock};

use arm_rules::{Entity, LocalizedRuleset, ValidationMode, ValidationResult};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use arm_rules::DerivedTotals;

use crate::error::AppError;
use crate::ruleset_io;
use crate::ruleset_io::EffectiveScores;

/// Holds the parsed, localized ruleset so validation does not re-read and
/// re-check the rules files on every keystroke. `None` until `load_ruleset`
/// succeeds.
///
/// The whole [`LocalizedRuleset`] is cached, not just its mechanics: the
/// Markdown export needs the localized display names, and re-reading the i18n
/// files per export would duplicate the loader for no gain.
#[derive(Default)]
pub struct AppState {
    pub ruleset: RwLock<Option<LocalizedRuleset>>,
    /// Mirror of the frontend's unsaved-changes state, so the window-close and
    /// app-quit handlers can prompt before discarding without a round-trip.
    pub close_guard: Mutex<CloseGuardState>,
}

/// Backend view of the close/quit guard, kept in sync by [`update_close_guard`].
#[derive(Default)]
pub struct CloseGuardState {
    /// Whether the entity has unsaved edits (mirrored from the frontend store).
    pub dirty: bool,
    /// Localized dialog strings, supplied by the frontend so no user-facing text
    /// lives in Rust.
    pub labels: CloseGuardLabels,
    /// A confirmation dialog is currently open; guards against stacking a second
    /// one when the user triggers close/quit again while it is showing.
    pub showing: bool,
    /// The user chose to discard, so the re-issued close/quit must pass through.
    pub confirmed: bool,
}

/// Localized strings for the "discard unsaved changes?" dialog. Deserialized
/// verbatim from the frontend; Rust never authors these.
#[derive(Default, Clone, serde::Deserialize)]
pub struct CloseGuardLabels {
    pub title: String,
    pub message: String,
    pub discard: String,
    pub cancel: String,
}

/// Mirrors the frontend's dirty flag and dialog strings into managed state for
/// the close/quit guard (see `main.rs`).
#[tauri::command]
pub fn update_close_guard(dirty: bool, labels: CloseGuardLabels, state: State<'_, AppState>) {
    let mut guard = state.close_guard.lock().expect("close guard lock poisoned");
    guard.dirty = dirty;
    guard.labels = labels;
}

/// Loads the ruleset for `lang`, caches it in managed state, and returns the
/// localized snapshot (mechanics + display text) for the frontend.
#[tauri::command]
pub fn load_ruleset(
    lang: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<LocalizedRuleset, AppError> {
    // Where the rules live depends on how the app was launched. For a dev run or
    // an installed bundle, `BaseDirectory::Resource` points at the right place;
    // for a portable Linux build it resolves to a nonexistent system path
    // (`/usr/lib/<name>`), so the directory next to the executable is the real
    // location. Offer both and let `pick_rules_dir` choose whichever exists.
    let mut candidates = Vec::new();
    if let Ok(resource) = app.path().resolve("rules", BaseDirectory::Resource) {
        candidates.push(resource);
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
    {
        candidates.push(exe_dir.join("rules"));
    }

    let rules_dir = ruleset_io::pick_rules_dir(&candidates).ok_or_else(|| AppError::Io {
        message: format!(
            "rules directory not found; looked in: {}",
            candidates
                .iter()
                .map(|c| c.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    })?;

    let localized = ruleset_io::load_ruleset_from_dir(&rules_dir, &lang)?;

    *state.ruleset.write().expect("ruleset lock poisoned") = Some(localized.clone());

    Ok(localized)
}

/// Validates an entity against the loaded ruleset under the given mode.
#[tauri::command]
pub fn validate_entity(
    entity: Entity,
    mode: ValidationMode,
    state: State<'_, AppState>,
) -> Result<ValidationResult, AppError> {
    let guard = state.ruleset.read().expect("ruleset lock poisoned");
    let ruleset = guard.as_ref().ok_or(AppError::NotLoaded)?;
    Ok(ruleset_io::validate_loaded(&entity, &ruleset.ruleset, mode))
}

/// Computes the effective-score bonuses (Puissant Ability, Great Characteristic)
/// the entity's virtues grant, for the frontend to display alongside the base
/// scores. Uses the same engine path as validation.
#[tauri::command]
pub fn effective_scores(
    entity: Entity,
    state: State<'_, AppState>,
) -> Result<EffectiveScores, AppError> {
    let guard = state.ruleset.read().expect("ruleset lock poisoned");
    let ruleset = guard.as_ref().ok_or(AppError::NotLoaded)?;
    Ok(ruleset_io::effective_scores_loaded(
        &entity,
        &ruleset.ruleset,
    ))
}

/// Computes the read-only play-stat totals (casting, lab, penetration, magic
/// resistance, combat, soak, encumbrance, fatigue, wounds, longevity,
/// decrepitude, warping) the character sheet displays. Pure engine path, mirrors
/// [`effective_scores`]; the frontend renders these numbers and computes no
/// mechanics itself.
#[tauri::command]
pub fn derived_totals(
    entity: Entity,
    state: State<'_, AppState>,
) -> Result<DerivedTotals, AppError> {
    let guard = state.ruleset.read().expect("ruleset lock poisoned");
    let ruleset = guard.as_ref().ok_or(AppError::NotLoaded)?;
    Ok(arm_rules::derived_totals(&entity, &ruleset.ruleset))
}

/// E2E seam: when set, save/load use this fixed path instead of opening a
/// native dialog. Native GTK dialogs can't be driven by WebDriver, so the
/// real-binary e2e sets this so the flow is deterministic and headless.
const E2E_FILE_ENV: &str = "ARM_E2E_FILE";

fn e2e_file_override() -> Option<std::path::PathBuf> {
    std::env::var_os(E2E_FILE_ENV).map(std::path::PathBuf::from)
}

/// An opened document: the deserialized entity plus the file it came from, so
/// the frontend can track it as the "current file" for subsequent direct saves.
#[derive(serde::Serialize)]
pub struct LoadedEntity {
    pub path: String,
    pub entity: Entity,
}

/// Writes the entity as canonical JSON. When `path` is `Some`, writes straight to
/// that file with no dialog (a plain Save to the current file); when `None`,
/// prompts for a destination (Save As / first Save). Returns the written path, or
/// `None` if a prompt was cancelled.
#[tauri::command]
pub async fn save_entity(
    entity: Entity,
    path: Option<String>,
    app: AppHandle,
) -> Result<Option<String>, AppError> {
    let ext = ruleset_io::entity_extension(entity.entity_kind);
    let target = match path {
        // A known current file: write directly, no dialog. `ensure_extension` is a
        // no-op for a path that already carries one.
        Some(path) => ruleset_io::ensure_extension(std::path::PathBuf::from(path), ext),
        None => match e2e_file_override() {
            Some(path) => path,
            None => {
                let Some(file) = app
                    .dialog()
                    .file()
                    // Ars Magica character/covenant files first (the default save
                    // type), then JSON for interop, then an all-files fallback.
                    .add_filter("Ars Magica character", &["armc", "armcov"])
                    .add_filter("JSON", &["json"])
                    .add_filter("All files", &["*"])
                    .set_file_name(ruleset_io::default_file_name(entity.entity_kind))
                    .blocking_save_file()
                else {
                    return Ok(None);
                };
                let chosen = file.into_path().map_err(|e| AppError::Io {
                    message: e.to_string(),
                })?;
                ruleset_io::ensure_extension(chosen, ext)
            }
        },
    };

    ruleset_io::save_entity_to_path(&entity, &target)?;
    Ok(Some(target.to_string_lossy().into_owned()))
}

/// E2E seam for the Markdown export, deliberately separate from [`E2E_FILE_ENV`]:
/// that one is a fixed `.json` path shared by save and open, so writing Markdown
/// through it would clobber the save file the same spec round-trips.
const E2E_EXPORT_FILE_ENV: &str = "ARM_E2E_EXPORT_FILE";

fn e2e_export_file_override() -> Option<std::path::PathBuf> {
    std::env::var_os(E2E_EXPORT_FILE_ENV).map(std::path::PathBuf::from)
}

/// Writes the entity as a Markdown character sheet. When `path` is `Some`, writes
/// straight to that file; when `None`, prompts for a destination. Returns the
/// written path, or `None` if a prompt was cancelled.
///
/// `labels` is the frontend's localized document chrome, keyed by the Fluent message
/// names [`export_label_keys`] lists — Rust authors none of the document's text.
#[tauri::command]
pub async fn export_markdown(
    entity: Entity,
    labels: std::collections::BTreeMap<String, String>,
    path: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Option<String>, AppError> {
    // The destination is resolved BEFORE the ruleset lock is taken: a native dialog
    // blocks until the user dismisses it, and holding the read guard across it would
    // stall a concurrent language switch (`load_ruleset` wants the write guard) for
    // exactly as long as the dialog stays open.
    let target = match path {
        Some(path) => ruleset_io::ensure_extension(
            std::path::PathBuf::from(path),
            ruleset_io::MARKDOWN_EXTENSION,
        ),
        None => match e2e_export_file_override() {
            Some(path) => path,
            None => {
                let Some(file) = app
                    .dialog()
                    .file()
                    .add_filter("Markdown", &["md"])
                    .set_file_name(ruleset_io::default_markdown_file_name(entity.entity_kind))
                    .blocking_save_file()
                else {
                    return Ok(None);
                };
                let chosen = file.into_path().map_err(|e| AppError::Io {
                    message: e.to_string(),
                })?;
                ruleset_io::ensure_extension(chosen, ruleset_io::MARKDOWN_EXTENSION)
            }
        },
    };

    let guard = state.ruleset.read().expect("ruleset lock poisoned");
    ruleset_io::export_markdown_to_path(&entity, guard.as_ref(), &labels, &target)?;
    Ok(Some(target.to_string_lossy().into_owned()))
}

/// The document-chrome label keys the Markdown export needs, so the frontend
/// resolves exactly those against its Fluent bundle and there is one source of
/// truth for the list (the engine's).
#[tauri::command]
pub fn export_label_keys() -> Vec<String> {
    arm_rules::export::LABEL_KEYS
        .iter()
        .map(|key| key.to_string())
        .collect()
}

/// Prompts for a file and deserializes the entity from it, returning it paired
/// with its path. Returns `None` if the dialog was cancelled.
#[tauri::command]
pub async fn load_entity(app: AppHandle) -> Result<Option<LoadedEntity>, AppError> {
    let path = match e2e_file_override() {
        Some(path) => path,
        None => {
            let Some(file) = app
                .dialog()
                .file()
                // Accept our own extensions and .json, plus an all-files fallback
                // so a character file is never hidden by the filter.
                .add_filter("Ars Magica character", &["armc", "armcov", "json"])
                .add_filter("All files", &["*"])
                .blocking_pick_file()
            else {
                return Ok(None);
            };
            file.into_path().map_err(|e| AppError::Io {
                message: e.to_string(),
            })?
        }
    };

    let entity = ruleset_io::load_entity_from_path(&path)?;
    Ok(Some(LoadedEntity {
        path: path.to_string_lossy().into_owned(),
        entity,
    }))
}
