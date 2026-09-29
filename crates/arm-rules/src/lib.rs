//! Pure rules engine for Ars Magica 5th Edition character and covenant generation.

#![deny(clippy::all)]

pub mod ability;
pub mod aging;
#[doc(inline)]
pub use aging::{
    AgingError, AgingNote, AgingOutcome, AgingPointAward, AgingPointTarget, AgingRow,
    AgingRowEffect, AgingRules, AgingTotal, AgingYear, AgingYearRequest, AgingYearResult,
    CrisisAllowance, CrisisAttendant, CrisisDie, CrisisModifier, CrisisModifierSource,
    CrisisOutcome, CrisisPreview, CrisisRow, CrisisRules, CrisisSeverity, CrisisSurvival,
    CrisisTotal, LivingCondition, LivingConditionsModifier, LongevityClamp, aging_schedule,
    aging_total, crisis_preview, crisis_survival, crisis_total, living_conditions_modifier,
    resolve_crisis_row, resolve_outcome, resolve_year, revert_year,
};
pub mod art;
pub mod catalogue;
#[doc(inline)]
pub use catalogue::{
    Catalogue, CatalogueValue, load_catalogue_names, load_parameter_catalogues,
    parse_catalogue_names,
};
pub mod characteristics;
pub mod childhood;
pub mod completeness;
#[doc(inline)]
pub use childhood::{
    ChildhoodEntry, ChildhoodPackage, ChildhoodRejection, apply_childhood_package, apply_package,
};
#[doc(inline)]
pub use completeness::{CompletenessReport, completeness};
pub mod derived;
pub mod effective;
pub mod equipment;
pub mod export;
pub mod grant;
pub mod house;
pub mod life_stage;
#[doc(inline)]
pub use life_stage::{
    AbilityRequirement, AbilityRequirementKind, ApprenticeshipRules, ChildhoodRules,
    LaterLifeRules, LifeStageBudget, LifeStagePlan, LifeStageRules, MagusMinimumAbility,
    PostApprenticeshipRules, magus_minimum_abilities,
};
pub mod migration;
pub mod mythic_companion;
pub mod ruleset;
pub mod spell;
pub mod spell_mastery;
pub mod types;
pub mod validation;

