//! Integration tests for the webview-free command logic. These exercise the
//! real loader, validator, and save/load against the repository's shipped rules
//! data — no Tauri runtime or webview required.

use std::fs;
use std::path::PathBuf;

use arm_app::error::AppError;
use arm_app::ruleset_io::{
    RULESET_ID, RULESET_VERSION, effective_scores_loaded, load_entity_from_path,
    load_ruleset_from_dir, pick_rules_dir, save_entity_to_path, validate_loaded,
};
use arm_rules::{ArtScore, Entity, Id, Ruleset, Selection, ValidationMode};
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

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
    // Catalogue size is data, not code: prove a known item loaded, never an exact
    // V/F total (which would break when any item is added to the JSON).
    assert!(
        localized
            .ruleset
            .item(&Id::new("virtue.the_gift"))
            .is_some()
    );
    assert!(localized.ruleset.profile(&Id::new("companion")).is_some());
}

#[test]
fn load_ruleset_yields_houses_with_localized_names() {
    // The shipped houses.json + its i18n must load through the real production
    // path (from_core_json in the engine tests does not exercise houses). Prove a
    // known House loaded and its display name is merged, never an exact count.
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    assert!(
        localized
            .ruleset
            .house(&Id::new("house.bjornaer"))
            .is_some()
    );
    assert_eq!(
        localized.display_name(&Id::new("house.bjornaer")),
        Some("Bjornaer")
    );
    // The House-granted Virtues and the Mystery abilities they seed also loaded.
    assert!(
        localized
            .ruleset
            .item(&Id::new("virtue.heartbeast"))
            .is_some()
    );
    assert!(
        localized
            .ruleset
            .ability(&Id::new("ability.heartbeast"))
            .is_some()
    );
}

#[test]
fn load_ruleset_yields_spells_with_localized_names() {
    // The shipped spells.json + its i18n must load through the real production
    // path. Prove a known spell loaded, its Technique/Form resolve, and its
    // display name is merged (localized) — never an exact count.
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let spell = localized
        .ruleset
        .spell(&Id::new("spell.pilum_of_fire"))
        .expect("Pilum of Fire loaded");
    assert_eq!(spell.technique, Id::new("art.creo"));
    assert_eq!(spell.form, Id::new("art.ignem"));
    assert_eq!(spell.level, Some(20));
    // A known non-ritual reads ritual = false (Pilum of Fire is Formulaic).
    assert!(!spell.ritual, "Pilum of Fire is not a ritual");
    assert_eq!(
        localized.display_name(&Id::new("spell.pilum_of_fire")),
        Some("Pilum of Fire")
    );
    // A General spell loads with no fixed level, and a known ritual reads
    // ritual = true (Aegis of the Hearth is a Year/Boundary ritual).
    let aegis = localized
        .ruleset
        .spell(&Id::new("spell.aegis_of_the_hearth"))
        .expect("Aegis of the Hearth loaded");
    assert_eq!(aegis.level, None);
    assert!(aegis.ritual, "Aegis of the Hearth is a ritual");
    // The magus profile carries the 120-level spell budget.
    assert_eq!(
        localized
            .ruleset
            .profile(&Id::new("magus"))
            .map(|p| p.spell_levels),
        Some(120)
    );
}

#[test]
fn load_ruleset_localizes_spell_names_in_german() {
    let localized = load_ruleset_from_dir(&rules_dir(), "de").unwrap();
    assert_eq!(
        localized.display_name(&Id::new("spell.pilum_of_fire")),
        Some("Pilum aus Feuer")
    );
}

#[test]
fn load_ruleset_yields_abilities_and_characteristics() {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    // Catalogue size is data, not code: prove the catalogue loaded via a known
    // ability, never an exact count (the full catalogue is a data-only add).
    assert!(
        localized
            .ruleset
            .ability(&Id::new("ability.awareness"))
            .is_some()
    );
    assert!(localized.ruleset.characteristic_rules().is_some());
    // Ability display names are merged into the localized text alongside V/F.
    assert_eq!(
        localized.display_name(&Id::new("ability.awareness")),
        Some("Awareness")
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

    // Ability names are localized too (Awareness -> Aufmerksamkeit).
    let aware = Id::new("ability.awareness");
    assert_ne!(
        en.display_name(&aware).unwrap(),
        de.display_name(&aware).unwrap()
    );
}

#[test]
fn sample_entity_with_characteristics_and_abilities_validates() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let entity = sample_entity();
    // The shipped sample now carries characteristics, ability scores, and a bank.
    assert_eq!(entity.schema_version, 12);
    assert!(!entity.characteristics.is_empty());
    assert!(!entity.ability_scores.is_empty());
    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(result.is_valid(), "unexpected issues: {:?}", result.issues);
}

