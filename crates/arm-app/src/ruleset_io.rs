//! Pure, webview-free command logic. Every function here takes explicit inputs
//! (paths, entities, refs) and no Tauri `State`/`AppHandle`, so the integration
//! tests in `tests/commands.rs` exercise the real logic without a running app.
//! The thin `#[tauri::command]` shims in `commands.rs` only resolve the rules
//! directory and managed state, then delegate here.

use std::fs;
use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use arm_rules::{
    AbilityBonus, Characteristic, Entity, EntityKind, LocalizedRuleset, Ruleset, RulesetSources,
    ValidationMode, ValidationResult, ability_bonuses, characteristic_bonuses, validate,
};
use serde::Serialize;

use crate::error::AppError;

/// The score bonuses a character's virtues grant, for the frontend to add onto
/// each displayed bought score. Only non-zero bonuses are present. Ability bonuses
/// are per-instance (a Puissant on "Brandenburg Lore" attaches to that row alone);
/// characteristic keys are the snake_case characteristic names.
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveScores {
    /// One entry per boosted ability instance (e.g. Puissant Ability +2).
    pub ability_bonuses: Vec<AbilityBonus>,
    /// Characteristic → bonus (e.g. Great Characteristic +1).
    pub characteristic_bonuses: BTreeMap<Characteristic, i32>,
}

/// Computes the virtue score bonuses for `entity` against a loaded ruleset.
pub fn effective_scores_loaded(entity: &Entity, ruleset: &Ruleset) -> EffectiveScores {
    EffectiveScores {
        ability_bonuses: ability_bonuses(entity, ruleset),
        characteristic_bonuses: characteristic_bonuses(entity, ruleset),
    }
}

/// File extension for a saved entity of the given kind: `armc` for characters,
/// `armcov` for covenants ("Ars Magica character/covenant"). They are plain JSON
/// underneath, but a distinct extension lets the OS associate and filter them.
pub fn entity_extension(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Character => "armc",
        EntityKind::Covenant => "armcov",
    }
}

/// Default save file name for a kind, e.g. `character.armc`.
pub fn default_file_name(kind: EntityKind) -> String {
    let base = match kind {
        EntityKind::Character => "character",
        EntityKind::Covenant => "covenant",
    };
    format!("{base}.{}", entity_extension(kind))
}

/// Appends `ext` when the chosen path has no extension, so a user who types just
/// "testchar" still gets "testchar.armc". An explicit extension (`.armc`,
/// `.json`, …) the user typed is respected.
pub fn ensure_extension(path: PathBuf, ext: &str) -> PathBuf {
    if path.extension().is_none() {
        path.with_extension(ext)
    } else {
        path
    }
}

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

    let ruleset = Ruleset::from_sources(RulesetSources {
        id: RULESET_ID,
        version: RULESET_VERSION,
        point_items: &point_items_json,
        type_profiles: &type_profiles_json,
        abilities: Some(&abilities_json),
        // An empty characteristics file means the ruleset ships no characteristic
        // rules (the `Option` is the engine's honest "absent" signal).
        characteristics: (!characteristics_json.is_empty())
            .then_some(characteristics_json.as_str()),
    })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_and_default_name_per_kind() {
        assert_eq!(entity_extension(EntityKind::Character), "armc");
        assert_eq!(entity_extension(EntityKind::Covenant), "armcov");
        assert_eq!(default_file_name(EntityKind::Character), "character.armc");
        assert_eq!(default_file_name(EntityKind::Covenant), "covenant.armcov");
    }

    #[test]
    fn ensure_extension_only_fills_when_missing() {
        // No extension -> append the kind's extension.
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar"), "armc"),
            PathBuf::from("/tmp/testchar.armc")
        );
        // An explicit extension the user typed is kept (incl. .armc and .json).
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar.armc"), "armc"),
            PathBuf::from("/tmp/testchar.armc")
        );
        assert_eq!(
            ensure_extension(PathBuf::from("/tmp/testchar.json"), "armc"),
            PathBuf::from("/tmp/testchar.json")
        );
    }
}