pub use ability::{
    Ability, AbilityCategory, AbilityXpRow, AdvancementTable, AgeAbilityCap, AgeAbilityCaps,
};
pub use art::{Art, ArtType};
pub use characteristics::{Characteristic, CharacteristicCost, CharacteristicRules};
// Curated public compute-function facade: the bare free functions below are the
// crate's public compute API, kept flat rather than hidden behind their module
// paths. `derived_totals` is the one the app actually calls across the crate
// boundary — it rolls up every other function here — so the rest are reachable
// for a caller that wants a single total without the whole roll-up, and are
// exercised individually by this crate's own tests.
pub use derived::{
    Addend, CastingTotal, CastingWithinFocus, CombatLine, DerivedTotals, EncumbranceTotal,
    FamiliarBinding, FamiliarReadout, FatigueLevel, FatigueTier, FocusPowerLine, LabTotal,
    LongevityBonus, LongevityHint, MagicResistance, MasterpieceCap, ModifierFamily,
    NonStandardCasting, PenetrationLine, SoakTotal, SurfacedModifier, TalismanCapacity, WoundBand,
    WoundRange, casting_totals, combat_totals, cord_points_spent, derived_totals, encumbrance,
    familiar_binding_level, familiar_invested_power_levels, familiar_readout, fatigue_levels,
    focus_power_lines, lab_totals, longevity_bonus, magic_resistance, masterpiece_item_cap,
    penetration, soak, surfaced_modifiers, talisman_capacity, wound_ranges,
};
// Curated public compute-function facade (see the `derived` re-export above):
// bare names such as `size`, `warping`, `confidence`, and `true_faith` are the
// intentional crate-root compute API that `arm-app` imports by name.
pub use effective::{
    AbilityBonus, AbilityFloor, AbilityInstanceRef, AbilityParameterOptions, ArtBonus,
    CharacteristicBonus, Confidence, LifeStageBlock, LinkTarget, ReputationGrant, RestrictedXpPool,
    SpellLevelCap, SupernaturalFreeSlots, Warping, WarpingOwed, XpAllocation, XpPoolOrigin,
    XpSolveBoundExceeded, ability_bonus, ability_bonuses, ability_parameter_options,
    ability_score_floors, age_ability_cap, age_max_ability_score, art_bonus, art_bonuses,
    characteristic_aging_drops, characteristic_bonuses, characteristic_cap, characteristic_caps,
    characteristic_floor, characteristic_floors, characteristic_points_granted,
    characteristic_score_bonus, checked_xp_allocation, confidence, decrepitude_points_total,
    decrepitude_score, effective_ability_score, effective_art_score,
    effective_characteristic_after_aging, effective_characteristic_score,
    effective_characteristics, effective_might, effective_spell_mastery, entity_grants,
    focus_points_budget, focus_points_used, is_hermetically_trained, item_level_budget,
    item_level_used, life_stage_spell_levels, power_levels_budget, powers_used, reputation_grants,
    resolved_spell_level, restricted_xp_pools, size, spell_level_cap, spell_level_caps,
    spell_levels_base, spell_levels_bonus, spell_levels_budget, spell_levels_used,
    spell_mastery_advancement_affinity, spell_mastery_floor, spell_mastery_xp,
    supernatural_free_slots, true_faith, unlink_ability_parameters, warping, warping_owed,
    warping_owed_grants, warping_points_total, warping_score,
};
pub use equipment::{Armor, Shield, Weapon, WeaponKind};
pub use export::{LABEL_KEYS, character_markdown};
pub use grant::{Grant, GrantConstraint, open_pick_satisfies, resolve_grants};
pub use house::{House, LineageType};
// The save-migration subsystem lives in its own module (`migration.rs`), but its
// three public names stay at the crate root, where `arm-app` has always imported
// them from.
pub use migration::{LoadedEntity, SCHEMA_VERSION, load_entity_migrating};
pub use mythic_companion::{MythicCompanionType, RequiredFlaw};
pub use ruleset::{IntegrityError, LocalizedRuleset, Ruleset, RulesetError, RulesetSources};
pub use spell::{Spell, SpellDuration, SpellRange, SpellTarget};
pub use spell_mastery::SpellMasteryAbility;
pub use types::{
    AbilityFunding, AbilityParameterValue, AbilityScore, AdvancementFactor, AdvancementSource,
    AgingEffect, AgingLogEntry, ArtScore, CastingScope, CategoryCap, CategoryRule,
    CharacteristicDeltaCap, Classification, CombatStat, CreationPhase, Effect, EnchantedDevice,
    Entity, EntityKind, EntityTypeProfile, EquipmentSlot, Familiar, FocusPower, GiftPolicy,
    HalvableTotal, HealthTrack, I18nEntry, Id, ItemKind, ItemPredicate, LineRange, LoadoutState,
    LongevityRitual, LongevitySource, MagicResistanceEffect, Magnitude, MightScore, ParamType,
    ParameterDef, ParameterDomain, PersonalityTrait, PointBudget, PointItem, Prereq, Realm,
    Reputation, ReputationType, RulesetRef, Selection, SourceRef, SpecialCasting, SpellSelection,
    SupernaturalPower, Talisman, TalismanAttunement, TalismanEffect, TwilightScar, ValidationMode,
};
pub use validation::{
    AgeInSagaYear, Balance, DEFAULT_SAGA_YEAR, IssueSeverity, PointCeilings, ValidationIssue,
    ValidationResult, age_in_saga_year, birth_year_in_saga_year, compute_balance,
    effective_point_ceilings, phases_in_force, validate,
};
