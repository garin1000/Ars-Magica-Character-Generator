//! Pure rules engine for Ars Magica 5th Edition character and covenant generation.

#![deny(clippy::all)]

pub mod ability;
pub mod art;
pub mod characteristics;
pub mod derived;
pub mod effective;
pub mod equipment;
pub mod grant;
pub mod house;
pub mod mythic_companion;
pub mod ruleset;
pub mod spell;
pub mod types;
pub mod validation;

pub use ability::{
    Ability, AbilityCategory, AbilityXpRow, AdvancementTable, AgeAbilityCap, AgeAbilityCaps,
};
pub use art::{Art, ArtType};
pub use characteristics::{Characteristic, CharacteristicCost, CharacteristicRules};
pub use derived::{
    Addend, CastingTotal, CastingWithinFocus, CombatLine, DerivedTotals, EncumbranceTotal,
    FatigueLevel, LabTotal, LongevityBonus, MagicResistance, MasterpieceCap, NonStandardCasting,
    PenetrationLine, SoakTotal, SurfacedModifier, WoundRange, casting_totals, combat_totals,
    derived_totals, encumbrance, fatigue_levels, lab_totals, longevity_bonus, magic_resistance,
    masterpiece_item_cap, penetration, soak, surfaced_modifiers, wound_ranges,
};
pub use effective::{
    AbilityBonus, AbilityFloor, ArtBonus, CharacteristicBonus, Confidence, ReputationGrant,
    RestrictedXpPool, SupernaturalFreeSlots, Warping, XpAllocation, ability_bonus, ability_bonuses,
    ability_score_floors, age_ability_cap, age_max_ability_score, art_bonus, art_bonuses,
    characteristic_aging_drops, characteristic_bonuses, characteristic_cap, characteristic_caps,
    characteristic_floor, characteristic_floors, characteristic_points_granted,
    characteristic_score_bonus, confidence, decrepitude_points_total, decrepitude_score,
    effective_ability_score, effective_art_score, effective_characteristic_after_aging,
    effective_characteristic_score, effective_characteristics, effective_might,
    effective_spell_mastery, entity_grants, item_level_budget, item_level_used,
    power_levels_budget, powers_used, reputation_grants, resolved_spell_level, restricted_xp_pools,
    size, spell_levels_budget, spell_levels_used, spell_mastery_floor, spell_mastery_xp,
    supernatural_free_slots, true_faith, warping, warping_points_total, warping_score,
    xp_allocation,
};
pub use equipment::{Armor, Shield, Weapon, WeaponKind};
pub use grant::{Grant, GrantConstraint, open_pick_satisfies, resolve_grants};
pub use house::{House, LineageType};
pub use mythic_companion::{MythicCompanionType, RequiredFlaw};
pub use ruleset::{IntegrityError, LocalizedRuleset, Ruleset, RulesetError, RulesetSources};
pub use spell::{Spell, SpellDuration, SpellRange, SpellTarget};
pub use types::{
    AbilityScore, AdvancementSource, AgingEffect, ArtScore, CastingScope, CategoryCap,
    Classification, CombatStat, Effect, EnchantedDevice, Entity, EntityKind, EntityTypeProfile,
    EquipmentSlot, Familiar, GiftPolicy, HalvableTotal, HealthTrack, I18nEntry, Id, ItemKind,
    LineRange, LoadedEntity, LongevityRitual, LongevitySource, MagicResistanceEffect, Magnitude,
    MightScore, ParamType, ParameterDef, ParameterDomain, PersonalityTrait, PointBudget, PointItem,
    Prereq, Realm, Reputation, ReputationType, RulesetRef, SCHEMA_VERSION, Selection, SourceRef,
    SpecialCasting, SpellSelection, SupernaturalPower, TalismanAttunement, TwilightScar,
    ValidationMode, load_entity_migrating,
};
pub use validation::{
    Balance, IssueSeverity, PointCeilings, ValidationIssue, ValidationResult, compute_balance,
    effective_point_ceilings, validate,
};
