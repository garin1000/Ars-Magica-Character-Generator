//! Pure, webview-free command logic. Every function here takes explicit inputs
//! (paths, entities, refs) and no Tauri `State`/`AppHandle`, so the integration
//! tests in `tests/commands.rs` exercise the real logic without a running app.
//! The thin `#[tauri::command]` shims in `commands.rs` only resolve the rules
//! directory and managed state, then delegate here.

use std::fs;
use std::path::{Path, PathBuf};

use std::collections::BTreeMap;

use arm_rules::{
    AbilityBonus, AbilityFloor, ArtBonus, Characteristic, CharacteristicBonus, Confidence, Entity,
    EntityKind, Grant, LocalizedRuleset, MightScore, PointCeilings, ReputationType,
    RestrictedXpPool, Ruleset, RulesetSources, Selection, SpellLevelCap, SupernaturalFreeSlots,
    ValidationMode, ValidationResult, WarpingOwed, ability_bonuses, ability_score_floors,
    age_ability_cap, art_bonuses, characteristic_aging_drops, characteristic_bonuses,
    characteristic_caps, characteristic_floors, characteristic_points_granted, confidence,
    decrepitude_score, effective_characteristics, effective_might, effective_point_ceilings,
    entity_grants, item_level_budget, item_level_used, power_levels_budget, powers_used,
    reputation_grants, size, spell_level_caps, spell_levels_base, spell_levels_budget,
    spell_levels_used, spell_mastery_advancement_affinity, spell_mastery_floor, spell_mastery_xp,
    supernatural_free_slots, true_faith, validate, warping, warping_owed, warping_owed_grants,
    xp_allocation,
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
    /// Net Characteristic-buy points granted by Improved / Weak Characteristics,
    /// on top of the ruleset's base `start_points`. Signed (Weak subtracts).
    pub characteristic_points_granted: i32,
    /// Free starting-score floors a virtue grants to an ability (e.g. Second
    /// Sight → Second Sight 1), for the ability row's effective-score display.
    pub ability_score_floors: Vec<AbilityFloor>,
    /// The character's derived Size (base 0; Large +1, Giant Blood +2, Small
    /// Frame −1, Dwarf −2), for the character-sheet Size readout.
    pub size: i32,
    /// Free effective-score bonuses to Characteristics (Giant Blood +1 Str/Sta,
    /// Dwarf −1), one per affected Characteristic, for the sheet to show the
    /// effective score alongside the bought one.
    pub characteristic_bonuses: Vec<CharacteristicBonus>,
    /// Effective Characteristic score after aging drops AND free virtue deltas,
    /// keyed by Characteristic — only the entries that differ from the bought
    /// score (the UI falls back to the bought score for the rest). The engine
    /// owns the floor clamp, so the UI never re-implements it.
    pub characteristic_effective: BTreeMap<Characteristic, i32>,
    /// Aging-drop count per Characteristic (only the non-zero entries), for the
    /// effective-score tooltip breakdown.
    pub characteristic_aging_drops: BTreeMap<Characteristic, u32>,
    /// Virtue/Flaw Selections the entity's House grants (derived, never persisted),
    /// so the V/F view renders them read-only without re-deriving. Emitted in the
    /// House's declared grant order for a stable UI + snapshot ordering.
    pub granted_selections: Vec<Selection>,
    /// Effective virtue/flaw point ceilings (base budget + Mythic Companion type
    /// bonus) so the balance bar shows the true budget (a Devil Child's 37/17,
    /// not the base 20/10). Budget numbers stay engine-authoritative.
    pub virtue_budget: u32,
    pub flaw_budget: u32,
    /// The magus's effective spell-levels budget (base + Skilled/Weak Parens
    /// modifiers) — the "available" side of the spell-levels bar. The base is the
    /// per-character `spell_levels_override` when set, else the type profile's base.
    pub spell_levels_budget: u32,
    /// The type profile's base spell-levels budget (120 for a magus), surfaced so
    /// the override field's placeholder shows the data-driven default rather than a
    /// hardcoded literal. Ignores the per-character override and V/F modifiers.
    pub spell_levels_profile_base: u32,
    /// The spell levels the chosen spells consume — the "used" side of the bar.
    pub spell_levels_used: u32,
    /// Per-Technique/Form maximum learnable spell level (Te + Fo + Int + Magic
    /// Theory + 3), so the spell picker greys a spell above the magus's cap
    /// without recomputing the derivation in JS. Empty for a non-magus (no Spells
    /// tab). Engine-authoritative; the UI only reads it.
    pub spell_level_caps: Vec<SpellLevelCap>,
    /// Effective Confidence Score / Points (type default + V/F), for the read-only
    /// Confidence readout. 0/0 for grogs (who have no Confidence).
    pub confidence_score: u8,
    pub confidence_points: u8,
    /// The Gift's free Supernatural-Ability slots: how many the character has
    /// (1 for a Gifted non-magus, else 0) and how many are already used. The
    /// ability picker greys a Supernatural Ability when `used >= total` and it is
    /// not already granted by a Virtue.
    pub supernatural_free_total: u8,
    pub supernatural_free_used: u8,
    /// The character's age → max-Ability-score cap (base, before Affinity's +2),
    /// surfaced so the UI shows one source of truth. `None` when age is unset.
    pub age_ability_cap: Option<u8>,
    /// The Reputation grants the character's V/F confer, so the UI only offers a
    /// Reputation add-control (pre-filled kind/score) when one exists.
    pub reputation_grants: Vec<ReputationGrant>,
    /// Derived Warping Score / Points: the UNIFIED total of stored Warping Points
    /// (`Entity::warping_points`) plus any granted by V/F (Warped by Magic → +5),
    /// with the score derived by inverting the advancement curve (15 points → 2).
    /// 0/0 when there is no Warping. Engine-authoritative; never recomputed in JS.
    pub warping_score: u8,
    pub warping_points: u32,
    /// The off-budget Virtues/Flaws a non-magus character owes from its Warping
    /// Score ("Effects of Warping", Core:16547-16561): the per-kind owed counts
    /// (for the "you gain N …" read-out). All zero for magi (exempt — Twilight
    /// instead) and any character owing nothing. Engine-authoritative.
    pub warping_owed: WarpingOwed,
    /// One OPEN grant per owed warping slot (stable `choice_key` + the constraint
    /// its fill must satisfy), so the UI renders one picker per slot filtered to
    /// eligible items. Empty for magi and characters owing nothing.
    pub warping_owed_grants: Vec<Grant>,
    /// Derived Decrepitude Score: the sum of accrued aging points across all
    /// Characteristics (`Entity::aging_points`) inverted through the advancement
    /// curve (17 aging points → Decrepitude 2). 0 when there are no aging points.
    pub decrepitude_score: u8,
    /// Derived True Faith Score granted by V/F (True Faith → 1); 0 when none.
    pub true_faith_score: u8,
    /// Derived starting enchanted-device level budget (Magic Items +25, Redcap
    /// 50); 0 when none. The character starts with this many levels of devices.
    pub item_level_budget: u32,
    /// The total device level the entity's `devices` consume — the "used" side of
    /// the item-level budget bar (engine-authoritative; the UI never recomputes it).
    pub item_level_used: u32,
    /// Derived Spell-Mastery XP pool (Mastered Spells +50 each); 0 when none.
    pub spell_mastery_xp: u32,
    /// Mastery-score floor every known spell gets (Flawless Magic → 1); 0 = none.
    pub spell_mastery_floor: u8,
    /// Whether a Virtue doubles all Spell-Mastery Advancement Totals (Flawless
    /// Magic), halving the XP each mastery point costs — so the UI's mastery
    /// accounting charges the same reduced cost the engine does.
    pub spell_mastery_advancement_doubled: bool,
    /// The supernatural being's effective Might Score + Realm (base + same-Realm
    /// Virtue grants), or `None` for an ordinary character. Engine-authoritative.
    pub might: Option<MightScore>,
    /// Derived power-levels budget the being's Might Virtues grant (Demonic Blood
    /// 30, Demonic Powers +20); 0 when none — the "available" side of the bar.
    pub power_levels_budget: u32,
    /// The total power level the being's `powers` consume — the "used" side of the
    /// power-levels bar (engine-authoritative; the UI never recomputes it).
    pub power_levels_used: u32,
}

