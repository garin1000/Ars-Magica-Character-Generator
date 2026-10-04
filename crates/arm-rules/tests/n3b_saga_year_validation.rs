//! N3b (D84.2 follow-up, Norbert 2026-10-04, option b): "the engine's normal
//! validation checks the pair every time and shows the existing warning 'saga year
//! before birth year'."
//!
//! Before this, `saga_year_before_birth_year` came only from the age derivation
//! (`validation/saga.rs::age_in_saga_year`), which the UI asks only when the birth
//! year is edited. N3 holds a typed birth year to `saga_year - 1`, so the derivation
//! never sees an impossible pair any more — but a save can still hold one
//! (hand-written, or the saga year moved back under the birth year later), and
//! opening it showed nothing. Now the one evaluation path, `validate`, reports it.
//!
//! No rulebook citation: no passage forbids an impossible date; the advisory exists
//! because `age` is a `u32` measured from these two years.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Entity, EntityKind, Id, RulesetRef};
use arm_rules::{CreationPhase, IssueSeverity, ValidationIssue, age_in_saga_year, validate};

fn shipped_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

/// A companion in a saga set in `saga_year`, born in `birth_year` (or with none set).
fn companion(saga_year: i32, birth_year: Option<i32>) -> Entity {
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.saga_year = saga_year;
    entity.birth_year = birth_year;
    entity
}

/// The `saga_year_before_birth_year` findings the ordinary validation pass reports.
fn saga_findings(entity: &Entity) -> Vec<ValidationIssue> {
    validate(entity, &shipped_ruleset())
        .issues
        .into_iter()
        .filter(|issue| issue.code == ValidationIssue::CODE_SAGA_YEAR_BEFORE_BIRTH_YEAR)
        .collect()
}

#[test]
fn validation_warns_when_a_stored_birth_year_is_after_the_saga_year() {
    let findings = saga_findings(&companion(1220, Some(1250)));

    assert_eq!(findings.len(), 1, "exactly one finding: {findings:?}");
    let issue = &findings[0];
    // The same finding the derivation has always raised: a warning on the Concept
    // step, which owns both fields, carrying both years for the message.
    assert_eq!(issue.severity, IssueSeverity::Warning);
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
fn validation_is_silent_for_a_birth_year_before_the_saga_year() {
    let findings = saga_findings(&companion(1220, Some(1219)));
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn validation_is_silent_for_a_birth_year_in_the_saga_year_itself() {
    // Age 0 is unusual but possible; the derivation has never warned about it either.
    let findings = saga_findings(&companion(1220, Some(1220)));
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn validation_is_silent_when_no_birth_year_is_set() {
    let findings = saga_findings(&companion(1220, None));
    assert!(findings.is_empty(), "{findings:?}");
}

/// One source of the rule: whatever the derivation says about a pair, the
/// validation pass says about the same pair stored on an entity — same code, same
/// severity, same phase, same args.
#[test]
fn validation_and_the_age_derivation_agree_on_every_pair() {
    for (saga_year, birth_year) in [
        (1220, 1250),
        (1220, 1221),
        (1220, 1220),
        (1220, 1190),
        (1197, 1220),
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
    ] {
        let derived = age_in_saga_year(saga_year, birth_year, None).issues;
        let validated = saga_findings(&companion(saga_year, Some(birth_year)));
        assert_eq!(
            validated, derived,
            "saga year {saga_year}, birth year {birth_year}"
        );
    }
}
