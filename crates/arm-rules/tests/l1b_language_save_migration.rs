//! L1b (try-out finding 6, Norbert's decisions C5): the save migration that
//! follows L1a's split of the shared language catalogue.
//!
//! L1a gave Dead Language and Living Language separate catalogues (Dead = Latin,
//! Hebrew, Gothic; Living = Arabic, Greek, Persian, Aramaic). A save written
//! before the split (schema < 22) may hold a living language under Dead Language
//! (or the mirror), which today loads silently with a value no picker offers.
//! The 21 -> 22 migration moves such an instance to the Ability whose catalogue
//! holds its value, keeping score, banked XP and specialty. On a collision (the
//! target already holds the same value) the instance with the HIGHER score is
//! kept, together with its own banked XP (C5a); the other is dropped.
//!
//! The move is gated on the file's RAW `schema_version` (< 22), read before any
//! other fold stamps the version. A hand-edited schema-22 save is never moved;
//! validation reports the misplaced value instead.
//!
//! This file pins the entity state and the validation finding. The load
//! result's notice field is pinned in `l1b_language_move_notice.rs`, which names
//! a type that does not exist yet and so fails to compile on its own.
//!
//! Red-checkpoint protocol, phase 1: no move exists and `SCHEMA_VERSION` is still
//! 21, so every test below except the marked pins fails on its own assertion.

use std::collections::BTreeMap;

use arm_rules::migration::{LoadedEntity, SCHEMA_VERSION, load_entity_migrating};
use arm_rules::ruleset::{Ruleset, RulesetSources};
use arm_rules::types::{AbilityParameterValue, AbilityScore, Entity, Id, Selection};
use arm_rules::{DEFAULT_SAGA_YEAR, IssueSeverity, validate};

/// The schema version that introduces the move. A literal, not
/// `SCHEMA_VERSION`: these tests pin what a save written AT 22 means, which
/// must not drift if a later bump moves `SCHEMA_VERSION` on.
const SPLIT_VERSION: u32 = 22;
/// The last schema written before the split.
const PRE_SPLIT_VERSION: u32 = 21;

const DEAD: &str = "ability.dead_language";
const LIVING: &str = "ability.living_language";

/// The shipped core ruleset, loaded exactly as `book_templates.rs` does.
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

/// The shipped catalogue names, both locales.
fn catalogue_names(ruleset: &Ruleset) -> BTreeMap<Id, Vec<String>> {
    let en = include_str!("../../../rules/i18n/en/parameter_catalogue.json");
    let de = include_str!("../../../rules/i18n/de/parameter_catalogue.json");
    arm_rules::load_catalogue_names(ruleset.parameter_catalogues(), en, de)
        .expect("catalogue names load")
}

/// A minimal companion save at `schema_version`, with `ability_scores` spliced
/// in verbatim (a JSON array body) and `extra` spliced in as further top-level
/// keys (empty, or `"selections": [...],`).
fn save(schema_version: u32, extra: &str, ability_scores: &str) -> String {
    format!(
        r#"{{
          "schema_version": {schema_version},
          "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
          "entity_kind": "character",
          "type_id": "companion",
          "ability_funding": "pool",
          "saga_year": 1220,
          {extra}
          "ability_scores": [ {ability_scores} ]
        }}"#
    )
}

fn try_load(json: &str) -> Result<LoadedEntity, serde_json::Error> {
    let ruleset = full_ruleset();
    let names = catalogue_names(&ruleset);
    load_entity_migrating(json, DEFAULT_SAGA_YEAR, &ruleset, &names)
}

fn load(json: &str) -> LoadedEntity {
    try_load(json).unwrap_or_else(|e| panic!("fixture save must load: {e}"))
}

/// Every row of `ability`, in save order.
fn rows<'a>(entity: &'a Entity, ability: &str) -> Vec<&'a AbilityScore> {
    entity
        .ability_scores
        .iter()
        .filter(|s| s.ability == Id::new(ability))
        .collect()
}

