//! The saga year, and the age ↔ birth-year link derived from it
//! (guided-creation-review-2026-08 #25).
//!
//! The saga year itself is deliberately **not** here: it is not a rule, so it is not
//! in the ruleset. Where it *is* changed in C8 — it lives on the entity
//! ([`crate::Entity::saga_year`], schema 17), because a storyguide runs more than one
//! saga and the app-settings file it used to live in holds exactly one number. That
//! number was right for one saga and silently wrong for every other, which meant
//! wrong ages and a spurious "not born yet" advisory on any character from the
//! others. What `settings.json` keeps is only the year a *new* document starts at.
//!
//! The cost of that choice, accepted with it: advancing a saga by a year means
//! touching each character. The named-sagas alternative (a list of sagas, each with a
//! year, referenced by id) was considered and rejected as a whole new domain concept.
//!
//! Either way the year reaches this module as a plain argument, and this crate still
//! reads no file.
//!
//! What *is* here is the arithmetic and the one advisory an impossible pair
//! produces, so the clamp policy has a single home and no caller — Rust or
//! frontend — restates it. The derivation has no mechanical effect: `age` and
//! `birth_year` are both already stored on the entity, and this only fills one in
//! from the other while the user types. The advisory is also part of the ordinary
//! validation pass (N3b, D84.2), from the same helper, so a stored impossible pair is
//! reported on every pass rather than only when the birth year is edited.

use super::{ValidationIssue, args};
use crate::types::{CreationPhase, Entity};

/// The calendar year the published setting stands in, and therefore the saga year
/// a fresh installation assumes — the fallback of last resort, behind both the
/// document's own [`crate::Entity::saga_year`] and the user's configured default for
/// new documents.
///
/// A rules value, not a preference: *"That domination persists until the present
/// day, 1220."*
/// Source: ArMDE:597
/// (corroborated at `ArMDE:364` — *"much like the Europe of 1220"* — and `ArMDE:440`).
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
///
/// `max_age` is the ruleset's app maximum age (`AgingRules::max_age`, an app limit
/// rather than a rule): the derived age clamps at it, silently — the birth-year
/// input is capped at `saga_year - max_age`, so only a crafted call reaches the
/// clamp. `None` applies no cap.
pub fn age_in_saga_year(saga_year: i32, birth_year: i32, max_age: Option<u32>) -> AgeInSagaYear {
    if let Some(issue) = not_born_yet(saga_year, birth_year) {
        return AgeInSagaYear {
            age: 0,
            issues: vec![issue],
        };
    }
    let years = i64::from(saga_year) - i64::from(birth_year);
    // The widest legal span (i32::MAX - i32::MIN) exceeds u32::MAX by one, so
    // saturate rather than wrap on a hand-edited extreme.
    let age = u32::try_from(years).unwrap_or(u32::MAX);
    AgeInSagaYear {
        age: max_age.map_or(age, |max_age| age.min(max_age)),
        issues: Vec::new(),
    }
}

/// Emits `saga_year_before_birth_year` when the entity's stored birth year is after
/// its saga year (N3b, D84.2). Part of [`super::validate`], so a save that already
/// holds such a pair — hand-written, or with the saga year moved back under the
/// birth year, which rewrites neither value — says so the moment it is opened, not
/// only when the birth year is next edited. An unset birth year says nothing.
pub(crate) fn validate_saga_year(entity: &Entity, issues: &mut Vec<ValidationIssue>) {
    let Some(birth_year) = entity.birth_year else {
        return;
    };
    issues.extend(not_born_yet(entity.saga_year, birth_year));
}

/// The one statement of the rule [`age_in_saga_year`] and [`validate_saga_year`]
/// share: a saga year before the birth year is a character not yet born. The same
/// year is age 0, unusual but possible, and advises nothing.
fn not_born_yet(saga_year: i32, birth_year: i32) -> Option<ValidationIssue> {
    if saga_year >= birth_year {
        return None;
    }
    Some(ValidationIssue::warning(
        ValidationIssue::CODE_SAGA_YEAR_BEFORE_BIRTH_YEAR,
        CreationPhase::Concept,
        args([
            ("saga_year", saga_year.to_string()),
            ("birth_year", birth_year.to_string()),
        ]),
        None,
    ))
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
        let derived = age_in_saga_year(DEFAULT_SAGA_YEAR, 1190, None);
        assert_eq!(derived.age, 30);
        assert!(derived.issues.is_empty());
    }

    #[test]
    fn the_same_year_is_age_zero_without_an_advisory() {
        // Born this year: unusual for a player character, but not impossible.
        let derived = age_in_saga_year(DEFAULT_SAGA_YEAR, DEFAULT_SAGA_YEAR, None);
        assert_eq!(derived.age, 0);
        assert!(derived.issues.is_empty(), "{:?}", derived.issues);
    }

    #[test]
    fn a_saga_year_one_before_the_birth_year_already_warns() {
        let derived = age_in_saga_year(DEFAULT_SAGA_YEAR, DEFAULT_SAGA_YEAR + 1, None);
        assert_eq!(derived.age, 0);
        assert_eq!(derived.issues.len(), 1);
        assert_eq!(derived.issues[0].severity, IssueSeverity::Warning);
        assert_eq!(derived.issues[0].phase, CreationPhase::Concept);
    }
}
