//! Pure rules engine for Ars Magica 5th Edition character and covenant generation.

#![deny(clippy::all)]

pub mod ruleset;
pub mod types;
pub mod validation;

pub use ruleset::{IntegrityError, LocalizedRuleset, Ruleset, RulesetError};
pub use types::*;
pub use validation::{IssueSeverity, ValidationIssue, ValidationResult, compute_balance, validate};
