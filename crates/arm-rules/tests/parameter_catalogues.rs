//! CV1 — catalogued parameter values: loading `rules/core/parameter_catalogues.json`
//! -shaped data and its `rules/i18n/{en,de}/parameter_catalogue.json` name files,
//! plus the load-time integrity rules the design note assigns to CV1 (unique
//! ids, both-locale names, no cross-locale case-folded name collision). See
//! `docs/vf-audit/design-cv-catalogued-values.md` §§ 1.3, 2.2, 2.3, 7.
//!
//! **Red-checkpoint protocol, phase 1.** `catalogue.rs`'s `load_parameter_catalogues`
//! and `load_catalogue_names` are signature-only stubs that always succeed with
//! an empty result, so every test below is expected to fail on its own
//! assertion right now (not to panic) — phase 2 makes them green.

use arm_rules::types::{Id, LineRange, SourceRef};
use arm_rules::{Catalogue, CatalogueValue, load_catalogue_names, load_parameter_catalogues};
use std::collections::BTreeMap;

/// A minimal, real (not placeholder) catalogue: the 2-value `organization`
/// catalogue the design note's CV1 row names explicitly (§ 10).
fn organization_catalogue_json() -> &'static str {
    r#"{
      "catalogues": [
        {
          "id": "catalogue.organization",
          "values": [
            { "id": "organization.house_bjornaer", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3563, 3565] } },
            { "id": "organization.order_of_hermes", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4063, 4065] } }
          ]
        }
      ]
    }"#
}

/// Same catalogue, built directly as Rust values rather than parsed from JSON
/// — used by the name-loading tests so they do not depend on
/// `load_parameter_catalogues` also being implemented.
fn organization_catalogues() -> BTreeMap<Id, Catalogue> {
    let catalogue = Catalogue {
        id: Id::new("catalogue.organization"),
        values: vec![
            CatalogueValue {
                id: Id::new("organization.house_bjornaer"),
                source: SourceRef::new(
                    "Ars Magica - Definitive Edition (Core Rules).md",
                    LineRange::new(3563, 3565),
                    "anchor",
                ),
            },
            CatalogueValue {
                id: Id::new("organization.order_of_hermes"),
                source: SourceRef::new(
                    "Ars Magica - Definitive Edition (Core Rules).md",
                    LineRange::new(4063, 4065),
                    "anchor",
                ),
            },
        ],
    };
    BTreeMap::from([(catalogue.id.clone(), catalogue)])
}

const COMPLETE_EN_NAMES: &str = r#"{ "names": [
  { "id": "organization.house_bjornaer", "name": "House Bjornaer" },
  { "id": "organization.order_of_hermes", "name": "Order of Hermes" }
] }"#;

const COMPLETE_DE_NAMES: &str = r#"{ "names": [
  { "id": "organization.house_bjornaer", "name": "Haus Bjornaer" },
  { "id": "organization.order_of_hermes", "name": "Orden des Hermes" }
] }"#;

#[test]
fn catalogue_loads_and_exposes_known_values_by_id() {
    let catalogues = load_parameter_catalogues(organization_catalogue_json())
        .expect("a well-formed catalogue file loads");

    let organization = catalogues
        .get(&Id::new("catalogue.organization"))
        .expect("catalogue.organization must be present");

    // Known ids present — never a total count (CLAUDE.md: catalogue size is
    // data, never code; a future sourcebook may add more organizations).
    assert!(
        organization
            .value(&Id::new("organization.house_bjornaer"))
            .is_some(),
        "organization.house_bjornaer must be exposed by id"
    );
    assert!(
        organization
            .value(&Id::new("organization.order_of_hermes"))
            .is_some(),
        "organization.order_of_hermes must be exposed by id"
    );
}

#[test]
fn catalogue_value_exposes_its_source_ref() {
    let catalogues = load_parameter_catalogues(organization_catalogue_json())
        .expect("a well-formed catalogue file loads");
    let organization = catalogues
        .get(&Id::new("catalogue.organization"))
        .expect("catalogue.organization must be present");
    let bjornaer = organization
        .value(&Id::new("organization.house_bjornaer"))
        .expect("organization.house_bjornaer must be present");

    assert_eq!(
        bjornaer.source,
        SourceRef::new(
            "Ars Magica - Definitive Edition (Core Rules).md",
            LineRange::new(3563, 3565),
            "anchor",
        ),
        "the value's SourceRef must be the one the JSON declared"
    );
}

#[test]
fn duplicate_catalogue_id_fails_to_load_naming_it() {
    let json = r#"{
      "catalogues": [
        { "id": "catalogue.organization", "values": [
          { "id": "organization.house_bjornaer", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3563, 3565] } }
        ] },
        { "id": "catalogue.organization", "values": [
          { "id": "organization.order_of_hermes", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4063, 4065] } }
        ] }
      ]
    }"#;

    let err = load_parameter_catalogues(json)
        .expect_err("a duplicate catalogue id must fail to load, not silently keep one");
    let message = err.to_string();
    assert!(
        message.contains("catalogue.organization"),
        "error must name the offending catalogue id, got: {message}"
    );
}

