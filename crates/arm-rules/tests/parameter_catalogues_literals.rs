//! CV3 — literal `AbilityRef.instance` values become catalogue ids (design
//! note § 1.1/§ 7, `docs/vf-audit/design-cv-catalogued-values.md`).
//!
//! Red-checkpoint protocol, phase 1: none of the checks below exist yet, and
//! none of the shipped `rules/core/virtues_flaws.json` literals have been
//! rewritten as catalogue ids — every test here is expected to fail on its
//! own assertion right now, not to panic. Phase 2 lands the data + integrity
//! check together.

use std::collections::BTreeMap;

use arm_rules::checked_xp_allocation;
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{
    AbilityParameterValue, AbilityScore, Entity, EntityKind, Id, RulesetRef, Selection,
};

// --- Load-integrity: a Literal instance must resolve inside the catalogue
// named by its ability's own `parameter` key (design note § 7) ---------------

/// Every fixture below needs a `personality`-category item once `point_items`
/// is non-empty (`Ruleset::validate_engine_required_categories`) — mirrors
/// `ruleset.rs::ruleset_with_param`'s own `flaw.optimistic` padding item.
const DUMMY_PERSONALITY_ITEM: &str = r#"{ "id": "flaw.dummy_personality", "kind": "flaw",
  "classification": "narrative", "magnitude": "minor", "categories": ["personality"] }"#;

const TEST_ABILITIES_JSON: &str = r#"{ "abilities": [
  { "id": "ability.test_lang", "category": "academic", "parameter": "language", "catalogued": true },
  { "id": "ability.test_prof", "category": "general", "parameter": "profession", "catalogued": true }
] }"#;

const TEST_CATALOGUES_JSON: &str = r#"{
  "catalogues": [
    { "id": "catalogue.language", "values": [
      { "id": "language.latin", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3711, 3713] } }
    ] },
    { "id": "catalogue.profession", "values": [
      { "id": "profession.falconer", "source": { "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3847, 3852] } }
    ] }
  ]
}"#;

fn ruleset_with_literal(instance_literal: &str) -> Result<Ruleset, arm_rules::RulesetError> {
    let point_items = format!(
        r#"[
          {{ "id": "virtue.test", "kind": "virtue", "classification": "creation_effect",
             "magnitude": "minor", "categories": ["general"],
             "effects": [{{ "type": "ability_authorization", "abilities": [
               {{ "ability": "ability.test_lang", "instance": {{ "literal": "{instance_literal}" }} }}
             ] }}] }},
          {DUMMY_PERSONALITY_ITEM}
        ]"#
    );
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: &point_items,
        type_profiles: "[]",
        abilities: Some(TEST_ABILITIES_JSON),
        parameter_catalogues: Some(TEST_CATALOGUES_JSON),
        ..RulesetSources::default()
    })
}

#[test]
fn literal_instance_naming_an_unknown_catalogue_value_fails_to_load() {
    let err = ruleset_with_literal("language.bogus")
        .expect_err("a literal naming no value in its ability's catalogue must fail the load");
    let message = err.to_string();
    assert!(
        message.contains("language.bogus") && message.contains("ability.test_lang"),
        "error must name both the offending literal id and the ability, got: {message}"
    );
}

#[test]
fn literal_instance_naming_a_value_from_the_wrong_catalogue_fails_to_load() {
    // "profession.falconer" is a real value — just not in `catalogue.language`,
    // the catalogue `ability.test_lang` (parameter `language`) resolves against.
    let err = ruleset_with_literal("profession.falconer").expect_err(
        "a literal naming a value from a DIFFERENT catalogue must fail the load, \
         not resolve across catalogues",
    );
    let message = err.to_string();
    assert!(
        message.contains("profession.falconer") && message.contains("ability.test_lang"),
        "error must name both the offending literal id and the ability, got: {message}"
    );
}

#[test]
fn literal_instance_naming_a_real_catalogue_value_loads_clean() {
    ruleset_with_literal("language.latin")
        .expect("a literal naming a value that IS in its ability's own catalogue must load");
}

// --- Load-integrity: a Bound-instance-declaring item must be max_total <= 1
// (design note § 7) -----------------------------------------------------------

fn ruleset_with_bound_item(max_total: Option<u8>) -> Result<Ruleset, arm_rules::RulesetError> {
    let max_total_field = match max_total {
        Some(n) => format!(r#", "max_total": {n}"#),
        None => String::new(),
    };
    let point_items = format!(
        r#"[
          {{ "id": "virtue.test_bound", "kind": "virtue", "classification": "creation_effect",
             "magnitude": "minor", "categories": ["general"],
             "parameters": [{{ "key": "guild", "type": "ref", "domain": "text" }}],
             "effects": [{{ "type": "restricted_ability_xp", "amount": 50,
               "instances": [{{ "ability": "ability.test_org", "instance": {{ "param": "guild" }} }}] }}]
             {max_total_field} }},
          {DUMMY_PERSONALITY_ITEM}
        ]"#
    );
    let abilities = r#"{ "abilities": [
      { "id": "ability.test_org", "category": "general", "parameter": "organization" }
    ] }"#;
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: &point_items,
        type_profiles: "[]",
        abilities: Some(abilities),
        ..RulesetSources::default()
    })
}

