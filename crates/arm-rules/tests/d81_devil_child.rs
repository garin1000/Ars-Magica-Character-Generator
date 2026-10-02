//! D81.13 (`docs/vf-audit/decisions.md`): a Devil Child's required Flaw is
//! "Tragic Life … or a suitable substitute" (ArMDE:2662). Tragic Life is a
//! Major Story Flaw by its own descriptor (ArMDE:6855-6856), so the substitute
//! must be a Major Story Flaw too, as each sibling type's substitute slot
//! already requires its own category.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, validate};

fn full_ruleset() -> Ruleset {
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
        spells: None,
        spell_mastery_abilities: None,
        equipment: None,
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: None,
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

fn devil_child_with_flaw(flaw: &str) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("mythic_companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.age = Some(25);
    e.mythic_type = Some(Id::new("mythic_type.devil_child"));
    e.selections = vec![
        Selection::new(Id::new("virtue.demonic_blood")),
        Selection::new(Id::new(flaw)),
    ];
    e
}

/// Whether the required-trait warning names Tragic Life, the slot's default.
fn tragic_life_reported_missing(e: &Entity, rs: &Ruleset) -> bool {
    validate(e, rs).issues.iter().any(|issue| {
        issue.code == ValidationIssue::CODE_MYTHIC_REQUIRED_TRAIT_MISSING
            && issue.args.get("item").map(String::as_str) == Some("flaw.tragic_life")
    })
}

#[test]
fn a_major_story_flaw_substitutes_for_tragic_life() {
    let rs = full_ruleset();
    let e = devil_child_with_flaw("flaw.black_sheep");
    assert!(
        !tragic_life_reported_missing(&e, &rs),
        "a Major Story Flaw is a suitable substitute for Tragic Life"
    );
}

#[test]
fn a_major_flaw_of_another_category_does_not_substitute_for_tragic_life() {
    let rs = full_ruleset();
    let e = devil_child_with_flaw("flaw.blind");
    assert!(
        tragic_life_reported_missing(&e, &rs),
        "Blind is a Major General Flaw, not a Story one, so it must not fill \
         the Devil Child's Tragic Life slot (D81.13)"
    );
}
