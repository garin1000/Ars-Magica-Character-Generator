//! Pure rules engine for Ars Magica 5th Edition character and covenant generation.

#![deny(clippy::all)]

pub mod ability;
pub mod characteristics;
pub mod effective;
pub mod ruleset;
pub mod types;
pub mod validation;

pub use ability::{Ability, AbilityCategory, AbilityXpRow, AdvancementTable};
pub use characteristics::{Characteristic, CharacteristicCost, CharacteristicRules};
pub use effective::{
    ability_bonus, ability_bonuses, characteristic_bonus, characteristic_bonuses,
    effective_ability_score, effective_characteristic,
};
pub use ruleset::{IntegrityError, LocalizedRuleset, Ruleset, RulesetError, RulesetSources};
pub use types::{
    AbilityScore, Effect, Entity, EntityKind, EntityTypeProfile, FlawCategoryCap, GiftPolicy,
    I18nEntry, Id, ItemKind, LineRange, Magnitude, ParamType, ParameterDef, ParameterDomain,
    PointBudget, PointItem, Prereq, RulesetRef, SCHEMA_VERSION, Selection, SourceRef,
    ValidationMode,
};
pub use validation::{
    Balance, IssueSeverity, ValidationIssue, ValidationResult, compute_balance, validate,
};