#[test]
fn load_ruleset_missing_language_is_io_error() {
    let err = load_ruleset_from_dir(&rules_dir(), "xx").unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

#[test]
fn pick_rules_dir_returns_first_candidate_that_holds_rules() {
    // A portable Linux build's `BaseDirectory::Resource` points at a system path
    // that does not exist; the picker must skip it and fall back to the real
    // directory next to the executable.
    let empty = tempfile::tempdir().unwrap();
    let picked = pick_rules_dir(&[empty.path().to_path_buf(), rules_dir()]);
    assert_eq!(picked, Some(rules_dir()));
}

#[test]
fn pick_rules_dir_is_none_when_no_candidate_holds_rules() {
    let empty = tempfile::tempdir().unwrap();
    assert_eq!(pick_rules_dir(&[empty.path().to_path_buf()]), None);
}

#[test]
fn load_ruleset_malformed_rules_is_ruleset_error() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("core")).unwrap();
    fs::create_dir_all(tmp.path().join("i18n/en")).unwrap();
    fs::write(tmp.path().join("core/virtues_flaws.json"), "not valid json").unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("core/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/characteristics.json"), "").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/equipment.json"), "{}").unwrap();

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
        // Carries a personality-category item so the only integrity failures are
        // the two unresolved prerequisites (not the engine-required-category check).
        r#"[
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.x"}},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "category": "general", "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.y"}},
          {"id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "category": "personality", "entity_kinds": ["character"]}
        ]"#,
    )
    .unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("core/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/characteristics.json"), "").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/equipment.json"), "{}").unwrap();

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
    assert!(
        advisory.warnings().next().is_some(),
        "advisory keeps warnings"
    );

    let silent = validate_loaded(&entity, &ruleset, ValidationMode::Silent);
    assert!(silent.issues.is_empty(), "silent clears all issues");
}

#[test]
fn over_budget_virtues_is_reported() {
    // A deliberately tiny type profile (1 virtue point) forces an overrun the
    // shipped companion profile can't express with the current catalogue.
    let core = rules_dir().join("core");
    let items = fs::read_to_string(core.join("virtues_flaws.json")).unwrap();
    let abilities = fs::read_to_string(core.join("abilities.json")).unwrap();
    let arts = fs::read_to_string(core.join("arts.json")).unwrap();
    let characteristics = fs::read_to_string(core.join("characteristics.json")).unwrap();
    let tiny_type = r#"[{
        "id": "tiny",
        "budget": { "virtue_points": 1, "flaw_points": 10 },
        "permitted_categories": ["general"],
        "creation_phases": ["virtues_flaws"]
    }]"#;
    // The catalogue's ability_score_grant effects reference abilities, so the
    // ruleset must carry the ability registry for referential integrity to pass.
    let ruleset = Ruleset::from_core_json_with_arts(
        RULESET_ID,
        RULESET_VERSION,
        &items,
        tiny_type,
        &abilities,
        &arts,
        &characteristics,
    )
    .unwrap();

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
fn save_stamps_current_schema_version() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.json");
    let mut entity = sample_entity();
    entity.schema_version = 3; // a stale in-memory version must be corrected on write

    save_entity_to_path(&entity, &path).unwrap();
    let written = fs::read_to_string(&path).unwrap();
    assert!(
        written.contains("\"schema_version\": 12"),
        "save must stamp the current schema version, got: {written}"
    );
}

#[test]
fn sample_save_loads_with_defaulted_aging_warping_annotations() {
    // A shipped example save carries none of the new annotation fields; loading it
    // must fill them with their empty/None defaults (additive backward compat).
    let path = repo_root().join("examples/companion_sample.json");
    let entity = load_entity_from_path(&path).unwrap();
    assert_eq!(entity.apparent_age, None);
    assert!(entity.warping_effect.is_empty());
    assert!(entity.decrepitude_effect.is_empty());
    assert!(entity.aging_log.is_empty());
}