fn catalogued(id: &str) -> Option<AbilityParameterValue> {
    Some(AbilityParameterValue::Catalogued { id: Id::new(id) })
}

/// The one row of `ability` holding catalogue value `value`, or a panic naming
/// every row of that Ability.
fn single_row<'a>(entity: &'a Entity, ability: &str, value: &str) -> &'a AbilityScore {
    let matching: Vec<_> = rows(entity, ability)
        .into_iter()
        .filter(|s| s.parameter == catalogued(value))
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "{ability} must hold exactly one {value} row, got: {:?}",
        rows(entity, ability)
    );
    matching[0]
}

// --- The move ------------------------------------------------------------------

/// A v21 save holding `language.arabic` under Dead Language: Arabic is a living
/// language now, so the instance moves to Living Language with its score, banked
/// XP and specialty intact, and the load stamps the new version.
#[test]
fn a_v21_dead_language_arabic_moves_to_living_language() {
    let loaded = load(&save(
        PRE_SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.dead_language", "score": 3, "specialty": "poetry",
             "parameter": { "id": "language.arabic" }, "banked_xp": 7 }"#,
    ));
    let entity = &loaded.entity;

    assert!(
        rows(entity, DEAD).is_empty(),
        "no Dead Language row may keep a living language, got: {:?}",
        entity.ability_scores
    );
    let moved = single_row(entity, LIVING, "language.arabic");
    assert_eq!(moved.score, 3, "the score travels with the instance");
    assert_eq!(
        moved.banked_xp, 7,
        "the banked XP travels with the instance"
    );
    assert_eq!(
        moved.specialty.as_deref(),
        Some("poetry"),
        "the specialty travels with the instance"
    );
    assert_eq!(
        entity.schema_version,
        arm_rules::SCHEMA_VERSION,
        "a load that moved something stamps the current version, like every other \
         meaning-changing fold"
    );
}