/// A Reputation a Virtue/Flaw authorizes the character to start with (the UI
/// pre-fills a new Reputation row from this; content is player-supplied).
#[derive(Debug, Clone, Serialize)]
pub struct ReputationGrant {
    pub kind: ReputationType,
    pub score: u8,
}

/// Computes the score effects for `entity` against a loaded ruleset.
pub fn effective_scores_loaded(entity: &Entity, ruleset: &Ruleset) -> EffectiveScores {
    let allocation = xp_allocation(entity, ruleset);
    let ceilings = effective_point_ceilings(entity, ruleset).unwrap_or(PointCeilings {
        virtue_ceiling: 0,
        flaw_ceiling: 0,
    });
    let profile = ruleset.profile(&entity.type_id);
    let spell_base = spell_levels_base(entity, profile);
    // Confidence is derived (type default + V/F); 0/0 when there is no profile.
    let confidence = profile
        .map(|p| confidence(p.confidence_score, p.confidence_points, entity, ruleset))
        .unwrap_or(Confidence {
            score: 0,
            points: 0,
        });
    let supernatural_free = profile
        .map(|p| supernatural_free_slots(entity, ruleset, p))
        .unwrap_or(SupernaturalFreeSlots { total: 0, used: 0 });
    let warping = warping(entity, ruleset);
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
        size: size(entity, ruleset),
        characteristic_bonuses: characteristic_bonuses(entity, ruleset),
        characteristic_effective: effective_characteristics(entity, ruleset),
        characteristic_aging_drops: characteristic_aging_drops(entity),
        granted_selections: entity_grants(entity, ruleset),
        virtue_budget: ceilings.virtue_ceiling,
        flaw_budget: ceilings.flaw_ceiling,
        spell_levels_budget: spell_levels_budget(spell_base, entity, ruleset),
        spell_levels_profile_base: profile.map(|p| p.spell_levels).unwrap_or(0),
        spell_levels_used: spell_levels_used(entity, ruleset),
        // The per-Te/Fo cap only matters on the (magus-only) Spells tab, so it is
        // computed only for a magus — other types ship an empty list.
        spell_level_caps: if profile.is_some_and(|p| p.is_magus) {
            spell_level_caps(entity, ruleset)
        } else {
            Vec::new()
        },
        confidence_score: confidence.score,
        confidence_points: confidence.points,
        supernatural_free_total: supernatural_free.total,
        supernatural_free_used: supernatural_free.used,
        age_ability_cap: age_ability_cap(entity, ruleset),
        // A player-chosen-kind grant (`kind == None`, e.g. Famous) authorizes any
        // type, so it is surfaced to the UI as one add-control per Reputation
        // type; concrete-kind grants pass through unchanged. Validation still
        // enforces the single-slot count (see `validate_reputations`).
        reputation_grants: reputation_grants(entity, ruleset)
            .into_iter()
            .flat_map(|grant| match grant.reputation_type {
                Some(kind) => vec![ReputationGrant {
                    kind,
                    score: grant.score,
                }],
                None => ReputationType::ALL
                    .into_iter()
                    .map(|kind| ReputationGrant {
                        kind,
                        score: grant.score,
                    })
                    .collect(),
            })
            .collect(),
        warping_score: warping.score,
        warping_points: warping.points,
        warping_owed: warping_owed(entity, ruleset),
        warping_owed_grants: warping_owed_grants(entity, ruleset),
        decrepitude_score: decrepitude_score(entity, ruleset),
        true_faith_score: true_faith(entity, ruleset),
        item_level_budget: item_level_budget(entity, ruleset),
        item_level_used: item_level_used(entity),
        spell_mastery_xp: spell_mastery_xp(entity, ruleset),
        spell_mastery_floor: spell_mastery_floor(entity, ruleset),
        spell_mastery_advancement_doubled: spell_mastery_advancement_affinity(entity, ruleset)
            .is_some(),
        might: effective_might(entity, ruleset),
        power_levels_budget: power_levels_budget(entity, ruleset),
        power_levels_used: powers_used(entity),
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

/// Given ordered candidate rules directories, returns the first one that
/// actually holds the rules data, or `None` when none do.
///
/// Tauri's `BaseDirectory::Resource` does not resolve to the executable's own
/// directory for a portable Linux build: `resource_dir` there falls back to a
/// system path (`/usr/lib/<name>`) that a portable extract never populates. So
/// the command offers both the resource path and the directory next to the
/// executable as candidates and lets this pick whichever is real. A candidate is
/// considered valid when it contains `core/character_types.json`, a required
/// rules file.
pub fn pick_rules_dir(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|dir| dir.join("core/character_types.json").is_file())
        .cloned()
}

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
    let spells_json = fs::read_to_string(rules_dir.join("core/spells.json"))?;
    let spell_mastery_abilities_json =
        fs::read_to_string(rules_dir.join("core/spell_mastery_abilities.json"))?;
    let equipment_json = fs::read_to_string(rules_dir.join("core/equipment.json"))?;
    let characteristics_json = fs::read_to_string(rules_dir.join("core/characteristics.json"))?;

    let ruleset = Ruleset::from_sources(RulesetSources {
        id: RULESET_ID,
        version: RULESET_VERSION,
        point_items: &point_items_json,
        type_profiles: &type_profiles_json,
        abilities: Some(&abilities_json),
        arts: Some(&arts_json),
        houses: Some(&houses_json),
        mythic_types: Some(&mythic_types_json),
        spells: Some(&spells_json),
        spell_mastery_abilities: Some(&spell_mastery_abilities_json),
        equipment: Some(&equipment_json),
        // An empty characteristics file means the ruleset ships no characteristic
        // rules (the `Option` is the engine's honest "absent" signal).
        characteristics: (!characteristics_json.is_empty())
            .then_some(characteristics_json.as_str()),
    })?;
    // Load the requested language's rules text. For any non-English language,
    // English is loaded as a per-field fallback so a not-yet-translated string
    // (e.g. a missing German spell description) surfaces the English text rather
    // than rendering empty. English needs no fallback (it is the source of truth).
    let i18n = read_i18n_sources(rules_dir, lang)?;
    let i18n_refs: Vec<&str> = i18n.iter().map(String::as_str).collect();
    let localized = if lang == "en" {
        LocalizedRuleset::from_merged(ruleset, &i18n_refs)?
    } else {
        let fallback = read_i18n_sources(rules_dir, "en")?;
        let fallback_refs: Vec<&str> = fallback.iter().map(String::as_str).collect();
        LocalizedRuleset::from_merged_with_fallback(ruleset, &i18n_refs, &fallback_refs)?
    };
    Ok(localized)
}