#[test]
fn arts_round_trip_and_puissant_art_reports_bonus() {
    // The shipped ruleset carries the Art registry and Puissant Art.
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    assert!(ruleset.art(&Id::new("art.ignem")).is_some());
    assert!(ruleset.art(&Id::new("art.creo")).is_some());

    // A character with two Arts and Puissant (Ignem). Arts draw from the shared
    // `xp_pool` alongside abilities (the sample already sets a pool).
    let mut entity = sample_entity();
    entity.art_scores = vec![
        ArtScore {
            art: Id::new("art.creo"),
            score: 3, // 6 xp
        },
        ArtScore {
            art: Id::new("art.ignem"),
            score: 5, // 15 xp
        },
    ];
    entity.selections.push(Selection::with_params(
        Id::new("virtue.puissant_art"),
        BTreeMap::from([("art".into(), Id::new("art.ignem"))]),
    ));

    // Puissant (Ignem) surfaces as a +3 bonus on Ignem only.
    let effective = effective_scores_loaded(&entity, &ruleset);
    let ignem = effective
        .art_bonuses
        .iter()
        .find(|b| b.art == Id::new("art.ignem"))
        .expect("Ignem bonus present");
    assert_eq!(ignem.bonus, 3);
    assert!(
        !effective
            .art_bonuses
            .iter()
            .any(|b| b.art == Id::new("art.creo")),
        "Creo is unboosted and omitted"
    );

    // The Arts survive a canonical save/load round trip; the save stamps the
    // current schema version.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("magus.json");
    save_entity_to_path(&entity, &path).unwrap();
    let reloaded = load_entity_from_path(&path).unwrap();
    assert_eq!(reloaded.schema_version, 12);
    assert_eq!(reloaded.art_scores, entity.art_scores);
}

#[test]
fn effective_scores_surface_aged_characteristic_and_drop_count() {
    // Core Rules.md:16613 worked example: Communication +2 with 3 aging points
    // drops once → effective +1. The summary must report the aged effective value
    // and the drop count, and omit unchanged Characteristics.
    use arm_rules::Characteristic;
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.characteristics.insert(Characteristic::Com, 2);
    entity.aging_points.insert(Characteristic::Com, 3);

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(
        effective.characteristic_effective.get(&Characteristic::Com),
        Some(&1)
    );
    assert_eq!(
        effective
            .characteristic_aging_drops
            .get(&Characteristic::Com),
        Some(&1)
    );
    // A Characteristic with no aging drop is omitted from the drop map.
    assert!(
        !effective
            .characteristic_aging_drops
            .contains_key(&Characteristic::Str)
    );
}

#[test]
fn effective_scores_surface_house_grants_read_only() {
    // The V/F view renders House grants read-only, so effective scores must carry
    // the derived grant Selections without the UI re-deriving them. A Bjornaer
    // magus is granted Heartbeast (a fixed grant), free of the point budget.
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.house = Some(Id::new("house.bjornaer"));

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert!(
        effective
            .granted_selections
            .iter()
            .any(|s| s.item_ref == Id::new("virtue.heartbeast")),
        "Bjornaer's granted Heartbeast should appear in granted_selections, got {:?}",
        effective.granted_selections
    );

    // A character with no House is granted nothing.
    let mut houseless = sample_entity();
    houseless.house = None;
    assert!(
        effective_scores_loaded(&houseless, &ruleset)
            .granted_selections
            .is_empty(),
        "a character with no House has no granted selections"
    );
}

#[test]
fn load_entity_from_missing_path_is_io_error() {
    let err = load_entity_from_path(&repo_root().join("does/not/exist.json")).unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

#[test]
fn effective_scores_surface_confidence_and_supernatural_slots() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // Companion default: Confidence 1/3, no free Supernatural slot (unGifted).
    let base = effective_scores_loaded(&companion, &ruleset);
    assert_eq!((base.confidence_score, base.confidence_points), (1, 3));
    assert_eq!(base.supernatural_free_total, 0);

    // Self-Confident raises Confidence to 2/5 (the shipped V/F).
    companion
        .selections
        .push(Selection::new(Id::new("virtue.self_confident")));
    let confident = effective_scores_loaded(&companion, &ruleset);
    assert_eq!(
        (confident.confidence_score, confident.confidence_points),
        (2, 5)
    );

    // Taking The Gift opens exactly one free Supernatural-Ability slot.
    companion
        .selections
        .push(Selection::new(Id::new("virtue.the_gift")));
    assert_eq!(
        effective_scores_loaded(&companion, &ruleset).supernatural_free_total,
        1
    );

    // Infamous surfaces a Local reputation grant for the UI add-control.
    companion
        .selections
        .push(Selection::new(Id::new("flaw.infamous")));
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert!(
        grants
            .iter()
            .any(|g| g.kind == arm_rules::ReputationType::Local && g.score == 4),
        "Infamous grants a level-4 Local reputation"
    );

    // A magus gets no free Supernatural slot (his free ability is Hermetic magic).
    let magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    assert_eq!(
        effective_scores_loaded(&magus, &ruleset).supernatural_free_total,
        0
    );
}

