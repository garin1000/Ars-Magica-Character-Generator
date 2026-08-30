//! The saga year, and the age ↔ birth-year link derived from it
//! (guided-creation-review-2026-08 #25).
//!
//! The saga year itself is deliberately **not** here, and not anywhere in this
//! crate: it is a saga fact — shared by every character in one saga, and advancing
//! as the saga is played — so it belongs neither on the entity (per-character
//! copies would immediately disagree) nor in the ruleset (it is not a rule). It
//! lives in an app-settings file owned by `arm-app`, and reaches this module as a
//! plain argument.
//!
//! What *is* here is the arithmetic and the one advisory an impossible pair
//! produces, so the clamp policy has a single home and no caller — Rust or
//! frontend — restates it. The derivation has no mechanical effect: `age` and
//! `birth_year` are both already stored on the entity, and this only fills one in
//! from the other while the user types.

use super::{ValidationIssue, args};
use crate::types::CreationPhase;

/// The calendar year the published setting stands in, and therefore the saga year
/// a fresh installation assumes.
///
/// A rules value, not a preference: *"That domination persists until the present
/// day, 1220."*
/// Source: Ars Magica - Definitive Edition (Core Rules).md:597
/// (corroborated at `:364` — *"much like the Europe of 1220"* — and `:440`).
pub const DEFAULT_SAGA_YEAR: i32 = 1220;

/// An age derived from a saga year and a birth year, with whatever the pair
/// warrants saying about it.
///
/// `Serialize` because this is a command payload: the frontend asks for the
/// derivation rather than restating the clamp policy in TypeScript.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AgeInSagaYear {
    /// The character's age in that year, clamped to the `u32` the entity stores.
    pub age: u32,
    /// Advisories about the pair. Empty for any pair that can really happen.
    pub issues: Vec<ValidationIssue>,
}

/// How old a character born in `birth_year` is in `saga_year`.
///
/// `birth_year` is `i32` and `age` is `u32`, so the subtraction is exactly where an
/// underflow would live. A saga year *before* the birth year — a character not yet
/// born, which an advancing saga year makes more likely rather than less — clamps
/// the age to 0 and says why, rather than wrapping to four billion.
///
/// Like the calendar year the aging engine computes as `birth_year + age`, this
/// ignores birthdays within the year; the two approximations are the same one.
pub fn age_in_saga_year(saga_year: i32, birth_year: i32) -> AgeInSagaYear {
    let years = i64::from(saga_year) - i64::from(birth_year);
    if years < 0 {
        return AgeInSagaYear {
            age: 0,
            issues: vec![ValidationIssue::warning(
                ValidationIssue::CODE_SAGA_YEAR_BEFORE_BIRTH_YEAR,
                CreationPhase::Concept,
                args([
                    ("saga_year", saga_year.to_string()),
                    ("birth_year", birth_year.to_string()),
                ]),
                None,
            )],
        };
    }
    AgeInSagaYear {
        // The widest legal span (i32::MAX - i32::MIN) exceeds u32::MAX by one, so
        // saturate rather than wrap on a hand-edited extreme.
        age: u32::try_from(years).unwrap_or(u32::MAX),
        issues: Vec::new(),
    }
}

/// Which year a character aged `age` in `saga_year` was born in.
///
/// The inverse of [`age_in_saga_year`], and the other half of "two views of one
/// fact". Saturates at the ends of the year range instead of wrapping; no advisory
/// is possible, because every `(saga_year, age)` pair names a real year.
pub fn birth_year_in_saga_year(saga_year: i32, age: u32) -> i32 {
    let year = i64::from(saga_year) - i64::from(age);
    i32::try_from(year).unwrap_or(i32::MIN)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::IssueSeverity;

    #[test]
    fn a_possible_pair_advises_nothing() {
        let derived = age_in_saga_year(DEFAULT_SAGA_YEAR, 1190);
        assert_eq!(derived.age, 30);
        assert!(derived.issues.is_empty());
    }

    #[test]
    fn the_same_year_is_age_zero_without_an_advisory() {
        // Born this year: unusual for a player character, but not impossible.
        let derived = age_in_saga_year(DEFAULT_SAGA_YEAR, DEFAULT_SAGA_YEAR);
        assert_eq!(derived.age, 0);
        assert!(derived.issues.is_empty(), "{:?}", derived.issues);
    }

    #[test]
    fn a_saga_year_one_before_the_birth_year_already_warns() {
        let derived = age_in_saga_year(DEFAULT_SAGA_YEAR, DEFAULT_SAGA_YEAR + 1);
        assert_eq!(derived.age, 0);
        assert_eq!(derived.issues.len(), 1);
        assert_eq!(derived.issues[0].severity, IssueSeverity::Warning);
        assert_eq!(derived.issues[0].phase, CreationPhase::Concept);
    }
}
