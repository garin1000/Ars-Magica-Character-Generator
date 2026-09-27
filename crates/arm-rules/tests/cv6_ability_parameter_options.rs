//! CV6 — `AbilityParameterOptions`, the engine-built parameter-picker option
//! list (design § 6.3, § 11 item 2, `docs/vf-audit/design-cv-catalogued-values.md`).
//!
//! Red-checkpoint protocol, phase 1: `ability_parameter_options` is a no-op
//! stub that always returns an empty `Vec` — every assertion below that
//! expects a populated entry is expected to fail on its own assertion, never
//! to panic. Phase 2 wires the real catalogue/link/hint computation in.
//!
//! No UI yet (CV7): these tests exercise the engine function directly.

use std::collections::BTreeMap;

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
};
use arm_rules::{AbilityParameterOptions, LinkTarget, ability_parameter_options};

/// The whole shipped ruleset — Craft Guild Training's Bound wiring and the
/// three shipped catalogues are the fixtures every test below exercises.
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

fn options_for<'a>(
    options: &'a [AbilityParameterOptions],
    ability: &str,
) -> Option<&'a AbilityParameterOptions> {
    options.iter().find(|o| o.ability == Id::new(ability))
}

/// § 6.3: `catalogued` lists this ability's catalogue values, in catalogue
/// order — a ruleset-wide property, independent of whether the character
/// bought anything yet. `virtue.educated` is the motivating scenario (its own
/// pool cites Latin), but the catalogue listing itself does not depend on it.
#[test]
fn dead_language_lists_the_language_catalogue_ids() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::new(Id::new("virtue.educated"))];
    entity.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.dead_language"),
        score: 1,
        specialty: None,
        parameter: Some(AbilityParameterValue::text("Latin")),
    }];

    let options = ability_parameter_options(&entity, &ruleset);
    let entry = options_for(&options, "ability.dead_language")
        .expect("ability.dead_language must have an options entry (it is catalogued)");
    assert!(
        entry.catalogued.contains(&Id::new("language.latin"))
            && entry.catalogued.contains(&Id::new("language.gothic")),
        "expected the language catalogue's ids, got: {:?}",
        entry.catalogued
    );
}

/// § 6.3: a character holding Craft Guild Training with its `guild` parameter
/// set offers that Virtue as a link target for Organization Lore.
#[test]
fn organization_lore_lists_a_link_target_for_craft_guild_training() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new("Smiths' Guild of Verdi"))]),
    )];

    let options = ability_parameter_options(&entity, &ruleset);
    let entry = options_for(&options, "ability.organization_lore")
        .expect("ability.organization_lore must have an options entry (it is catalogued)");
    assert!(
        entry.linked.contains(&LinkTarget {
            item: Id::new("virtue.craft_guild_training"),
            param: "guild".into(),
            // CV7 adds `resolved` — this fixture's own guild value (see
            // cv7_link_target_resolved.rs for the dedicated resolved-value
            // coverage).
            resolved: Some("Smiths' Guild of Verdi".into()),
        }),
        "expected a link target naming Craft Guild Training's own 'guild' parameter, got: {:?}",
        entry.linked
    );
}

/// The negative half: with no such Virtue, there is no link target.
#[test]
fn organization_lore_has_no_link_target_without_the_virtue() {
    let ruleset = full_ruleset();
    let entity = companion();

    let options = ability_parameter_options(&entity, &ruleset);
    let entry = options_for(&options, "ability.organization_lore")
        .expect("ability.organization_lore must have an options entry (it is catalogued)");
    assert!(
        entry.linked.is_empty(),
        "expected no link target without Craft Guild Training, got: {:?}",
        entry.linked
    );
}

/// Design § 4.1: two effective occurrences of Craft Guild Training make its
/// `guild` source ambiguous — never offered as a link target, exactly like
/// matching treats it as satisfying nothing.
#[test]
fn an_ambiguous_duplicate_target_is_excluded_from_linked() {
    let ruleset = full_ruleset();
    let mut entity = companion();
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

    let options = ability_parameter_options(&entity, &ruleset);
    let entry = options_for(&options, "ability.organization_lore")
        .expect("ability.organization_lore must have an options entry (it is catalogued)");
    assert!(
        entry.linked.is_empty(),
        "an ambiguous Bound source must never be offered as a link target, got: {:?}",
        entry.linked
    );
}

