//! I5b: a grant's fixed free-text parameter value is translatable text, so it
//! may not sit in a language-neutral mechanics file as English words.
//!
//! House Tremere grants "Minor Magical Focus (certamen)" (ArMDE:2281 `####
//! Hermetic Houses Summary`; also ArMDE:2064 `#### Tremere`), which the German
//! book prints as "Kleiner Magischer Fokus (Certamen)" (its same line). The
//! focus is a `text`-domain parameter, so a value typed into `rules/core/` was
//! shown as typed in both locales — lowercase in German, where Certamen is a
//! noun (`rules/source/de/translation-tables/grundbegriffe.md`, Certamen row).
//!
//! The fix follows the precedent CV1 set for rules-authored Ability instances
//! (`"literal": "language.latin"`): the core names a stable parameter-catalogue
//! value id, the catalogue value carries the rulebook provenance, and
//! `rules/i18n/<lang>/parameter_catalogue.json` supplies its text per locale.
//! The app already merges those names into the localized id → name map
//! (`ruleset_io.rs::merge_catalogue_display_names`), which is what both the UI's
//! `selectionParamLabel` and the export's `Doc::value_template` resolve a value
//! through — so the localized text reaches both surfaces from one place.

use arm_rules::export::{LABEL_KEYS, character_markdown};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::{Grant, I18nEntry};
use std::collections::{BTreeMap, BTreeSet};

/// The catalogue value House Tremere's focus names.
const CERTAMEN: &str = "magical_focus.certamen";

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

const EN_CATALOGUE_NAMES: &str = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
const DE_CATALOGUE_NAMES: &str = include_str!("../../../rules/i18n/de/parameter_catalogue.json");

/// Merges one locale's catalogue value names into the localized id → name map,
/// as the app does on every load (`ruleset_io.rs::merge_catalogue_display_names`).
fn with_catalogue_names(mut localized: LocalizedRuleset, names_json: &str) -> LocalizedRuleset {
    let names = arm_rules::parse_catalogue_names(names_json).expect("catalogue names parse");
    for (id, name) in names {
        localized.i18n.insert(
            id,
            I18nEntry {
                name,
                name_unfilled: None,
                summary: None,
                description: None,
                abbreviation: None,
                specialties: Vec::new(),
            },
        );
    }
    localized
}

/// The shipped ruleset as the app loads it in English.
fn shipped_en() -> LocalizedRuleset {
    let localized = LocalizedRuleset::from_merged(shipped_core(), &EN_I18N)
        .expect("the English rules text loads");
    with_catalogue_names(localized, EN_CATALOGUE_NAMES)
}

/// The shipped ruleset as the app loads it in German (English fallback).
fn shipped_de() -> LocalizedRuleset {
    let localized = LocalizedRuleset::from_merged_with_fallback(shipped_core(), &DE_I18N, &EN_I18N)
        .expect("the German rules text loads");
    with_catalogue_names(localized, DE_CATALOGUE_NAMES)
}

/// One fixed parameter value a shipped grant gives a `text`-domain parameter.
#[derive(Debug)]
struct GrantedText {
    owner: Id,
    item: Id,
    key: String,
    value: String,
}