/// Extracts every fixed issue code from the engine's validation source so the
/// Fluent coverage check tracks the codes the engine actually emits. The engine
/// declares the closed set as `pub const CODE_*: &'static str = "...";` and
/// emits them via `ValidationIssue::CODE_*`, so the code values live in those
/// const definitions.
fn validation_codes() -> Vec<String> {
    // The engine's `validation` module was split into a directory
    // (`validation/mod.rs` + cohesive submodules); scan every `.rs` file in it so
    // codes declared in any submodule are still tracked.
    let dir = repo_root().join("crates/arm-rules/src/validation");
    let mut codes = Vec::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let full = fs::read_to_string(&path).unwrap();
        // Ignore each file's `#[cfg(test)]` module, whose fixtures use fake codes.
        let src = full.split("mod tests").next().unwrap();
        // Each fixed code is the string literal in a `const CODE_* : &'static str =
        // "code";` definition.
        for fragment in src.split("const CODE_").skip(1) {
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

/// The per-category flaw caps emit codes derived at runtime from the category
/// slug (`too_many_[major_]<category>_flaws`), so they are not source literals
/// the scraper above can see. Recompute them from the shipped profiles so the
/// Fluent coverage check still tracks every code the engine can actually emit.
fn dynamic_flaw_cap_codes() -> Vec<String> {
    let json = fs::read_to_string(repo_root().join("rules/core/character_types.json")).unwrap();
    let profiles: serde_json::Value = serde_json::from_str(&json).unwrap();
    let mut codes = Vec::new();
    for profile in profiles.as_array().unwrap() {
        let caps = profile["budget"]["flaw_category_caps"].as_array();
        for cap in caps.into_iter().flatten() {
            let category = cap["category"].as_str().unwrap();
            let major_only = cap["major_only"].as_bool().unwrap_or(false);
            codes.push(if major_only {
                format!("too_many_major_{category}_flaws")
            } else {
                format!("too_many_{category}_flaws")
            });
        }
    }
    codes.sort();
    codes.dedup();
    assert!(
        !codes.is_empty(),
        "expected shipped flaw-category cap codes"
    );
    codes
}

/// The per-category *virtue* caps emit codes derived at runtime from the
/// category slug (`too_many_[major_]<category>_virtues`) — the virtue analog of
/// [`dynamic_flaw_cap_codes`]. The magus `≤1 Major Hermetic Virtue` cap lives
/// here, so recompute it from the shipped profiles for the Fluent coverage check.
fn dynamic_virtue_cap_codes() -> Vec<String> {
    let json = fs::read_to_string(repo_root().join("rules/core/character_types.json")).unwrap();
    let profiles: serde_json::Value = serde_json::from_str(&json).unwrap();
    let mut codes = Vec::new();
    for profile in profiles.as_array().unwrap() {
        let caps = profile["budget"]["virtue_category_caps"].as_array();
        for cap in caps.into_iter().flatten() {
            let category = cap["category"].as_str().unwrap();
            let major_only = cap["major_only"].as_bool().unwrap_or(false);
            codes.push(if major_only {
                format!("too_many_major_{category}_virtues")
            } else {
                format!("too_many_{category}_virtues")
            });
        }
    }
    codes.sort();
    codes.dedup();
    assert!(
        !codes.is_empty(),
        "expected shipped virtue-category cap codes"
    );
    codes
}

#[test]
fn devil_child_resolves_infernal_might_and_power_budget_end_to_end() {
    // Building a Devil Child mythic companion against the shipped rules must yield
    // a nonzero effective Infernal Might and power-levels budget: the type requires
    // Demonic Blood (Infernal Might 5 + 30 power levels, RoP:Infernal:4120-4122) and
    // grants a choice of Demonic Might (+2) or Demonic Powers (+20 levels).
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("mythic_companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    entity.mythic_type = Some(Id::new("mythic_type.devil_child"));
    // Demonic Blood is a *required* Virtue (user-selected), not a type grant.
    entity
        .selections
        .push(Selection::new(Id::new("virtue.demonic_blood")));
    // Pick Demonic Might for the type's free-Minor choice (+2 Infernal Might).
    entity.mythic_choices.insert(
        "devil_child_free_minor".into(),
        Selection::new(Id::new("virtue.demonic_might")),
    );

    let scores = effective_scores_loaded(&entity, &ruleset);
    let might = scores
        .might
        .expect("a Devil Child has an effective Might score");
    assert_eq!(might.realm, arm_rules::Realm::Infernal);
    assert_eq!(might.score, 7); // 5 (Demonic Blood) + 2 (Demonic Might grant)
    assert_eq!(scores.power_levels_budget, 30); // Demonic Blood's 30 levels
}

#[test]
fn every_validation_code_has_a_fluent_key_in_each_locale() {
    let mut codes = validation_codes();
    codes.extend(dynamic_flaw_cap_codes());
    codes.extend(dynamic_virtue_cap_codes());
    codes.sort();
    codes.dedup();
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
