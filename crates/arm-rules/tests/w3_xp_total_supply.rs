//! W3 (try-out finding 10, `tmp/tryout-findings-2026-10-03.md`): the XP bar must
//! say how much experience the character has IN TOTAL, beside the per-pool chips.
//!
//! `XpAllocation::max_flow` cannot be that figure: it is the demand the pools can
//! FUND, so for every legal character it equals `total_demand` and a "spent N of M"
//! read-out would always say "N of N". The total is the supply instead — the general
//! pool plus every restricted pool the flow solve is given (ability-XP grants,
//! life-stage blocks, and the Spell-Mastery pool, which `restricted` never
//! surfaces). Pinned here against the REAL shipped `rules/core/virtues_flaws.json`:
//! Educated grants a 50-point restricted pool (ArMDE:3711-3713) and Mastered
//! Spells a 50-point Spell-Mastery pool (ArMDE:4471-4474).

use arm_rules::checked_xp_allocation;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection};

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
        spell_mastery_abilities: None,
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

/// A pool-funded magus with a typed 240 and both pool-granting Virtues.
fn magus_with_both_grants(pool: u32) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.xp_pool = pool;
    e.selections = vec![
        Selection::new(Id::new("virtue.educated")),
        Selection::new(Id::new("virtue.mastered_spells")),
    ];
    e
}

/// The supply is every pool's size summed — 240 general + 50 Educated + 50 Mastery —
/// whatever the character has spent so far, including nothing at all.
#[test]
fn total_supply_sums_the_general_restricted_and_mastery_pools() {
    let ruleset = full_ruleset();
    let magus = magus_with_both_grants(240);

    let allocation = checked_xp_allocation(&magus, &ruleset).expect("solve stays in bounds");

    assert_eq!(allocation.total_supply, 340);
}

/// The reason the figure exists: a legal, under-spent character's `max_flow` is its
/// demand, so only `total_supply` can tell the player how much is left overall.
#[test]
fn total_supply_is_not_the_funded_demand_for_a_legal_character() {
    let ruleset = full_ruleset();
    let mut magus = magus_with_both_grants(240);
    // Awareness 2 costs 15 (the advancement table), funded from the general pool.
    magus.ability_scores = vec![AbilityScore::new(Id::new("ability.awareness"), 2)];

    let allocation = checked_xp_allocation(&magus, &ruleset).expect("solve stays in bounds");

    assert_eq!(allocation.total_demand, 15);
    assert_eq!(allocation.max_flow, 15);
    assert_eq!(allocation.total_supply, 340);
}

/// A crafted save typing the pool at `u32::MAX` must saturate, not overflow (a
/// panic in release, which keeps overflow checks on).
#[test]
fn total_supply_saturates_instead_of_overflowing() {
    let ruleset = full_ruleset();
    let magus = magus_with_both_grants(u32::MAX);

    let allocation = checked_xp_allocation(&magus, &ruleset).expect("solve stays in bounds");

    assert_eq!(allocation.total_supply, u32::MAX);
}
