//! Integration tests for the webview-free command logic. These exercise the
//! real loader, validator, and save/load against the repository's shipped rules
//! data — no Tauri runtime or webview required.

use std::fs;
use std::path::PathBuf;

use arm_app::error::AppError;
use arm_app::ruleset_io::{
    RULESET_ID, RULESET_VERSION, load_entity_from_path, load_ruleset_from_dir, save_entity_to_path,
    validate_loaded,
};
use arm_rules::{Entity, Id, Ruleset, ValidationMode};
use pretty_assertions::assert_eq;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules_dir() -> PathBuf {
    repo_root().join("rules")
}

fn sample_entity() -> Entity {
    let json = fs::read_to_string(repo_root().join("examples/companion_sample.json")).unwrap();
    serde_json::from_str(&json).unwrap()
}

#[test]
fn load_ruleset_yields_companion_profile_and_all_items() {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    // `Ruleset.id` is now an `Id` newtype; compare its string form.
    assert_eq!(localized.ruleset.id.as_str(), RULESET_ID);
    assert_eq!(localized.ruleset.version, RULESET_VERSION);
    assert_eq!(localized.ruleset.point_items.len(), 10);
    assert!(
        localized
            .ruleset
            .type_profiles
            .contains_key(&Id::new("companion"))
    );
}

#[test]
fn load_ruleset_localized_names_differ_between_languages() {
    let en = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let de = load_ruleset_from_dir(&rules_dir(), "de").unwrap();

    let id = Id::new("flaw.poor_student");
    let en_name = en.display_name(&id).expect("en name present");
    let de_name = de.display_name(&id).expect("de name present");
    assert_ne!(en_name, de_name, "translations should differ");
}

#[test]
fn load_ruleset_missing_language_is_io_error() {
    let err = load_ruleset_from_dir(&rules_dir(), "xx").unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

#[test]
fn load_ruleset_malformed_rules_is_ruleset_error() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("core")).unwrap();
    fs::create_dir_all(tmp.path().join("i18n/en")).unwrap();
    fs::write(tmp.path().join("core/virtues_flaws.json"), "not valid json").unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();

    let err = load_ruleset_from_dir(tmp.path(), "en").unwrap_err();
    let AppError::Ruleset {
        ruleset_kind,
        errors,
    } = &err
    else {
        panic!("expected ruleset error, got {err:?}");
    };
    assert_eq!(ruleset_kind, "parse", "malformed JSON is a parse failure");
    assert_eq!(errors.len(), 1, "parse failure carries one message");

    // The serialized payload the frontend receives must carry the engine kind
    // and the per-violation list, not a newline-joined English blob.
    let json: serde_json::Value = serde_json::to_value(&err).unwrap();
    assert_eq!(json["kind"], "ruleset");
    assert_eq!(json["ruleset_kind"], "parse");
    assert!(json["errors"].is_array(), "errors serialized as a list");
}

#[test]
fn integrity_failure_preserves_individual_messages() {
    // Two distinct unknown-prereq references must survive as two list entries,
    // with the engine's "integrity" kind preserved (not flattened to English).
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("core")).unwrap();
    fs::create_dir_all(tmp.path().join("i18n/en")).unwrap();
    fs::write(
        tmp.path().join("core/virtues_flaws.json"),
        r#"[
          {"id": "virtue.a", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"has": "virtue.x"}},
          {"id": "virtue.b", "kind": "virtue", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"has": "virtue.y"}}
        ]"#,
    )
    .unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();

    let err = load_ruleset_from_dir(tmp.path(), "en").unwrap_err();
    let AppError::Ruleset {
        ruleset_kind,
        errors,
    } = &err
    else {
        panic!("expected ruleset error, got {err:?}");
    };
    assert_eq!(ruleset_kind, "integrity");
    assert_eq!(
        errors.len(),
        2,
        "two distinct integrity violations preserved"
    );
}

#[test]
fn sample_companion_is_valid_in_enforced_mode() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let result = validate_loaded(&sample_entity(), &ruleset, ValidationMode::Enforced);
    assert!(result.is_valid(), "unexpected issues: {:?}", result.issues);
}

