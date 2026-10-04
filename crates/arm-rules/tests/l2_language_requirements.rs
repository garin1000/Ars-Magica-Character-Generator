//! L2 (try-out finding 8, ruling F5): the language checks look at the actual
//! language, now that languages are catalogued (CV, L1a).
//!
//! 1. **The magus minimum is Latin, not any Dead Language.** "Magi must have the
//!    following minimum Abilities: Parma Magica 1, Magic Theory 1, Latin 1"
//!    (ArMDE:2437), and the recommendation is "Latin 4" (ArMDE:2451-2455). The
//!    requirement names the catalogue value `language.latin`, so Hebrew or Gothic
//!    no longer satisfies it.
//! 2. **The Academic requirement names its four languages.** "learning an Academic
//!    Knowledge normally requires a Latin, Greek, Hebrew, or Arabic score of at
//!    least 3" (ArMDE:7151); "Arabic, Greek and Hebrew fill similar functions,
//!    although of these only Hebrew is a dead language" (ArMDE:7432). So Latin or
//!    Hebrew as a Dead Language, or Greek or Arabic as a Living Language, at 3.
//! 3. **Typed text.** Norbert's decision on finding 8: typed text that equals a
//!    catalogue name (case-insensitive) counts as that language, anything else does
//!    not. A save's typed name is converted to the catalogue id on load
//!    (`migration.rs::fold_catalogue_matching`, both locales), pinned here. The
//!    in-session case (text typed into "Other…" and never reloaded) is pinned in
//!    `l2_typed_language_in_session.rs`. With no names attached to the ruleset, the
//!    engine treats typed text as no catalogue value at all (pinned below).
//!
//! Red-checkpoint protocol, phase 1: the shipped requirements name no instance and
//! the Academic check accepts any Dead Language, so every test not marked as a pin
//! fails on its own assertion.

use std::collections::BTreeMap;

use arm_rules::life_stage::AbilityRequirementKind;
use arm_rules::migration::{SCHEMA_VERSION, load_entity_migrating};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityParameterValue, AbilityScore, Entity, Id};
use arm_rules::{DEFAULT_SAGA_YEAR, magus_minimum_abilities, validate};

const DEAD: &str = "ability.dead_language";
const LIVING: &str = "ability.living_language";
const ACADEMIC_WARNING: &str = "academic_ability_without_scholarly_language";
const MINIMUM_ERROR: &str = "magus_minimum_ability";
const RECOMMENDED_WARNING: &str = "magus_recommended_ability";

const ABILITIES_JSON: &str = include_str!("../../../rules/core/abilities.json");
const LIFE_STAGES_JSON: &str = include_str!("../../../rules/core/life_stages.json");

/// The shipped core ruleset, loaded exactly as `book_templates.rs` does, with
/// `abilities.json` and `life_stages.json` passed in so a test can vary them.
fn ruleset_with(abilities: &str, life_stages: &str) -> Result<Ruleset, String> {
    Ruleset::from_sources(RulesetSources {
        id: "arm5-core",
        version: "2024.1",
        point_items: include_str!("../../../rules/core/virtues_flaws.json"),
        type_profiles: include_str!("../../../rules/core/character_types.json"),
        abilities: Some(abilities),
        arts: Some(include_str!("../../../rules/core/arts.json")),
        houses: Some(include_str!("../../../rules/core/houses.json")),
        mythic_types: Some(include_str!(
            "../../../rules/core/mythic_companion_types.json"
        )),
        spells: Some(include_str!("../../../rules/core/spells.json")),
        spell_mastery_abilities: None,
        equipment: Some(include_str!("../../../rules/core/equipment.json")),
        characteristics: Some(include_str!("../../../rules/core/characteristics.json")),
        life_stages: Some(life_stages),
        childhoods: None,
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
    })
    .map_err(|e| e.to_string())
}

/// The load error for these rules files, or `None` when they load. Returns only
/// the message, so a test expecting a failure never dumps a whole `Ruleset`.
fn load_error(abilities: &str, life_stages: &str) -> Option<String> {
    ruleset_with(abilities, life_stages).err()
}

fn full_ruleset() -> Ruleset {
    ruleset_with(ABILITIES_JSON, LIFE_STAGES_JSON).expect("shipped core ruleset loads")
}

