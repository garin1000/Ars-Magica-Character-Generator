//! Spell Mastery per-ability limits — PHASE 1 (tests only), parked RED.
//!
//! Two gaps found by `tmp/small-catalogues-audit.md` (findings S1/S2) in
//! `validation/magus.rs::validate_spell_mastery_abilities`:
//!
//! - S1: Quiet Casting (ArMDE:9578-9580) — "A maga may take this ability
//!   twice." The catalogue's only cap mechanism is the boolean `repeatable`
//!   flag, so a third (or later) pick validates cleanly today.
//! - S2: Ceremonial Casting (ArMDE:9534), Fast Casting (ArMDE:9540), and Quick
//!   Casting (ArMDE:9576) each state "may not be taken for Ritual spells" — nothing
//!   checks the target spell's `ritual` flag before permitting one of these
//!   three.
//!
//! Proposed error codes (not yet defined anywhere — this file asserts on the
//! literal strings, per the brief, since the constants do not exist until
//! Phase 2):
//! - `too_many_of_mastery_ability` — a per-ability `max_count` is exceeded.
//! - `mastery_ability_forbidden_for_ritual` — a Ritual-forbidden ability is
//!   chosen for a spell whose `ritual` flag is `true`.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Entity, EntityKind, Id, RulesetRef, SpellSelection};
use arm_rules::validation::validate;

/// The full shipped core ruleset, including the real spell catalogue and the
/// real Spell Mastery special-ability catalogue — this screen must exercise
/// the shipped data, not a synthetic fixture, per the brief.
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
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: Some(include_str!(
            "../../../rules/core/spell_mastery_abilities.json"
        )),
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(include_str!("../../../rules/core/life_stages.json")),
        childhoods: None,
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .expect("shipped core ruleset loads")
}

fn magus(spells: Vec<SpellSelection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.spells = spells;
    e
}

/// A spell selection with a high bought mastery score, so the effective
/// mastery (`max(mastery, granted floor)`) is never the limiting factor —
/// isolates the per-ability cap under test from the separate
/// `too_many_mastery_abilities` (score-vs-count) check.
fn mastered_spell(spell_id: &str, abilities: Vec<&str>) -> SpellSelection {
    let mut s = SpellSelection::new(Id::new(spell_id));
    s.mastery = Some(10);
    s.mastery_abilities = abilities.into_iter().map(Id::new).collect();
    s
}

fn codes(e: &Entity, rs: &Ruleset) -> Vec<String> {
    validate(e, rs).issues.into_iter().map(|i| i.code).collect()
}

// --- S1: Quiet Casting capped at 2 per spell --------------------------------

/// ArMDE:9578-9580: "A maga may take this ability twice." Two picks on a
/// real (non-Ritual) shipped spell must validate clean of any per-ability cap
/// error — today this is already true (no such check exists at all), so this
/// assertion stays green across Phase 1→2 and only guards against a future
/// regression as narrow as "cap at 1".
#[test]
fn quiet_casting_twice_is_not_too_many() {
    let rs = full_ruleset();
    let e = magus(vec![mastered_spell(
        "spell.pilum_of_fire",
        vec![
            "spell_mastery_ability.quiet_casting",
            "spell_mastery_ability.quiet_casting",
        ],
    )]);
    let codes = codes(&e, &rs);
    assert!(
        !codes.contains(&"too_many_of_mastery_ability".to_string()),
        "two Quiet Casting picks must not trip a per-ability cap: {codes:?}"
    );
}

/// ArMDE:9578-9580's "twice" is a hard ceiling, not "repeatable" in the
/// unlimited sense Precise/Quick Casting get. A third pick must error with
/// the new per-ability-max code. RED today: `repeatable: true` plus no
/// `max_count` field means the validator has no way to reject this.
#[test]
fn quiet_casting_thrice_is_too_many() {
    let rs = full_ruleset();
    let e = magus(vec![mastered_spell(
        "spell.pilum_of_fire",
        vec![
            "spell_mastery_ability.quiet_casting",
            "spell_mastery_ability.quiet_casting",
            "spell_mastery_ability.quiet_casting",
        ],
    )]);
    let codes = codes(&e, &rs);
    assert!(
        codes.contains(&"too_many_of_mastery_ability".to_string()),
        "three Quiet Casting picks on the same spell must exceed its book-stated \
         cap of 2: {codes:?}"
    );
}