/// Every value a shipped grant source (each House's and each Mythic Companion
/// type's fixed grants and choice options) fixes for a parameter the granted
/// item declares in the `text` domain. Open grants carry no fixed values.
fn shipped_granted_texts(ruleset: &Ruleset) -> Vec<GrantedText> {
    let owners = ruleset
        .houses()
        .map(|house| (&house.id, &house.grants))
        .chain(
            ruleset
                .mythic_types()
                .map(|mtype| (&mtype.id, &mtype.grants)),
        );
    let mut found = Vec::new();
    for (owner, grants) in owners {
        for grant in grants {
            let picks: Vec<(Id, BTreeMap<String, String>)> = match grant {
                Grant::Fixed { item, params, .. } => vec![(
                    item.clone(),
                    params
                        .iter()
                        .map(|(key, value)| (key.clone(), value.as_str().to_string()))
                        .collect(),
                )],
                Grant::Choice { options, .. } => options
                    .iter()
                    .map(|option| {
                        let singles = option
                            .params
                            .iter()
                            .filter_map(|(key, value)| {
                                Some((key.clone(), value.as_single()?.as_str().to_string()))
                            })
                            .collect();
                        (option.item_ref.clone(), singles)
                    })
                    .collect(),
                Grant::Open { .. } => Vec::new(),
            };
            for (item, params) in picks {
                let declared = ruleset
                    .item(&item)
                    .unwrap_or_else(|| panic!("{owner}: {item} is not a shipped point item"));
                for (key, value) in params {
                    let is_text = declared
                        .parameters
                        .iter()
                        .any(|p| p.key == key && p.domain == ParameterDomain::Text);
                    if is_text {
                        found.push(GrantedText {
                            owner: owner.clone(),
                            item: item.clone(),
                            key,
                            value,
                        });
                    }
                }
            }
        }
    }
    found
}

/// Whether `value` is a value of any shipped parameter catalogue.
fn is_catalogue_value(ruleset: &Ruleset, value: &str) -> bool {
    ruleset
        .parameter_catalogues()
        .values()
        .any(|catalogue| catalogue.value(&Id::new(value)).is_some())
}

// --- the data: a stable key in core, its text in i18n ----------------------------

/// House Tremere's focus is the catalogue value `magical_focus.certamen`, not the
/// English word, and that value exists in a shipped parameter catalogue.
#[test]
fn tremere_focus_names_a_catalogue_value_not_english_text() {
    let ruleset = shipped_core();
    let tremere = ruleset
        .house(&Id::new("house.tremere"))
        .expect("House Tremere ships");
    let focus = tremere.grants.iter().find_map(|grant| match grant {
        Grant::Fixed { item, params, .. } if item.as_str() == "virtue.minor_magical_focus" => {
            params.get("focus")
        }
        _ => None,
    });
    assert_eq!(focus, Some(&Id::new(CERTAMEN)));
    assert!(
        is_catalogue_value(&ruleset, CERTAMEN),
        "{CERTAMEN} must be a value of a shipped parameter catalogue"
    );
}

/// No shipped grant fixes a `text`-domain value as literal text: every such
/// value is a parameter-catalogue value id, so its words live in `rules/i18n/`.
#[test]
fn every_fixed_grant_text_value_is_a_catalogue_value() {
    let ruleset = shipped_core();
    let texts = shipped_granted_texts(&ruleset);
    assert!(
        texts
            .iter()
            .any(|t| t.owner.as_str() == "house.tremere" && t.key == "focus"),
        "vacuity guard: House Tremere's focus must be among the granted text values: {texts:?}"
    );
    let literals: Vec<&GrantedText> = texts
        .iter()
        .filter(|t| !is_catalogue_value(&ruleset, &t.value))
        .collect();
    assert!(
        literals.is_empty(),
        "grant text values that are not catalogue values (translatable text in core): {literals:?}"
    );
}

