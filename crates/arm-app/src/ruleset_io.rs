//! Pure, webview-free command logic. Every function here takes explicit inputs
//! (paths, entities, refs) and no Tauri `State`/`AppHandle`, so the integration
//! tests in `tests/commands.rs` exercise the real logic without a running app.
//! The thin `#[tauri::command]` shims in `commands.rs` only resolve the rules
//! directory and managed state, then delegate here.

use std::fs;
use std::path::Path;

use arm_rules::{Entity, LocalizedRuleset, Ruleset, ValidationMode, ValidationResult, validate};

use crate::error::AppError;

/// Stable ID + version of the shipped ruleset. These are slug-style identifiers,
/// not user-facing text, so they live in code rather than Fluent.
pub const RULESET_ID: &str = "arm5-core";
pub const RULESET_VERSION: &str = "2024.1";

/// Loads the shipped ruleset for `lang` from a rules directory laid out as
/// `core/*.json` + `i18n/<lang>/*.json`, parsing and integrity-checking it via
/// the engine. Returns the ruleset paired with localized display text.
pub fn load_ruleset_from_dir(rules_dir: &Path, lang: &str) -> Result<LocalizedRuleset, AppError> {
    let point_items_json = fs::read_to_string(rules_dir.join("core/virtues_flaws.json"))?;
    let type_profiles_json = fs::read_to_string(rules_dir.join("core/character_types.json"))?;
    let abilities_json = fs::read_to_string(rules_dir.join("core/abilities.json"))?;
    let characteristics_json = fs::read_to_string(rules_dir.join("core/characteristics.json"))?;

    let vf_i18n = fs::read_to_string(rules_dir.join(format!("i18n/{lang}/virtues_flaws.json")))?;
    let ability_i18n = fs::read_to_string(rules_dir.join(format!("i18n/{lang}/abilities.json")))?;

    let ruleset = Ruleset::from_core_json(
        RULESET_ID,
        RULESET_VERSION,
        &point_items_json,
        &type_profiles_json,
        &abilities_json,
        &characteristics_json,
    )?;
    let localized = LocalizedRuleset::from_merged(ruleset, &[&vf_i18n, &ability_i18n])?;
    Ok(localized)
}

/// Validates an entity against a loaded ruleset and applies the caller's mode
/// (Enforced keeps errors, Advisory downgrades them to warnings, Silent clears).
pub fn validate_loaded(
    entity: &Entity,
    ruleset: &Ruleset,
    mode: ValidationMode,
) -> ValidationResult {
    validate(entity, ruleset).apply_mode(mode)
}

/// Writes an entity to `path` as canonical, pretty JSON. The entity is
/// normalized first (sorting selections and parameters) so the output is
/// byte-stable for zero-noise git diffs — the engine no longer sorts implicitly
/// on serialize.
pub fn save_entity_to_path(entity: &Entity, path: &Path) -> Result<(), AppError> {
    let mut canonical = entity.clone();
    canonical.normalize();
    let json = serde_json::to_string_pretty(&canonical)?;
    fs::write(path, json)?;
    Ok(())
}

/// Reads and deserializes an entity from `path`.
pub fn load_entity_from_path(path: &Path) -> Result<Entity, AppError> {
    let json = fs::read_to_string(path)?;
    let entity: Entity = serde_json::from_str(&json)?;
    Ok(entity)
}
