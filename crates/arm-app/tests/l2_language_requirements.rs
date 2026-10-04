//! L2, item 3 (Norbert's decision on try-out finding 8), through the app's own
//! ruleset loader: the ruleset the app validates with carries both locales'
//! catalogue names, so text typed into the "Other…" field that equals a catalogue
//! name counts as that language in the session, before any reload.
//!
//! This is the path every in-session `validate_entity` / `effective_scores` call
//! takes (`commands.rs` hands `LoadedRuleset.ruleset` to `validate_loaded` and
//! `effective_scores_loaded`), and the one the e2e helper
//! `satisfyMagusMinimums('Latin')` exercises by typing "Latin".
//!
//! Red-checkpoint protocol, phase 1: today any Dead Language satisfies the
//! minimum, so the two typed-Latin cases are green pins (they guard that L2's
//! narrowing does not lose typed names), and the "Old Norse" case is RED.

use std::path::PathBuf;

use arm_app::effective_dto::effective_scores_loaded;
use arm_app::ruleset_io::{load_ruleset_from_dir, validate_loaded};
use arm_rules::{AbilityParameterValue, AbilityScore, Entity, Id, ValidationMode};

const DEAD: &str = "ability.dead_language";
const MINIMUM_ERROR: &str = "magus_minimum_ability";

fn rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules")
}

/// A magus holding Parma Magica 1, Magic Theory 1 and Dead Language 1 with the
/// given typed instance — the three minimums of ArMDE:2437 as the e2e helper buys
/// them.
fn magus_typing(language: &str) -> Entity {
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new("arm5-core"), "2024.1"),
    );
    let mut dead = AbilityScore::new(Id::new(DEAD), 1);
    dead.parameter = Some(AbilityParameterValue::text(language));
    magus.ability_scores = vec![
        AbilityScore::new(Id::new("ability.parma_magica"), 1),
        AbilityScore::new(Id::new("ability.magic_theory"), 1),
        dead,
    ];
    magus
}

/// Whether validation reports the Latin minimum (a `magus_minimum_ability` about
/// Dead Language) as unmet.
fn latin_minimum_unmet(entity: &Entity, lang: &str) -> bool {
    let ruleset = load_ruleset_from_dir(&rules_dir(), lang).unwrap().ruleset;
    validate_loaded(entity, &ruleset, ValidationMode::Enforced)
        .issues
        .iter()
        .any(|issue| {
            issue.code == MINIMUM_ERROR
                && issue.args.get("ability").map(String::as_str) == Some(DEAD)
        })
}

/// Typed "Latin", and the German "Latein" under an English UI, meet the minimum
/// in the session, in both the validation and the checklist.
#[test]
fn typed_latin_meets_the_minimum_through_the_app_ruleset() {
    for (lang, typed) in [("en", "Latin"), ("en", "Latein"), ("de", "latein")] {
        let magus = magus_typing(typed);
        assert!(
            !latin_minimum_unmet(&magus, lang),
            "UI {lang}: typed {typed:?} under Dead Language is Latin 1"
        );
        let ruleset = load_ruleset_from_dir(&rules_dir(), lang).unwrap().ruleset;
        let checklist = effective_scores_loaded(&magus, &ruleset).magus_minimum_abilities;
        assert!(
            checklist
                .iter()
                .filter(|row| row.ability == Id::new(DEAD))
                .any(|row| row.min_score == 1 && row.met),
            "UI {lang}: the checklist must agree with validation for {typed:?}: {checklist:?}"
        );
    }
}

/// Typed text naming no catalogue language is not Latin.
#[test]
fn typed_text_naming_no_catalogue_language_leaves_the_minimum_unmet() {
    assert!(latin_minimum_unmet(&magus_typing("Old Norse"), "en"));
}

/// A temp copy of the shipped `rules/core` and `rules/i18n/en`, plus, for each
/// `(lang, json)` in `extra`, an `i18n/<lang>/parameter_catalogue.json`.
fn staged_rules(extra: &[(&str, &str)]) -> tempfile::TempDir {
    let staged = tempfile::tempdir().unwrap();
    for sub in ["core", "i18n/en"] {
        let target = staged.path().join(sub);
        std::fs::create_dir_all(&target).unwrap();
        for entry in std::fs::read_dir(rules_dir().join(sub)).unwrap() {
            let source = entry.unwrap().path();
            if source.is_file() {
                std::fs::copy(&source, target.join(source.file_name().unwrap())).unwrap();
            }
        }
    }
    for (lang, json) in extra {
        let target = staged.path().join("i18n").join(lang);
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("parameter_catalogue.json"), json).unwrap();
    }
    staged
}

fn latin_minimum_unmet_in(rules: &std::path::Path, entity: &Entity) -> bool {
    let ruleset = load_ruleset_from_dir(rules, "en").unwrap().ruleset;
    validate_loaded(entity, &ruleset, ValidationMode::Enforced)
        .issues
        .iter()
        .any(|issue| {
            issue.code == MINIMUM_ERROR
                && issue.args.get("ability").map(String::as_str) == Some(DEAD)
        })
}

/// The names come from every locale the rules directory ships, not a fixed
/// list: a third locale's name for Latin counts under an English UI.
#[test]
fn every_locale_the_rules_dir_ships_names_the_language() {
    let staged = staged_rules(&[(
        "la",
        r#"{ "names": [ { "id": "language.latin", "name": "Lingua Latina" } ] }"#,
    )]);
    assert!(!latin_minimum_unmet_in(
        staged.path(),
        &magus_typing("lingua latina")
    ));
}

/// And only those: a rules directory shipping English alone does not know the
/// German "Latein".
#[test]
fn a_locale_the_rules_dir_does_not_ship_names_nothing() {
    let staged = staged_rules(&[]);
    assert!(!latin_minimum_unmet_in(
        staged.path(),
        &magus_typing("Latin")
    ));
    assert!(latin_minimum_unmet_in(
        staged.path(),
        &magus_typing("Latein")
    ));
}