/// The shipped catalogue names, both locales.
fn catalogue_names(ruleset: &Ruleset) -> BTreeMap<Id, Vec<String>> {
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

fn catalogued(id: &str) -> Option<AbilityParameterValue> {
    Some(AbilityParameterValue::Catalogued { id: Id::new(id) })
}

fn typed(text: &str) -> Option<AbilityParameterValue> {
    Some(AbilityParameterValue::text(text))
}

fn score(ability: &str, parameter: Option<AbilityParameterValue>, score: u8) -> AbilityScore {
    let mut row = AbilityScore::new(Id::new(ability), score);
    row.parameter = parameter;
    row
}

/// An in-memory character of `type_id` holding exactly `scores`.
fn character(type_id: &str, scores: Vec<AbilityScore>) -> Entity {
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new(type_id),
        arm_rules::RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    entity.ability_scores = scores;
    entity
}

/// A magus save at the current schema whose only Ability is Dead Language with
/// the given `parameter` JSON (e.g. `{ "text": "latin" }`), loaded through the
/// real load path with both locales' catalogue names.
fn loaded_magus_with_dead_language(parameter_json: &str, at: u8) -> Entity {
    let json = format!(
        r#"{{
          "schema_version": {SCHEMA_VERSION},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "magus",
          "ability_funding": "pool",
          "saga_year": 1220,
          "ability_scores": [
            {{ "ability": "{DEAD}", "score": {at}, "parameter": {parameter_json} }}
          ]
        }}"#
    );
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    load_entity_migrating(&json, DEFAULT_SAGA_YEAR, &ruleset, &names)
        .unwrap_or_else(|e| panic!("fixture save must load: {e}"))
        .entity
}

/// `(min_score, met)` of the dead-language rows of the magus checklist,
/// required first, then recommended.
fn dead_language_rows(
    entity: &Entity,
    ruleset: &Ruleset,
) -> Vec<(AbilityRequirementKind, u8, bool)> {
    magus_minimum_abilities(entity, ruleset)
        .into_iter()
        .filter(|row| row.ability == Id::new(DEAD))
        .map(|row| (row.requirement, row.min_score, row.met))
        .collect()
}

fn issue_codes(entity: &Entity, ruleset: &Ruleset) -> Vec<String> {
    validate(entity, ruleset)
        .issues
        .into_iter()
        .map(|issue| issue.code)
        .collect()
}

// --- 1. The magus minimum is Latin -----------------------------------------------

/// Both Latin rows of the shipped data name the catalogue value, not just the
/// Ability: "Latin 1" (ArMDE:2437) and the recommended "Latin 4" (ArMDE:2455).
#[test]
fn the_shipped_latin_minimum_and_recommendation_name_language_latin() {
    let rs = full_ruleset();
    let rows: Vec<(AbilityRequirementKind, Option<String>)> =
        magus_minimum_abilities(&character("magus", vec![]), &rs)
            .into_iter()
            .filter(|row| row.ability == Id::new(DEAD))
            .map(|row| (row.requirement, row.parameter.map(|p| p.to_string())))
            .collect();
    assert_eq!(
        rows,
        vec![
            (
                AbilityRequirementKind::Required,
                Some("language.latin".to_string())
            ),
            (
                AbilityRequirementKind::Recommended,
                Some("language.latin".to_string())
            ),
        ],
        "ArMDE:2437 demands Latin 1 and ArMDE:2455 recommends Latin 4, by name"
    );
}

/// The reversal of the old approximation: a magus whose only Dead Language is
/// Hebrew (or Gothic) is not admitted, however high the score.
#[test]
fn another_dead_language_does_not_meet_the_latin_minimum() {
    let rs = full_ruleset();
    for other in ["language.hebrew", "language.gothic"] {
        let magus = character("magus", vec![score(DEAD, catalogued(other), 5)]);
        assert_eq!(
            dead_language_rows(&magus, &rs),
            vec![
                (AbilityRequirementKind::Required, 1, false),
                (AbilityRequirementKind::Recommended, 4, false),
            ],
            "Dead Language {other} 5 is not Latin"
        );
        let codes = issue_codes(&magus, &rs);
        assert!(
            codes.iter().any(|c| c == MINIMUM_ERROR),
            "{other}: the Latin minimum must be an error, got {codes:?}"
        );
        assert!(
            codes.iter().any(|c| c == RECOMMENDED_WARNING),
            "{other}: the Latin recommendation must be a warning, got {codes:?}"
        );
    }
}