#[test]
fn duplicate_value_id_within_catalogue_fails_to_load_naming_it() {
    let json = r#"{
      "catalogues": [
        { "id": "catalogue.organization", "values": [
          { "id": "organization.house_bjornaer", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [3563, 3565] } },
          { "id": "organization.house_bjornaer", "source": { "anchor": "anchor", "file": "Ars Magica - Definitive Edition (Core Rules).md", "lines": [4063, 4065] } }
        ] }
      ]
    }"#;

    let err = load_parameter_catalogues(json).expect_err(
        "a value id duplicated within one catalogue must fail to load, not silently keep one",
    );
    let message = err.to_string();
    assert!(
        message.contains("organization.house_bjornaer"),
        "error must name the offending value id, got: {message}"
    );
}

#[test]
fn catalogue_names_present_for_known_ids_in_both_locales() {
    let catalogues = organization_catalogues();
    let names = load_catalogue_names(&catalogues, COMPLETE_EN_NAMES, COMPLETE_DE_NAMES)
        .expect("complete, non-colliding both-locale names must load");

    let bjornaer_names = names
        .get(&Id::new("organization.house_bjornaer"))
        .expect("organization.house_bjornaer must have a resolved name entry");
    assert!(
        bjornaer_names.iter().any(|n| n == "House Bjornaer"),
        "the English name must be exposed, got: {bjornaer_names:?}"
    );
    assert!(
        bjornaer_names.iter().any(|n| n == "Haus Bjornaer"),
        "the German name must be exposed, got: {bjornaer_names:?}"
    );
}

#[test]
fn missing_de_name_fails_to_load_naming_the_id() {
    let catalogues = organization_catalogues();
    let de_missing_one = r#"{ "names": [
      { "id": "organization.house_bjornaer", "name": "Haus Bjornaer" }
    ] }"#;

    let err = load_catalogue_names(&catalogues, COMPLETE_EN_NAMES, de_missing_one).expect_err(
        "a catalogue value with no German name must fail to load, not silently pass through",
    );
    let message = err.to_string();
    assert!(
        message.contains("organization.order_of_hermes"),
        "error must name the id missing its German name, got: {message}"
    );
}

#[test]
fn cross_locale_name_collision_fails_to_load_naming_both_ids() {
    let catalogues = organization_catalogues();
    // "order_of_hermes"'s German name is deliberately spelled to collide,
    // case-insensitively and trimmed, with "house_bjornaer"'s English name.
    let en_names = r#"{ "names": [
      { "id": "organization.house_bjornaer", "name": "House Bjornaer" },
      { "id": "organization.order_of_hermes", "name": "Order of Hermes" }
    ] }"#;
    let de_names_colliding = r#"{ "names": [
      { "id": "organization.house_bjornaer", "name": "Haus Bjornaer" },
      { "id": "organization.order_of_hermes", "name": "  house bjornaer  " }
    ] }"#;

    let err = load_catalogue_names(&catalogues, en_names, de_names_colliding).expect_err(
        "two values colliding under trimmed, case-folded name comparison must fail to load",
    );
    let message = err.to_string();
    assert!(
        message.contains("organization.house_bjornaer")
            && message.contains("organization.order_of_hermes"),
        "error must name BOTH colliding ids, got: {message}"
    );
}

/// The real shipped catalogue and its both-locale names — a regression guard
/// on top of the fixture-based tests above, so the production data itself
/// (all three catalogues, including the five Educated-family values Norbert
/// approved for CV1: Arabic, Persian, Greek, Aramaic, Profession: Merchant)
/// loads clean end-to-end.
#[test]
fn shipped_catalogues_and_names_load_clean() {
    let catalogues_json = include_str!("../../../rules/core/parameter_catalogues.json");
    let en_json = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de_json = include_str!("../../../rules/i18n/de/parameter_catalogue.json");

    let catalogues =
        load_parameter_catalogues(catalogues_json).expect("shipped catalogue file loads clean");

    // Known ids present — never a total count. L1a: Dead and Living Language
    // have separate catalogues (try-out finding 6).
    for (catalogue_id, value_id) in [
        ("catalogue.language_dead", "language.latin"),
        ("catalogue.language_living", "language.arabic"),
        ("catalogue.language_living", "language.persian"),
        ("catalogue.language_living", "language.greek"),
        ("catalogue.language_living", "language.aramaic"),
        ("catalogue.organization", "organization.house_bjornaer"),
        ("catalogue.organization", "organization.order_of_hermes"),
        ("catalogue.profession", "profession.merchant"),
    ] {
        let catalogue = catalogues
            .get(&Id::new(catalogue_id))
            .unwrap_or_else(|| panic!("{catalogue_id} must be present"));
        assert!(
            catalogue.value(&Id::new(value_id)).is_some(),
            "{value_id} must be exposed by id in {catalogue_id}"
        );
    }

    let names = load_catalogue_names(&catalogues, en_json, de_json)
        .expect("shipped both-locale names load clean against the shipped catalogues");
    let latin_names = names
        .get(&Id::new("language.latin"))
        .expect("language.latin must have resolved names");
    assert!(latin_names.iter().any(|n| n == "Latin"));
    assert!(latin_names.iter().any(|n| n == "Latein"));
}