/// Design § 11 item 2: a bought `Text` value that leaves Educated's own Latin
/// instance unmet (typed by hand, never resolved to the catalogue id) costs
/// the player something — the hint must fire.
#[test]
fn hint_set_when_a_text_value_leaves_an_educated_instance_unmet() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::new(Id::new("virtue.educated"))];
    entity.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.dead_language"),
        score: 1,
        specialty: None,
        // Typed by hand, not selected from the catalogue — never satisfies
        // Educated's `Literal { "language.latin" }` instance (design § 4 rule
        // 1: a Literal is satisfied ONLY by `Catalogued`).
        parameter: Some(AbilityParameterValue::text("Latin")),
    }];

    let options = ability_parameter_options(&entity, &ruleset);
    let entry = options_for(&options, "ability.dead_language").expect("entry must exist");
    assert!(
        entry.hint,
        "a Text value leaving Educated's Latin instance unmet must set the hint"
    );
}

/// Same shape, the Bound/Guild case: a typed Organization Lore value that
/// does not spell Craft Guild Training's own guild leaves that instance
/// unmet.
#[test]
fn hint_set_when_a_text_value_leaves_a_guild_instance_unmet() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new("Smiths' Guild of Verdi"))]),
    )];
    entity.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.organization_lore"),
        score: 1,
        specialty: None,
        parameter: Some(AbilityParameterValue::text("A Completely Different Guild")),
    }];

    let options = ability_parameter_options(&entity, &ruleset);
    let entry = options_for(&options, "ability.organization_lore").expect("entry must exist");
    assert!(
        entry.hint,
        "a Text value leaving Craft Guild Training's guild instance unmet must set the hint"
    );
}

/// The hint is conditional, not static (§ 11 item 2): an Ability no
/// Literal/Bound instance from any of the character's own items targets
/// carries no hint at all — Craft, with no Forge Companion held, is the
/// design note's own example.
#[test]
fn hint_not_set_for_an_ability_with_no_targeting_rule() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    // Holds Craft Guild Training (targets Organization Lore, not Craft) so the
    // character is not simply empty — but nothing here targets `ability.craft`.
    entity.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new("Smiths' Guild of Verdi"))]),
    )];
    entity.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.craft"),
        score: 1,
        specialty: Some("Blacksmith".into()),
        parameter: Some(AbilityParameterValue::text("Blacksmithing")),
    }];

    let options = ability_parameter_options(&entity, &ruleset);
    assert!(
        options
            .iter()
            .all(|o| o.ability != Id::new("ability.craft") || !o.hint),
        "Craft has no Literal/Bound instance from any held item, so it must carry no hint"
    );
}

/// Non-vacuous companion to the Craft test above (coordinator review):
/// `ability.dead_language` IS catalogued, so its entry always exists — unlike
/// Craft, whose absence let the "no hint" assertion pass without the engine
/// doing any work. Here the entry must exist (catalogued) AND the hint must
/// still be false, because nothing the character holds carries a Dead
/// Language Literal or Bound instance — "Klingon" costs the player nothing by
/// staying free text.
#[test]
fn hint_not_set_for_a_catalogued_ability_with_no_targeting_rule() {
    let ruleset = full_ruleset();
    let mut entity = companion();
    entity.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.dead_language"),
        score: 1,
        specialty: None,
        parameter: Some(AbilityParameterValue::text("Klingon")),
    }];

    let options = ability_parameter_options(&entity, &ruleset);
    let entry = options_for(&options, "ability.dead_language")
        .expect("ability.dead_language must have an options entry (it is catalogued)");
    assert!(
        !entry.hint,
        "no item targets Dead Language for this character, so the hint must not fire"
    );
}
