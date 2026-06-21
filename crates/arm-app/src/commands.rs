//! Thin `#[tauri::command]` shims. They resolve the rules directory and managed
//! state, then delegate to the webview-free logic in [`crate::ruleset_io`].

use std::sync::RwLock;

use arm_rules::{Entity, LocalizedRuleset, Ruleset, ValidationMode, ValidationResult};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::AppError;
use crate::ruleset_io;

/// Holds the parsed ruleset so validation does not re-read and re-check the
/// rules files on every keystroke. `None` until `load_ruleset` succeeds.
#[derive(Default)]
pub struct AppState {
    pub ruleset: RwLock<Option<Ruleset>>,
}

/// Loads the ruleset for `lang`, caches it in managed state, and returns the
/// localized snapshot (mechanics + display text) for the frontend.
#[tauri::command]
pub fn load_ruleset(
    lang: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<LocalizedRuleset, AppError> {
    let rules_dir = app
        .path()
        .resolve("rules", BaseDirectory::Resource)
        .map_err(|e| AppError::Io {
            message: e.to_string(),
        })?;

    let localized = ruleset_io::load_ruleset_from_dir(&rules_dir, &lang)?;

    *state.ruleset.write().expect("ruleset lock poisoned") = Some(localized.ruleset.clone());

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
    Ok(ruleset_io::validate_loaded(&entity, ruleset, mode))
}

/// E2E seam: when set, save/load use this fixed path instead of opening a
/// native dialog. Native GTK dialogs can't be driven by WebDriver, so the
/// real-binary e2e sets this so the flow is deterministic and headless.
const E2E_FILE_ENV: &str = "ARM_E2E_FILE";

fn e2e_file_override() -> Option<std::path::PathBuf> {
    std::env::var_os(E2E_FILE_ENV).map(std::path::PathBuf::from)
}

/// Prompts for a destination and writes the entity as canonical JSON.
/// Returns the chosen path, or `None` if the dialog was cancelled.
#[tauri::command]
pub async fn save_entity(entity: Entity, app: AppHandle) -> Result<Option<String>, AppError> {
    let path = match e2e_file_override() {
        Some(path) => path,
        None => {
            let Some(file) = app
                .dialog()
                .file()
                .add_filter("Character", &["json"])
                .blocking_save_file()
            else {
                return Ok(None);
            };
            file.into_path().map_err(|e| AppError::Io {
                message: e.to_string(),
            })?
        }
    };

    ruleset_io::save_entity_to_path(&entity, &path)?;
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Prompts for a file and deserializes the entity from it.
/// Returns `None` if the dialog was cancelled.
#[tauri::command]
pub async fn load_entity(app: AppHandle) -> Result<Option<Entity>, AppError> {
    let path = match e2e_file_override() {
        Some(path) => path,
        None => {
            let Some(file) = app
                .dialog()
                .file()
                .add_filter("Character", &["json"])
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
    Ok(Some(entity))
}
