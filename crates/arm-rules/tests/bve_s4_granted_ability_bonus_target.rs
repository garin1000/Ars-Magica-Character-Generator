//! bve S4 (`tmp/bve-sweep.md`): an Ability held through a Virtue's granted
//! score is a held Ability, so Puissant / Affinity naming it do not dangle.
//!
//! ArMDE:4890 (Second Sight): "Choosing this Virtue confers the Ability Second
//! Sight 1". ArMDE:4816 (Puissant Ability): "add 2 to its value whenever you
//! use it". The engine already folds the grant into the effective score
//! (`effective/ability.rs::effective_ability_score` = max(bought, floor) +
//! bonus = 3), yet `validation/selections.rs::validate_ability_bonus_targets`
//! reads only the bought `entity.ability_scores` rows when it asks whether the
//! target is held, and raises the ERROR `ability_bonus_dangling_target`. Its
//! sibling `validate_category_effect_prohibitions` already counts bought rows
//! plus `ability_score_floors`.
//!
//! Every test runs against the real shipped ruleset. No ability row is bought:
//! a granted floor costs no XP, and the Abilities tab shows it without one.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{ValidationIssue, validate};
use arm_rules::{ability_score_floors, effective_ability_score};
use std::collections::BTreeMap;

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

fn companion(selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

fn second_sight_virtue() -> Selection {
    Selection::new(Id::new("virtue.second_sight"))
}

fn naming_second_sight(item: &str) -> Selection {
    Selection::with_params(
        Id::new(item),
        BTreeMap::from([("ability".to_string(), Id::new("ability.second_sight"))]),
    )
}

fn dangling_issues(e: &Entity, rs: &Ruleset) -> Vec<ValidationIssue> {
    validate(e, rs)
        .issues
        .into_iter()
        .filter(|issue| issue.code == ValidationIssue::CODE_ABILITY_BONUS_DANGLING_TARGET)
        .collect()
}

fn assert_second_sight_is_held_only_through_the_grant(e: &Entity, rs: &Ruleset) {
    assert!(
        e.ability_scores.is_empty(),
        "precondition: no bought ability row"
    );
    let floors = ability_score_floors(e, rs);
    assert!(
        floors
            .iter()
            .any(|floor| floor.ability == Id::new("ability.second_sight")
                && floor.parameter.is_none()
                && floor.floor == 1),
        "precondition: the Second Sight Virtue grants Second Sight 1 (ArMDE:4890), got {floors:?}"
    );
}

#[test]
fn puissant_second_sight_on_the_granted_score_is_not_dangling() {
    let rs = load_ruleset();
    let e = companion(vec![
        second_sight_virtue(),
        naming_second_sight("virtue.puissant_ability"),
    ]);
    assert_second_sight_is_held_only_through_the_grant(&e, &rs);
    assert_eq!(
        effective_ability_score(&e, &rs, &Id::new("ability.second_sight"), None),
        3,
        "precondition: the engine already reads granted 1 + Puissant 2 = 3"
    );

    let dangling = dangling_issues(&e, &rs);
    assert!(
        dangling.is_empty(),
        "ArMDE:4890 + :4816: the character holds Second Sight 1, so Puissant's +2 \
         has a target; got {dangling:?}"
    );
}

#[test]
fn affinity_with_second_sight_on_the_granted_score_is_not_dangling() {
    let rs = load_ruleset();
    let e = companion(vec![
        second_sight_virtue(),
        naming_second_sight("virtue.affinity_ability"),
    ]);
    assert_second_sight_is_held_only_through_the_grant(&e, &rs);

    let dangling = dangling_issues(&e, &rs);
    assert!(
        dangling.is_empty(),
        "ArMDE:4890: the character holds Second Sight 1, so Affinity has a target; \
         got {dangling:?}"
    );
}

/// Guard, green today: with no grant and no bought row, Puissant still dangles.
#[test]
fn puissant_second_sight_without_the_virtue_or_a_row_still_dangles() {
    let rs = load_ruleset();
    let e = companion(vec![naming_second_sight("virtue.puissant_ability")]);
    assert!(
        ability_score_floors(&e, &rs).is_empty(),
        "precondition: nothing grants Second Sight"
    );

    assert_eq!(
        dangling_issues(&e, &rs).len(),
        1,
        "no score at all means the +2 attaches to nothing"
    );
}
