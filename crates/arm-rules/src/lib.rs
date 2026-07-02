//! Pure rules engine for Ars Magica 5th Edition character and covenant generation.

#![deny(clippy::all)]

pub mod ability;
pub mod art;
pub mod characteristics;
pub mod effective;
pub mod house;
pub mod ruleset;
pub mod types;
pub mod validation;

pub use ability::{Ability, AbilityCategory, AbilityXpRow, AdvancementTable};
pub use art::{Art, ArtType, ArtsFile};
pub use characteristics::{Characteristic, CharacteristicCost, CharacteristicRules};
pub use effective::{
    AbilityBonus, AbilityFloor, ArtBonus, RestrictedXpPool, XpAllocation, ability_bonus,
    ability_bonuses, ability_score_floors, art_bonus, art_bonuses, characteristic_cap,
    characteristic_caps, characteristic_floor, characteristic_floors,
    characteristic_points_granted, effective_ability_score, effective_art_score,
    restricted_xp_pools, xp_allocation,
};
pub use house::{GrantConstraint, House, HouseGrant, HousesFile, LineageType};
pub use ruleset::{IntegrityError, LocalizedRuleset, Ruleset, RulesetError, RulesetSources};
pub use types::{
    AbilityScore, ArtScore, Effect, Entity, EntityKind, EntityTypeProfile, FlawCategoryCap,
    GiftPolicy, I18nEntry, Id, ItemKind, LineRange, Magnitude, ParamType, ParameterDef,
    ParameterDomain, PointBudget, PointItem, Prereq, RulesetRef, SCHEMA_VERSION, Selection,
    SourceRef, ValidationMode,
};
pub use validation::{
    Balance, IssueSeverity, ValidationIssue, ValidationResult, compute_balance, validate,
};
