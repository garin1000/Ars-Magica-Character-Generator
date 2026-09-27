//! CV7 — `LinkTarget::resolved` (design § 6.3/§ 6.4,
//! `docs/vf-audit/design-cv-catalogued-values.md`): the picker's combo box
//! must show a link target's CURRENT value ("linked to «Craft Guild
//! Training»: Smiths' Guild of Verdi"), not a raw `(item, param)` pair — so
//! the engine bundles the already-computed resolution alongside the target
//! rather than making the UI look it up a second time.
//!
//! Red-checkpoint protocol, phase 1: `ability_parameter_options`'s builder
//! stubs `resolved` to `None` unconditionally (see
//! `effective/parameter_options.rs`) — every assertion below that expects a
//! populated `Some(..)` is expected to fail on its own assertion. Phase 2
//! carries the already-computed `AuthorizedAbility::instance` value through.

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{Entity, EntityKind, Id, RulesetRef, Selection};
use arm_rules::{LinkTarget, ability_parameter_options};

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

fn companion() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.xp_pool = 0;
    e
}

fn organization_lore_link_target(entity: &Entity, ruleset: &Ruleset) -> LinkTarget {
    let options = ability_parameter_options(entity, ruleset);
    let entry = options
        .iter()
        .find(|o| o.ability == Id::new("ability.organization_lore"))
        .expect("ability.organization_lore must have an options entry (it is catalogued)");
    entry
        .linked
        .iter()
        .find(|lt| lt.item == Id::new("virtue.craft_guild_training"))
        .expect("expected a link target naming Craft Guild Training")
        .clone()
}

/// The motivating scenario: Craft Guild Training's `guild` parameter is set,
/// so the link target the picker offers must carry that CURRENT value —
/// never a raw `(item, param)` pair for the UI to resolve itself.
#[test]
fn a_set_guild_resolves_to_its_current_text() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new("Smiths' Guild of Verdi"))]),
    )];

    let target = organization_lore_link_target(&entity, &ruleset);
    assert_eq!(
        target.resolved,
        Some("Smiths' Guild of Verdi".to_string()),
        "a set guild must resolve to its own current text, got: {:?}",
        target.resolved
    );
}

/// Declared but never filled in: the source exists (unambiguously) but has no
/// value yet, which is a DIFFERENT outcome from "no source at all" (excluded
/// entirely, `cv6_ability_parameter_options.rs`) — the target is still
/// offered, just with nothing resolved yet.
#[test]
fn a_declared_but_unset_guild_resolves_to_none() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::new(Id::new("virtue.craft_guild_training"))];

    let target = organization_lore_link_target(&entity, &ruleset);
    assert_eq!(
        target.resolved, None,
        "a declared-but-unset guild must resolve to None, got: {:?}",
        target.resolved
    );
}