#[test]
fn bound_declaring_item_with_max_total_above_one_fails_to_load() {
    let err = ruleset_with_bound_item(Some(2)).expect_err(
        "a Bound-instance-declaring item with max_total > 1 must fail the load: \
         (item_ref, param) is the only handle a Bound source has, and two copies \
         would make it ambiguous by construction (design note § 7)",
    );
    let message = err.to_string();
    assert!(
        message.contains("virtue.test_bound"),
        "error must name the offending item, got: {message}"
    );
}

#[test]
fn bound_declaring_item_with_default_max_total_loads_clean() {
    ruleset_with_bound_item(None)
        .expect("max_total defaults to 1 (D10), which is legal for a Bound-declaring item");
}

// --- Shipped data: literal instances are catalogue ids, and Craft Guild
// Training's guild funding, against the REAL ruleset ------------------------

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

/// `parameter` is wrapped as `Catalogued`, not `Text`: every call below names a
/// real catalogue id (`language.gothic`, `organization.house_bjornaer`, …), and
/// design § 4 rule 1's `Literal` instance is satisfied ONLY by a `Catalogued`
/// id match, never by `Text` holding the identical letters.
fn companion_with(selection: &str, ability: &str, parameter: &str) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.xp_pool = 0;
    e.selections = vec![Selection::new(Id::new(selection))];
    e.ability_scores = vec![AbilityScore {
        ability: Id::new(ability),
        score: 1,
        specialty: None,
        parameter: Some(AbilityParameterValue::Catalogued {
            id: Id::new(parameter),
        }),
    }];
    e
}

/// Known examples across all three catalogues (never a total count,
/// CLAUDE.md: catalogue size is data) — `virtue.clan_ilfetu`'s pool must fund
/// the catalogue ids `language.gothic`/`organization.house_bjornaer`, not the
/// bare rulebook words `gothic`/`house_bjornaer` the shipped data still uses
/// today.
#[test]
fn clan_ilfetu_pool_funds_the_gothic_language_catalogue_id() {
    let ruleset = full_ruleset();
    let entity = companion_with(
        "virtue.clan_ilfetu",
        "ability.dead_language",
        "language.gothic",
    );
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Clan Ilfetu's pool must fund the Gothic instance by its catalogue id \
         'language.gothic', not the shipped literal 'gothic'"
    );
}

#[test]
fn clan_ilfetu_pool_funds_the_house_bjornaer_organization_catalogue_id() {
    let ruleset = full_ruleset();
    let entity = companion_with(
        "virtue.clan_ilfetu",
        "ability.organization_lore",
        "organization.house_bjornaer",
    );
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Clan Ilfetu's pool must fund the House Bjornaer instance by its catalogue id \
         'organization.house_bjornaer', not the shipped literal 'house_bjornaer'"
    );
}

#[test]
fn hermetic_experience_pool_funds_the_order_of_hermes_catalogue_id() {
    let ruleset = full_ruleset();
    let entity = companion_with(
        "virtue.hermetic_experience",
        "ability.organization_lore",
        "organization.order_of_hermes",
    );
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Hermetic Experience's pool must fund the Order of Hermes instance by its \
         catalogue id 'organization.order_of_hermes', not the shipped literal \
         'order_of_hermes'"
    );
}

/// `virtue.falconer`'s pool cites BOTH a language (Latin) and a profession
/// (Falconer) literal on the same item — covers the `profession` catalogue
/// with a second, independent example from `clan_ilfetu`'s.
#[test]
fn falconer_pool_funds_the_falconer_profession_catalogue_id() {
    let ruleset = full_ruleset();
    let entity = companion_with(
        "virtue.falconer",
        "ability.profession",
        "profession.falconer",
    );
    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "Falconer's pool must fund the Falconer instance by its catalogue id \
         'profession.falconer', not the shipped literal 'falconer'"
    );
}

/// § 1.1a: `virtue.craft_guild_training` gains its own `guild` text parameter
/// and its Organization Lore pool instance becomes `Bound` to it (replacing
/// the defective literal `"guild"`, which named no single universal
/// organization). The pool must fund the Organization Lore matching the
/// character's OWN chosen guild, and must NOT fund an unrelated one.
#[test]
fn craft_guild_training_funds_the_organization_lore_matching_its_own_guild() {
    let ruleset = full_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.xp_pool = 0;
    entity.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".into(), Id::new("Smiths' Guild of Verdi"))]),
    )];
    entity.ability_scores = vec![AbilityScore {
        ability: Id::new("ability.organization_lore"),
        score: 1,
        specialty: None,
        parameter: Some(AbilityParameterValue::text("Smiths' Guild of Verdi")),
    }];

    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert_eq!(
        allocation.max_flow, allocation.total_demand,
        "the pool must fund the Organization Lore matching the character's OWN \
         chosen guild"
    );
}

#[test]
fn craft_guild_training_does_not_fund_a_different_organization_lore() {
    let ruleset = full_ruleset();
    let mut entity = Entity::new(
        EntityKind::Character,
        Id::new("companion"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.xp_pool = 0;
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

    let allocation = checked_xp_allocation(&entity, &ruleset).expect("solve stays in bounds");
    assert!(
        allocation.max_flow < allocation.total_demand,
        "an Organization Lore instance naming a DIFFERENT guild must NOT be funded \
         by this character's own Craft Guild Training pool"
    );
}
