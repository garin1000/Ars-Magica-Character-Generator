//! D48/Phase 2 C4 — restricted-XP pools that fund a specific Ability *instance*,
//! not just its id (`docs/vf-audit/decisions.md` D48). Two things pinned here
//! against the REAL shipped `rules/core/virtues_flaws.json`:
//!
//! - `virtue.marshal`'s 50-point pool must fund **Profession: Marshal** only,
//!   never an unrelated Profession instance (Profession: Sailor) — the actual
//!   over-permission Q-47/D48 fixes.
//! - `virtue.master_bard`'s pool keeps its `amount: 240` and its full six-Ability
//!   eligibility (Area Lore, Art of Memory, Faerie Lore, Magic Lore,
//!   Organization Lore, Profession) after the `instances` rewrite — the
//!   regression the design note calls out by name.

use arm_rules::checked_xp_allocation;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
};

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

/// `parameter` is wrapped as `Catalogued`, not `Text`: every instance this
/// helper is called with below names a real `catalogue.profession` value
/// (`profession.marshal`, `profession.storyteller`) — design § 4 rule 1's
/// `Literal` instance is satisfied ONLY by a `Catalogued` id match, never by
/// `Text` holding the identical letters.
fn companion_with(selection: &str, ability: &str, parameter: &str) -> Entity {
    companion_with_parameter(
        selection,
        ability,
        AbilityParameterValue::Catalogued {
            id: Id::new(parameter),
        },
    )
}

/// The `sailor` sibling above: a Profession instance NOT in the catalogue at
/// all (D48's negative case), so it stays free `Text` — exactly what a
/// player who typed an uncatalogued profession would have stored.
fn companion_with_text(selection: &str, ability: &str, parameter: &str) -> Entity {
    companion_with_parameter(selection, ability, AbilityParameterValue::text(parameter))
}

fn companion_with_parameter(
    selection: &str,
    ability: &str,
    parameter: AbilityParameterValue,
) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    // No general pool at all: the only way the bought score can be funded is
    // the restricted pool the selection grants, so whether the pool covers
    // this exact instance is directly observable from `max_flow`.
    e.xp_pool = 0;
    e.selections = vec![Selection::new(Id::new(selection))];
    e.ability_scores = vec![AbilityScore {
        ability: Id::new(ability),
        score: 1,
        specialty: None,
        parameter: Some(parameter),
        banked_xp: 0,
    }];
    e
}

/// The actual D48 defect: `virtue.marshal` (ArMDE:4449-4456) ties its 50 points
/// to "Profession: Marshal", never "any Profession". A companion whose ONLY
/// funding source is Marshal's pool and who bought Profession: Sailor (not
/// Marshal) must be UNDER-funded — `max_flow < total_demand` — because the
/// pool does not cover that instance.
#[test]
fn marshal_pool_does_not_fund_an_unrelated_profession_instance() {
    let ruleset = full_ruleset();
    let sailor = companion_with_text("virtue.marshal", "ability.profession", "sailor");

    let allocation = checked_xp_allocation(&sailor, &ruleset).expect("solve stays in bounds");

    assert!(
        allocation.max_flow < allocation.total_demand,
        "Profession: Sailor must NOT be funded by Marshal's pool (D48) — got \
         max_flow {} >= total_demand {}",
        allocation.max_flow,
        allocation.total_demand
    );
}

/// The positive half of the same fix: Profession: **Marshal** specifically
/// (the instance the passage actually names) IS funded, fully, by the same
/// pool and no general pool at all.
#[test]
fn marshal_pool_funds_its_own_named_profession_instance() {
    let ruleset = full_ruleset();
    // CV3 (design-cv-catalogued-values.md § 1.1): the shipped literal becomes
    // the catalogue id `profession.marshal`, not the bare rulebook word.
    let marshal = companion_with("virtue.marshal", "ability.profession", "profession.marshal");

    let allocation = checked_xp_allocation(&marshal, &ruleset).expect("solve stays in bounds");

    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Profession: Marshal must be fully funded by Marshal's own pool"
    );
}

/// Master Bard regression (design note § 5), exercised end-to-end through the
/// public flow-solve API only (`Ruleset::point_items` is `pub(crate)`, not
/// reachable from an integration test): the rewrite that moves Profession into
/// `instances` must not touch `amount` or drop Faerie Lore / Magic Lore from
/// the unscoped `abilities` eligibility. Faerie Lore 9 (225 XP) + Magic Lore 2
/// (15 XP) sums to exactly 240 — the pool's own stated total — with no
/// general pool at all, so full funding pins `amount == 240` and that both
/// Abilities are still eligible in one assertion.
#[test]
fn master_bard_pool_is_exactly_240_and_still_funds_faerie_and_magic_lore() {
    let ruleset = full_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.xp_pool = 0;
    entity.selections = vec![Selection::new(Id::new("virtue.master_bard"))];
    entity.ability_scores = vec![
        AbilityScore {
            ability: Id::new("ability.faerie_lore"),
            score: 9,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        },
        AbilityScore {
            ability: Id::new("ability.magic_lore"),
            score: 2,
            specialty: None,
            parameter: None,
            banked_xp: 0,
        },
    ];

    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(allocation.total_demand, 240);
    assert_eq!(
        allocation.max_flow, 240,
        "Faerie Lore 9 + Magic Lore 2 (225 + 15 = 240 XP) must be exactly, fully funded \
         by Master Bard's own pool with no general pool at all"
    );

    // One more XP of demand than the pool holds must overflow, pinning that
    // the pool grants exactly 240 — not 241, not unlimited.
    entity.ability_scores.push(AbilityScore {
        ability: Id::new("ability.organization_lore"),
        score: 1,
        specialty: None,
        parameter: Some(AbilityParameterValue::text("guild")),
        banked_xp: 0,
    });
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(allocation.total_demand, 245);
    assert_eq!(
        allocation.max_flow, 240,
        "Master Bard's pool must cap at exactly 240, no more"
    );
}

/// Master Bard's pool, exercised end-to-end: Profession: Storyteller is
/// funded, Profession: Sailor is not, Faerie Lore (unscoped) still is.
#[test]
fn master_bard_pool_funds_storyteller_and_faerie_lore_but_not_an_unrelated_profession() {
    let ruleset = full_ruleset();

    // CV3: catalogue id `profession.storyteller`, not the bare rulebook word.
    let storyteller = companion_with(
        "virtue.master_bard",
        "ability.profession",
        "profession.storyteller",
    );
    let allocation = checked_xp_allocation(&storyteller, &ruleset).expect("solve stays in bounds");
    assert_eq!(allocation.max_flow, allocation.total_demand);

    let sailor = companion_with_text("virtue.master_bard", "ability.profession", "sailor");
    let allocation = checked_xp_allocation(&sailor, &ruleset).expect("solve stays in bounds");
    assert!(allocation.max_flow < allocation.total_demand);

    let mut faerie_lore = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    faerie_lore.xp_pool = 0;
    faerie_lore.selections = vec![Selection::new(Id::new("virtue.master_bard"))];
    faerie_lore.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.faerie_lore"),
        score: 1,
        specialty: None,
        parameter: None,
        banked_xp: 0,
    }];
    let allocation = checked_xp_allocation(&faerie_lore, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Faerie Lore stays in `abilities` (unscoped), so it must still be fully funded"
    );
}
