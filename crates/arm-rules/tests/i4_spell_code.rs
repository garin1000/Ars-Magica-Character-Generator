//! I4 (try-out finding 22): a spell's book code, requisites included.
//!
//! The rulebook writes a spell's Arts as one short code: the Technique
//! abbreviation, then the Form's, with a requisite in parentheses after the Art
//! it belongs with — a Technique requisite after the Technique, a Form requisite
//! after the Form, several of them comma-separated:
//!
//! - "Cr(Re)Ig 30 (Base 5, +1 Touch, +2 Sun, +1 requisite, +1 constant effect)"
//!   (ArMDE:19301)
//! - "ReAq(Co) 30 (Base 5, +1 Touch, +2 Sun, +1 requisite, +1 constant effect)"
//!   (ArMDE:19166)
//! - "MuTe(Aq, Co, An) 25 (Base 3, +2 Voice, +2 affect metal, +2 affect humans
//!   and animals)" (ArMDE:19241)
//!
//! `LocalizedRuleset::spell_code` is the one place that composes it, without the
//! level: the Spells tab appends the level in TypeScript, and the Markdown
//! export appends it in `export/sections.rs::spell_code`. The abbreviations are
//! the Art catalogue's own (`rules/i18n/<lang>/arts.json`), never a table in code;
//! the requisites are the spell's `requisites` ids, in data order.

use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::Id;

/// The whole shipped ruleset, localized with the shipped rules text of `lang`.
fn shipped(lang: &str) -> LocalizedRuleset {
    let ruleset = Ruleset::from_sources(RulesetSources {
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
    .expect("the shipped ruleset loads");
    let i18n: [&str; 2] = match lang {
        "de" => [
            include_str!("../../../rules/i18n/de/arts.json"),
            include_str!("../../../rules/i18n/de/spells.json"),
        ],
        _ => [
            include_str!("../../../rules/i18n/en/arts.json"),
            include_str!("../../../rules/i18n/en/spells.json"),
        ],
    };
    LocalizedRuleset::from_merged(ruleset, &i18n).expect("the shipped rules text loads")
}

/// A ruleset holding only Arts and spells — enough to compose codes the shipped
/// catalogue has no spell for (two Form requisites, two Technique requisites).
fn synthetic(spells: &str) -> LocalizedRuleset {
    let arts = r#"{ "advancement": [], "arts": [
      { "id": "art.creo", "art_type": "technique" },
      { "id": "art.muto", "art_type": "technique" },
      { "id": "art.rego", "art_type": "technique" },
      { "id": "art.animal", "art_type": "form" },
      { "id": "art.aquam", "art_type": "form" },
      { "id": "art.corpus", "art_type": "form" },
      { "id": "art.ignem", "art_type": "form" },
      { "id": "art.terram", "art_type": "form" }
    ] }"#;
    let ruleset = Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: "[]",
        type_profiles: "[]",
        arts: Some(arts),
        spells: Some(spells),
        ..RulesetSources::default()
    })
    .expect("the synthetic ruleset loads");
    let i18n = r#"{
      "art.creo": { "name": "Creo", "abbreviation": "Cr" },
      "art.muto": { "name": "Muto", "abbreviation": "Mu" },
      "art.rego": { "name": "Rego", "abbreviation": "Re" },
      "art.animal": { "name": "Animal", "abbreviation": "An" },
      "art.aquam": { "name": "Aquam", "abbreviation": "Aq" },
      "art.corpus": { "name": "Corpus", "abbreviation": "Co" },
      "art.ignem": { "name": "Ignem", "abbreviation": "Ig" },
      "art.terram": { "name": "Terram", "abbreviation": "Te" }
    }"#;
    LocalizedRuleset::new(ruleset, i18n).expect("the synthetic rules text loads")
}

fn code(localized: &LocalizedRuleset, spell: &str) -> Option<String> {
    localized.spell_code(&Id::new(spell))
}

