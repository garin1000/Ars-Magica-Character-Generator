//! The saga year and the age ↔ birth-year link it derives
//! (guided-creation-review-2026-08 #25, Slice 12).
//!
//! The saga year itself is **not** engine state — it is an app-level setting owned
//! by `arm-app`, because it is a saga fact shared across characters rather than a
//! rule or a property of one character. What lives here is only the arithmetic and
//! the one advisory an impossible pair produces, so the clamp policy has a single
//! home and the frontend computes no mechanics of its own.

use arm_rules::{CreationPhase, IssueSeverity, ValidationIssue};

#[test]
fn the_default_saga_year_is_the_year_the_published_setting_stands_in() {
    // A rules value, not a preference: "That domination persists until the present
    // day, 1220." — Ars Magica - Definitive Edition (Core Rules).md:597
    assert_eq!(arm_rules::DEFAULT_SAGA_YEAR, 1220);
}

#[test]
fn an_age_is_the_years_between_the_birth_year_and_the_saga_year() {
    let derived = arm_rules::age_in_saga_year(1220, 1190);
    assert_eq!(derived.age, 30);
    assert!(
        derived.issues.is_empty(),
        "a possible pair advises nothing: {:?}",
        derived.issues
    );
}

#[test]
fn a_birth_year_is_the_saga_year_less_the_age() {
    assert_eq!(arm_rules::birth_year_in_saga_year(1220, 30), 1190);
    // The link is exact in both directions, so a round trip is the identity.
    assert_eq!(
        arm_rules::age_in_saga_year(1220, arm_rules::birth_year_in_saga_year(1220, 30)).age,
        30
    );
}

#[test]
fn a_saga_year_before_the_birth_year_clamps_the_age_to_zero_and_warns() {
    // `birth_year` is i32 and `age` is u32, so this subtraction is exactly where an
    // underflow would live. It clamps and says why instead.
    let derived = arm_rules::age_in_saga_year(1220, 1250);
    assert_eq!(derived.age, 0);

    assert_eq!(derived.issues.len(), 1, "{:?}", derived.issues);
    let issue = &derived.issues[0];
    assert_eq!(
        issue.code,
        ValidationIssue::CODE_SAGA_YEAR_BEFORE_BIRTH_YEAR
    );
    // Advisory, not blocking: the pair is impossible, but nothing about it makes the
    // character illegal — the saga year is an editing aid with no mechanical effect.
    assert_eq!(issue.severity, IssueSeverity::Warning);
    // The Concept step owns both fields, so that is where it can be fixed.
    assert_eq!(issue.phase, CreationPhase::Concept);
    assert_eq!(
        issue.args.get("saga_year").map(String::as_str),
        Some("1220")
    );
    assert_eq!(
        issue.args.get("birth_year").map(String::as_str),
        Some("1250")
    );
}

#[test]
fn neither_direction_overflows_at_the_edges_of_the_year_range() {
    // A hand-edited save can carry any i32 birth year and any u32 age; neither may
    // panic in a release build's wrapping arithmetic or a debug build's overflow
    // check.
    assert_eq!(arm_rules::age_in_saga_year(i32::MIN, i32::MAX).age, 0);
    assert_eq!(
        arm_rules::age_in_saga_year(i32::MAX, i32::MIN).age,
        u32::MAX,
        "the widest possible span saturates rather than wrapping"
    );
    assert_eq!(
        arm_rules::birth_year_in_saga_year(i32::MIN, u32::MAX),
        i32::MIN
    );
    assert_eq!(arm_rules::birth_year_in_saga_year(i32::MAX, 0), i32::MAX);
}