/// i18n parity: every fixed grant text value has a non-empty name in EVERY
/// shipped locale directory — read from disk, so a locale added later is held to
/// the same rule with no code change.
#[test]
fn every_fixed_grant_text_value_has_text_in_every_shipped_locale() {
    let ruleset = shipped_core();
    let texts = shipped_granted_texts(&ruleset);
    assert!(!texts.is_empty(), "vacuity guard: no granted text values");

    let i18n_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rules/i18n");
    let mut locales: Vec<std::path::PathBuf> = std::fs::read_dir(&i18n_root)
        .expect("rules/i18n is readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    locales.sort();
    assert!(locales.len() >= 2, "en and de ship: {locales:?}");

    let mut missing = Vec::new();
    for locale in &locales {
        let json = std::fs::read_to_string(locale.join("parameter_catalogue.json"))
            .unwrap_or_else(|e| panic!("{}: no parameter_catalogue.json: {e}", locale.display()));
        let names = arm_rules::parse_catalogue_names(&json).expect("catalogue names parse");
        for text in &texts {
            let named = names
                .get(&Id::new(text.value.as_str()))
                .is_some_and(|name| !name.trim().is_empty());
            if !named {
                missing.push(format!(
                    "{}: {} {} {}={}",
                    locale.display(),
                    text.owner,
                    text.item,
                    text.key,
                    text.value
                ));
            }
        }
    }
    assert!(missing.is_empty(), "values without text: {missing:?}");
}

/// The words themselves, as each book prints them: "Minor Magical Focus
/// (certamen)." (ArMDE:2281) and "Kleiner Magischer Fokus (Certamen)." (the
/// German book, same line). Certamen is kept untranslated, Latin, and a German
/// noun is capitalised (`grundbegriffe.md`, Certamen row).
#[test]
fn certamen_reads_as_each_book_prints_it() {
    let en = arm_rules::parse_catalogue_names(EN_CATALOGUE_NAMES).expect("en names parse");
    let de = arm_rules::parse_catalogue_names(DE_CATALOGUE_NAMES).expect("de names parse");
    assert_eq!(
        en.get(&Id::new(CERTAMEN)).map(String::as_str),
        Some("certamen")
    );
    assert_eq!(
        de.get(&Id::new(CERTAMEN)).map(String::as_str),
        Some("Certamen")
    );
}

// --- load-time integrity ----------------------------------------------------------

/// The focus Virtue, plus one item per engine-required V/F category so the
/// fixture ruleset passes the unrelated category-presence check.
const FOCUS_ITEM: &str = r#"[
    { "id": "virtue.minor_magical_focus", "kind": "virtue", "classification": "in_play_effect",
      "magnitude": "minor", "categories": ["hermetic"], "entity_kinds": ["character"],
      "parameters": [{ "key": "focus", "type": "ref", "domain": "text" }] },
    { "id": "virtue.self_confident", "kind": "virtue", "classification": "narrative",
      "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"] },
    { "id": "flaw.optimistic", "kind": "flaw", "classification": "narrative",
      "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"] }
]"#;

const FOCUS_CATALOGUE: &str = r#"{ "catalogues": [
    { "id": "catalogue.magical_focus", "values": [
        { "id": "magical_focus.certamen",
          "source": { "anchor": "hermetic-houses-summary",
                      "file": "Ars Magica - Definitive Edition (Core Rules).md",
                      "lines": [2281, 2281] } }
    ] }
] }"#;

fn house_granting_focus(value: &str) -> String {
    format!(
        r#"{{ "houses": [
            {{ "id": "house.tremere", "lineage_type": "true_lineage",
               "grants": [ {{ "kind": "fixed", "item": "virtue.minor_magical_focus",
                              "params": {{ "focus": "{value}" }} }} ] }}
        ] }}"#
    )
}

fn load_with_house(houses: &str) -> Result<Ruleset, arm_rules::RulesetError> {
    Ruleset::from_sources(RulesetSources {
        id: "test",
        version: "1",
        point_items: FOCUS_ITEM,
        type_profiles: "[]",
        abilities: None,
        arts: None,
        houses: Some(houses),
        mythic_types: None,
        spells: None,
        spell_mastery_abilities: None,
        equipment: None,
        characteristics: None,
        life_stages: None,
        childhoods: None,
        aging: None,
        parameter_catalogues: Some(FOCUS_CATALOGUE),
    })
}

/// A grant fixing a `text`-domain parameter to literal words fails the load,
/// naming the owner, the parameter and the value — so translatable text cannot
/// creep back into a mechanics file unnoticed.
#[test]
fn a_literal_text_value_in_a_grant_fails_the_load() {
    let Err(error) = load_with_house(&house_granting_focus("certamen")) else {
        panic!("a literal text value in a fixed grant must fail integrity, but the ruleset loaded");
    };
    let error = error.to_string();
    for needle in ["house.tremere", "focus", "certamen"] {
        assert!(error.contains(needle), "error must name {needle}: {error}");
    }
}