/// A companion picking a forbidden hermetic virtue produces errors. We reuse
/// this entity to prove the three validation modes behave differently.
fn companion_with_forbidden_item() -> Entity {
    let mut entity = sample_entity();
    entity
        .selections
        .push(arm_rules::Selection::new(Id::new("virtue.gentle_gift")));
    entity
}

#[test]
fn forbidden_item_errors_in_enforced_but_downgrades_in_advisory_and_clears_in_silent() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let entity = companion_with_forbidden_item();

    let enforced = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(!enforced.is_valid(), "should have errors in enforced mode");

    let advisory = validate_loaded(&entity, &ruleset, ValidationMode::Advisory);
    assert!(advisory.is_valid(), "advisory has no errors");
    assert!(!advisory.warnings().is_empty(), "advisory keeps warnings");

    let silent = validate_loaded(&entity, &ruleset, ValidationMode::Silent);
    assert!(silent.issues.is_empty(), "silent clears all issues");
}

#[test]
fn over_budget_virtues_is_reported() {
    // A deliberately tiny type profile (1 virtue point) forces an overrun the
    // shipped companion profile can't express with the current catalogue.
    let items = fs::read_to_string(rules_dir().join("core/virtues_flaws.json")).unwrap();
    let tiny_type = r#"[{
        "id": "tiny",
        "budget": { "virtue_points": 1, "flaw_points": 10 },
        "permitted_categories": ["general"],
        "creation_phases": ["virtues_flaws"]
    }]"#;
    let ruleset = Ruleset::from_json(RULESET_ID, RULESET_VERSION, &items, tiny_type).unwrap();

    let entity: Entity = serde_json::from_str(&format!(
        r#"{{
            "schema_version": 1,
            "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
            "entity_kind": "character",
            "type_id": "tiny",
            "selections": [{{ "ref": "virtue.keen_vision" }}, {{ "ref": "virtue.large" }}]
        }}"#
    ))
    .unwrap();

    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(
        result
            .issues
            .iter()
            .any(|i| i.code == "over_budget_virtues"),
        "expected over_budget_virtues, got {:?}",
        result.issues
    );
}

#[test]
fn save_then_load_round_trips_with_byte_stable_canonical_json() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.json");
    let entity = sample_entity();

    save_entity_to_path(&entity, &path).unwrap();
    let first = fs::read_to_string(&path).unwrap();

    let reloaded = load_entity_from_path(&path).unwrap();
    assert_eq!(reloaded, entity, "round trip must preserve the entity");

    // Re-saving the reloaded entity yields byte-identical output.
    save_entity_to_path(&reloaded, &path).unwrap();
    let second = fs::read_to_string(&path).unwrap();
    assert_eq!(first, second, "canonical output must be byte-stable");
}

#[test]
fn load_entity_from_missing_path_is_io_error() {
    let err = load_entity_from_path(&repo_root().join("does/not/exist.json")).unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

/// Extracts every `code: "..."` literal from the engine's validation source so
/// the Fluent coverage check tracks the codes the engine actually emits.
fn validation_codes() -> Vec<String> {
    let full = fs::read_to_string(repo_root().join("crates/arm-rules/src/validation.rs")).unwrap();
    // Ignore the in-file `#[cfg(test)]` module, whose fixtures use fake codes.
    let src = full.split("mod tests").next().unwrap();
    let mut codes = Vec::new();
    // The engine emits issues via `ValidationIssue::error("code", ...)` /
    // `::warning("code", ...)`; the code is the first string literal after the
    // opening paren (it may sit on the next line).
    for marker in ["::error(", "::warning("] {
        for fragment in src.split(marker).skip(1) {
            let Some(open) = fragment.find('"') else {
                continue;
            };
            let rest = &fragment[open + 1..];
            if let Some(end) = rest.find('"') {
                codes.push(rest[..end].to_string());
            }
        }
    }
    codes.sort();
    codes.dedup();
    assert!(!codes.is_empty(), "expected to find validation codes");
    codes
}

#[test]
fn every_validation_code_has_a_fluent_key_in_each_locale() {
    let codes = validation_codes();
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for code in &codes {
            assert!(
                ftl.contains(&format!("issue-{code} =")),
                "locale '{lang}' is missing key 'issue-{code}'"
            );
        }
    }
}
