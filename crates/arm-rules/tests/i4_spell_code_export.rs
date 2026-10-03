//! I4: the spell code reaches the UI and the Markdown export from one source.
//!
//! - The UI receives the code per spell on the serialized `LocalizedRuleset`
//!   (`spell_codes`, keyed by spell id), composed by the engine — so the Spells
//!   tab never re-composes it from Art abbreviations.
//! - The export prints the same code plus the level in the book's form, a space
//!   apart: "Cr(Re)Ig 30" (ArMDE:19301), not `CrIg20`.
//! - A spell the player marked within a Magical Focus (ArMDE:4399-4422) or
//!   within a Potent Magic field (ArMDE:4740-4748) carries a short localized
//!   marker after the code (after-deadline answer 4), never a new column (D73.2).

use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use std::collections::{BTreeMap, BTreeSet};

const WITHIN_FOCUS_KEY: &str = "export-spell-within-focus";
const WITHIN_POTENT_FIELD_KEY: &str = "export-spell-within-potent-field";

fn shipped_core() -> Ruleset {
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
    .expect("the shipped ruleset loads")
}

const EN_I18N: [&str; 9] = [
    include_str!("../../../rules/i18n/en/virtues_flaws.json"),
    include_str!("../../../rules/i18n/en/abilities.json"),
    include_str!("../../../rules/i18n/en/arts.json"),
    include_str!("../../../rules/i18n/en/houses.json"),
    include_str!("../../../rules/i18n/en/mythic_companion_types.json"),
    include_str!("../../../rules/i18n/en/spells.json"),
    include_str!("../../../rules/i18n/en/spell_mastery_abilities.json"),
    include_str!("../../../rules/i18n/en/equipment.json"),
    include_str!("../../../rules/i18n/en/aging.json"),
];

const DE_I18N: [&str; 9] = [
    include_str!("../../../rules/i18n/de/virtues_flaws.json"),
    include_str!("../../../rules/i18n/de/abilities.json"),
    include_str!("../../../rules/i18n/de/arts.json"),
    include_str!("../../../rules/i18n/de/houses.json"),
    include_str!("../../../rules/i18n/de/mythic_companion_types.json"),
    include_str!("../../../rules/i18n/de/spells.json"),
    include_str!("../../../rules/i18n/de/spell_mastery_abilities.json"),
    include_str!("../../../rules/i18n/de/equipment.json"),
    include_str!("../../../rules/i18n/de/aging.json"),
];

fn shipped_en() -> LocalizedRuleset {
    LocalizedRuleset::from_merged(shipped_core(), &EN_I18N).expect("the English rules text loads")
}

/// Every chrome key resolved to itself (as `export_golden.rs` does), with the two
/// spell markers given readable values so a row reads as the sheet does.
fn labels(rs: &LocalizedRuleset) -> BTreeMap<String, String> {
    let mut keys: BTreeSet<String> = LABEL_KEYS.iter().map(|k| k.to_string()).collect();
    for profile in rs.ruleset.profiles() {
        keys.insert(format!("type-{}", profile.id));
    }
    for spell in rs.ruleset.spells() {
        for param in &spell.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    let mut map: BTreeMap<String, String> = keys.into_iter().map(|k| (k.clone(), k)).collect();
    map.insert(WITHIN_FOCUS_KEY.to_string(), "· Focus".to_string());
    map.insert(WITHIN_POTENT_FIELD_KEY.to_string(), "· Potent".to_string());
    map
}

fn magus_knowing(spells: Vec<SpellSelection>) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.spells = spells;
    e.normalize();
    e
}

fn spell(id: &str, within_focus: bool, within_potent_field: bool) -> SpellSelection {
    let mut s = SpellSelection::new(Id::new(id));
    s.within_focus = within_focus;
    s.within_potent_field = within_potent_field;
    s
}

fn export(spells: Vec<SpellSelection>) -> String {
    let rs = shipped_en();
    character_markdown(&magus_knowing(spells), &rs, &labels(&rs))
        .expect("every chrome key and every id in the fixture resolves")
}

// --- the code reaches the UI ------------------------------------------------

/// The serialized `LocalizedRuleset` carries each spell's engine-composed code,
/// keyed by spell id, so the Spells tab reads it instead of composing it.
#[test]
fn the_serialized_ruleset_carries_each_spells_code() {
    let json = serde_json::to_value(shipped_en()).expect("the ruleset serializes");
    let codes = &json["spell_codes"];
    assert_eq!(codes["spell.pilum_of_fire"], "CrIg", "{codes}");
    assert_eq!(codes["spell.phantasmal_fire"], "CrIm(Ig)", "{codes}");
    assert_eq!(codes["spell.coat_of_flame"], "Cr(Re)Ig", "{codes}");
    assert_eq!(codes["spell.fog_of_confusion"], "Mu(Re)Au(Im)", "{codes}");
}