/// Typed text naming a living language (any case, either locale, a pre-18 bare
/// string or a `{text}` value) under Dead Language moves too, and lands as the
/// catalogue id, not as text. It must not ALSO be reported as an unrecognized
/// value: it was recognized, just under the wrong Ability.
#[test]
fn typed_living_language_text_under_dead_language_moves_as_its_catalogue_id() {
    for (version, parameter) in [
        (PRE_SPLIT_VERSION, r#"{ "text": "greek" }"#),
        (PRE_SPLIT_VERSION, r#"{ "text": " Griechisch " }"#),
        (17, r#""Greek""#),
    ] {
        let loaded = load(&save(
            version,
            "",
            &format!(
                r#"{{ "ability": "ability.dead_language", "score": 2, "parameter": {parameter} }}"#
            ),
        ));
        let entity = &loaded.entity;
        assert!(
            rows(entity, DEAD).is_empty(),
            "{parameter} at v{version}: no Dead Language row may keep Greek, got: {:?}",
            entity.ability_scores
        );
        assert_eq!(
            single_row(entity, LIVING, "language.greek").score,
            2,
            "{parameter} at v{version}: Greek moves to Living Language with its score"
        );
        assert!(
            loaded.unresolved_catalogued_parameters.is_empty(),
            "{parameter} at v{version}: a moved value is recognized, not unresolved, got: {:?}",
            loaded.unresolved_catalogued_parameters
        );
    }
}

/// The mirror: a dead language (Latin by id, Hebrew typed in German) under
/// Living Language moves to Dead Language.
#[test]
fn a_dead_language_under_living_language_moves_to_dead_language() {
    let loaded = load(&save(
        PRE_SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.living_language", "score": 5, "parameter": { "id": "language.latin" }, "banked_xp": 4 },
           { "ability": "ability.living_language", "score": 1, "parameter": { "text": "hebräisch" } }"#,
    ));
    let entity = &loaded.entity;

    assert!(
        rows(entity, LIVING).is_empty(),
        "no Living Language row may keep a dead language, got: {:?}",
        entity.ability_scores
    );
    let latin = single_row(entity, DEAD, "language.latin");
    assert_eq!((latin.score, latin.banked_xp), (5, 4));
    assert_eq!(single_row(entity, DEAD, "language.hebrew").score, 1);
}

/// A value that belongs where it is is never moved, and a save with nothing to
/// move keeps the version it was written at (no stamp without a change).
/// Pin: green today, must stay green.
#[test]
fn a_v21_save_with_correctly_placed_languages_is_left_alone() {
    let loaded = load(&save(
        PRE_SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.dead_language", "score": 4, "parameter": { "id": "language.latin" } },
           { "ability": "ability.living_language", "score": 3, "parameter": { "id": "language.arabic" } }"#,
    ));
    let entity = &loaded.entity;
    assert_eq!(single_row(entity, DEAD, "language.latin").score, 4);
    assert_eq!(single_row(entity, LIVING, "language.arabic").score, 3);
    assert_eq!(
        entity.schema_version, PRE_SPLIT_VERSION,
        "nothing moved, so nothing is stamped"
    );
}

// --- Collision (C5a) -------------------------------------------------------------

/// The moved instance has the higher score: it is kept, with its OWN banked XP
/// and specialty; the lower existing instance is dropped.
#[test]
fn a_collision_keeps_the_moved_instance_when_its_score_is_higher() {
    let loaded = load(&save(
        PRE_SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.dead_language", "score": 4, "specialty": "law",
             "parameter": { "id": "language.arabic" }, "banked_xp": 2 },
           { "ability": "ability.living_language", "score": 2, "specialty": "trade",
             "parameter": { "id": "language.arabic" }, "banked_xp": 9 }"#,
    ));
    let entity = &loaded.entity;
    assert!(
        rows(entity, DEAD).is_empty(),
        "got: {:?}",
        entity.ability_scores
    );
    let kept = single_row(entity, LIVING, "language.arabic");
    assert_eq!(
        (kept.score, kept.banked_xp, kept.specialty.as_deref()),
        (4, 2, Some("law")),
        "the higher score is kept together with its own banked XP and specialty"
    );
}

/// The existing instance has the higher score: it stays, and the moved one is
/// dropped. The existing value is typed text here ("Arabic"), which still
/// counts as the same value.
#[test]
fn a_collision_keeps_the_existing_instance_when_its_score_is_higher() {
    let loaded = load(&save(
        PRE_SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.dead_language", "score": 1, "parameter": { "id": "language.arabic" }, "banked_xp": 3 },
           { "ability": "ability.living_language", "score": 5, "parameter": { "text": "Arabic" } }"#,
    ));
    let entity = &loaded.entity;
    assert!(
        rows(entity, DEAD).is_empty(),
        "got: {:?}",
        entity.ability_scores
    );
    let kept = single_row(entity, LIVING, "language.arabic");
    assert_eq!(
        (kept.score, kept.banked_xp),
        (5, 0),
        "the existing, higher instance is kept with its own (zero) banked XP"
    );
}

/// Equal scores: the instance with more banked XP is kept, so no progress the
/// player recorded is thrown away when the scores cannot decide.
#[test]
fn a_collision_at_equal_scores_keeps_the_instance_with_more_banked_xp() {
    let loaded = load(&save(
        PRE_SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.dead_language", "score": 3, "parameter": { "id": "language.arabic" }, "banked_xp": 10 },
           { "ability": "ability.living_language", "score": 3, "parameter": { "id": "language.arabic" }, "banked_xp": 1 }"#,
    ));
    let kept = single_row(&loaded.entity, LIVING, "language.arabic");
    assert_eq!((kept.score, kept.banked_xp), (3, 10));
}

// --- Version gating and untrusted versions ----------------------------------------

/// A save written at 22 with the same content is NOT moved: the move belongs
/// to the 21 -> 22 migration only, and a 22 file was written by a build that
/// already had the split, so the value is a hand edit. It loads unchanged and
/// keeps its version. (Phase 1: the 21 build refuses a 22 file outright, so
/// this fails on the `is_ok` assertion.)
#[test]
fn a_v22_save_with_a_misplaced_language_is_not_moved() {
    let loaded = try_load(&save(
        SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.dead_language", "score": 3, "parameter": { "id": "language.arabic" } }"#,
    ));
    assert!(
        loaded.is_ok(),
        "a v22 save must load under the v22 build, got: {:?}",
        loaded.err()
    );
    let entity = loaded.unwrap().entity;
    assert_eq!(single_row(&entity, DEAD, "language.arabic").score, 3);
    assert!(
        rows(&entity, LIVING).is_empty(),
        "got: {:?}",
        entity.ability_scores
    );
    assert_eq!(entity.schema_version, SPLIT_VERSION);
}

/// `schema_version` is untrusted input. A version from the future is refused
/// (never half-migrated), whatever it claims; version 0 is simply old and is
/// migrated like any pre-22 save. None of them panics.
#[test]
fn untrusted_schema_versions_are_handled_safely() {
    let body = r#"{ "ability": "ability.dead_language", "score": 2, "parameter": { "id": "language.arabic" } }"#;
    for future in [9999, u32::MAX] {
        assert!(
            try_load(&save(future, "", body)).is_err(),
            "schema_version {future} is newer than the build and must be refused"
        );
    }

    let loaded = load(&save(0, "", body));
    assert!(
        rows(&loaded.entity, DEAD).is_empty(),
        "schema_version 0 predates the split, so the move runs, got: {:?}",
        loaded.entity.ability_scores
    );
    assert_eq!(
        single_row(&loaded.entity, LIVING, "language.arabic").score,
        2
    );
}

// --- Round trip ---------------------------------------------------------------------

/// Saves exactly as `arm-app`'s `ruleset_io.rs::save_entity_to_path` does: stamp
/// the current version, normalize, pretty-print.
fn save_bytes(mut entity: Entity) -> String {
    entity.schema_version = SCHEMA_VERSION;
    entity.normalize();
    serde_json::to_string_pretty(&entity).expect("an entity serializes")
}

/// A migrated save must not churn: load (moves) -> save -> load -> save is
/// byte-identical, and the second load moves nothing.
#[test]
fn a_migrated_save_is_byte_stable_across_save_load_save() {
    let first = load(&save(
        PRE_SPLIT_VERSION,
        "",
        r#"{ "ability": "ability.dead_language", "score": 4, "parameter": { "id": "language.arabic" }, "banked_xp": 2 },
           { "ability": "ability.living_language", "score": 2, "parameter": { "id": "language.arabic" } },
           { "ability": "ability.living_language", "score": 1, "parameter": { "id": "language.latin" } },
           { "ability": "ability.dead_language", "score": 3, "parameter": { "text": "Gothic" } }"#,
    ))
    .entity;
    assert_eq!(
        single_row(&first, LIVING, "language.arabic").score,
        4,
        "premise: the first load moved Arabic and resolved the collision"
    );
    assert_eq!(single_row(&first, DEAD, "language.latin").score, 1);

    let first_bytes = save_bytes(first);
    let second_bytes = save_bytes(load(&first_bytes).entity);
    assert_eq!(first_bytes, second_bytes);
}

// --- Validation of a hand-edited (>= 22) save ----------------------------------------

fn companion_with(ability: &str, parameter: AbilityParameterValue) -> Entity {
    let mut entity: Entity = serde_json::from_str(&save(PRE_SPLIT_VERSION, "", "")).unwrap();
    let mut score = AbilityScore::new(Id::new(ability), 1);
    score.parameter = Some(parameter);
    entity.ability_scores.push(score);
    entity
}

const OUTSIDE_CATALOGUE: &str = "ability_parameter_outside_catalogue";

fn outside_catalogue_issues(entity: &Entity) -> Vec<arm_rules::ValidationIssue> {
    validate(entity, &full_ruleset())
        .issues
        .into_iter()
        .filter(|i| i.code == OUTSIDE_CATALOGUE)
        .collect()
}

/// A catalogued id outside its Ability's own catalogue is an error naming the
/// Ability and the value, so a hand-edited 22 save (which the migration
/// deliberately leaves alone) is not silently accepted with a value no picker
/// offers. The same holds for an id that is in no catalogue at all.
#[test]
fn a_catalogued_id_outside_its_abilitys_catalogue_is_an_error() {
    for (ability, value) in [
        (DEAD, "language.arabic"),
        (LIVING, "language.latin"),
        (DEAD, "language.klingon"),
    ] {
        let issues = outside_catalogue_issues(&companion_with(
            ability,
            AbilityParameterValue::Catalogued { id: Id::new(value) },
        ));
        assert_eq!(
            issues.len(),
            1,
            "{ability} holding {value} must raise exactly one {OUTSIDE_CATALOGUE}, got: {issues:?}"
        );
        let issue = &issues[0];
        assert_eq!(issue.severity, IssueSeverity::Error);
        assert_eq!(
            issue.args.get("ability").map(String::as_str),
            Some(ability),
            "the finding names the Ability"
        );
        assert_eq!(
            issue.args.get("parameter").map(String::as_str),
            Some(value),
            "the finding names the value as `parameter`, so the UI composes the Ability \
             instance's name from the two (derive.ts ABILITY_INSTANCE_ARG)"
        );
    }
}

/// Pin (green today, must stay green): a value inside its Ability's catalogue,
/// and free text, raise no such finding.
#[test]
fn a_value_inside_its_catalogue_or_free_text_is_not_flagged() {
    for (ability, parameter) in [
        (
            DEAD,
            AbilityParameterValue::Catalogued {
                id: Id::new("language.latin"),
            },
        ),
        (
            LIVING,
            AbilityParameterValue::Catalogued {
                id: Id::new("language.greek"),
            },
        ),
        (DEAD, AbilityParameterValue::text("Old Norse")),
    ] {
        let issues = outside_catalogue_issues(&companion_with(ability, parameter.clone()));
        assert!(
            issues.is_empty(),
            "{ability} holding {parameter:?} must not be flagged, got: {issues:?}"
        );
    }
}

// --- Turb Trained -------------------------------------------------------------------

/// Turb Trained may learn "whichever single dead language the magi speak"
/// (ArMDE:5181), and L1a narrowed its `language` choice to the dead languages.
/// A pre-split save that chose Greek keeps that choice exactly as written: the
/// migration moves Ability instances, never a Virtue's recorded choice, so the
/// player sees `unknown_param_value` and re-chooses. Pin: green today (L1a), must
/// stay green through L1b.
#[test]
fn a_turb_trained_living_language_choice_is_left_for_validation_to_report() {
    let loaded = load(&save(
        PRE_SPLIT_VERSION,
        r#""selections": [ { "ref": "virtue.turb_trained", "params": { "language": "language.greek" } } ],"#,
        "",
    ));
    let entity = &loaded.entity;
    assert_eq!(
        entity.selections,
        vec![Selection::with_params(
            Id::new("virtue.turb_trained"),
            BTreeMap::from([("language".to_string(), Id::new("language.greek"))]),
        )],
        "the Virtue's choice is never rewritten by the migration"
    );
    let codes: Vec<String> = validate(entity, &full_ruleset())
        .issues
        .into_iter()
        .map(|i| i.code)
        .collect();
    assert!(
        codes.iter().any(|c| c == "unknown_param_value"),
        "Turb Trained's Greek is no longer a legal choice and must be reported, got: {codes:?}"
    );
}
