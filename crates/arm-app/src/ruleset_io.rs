//! Pure, webview-free command logic. Every function here takes explicit inputs
//! (paths, entities, refs) and no Tauri `State`/`AppHandle`, so the integration
//! tests in `tests/commands.rs` exercise the real logic without a running app.
//! The thin `#[tauri::command]` shims in `commands.rs` only resolve the rules
//! directory and managed state, then delegate here.

use std::fs;
use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use arm_rules::{
    AbilityBonus, AbilityFloor, ArtBonus, Characteristic, Entity, EntityKind, LocalizedRuleset,
    RestrictedXpPool, Ruleset, RulesetSources, Selection, ValidationMode, ValidationResult,
    ability_bonuses, ability_score_floors, art_bonuses, characteristic_caps, characteristic_floors,
    characteristic_points_granted, granted_selections, validate, xp_allocation,
};
use serde::Serialize;

use crate::error::AppError;

/// The score effects a character's virtues/flaws produce, for the frontend.
/// Ability bonuses are per-instance and only the non-zero ones are present (a
/// Puissant on "Brandenburg Lore" attaches to that row alone). Characteristic
/// caps/floors are the per-characteristic buy limits — keyed by the snake_case
/// characteristic name and present for all eight — that Great/Poor
/// (Characteristic) widen, so the UI clamps the spinners to them.
#[derive(Debug, Clone, Serialize)]
pub struct EffectiveScores {
    /// One entry per boosted ability instance (e.g. Puissant Ability +2).
    pub ability_bonuses: Vec<AbilityBonus>,
    /// One entry per boosted Art (e.g. Puissant Art +3).
    pub art_bonuses: Vec<ArtBonus>,
    /// Characteristic → highest buyable score (Great Characteristic raises it).
    pub characteristic_caps: BTreeMap<Characteristic, i32>,
    /// Characteristic → lowest buyable score (Poor Characteristic lowers it).
    pub characteristic_floors: BTreeMap<Characteristic, i32>,
    /// Total experience the entity's Ability+Art spends demand, after Affinity
    /// reductions — the authoritative "spent" the UI shows (it must not recompute
    /// it without Affinity).
    pub xp_total_demand: u32,
    /// Experience drawn from the general pool (`Entity::xp_pool`) by the
    /// allocation; restricted pools cover the rest.
    pub xp_general_used: u32,
    /// The restricted experience pools (Educated/Warrior/Privileged) with how
    /// much of each the allocation consumes, for the per-pool XP bar.
    pub restricted_xp_pools: Vec<RestrictedXpPool>,
    /// Extra Characteristic-buy points granted by Improved Characteristics, on
    /// top of the ruleset's base `start_points`.
    pub characteristic_points_granted: u32,
    /// Free starting-score floors a virtue grants to an ability (e.g. Second
    /// Sight → Second Sight 1), for the ability row's effective-score display.
    pub ability_score_floors: Vec<AbilityFloor>,
    /// Virtue/Flaw Selections the entity's House grants (derived, never persisted),
    /// so the V/F view renders them read-only without re-deriving. Emitted in the
    /// House's declared grant order for a stable UI + snapshot ordering.
    pub granted_selections: Vec<Selection>,
}

/// Computes the score effects for `entity` against a loaded ruleset.
pub fn effective_scores_loaded(entity: &Entity, ruleset: &Ruleset) -> EffectiveScores {
    let allocation = xp_allocation(entity, ruleset);
    EffectiveScores {
        ability_bonuses: ability_bonuses(entity, ruleset),
        art_bonuses: art_bonuses(entity, ruleset),
        characteristic_caps: characteristic_caps(entity, ruleset),
        characteristic_floors: characteristic_floors(entity, ruleset),
        xp_total_demand: allocation.total_demand,
        xp_general_used: allocation.general_used,
        restricted_xp_pools: allocation.restricted,
        characteristic_points_granted: characteristic_points_granted(entity, ruleset),
        ability_score_floors: ability_score_floors(entity, ruleset),
        granted_selections: granted_selections(entity, ruleset),
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
    let arts_json = fs::read_to_string(rules_dir.join("core/arts.json"))?;
    let houses_json = fs::read_to_string(rules_dir.join("core/houses.json"))?;
    let mythic_types_json = fs::read_to_string(rules_dir.join("core/mythic_companion_types.json"))?;
    let characteristics_json = fs::read_to_string(rules_dir.join("core/characteristics.json"))?;

    let vf_i18n = fs::read_to_string(rules_dir.join(format!("i18n/{lang}/virtues_flaws.json")))?;
    let ability_i18n = fs::read_to_string(rules_dir.join(format!("i18n/{lang}/abilities.json")))?;
    let art_i18n = fs::read_to_string(rules_dir.join(format!("i18n/{lang}/arts.json")))?;
    let house_i18n = fs::read_to_string(rules_dir.join(format!("i18n/{lang}/houses.json")))?;
    let mythic_i18n =
        fs::read_to_string(rules_dir.join(format!("i18n/{lang}/mythic_companion_types.json")))?;

    let ruleset = Ruleset::from_sources(RulesetSources {
        id: RULESET_ID,
        version: RULESET_VERSION,
        point_items: &point_items_json,
        type_profiles: &type_profiles_json,
        abilities: Some(&abilities_json),
        arts: Some(&arts_json),
        houses: Some(&houses_json),
        mythic_types: Some(&mythic_types_json),
        // An empty characteristics file means the ruleset ships no characteristic
        // rules (the `Option` is the engine's honest "absent" signal).
        characteristics: (!characteristics_json.is_empty())
            .then_some(characteristics_json.as_str()),
    })?;
    let localized = LocalizedRuleset::from_merged(
        ruleset,
        &[
            &vf_i18n,
            &ability_i18n,
            &art_i18n,
            &house_i18n,
            &mythic_i18n,
        ],
    )?;
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
