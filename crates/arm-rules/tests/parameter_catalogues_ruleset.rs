//! CV2 — wiring CV1's parameter catalogues into the real `Ruleset` load path,
//! and `Ability.catalogued` (design note § 2, § 10's CV2 row).
//!
//! Red-checkpoint protocol, phase 1: `Ruleset`/`RulesetSources` now carry a
//! `parameter_catalogues` field, but `ruleset/parse.rs::assemble_ruleset`
//! stubs it to an always-empty map (ignoring whatever JSON is passed) — so
//! every test below is expected to fail on its own assertion right now, not to
//! panic. Phase 2 wires the real parse + pre-integrity checks in.

use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::Id;

/// The 2-value organization catalogue from CV1, as a `RulesetSources`-shaped
/// fixture (not `load_parameter_catalogues` directly — this test is about the
/// real `Ruleset::from_sources` load path, CV1's own standalone loader is
/// already covered by `tests/parameter_catalogues.rs`).
const ORGANIZATION_CATALOGUE_JSON: &str = r#"{
  "catalogues": [
    {
      "id": "catalogue.organization",
      "values": [
        { "id": "organization.house_bjornaer", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3563, 3565] } },
        { "id": "organization.order_of_hermes", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4063, 4065] } }
      ]
    }
  ]
}"#;

fn ruleset_with_catalogues(catalogues_json: &str) -> Result<Ruleset, arm_rules::RulesetError> {
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: "[]",
        type_profiles: "[]",
        parameter_catalogues: Some(catalogues_json),
        ..RulesetSources::default()
    })
}

#[test]
fn ruleset_from_sources_exposes_catalogues_by_id() {
    let ruleset = ruleset_with_catalogues(ORGANIZATION_CATALOGUE_JSON)
        .expect("a well-formed catalogue file loads through the real Ruleset pipeline");

    let organization = ruleset
        .catalogue(&Id::new("catalogue.organization"))
        .expect("catalogue.organization must be exposed through Ruleset::catalogue");

    // Known ids present — never a total count (CLAUDE.md: catalogue size is data).
    assert!(
        organization
            .value(&Id::new("organization.house_bjornaer"))
            .is_some(),
        "organization.house_bjornaer must be exposed by id"
    );
    assert!(
        ruleset
            .catalogues()
            .any(|c| c.id == Id::new("catalogue.organization")),
        "Ruleset::catalogues must iterate the loaded catalogue"
    );
}

#[test]
fn ruleset_from_sources_fails_the_real_load_on_a_broken_catalogue() {
    let duplicate_id_json = r#"{
      "catalogues": [
        { "id": "catalogue.organization", "values": [
          { "id": "organization.house_bjornaer", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3563, 3565] } }
        ] },
        { "id": "catalogue.organization", "values": [
          { "id": "organization.order_of_hermes", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4063, 4065] } }
        ] }
      ]
    }"#;

    let err = ruleset_with_catalogues(duplicate_id_json)
        .expect_err("a duplicate catalogue id must fail the REAL Ruleset::from_sources load");
    let message = err.to_string();
    assert!(
        message.contains("catalogue.organization"),
        "error must name the offending catalogue id, got: {message}"
    );
}

/// Design note § 1.3: `ability.dead_language`/`ability.living_language` each
/// name their own language catalogue (L1a); `ability.profession` uses `profession`;
/// `ability.organization_lore` uses `organization`. `ability.craft`,
/// `ability.area_lore`, `ability.mystery_cult_lore` stay uncatalogued.
#[test]
fn listed_abilities_read_as_catalogued_in_the_shipped_data() {
    // `from_json_with_abilities` cannot also pass `parameter_catalogues`, and
    // the shipped abilities file now declares catalogued abilities — so this
    // needs the full `from_sources`, with the real shipped catalogues too, or
    // `validate_catalogued_abilities` rejects the load.
    let ruleset = Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: "[]",
        type_profiles: "[]",
        abilities: Some(include_str!("../../../rules/core/abilities.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("the shipped abilities file loads");

    for catalogued_id in [
        "ability.dead_language",
        "ability.living_language",
        "ability.profession",
        "ability.organization_lore",
    ] {
        let ability = ruleset
            .ability(&Id::new(catalogued_id))
            .unwrap_or_else(|| panic!("{catalogued_id} must be present in the shipped catalogue"));
        assert!(
            ability.catalogued,
            "{catalogued_id} must read as catalogued: true"
        );
    }

    for uncatalogued_id in [
        "ability.craft",
        "ability.area_lore",
        "ability.mystery_cult_lore",
    ] {
        let ability = ruleset
            .ability(&Id::new(uncatalogued_id))
            .unwrap_or_else(|| {
                panic!("{uncatalogued_id} must be present in the shipped catalogue")
            });
        assert!(
            !ability.catalogued,
            "{uncatalogued_id} must stay uncatalogued (design note § 1.3)"
        );
    }
}

#[test]
fn catalogued_ability_without_parameter_fails_to_load() {
    let abilities_json = r#"{ "abilities": [
      { "id": "ability.test", "category": "general", "catalogued": true }
    ] }"#;

    let err = Ruleset::from_json_with_abilities("test", "1", "[]", "[]", abilities_json)
        .expect_err("catalogued: true with no parameter must fail to load");
    let message = err.to_string();
    assert!(
        message.contains("ability.test"),
        "error must name the offending ability id, got: {message}"
    );
}

#[test]
fn catalogued_ability_naming_no_catalogue_fails_to_load() {
    let abilities_json = r#"{ "abilities": [
      { "id": "ability.test", "category": "general", "parameter": "nonexistent_key", "catalogued": true }
    ] }"#;

    let err = Ruleset::from_json_with_abilities("test", "1", "[]", "[]", abilities_json)
        .expect_err("a catalogued ability whose key names no catalogue must fail to load");
    let message = err.to_string();
    assert!(
        message.contains("ability.test") || message.contains("nonexistent_key"),
        "error must name the offending ability or its parameter key, got: {message}"
    );
}
