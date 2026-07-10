//! Pure rules engine for Ars Magica 5th Edition character and covenant generation.

#![deny(clippy::all)]

pub mod ability;
pub mod art;
pub mod characteristics;
pub mod effective;
pub mod grant;
pub mod house;
pub mod mythic_companion;
pub mod ruleset;
pub mod spell;
pub mod types;
pub mod validation;

pub use ability::{Ability, AbilityCategory, AbilityXpRow, AdvancementTable};
pub use art::{Art, ArtType};
pub use characteristics::{Characteristic, CharacteristicCost, CharacteristicRules};
pub use effective::{
    AbilityBonus, AbilityFloor, ArtBonus, RestrictedXpPool, XpAllocation, ability_bonus,
    ability_bonuses, ability_score_floors, art_bonus, art_bonuses, characteristic_cap,
    characteristic_caps, characteristic_floor, characteristic_floors,
    characteristic_points_granted, effective_ability_score, effective_art_score, entity_grants,
    resolved_spell_level, restricted_xp_pools, spell_levels_budget, spell_levels_used,
    xp_allocation,
};
pub use grant::{Grant, GrantConstraint, open_pick_satisfies, resolve_grants};
pub use house::{House, LineageType, granted_selections};
pub use mythic_companion::{MythicCompanionType, RequiredFlaw};
pub use ruleset::{IntegrityError, LocalizedRuleset, Ruleset, RulesetError, RulesetSources};
pub use spell::Spell;
pub use types::{
    AbilityScore, ArtScore, CategoryCap, Effect, Entity, EntityKind, EntityTypeProfile, GiftPolicy,
    I18nEntry, Id, ItemKind, LineRange, Magnitude, ParamType, ParameterDef, ParameterDomain,
    PointBudget, PointItem, Prereq, RulesetRef, SCHEMA_VERSION, Selection, SourceRef,
    SpellSelection, ValidationMode,
};
pub use validation::{
    Balance, IssueSeverity, ValidationIssue, ValidationResult, compute_balance,
    effective_point_ceilings, validate,
};