/// A spell with no requisite is the plain Technique+Form pair.
/// Pilum of Fire, a Creo Ignem spell of level 20 with no "Req:" line
/// (ArMDE:14248-14253).
#[test]
fn a_spell_without_requisites_is_technique_then_form() {
    assert_eq!(
        code(&shipped("en"), "spell.pilum_of_fire").as_deref(),
        Some("CrIg")
    );
}

/// A Form requisite follows the Form. Phantasmal Fire, CrIm 20, "Req: Ignem"
/// (ArMDE:14554-14558).
#[test]
fn a_form_requisite_follows_the_form() {
    assert_eq!(
        code(&shipped("en"), "spell.phantasmal_fire").as_deref(),
        Some("CrIm(Ig)")
    );
}

/// A Technique requisite follows the Technique. Coat of Flame, CrIg 25,
/// "Req: Rego" (ArMDE:14262-14266) — the shape of "Cr(Re)Ig 30" (ArMDE:19301).
#[test]
fn a_technique_requisite_follows_the_technique() {
    assert_eq!(
        code(&shipped("en"), "spell.coat_of_flame").as_deref(),
        Some("Cr(Re)Ig")
    );
}

/// Each requisite goes to its own Art, whichever order the data lists them in.
/// Fog of Confusion, MuAu 45, "Req: Imaginem, Rego" (ArMDE:13241-13245): the
/// Rego requisite is a Technique, the Imaginem one a Form.
#[test]
fn mixed_requisites_each_follow_their_own_art() {
    assert_eq!(
        code(&shipped("en"), "spell.fog_of_confusion").as_deref(),
        Some("Mu(Re)Au(Im)")
    );
}

/// The Art abbreviations are Latin, so the code is the same in every language.
#[test]
fn the_code_is_the_same_in_german() {
    let de = shipped("de");
    assert_eq!(
        code(&de, "spell.phantasmal_fire").as_deref(),
        Some("CrIm(Ig)")
    );
    assert_eq!(
        code(&de, "spell.coat_of_flame").as_deref(),
        Some("Cr(Re)Ig")
    );
}

/// Several Form requisites are comma+space separated, in data order — not
/// sorted: "MuTe(Aq, Co, An) 25" (ArMDE:19241).
#[test]
fn several_form_requisites_are_comma_separated_in_data_order() {
    let rs = synthetic(
        r#"{ "spells": [
          { "id": "spell.dissolution", "technique": "art.muto", "form": "art.terram",
            "level": 25, "requisites": ["art.aquam", "art.corpus", "art.animal"] }
        ] }"#,
    );
    assert_eq!(
        code(&rs, "spell.dissolution").as_deref(),
        Some("MuTe(Aq, Co, An)")
    );
}

/// The book's Technique-requisite and Form-requisite examples, composed from
/// synthetic spells: "Cr(Re)Ig 30" (ArMDE:19301), "ReAq(Co) 30" (ArMDE:19166).
/// Two Technique requisites take the same comma form as two Form requisites.
#[test]
fn the_books_power_codes_compose_the_same_way() {
    let rs = synthetic(
        r#"{ "spells": [
          { "id": "spell.burn", "technique": "art.creo", "form": "art.ignem",
            "level": 30, "requisites": ["art.rego"] },
          { "id": "spell.drown", "technique": "art.rego", "form": "art.aquam",
            "level": 30, "requisites": ["art.corpus"] },
          { "id": "spell.two_techniques", "technique": "art.creo", "form": "art.ignem",
            "level": 30, "requisites": ["art.rego", "art.muto"] }
        ] }"#,
    );
    assert_eq!(code(&rs, "spell.burn").as_deref(), Some("Cr(Re)Ig"));
    assert_eq!(code(&rs, "spell.drown").as_deref(), Some("ReAq(Co)"));
    assert_eq!(
        code(&rs, "spell.two_techniques").as_deref(),
        Some("Cr(Re, Mu)Ig")
    );
}

/// A spell no catalogue holds has no Arts to abbreviate.
#[test]
fn an_unknown_spell_has_no_code() {
    assert_eq!(code(&shipped("en"), "spell.not_in_the_catalogue"), None);
}