/// The app loads German with an English fallback (`from_merged_with_fallback`);
/// that construction path carries the codes too, and they are the same Latin
/// abbreviations.
#[test]
fn the_fallback_construction_path_carries_the_codes_too() {
    let de = LocalizedRuleset::from_merged_with_fallback(shipped_core(), &DE_I18N, &EN_I18N)
        .expect("the German rules text loads");
    let json = serde_json::to_value(de).expect("the ruleset serializes");
    assert_eq!(
        json["spell_codes"]["spell.phantasmal_fire"], "CrIm(Ig)",
        "{}",
        json["spell_codes"]
    );
}

// --- the export prints the same code ----------------------------------------

/// The level follows the code a space apart, as the book prints it: "Cr(Re)Ig 30"
/// (ArMDE:19301).
#[test]
fn the_export_prints_code_space_level() {
    let doc = export(vec![spell("spell.pilum_of_fire", false, false)]);
    assert!(doc.contains("| CrIg 20 |"), "{doc}");
}

/// Phantasmal Fire, CrIm 20, "Req: Ignem" (ArMDE:14554-14558).
#[test]
fn the_export_prints_a_form_requisite() {
    let doc = export(vec![spell("spell.phantasmal_fire", false, false)]);
    assert!(doc.contains("| CrIm(Ig) 20 |"), "{doc}");
}

/// Coat of Flame, CrIg 25, "Req: Rego" (ArMDE:14262-14266).
#[test]
fn the_export_prints_a_technique_requisite() {
    let doc = export(vec![spell("spell.coat_of_flame", false, false)]);
    assert!(doc.contains("| Cr(Re)Ig 25 |"), "{doc}");
}

// --- the Focus / Potent markers ---------------------------------------------

#[test]
fn a_spell_within_focus_carries_the_focus_marker() {
    let doc = export(vec![spell("spell.pilum_of_fire", true, false)]);
    assert!(doc.contains("| CrIg 20 · Focus |"), "{doc}");
}

#[test]
fn a_spell_within_the_potent_field_carries_the_potent_marker() {
    let doc = export(vec![spell("spell.pilum_of_fire", false, true)]);
    assert!(doc.contains("| CrIg 20 · Potent |"), "{doc}");
}

/// The two marks are independent (D79): a spell may carry both, Focus first.
#[test]
fn a_spell_marked_both_ways_carries_both_markers() {
    let doc = export(vec![spell("spell.phantasmal_fire", true, true)]);
    assert!(doc.contains("| CrIm(Ig) 20 · Focus · Potent |"), "{doc}");
}

#[test]
fn an_unmarked_spell_carries_no_marker() {
    let doc = export(vec![spell("spell.pilum_of_fire", false, false)]);
    assert!(
        !doc.contains("· Focus") && !doc.contains("· Potent"),
        "{doc}"
    );
}

// --- the marker labels --------------------------------------------------------

/// Both markers are chrome the caller resolves, so the formatter declares them.
#[test]
fn label_keys_declare_both_spell_markers() {
    assert!(LABEL_KEYS.contains(&WITHIN_FOCUS_KEY), "{LABEL_KEYS:?}");
    assert!(
        LABEL_KEYS.contains(&WITHIN_POTENT_FIELD_KEY),
        "{LABEL_KEYS:?}"
    );
}

/// The markers' wording in both locales. German per the translation tables:
/// "Major Magical Focus | Großer Magischer Fokus" and "Potent Magic | Potente
/// Magie" (`rules/source/de/translation-tables/tugenden-fehler.md`); a standalone
/// label takes the uninflected form.
#[test]
fn both_locales_word_the_markers() {
    let en = include_str!("../../../locales/en/main.ftl");
    let de = include_str!("../../../locales/de/main.ftl");
    assert!(
        en.contains("\nexport-spell-within-focus = · Focus\n"),
        "en lacks the Focus marker"
    );
    assert!(
        en.contains("\nexport-spell-within-potent-field = · Potent\n"),
        "en lacks the Potent marker"
    );
    assert!(
        de.contains("\nexport-spell-within-focus = · Fokus\n"),
        "de lacks the Fokus marker"
    );
    assert!(
        de.contains("\nexport-spell-within-potent-field = · Potent\n"),
        "de lacks the Potent marker"
    );
}