/// Pin: catalogued Latin meets both rows, by its bought score.
#[test]
fn catalogued_latin_meets_the_minimum_and_the_recommendation() {
    let rs = full_ruleset();
    let magus = character("magus", vec![score(DEAD, catalogued("language.latin"), 4)]);
    assert_eq!(
        dead_language_rows(&magus, &rs),
        vec![
            (AbilityRequirementKind::Required, 1, true),
            (AbilityRequirementKind::Recommended, 4, true),
        ]
    );
    let only_one = character("magus", vec![score(DEAD, catalogued("language.latin"), 1)]);
    assert_eq!(
        dead_language_rows(&only_one, &rs),
        vec![
            (AbilityRequirementKind::Required, 1, true),
            (AbilityRequirementKind::Recommended, 4, false),
        ]
    );
}

/// Pin: a save holding typed "latin" (English, any case) or "  Latein " (German,
/// padded) counts as Latin, because the load converts it to the catalogue id.
#[test]
fn typed_latin_in_either_locale_counts_once_loaded() {
    let rs = full_ruleset();
    for typed_name in [r#"{ "text": "latin" }"#, r#"{ "text": "  Latein " }"#] {
        let magus = loaded_magus_with_dead_language(typed_name, 4);
        assert_eq!(
            magus.ability_scores[0].parameter,
            catalogued("language.latin"),
            "{typed_name} must load as the catalogue value"
        );
        assert_eq!(
            dead_language_rows(&magus, &rs),
            vec![
                (AbilityRequirementKind::Required, 1, true),
                (AbilityRequirementKind::Recommended, 4, true),
            ],
            "{typed_name}"
        );
    }
}

