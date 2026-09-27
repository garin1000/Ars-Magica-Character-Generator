//! CV5 — Bound/Link matching semantics (design § 4/4.1, `docs/vf-audit/design-cv-catalogued-values.md`).
//!
//! Red-checkpoint protocol, phase 1: `AbilityParameterValue::Linked` never
//! satisfies a `Bound` pool today (`effective/xp.rs::instance_satisfied`'s
//! catch-all always returns `false` for it), rule 2's content match is exact
//! rather than case-insensitive/trimmed, and the ambiguity guard
//! (`crate::effective::resolve_link`) is a no-op stub — every positive
//! assertion below is expected to fail on its own assertion, not to panic.
//! Phase 2 wires the real § 4 matching and § 4.1 ambiguity guard in.

use std::collections::BTreeMap;

use arm_rules::checked_xp_allocation;
use arm_rules::migration::load_entity_migrating;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
};
use arm_rules::validation::{ValidationIssue, validate};

/// The whole shipped ruleset — Craft Guild Training's real Bound wiring (CV3)
/// is the fixture every test below exercises.
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

/// A companion holding Craft Guild Training once, with one Organization Lore
/// row carrying `parameter`.
fn companion_with_guild_and_ability(guild: &str, parameter: AbilityParameterValue) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.xp_pool = 0;
    e.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new(guild))]),
    )];
    e.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.organization_lore"),
        score: 1,
        specialty: None,
        parameter: Some(parameter),
    }];
    e
}

/// Design § 4 rule 1: a `Linked` value naming the SAME declaring item and
/// parameter as the pool's own `Bound` source funds it structurally — "no
/// string comparison at all; true by construction" — so it funds regardless
/// of what the guild's current text happens to be.
#[test]
fn linked_value_funds_its_own_virtues_bound_pool() {
    let ruleset = full_ruleset();
    let entity = companion_with_guild_and_ability(
        "Smiths' Guild of Verdi",
        AbilityParameterValue::Linked {
            item: Id::new("virtue.craft_guild_training"),
            param: "guild".into(),
        },
    );
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "a Linked value naming Craft Guild Training's own 'guild' parameter must fund its \
         pool structurally (design § 4 rule 1)"
    );
}

/// The negative half of rule 1: a `Linked` value naming a DIFFERENT declaring
/// item never satisfies this pool.
#[test]
fn linked_value_to_a_different_item_does_not_fund_the_pool() {
    let ruleset = full_ruleset();
    let entity = companion_with_guild_and_ability(
        "Smiths' Guild of Verdi",
        AbilityParameterValue::Linked {
            item: Id::new("virtue.forge_companion"),
            param: "guild".into(),
        },
    );
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert!(
        allocation.max_flow < allocation.total_demand,
        "a Linked value naming a DIFFERENT declaring item must not fund Craft Guild \
         Training's pool"
    );
}

/// Design § 4 rule 2: a typed value spelling the Bound source's current text
/// in a different case, with padding whitespace, still funds — "a player who
/// typed the guild's name by hand instead of linking it is not punished."
#[test]
fn case_insensitive_trimmed_text_funds_the_bound_pool() {
    let ruleset = full_ruleset();
    let entity = companion_with_guild_and_ability(
        "Smiths' Guild of Verdi",
        AbilityParameterValue::text("  SMITHS' GUILD OF VERDI  "),
    );
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "rule 2's content match must be case-insensitive and trimmed"
    );
}

/// Design § 4.1: two effective occurrences of the SAME declaring item — here,
/// bought twice, the same reachable state a bought-plus-granted duplicate
/// produces (design § 0/§ 4.2) — make `(item_ref, param)` ambiguous. A
/// `Linked` value pointed at it must satisfy NOTHING, never guessing which of
/// the two selections it means, and `validate` must raise
/// `issue-ambiguous_bound_parameter`.
#[test]
fn bought_twice_with_a_link_resolves_ambiguous_and_funds_nothing() {
    let ruleset = full_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.xp_pool = 0;
    entity.selections = vec![
        Selection::with_params(
            Id::new("virtue.craft_guild_training"),
            BTreeMap::from([("guild".into(), Id::new("Smiths' Guild of Verdi"))]),
        ),
        Selection::with_params(
            Id::new("virtue.craft_guild_training"),
            BTreeMap::from([("guild".into(), Id::new("Different Guild"))]),
        ),
    ];
    entity.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.organization_lore"),
        score: 1,
        specialty: None,
        parameter: Some(AbilityParameterValue::Linked {
            item: Id::new("virtue.craft_guild_training"),
            param: "guild".into(),
        }),
    }];

    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert!(
        allocation.max_flow < allocation.total_demand,
        "an ambiguous Bound source must fund nothing, never pick either occurrence"
    );

    let result = validate(&entity, &ruleset);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == ValidationIssue::CODE_AMBIGUOUS_BOUND_PARAMETER),
        "expected an `ambiguous_bound_parameter` issue, got: {:?}",
        result.issues.iter().map(|i| &i.code).collect::<Vec<_>>()
    );
}

/// Design § 3.3, extended to `Linked`: a value naming keys from more than one
/// `AbilityParameterValue` variant at once (here, `Linked`'s `item`/`param`
/// alongside `Text`'s `text`) is not a shape any writer of this format
/// produces — only a hand-edited or adversarial save could. `deny_unknown_fields`
/// must reject it and fail the whole entity load with a clear error, never
/// silently pick one variant and drop the rest.
#[test]
fn a_link_shape_mixed_with_text_fields_is_rejected_not_misread() {
    let ruleset = full_ruleset();
    let names: BTreeMap<Id, Vec<String>> = BTreeMap::new();
    let json = r#"{
          "schema_version": 17,
          "ruleset": { "id": "arm5-core", "version": "2024.1" },
          "entity_kind": "character",
          "type_id": "companion",
          "ability_scores": [
            { "ability": "ability.organization_lore", "score": 1,
               "parameter": { "item": "virtue.craft_guild_training", "param": "guild", "text": "evil" } }
          ]
        }"#
    .to_string();
    let err = load_entity_migrating(&json, 1220, &ruleset, &names)
        .expect_err("a value naming keys from two AbilityParameterValue variants must be rejected");
    assert!(
        err.to_string().contains("did not match any variant"),
        "must be rejected as an unrecognised shape, not some other error: {err}"
    );
}
