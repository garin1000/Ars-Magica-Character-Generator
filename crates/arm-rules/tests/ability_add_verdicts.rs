//! Manual-test finding #25 (`docs/manual-testing-findings-2026-11-15.md`): in
//! Enforced mode the Ability picker does not grey out a gated category
//! (Academic, Arcane, Martial, Supernatural) the character cannot currently
//! fund — e.g. a companion with no authorizing Virtue is offered Arcane
//! Abilities. `authorization.rs::ability_add_verdicts` is the engine's one
//! verdict behind the picker's Add-button gate, the Ability twin of
//! `selections.rs::vf_add_verdicts`; these tests pin its behavior directly,
//! without going through a bought Ability score at all.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{
    AbilityAddBlockReason, ValidationIssue, ability_add_verdicts, validate,
};

const SHIPPED_HOUSES: &str = include_str!("../../../rules/core/houses.json");

fn load_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        houses: Some(SHIPPED_HOUSES),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .unwrap()
}

/// Same shape as `x1_authorization_family.rs::entity` — duplicated since
/// integration test binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

/// A companion with no selections at all — no Gift, no authorizing Virtue.
fn bare_companion() -> Entity {
    entity("companion", vec![])
}

#[test]
fn companion_without_a_virtue_is_blocked_from_an_arcane_ability() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = ability_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("ability.magic_theory")),
        Some(&AbilityAddBlockReason::RequiresVirtue),
        "a companion with no authorizing Virtue must be blocked from Magic \
         Theory (Arcane), got {verdicts:?}"
    );
}

#[test]
fn companion_with_arcane_lore_is_not_blocked_from_an_arcane_ability() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.arcane_lore"))],
    );
    let verdicts = ability_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("ability.magic_theory")),
        "Arcane Lore must authorize Magic Theory, got {verdicts:?}"
    );
}

#[test]
fn a_magus_is_never_blocked_from_an_arcane_ability() {
    let rs = load_ruleset();
    let e = entity(
        "magus",
        vec![
            Selection::new(Id::new("virtue.the_gift")),
            Selection::new(Id::new("virtue.hermetic_magus")),
        ],
    );
    let verdicts = ability_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("ability.magic_theory")),
        "a Hermetically trained magus needs no Virtue for Arcane Abilities, \
         got {verdicts:?}"
    );
}

/// Covenant Upbringing (`flaw.covenant_upbringing`, ArMDE:5938-5940)
/// authorizes Latin alone — an instance-scoped `ability_authorization`. The
/// parameterized Ability `ability.dead_language` stays addable (at least one
/// instance, Latin, can be added), but the Academic category is not opened
/// wholesale: `ability.artes_liberales` stays blocked.
#[test]
fn covenant_upbringing_keeps_dead_language_addable_but_not_the_rest_of_academic() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("flaw.covenant_upbringing"))],
    );
    let verdicts = ability_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("ability.dead_language")),
        "an instance-scoped authorization (Latin) must keep the parameterized \
         Ability addable, got {verdicts:?}"
    );
    assert_eq!(
        verdicts.get(&Id::new("ability.artes_liberales")),
        Some(&AbilityAddBlockReason::RequiresVirtue),
        "Covenant Upbringing authorizes Latin alone, not the whole Academic \
         category, got {verdicts:?}"
    );
}

#[test]
fn a_gift_less_companion_is_blocked_from_a_supernatural_ability() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = ability_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("ability.second_sight")),
        Some(&AbilityAddBlockReason::SupernaturalRequiresVirtue),
        "a companion with no Gift and no granting Virtue must be blocked from \
         a Supernatural Ability, got {verdicts:?}"
    );
}

#[test]
fn second_sight_grants_access_to_its_own_supernatural_ability() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.second_sight"))],
    );
    let verdicts = ability_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("ability.second_sight")),
        "the Second Sight Virtue must authorize its own Supernatural Ability, \
         got {verdicts:?}"
    );
}

#[test]
fn a_gifted_companion_may_add_one_supernatural_ability_via_the_free_slot() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.the_gift"))],
    );
    let verdicts = ability_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("ability.second_sight")),
        "the Gift's one free slot must leave a Supernatural Ability addable, \
         got {verdicts:?}"
    );
}

fn is_authorization_error(issue: &ValidationIssue) -> bool {
    issue.code == ValidationIssue::CODE_ABILITY_CATEGORY_REQUIRES_VIRTUE
        || issue.code == ValidationIssue::CODE_SUPERNATURAL_ABILITY_REQUIRES_VIRTUE
}

/// Agreement sweep: for every shipped character type and every catalogue
/// Ability the add-verdict does NOT already refuse on a minimal entity of
/// that type, buying it at score 1 must never make
/// [`arm_rules::validation::validate`] raise an authorization error — the
/// verdict and the validator must agree on every real catalogue entry. No
/// catalogue totals are asserted (CLAUDE.md: catalogue size is data).
#[test]
fn the_add_verdict_agrees_with_validate_on_every_authorization_issue_for_every_type() {
    let rs = load_ruleset();
    for profile in rs.profiles() {
        let base = entity(profile.id.as_str(), vec![]);
        let verdicts = ability_add_verdicts(&base, &rs);
        for ability in rs.abilities() {
            if verdicts.contains_key(&ability.id) {
                continue; // already refused — nothing to check here
            }
            let mut candidate = base.clone();
            candidate
                .ability_scores
                .push(AbilityScore::new(ability.id.clone(), 1));
            let result = validate(&candidate, &rs);
            let auth_issues: Vec<&ValidationIssue> = result
                .errors()
                .filter(|i| is_authorization_error(i))
                .collect();
            assert!(
                auth_issues.is_empty(),
                "{}: adding unblocked Ability {} raised an authorization error \
                 the verdict should have caught: {auth_issues:?}",
                profile.id,
                ability.id,
            );
        }
    }
}
