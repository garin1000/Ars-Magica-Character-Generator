//! Manual-test findings #2/#19 (`docs/manual-testing-findings-2026-10-18.md`):
//! the picker offers every catalogue Virtue/Flaw regardless of whether the
//! engine already knows, right now, that adding it is illegal — a House-gated
//! Flaw (Brutal Artist) is offered to everyone, and every Hermetic entry is
//! offered to a Gift-less companion even though the profile forbids the
//! `hermetic` category for one. `selections.rs::vf_add_verdicts` is the
//! engine's one verdict behind the picker's Add-button gate; these tests pin
//! its behavior directly, without going through a picked selection at all.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::{VfAddBlockReason, vf_add_verdicts};

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

/// Same shape as `x3_trained_gate.rs::entity` — duplicated since integration
/// test binaries cannot share private helpers.
fn entity(type_id: &str, selections: Vec<Selection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new(type_id),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = selections;
    e
}

/// A companion with no selections at all — no Gift, no House.
fn bare_companion() -> Entity {
    entity("companion", vec![])
}

#[test]
fn companion_without_the_gift_is_blocked_from_a_hermetic_virtue_by_category() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = vf_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("virtue.puissant_art")),
        Some(&VfAddBlockReason::ForbiddenCategory),
        "a Gift-less companion must be blocked from a Hermetic Virtue by category, \
         got {verdicts:?}"
    );
}

#[test]
fn the_gift_itself_is_never_blocked_for_a_gift_less_companion() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("virtue.the_gift")),
        "virtue.the_gift must never appear in the blocked map, even though its own \
         category (special) is not in the companion's permitted set — got {verdicts:?}"
    );
}

#[test]
fn magus_of_a_different_house_is_blocked_from_brutal_artist_by_prereq() {
    let rs = load_ruleset();
    let mut e = entity("magus", vec![]);
    e.house = Some(Id::new("house.tremere"));
    let verdicts = vf_add_verdicts(&e, &rs);
    assert_eq!(
        verdicts.get(&Id::new("flaw.brutal_artist")),
        Some(&VfAddBlockReason::PrereqNotMet),
        "a magus of House Tremere must be blocked from Brutal Artist by its own \
         House(house.jerbiton) prerequisite, got {verdicts:?}"
    );
}

#[test]
fn magus_of_house_jerbiton_is_not_blocked_from_brutal_artist() {
    let rs = load_ruleset();
    let mut e = entity("magus", vec![]);
    e.house = Some(Id::new("house.jerbiton"));
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("flaw.brutal_artist")),
        "a magus of House Jerbiton must not be blocked from Brutal Artist, got {verdicts:?}"
    );
}

/// `flaw.university_dean`'s prerequisite is
/// `all[has(virtue.doctor_in_faculty), age_min(40)]`. With
/// `virtue.doctor_in_faculty` held and no age set, `age_min` resolves
/// `Tri::Unknown` (unset age is Unknown, not False — the `Prereq::AgeMin` doc
/// comment), so the whole `all` resolves Unknown too (no child is a definite
/// False to short-circuit on) — and an Unknown outcome must never block.
#[test]
fn an_unevaluable_prerequisite_never_blocks() {
    let rs = load_ruleset();
    let e = entity(
        "companion",
        vec![Selection::new(Id::new("virtue.doctor_in_faculty"))],
    );
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("flaw.university_dean")),
        "an item whose prerequisite cannot yet be decided (age unset) must not be \
         blocked, got {verdicts:?}"
    );
}

#[test]
fn a_legal_item_for_the_profile_is_absent_from_the_map() {
    let rs = load_ruleset();
    let e = bare_companion();
    let verdicts = vf_add_verdicts(&e, &rs);
    assert!(
        !verdicts.contains_key(&Id::new("virtue.bureaucrat")),
        "virtue.bureaucrat (social_status, no prerequisite) must be legal for a bare \
         companion, got {verdicts:?}"
    );
}