/// Typed text naming no catalogue language ("Other…" → "Old Norse") is not Latin.
#[test]
fn typed_text_naming_no_catalogue_language_does_not_meet_the_minimum() {
    let rs = full_ruleset();
    let magus = loaded_magus_with_dead_language(r#"{ "text": "Old Norse" }"#, 4);
    assert_eq!(magus.ability_scores[0].parameter, typed("Old Norse"));
    assert_eq!(
        dead_language_rows(&magus, &rs),
        vec![
            (AbilityRequirementKind::Required, 1, false),
            (AbilityRequirementKind::Recommended, 4, false),
        ]
    );
}

/// With no catalogue names attached to the ruleset (a bare engine call), typed
/// text is never taken for a catalogue value, so in-memory "Latin" does not
/// count. The app attaches the names (`l2_typed_language_in_session.rs`).
#[test]
fn without_catalogue_names_typed_text_is_no_catalogue_value() {
    let rs = full_ruleset();
    let magus = character("magus", vec![score(DEAD, typed("Latin"), 4)]);
    assert_eq!(
        dead_language_rows(&magus, &rs),
        vec![
            (AbilityRequirementKind::Required, 1, false),
            (AbilityRequirementKind::Recommended, 4, false),
        ]
    );
}

/// A requirement naming a value outside its Ability's own catalogue is a broken
/// rules file: Arabic is a Living Language (ArMDE:7432), so "Dead Language:
/// Arabic" can never be bought, and the load must fail naming the value.
#[test]
fn a_minimum_naming_a_value_outside_its_abilitys_catalogue_fails_to_load() {
    let mut life_stages: serde_json::Value = serde_json::from_str(LIFE_STAGES_JSON).unwrap();
    let minimums = life_stages["apprenticeship"]["minimum_abilities"]
        .as_array_mut()
        .unwrap();
    let dead = minimums
        .iter_mut()
        .find(|r| r["ability"] == DEAD)
        .expect("the shipped minimums demand a dead language");
    dead["parameter"] = serde_json::Value::from("language.arabic");
    let error = load_error(ABILITIES_JSON, &life_stages.to_string())
        .expect("a minimum naming Dead Language: Arabic must fail the load");
    assert!(
        error.contains("language.arabic"),
        "the load error must name the offending value: {error}"
    );
}

// --- 2. The Academic requirement names its four languages ------------------------

/// An Academic Ability (Artes Liberales 1) plus the given languages.
fn scholar(languages: Vec<AbilityScore>) -> Entity {
    let mut scores = vec![score("ability.artes_liberales", None, 1)];
    scores.extend(languages);
    character("magus", scores)
}

fn academic_warnings(entity: &Entity, ruleset: &Ruleset) -> usize {
    issue_codes(entity, ruleset)
        .iter()
        .filter(|c| c.as_str() == ACADEMIC_WARNING)
        .count()
}

/// Greek and Arabic are not dead languages (ArMDE:7432), so they are bought as
/// Living Language — and at 3 they satisfy ArMDE:7151 (ruling F5).
#[test]
fn greek_or_arabic_at_three_as_a_living_language_satisfies_the_academic_requirement() {
    let rs = full_ruleset();
    for value in ["language.greek", "language.arabic"] {
        let entity = scholar(vec![score(LIVING, catalogued(value), 3)]);
        assert_eq!(
            academic_warnings(&entity, &rs),
            0,
            "Living Language {value} 3 satisfies ArMDE:7151"
        );
    }
}

/// Pin: Latin and Hebrew, the two dead ones, satisfy it as a Dead Language.
#[test]
fn latin_or_hebrew_at_three_as_a_dead_language_satisfies_the_academic_requirement() {
    let rs = full_ruleset();
    for value in ["language.latin", "language.hebrew"] {
        let entity = scholar(vec![score(DEAD, catalogued(value), 3)]);
        assert_eq!(
            academic_warnings(&entity, &rs),
            0,
            "Dead Language {value} 3 satisfies ArMDE:7151"
        );
    }
}

/// Any other language does not: Gothic (a dead language ArMDE:7151 does not
/// name), Persian (a living one it does not name), and typed text naming no
/// catalogue language.
#[test]
fn a_language_the_rules_do_not_name_does_not_satisfy_the_academic_requirement() {
    let rs = full_ruleset();
    let cases = [
        (
            "Dead Language: Gothic 3",
            score(DEAD, catalogued("language.gothic"), 3),
        ),
        (
            "Living Language: Persian 3",
            score(LIVING, catalogued("language.persian"), 3),
        ),
        (
            "Dead Language: typed Old Norse 3",
            score(DEAD, typed("Old Norse"), 3),
        ),
    ];
    for (case, language) in cases {
        let entity = scholar(vec![language]);
        assert_eq!(
            academic_warnings(&entity, &rs),
            1,
            "{case} must leave the ArMDE:7151 warning standing"
        );
    }
}

/// Pin: the score must reach 3.
#[test]
fn a_named_language_below_three_does_not_satisfy_the_academic_requirement() {
    let rs = full_ruleset();
    for language in [
        score(LIVING, catalogued("language.greek"), 2),
        score(DEAD, catalogued("language.latin"), 2),
    ] {
        let entity = scholar(vec![language.clone()]);
        assert_eq!(academic_warnings(&entity, &rs), 1, "{language:?}");
    }
}

/// Pin: the languages themselves do not trigger the check. Dead Language is an
/// Academic Ability, but holding only a Dead Language raises no warning.
#[test]
fn holding_only_a_language_raises_no_academic_warning() {
    let rs = full_ruleset();
    let entity = character("magus", vec![score(DEAD, catalogued("language.gothic"), 1)]);
    assert_eq!(academic_warnings(&entity, &rs), 0);
}

/// The warning names every qualifying language, in the order the rules data
/// lists them (Dead Language's Latin and Hebrew, then Living Language's Greek and
/// Arabic), as one comma-joined arg the UI localizes — and the score.
#[test]
fn the_academic_warning_names_every_qualifying_language_and_the_score() {
    let rs = full_ruleset();
    let entity = scholar(vec![]);
    let warning = validate(&entity, &rs)
        .issues
        .into_iter()
        .find(|issue| issue.code == ACADEMIC_WARNING)
        .expect("Artes Liberales with no language raises the warning");
    let expected: BTreeMap<String, String> = BTreeMap::from([
        (
            "languages".to_string(),
            "language.latin, language.hebrew, language.greek, language.arabic".to_string(),
        ),
        ("min".to_string(), "3".to_string()),
    ]);
    assert_eq!(
        warning
            .args
            .into_iter()
            .collect::<BTreeMap<String, String>>(),
        expected
    );
}

/// A scholarly language naming a value outside its Ability's catalogue is a
/// broken rules file, and the load must fail naming the value.
#[test]
fn a_scholarly_language_naming_a_value_outside_its_abilitys_catalogue_fails_to_load() {
    let mut abilities: serde_json::Value = serde_json::from_str(ABILITIES_JSON).unwrap();
    abilities["scholarly_language"] = serde_json::json!({
        "min_score": 3,
        "languages": [
            { "ability": DEAD, "values": ["language.latin", "language.persian"] },
            { "ability": LIVING, "values": ["language.greek", "language.arabic"] }
        ]
    });
    let error = load_error(&abilities.to_string(), LIFE_STAGES_JSON)
        .expect("Dead Language: Persian must fail the load");
    assert!(
        error.contains("language.persian"),
        "the load error must name the offending value: {error}"
    );
}
