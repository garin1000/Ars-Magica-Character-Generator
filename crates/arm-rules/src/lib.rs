//! Pure rules engine for Ars Magica 5th Edition character and covenant generation.

#![deny(clippy::all)]

pub mod ability;
pub mod characteristics;
pub mod ruleset;
pub mod types;
pub mod validation;

pub use ability::{Ability, AbilityCategory, AbilityXpRow, AdvancementTable};
pub use characteristics::{Characteristic, CharacteristicCost, CharacteristicRules};
pub use ruleset::{IntegrityError, LocalizedRuleset, Ruleset, RulesetError};
pub use types::{
    Entity, EntityKind, EntityTypeProfile, FlawCategoryCap, GiftPolicy, I18nEntry, Id, ItemKind,
    LineRange, Magnitude, ParamType, ParameterDef, ParameterDomain, PointBudget, PointItem, Prereq,
    RulesetRef, Selection, SourceRef, ValidationMode,
};
pub use validation::{
    Balance, IssueSeverity, ValidationIssue, ValidationResult, compute_balance, validate,
};
