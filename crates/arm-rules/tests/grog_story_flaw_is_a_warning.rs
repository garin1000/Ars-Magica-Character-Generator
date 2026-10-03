//! F10 (`tmp/ftl-rules-audit.md`): a grog's Story Flaw is discouraged, not
//! forbidden. ArMDE:1009 and :2826 say grogs "should not" take Story Flaws, so
//! Norbert ruled on 2026-10-03 that it is the soft "more than recommended"
//! warning only — never a `category_not_permitted` error.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{IssueSeverity, ValidationIssue, validate};

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

fn grog_with_animal_companion() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("grog"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    // A shipped Minor Story Flaw (ArMDE:5671-5673).
    e.selections = vec![Selection::new(Id::new("flaw.animal_companion"))];
    e
}

#[test]
fn a_grogs_story_flaw_is_not_a_category_error() {
    let rs = load_ruleset();
    let result = validate(&grog_with_animal_companion(), &rs);

    let not_permitted: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.code == ValidationIssue::CODE_CATEGORY_NOT_PERMITTED)
        .collect();
    assert!(
        not_permitted.is_empty(),
        "ArMDE:1009, :2826 — grogs 'should not' take Story Flaws; that is advice, \
         not a ban: {not_permitted:?}"
    );
}

#[test]
fn a_grogs_story_flaw_warns_more_than_recommended() {
    let rs = load_ruleset();
    let result = validate(&grog_with_animal_companion(), &rs);

    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "too_many_story_flaws" && i.severity == IssueSeverity::Warning),
        "the grog profile's soft story cap of 0 must still warn: {:?}",
        result.issues
    );
}