/// Reads the eight `i18n/<lang>/*.json` rules-text files for a language, in the
/// stable domain order the localized ruleset merges them. A missing file is an
/// error (each language ships the full set), surfaced to the caller.
fn read_i18n_sources(rules_dir: &Path, lang: &str) -> Result<Vec<String>, AppError> {
    const FILES: [&str; 8] = [
        "virtues_flaws.json",
        "abilities.json",
        "arts.json",
        "houses.json",
        "mythic_companion_types.json",
        "spells.json",
        "spell_mastery_abilities.json",
        "equipment.json",
    ];
    FILES
        .iter()
        .map(|file| {
            fs::read_to_string(rules_dir.join(format!("i18n/{lang}/{file}")))
                .map_err(AppError::from)
        })
        .collect()
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
    // Stamp the current schema version so app-written saves never drift from the
    // engine's `SCHEMA_VERSION` (a save always reflects the shape it was written by).
    canonical.schema_version = arm_rules::SCHEMA_VERSION;
    canonical.normalize();
    let json = serde_json::to_string_pretty(&canonical)?;
    fs::write(path, json)?;
    Ok(())
}

/// Reads and deserializes an entity from `path`, applying save migrations.
///
/// A pre-schema-10 save's manual `aging_reductions` are folded into `aging_points`
/// (aging drops are now derived); the migration is logged so a stale save is
/// visibly upgraded on load. See [`arm_rules::load_entity_migrating`].
pub fn load_entity_from_path(path: &Path) -> Result<Entity, AppError> {
    let json = fs::read_to_string(path)?;
    let loaded = arm_rules::load_entity_migrating(&json)?;
    if !loaded.migrated_aging_characteristics.is_empty() {
        let characteristics: Vec<String> = loaded
            .migrated_aging_characteristics
            .iter()
            .map(|c| c.to_string())
            .collect();
        eprintln!(
            "save migration: folded legacy aging_reductions into aging_points for {} ({})",
            path.display(),
            characteristics.join(", ")
        );
    }
    Ok(loaded.entity)
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