/// The same grant naming a catalogue value loads clean (guard for the rule above).
#[test]
fn a_catalogue_value_in_a_grant_loads() {
    load_with_house(&house_granting_focus(CERTAMEN)).expect("a catalogue value is a stable key");
}

// --- the export shows the localized text --------------------------------------

/// Every chrome key resolved to itself, as `export_golden.rs` does.
fn labels(rs: &LocalizedRuleset) -> BTreeMap<String, String> {
    let mut keys: BTreeSet<String> = LABEL_KEYS.iter().map(|k| k.to_string()).collect();
    for profile in rs.ruleset.profiles() {
        keys.insert(format!("type-{}", profile.id));
    }
    for item in rs.ruleset.items() {
        for category in &item.categories {
            keys.insert(format!("category-{category}"));
        }
        for param in &item.parameters {
            keys.insert(format!("param-label-{}", param.key));
        }
    }
    let mut labels: BTreeMap<String, String> = keys.into_iter().map(|k| (k.clone(), k)).collect();
    // The real separator both `locales/*/main.ftl` ship, as the app prints it.
    labels.insert("restricted-xp-list-separator".to_string(), ",".to_string());
    labels
}

fn tremere_magus() -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.house = Some(Id::new("house.tremere"));
    e.normalize();
    e
}

fn export(rs: &LocalizedRuleset) -> String {
    character_markdown(&tremere_magus(), rs, &labels(rs))
        .expect("every chrome key and every id in the fixture resolves")
}

#[test]
fn the_english_export_names_the_granted_focus() {
    let doc = export(&shipped_en());
    assert!(doc.contains("Minor Magical Focus (certamen)"), "{doc}");
    assert!(!doc.contains("magical_focus."), "a raw id leaked: {doc}");
}

#[test]
fn the_german_export_names_the_granted_focus() {
    let doc = export(&shipped_de());
    assert!(doc.contains("Kleiner Magischer Fokus (Certamen)"), "{doc}");
    assert!(!doc.contains("magical_focus."), "a raw id leaked: {doc}");
}

// --- the player's own focus stays free text -----------------------------------

const FOCUS: &str = "virtue.minor_magical_focus";

/// A Housless magus who BUYS Minor Magical Focus with the given `focus` value.
fn magus_buying_focus(focus: &str) -> Entity {
    let mut e = Entity::new(
        EntityKind::Character,
        Id::new("magus"),
        RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    e.selections = vec![Selection::with_params(
        Id::new(FOCUS),
        BTreeMap::from([("focus".to_string(), Id::new(focus))]),
    )];
    e.normalize();
    e
}

/// Every validation issue about the Magical Focus itself — what a bad `focus`
/// value would raise (`missing_param`, `unknown_param_value`, …).
fn focus_issues(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    arm_rules::validation::validate(entity, ruleset)
        .issues
        .into_iter()
        .filter(|issue| {
            issue
                .context
                .as_ref()
                .is_some_and(|id| id.as_str() == FOCUS)
        })
        .map(|issue| issue.code)
        .collect()
}

/// The catalogue key is for rules DATA only: a player still types their own
/// focus as free text, and it validates and exports as typed.
#[test]
fn a_typed_focus_is_still_free_text() {
    let ruleset = shipped_core();
    let fire = magus_buying_focus("fire");
    assert_eq!(focus_issues(&fire, &ruleset), Vec::<String>::new());

    let rs = shipped_de();
    let doc = character_markdown(&fire, &rs, &labels(&rs)).expect("the sheet exports");
    assert!(doc.contains("Kleiner Magischer Fokus (fire)"), "{doc}");
}

/// A catalogue id on the `text`-domain parameter is a legal value: the granted
/// Tremere copy, and a bought copy naming the same key, both validate clean.
#[test]
fn a_catalogue_id_on_the_text_parameter_validates() {
    let ruleset = shipped_core();
    assert_eq!(
        focus_issues(&tremere_magus(), &ruleset),
        Vec::<String>::new()
    );
    assert_eq!(
        focus_issues(&magus_buying_focus(CERTAMEN), &ruleset),
        Vec::<String>::new()
    );
}