/// Sanity floor: Precise/Quick Casting are also `repeatable: true` but carry
/// no stated ceiling ("multiple times", ArMDE:9572, :9576) — three picks of
/// Quick Casting must NOT trip the new cap, so the fix (Phase 2) must key the
/// cap on Quiet Casting specifically, not on `repeatable` in general.
#[test]
fn quick_casting_thrice_is_not_too_many() {
    let rs = full_ruleset();
    let e = magus(vec![mastered_spell(
        "spell.pilum_of_fire",
        vec![
            "spell_mastery_ability.quick_casting",
            "spell_mastery_ability.quick_casting",
            "spell_mastery_ability.quick_casting",
        ],
    )]);
    let codes = codes(&e, &rs);
    assert!(
        !codes.contains(&"too_many_of_mastery_ability".to_string()),
        "Quick Casting has no book-stated ceiling, three picks must not error: {codes:?}"
    );
}

// --- S2: Ceremonial/Fast/Quick Casting forbidden for Ritual spells ---------

/// `spell.aegis_of_the_hearth` is a real shipped Ritual spell
/// (`rules/core/spells.json`, `"ritual": true`). Each of the three
/// Ritual-forbidden abilities must error when chosen for it.
#[test]
fn ceremonial_fast_and_quick_casting_error_on_a_ritual_spell() {
    let rs = full_ruleset();
    for ability in [
        "spell_mastery_ability.ceremonial_casting",
        "spell_mastery_ability.fast_casting",
        "spell_mastery_ability.quick_casting",
    ] {
        let e = magus(vec![mastered_spell(
            "spell.aegis_of_the_hearth",
            vec![ability],
        )]);
        let codes = codes(&e, &rs);
        assert!(
            codes.contains(&"mastery_ability_forbidden_for_ritual".to_string()),
            "{ability} on a Ritual spell must be forbidden: {codes:?}"
        );
    }
}

/// The same three abilities on a real shipped non-Ritual spell
/// (`spell.pilum_of_fire`) must NOT trip the Ritual-forbidden error.
#[test]
fn ceremonial_fast_and_quick_casting_are_clean_on_a_non_ritual_spell() {
    let rs = full_ruleset();
    for ability in [
        "spell_mastery_ability.ceremonial_casting",
        "spell_mastery_ability.fast_casting",
        "spell_mastery_ability.quick_casting",
    ] {
        let e = magus(vec![mastered_spell("spell.pilum_of_fire", vec![ability])]);
        let codes = codes(&e, &rs);
        assert!(
            !codes.contains(&"mastery_ability_forbidden_for_ritual".to_string()),
            "{ability} on a non-Ritual spell must not be forbidden: {codes:?}"
        );
    }
}

/// Multiple Casting is explicitly the opposite case (ArMDE:9560: "This
/// special ability may be taken for Ritual spells") — it must never trip the
/// Ritual-forbidden error, pinning that the Phase 2 fix's forbidden set is
/// exactly {Ceremonial, Fast, Quick} and not every mastery ability.
#[test]
fn multiple_casting_is_clean_on_a_ritual_spell() {
    let rs = full_ruleset();
    let e = magus(vec![mastered_spell(
        "spell.aegis_of_the_hearth",
        vec!["spell_mastery_ability.multiple_casting"],
    )]);
    let codes = codes(&e, &rs);
    assert!(
        !codes.contains(&"mastery_ability_forbidden_for_ritual".to_string()),
        "Multiple Casting is explicitly allowed on Ritual spells: {codes:?}"
    );
}
