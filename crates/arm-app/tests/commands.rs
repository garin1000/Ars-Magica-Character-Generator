//! Integration tests for the webview-free command logic. These exercise the
//! real loader, validator, and save/load against the repository's shipped rules
//! data — no Tauri runtime or webview required.

use std::fs;
use std::path::{Path, PathBuf};

use arm_app::effective_dto::effective_scores_loaded;
use arm_app::error::AppError;
use arm_app::ruleset_io::{
    AgingApplication, AgingProjection, AgingReversion, ChildhoodApplication,
    apply_childhood_package_loaded, ensure_extension, export_markdown_to_path,
    load_catalogue_names_from_dir, load_entity_from_path, load_ruleset_from_dir,
    missing_core_files, path_text, pick_rules_dir, save_entity_to_path,
    unlink_ability_parameters_loaded, validate_loaded,
};
use arm_rules::{
    AbilityParameterValue, AbilityScore, ArtScore, CreationPhase, Entity, Id, Ruleset,
    RulesetSources, Selection, SpellSelection, ValidationMode, derived_totals,
};
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rules_dir() -> PathBuf {
    repo_root().join("rules")
}

/// The identity the **shipped** `rules/core/ruleset.json` declares, as a fixture
/// value for the saves these tests build. It lives here, in the tests, rather
/// than in the binary: the app must not author a ruleset's identity (full-audit
/// V6), so the only thing left that may name it is an expectation — and
/// [`load_ruleset_declares_the_identity_the_rules_data_carries`] pins the pair
/// against the file, so shipping different data fails loudly instead of drifting.
const RULESET_ID: &str = "arm5-core";
const RULESET_VERSION: &str = "2024.1";

fn sample_entity() -> Entity {
    let json = fs::read_to_string(repo_root().join("examples/companion_sample.json")).unwrap();
    serde_json::from_str(&json).unwrap()
}

/// The real shipped ruleset + catalogue names, for `load_entity_from_path`'s
/// CV4 dependency — every test in this file that loads a save through the real
/// app path needs both (design § 5.6).
fn shipped_ruleset_and_names() -> (Ruleset, BTreeMap<Id, Vec<String>>) {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let names = load_catalogue_names_from_dir(&rules_dir(), &localized.ruleset).unwrap();
    (localized.ruleset, names)
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

/// The shipped `core/childhoods.json` + its i18n must reach the app through the
/// real production loader: a package the engine can price is useless if the
/// binary never reads the file. Proves a known package loaded with the slots a UI
/// has to ask for, and that its name is localized in both languages — never a
/// package count, which is data.
///
/// Source: ArMDE:2388 (Traveling
/// Childhood), and its German name at
/// `Ars Magica Definitive Edition Basisregeln.md:2388`.
#[test]
fn load_ruleset_yields_childhood_packages_with_localized_names() {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let traveling = localized
        .ruleset
        .childhood(&Id::new("childhood.traveling"))
        .expect("Traveling Childhood loaded");

    // "Area A Lore 1, Area B Lore 1 … Living Language 1": two instances of one
    // parameterized Ability plus the spread's second language, each a slot the
    // player fills in.
    let slots: Vec<(&str, &str)> = traveling
        .slots()
        .map(|(slot, ability)| (slot, ability.as_str()))
        .collect();
    assert_eq!(
        slots,
        vec![
            ("area_a", "ability.area_lore"),
            ("area_b", "ability.area_lore"),
            ("language", "ability.living_language"),
        ]
    );
    assert_eq!(
        localized.display_name(&Id::new("childhood.traveling")),
        Some("Traveling Childhood")
    );

    let german = load_ruleset_from_dir(&rules_dir(), "de").unwrap();
    assert_eq!(
        german.display_name(&Id::new("childhood.traveling")),
        Some("Reisende Kindheit")
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
fn german_spell_descriptions_are_never_empty_via_english_fallback() {
    // German spell tooltips render the description; where a German description is
    // not yet translated, the loader fills it from English so the tooltip is never
    // empty. The name stays German (a present field is never overwritten by the
    // fallback). This exercises the loader's non-English fallback path end to end.
    let de = load_ruleset_from_dir(&rules_dir(), "de").unwrap();
    let pilum = Id::new("spell.pilum_of_fire");

    let de_desc = de
        .description(&pilum)
        .expect("German spell description must be present (via fallback if untranslated)");
    assert!(!de_desc.is_empty());
    assert_eq!(
        de.display_name(&pilum),
        Some("Pilum aus Feuer"),
        "the German name is not overwritten by the English fallback"
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

/// The shipped `core/aging.json` must reach the engine the way the app loads it —
/// through `load_ruleset_from_dir`, not through a test that reads the file bytes
/// itself. Structural assertions only: the two tables are catalogue *data*, so
/// their row counts belong in `data_integrity.rs`, never here.
#[test]
fn load_ruleset_yields_the_shipped_aging_tables() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let aging = ruleset
        .aging()
        .expect("the shipped ruleset ships aging rules");
    // "Characters begin aging in the Winter after they turn 35"
    // (ArMDE:16565).
    assert_eq!(aging.start_age, 35);
    assert!(
        !aging.living_conditions.is_empty(),
        "the Living Conditions table reached the engine"
    );
    assert!(
        !aging.outcomes.is_empty(),
        "the Aging Roll table reached the engine"
    );
}

/// Every shipped character type ends its guided rail with the Aging phase, and
/// the position is the point of the test: it guards against a later reorder that
/// would look harmless and quietly compute the aging total from an unfinished
/// character.
///
/// Aging is last because the rulebook puts it there. The creation summary
/// (ArMDE:2205-2222) runs steps 1..11
/// and never mentions aging at all; aging enters only in the next section,
/// "Starting Character Age", whose `ArMDE:2232` places the rolls "before the game
/// begins" — the last thing done to a built character, not one of the steps that
/// build it. Mechanically the total needs the finished character too: it reads
/// the final age, the final Characteristics (aging points reduce them, `ArMDE:16579`)
/// and the Longevity Ritual bonus, so any earlier slot would total up a
/// half-built character. The wizard appends its implicit `review` step after the
/// declared list, so declaring `aging` last makes the rail end
/// `… -> aging -> review`.
#[test]
fn every_shipped_profile_declares_the_aging_phase_last() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    assert!(ruleset.profile_count() > 0, "the ruleset ships profiles");
    for profile in ruleset.profiles() {
        assert_eq!(
            profile.creation_phases.last().map(|rule| rule.phase()),
            Some(CreationPhase::Aging),
            "profile '{}' must declare the aging phase last",
            profile.id
        );
    }
}

/// A2/D56: the shipped companion profile now declares Arts/Spells
/// conditionally on `hermetically_trained` (row 20/§ 6). A plain companion —
/// today's status quo, since `flaw.abandoned_apprentice` is not touched until
/// D3 — must see neither phase; a real magus, whose entries are all
/// unconditional, still sees both.
#[test]
fn a_plain_shipped_companion_sees_neither_arts_nor_spells_phases() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(ruleset.id.clone(), ruleset.version.clone()),
    );
    let phases = arm_rules::phases_in_force(&companion, &ruleset);
    assert!(
        !phases.contains(&CreationPhase::Arts) && !phases.contains(&CreationPhase::Spells),
        "an ordinary companion must not see the Arts/Spells phases: {phases:?}"
    );

    let magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(ruleset.id.clone(), ruleset.version.clone()),
    );
    let magus_phases = arm_rules::phases_in_force(&magus, &ruleset);
    assert!(
        magus_phases.contains(&CreationPhase::Arts)
            && magus_phases.contains(&CreationPhase::Spells),
        "a magus must still see the Arts/Spells phases: {magus_phases:?}"
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
    // The shipped sample now carries characteristics, ability scores, and a bank,
    // and is kept at the current schema version so a save/load round trip on it is
    // an identity (see `save_then_load_round_trips_with_byte_stable_canonical_json`).
    assert_eq!(entity.schema_version, 21);
    assert!(!entity.characteristics.is_empty());
    assert!(!entity.ability_scores.is_empty());
    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(result.is_valid(), "unexpected issues: {:?}", result.issues);
}

/// The payload the wizard reads is the one this command returns, and it must carry
/// the completeness report against the **shipped** profiles — the phase lists no
/// test fixture can stand in for. A brand-new magus has touched nothing, so every
/// declared step is outstanding.
#[test]
fn validating_a_fresh_character_reports_its_untouched_phases() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let entity = arm_rules::Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(ruleset.id.clone(), ruleset.version.clone()),
    );

    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    // Every declared step of a brand-new character is untouched: the read-only
    // `type` step that used to be the one exemption is gone (review #1).
    // `phases_in_force`, not the raw declared list (A2/D56): the magus's own
    // entries are all unconditionally in force, so this is unchanged for the
    // shipped profile, but it is the resolution the completeness report must
    // agree with once a conditional phase exists at all.
    let expected = arm_rules::phases_in_force(&entity, &ruleset);
    assert_eq!(result.completeness.incomplete_phases, expected);
}

/// The completeness report crosses the Tauri boundary on the validation payload and
/// `ui/src/lib/types.ts` mirrors it **by hand** — a sibling of
/// [`every_aging_field_is_mirrored_in_the_frontend_types`], for the same reason: a
/// renamed Rust field would leave the mirror compiling and the rail silently
/// reading `undefined`, i.e. nothing ever marked incomplete.
#[test]
fn the_completeness_report_is_mirrored_in_the_frontend_types() {
    let result = arm_rules::ValidationResult {
        issues: vec![],
        completeness: arm_rules::CompletenessReport {
            incomplete_phases: vec![arm_rules::CreationPhase::Abilities],
        },
    };

    let mut keys = std::collections::BTreeSet::new();
    mirrored_keys(&serde_json::to_value(&result).unwrap(), &mut keys);
    assert!(
        keys.contains("completeness") && keys.contains("incomplete_phases"),
        "expected the validation payload to carry the completeness field names, got {keys:?}"
    );

    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for key in keys.iter().map(String::as_str) {
        assert!(
            types.contains(&format!("{key}:")) || types.contains(&format!("{key}?:")),
            "ui/src/lib/types.ts declares no '{key}' property"
        );
    }
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

/// V9: a directory carrying only `core/character_types.json` used to pass
/// `pick_rules_dir`'s check (a single-file presence test), get accepted as
/// "the" rules directory, and only then fail deep inside
/// `load_ruleset_from_dir` with a raw "file not found" for whichever of the
/// other dozen files was missing — a confusing error that named neither the
/// directory nor what was actually wrong with it. The picker must now reject
/// a stale/partial directory outright and skip to the next candidate.
#[test]
fn pick_rules_dir_rejects_a_directory_carrying_only_one_required_file() {
    let stale = tempfile::tempdir().unwrap();
    fs::create_dir_all(stale.path().join("core")).unwrap();
    fs::write(stale.path().join("core/character_types.json"), "[]").unwrap();

    let picked = pick_rules_dir(&[stale.path().to_path_buf(), rules_dir()]);
    assert_eq!(
        picked,
        Some(rules_dir()),
        "a partial directory must be skipped in favor of a complete one"
    );
    assert_eq!(
        pick_rules_dir(&[stale.path().to_path_buf()]),
        None,
        "a partial directory alone must not be picked"
    );
}

/// [`missing_core_files`] is what lets a caller build a message naming
/// exactly what is missing, rather than a bare "not found" (V9).
#[test]
fn missing_core_files_names_every_absent_file() {
    let stale = tempfile::tempdir().unwrap();
    fs::create_dir_all(stale.path().join("core")).unwrap();
    fs::write(stale.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(stale.path().join("core/virtues_flaws.json"), "[]").unwrap();

    let missing = missing_core_files(stale.path());
    assert!(!missing.contains(&"core/character_types.json"));
    assert!(!missing.contains(&"core/virtues_flaws.json"));
    // Every other required core file is genuinely absent from this fixture.
    assert!(missing.contains(&"core/ruleset.json"));
    assert!(missing.contains(&"core/abilities.json"));
    assert!(missing.contains(&"core/arts.json"));
    assert!(missing.contains(&"core/houses.json"));
    assert!(missing.contains(&"core/mythic_companion_types.json"));
    assert!(missing.contains(&"core/spells.json"));
    assert!(missing.contains(&"core/spell_mastery_abilities.json"));
    assert!(missing.contains(&"core/equipment.json"));
    assert!(missing.contains(&"core/characteristics.json"));
    assert!(missing.contains(&"core/life_stages.json"));
    assert!(missing.contains(&"core/childhoods.json"));
    assert!(missing.contains(&"core/aging.json"));
    assert!(missing.contains(&"core/parameter_catalogues.json"));
    assert_eq!(missing.len(), 13);
}

#[test]
fn missing_core_files_is_empty_for_the_real_shipped_rules_directory() {
    assert_eq!(missing_core_files(&rules_dir()), Vec::<&str>::new());
}

#[test]
fn missing_core_files_is_the_full_list_for_a_directory_that_does_not_exist() {
    let missing = missing_core_files(&PathBuf::from("/does/not/exist/at/all"));
    assert_eq!(missing.len(), 15);
}

/// A temp copy of the shipped `rules/` (core + English i18n) whose
/// `core/ruleset.json` carries `identity` verbatim. A copy of the real data
/// rather than a synthetic minimal ruleset: what is under test is the shipped
/// loader reading the shipped files, with only the identity file swapped —
/// exactly the portable "house-ruled `rules/` beside the binary" layout.
fn staged_rules_with_identity(identity: &str) -> tempfile::TempDir {
    let staged = tempfile::tempdir().unwrap();
    for sub in ["core", "i18n/en"] {
        let target = staged.path().join(sub);
        fs::create_dir_all(&target).unwrap();
        for entry in fs::read_dir(rules_dir().join(sub)).unwrap() {
            let source = entry.unwrap().path();
            if source.is_file() {
                fs::copy(&source, target.join(source.file_name().unwrap())).unwrap();
            }
        }
    }
    fs::write(staged.path().join("core/ruleset.json"), identity).unwrap();
    staged
}

/// V6: a ruleset's identity is a property of the rules data, never of the
/// executable. It used to be a pair of `const`s in the binary, so a house-ruled
/// `rules/` directory produced saves stamped with the shipped ruleset's id and
/// version — indistinguishable from characters built against the real one.
#[test]
fn ruleset_identity_is_read_from_the_rules_data() {
    let staged = staged_rules_with_identity(r#"{ "id": "house-rules", "version": "9.9" }"#);

    let localized = load_ruleset_from_dir(staged.path(), "en").unwrap();

    assert_eq!(localized.ruleset.id.as_str(), "house-rules");
    assert_eq!(localized.ruleset.version, "9.9");
}

/// The shipped data's identity, and the fixture constants above, are one pair:
/// what `rules/core/ruleset.json` declares is what a loaded ruleset reports, and
/// every save these tests stamp with [`RULESET_ID`] is therefore stamped with the
/// shipped ruleset's own id. Editing the file without editing the constants fails
/// here rather than drifting.
#[test]
fn load_ruleset_declares_the_identity_the_rules_data_carries() {
    let declared: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(rules_dir().join("core/ruleset.json")).unwrap())
            .unwrap();

    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    assert_eq!(declared["id"], RULESET_ID);
    assert_eq!(declared["version"], RULESET_VERSION);
    assert_eq!(localized.ruleset.id.as_str(), declared["id"]);
    assert_eq!(localized.ruleset.version, declared["version"]);
}

/// The identity file is rules data like any other, so a malformed one fails the
/// load loudly rather than falling back to a default identity nothing declared.
#[test]
fn a_malformed_ruleset_identity_file_fails_the_load() {
    let staged = staged_rules_with_identity("not valid json");

    let err = load_ruleset_from_dir(staged.path(), "en").unwrap_err();

    let AppError::Ruleset {
        ruleset_kind,
        errors,
    } = &err
    else {
        panic!("expected ruleset error, got {err:?}");
    };
    assert_eq!(ruleset_kind, "parse", "malformed JSON is a parse failure");
    assert_eq!(errors.len(), 1, "parse failure carries one message");
    assert!(
        errors[0].contains("core/ruleset.json"),
        "the message must name the offending file, got {errors:?}"
    );
}

/// An empty id or version is malformed too: it would stamp every save with a
/// blank provenance that can never be compared against anything.
#[test]
fn a_blank_ruleset_identity_fails_the_load() {
    for identity in [
        r#"{ "id": "", "version": "2024.1" }"#,
        r#"{ "id": "arm5-core", "version": "" }"#,
    ] {
        let staged = staged_rules_with_identity(identity);

        let err = load_ruleset_from_dir(staged.path(), "en").unwrap_err();

        let AppError::Ruleset {
            ruleset_kind,
            errors,
        } = &err
        else {
            panic!("expected ruleset error for {identity}, got {err:?}");
        };
        assert_eq!(ruleset_kind, "parse");
        assert!(
            errors[0].contains("core/ruleset.json"),
            "the message must name the offending file, got {errors:?}"
        );
    }
}

/// The two halves of V6 meeting through the real production path: a character
/// authored against a house-ruled rules directory, reopened under the shipped
/// one, is told rather than silently re-priced.
#[test]
fn a_save_from_another_rules_directory_warns_when_reopened() {
    let staged = staged_rules_with_identity(r#"{ "id": "house-rules", "version": "9.9" }"#);
    let house_ruled = load_ruleset_from_dir(staged.path(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.ruleset =
        arm_rules::RulesetRef::new(house_ruled.id.clone(), house_ruled.version.clone());
    let shipped = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let result = validate_loaded(&entity, &shipped, ValidationMode::Enforced);

    let issue = result
        .issues
        .iter()
        .find(|i| i.code == "ruleset_mismatch")
        .unwrap_or_else(|| panic!("expected a mismatch warning, got {:?}", result.issues));
    assert_eq!(issue.args["saved_ruleset"], "house-rules");
    assert_eq!(issue.args["saved_version"], "9.9");
    assert_eq!(issue.args["loaded_ruleset"], RULESET_ID);
    assert_eq!(issue.args["loaded_version"], RULESET_VERSION);
    assert!(
        result.is_valid(),
        "a mismatch must not block the document: {:?}",
        result.issues
    );
}

/// And the ordinary case stays silent: a shipped example, validated against the
/// shipped ruleset, must not carry the warning — otherwise it is noise on every
/// document rather than a signal on the rare one.
#[test]
fn a_shipped_example_raises_no_ruleset_mismatch() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let result = validate_loaded(&sample_entity(), &ruleset, ValidationMode::Enforced);

    assert!(
        !result.issues.iter().any(|i| i.code == "ruleset_mismatch"),
        "issues: {:?}",
        result.issues
    );
}

#[test]
fn load_ruleset_malformed_rules_is_ruleset_error() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join("core")).unwrap();
    fs::create_dir_all(tmp.path().join("i18n/en")).unwrap();
    fs::write(
        tmp.path().join("core/ruleset.json"),
        r#"{ "id": "test", "version": "1" }"#,
    )
    .unwrap();
    fs::write(tmp.path().join("core/virtues_flaws.json"), "not valid json").unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("core/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spell_mastery_abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/characteristics.json"), "").unwrap();
    fs::write(tmp.path().join("core/life_stages.json"), "").unwrap();
    fs::write(tmp.path().join("core/childhoods.json"), "").unwrap();
    fs::write(tmp.path().join("core/aging.json"), "").unwrap();
    fs::write(tmp.path().join("core/parameter_catalogues.json"), "").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/spells.json"), "{}").unwrap();
    fs::write(
        tmp.path().join("i18n/en/spell_mastery_abilities.json"),
        "{}",
    )
    .unwrap();
    fs::write(tmp.path().join("i18n/en/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/childhoods.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/aging.json"), "{}").unwrap();

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
        tmp.path().join("core/ruleset.json"),
        r#"{ "id": "test", "version": "1" }"#,
    )
    .unwrap();
    fs::write(
        tmp.path().join("core/virtues_flaws.json"),
        // Carries a personality-category item so the only integrity failures are
        // the two unresolved prerequisites (not the engine-required-category check).
        r#"[
          {"id": "virtue.a", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.x"}},
          {"id": "virtue.b", "kind": "virtue", "classification": "narrative", "magnitude": "minor", "categories": ["general"], "entity_kinds": ["character"], "prerequisites": {"kind": "has", "value": "virtue.y"}},
          {"id": "flaw.optimistic", "kind": "flaw", "classification": "narrative", "magnitude": "major", "categories": ["personality"], "entity_kinds": ["character"]}
        ]"#,
    )
    .unwrap();
    fs::write(tmp.path().join("core/character_types.json"), "[]").unwrap();
    fs::write(tmp.path().join("core/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spells.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/spell_mastery_abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("core/characteristics.json"), "").unwrap();
    fs::write(tmp.path().join("core/life_stages.json"), "").unwrap();
    fs::write(tmp.path().join("core/childhoods.json"), "").unwrap();
    fs::write(tmp.path().join("core/aging.json"), "").unwrap();
    fs::write(tmp.path().join("core/parameter_catalogues.json"), "").unwrap();
    fs::write(tmp.path().join("i18n/en/virtues_flaws.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/abilities.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/arts.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/houses.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/mythic_companion_types.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/spells.json"), "{}").unwrap();
    fs::write(
        tmp.path().join("i18n/en/spell_mastery_abilities.json"),
        "{}",
    )
    .unwrap();
    fs::write(tmp.path().join("i18n/en/equipment.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/childhoods.json"), "{}").unwrap();
    fs::write(tmp.path().join("i18n/en/aging.json"), "{}").unwrap();

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

/// E4: the integrity messages are diagnostics for whoever is editing
/// `rules/`, so a terminal-launched binary must print them in full. Before this
/// they were computed, packed into [`AppError::Ruleset`], shipped over IPC and
/// read by nobody.
#[test]
fn ruleset_diagnostics_name_the_kind_and_list_every_message() {
    let err = AppError::Ruleset {
        ruleset_kind: "integrity".to_string(),
        errors: vec![
            "unknown prerequisite 'virtue.x' referenced by 'virtue.a'".to_string(),
            "unknown prerequisite 'virtue.y' referenced by 'virtue.b'".to_string(),
        ],
    };

    let mut sink: Vec<u8> = Vec::new();
    err.write_ruleset_diagnostics(&mut sink).unwrap();
    let report = String::from_utf8(sink).unwrap();

    // One header naming which check failed, then one line per violation — never a
    // single `; `-joined blob, which is what `Display` produces and what a reader
    // scanning for an id cannot use.
    let lines: Vec<&str> = report.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "a header plus one line per message: {report}"
    );
    assert!(
        lines[0].contains("integrity"),
        "header names the failing check: {report}"
    );
    assert!(
        lines[1].contains("unknown prerequisite 'virtue.x' referenced by 'virtue.a'"),
        "first violation printed verbatim: {report}"
    );
    assert!(
        lines[2].contains("unknown prerequisite 'virtue.y' referenced by 'virtue.b'"),
        "second violation printed verbatim: {report}"
    );
}

/// The emitter is called from the one `?` chain that loads the ruleset, which
/// also carries [`AppError::Io`] (no rules directory at all) — so every other
/// variant has to be silent rather than printing an empty header.
#[test]
fn a_non_ruleset_failure_writes_no_diagnostics() {
    let err = AppError::Io {
        message: "no such file".to_string(),
    };

    let mut sink: Vec<u8> = Vec::new();
    err.write_ruleset_diagnostics(&mut sink).unwrap();

    assert!(sink.is_empty(), "only a ruleset failure has diagnostics");
}

/// Printing must not consume the failure: the frontend still needs the very same
/// [`AppError`] to render its localized sentence and its detail disclosure.
#[test]
fn reporting_a_failure_hands_the_same_error_back() {
    let err = AppError::Ruleset {
        ruleset_kind: "parse".to_string(),
        errors: vec!["virtues_flaws.json: expected value at line 1".to_string()],
    };

    let returned = err.reported();

    let AppError::Ruleset {
        ruleset_kind,
        errors,
    } = returned
    else {
        panic!("expected the ruleset error back unchanged");
    };
    assert_eq!(ruleset_kind, "parse");
    assert_eq!(errors, vec!["virtues_flaws.json: expected value at line 1"]);
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
    let houses = fs::read_to_string(core.join("houses.json")).unwrap();
    let parameter_catalogues = fs::read_to_string(core.join("parameter_catalogues.json")).unwrap();
    // D3: `flaw.abandoned_apprentice` now carries `TruncatedApprenticeshipXp`,
    // which requires an `apprenticeship` block to bound its parameter
    // against — the shipped ruleset always ships one.
    let life_stages = fs::read_to_string(core.join("life_stages.json")).unwrap();
    let tiny_type = r#"[{
        "id": "tiny",
        "budget": { "virtue_points": 1, "flaw_points": 10 },
        "permitted_categories": ["general"],
        "creation_phases": ["virtues_flaws"]
    }]"#;
    // The catalogue's ability_score_grant effects reference abilities, and the
    // four Outer-Mystery Virtues carry a `House` prerequisite, so the ruleset
    // must carry both registries for referential integrity to pass.
    let ruleset = Ruleset::from_sources(RulesetSources {
        id: RULESET_ID,
        version: RULESET_VERSION,
        point_items: &items,
        type_profiles: tiny_type,
        abilities: Some(&abilities),
        arts: Some(&arts),
        houses: Some(&houses),
        characteristics: Some(characteristics.as_str()),
        life_stages: Some(&life_stages),
        parameter_catalogues: Some(&parameter_catalogues),
        ..RulesetSources::default()
    })
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
    let (ruleset, names) = shipped_ruleset_and_names();

    save_entity_to_path(&entity, &path).unwrap();
    let first = fs::read_to_string(&path).unwrap();

    let reloaded = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap()
    .entity;
    assert_eq!(reloaded, entity, "round trip must preserve the entity");

    // Re-saving the reloaded entity yields byte-identical output.
    save_entity_to_path(&reloaded, &path).unwrap();
    let second = fs::read_to_string(&path).unwrap();
    assert_eq!(first, second, "canonical output must be byte-stable");
}

/// A pre-schema-14 save carrying the flat `talisman_attunements` list migrates
/// through the **real** load path the app uses, not just the engine helper: the
/// attunements arrive under `Entity.talisman`, the version is bumped to current, and
/// re-saving writes only the new shape. This is the engine-boundary half of the
/// migration proof; the UI-boundary half is the `talisman` describe in
/// `ui/e2e/specs/magus-possessions.e2e.js`.
#[test]
fn legacy_talisman_save_migrates_through_the_real_load_path() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("legacy-magus.json");
    fs::write(
        &path,
        format!(
            r#"{{
              "schema_version": 13,
              "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
              "entity_kind": "character",
              "type_id": "magus",
              "selections": [{{ "ref": "virtue.the_gift" }}],
              "talisman_attunements": [
                {{ "description": "Projecting bolts and missiles", "bonus": 3 }}
              ]
            }}"#
        ),
    )
    .unwrap();

    let (ruleset, names) = shipped_ruleset_and_names();
    let migrated = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap()
    .entity;
    assert_eq!(
        migrated.schema_version, 21,
        "the field move bumps the schema"
    );
    let talisman = migrated
        .talisman
        .as_ref()
        .expect("legacy attunements become a talisman");
    assert_eq!(talisman.attunements.len(), 1);
    assert_eq!(talisman.attunements[0].bonus, 3);
    // Nothing is invented for the parts the old shape never stored.
    assert_eq!(talisman.description, "");
    assert!(talisman.effects.is_empty());

    // Saving the migrated entity writes the new shape only.
    save_entity_to_path(&migrated, &path).unwrap();
    let written = fs::read_to_string(&path).unwrap();
    assert!(!written.contains("talisman_attunements"), "got: {written}");
    assert!(written.contains("\"talisman\""), "got: {written}");
    assert!(written.contains("\"schema_version\": 21"), "got: {written}");
}

/// C8: the app's load door is what carries the user's configured default into the
/// engine's migration. `arm-rules` cannot read `settings.json` (engine purity), so
/// the year a pre-17 save inherits is decided here, by the caller that CAN.
#[test]
fn a_pre_17_save_inherits_the_configured_default_through_the_real_load_path() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("iberia-companion.json");
    fs::write(
        &path,
        format!(
            r#"{{
              "schema_version": 16,
              "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
              "entity_kind": "character",
              "type_id": "companion",
              "ability_funding": "pool",
              "age": 30,
              "birth_year": 1167
            }}"#
        ),
    )
    .unwrap();

    let (ruleset, names) = shipped_ruleset_and_names();
    let migrated = load_entity_from_path(&path, 1197, Some(&ruleset), Some(&names))
        .unwrap()
        .entity;
    assert_eq!(migrated.saga_year, 1197);
    assert_eq!(migrated.schema_version, arm_rules::SCHEMA_VERSION);

    // Round trip: saved and reopened under a DIFFERENT default, the year the
    // document now owns is the one that answers.
    save_entity_to_path(&migrated, &path).unwrap();
    let reopened = load_entity_from_path(&path, 1000, Some(&ruleset), Some(&names))
        .unwrap()
        .entity;
    assert_eq!(reopened.saga_year, 1197);
}

/// G5 (`tmp/export-audit.md`): `within_focus` predates schema 21, so a schema-20
/// save can carry it. The `virtue.rard` selection forces the 20 -> 21 rename
/// fold to run, and the mark must come through it, and through a re-save, intact.
#[test]
fn a_schema_20_save_keeps_a_within_focus_mark_through_migration() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("schema-20-magus.json");
    fs::write(
        &path,
        format!(
            r#"{{
              "schema_version": 20,
              "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
              "entity_kind": "character",
              "type_id": "magus",
              "ability_funding": "pool",
              "saga_year": 1220,
              "selections": [{{ "ref": "virtue.rard" }}],
              "spells": [{{ "spell": "spell.pilum_of_fire", "within_focus": true }}]
            }}"#
        ),
    )
    .unwrap();

    let (ruleset, names) = shipped_ruleset_and_names();
    let migrated = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap()
    .entity;
    assert_eq!(
        migrated.selections,
        vec![Selection::new(Id::new("virtue.bard"))],
        "fixture premise: the 20 -> 21 fold ran"
    );
    assert_eq!(migrated.schema_version, 21);
    assert_eq!(migrated.spells.len(), 1);
    assert!(
        migrated.spells[0].within_focus,
        "the within-focus mark must survive the schema-21 migration"
    );

    save_entity_to_path(&migrated, &path).unwrap();
    let reopened = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap()
    .entity;
    assert!(
        reopened.spells[0].within_focus,
        "the mark must survive the re-save of the migrated document too"
    );
}

/// G1 (`tmp/export-audit.md`), through the app's real save and load doors: two
/// Incompatible Arts copies with distinct pairs both survive, each with all four
/// combination keys, and the canonical output is byte-stable.
#[test]
fn two_incompatible_arts_copies_survive_save_then_load() {
    let copy = |t1: &str, f1: &str, t2: &str, f2: &str| {
        Selection::with_params(
            Id::new("flaw.incompatible_arts"),
            BTreeMap::from([
                ("technique_1".to_string(), Id::new(t1)),
                ("form_1".to_string(), Id::new(f1)),
                ("technique_2".to_string(), Id::new(t2)),
                ("form_2".to_string(), Id::new(f2)),
            ]),
        )
    };
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("incompatible-arts.json");
    let mut entity = sample_entity();
    entity.type_id = Id::new("magus");
    entity.selections = vec![
        copy("art.perdo", "art.ignem", "art.muto", "art.aquam"),
        copy("art.creo", "art.animal", "art.rego", "art.herbam"),
    ];
    entity.normalize();
    let (ruleset, names) = shipped_ruleset_and_names();

    save_entity_to_path(&entity, &path).unwrap();
    let first = fs::read_to_string(&path).unwrap();
    let reloaded = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap()
    .entity;

    assert_eq!(reloaded, entity, "both copies must round-trip unchanged");
    assert_eq!(reloaded.selections.len(), 2, "neither copy may be dropped");
    for selection in &reloaded.selections {
        let keys: Vec<&str> = selection.params.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            vec!["form_1", "form_2", "technique_1", "technique_2"],
            "every copy must keep all four combination keys"
        );
    }
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
        written.contains("\"schema_version\": 21"),
        "save must stamp the current schema version, got: {written}"
    );
}

/// Cluster C (CRITICAL — Gerda #2 + Klaus F6): a save must never truncate the
/// previous file in place. `fs::write` is `File::create` (O_TRUNC) +
/// `write_all`, so the character already on disk is emptied *before* the new
/// bytes land and any failure in between — a full disk, a lost network mount, a
/// process kill — leaves nothing at all. Plain Save writes straight to the
/// current path with no dialog, dozens of times a session, and there is no other
/// copy anywhere.
///
/// A second hard link is the witness: it names the very bytes the first save
/// wrote, and they stay reachable under it for as long as nothing overwrites
/// them. A save that REPLACES the file (scratch file in the same directory,
/// flushed, then renamed over the target) leaves them untouched; a save that
/// truncates in place overwrites them, which is precisely the act that destroys
/// the user's only copy when the write then fails halfway.
#[test]
fn a_re_save_replaces_the_previous_file_instead_of_truncating_it() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.armc");
    let entity = sample_entity();

    save_entity_to_path(&entity, &path).unwrap();
    let previous = fs::read_to_string(&path).unwrap();

    let witness = tmp.path().join("previous-bytes");
    fs::hard_link(&path, &witness).unwrap();

    let mut edited = entity.clone();
    edited.name = "Gerhard von Ratzeburg".to_owned();
    save_entity_to_path(&edited, &path).unwrap();

    assert_eq!(
        fs::read_to_string(&witness).unwrap(),
        previous,
        "the previous save's bytes must still be intact: a save that overwrites \
         them in place is one crash away from leaving the user with an empty file"
    );
    let written = fs::read_to_string(&path).unwrap();
    assert!(
        written.contains("Gerhard von Ratzeburg"),
        "the new save must be complete at the target path, got: {written}"
    );
}

/// The other half of Cluster C: replacing a file through a scratch copy must not
/// leave that scratch copy behind. A `.armc.tmp-…` sitting next to every save the
/// user makes would be its own defect — and it is what an atomic write does if it
/// forgets to clean up.
#[test]
fn a_successful_save_leaves_no_scratch_file_beside_the_target() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.armc");

    save_entity_to_path(&sample_entity(), &path).unwrap();
    save_entity_to_path(&sample_entity(), &path).unwrap();

    let mut entries: Vec<String> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec!["character.armc".to_string()],
        "a save must leave exactly the file it was asked to write"
    );
}

/// Cluster C's third site: the Markdown export writes the same way, so it gets
/// the same guarantee. Milder — an export is reproducible from the save — but a
/// second way to write a file is a second thing to keep correct, so both go
/// through one helper.
#[test]
fn a_re_export_replaces_the_previous_file_instead_of_truncating_it() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("companion.md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let labels = locale_labels("en");

    export_markdown_to_path(&sample_entity(), Some(&localized), &labels, &path).unwrap();
    let previous = fs::read_to_string(&path).unwrap();

    let witness = tmp.path().join("previous-bytes");
    fs::hard_link(&path, &witness).unwrap();

    let mut edited = sample_entity();
    edited.name = "Gerhard von Ratzeburg".to_owned();
    export_markdown_to_path(&edited, Some(&localized), &labels, &path).unwrap();

    assert_eq!(
        fs::read_to_string(&witness).unwrap(),
        previous,
        "the previous export's bytes must still be intact"
    );
    assert!(
        fs::read_to_string(&path).unwrap().contains("Gerhard"),
        "the new export must be complete at the target path"
    );
    let mut entries: Vec<String> = fs::read_dir(tmp.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    entries.sort();
    assert_eq!(
        entries,
        vec!["companion.md".to_string(), "previous-bytes".to_string()],
        "an export must leave no scratch file behind"
    );
}

/// Klaus F5 (MINOR): `lang` is joined straight into a path
/// (`rules_dir/i18n/<lang>/<file>`), and it arrives as a bare `String` — from
/// the `load_ruleset` command with no allowlist, and from `settings.json`, whose
/// deliberately lenient deserializer accepts any string at all. `"../../.."`
/// resolves outside the rules directory.
///
/// The impact is near-inert by this app's threat model (`CLAUDE.md`): the access
/// is read-only, the final component is one of ten fixed filenames, and there is
/// no remote attacker. It is fixed because it is the codebase's only unvalidated
/// string-to-path join, and the next one might not be inert.
///
/// The guard sits in `read_i18n_sources`, not in the command shim, so the
/// settings path and the IPC path share one check.
#[test]
fn a_language_that_is_not_a_language_tag_is_refused() {
    for rejected in ["../etc", "en/../..", "", ".", "en/de"] {
        let err = load_ruleset_from_dir(&rules_dir(), rejected).unwrap_err();
        let AppError::Io { message } = err else {
            panic!("expected an Io refusal for {rejected:?}, got {err:?}");
        };
        assert!(
            message.contains(rejected),
            "the refusal must name the value it rejected ({rejected:?}), got: \
             {message}"
        );
    }
}

/// The other half: the languages the app actually ships still load. A guard that
/// refused a real language would take the whole ruleset down with it.
#[test]
fn the_shipped_languages_pass_the_language_tag_guard() {
    for accepted in ["en", "de"] {
        assert!(
            load_ruleset_from_dir(&rules_dir(), accepted).is_ok(),
            "{accepted} must still load"
        );
    }
}

/// Viktor #4 (MINOR): a schema migration must reach the user, not a terminal
/// nobody is looking at. The load used to format its report into an English
/// `eprintln!` and return a bare `Entity`, dropping it — and a GUI binary
/// launched from a desktop launcher has no attached terminal at all. The
/// migration is lossy (`minimal_aging_points_for_drops` reconstructs a
/// *minimal* total, not the one the character really accumulated) and the next
/// save makes it permanent, so going unmentioned is the problem.
///
/// The engine already holds up its end: `migration.rs::LoadedEntity` carries the
/// migrated Characteristics and states that "the caller surfaces a localized
/// notice; the engine holds no user-facing string". This is the caller doing
/// so — the app's load door hands the report on rather than swallowing it.
#[test]
fn the_load_reports_which_characteristics_a_migration_rewrote() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("legacy-companion.json");
    fs::write(
        &path,
        format!(
            r#"{{
              "schema_version": 9,
              "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
              "entity_kind": "character",
              "type_id": "companion",
              "characteristics": {{ "com": 2 }},
              "aging_reductions": {{ "com": 1 }}
            }}"#
        ),
    )
    .unwrap();

    let (ruleset, names) = shipped_ruleset_and_names();
    let loaded = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap();
    assert_eq!(
        loaded.migrated_aging_characteristics,
        vec![arm_rules::Characteristic::Com],
        "the rewritten Characteristics must reach the caller that can tell the \
         user about them"
    );
    assert_eq!(loaded.entity.schema_version, arm_rules::SCHEMA_VERSION);
}

/// A save that needed no migration reports none, so the frontend has nothing to
/// announce — the ordinary case, and the one a notice must not fire on.
#[test]
fn an_up_to_date_save_reports_no_migration() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("current.armc");
    save_entity_to_path(&sample_entity(), &path).unwrap();

    let (ruleset, names) = shipped_ruleset_and_names();
    let loaded = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap();
    assert!(loaded.migrated_aging_characteristics.is_empty());
}

/// The wire contract the frontend reads. `path` and `entity` keep their names —
/// the shape the UI already consumes is unchanged — and the migration report
/// rides alongside them, always present so the frontend has one field to check
/// rather than an optional to distinguish from a stale build.
///
/// The struct is `OpenedDocument`, not `LoadedEntity` (Viktor #8): it is what
/// `load_entity` returns, i.e. an opened document, while the engine's
/// `LoadedEntity` is a migration outcome — and both names were in scope in this
/// crate meaning different things.
#[test]
fn the_opened_document_dto_carries_the_migration_report_to_the_frontend() {
    use arm_app::commands::OpenedDocument;

    let document = OpenedDocument {
        path: "/home/u/gerhard.armc".to_owned(),
        entity: sample_entity(),
        migrated_aging_characteristics: vec![arm_rules::Characteristic::Com],
        unresolved_catalogued_parameters: Vec::new(),
        migrated_catalogued_parameters: Vec::new(),
    };
    let json: serde_json::Value = serde_json::to_value(&document).unwrap();

    assert_eq!(json["path"], "/home/u/gerhard.armc");
    assert!(json["entity"].is_object(), "got: {json}");
    assert_eq!(
        json["migrated_aging_characteristics"],
        serde_json::json!(["com"]),
        "the frontend needs the Characteristics themselves, never an English \
         sentence: it resolves its own Fluent notice from them"
    );
}

/// CV4b (design § 5.5, plan-review gap): an unresolved catalogued parameter —
/// a value that did not spell out any catalogue entry's name in either locale
/// and so stayed free text — must reach the frontend exactly like a migrated
/// aging Characteristic does, not stay an engine-only report. A previously
/// working-by-luck authorization or restricted-pool funding can silently stop
/// applying once § 4 rule 1's Literal-only matching is enforced, so the player
/// must be told, not left to discover it.
///
/// End-to-end through the real load path (`load_entity_from_path`), not a
/// hand-built `LoadedEntity`: this is what actually proves an unknown language
/// in a real save reaches the wire DTO. The Ability travels as its own id, not
/// a sentence — the frontend resolves the localized name, exactly as
/// `migrated_aging_characteristics` documents for its own case.
#[test]
fn opened_document_carries_unresolved_catalogued_parameters() {
    use arm_app::commands::opened_document;

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("unknown-language.json");
    fs::write(
        &path,
        format!(
            r#"{{
              "schema_version": 18,
              "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
              "entity_kind": "character",
              "type_id": "companion",
              "ability_scores": [
                {{ "ability": "ability.dead_language", "score": 1, "parameter": {{ "text": "Klingon" }} }}
              ]
            }}"#
        ),
    )
    .unwrap();

    let (ruleset, names) = shipped_ruleset_and_names();
    let loaded = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap();
    assert_eq!(
        loaded.unresolved_catalogued_parameters,
        vec![(Id::new("ability.dead_language"), "Klingon".to_string())],
        "sanity: the engine's own report must already carry it"
    );

    // Through the SAME conversion `load_entity`'s command body uses, not a
    // hand-built literal — so this proves the wiring, not just the type.
    let document = opened_document(path.to_string_lossy().into_owned(), loaded);
    let json: serde_json::Value = serde_json::to_value(&document).unwrap();

    assert_eq!(
        json["unresolved_catalogued_parameters"],
        serde_json::json!([{ "ability": "ability.dead_language", "text": "Klingon" }]),
        "the frontend needs the Ability id and the typed text themselves, \
         never an English sentence and never a raw id rendered as one: got {json}"
    );
}

/// CV4b's positive counterpart (design § 5.5): a value the fold DID recognize
/// as a catalogue entry's name is reported too — "what you typed is now
/// linked to its catalogue entry" — not only the failure case. Same
/// through-the-real-conversion shape as the test above.
#[test]
fn opened_document_carries_migrated_catalogued_parameters() {
    use arm_app::commands::opened_document;

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("recognized-language.json");
    fs::write(
        &path,
        format!(
            r#"{{
              "schema_version": 18,
              "ruleset": {{ "id": "{RULESET_ID}", "version": "{RULESET_VERSION}" }},
              "entity_kind": "character",
              "type_id": "companion",
              "ability_scores": [
                {{ "ability": "ability.dead_language", "score": 1, "parameter": {{ "text": "latin" }} }}
              ]
            }}"#
        ),
    )
    .unwrap();

    let (ruleset, names) = shipped_ruleset_and_names();
    let loaded = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap();
    assert_eq!(
        loaded.migrated_catalogued_parameters,
        vec![(
            Id::new("ability.dead_language"),
            "latin".to_string(),
            Id::new("language.latin")
        )],
        "sanity: the engine's own report must already carry it"
    );

    let document = opened_document(path.to_string_lossy().into_owned(), loaded);
    let json: serde_json::Value = serde_json::to_value(&document).unwrap();

    assert_eq!(
        json["migrated_catalogued_parameters"],
        serde_json::json!([{
            "ability": "ability.dead_language",
            "text": "latin",
            "resolved": "language.latin"
        }]),
        "the frontend needs the Ability id, the typed text AND the resolved \
         catalogue id themselves, never an English sentence: got {json}"
    );
}

/// Gerda #5 (MAJOR): the path the frontend adopts as `currentPath` must be the
/// real path or an error, never a lookalike. A filename on Linux is an arbitrary
/// byte string, and `to_string_lossy` substitutes U+FFFD for every byte that is
/// not UTF-8 — so a save into such a directory used to hand the frontend a path
/// that does not exist, which every later plain Save would then write to. The
/// user edits for an hour, presses Ctrl+S, and the file they believe they
/// updated still holds the old version; the dirty flag is cleared, so the
/// close guard does not warn them either.
///
/// Refusing is the honest answer: the user is told, rather than silently
/// misdirected.
#[test]
fn a_utf8_path_is_reported_verbatim() {
    assert_eq!(
        path_text(Path::new("/home/u/gerhard.armc")).unwrap(),
        "/home/u/gerhard.armc"
    );
}

/// The other half. Only reachable on a platform whose paths are bytes rather
/// than (well-formed or not) UTF-16, which is why it is `cfg(unix)` — the check
/// itself is unconditional.
#[cfg(unix)]
#[test]
fn a_path_that_is_not_utf8_is_refused_rather_than_mangled() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    // 0x80 is a continuation byte with nothing to continue: a valid file name,
    // not valid UTF-8.
    let path = PathBuf::from(OsString::from_vec(b"gerhard-\x80.armc".to_vec()));

    let err = path_text(&path).unwrap_err();
    let AppError::Io { message } = err else {
        panic!("a path that cannot be reported must be an Io failure, got {err:?}");
    };
    assert!(
        message.contains("gerhard-"),
        "the failure must name the path it is about, got: {message}"
    );
}

#[test]
fn sample_save_loads_with_defaulted_aging_warping_annotations() {
    // A shipped example save carries none of the new annotation fields; loading it
    // must fill them with their empty/None defaults (additive backward compat).
    let path = repo_root().join("examples/companion_sample.json");
    let (ruleset, names) = shipped_ruleset_and_names();
    let entity = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap()
    .entity;
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
        ArtScore::new(Id::new("art.creo"), 3),  // 6 xp
        ArtScore::new(Id::new("art.ignem"), 5), // 15 xp
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
    let names = load_catalogue_names_from_dir(&rules_dir(), &ruleset).unwrap();
    let reloaded = load_entity_from_path(
        &path,
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap()
    .entity;
    assert_eq!(reloaded.schema_version, 21);
    assert_eq!(reloaded.art_scores, entity.art_scores);
}

#[test]
fn effective_scores_surface_aged_characteristic_and_drop_count() {
    // ArMDE:16613 worked example: Communication +2 with 3 aging points
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

/// "a character over the age of 35 must make aging rolls … before the game begins"
/// (ArMDE:2232), and aging starts "the Winter after they turn 35"
/// (`ArMDE:16565`) — so a character of 40 owes one roll a year from 36 through 40, five
/// in all. The whole read-out is a pure function of the character and the rules:
/// the die is the player's and never reaches the entity, which is why the
/// die-independent half rides on the always-recomputed payload.
#[test]
fn effective_scores_report_the_aging_rolls_a_character_of_forty_owes() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();

    let aging = effective_scores_loaded(&entity, &ruleset)
        .aging
        .expect("the shipped ruleset carries aging rules");

    assert_eq!(
        aging.begins_after_age, 35,
        "the book's own number (ArMDE:16565)"
    );
    assert_eq!(
        aging.first_roll_age, 36,
        "the Winter after 35 falls in year 36"
    );
    assert_eq!(aging.rolls_owed, 5, "36, 37, 38, 39 and 40");
    assert_eq!(aging.rolls_recorded, 0, "an empty log has settled none");
    assert_eq!(aging.schedule.len(), 5);
    assert_eq!(aging.schedule.first().map(|year| year.age), Some(36));
    assert_eq!(aging.schedule.last().map(|year| year.age), Some(40));
    assert!(
        aging.schedule.iter().all(|year| !year.recorded),
        "nothing is recorded yet: {:?}",
        aging.schedule
    );

    // "age/10 (round up)" (`ArMDE:16567`) at the ACTUAL age (`ArMDE:16577`).
    assert_eq!(aging.age_modifier, 4);
    // No ritual, so no bonus and no `ArMDE:16575` clamp standing over this character.
    assert_eq!(aging.longevity_modifier, 0);
    assert!(!aging.longevity_clamp_active);
    // The whole non-die half, so the UI adds only the number the player typed — and
    // it is exactly the sum of the terms the read-out names, the Virtue/Flaw one
    // included. Naming three of four is what made the displayed formula contradict
    // itself (guided-creation-review-2026-08 #22).
    assert_eq!(
        aging.fixed_total,
        aging.age_modifier - aging.living_conditions_modifier - aging.longevity_modifier
            + aging.trait_modifier
    );
}

/// A character of 40 rolling a 10: `10 + ⌈40/10⌉ = 14`, which the shipped table
/// answers with "1 Aging Point in Qik" (ArMDE:16603). One less on the die
/// lands on 13 — "Gain sufficient Aging Points … to reach the next level in
/// Decrepitude, and Crisis" (`ArMDE:16602`) — so the preview must say a Crisis follows.
///
/// The die is the player's, typed in and never stored, which is why this is a
/// command rather than a field of the character.
#[test]
fn aging_preview_totals_the_typed_die_and_names_the_outcome() {
    use arm_rules::{AgingPointTarget, Characteristic};
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.living_conditions.clear();
    entity.longevity_ritual = None;

    let AgingProjection::Previewed { total, outcome, .. } =
        arm_app::ruleset_io::aging_preview_loaded(
            &entity,
            &ruleset,
            40,
            10,
            &BTreeMap::new(),
            None,
        )
    else {
        panic!("the shipped ruleset carries aging rules");
    };
    assert_eq!(total.die, 10);
    assert_eq!(total.age_modifier, 4);
    assert_eq!(total.total, 14);
    assert!(!total.capped_by_longevity, "no ritual, so no :16575 clamp");
    assert_eq!(outcome.total, 14);
    assert!(
        outcome.apparent_age_increases,
        "14 is well over the 3 of :16600"
    );
    assert_eq!(
        outcome
            .awards
            .iter()
            .map(|award| award.target.clone())
            .collect::<Vec<_>>(),
        vec![AgingPointTarget::Named(Characteristic::Qik)],
        "row 14 names Quickness and nothing else"
    );
    assert!(!outcome.crisis);

    let AgingProjection::Previewed { total, outcome, .. } =
        arm_app::ruleset_io::aging_preview_loaded(&entity, &ruleset, 40, 9, &BTreeMap::new(), None)
    else {
        panic!("the shipped ruleset carries aging rules");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis, "13 is the first Crisis row (ArMDE:16602)");
}

/// A mistyped die has to be recoverable — a magus of 60 owes 25 rolls (`ArMDE:2232`) —
/// so applying a year and reverting it must leave the character it started from,
/// byte for byte, not merely something equivalent.
#[test]
fn an_applied_aging_year_reverts_to_the_character_it_started_from() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.normalize();
    let before = serde_json::to_string(&entity).unwrap();

    // Row 14 names its own Characteristic, so the player places nothing.
    let AgingApplication::Applied {
        entity: applied,
        total,
        outcome,
        ..
    } = arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 10, &BTreeMap::new(), None)
    else {
        panic!("a year the character owes and has not rolled applies");
    };
    assert_eq!(
        total.total, 14,
        "the applied roll comes back with the entity"
    );
    assert!(!outcome.crisis);
    assert_ne!(
        serde_json::to_string(&*applied).unwrap(),
        before,
        "the year was written"
    );
    assert_eq!(applied.aging_log.len(), 1);

    let AgingReversion::Reverted { entity: reverted } =
        arm_app::ruleset_io::aging_revert_loaded(&applied, &ruleset, 40)
    else {
        panic!("the year just applied is recorded, so it reverts");
    };
    assert_eq!(serde_json::to_string(&*reverted).unwrap(), before);

    // A year no entry records is a refusal the player is told about, never a
    // silent no-op — and it crosses the boundary as a localizable finding.
    let AgingReversion::Rejected { issues } =
        arm_app::ruleset_io::aging_revert_loaded(&entity, &ruleset, 40)
    else {
        panic!("nothing is recorded for that year");
    };
    assert_eq!(
        issues
            .iter()
            .map(|issue| issue.code.as_str())
            .collect::<Vec<_>>(),
        vec![arm_rules::ValidationIssue::CODE_AGING_YEAR_NOT_RECORDED]
    );
}

/// The calculator has to show the Crisis BEFORE Apply, and it has to show the one
/// Apply will write — which is not the one a bare `crisis_preview` of the
/// character standing in front of you answers.
///
/// > **Crisis:** Increase the character's Decrepitude first, and then roll on the
/// > Crisis Table. (`ArMDE:16619`)
///
/// The five Aging Points row 13 awards ARE that increase, so the CRISIS TOTAL adds
/// the Decrepitude the year itself raised. Read off the character before the year
/// is applied, the same die answers 14 and the same player would then watch the
/// log record 15. So the preview resolves the year in memory and throws the
/// character away: the preview and the apply send one identical request, and the
/// reading cannot disagree with what lands.
#[test]
fn aging_preview_reads_the_crisis_off_the_year_it_would_apply() {
    use arm_rules::Characteristic;
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.living_conditions.clear();
    entity.longevity_ritual = None;

    let distribution = BTreeMap::from([(Characteristic::Sta, 5)]);
    let AgingProjection::Previewed {
        total,
        outcome,
        crisis,
    } = arm_app::ruleset_io::aging_preview_loaded(
        &entity,
        &ruleset,
        40,
        9,
        &distribution,
        Some(10),
    )
    else {
        panic!("the shipped ruleset carries aging rules");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis, "13 is the first Crisis row (ArMDE:16602)");
    let previewed = crisis.expect("a Crisis with a die rolled reads whole");
    assert_eq!(
        previewed.total.decrepitude_score, 1,
        "the increase :16619 puts first"
    );
    assert_eq!(previewed.total.total, 15);
    assert_eq!(previewed.row, Id::new("crisis.minor_illness"));

    // THE POINT: what the player is shown is what the year writes.
    let AgingApplication::Applied {
        crisis: applied, ..
    } = arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 9, &distribution, Some(10))
    else {
        panic!("the same request applies");
    };
    assert_eq!(applied.as_deref(), Some(&*previewed));

    // And the reading the un-applied character would give is the wrong one, which
    // is what makes resolving the year first load-bearing rather than tidy.
    assert_eq!(
        arm_rules::crisis_preview(&entity, &ruleset, 40, 10)
            .expect("the shipped ruleset carries a Crisis Table")
            .total
            .total,
        14,
        "one Decrepitude short, because this year's points have not landed"
    );

    // Before the points are placed there is no honest Crisis to read, but the
    // total and the outcome still stand: the player is told a Crisis follows and
    // what to place, which is the order :16619 asks for.
    let AgingProjection::Previewed {
        total,
        outcome,
        crisis,
    } = arm_app::ruleset_io::aging_preview_loaded(
        &entity,
        &ruleset,
        40,
        9,
        &BTreeMap::new(),
        Some(10),
    )
    else {
        panic!("an unplaced distribution is not a reason to withhold the total");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis);
    assert_eq!(crisis, None);
}

/// The Crisis the player rolled has to reach the character, and the two things
/// only the applied year can say have to reach the player.
///
/// A companion of 40 rolling a 9 totals `9 + ⌈40/10⌉ = 13`, which is "Gain
/// sufficient Aging Points (in any Characteristics) to reach the next level in
/// Decrepitude, and Crisis" (`ArMDE:16602`) — five points from nothing. The Crisis is
/// then read off the character those five points already made (`ArMDE:16619`), so a
/// Simple Die of 10 totals `10 + 4 + 1 = 15`: the minor illness of `ArMDE:16628`,
/// survivable on a Stamina stress roll against an Ease Factor of 3 or a CrCo20.
///
/// Until the die crossed this edge every Crisis the shipped app recorded was owed
/// and unrolled, whatever the player had thrown.
#[test]
fn an_applied_crisis_year_comes_back_with_the_crisis_and_the_ritual_it_spent() {
    use arm_rules::{AgingNote, Characteristic, CrisisOutcome, CrisisSeverity, LongevityRitual};
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.living_conditions.clear();
    // A ritual with no bonus leaves the AGING TOTAL alone, so the year still lands
    // on 13 — and it is still a ritual the Crisis spends (`ArMDE:16573`).
    entity.longevity_ritual = Some(LongevityRitual {
        source: arm_rules::LongevitySource::External,
        bonus: Some(0),
        focus: String::new(),
    });

    let distribution = BTreeMap::from([(Characteristic::Sta, 5)]);
    let AgingApplication::Applied {
        total,
        outcome,
        crisis,
        notes,
        ..
    } = arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 9, &distribution, Some(10))
    else {
        panic!("a year the character owes and has not rolled applies");
    };
    assert_eq!(total.total, 13);
    assert!(outcome.crisis, "13 is the first Crisis row (ArMDE:16602)");

    let crisis = crisis.expect("a Crisis with a die rolled comes back resolved");
    assert_eq!(
        crisis.total.die, 10,
        "the player's Simple Die (ArMDE:16621)"
    );
    assert_eq!(crisis.total.age_modifier, 4);
    assert_eq!(
        crisis.total.decrepitude_score, 1,
        "the five points this very year awarded, counted first (ArMDE:16619)"
    );
    assert_eq!(crisis.total.total, 15);
    assert_eq!(crisis.row, Id::new("crisis.minor_illness"));
    assert_eq!(
        crisis.outcome,
        CrisisOutcome::Illness {
            severity: CrisisSeverity::Minor,
            ease_factor: Some(3),
            ritual_level: 20,
        }
    );
    let survival = crisis
        .survival
        .expect("an illness is survivable, so it has a read-out");
    assert_eq!(survival.ease_factor, Some(3));
    assert_eq!(survival.ritual_level, 20);
    assert_eq!(
        survival.allowances.len(),
        1,
        "one doctor only (ArMDE:16634)"
    );

    // "its power is spent, and the focal ritual must be performed again"
    // (`ArMDE:16573`) — reported, because the entity keeps the stored choice.
    assert_eq!(notes, vec![AgingNote::LongevityRitualSpent]);

    // A year with no Crisis die is still written, with the Crisis owed and
    // unrolled — the aging roll happened whether or not the second die was thrown.
    let AgingApplication::Applied { crisis: none, .. } =
        arm_app::ruleset_io::aging_apply_loaded(&entity, &ruleset, 40, 9, &distribution, None)
    else {
        panic!("an unrolled Crisis is a legitimate state, not a refusal");
    };
    assert_eq!(none, None);
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
fn effective_scores_surface_virtue_flaw_balance() {
    // The balance bar must read the engine's own spent Virtue/Flaw points
    // (`validation::compute_balance` — the same function `export.rs`'s Markdown
    // export and the over-budget/unbalanced-Virtues validation issues already
    // read) rather than re-deriving them a third time in TypeScript. Audit
    // finding G1 (round 4): the same defect class already fixed once for
    // `characteristic_points_used` (VA1/GF1/GD4).
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    let bare = effective_scores_loaded(&companion, &ruleset);
    assert_eq!(
        (bare.virtue_points, bare.flaw_points),
        (0, 0),
        "no selections means no spent points"
    );

    // Self-Confident is a shipped Minor Virtue (1 point); Infamous is a shipped
    // Minor Flaw (1 point) — rules/core/virtues_flaws.json.
    companion
        .selections
        .push(Selection::new(Id::new("virtue.self_confident")));
    companion
        .selections
        .push(Selection::new(Id::new("flaw.infamous")));
    let effective = effective_scores_loaded(&companion, &ruleset);
    assert_eq!(effective.virtue_points, 1, "one minor virtue = 1 point");
    assert_eq!(effective.flaw_points, 1, "one minor flaw = 1 point");
}

#[test]
fn load_entity_from_missing_path_is_io_error() {
    let (ruleset, names) = shipped_ruleset_and_names();
    let err = load_entity_from_path(
        &repo_root().join("does/not/exist.json"),
        arm_rules::DEFAULT_SAGA_YEAR,
        Some(&ruleset),
        Some(&names),
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

/// CV4's new guard (design § 5.6): opening a save before any ruleset has loaded
/// fails with the same `AppError::NotLoaded` every other ruleset-dependent
/// command reports, rather than a panic or a confusing deserialize error. The
/// absent ruleset is an argument here (mirroring `export_markdown_to_path`'s own
/// `Option` gate), which is what makes the case reachable without a Tauri
/// runtime — the app's own `commands::load_entity` shim reads this same
/// `Option` from `AppState.ruleset`, which starts `None` until `load_ruleset`
/// succeeds.
///
/// A freshly-constructed `Entity` (no ability scores) is used here rather
/// than `sample_entity()`, so this test fails for the guard's own reason and
/// not for an unrelated ability-parameter parse issue.
#[test]
fn load_entity_without_a_loaded_ruleset_is_not_loaded_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("character.json");
    let entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    save_entity_to_path(&entity, &path).unwrap();

    let err = load_entity_from_path(&path, arm_rules::DEFAULT_SAGA_YEAR, None, None).unwrap_err();
    assert!(matches!(err, AppError::NotLoaded), "got {err:?}");
}

#[test]
fn effective_scores_surface_a_reputation_grants_max_score() {
    // Outsider (D11/Q5): "a bad Reputation of level 1 to 3" (ArMDE:6554) is the
    // one grant in the whole catalogue that states a range, so the UI needs the
    // upper bound to let the player pick within it.
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    companion
        .selections
        .push(Selection::new(Id::new("flaw.outsider_major")));
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert_eq!(grants.len(), 1, "{grants:?}");
    assert_eq!(grants[0].score, 1);
    assert_eq!(grants[0].max_score, Some(3));

    // Every other granter leaves it absent — exact, not a range.
    companion.selections = vec![Selection::new(Id::new("flaw.infamous"))];
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert_eq!(grants[0].max_score, None, "{grants:?}");
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

    // Infamous surfaces a wildcard reputation slot naming the Flaw that opened
    // it. F-450 (D11/Q5): the passage states no audience ("a level 4 bad
    // Reputation", ArMDE:6312), so the shipped `kind: "local"` was a hardcoded
    // invention — its twin `virtue.famous` already ships the wildcard for the
    // same shape.
    companion
        .selections
        .push(Selection::new(Id::new("flaw.infamous")));
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert_eq!(grants.len(), 1, "one grant per granting V/F: {grants:?}");
    assert_eq!(grants[0].kind, None, "Infamous fixes no Reputation type");
    assert_eq!(grants[0].score, 4);
    assert_eq!(grants[0].source, Id::new("flaw.infamous"));

    // Famous leaves the type to the player. That is ONE slot the player types
    // themselves, not one slot per Reputation type — `validate_reputations`
    // allows exactly one wildcard Reputation, so flattening it into four
    // offered four ways to overspend a single legal slot.
    companion
        .selections
        .push(Selection::new(Id::new("virtue.famous")));
    let grants = effective_scores_loaded(&companion, &ruleset).reputation_grants;
    assert_eq!(grants.len(), 2, "one grant per granting V/F: {grants:?}");
    let wildcard = grants
        .iter()
        .find(|g| g.source == Id::new("virtue.famous"))
        .expect("Famous surfaces a grant");
    assert_eq!(wildcard.kind, None, "Famous fixes no Reputation type");
    assert_eq!(wildcard.score, 4);

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

/// The Abilities view has to show a magus which Hermetic minimums it still owes, so
/// the checklist crosses the boundary with the effective scores rather than being
/// re-derived in JS from the rules file. Against the **real shipped ruleset**: the
/// three minimums of ArMDE:2437 plus the four recommendations of
/// `ArMDE:2451-2461`, seven rows.
///
/// The same payload carries `xp_general_pool` — the pool the solve funds from, which
/// `xp_general_used` (a max-flow value) cannot be divided by.
#[test]
fn effective_scores_surface_the_magus_minimum_ability_checklist() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    let checklist = effective_scores_loaded(&magus, &ruleset).magus_minimum_abilities;
    assert_eq!(
        checklist.len(),
        7,
        "3 required + 4 recommended: {checklist:?}"
    );
    assert!(
        checklist.iter().all(|row| !row.met),
        "a fresh magus owes all of them: {checklist:?}"
    );

    // Buying Magic Theory 1 flips exactly one row. Magic Theory, not Parma Magica:
    // Parma 1 is demanded by BOTH lists (`ArMDE:2437` and `ArMDE:2459`), so buying it correctly
    // flips two, while the recommended Magic Theory threshold is 3 — which makes this
    // the only clean single-flip probe.
    magus.ability_scores = vec![arm_rules::AbilityScore::new(
        Id::new("ability.magic_theory"),
        1,
    )];
    let checklist = effective_scores_loaded(&magus, &ruleset).magus_minimum_abilities;
    assert_eq!(checklist.iter().filter(|row| row.met).count(), 1);
    let met = checklist
        .iter()
        .find(|row| row.met)
        .expect("one row is met now");
    assert_eq!(met.ability, Id::new("ability.magic_theory"));
    assert_eq!(met.min_score, 1);
    assert_eq!(met.score, 1);

    // The magus's general pool is its apprenticeship experience once it is built
    // through its life stages, and the typed pool otherwise.
    magus.xp_pool = 240;
    let scores = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(scores.xp_general_pool, 240);
    assert_eq!(scores.xp_general_bonus, 0);

    // Skilled Parens raises that pool by 60 — "an additional 60 experience points …
    // during apprenticeship" (ArMDE:4966). The bar shows the typed 240 as the
    // editable total, so the bonus has to reach it as a figure of its own; without it
    // the pool and the field it is entered in differ with nothing to explain the gap.
    magus.selections = vec![arm_rules::Selection::new(Id::new("virtue.skilled_parens"))];
    let scores = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(scores.xp_general_pool, 300);
    assert_eq!(scores.xp_general_bonus, 60);
    magus.selections.clear();

    // A companion is not admitted to the Order, so it gets no checklist at all —
    // exactly like the magus-only spell-level caps.
    let companion = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    assert!(
        effective_scores_loaded(&companion, &ruleset)
            .magus_minimum_abilities
            .is_empty()
    );
}

/// The XP bar must be able to show an OVERSPENT pool as a negative "Available".
/// `xp_general_used` cannot express that: it is a max-flow value capped by the pool
/// itself, so `pool - general_used` never goes below zero. The overspend lives in
/// `total_demand - max_flow` (the same figure `not_enough_xp` reports as its
/// shortfall), so `max_flow` has to reach the frontend.
#[test]
fn effective_scores_surface_max_flow_so_the_ui_can_show_an_overspent_pool() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // A 10-xp pool buying an Ability score of 2 (15 xp on the advancement table):
    // a 5-xp overspend.
    entity.xp_pool = 10;
    entity.ability_scores = vec![arm_rules::AbilityScore::new(
        Id::new("ability.awareness"),
        2,
    )];

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(effective.xp_total_demand, 15, "score 2 costs 15 xp");
    // The general pool is drained but cannot cover the demand...
    assert_eq!(effective.xp_general_used, 10);
    // ...so the fundable total stops at the pool, and the 5-xp shortfall is exactly
    // total_demand - max_flow — the negative the bar renders as "Available: -5".
    assert_eq!(effective.xp_max_flow, 10);
    assert_eq!(effective.xp_total_demand - effective.xp_max_flow, 5);

    // A pool that covers the spend reports no shortfall.
    entity.xp_pool = 50;
    let legal = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(legal.xp_total_demand, legal.xp_max_flow);
}

/// The Characteristic point cost is surfaced by the engine rather than recomputed
/// in the frontend. `ui/src/lib/derive.ts` carried its own copy of the point-buy
/// table (audit findings VA1/GF1/GD4, raised by three separate reviewers), which
/// could drift from `CharacteristicRules::total_cost` silently. The payload now
/// carries the authoritative figure so the UI has nothing to recompute.
///
/// Uses the rulebook's own worked example so the assertion is anchored to the
/// source, not to whatever the code happens to return:
/// ArMDE:2358 — Int +3 (6), Per +1 (1),
/// Pre -3 (-6), Com -1 (-1), Sta 0 (0), Qik +2 (3), Str +2 (3), Dex +1 (1) => 7.
#[test]
fn effective_scores_surface_the_characteristic_points_used() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    use arm_rules::Characteristic;
    entity.characteristics = std::collections::BTreeMap::from([
        (Characteristic::Int, 3),
        (Characteristic::Per, 1),
        (Characteristic::Pre, -3),
        (Characteristic::Com, -1),
        (Characteristic::Sta, 0),
        (Characteristic::Qik, 2),
        (Characteristic::Str, 2),
        (Characteristic::Dex, 1),
    ]);

    let effective = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(
        effective.characteristic_points_used, 7,
        "the rulebook's worked example nets gains against spends"
    );
}

/// The V/F spell-levels contribution is surfaced as its OWN payload figure, not
/// only folded into the effective budget, so the spell-levels bar can show the
/// editable base beside a labelled bonus — the way the XP bar lists extra pools
/// beside the general one.
#[test]
fn effective_scores_surface_the_spell_levels_bonus_separately() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // No spell-levels V/F: the base profile budget with nothing added.
    let plain = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(plain.spell_levels_profile_base, 120);
    assert_eq!(plain.spell_levels_budget, 120);
    assert_eq!(plain.spell_levels_bonus, 0);

    // Skilled Parens (+30) reports the bonus on its own, and the budget still
    // carries the total, so base + bonus == budget.
    magus
        .selections
        .push(arm_rules::Selection::new(Id::new("virtue.skilled_parens")));
    let boosted = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(boosted.spell_levels_bonus, 30);
    assert_eq!(boosted.spell_levels_budget, 150);

    // The bonus is independent of the per-character base override: overriding the
    // base to 80 keeps the +30 reportable and the budget at 110.
    magus.spell_levels_override = Some(80);
    let overridden = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(overridden.spell_levels_bonus, 30);
    assert_eq!(overridden.spell_levels_budget, 110);
}

/// The levels of spells a magus took out of its post-Gauntlet points reach the
/// frontend as a figure of their own, beside the profile base and the V/F bonus, so
/// the spell-levels bar can label all three parts of the budget rather than folding
/// them into an unexplained total.
///
/// They are additive, not a second budget: the profile's 120 are apprenticeship's
/// (`ArMDE:2435`), while these are the player's chosen slice of the fungible "30 points
/// per year" (`ArMDE:2471`).
#[test]
fn effective_scores_surface_the_post_gauntlet_spell_levels_separately() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    // A magus of 60 gauntleted at 25: 35 years at 30 points each, 300 of them taken
    // as levels of spells.
    magus.age = Some(60);
    // The funding mode is stored since schema 16, so a plan needs it to be live.
    magus.ability_funding = arm_rules::AbilityFunding::LifeStages;
    magus.life_stages = Some(arm_rules::LifeStagePlan {
        gauntlet_age: Some(25),
        post_gauntlet_spell_levels: 300,
        ..arm_rules::LifeStagePlan::default()
    });
    let experienced = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(experienced.spell_levels_life_stage, 300);
    assert_eq!(experienced.spell_levels_profile_base, 120);
    assert_eq!(experienced.spell_levels_bonus, 0);
    // base + bonus + life stage == budget.
    assert_eq!(experienced.spell_levels_budget, 420);

    // A magus standing at its Gauntlet has lived no year past it, so it takes no
    // levels out of them and its budget is the profile's 120 — the pre-6b5 number.
    magus.age = Some(25);
    magus.life_stages = Some(arm_rules::LifeStagePlan::default());
    let fresh = effective_scores_loaded(&magus, &ruleset);
    assert_eq!(fresh.spell_levels_life_stage, 0);
    assert_eq!(fresh.spell_levels_budget, 120);
}

/// A companion built through its life stages, speaking `native_language` — the
/// character a Sample Childhood package is applied to (childhood exists only in
/// life-stage mode, and the package's native-language entry takes the plan's
/// language).
fn life_stage_companion(native_language: &str) -> Entity {
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    entity.age = Some(25);
    // The funding mode is stored since schema 16, so a plan needs it to be live.
    entity.ability_funding = arm_rules::AbilityFunding::LifeStages;
    entity.life_stages = Some(arm_rules::LifeStagePlan {
        native_language: Some(native_language.to_string()),
        ..arm_rules::LifeStagePlan::default()
    });
    entity
}

/// The entity's Ability rows as `(ability, parameter, score)`, in the canonical
/// order the applied entity comes back normalized into. Every parameter this
/// module's fixtures produce is free text (`AbilityParameterValue::Text`, e.g.
/// a native language) — a `Catalogued`/`Linked` value has no bare-string form
/// and is deliberately not matched here.
fn ability_rows(entity: &Entity) -> Vec<(&str, Option<&str>, u8)> {
    entity
        .ability_scores
        .iter()
        .map(|row| {
            let parameter = match &row.parameter {
                Some(arm_rules::AbilityParameterValue::Text { text }) => Some(text.as_str()),
                Some(_) => panic!("this test module's fixtures only ever produce free text"),
                None => None,
            };
            (row.ability.as_str(), parameter, row.score)
        })
        .collect()
}

/// Applying a shipped package through the command path writes its entries as
/// ordinary bought Ability rows, the native-language one under the language the
/// plan names — "Athletic Childhood: Athletics 2, Brawl 2, Native Language 5,
/// Swim 2" (ArMDE:2384).
#[test]
fn apply_childhood_package_writes_the_package_rows() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let outcome = apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.athletic"),
        &BTreeMap::new(),
        &ruleset,
    );

    let ChildhoodApplication::Applied { entity } = outcome else {
        panic!("Athletic Childhood asks the player for nothing, so it applies: {outcome:?}");
    };
    assert_eq!(
        ability_rows(&entity),
        vec![
            ("ability.athletics", None, 2),
            ("ability.brawl", None, 2),
            ("ability.living_language", Some("German"), 5),
            ("ability.swim", None, 2),
        ]
    );
    assert_eq!(
        entity
            .life_stages
            .and_then(|plan| plan.childhood_package)
            .as_ref(),
        Some(&Id::new("childhood.athletic")),
        "the package taken is recorded on the plan"
    );
}

/// A slot the player never answered is reported as an ordinary
/// `ValidationIssue`, carrying the Ability, its parameter key, and the slot the
/// UI highlights — Traveling asks for two Area Lores plus a second language
/// (ArMDE:2388), and only two of the three arrive here.
#[test]
fn apply_childhood_package_rejects_an_unfilled_slot() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut slot_values = BTreeMap::new();
    slot_values.insert("area_a".to_string(), "Rhine".to_string());
    slot_values.insert("language".to_string(), "Italian".to_string());

    let outcome = apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.traveling"),
        &slot_values,
        &ruleset,
    );

    let ChildhoodApplication::Rejected { issues } = outcome else {
        panic!("area_b was never answered, so nothing may be written: {outcome:?}");
    };
    assert_eq!(
        issues.len(),
        1,
        "one unanswered slot, one issue: {issues:?}"
    );
    let issue = &issues[0];
    assert_eq!(issue.code, "childhood_slot_unfilled");
    assert_eq!(
        issue.args.get("ability").map(String::as_str),
        Some("ability.area_lore")
    );
    assert_eq!(issue.args.get("key").map(String::as_str), Some("area"));
    assert_eq!(issue.args.get("slot").map(String::as_str), Some("area_b"));
}

/// The outcome crosses IPC as a tagged union, because the frontend switches on
/// `status` — and rejections travel as issue codes, never as English prose.
#[test]
fn apply_childhood_package_serializes_as_a_tagged_union() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let applied = serde_json::to_value(apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.athletic"),
        &BTreeMap::new(),
        &ruleset,
    ))
    .unwrap();
    assert_eq!(applied["status"], "applied");
    assert!(
        applied["entity"].is_object(),
        "an applied outcome carries the entity: {applied}"
    );

    // A package id the ruleset does not ship is a rejection, not a silent no-op.
    let rejected = serde_json::to_value(apply_childhood_package_loaded(
        &life_stage_companion("German"),
        &Id::new("childhood.nonesuch"),
        &BTreeMap::new(),
        &ruleset,
    ))
    .unwrap();
    assert_eq!(rejected["status"], "rejected");
    assert_eq!(rejected["issues"][0]["code"], "childhood_package_unknown");
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
    // Demonic Blood (Infernal Might 5 + 30 power levels, RoP:I:4120-4122) and
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

/// The Focus Power pool crosses IPC as its own pair of numbers, because the
/// panel draws its own bar for it: 25 points per copy of the Virtue
/// (`ArMDE:3899`), spent 2 per level of effect and 1 per point of Penetration.
/// A character without the Virtue reports 0/0 and is otherwise untouched.
#[test]
fn effective_scores_carry_the_focus_power_pool() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );

    let plain = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(plain.focus_points_budget, 0);
    assert_eq!(plain.focus_points_used, 0);

    entity
        .selections
        .push(Selection::new(Id::new("virtue.focus_power")));
    entity.focus_powers = vec![arm_rules::FocusPower {
        name: "Wolves".into(),
        max_level: 10,
        penetration: 5,
    }];

    let scores = effective_scores_loaded(&entity, &ruleset);
    assert_eq!(scores.focus_points_budget, 25);
    assert_eq!(scores.focus_points_used, 25); // 2 × 10 + 5
    // The level-denominated budget is untouched by any of it.
    assert_eq!(scores.power_levels_used, 0);
}

/// A Devil Child, mid-build against the shipped rules.
fn devil_child(bought_demonic_might: usize, demonic_blood: bool) -> Entity {
    let mut entity = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("mythic_companion"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    entity.mythic_type = Some(Id::new("mythic_type.devil_child"));
    if demonic_blood {
        entity
            .selections
            .push(Selection::new(Id::new("virtue.demonic_blood")));
    }
    for _ in 0..bought_demonic_might {
        entity
            .selections
            .push(Selection::new(Id::new("virtue.demonic_might")));
    }
    entity
}

fn issue_codes(result: &arm_rules::ValidationResult) -> Vec<String> {
    result.issues.iter().map(|i| i.code.clone()).collect()
}

#[test]
fn a_granted_demonic_might_counts_toward_its_own_half_of_virtues_ratio() {
    // Demonic Blood (Major, 3) + three bought Demonic Might (1 each) is 3 capped
    // points of a 6-point Virtue total — exactly half, so bought copies alone are
    // clean. Devil Child's free Demonic Might makes it 4 of 7, which is over half.
    // Whether this warns is therefore exactly the question of whether a *granted*
    // copy counts (ArMDE:3665, :3673).
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let bought_only = devil_child(3, true);
    let result = validate_loaded(&bought_only, &ruleset, ValidationMode::Enforced);
    assert!(
        !issue_codes(&result).contains(&"too_large_share".to_string()),
        "three bought Demonic Might of six Virtue points is exactly half: {:?}",
        result.issues
    );

    let mut with_grant = bought_only;
    with_grant.mythic_choices.insert(
        "devil_child_free_minor".into(),
        Selection::new(Id::new("virtue.demonic_might")),
    );
    let result = validate_loaded(&with_grant, &ruleset, ValidationMode::Enforced);
    assert!(
        issue_codes(&result).contains(&"too_large_share".to_string()),
        "the granted Demonic Might must count toward the ratio: {:?}",
        result.issues
    );
}

#[test]
fn a_devil_child_without_demonic_blood_yet_warns_on_the_ratio() {
    // Intended, pinned behaviour, not an accident: before Demonic Blood is
    // bought, the granted Demonic Might is 1 point of a 1-point Virtue total
    // (Devil Child itself is Free and lifts no denominator), so the ratio warns.
    // This is transient mid-edit noise that warning severity exists to absorb,
    // and the missing prerequisite is reported alongside it.
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = devil_child(0, false);
    entity.mythic_choices.insert(
        "devil_child_free_minor".into(),
        Selection::new(Id::new("virtue.demonic_might")),
    );

    let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
    assert!(
        issue_codes(&result).contains(&"too_large_share".to_string()),
        "a granted Demonic Might with no other Virtue points is all of them: {:?}",
        result.issues
    );
}

/// The real UI strings for a language, keyed by Fluent message name — the map the
/// frontend hands to the Markdown export. Only argument-free single-line messages
/// are usable: the engine links no Fluent formatter, so a message interpolating
/// `{ $arg }` could never be resolved there (see `arm_rules::export`).
fn locale_labels(lang: &str) -> BTreeMap<String, String> {
    let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
    ftl.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split_once(" = "))
        .filter(|(_, value)| !value.contains('{'))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

/// Exporting writes a Markdown document whose chrome is the **real** English UI
/// wording, not a synthetic key map: the headings, column headers, and the
/// untitled-character title all come from `locales/en/main.ftl`. This is the
/// production path the export command takes; the engine's own golden test
/// deliberately feeds a `key -> key` map instead.
#[test]
fn exporting_writes_markdown_with_the_real_english_labels() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("companion.md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();
    let labels = locale_labels("en");

    export_markdown_to_path(&sample_entity(), Some(&localized), &labels, &path).unwrap();
    let doc = fs::read_to_string(&path).unwrap();

    // The sample has no name, so the title is the localized untitled marker, and
    // the subtitle names the character type.
    assert!(doc.starts_with("# Untitled character\n"), "got: {doc}");
    assert!(doc.contains("*Companion*"), "got: {doc}");
    // Sections use the shipped English headings.
    assert!(doc.contains("## Characteristics"), "got: {doc}");
    assert!(doc.contains("## Virtues & Flaws"), "got: {doc}");
    assert!(doc.contains("## Abilities"), "got: {doc}");
    // Values: a bought Characteristic, and Awareness 2 raised to an effective 4 by
    // the sample's Puissant Awareness.
    assert!(doc.contains("| Intelligence | +2 |"), "got: {doc}");
    // The Virtue row carries its type (the item's category) and its magnitude, both
    // localized — `category-general` is one of the families the frontend composes.
    assert!(
        doc.contains("| Puissant Awareness | General | Minor |"),
        "got: {doc}"
    );
    assert!(
        doc.contains("| Awareness | searching | 2 | 4 |"),
        "got: {doc}"
    );
    assert!(doc.contains("- **XP pool**: "), "got: {doc}");
    // No chrome key leaks through as its own label — that is the fallback the
    // formatter uses for a key the caller failed to supply.
    assert!(!doc.contains("export-col-"), "got: {doc}");
    assert!(!doc.contains("identity-name"), "got: {doc}");
}

/// A destination inside a directory that does not exist is a filesystem failure,
/// surfaced as `AppError::Io` (the frontend maps the variant to its own message).
#[test]
fn exporting_into_a_missing_directory_is_io_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("no-such-dir").join("companion.md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    let err = export_markdown_to_path(
        &sample_entity(),
        Some(&localized),
        &locale_labels("en"),
        &path,
    )
    .unwrap_err();
    assert!(matches!(err, AppError::Io { .. }), "got {err:?}");
}

/// A destination the user typed without an extension gets `.md`, the same way a
/// save without one gets `.armc` — this is the enforcement the export command
/// applies before writing.
#[test]
fn exporting_a_path_without_an_extension_writes_a_dot_md_file() {
    let tmp = tempfile::tempdir().unwrap();
    let target = ensure_extension(tmp.path().join("companion"), "md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    export_markdown_to_path(
        &sample_entity(),
        Some(&localized),
        &locale_labels("en"),
        &target,
    )
    .unwrap();

    assert_eq!(target, tmp.path().join("companion.md"));
    assert!(target.is_file(), "the .md file must exist");
}

/// Exporting before a ruleset is loaded cannot resolve a single display name, so
/// it fails with the same `NotLoaded` the other ruleset-dependent commands use.
/// The absent ruleset is an argument here rather than a lock read inside the
/// command, which is what makes the case reachable without a Tauri runtime.
#[test]
fn exporting_without_a_loaded_ruleset_is_not_loaded_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("companion.md");

    let err =
        export_markdown_to_path(&sample_entity(), None, &locale_labels("en"), &path).unwrap_err();
    assert!(matches!(err, AppError::NotLoaded), "got {err:?}");
    assert!(
        !path.exists(),
        "a failed export must not leave a file behind"
    );
}

/// A label map missing even one chrome key fails the whole export rather than
/// letting the raw key reach the document (`arm_rules::export::ExportError`,
/// CLAUDE.md: "never render a raw ID or enum value as a user-facing label").
/// `AppError::Export` carries the engine's own list of everything unresolved, and
/// nothing is written.
#[test]
fn exporting_with_an_incomplete_label_map_is_export_error() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("companion.md");
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    let err = export_markdown_to_path(&sample_entity(), Some(&localized), &BTreeMap::new(), &path)
        .unwrap_err();
    let AppError::Export { missing } = err else {
        panic!("expected AppError::Export, got {err:?}");
    };
    assert!(!missing.is_empty(), "expected at least one missing key");
    assert!(
        !path.exists(),
        "a failed export must not leave a file behind"
    );
}

/// The command that tells the frontend which labels to send must hand back the
/// engine's own list, never a hand-maintained copy of it.
#[test]
fn export_label_keys_command_returns_the_engine_list() {
    assert_eq!(
        arm_app::commands::export_label_keys(),
        arm_rules::export::LABEL_KEYS
            .iter()
            .map(|key| key.to_string())
            .collect::<Vec<String>>()
    );
}

/// `derive_age` is a thin `#[tauri::command]` delegation to the engine's own
/// [`arm_rules::age_in_saga_year`] (coverage slice, round 3): the clamp policy
/// has one home, so the command must hand back exactly what the engine
/// computes, not a reimplementation of the subtraction.
#[test]
fn derive_age_command_delegates_to_the_engine_computation() {
    assert_eq!(
        arm_app::commands::derive_age(1220, 1190, None),
        arm_rules::age_in_saga_year(1220, 1190, None)
    );
    // The underflow-clamp branch too: a saga year before the birth year.
    assert_eq!(
        arm_app::commands::derive_age(1150, 1190, None),
        arm_rules::age_in_saga_year(1150, 1190, None)
    );
}

/// Slice A1: the frontend hands the ruleset's maximum age along, and the command
/// passes it to the engine, so an age derived from a far-past birth year clamps
/// at the maximum instead of reaching the aging schedule unbounded.
#[test]
fn derive_age_command_clamps_at_the_max_age_it_is_handed() {
    assert_eq!(arm_app::commands::derive_age(1220, 120, Some(500)).age, 500);
}

/// The other view of the same fact: `derive_birth_year` delegates to
/// [`arm_rules::birth_year_in_saga_year`] unchanged.
#[test]
fn derive_birth_year_command_delegates_to_the_engine_computation() {
    assert_eq!(
        arm_app::commands::derive_birth_year(1220, 30),
        arm_rules::birth_year_in_saga_year(1220, 30)
    );
}

/// `AppError`'s `Display` impl is what a terminal-launched binary and any
/// `{err}` formatting show; every variant must print its own distinguishing
/// detail rather than a bare discriminant (coverage slice, round 3 — the impl
/// had no test of any kind before).
#[test]
fn display_formats_every_variant_with_its_distinguishing_detail() {
    assert_eq!(
        AppError::Io {
            message: "disk full".to_string()
        }
        .to_string(),
        "io error: disk full"
    );
    assert_eq!(
        AppError::Ruleset {
            ruleset_kind: "integrity".to_string(),
            errors: vec!["a".to_string(), "b".to_string()],
        }
        .to_string(),
        "ruleset error (integrity): a; b"
    );
    assert_eq!(AppError::NotLoaded.to_string(), "no ruleset loaded");
    assert_eq!(
        AppError::Serialize {
            message: "unexpected EOF".to_string()
        }
        .to_string(),
        "serialize error: unexpected EOF"
    );
    assert_eq!(
        AppError::Export {
            missing: vec!["spell.x".to_string()]
        }
        .to_string(),
        "export error: spell.x"
    );
    assert_eq!(
        AppError::Menu {
            message: "no window".to_string()
        }
        .to_string(),
        "menu error: no window"
    );
}

/// A malformed-JSON `serde_json::Error` must convert to `AppError::Serialize`
/// carrying the same message, not get silently dropped or miscategorized as
/// `AppError::Io`.
#[test]
fn a_malformed_json_error_converts_to_a_serialize_app_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    let expected_message = json_err.to_string();

    let app_err: AppError = json_err.into();

    let AppError::Serialize { message } = app_err else {
        panic!("expected AppError::Serialize, got {app_err:?}");
    };
    assert_eq!(message, expected_message);
}

/// Every document-chrome key the exporter can ask for must exist in every locale,
/// or the exported sheet would print the raw key. Mirrors
/// `every_validation_code_has_a_fluent_key_in_each_locale`.
#[test]
fn every_export_label_key_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for key in arm_rules::export::LABEL_KEYS {
            assert!(
                ftl.contains(&format!("{key} =")),
                "locale '{lang}' is missing key '{key}'"
            );
        }
    }
}

/// The wizard's step rail labels each creation phase through `phase-<slug>`, so a
/// phase with that key missing would render as its raw slug — the one thing a label
/// may never do. `CreationPhase::ALL` is the source of the set, so adding a phase
/// fails this test until both locales carry the key.
///
/// The `wizard-guidance-<slug>` half of this check went with the guidance paragraph
/// itself (manual-testing-findings #21): the wizard no longer explains a step, so
/// demanding copy per phase would demand a string nothing renders.
#[test]
fn every_creation_phase_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for phase in arm_rules::CreationPhase::ALL {
            let key = format!("phase-{phase}");
            assert!(
                ftl.contains(&format!("{key} =")),
                "locale '{lang}' is missing key '{key}'"
            );
        }
    }
}

/// A life-stage XP pool is labelled by which block it is, not by the abilities it
/// may fund (its list is the whole childhood spread, and both childhood blocks
/// share it), so every block needs its own key or the bar would print the slug.
#[test]
fn every_life_stage_xp_pool_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for block in arm_rules::LifeStageBlock::ALL {
            assert!(
                ftl.contains(&format!("xp-pool-{block} =")),
                "locale '{lang}' is missing key 'xp-pool-{block}'"
            );
        }
    }
}

/// The frontend filters each wizard step's findings on the issue's phase, so its
/// `CreationPhase` union has to hold every variant the engine can send. A missing
/// arm is not a type error on the JS side — it is a step that silently shows
/// nothing — so the Rust enum is the source and this test pins the mirror.
#[test]
fn every_creation_phase_is_mirrored_in_the_frontend_union() {
    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for phase in arm_rules::CreationPhase::ALL {
        assert!(
            types.contains(&format!("'{phase}'")),
            "ui/src/lib/types.ts is missing the CreationPhase member '{phase}'"
        );
    }
}

/// The surfaced-modifier read-out labels every aging modifier through
/// `derived-detail-<slug>`, so a kind no locale names would reach the panel as its
/// own raw slug — the one thing a label may never do. `AgingEffect::ALL` is the
/// source of the set, so adding a kind fails this test until both locales carry it.
#[test]
fn every_aging_effect_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for kind in arm_rules::AgingEffect::ALL {
            assert!(
                ftl.contains(&format!("derived-detail-{kind} =")),
                "locale '{lang}' is missing key 'derived-detail-{kind}'"
            );
        }
    }
}

/// Sabine → Erika E2 (round 3): `AgingEffect` above is only one of the **five**
/// engine taxonomies that reach `derived-detail-<slug>`. The Surfaced Modifiers
/// panel composes that key from whatever string the engine put in
/// `SurfacedModifier::detail` (`DerivedSurfacedModifiersSection.svelte`), and
/// `translate` returns the key itself when the message is missing — so a slug no
/// locale names is rendered verbatim on screen, which is the one thing a label may
/// never do. The other four were guarded by nothing.
///
/// `AdvancementSource` is the sharp one: `derived.rs::in_play_mods` stringifies it
/// with no `match` at all, so a new variant needs **no code change anywhere** to
/// reach the panel — every gate stays green while a player reads
/// `derived-detail-correspondence` as a bullet, in English and in German.
///
/// The variant lists live in the helpers below rather than as a `pub const ALL` on
/// each enum — which is where `AgingEffect`, `CreationPhase` and `CrisisSeverity`
/// keep theirs, and is the tidier home — because this round's slice may not edit
/// `crates/arm-rules`. The exhaustive `match` inside each helper is what keeps
/// them honest from this side of the seam: a new variant fails this file's
/// compile, which is the prompt to list it. The key itself is always built from
/// `Display`, never from a literal, so the test cannot agree with itself while
/// disagreeing with the panel.
#[test]
fn every_surfaced_modifier_detail_has_a_fluent_key_in_each_locale() {
    let mut keys: Vec<String> = Vec::new();
    for source in every_advancement_source() {
        keys.push(format!("derived-detail-{source}"));
    }
    for kind in every_special_casting() {
        keys.push(format!("derived-detail-{kind}"));
    }
    for kind in surfaced_magic_resistance_effects() {
        keys.push(format!("derived-detail-{kind}"));
    }
    for track in surfaced_health_tracks() {
        keys.push(format!("derived-detail-{track}"));
    }

    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for key in &keys {
            assert!(
                ftl.contains(&format!("{key} =")),
                "locale '{lang}' is missing key '{key}'"
            );
        }
    }
}

/// The other half of the same panel: each row is prefixed with its family's label
/// (`derived-surfaced-<slug>`), from the one enum that tags every surfaced row.
/// `ModifierFamily::AbilityRoll`'s *detail* is deliberately exempt from the test
/// above — it is a free-text Ability subject, returned unmapped by `detailLabel` —
/// but its family label is composed exactly like the other five and needs its key.
#[test]
fn every_surfaced_modifier_family_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for family in every_modifier_family() {
            let key = format!("derived-surfaced-{family}");
            assert!(
                ftl.contains(&format!("{key} =")),
                "locale '{lang}' is missing key '{key}'"
            );
        }
    }
}

/// Every `AdvancementSource`. All of them reach the panel: `Effect::AdvancementMod`
/// is surfaced unconditionally, with no classification step at all.
fn every_advancement_source() -> Vec<arm_rules::AdvancementSource> {
    use arm_rules::AdvancementSource as Source;

    let all = vec![
        Source::Taught,
        Source::Book,
        Source::Vis,
        Source::Practice,
        Source::Adventure,
        Source::Insight,
        Source::Teaching,
        Source::Authoring,
        Source::SpellMastery,
        Source::All,
    ];
    for source in &all {
        // Exhaustive tripwire: a new variant is a compile error here, which is the
        // prompt to add it to the list above.
        match source {
            Source::Taught
            | Source::Book
            | Source::Vis
            | Source::Practice
            | Source::Adventure
            | Source::Insight
            | Source::Teaching
            | Source::Authoring
            | Source::SpellMastery
            | Source::All => {}
        }
    }
    all
}

/// Every `AdvancementFactor` (D55, Q6). A separate namespace from
/// `every_advancement_source`'s `derived-detail-<slug>`: the factor is
/// rendered through its own `derived-factor-<slug>` key
/// (`DerivedSurfacedModifiersSection.svelte`), because it labels a
/// *multiplier*, not a source — a row carrying one names both.
fn every_advancement_factor() -> Vec<arm_rules::AdvancementFactor> {
    use arm_rules::AdvancementFactor as Factor;

    let all = vec![Factor::Half];
    for factor in &all {
        // Exhaustive tripwire — see `every_advancement_source`.
        match factor {
            Factor::Half => {}
        }
    }
    all
}

/// The factor a halving `advancement_mod` row carries must render as a Fluent
/// string in both locales, never the raw slug — the whole point of D55/Q6.
#[test]
fn every_advancement_factor_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for factor in every_advancement_factor() {
            let key = format!("derived-factor-{factor}");
            assert!(
                ftl.contains(&format!("{key} =")),
                "locale '{lang}' is missing key '{key}'"
            );
        }
    }
}

/// Every `SpecialCasting`. The first three are folded into casting cells rather
/// than surfaced (`derived.rs::in_play_mods`), but they carry labels of their own
/// and are listed here too: the classification of a quirk is a rules decision that
/// can change, and a key that already exists costs nothing to keep.
fn every_special_casting() -> Vec<arm_rules::SpecialCasting> {
    use arm_rules::SpecialCasting as Casting;

    let all = vec![
        Casting::QuietWords,
        Casting::SubtleGestures,
        Casting::DeftForm,
        Casting::Diedne,
        Casting::FaerieRaised,
        Casting::LifeLinkedSpontaneous,
        Casting::SpellImprovisation,
        Casting::Mercurian,
        Casting::LifeBoost,
        Casting::Circumstantial,
        Casting::DoubledAuraPenalty,
    ];
    for kind in &all {
        // Exhaustive tripwire — see `every_advancement_source`.
        match kind {
            Casting::QuietWords
            | Casting::SubtleGestures
            | Casting::DeftForm
            | Casting::Diedne
            | Casting::FaerieRaised
            | Casting::LifeLinkedSpontaneous
            | Casting::SpellImprovisation
            | Casting::Mercurian
            | Casting::LifeBoost
            | Casting::Circumstantial
            | Casting::DoubledAuraPenalty => {}
        }
    }
    all
}

/// The `MagicResistanceEffect`s that reach the panel as a detail slug.
/// `NoFormBonus` and `HalvedParma` are the two that do not: each folds into the
/// flat per-Form Magic Resistance number — for the one Form its own selection
/// names — instead of being listed, so neither carries a label and neither must
/// demand one. The split mirrors `derived.rs::in_play_mods`, and the `match`
/// forces a new variant to be classified rather than silently landing on either
/// side.
fn surfaced_magic_resistance_effects() -> Vec<arm_rules::MagicResistanceEffect> {
    use arm_rules::MagicResistanceEffect as Mr;

    let mut surfaced = Vec::new();
    for kind in [
        Mr::NoFormBonus,
        Mr::HalvedParma,
        Mr::AuraBonus,
        Mr::SusceptibleFaerie,
        Mr::SusceptibleInfernal,
        Mr::ConditionalPenetrationWaiver,
    ] {
        match kind {
            Mr::NoFormBonus | Mr::HalvedParma => {}
            Mr::AuraBonus
            | Mr::SusceptibleFaerie
            | Mr::SusceptibleInfernal
            | Mr::ConditionalPenetrationWaiver => surfaced.push(kind),
        }
    }
    surfaced
}

/// The health tracks that reach the panel as a detail slug. The two penalty tracks
/// are folded into the fatigue/wound read-outs instead of being listed, so they
/// carry no label; the split mirrors `derived.rs::surfaced_modifiers`.
fn surfaced_health_tracks() -> Vec<arm_rules::HealthTrack> {
    use arm_rules::HealthTrack as Track;

    let mut surfaced = Vec::new();
    for track in [
        Track::FatiguePenalty,
        Track::WoundPenalty,
        Track::FatigueRoll,
        Track::CastingFatigue,
        Track::Recovery,
    ] {
        match track {
            Track::FatiguePenalty | Track::WoundPenalty => {}
            Track::FatigueRoll | Track::CastingFatigue | Track::Recovery => surfaced.push(track),
        }
    }
    surfaced
}

/// Every `ModifierFamily` — every one of them prefixes a row in the panel.
fn every_modifier_family() -> Vec<arm_rules::ModifierFamily> {
    use arm_rules::ModifierFamily as Family;

    let all = vec![
        Family::Aging,
        Family::Advancement,
        Family::SpecialCasting,
        Family::AbilityRoll,
        Family::HealthRoll,
        Family::MagicResistance,
        Family::PhysicalActivity,
    ];
    for family in &all {
        // Exhaustive tripwire — see `every_advancement_source`.
        match family {
            Family::Aging
            | Family::Advancement
            | Family::SpecialCasting
            | Family::AbilityRoll
            | Family::HealthRoll
            | Family::MagicResistance
            | Family::PhysicalActivity => {}
        }
    }
    all
}

/// The crisis panel names an illness's severity through `crisis-severity-<slug>`,
/// and the log entry that records one does the same. `CrisisSeverity` is a Rust
/// taxonomy, so a rank no locale names would reach the screen as its own raw slug —
/// the one thing a label may never do. `CrisisSeverity::ALL` is the source of the
/// set, so adding a rank fails this test until both locales carry it.
#[test]
fn every_crisis_severity_has_a_fluent_key_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for severity in arm_rules::CrisisSeverity::ALL {
            assert!(
                ftl.contains(&format!("crisis-severity-{severity} =")),
                "locale '{lang}' is missing key 'crisis-severity-{severity}'"
            );
        }
    }
}

/// The frontend's `CrisisSeverity` union types the rank a log entry records. A
/// missing member is caught by nothing the engine runs, so the Rust enum is the
/// source and this test pins the mirror.
#[test]
fn every_crisis_severity_is_mirrored_in_the_frontend_union() {
    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for severity in arm_rules::CrisisSeverity::ALL {
        assert!(
            types.contains(&format!("'{severity}'")),
            "ui/src/lib/types.ts is missing the CrisisSeverity member '{severity}'"
        );
    }
}

/// The frontend's `AgingEffect` union types every `aging_mod` effect it reads off a
/// loaded item. A missing member is not caught by anything the engine runs, so the
/// Rust enum is the source and this test pins the mirror.
#[test]
fn every_aging_effect_is_mirrored_in_the_frontend_union() {
    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    for kind in arm_rules::AgingEffect::ALL {
        assert!(
            types.contains(&format!("'{kind}'")),
            "ui/src/lib/types.ts is missing the AgingEffect member '{kind}'"
        );
    }
}

/// `ui/src/lib/state.svelte.ts` re-declares `SCHEMA_VERSION` by hand — the frontend
/// stamps it onto every entity it builds from scratch — and TypeScript cannot notice
/// when the Rust constant moves. A stale mirror is silent: the app keeps running and
/// writes saves labelled with a version the engine no longer speaks, so the Rust
/// constant is the source and this test pins the mirror.
#[test]
fn the_frontend_mirrors_the_engine_schema_version() {
    let state = fs::read_to_string(repo_root().join("ui/src/lib/state.svelte.ts")).unwrap();
    let declaration = format!(
        "export const SCHEMA_VERSION = {};",
        arm_rules::SCHEMA_VERSION
    );
    assert!(
        state.contains(&declaration),
        "ui/src/lib/state.svelte.ts must declare `{declaration}`"
    );
}

/// The same pin for `DEFAULT_SAGA_YEAR` (C8). The frontend needs a year the instant
/// its module loads — `AppStore.entity`'s initial placeholder carries a required
/// `saga_year`, and no IPC call can have answered yet — so the engine's constant is
/// mirrored by hand exactly as `SCHEMA_VERSION` is. A stale mirror is silent in the
/// same way: every year the user sees comes from `read_settings` or from the opened
/// document, so the wrong value here would surface only in the seconds before the
/// first settings read, and then only as a year nobody typed.
#[test]
fn the_frontend_mirrors_the_engine_default_saga_year() {
    let state = fs::read_to_string(repo_root().join("ui/src/lib/state.svelte.ts")).unwrap();
    let declaration = format!(
        "export const DEFAULT_SAGA_YEAR = {};",
        arm_rules::DEFAULT_SAGA_YEAR
    );
    assert!(
        state.contains(&declaration),
        "ui/src/lib/state.svelte.ts must declare `{declaration}`"
    );
}

/// Provenance is deliberately not mirrored to the frontend — `PointItem` and
/// `House` drop their `source` too — so a `SourceRef` is never a drift risk and its
/// key (and the `file`/`lines` inside it) is skipped by [`mirrored_keys`].
const PROVENANCE_KEY: &str = "source";

/// Every field name `value` carries, recursively, as the frontend sees them:
/// nested objects and array elements contribute their keys as well, so mirroring
/// `LifeStageRules` covers the `ChildhoodRules`/`LaterLifeRules` inside it and
/// mirroring a `ChildhoodPackage` covers its `ChildhoodEntry` rows.
///
/// [`PROVENANCE_KEY`] is skipped whole, subtree included.
fn mirrored_keys(value: &serde_json::Value, into: &mut std::collections::BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                if key == PROVENANCE_KEY {
                    continue;
                }
                into.insert(key.clone());
                mirrored_keys(nested, into);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                mirrored_keys(item, into);
            }
        }
        _ => {}
    }
}

/// The life-stage payloads cross the Tauri boundary as JSON, and
/// `ui/src/lib/types.ts` mirrors them **by hand**. TypeScript cannot notice when a
/// Rust field is renamed — the mirror keeps compiling against a key the engine no
/// longer sends, and the life-stage surface silently reads `undefined` — so the
/// serialized shape is the source and this test pins the mirror.
///
/// Every optional field is populated on purpose: `skip_serializing_if` would
/// otherwise drop `slot`, `native`, `childhood_package` and `native_language` from
/// the serialization and hide them from the check.
///
/// **Scope: these six types plus the two `Ruleset`/`Entity` member names.** It is
/// deliberately NOT an assertion over `Ruleset`'s whole key set — `types.ts` omits
/// `age_ability_caps`, `categories_requiring_virtue` and `scholarly_language`, so
/// widening it that far could only fail. Mirroring those is separate work, not a
/// reason to loosen or "tighten" this test.
#[test]
fn every_life_stage_field_is_mirrored_in_the_frontend_types() {
    let plan = arm_rules::LifeStagePlan {
        native_language: Some("German".to_string()),
        childhood_package: Some(Id::new("childhood.athletic")),
        // The three post-Gauntlet choices, populated for the same reason every other
        // optional field here is: `skip_serializing_if` would otherwise drop them
        // from the serialization and hide them from the check.
        gauntlet_age: Some(25),
        post_gauntlet_lab_seasons: 13,
        post_gauntlet_spell_levels: 300,
    };
    let budget = arm_rules::LifeStageBudget {
        childhood_native_xp: 75,
        childhood_spread_xp: 45,
        later_life_years: 20,
        later_life_rate: 15,
        later_life_xp: 300,
        apprenticeship_years: 15,
        apprenticeship_xp: 240,
        // A magus of 60 gauntleted at 25, ten of its 13 lab seasons charged (three
        // full lab years of three, plus one: F1) and 300 of
        // the remaining points taken as spell levels. Unlike the plan's choices
        // above, these five are unconditional fields of the derived budget: the
        // engine sends them on every payload, so `types.ts` mirrors them here.
        gauntlet_age: 25,
        post_gauntlet_years: 35,
        post_gauntlet_points: 950,
        post_gauntlet_spell_levels: 300,
        post_gauntlet_xp: 650,
        // D3/D64: 0 for this magus fixture, like every other non-Abandoned-
        // Apprentice character — still mirrored so a future rename is caught.
        truncated_training_years: 0,
        truncated_training_xp: 0,
        truncated_training_spell_levels: 0,
        truncated_training_post_span_years: 0,
        truncated_training_post_span_xp: 0,
    };
    let rules = arm_rules::LifeStageRules {
        apprenticeship: Some(arm_rules::ApprenticeshipRules {
            // Populated on purpose, like every other optional field here: the
            // Gauntlet-age field's placeholder is what blank means, and it reads the
            // baseline off this key.
            default_gauntlet_age: Some(25),
            minimum_abilities: vec![arm_rules::AbilityRequirement {
                ability: Id::new("ability.parma_magica"),
                exemplar: None,
                min_score: 1,
                parameter: None,
            }],
            recommended_abilities: vec![arm_rules::AbilityRequirement {
                ability: Id::new("ability.dead_language"),
                // Populated on purpose, like every other optional field here.
                exemplar: Some("latin".to_string()),
                min_score: 4,
                parameter: Some("Latin".to_string()),
            }],
            recommended_xp: 90,
            xp: 240,
            years: 15,
            truncated_xp_per_year: 16,
            truncated_spell_levels_per_year: 8,
        }),
        childhood: arm_rules::ChildhoodRules {
            years: 5,
            native_language_ability: Id::new("ability.living_language"),
            native_language_xp: 75,
            spread_xp: 45,
            spread_abilities: [Id::new("ability.swim")].into_iter().collect(),
        },
        later_life: arm_rules::LaterLifeRules { xp_per_year: 15 },
        // Populated on purpose, like every other optional field here: the block is
        // what a UI showing the post-Gauntlet rate reads it off, so its three field
        // names have to be mirrored.
        post_apprenticeship: Some(arm_rules::PostApprenticeshipRules {
            lab_season_cost: 10,
            max_charged_lab_seasons_per_year: 3,
            points_per_year: 30,
        }),
    };
    // One row of the magus checklist, which reaches the frontend on
    // `EffectiveScores.magus_minimum_abilities` — the payload the Abilities view
    // renders. Its `met`/`requirement` are the two fields nothing else carries, so
    // without this row a rename of either would leave `types.ts` compiling and the
    // checklist silently reading `undefined`.
    let minimum = arm_rules::MagusMinimumAbility {
        ability: Id::new("ability.dead_language"),
        // Populated on purpose, like every other optional field here.
        exemplar: Some("latin".to_string()),
        parameter: Some("Latin".to_string()),
        min_score: 1,
        score: 0,
        met: false,
        requirement: arm_rules::AbilityRequirementKind::Required,
    };
    let package = arm_rules::ChildhoodPackage {
        id: Id::new("childhood.traveling"),
        entries: vec![arm_rules::ChildhoodEntry {
            ability: Id::new("ability.area_lore"),
            score: 1,
            slot: Some("area_a".to_string()),
            native: true,
        }],
        source: Some(arm_rules::SourceRef::new(
            "Ars Magica - Definitive Edition (Core Rules).md",
            arm_rules::LineRange::new(2388, 2388),
            "anchor",
        )),
    };

    let mut keys = std::collections::BTreeSet::new();
    for payload in [
        serde_json::to_value(&plan).unwrap(),
        serde_json::to_value(budget).unwrap(),
        serde_json::to_value(&rules).unwrap(),
        serde_json::to_value(&package).unwrap(),
        serde_json::to_value(&minimum).unwrap(),
    ] {
        mirrored_keys(&payload, &mut keys);
    }
    // A floor, so a collector that silently gathered nothing cannot look green.
    assert!(
        keys.len() >= 30,
        "expected the life-stage payloads to carry at least 30 field names, got {keys:?}"
    );

    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    // The two member names the frontend reaches the whole surface through: the
    // ruleset's `life_stages`/`childhoods` catalogues and the entity's own plan.
    for key in keys
        .iter()
        .map(String::as_str)
        .chain(["life_stages", "childhoods"])
    {
        assert!(
            types.contains(&format!("{key}:")) || types.contains(&format!("{key}?:")),
            "ui/src/lib/types.ts declares no '{key}' property"
        );
    }
}

/// The aging payloads cross the Tauri boundary as JSON and `ui/src/lib/types.ts`
/// mirrors them **by hand**, exactly like the life-stage ones — so this is a
/// SIBLING of [`every_life_stage_field_is_mirrored_in_the_frontend_types`], not a
/// widening of it. That test's scope note fixes it at six named types plus two
/// member names, and folding the aging surface into it would dilute its floor
/// rather than add a check.
///
/// Every optional field is populated on purpose: `skip_serializing_if` would
/// otherwise drop `year`, `age`, `die`, `total`, `living_conditions`,
/// `apparent_age_increased`, `crisis` and the four fields a resolved Crisis
/// records (`crisis_die`, `crisis_total`, `crisis_row`, `crisis_severity`) from
/// the serialization and hide them from the check.
///
/// **`AgingLogEntry::points` is deliberately left empty.** It is a
/// `BTreeMap<Characteristic, u8>`, so its serialized *keys* are Characteristic
/// slugs (`sta`) rather than field names, and [`mirrored_keys`] cannot tell the
/// two apart — a populated map would demand a `sta:` property of `types.ts`. The
/// key itself is covered by the chained member names below.
#[test]
fn every_aging_field_is_mirrored_in_the_frontend_types() {
    let readout = arm_app::effective_dto::AgingReadout {
        first_roll_age: 36,
        begins_after_age: 35,
        // Two years, the first dated: a `None` calendar year would be skipped and
        // hide the `year` key, so one of each proves both shapes serialize.
        schedule: vec![
            arm_app::effective_dto::AgingScheduleYear {
                age: 36,
                year: Some(1216),
                recorded: true,
            },
            arm_app::effective_dto::AgingScheduleYear {
                age: 37,
                year: None,
                recorded: false,
            },
        ],
        rolls_owed: 2,
        rolls_recorded: 1,
        age_modifier: 4,
        living_conditions_modifier: -3,
        longevity_modifier: 5,
        // Faerie Blood's -1 (`ArMDE:3801`): the term the book's three-line formula does
        // not name, which the read-out must still surface for its own arithmetic to
        // add up (guided-creation-review-2026-08 #22).
        trait_modifier: -1,
        // Populated on purpose: `true` is the standing `ArMDE:16575` predicate, and the
        // field is the one thing that tells it apart from the per-roll cap flag.
        longevity_clamp_active: true,
        // 4 - (-3) - 5 + (-1): the terms above, as the engine sums them.
        fixed_total: 1,
    };
    let entry = arm_rules::AgingLogEntry {
        year: Some(1220),
        age: Some(40),
        effect: "Grey at the temples.".to_string(),
        die: Some(9),
        total: Some(13),
        living_conditions: [Id::new("living_condition.work_in_a_mine")]
            .into_iter()
            .collect(),
        points: BTreeMap::new(),
        apparent_age_increased: true,
        crisis: true,
        crisis_die: Some(7),
        crisis_total: Some(12),
        crisis_row: Some(Id::new("crisis.minor_illness")),
        crisis_severity: Some(arm_rules::CrisisSeverity::Minor),
    };

    let mut keys = std::collections::BTreeSet::new();
    for payload in [
        serde_json::to_value(&readout).unwrap(),
        serde_json::to_value(&entry).unwrap(),
    ] {
        mirrored_keys(&payload, &mut keys);
    }
    // A floor, so a collector that silently gathered nothing cannot look green.
    assert!(
        keys.len() >= 15,
        "expected the aging payloads to carry at least 15 field names, got {keys:?}"
    );

    let types = fs::read_to_string(repo_root().join("ui/src/lib/types.ts")).unwrap();
    // The two member names the frontend reaches the rest of the surface through:
    // the effective-scores read-out and the character's own standing conditions.
    for key in keys
        .iter()
        .map(String::as_str)
        .chain(["aging", "living_conditions", "points"])
    {
        assert!(
            types.contains(&format!("{key}:")) || types.contains(&format!("{key}?:")),
            "ui/src/lib/types.ts declares no '{key}' property"
        );
    }
}

/// Every `{placeholder}` key the shipped catalogues declare, across the three
/// parameterized kinds (Virtues/Flaws, Abilities, spells). Each names both the
/// placeholder in an item's localized name and its `param-label-<key>` label — the
/// slot label the Markdown export prints wherever a parameterized name has no chosen
/// value (`export::Doc::parameterized_name`). The family is catalogue *data*, so it
/// is recomputed from the shipped rules rather than listed in `LABEL_KEYS`; the
/// frontend composes the same set for the label map it sends
/// (`composedExportLabelKeys` in `ui/src/lib/state.svelte.ts`).
fn shipped_parameter_keys() -> Vec<String> {
    let read = |relative: &str| -> serde_json::Value {
        let json = fs::read_to_string(repo_root().join(relative)).unwrap();
        serde_json::from_str(&json).unwrap()
    };
    let mut keys: Vec<String> = Vec::new();
    let collect = |items: Option<&Vec<serde_json::Value>>, keys: &mut Vec<String>| {
        for item in items.into_iter().flatten() {
            for parameter in item["parameters"].as_array().into_iter().flatten() {
                keys.push(parameter["key"].as_str().unwrap().to_string());
            }
        }
    };
    let items = read("rules/core/virtues_flaws.json");
    collect(items.as_array(), &mut keys);
    let spells = read("rules/core/spells.json");
    collect(spells["spells"].as_array(), &mut keys);
    let abilities = read("rules/core/abilities.json");
    for ability in abilities["abilities"].as_array().into_iter().flatten() {
        if let Some(key) = ability["parameter"].as_str() {
            keys.push(key.to_string());
        }
    }
    keys.sort();
    keys.dedup();
    // One known key per scanned catalogue, so a scan that silently collected nothing
    // cannot leave the coverage check looking green.
    for known in ["ability", "form", "language"] {
        assert!(
            keys.contains(&known.to_string()),
            "expected the shipped parameter key '{known}', got {keys:?}"
        );
    }
    keys
}

/// The slot label for a parameterized name is composed from the parameter key, so a
/// key the locales do not translate would reach the exported sheet (and the picker)
/// as its own Fluent key. Recomputed from the catalogue, so a newly parameterized
/// item fails here until both locales name its slot.
#[test]
fn every_shipped_parameter_key_has_a_param_label_in_each_locale() {
    for lang in ["en", "de"] {
        let ftl = fs::read_to_string(repo_root().join(format!("locales/{lang}/main.ftl"))).unwrap();
        for key in shipped_parameter_keys() {
            assert!(
                ftl.contains(&format!("param-label-{key} =")),
                "locale '{lang}' is missing key 'param-label-{key}'"
            );
        }
    }
}

/// The CC-BY-SA rules text is bundled into every installer, so its attribution
/// has to travel with it. `rules/NOTICE.md` is that attribution, and the only
/// legal text that reaches a Linux user's disk — the deb/rpm/AppImage bundlers
/// never read `bundle.licenseFile`. Without this test the sole check on it is a
/// real bundle build, which only runs after a release tag is already pushed.
#[test]
fn attribution_notice_ships_with_the_bundled_rules_data() {
    let notice = repo_root().join("rules/NOTICE.md");
    assert!(
        notice.is_file(),
        "rules/NOTICE.md is missing; the bundled rules data would ship without \
         its CC-BY-SA attribution"
    );

    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("crates/arm-app/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let resources = &config["bundle"]["resources"];
    assert!(
        resources
            .as_object()
            .expect("bundle.resources should be a map of source -> destination")
            .contains_key("../../rules/NOTICE.md"),
        "tauri.conf.json bundle.resources must map ../../rules/NOTICE.md so the \
         notice is installed alongside rules/core and rules/i18n"
    );
}

/// K1/VA4: `app.security.csp` must never regress to `null` (which disables
/// Tauri's CSP injection into the webview entirely). No injection sink exists
/// today (no `{@html}`/`innerHTML` anywhere in the frontend), but the CSP is
/// the defence-in-depth backstop for the day one is introduced by accident —
/// and in a Tauri webview that backstop matters more than in an ordinary
/// browser tab, because script running there sits behind the same origin the
/// `invoke()` IPC bridge trusts.
#[test]
fn csp_is_set_and_restrictive() {
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("crates/arm-app/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("app.security.csp must be a restrictive policy string, not null");
    assert!(
        csp.contains("default-src 'self'"),
        "csp must default-deny to the app's own origin, got: {csp}"
    );
    assert!(
        !csp.contains("unsafe-eval"),
        "csp must not permit unsafe-eval, got: {csp}"
    );
}

/// Each rules directory states its own licensing, because the two are not the
/// same: `rules/core/` is MIT throughout, while `rules/i18n/` is MIT in form and
/// CC-BY-SA 4.0 in the rules text it carries. Both ride into the installers with
/// the directories they describe.
#[test]
fn each_rules_directory_carries_its_own_license_file() {
    for dir in ["core", "i18n"] {
        let license = rules_dir().join(dir).join("LICENSE");
        assert!(
            license.is_file(),
            "rules/{dir}/LICENSE is missing; the bundled data would not state \
             which license covers it"
        );
    }
}

/// GA1: a confirmed discard latches `CloseGuardState::confirmed = true` (see
/// `main.rs`'s `guard_blocks_quit`) so a re-issued close/quit belonging to the
/// SAME confirmed action passes through without a second dialog. That latch
/// must not survive into the NEXT dirty-state report: `update_close_guard`
/// fires on every dirty-state transition the frontend reports (whenever the
/// entity's snapshot-compared `dirty` flag changes — see
/// `AppStore.closeGuardPayload` / the `$effect` in `App.svelte`), and a stale
/// `confirmed == true` there would let a LATER close/quit skip the
/// discard-confirmation dialog for edits the user never actually confirmed
/// discarding. Dormant today (no window-reactivation path exists in
/// `main.rs` yet — closing the window force-closes it via `destroy()`), but a
/// real one-way latch with no reset otherwise, on exactly the surface
/// CLAUDE.md's "Unsaved-changes guard" section calls load-bearing on macOS,
/// where `RunEvent::ExitRequested` (Cmd+Q) is independent of the window-close
/// path and does not itself clear anything.
#[test]
fn a_fresh_dirty_state_report_clears_the_confirmed_discard_latch() {
    use arm_app::commands::{CloseGuardLabels, CloseGuardState};

    let mut guard = CloseGuardState {
        dirty: true,
        // As main.rs's dialog callback leaves it once the user confirms
        // discarding: `guard.confirmed = true;` right before `on_discard` runs.
        confirmed: true,
        ..CloseGuardState::default()
    };

    // The frontend reports a fresh dirty state — e.g. the character was edited
    // again after the window that showed the dialog was destroyed but the
    // process (macOS) lived on.
    guard.report_dirty_state(true, CloseGuardLabels::default());

    assert!(
        !guard.confirmed,
        "a fresh dirty-state report must clear the one-shot discard latch, or a \
         later close/quit could silently skip the confirmation dialog for edits \
         the user never confirmed discarding"
    );
}

/// Erika F4 (MAJOR): the close/quit guard is the mandatory product behaviour
/// `CLAUDE.md` singles out, and until this test its decision logic had no
/// coverage at any level. `guard_blocks_quit` (`main.rs`) lives in a binary, the
/// crate does not enable Tauri's `test` feature, and WebDriver cannot answer a
/// native GTK dialog — so of its branches the e2e suite reached exactly two
/// (clean-allow and dirty-show-dialog) and nothing at all reached the
/// `confirmed` pass-through, the `showing` no-second-dialog branch, or either
/// dialog answer. Inverting one `!` in the callback made Cancel *discard* the
/// user's work with every gate still green.
///
/// The decision is pure over [`CloseGuardState`] — Tauri appears only in the
/// dialog presentation — so it is [`guard_decision`] now, and this table walks
/// every row of it.
#[test]
fn the_close_guard_allows_a_quit_with_nothing_unsaved() {
    use arm_app::commands::{CloseGuardState, Decision, guard_decision};

    let mut guard = CloseGuardState::default();
    assert_eq!(guard_decision(&mut guard), Decision::Allow);
    assert!(
        !guard.showing,
        "a clean document must not arm the dialog latch"
    );
}

/// The `confirmed` pass-through: the user has already answered "discard", and
/// the close/quit that answer re-issues must go through without a second
/// dialog. Dirty is still true — the edits were never saved, that is the whole
/// point — so nothing but this latch distinguishes it from the row below.
#[test]
fn the_close_guard_lets_an_already_confirmed_discard_through() {
    use arm_app::commands::{CloseGuardState, Decision, guard_decision};

    let mut guard = CloseGuardState {
        dirty: true,
        confirmed: true,
        ..CloseGuardState::default()
    };
    assert_eq!(guard_decision(&mut guard), Decision::Allow);
}

/// The live case: unsaved edits, nothing confirmed, no dialog open yet. The quit
/// is blocked AND the caller is told to put the confirmation up — and the latch
/// is armed as part of the decision, so the row below can distinguish itself.
#[test]
fn the_close_guard_blocks_a_dirty_quit_and_asks_for_the_dialog() {
    use arm_app::commands::{CloseGuardState, Decision, guard_decision};

    let mut guard = CloseGuardState {
        dirty: true,
        ..CloseGuardState::default()
    };
    assert_eq!(guard_decision(&mut guard), Decision::BlockAndShow);
    assert!(
        guard.showing,
        "the decision must record that a dialog is now up, or a second \
         close/quit would stack another one on top of it"
    );
}

/// Alt+F4 while the confirmation is already on screen. Still blocked, but the
/// caller must NOT show a second dialog: two stacked confirmations for one
/// document means the user answers twice to discard once, and the second answer
/// arrives against a window that is already gone.
#[test]
fn the_close_guard_does_not_stack_a_second_dialog() {
    use arm_app::commands::{CloseGuardState, Decision, guard_decision};

    let mut guard = CloseGuardState {
        dirty: true,
        showing: true,
        ..CloseGuardState::default()
    };
    assert_eq!(guard_decision(&mut guard), Decision::Block);
    assert!(guard.showing, "the open dialog is still open");
}

/// The user pressed Discard. The dialog is down, and the latch is set so the
/// close/quit the callback re-issues passes straight through the decision above.
#[test]
fn confirming_the_discard_latches_the_re_issued_quit_through() {
    use arm_app::commands::{CloseGuardState, apply_dialog_answer};

    let mut guard = CloseGuardState {
        dirty: true,
        showing: true,
        ..CloseGuardState::default()
    };
    apply_dialog_answer(&mut guard, true);

    assert!(!guard.showing, "the dialog is no longer on screen");
    assert!(
        guard.confirmed,
        "without the latch the re-issued close/quit would raise a second dialog"
    );
}

/// The user pressed Cancel — the one row where getting the sign wrong destroys
/// the document. The dialog comes down and NOTHING else changes: the work is
/// still unsaved, still unconfirmed, and the next quit must ask again.
#[test]
fn cancelling_the_discard_keeps_the_unsaved_work() {
    use arm_app::commands::{CloseGuardState, Decision, apply_dialog_answer, guard_decision};

    let mut guard = CloseGuardState {
        dirty: true,
        showing: true,
        ..CloseGuardState::default()
    };
    apply_dialog_answer(&mut guard, false);

    assert!(!guard.showing, "the dialog is no longer on screen");
    assert!(
        !guard.confirmed,
        "Cancel must never latch a discard — that inversion turns the button \
         that protects the user's work into the one that destroys it"
    );
    assert!(guard.dirty, "the edits are still unsaved");
    assert_eq!(
        guard_decision(&mut guard),
        Decision::BlockAndShow,
        "the next quit must ask again"
    );
}

/// Erika E1 (round 2, MAJOR): F4 moved the guard's *state* decisions here but
/// left both of its *action* decisions inside `main.rs::guard_blocks_quit`,
/// where nothing at any level could reach them — inverting either still
/// discarded the user's work with every gate green. [`Decision::blocks`] is the
/// first of the two: the `Decision` → "call `prevent_close`/`prevent_exit`"
/// mapping, which `guard_blocks_quit` used to spell as a `match` of its own.
///
/// `Allow` and `BlockAndShow` were at least reachable end-to-end (a clean quit
/// and a dirty quit in the e2e specs). **`Block` was covered by nothing**: each
/// dirty spec issues `request_exit` exactly once, so a second close/quit while
/// the confirmation is on screen — the entire reason `Block` exists — was never
/// exercised anywhere.
#[test]
fn the_allow_decision_lets_the_close_proceed() {
    use arm_app::commands::Decision;

    assert!(
        !Decision::Allow.blocks(),
        "nothing is unsaved, so the close/quit must not be prevented"
    );
}

/// The row no test and no e2e spec reached before: Alt+F4 (or Cmd+Q) a second
/// time while the confirmation is already up. Mapping this to "do not block"
/// closes the window out from under the open dialog with the edits unsaved.
#[test]
fn the_block_decision_refuses_a_second_close_while_the_dialog_is_up() {
    use arm_app::commands::Decision;

    assert!(
        Decision::Block.blocks(),
        "a close/quit arriving while the confirmation is on screen must be \
         prevented; allowing it destroys the window out from under the dialog \
         with the work unsaved"
    );
}

/// The live case: the confirmation is going up now, so the close/quit that
/// triggered it must be prevented and re-issued only if the user says discard.
#[test]
fn the_block_and_show_decision_refuses_the_close() {
    use arm_app::commands::Decision;

    assert!(
        Decision::BlockAndShow.blocks(),
        "the close/quit that raises the confirmation must itself be prevented"
    );
}

/// Erika E1 (round 3, MINOR): the *fifth* decision — which decisions warrant
/// putting the confirmation on screen — was still spelled as a comparison inside
/// `main.rs::guard_blocks_quit`, under a doc comment asserting that no decision was
/// left there at all. It is the decision that gives [`Decision::Block`] its
/// meaning, and the `guard_decision` half of that contract is pinned
/// (`the_close_guard_does_not_stack_a_second_dialog`) while the caller-side half
/// was not: nothing at any level reached it, because `main.rs` has no unit seam,
/// and each dirty e2e spec issues exactly one close/quit — so "blocked" and
/// "blocked without a second dialog" are indistinguishable to WebDriver, which
/// cannot see a native GTK dialog in the first place.
///
/// `Allow` is the trivially safe row.
#[test]
fn the_allow_decision_raises_no_dialog() {
    use arm_app::commands::Decision;

    assert!(
        !Decision::Allow.shows_dialog(),
        "nothing is unsaved, so there is nothing to confirm"
    );
}

/// **The row that matters.** A second Alt+F4 (or Cmd+Q) while the confirmation is
/// already on screen must be refused *silently*: stacking a second native
/// confirmation on the same document makes the user answer twice to discard once,
/// and the second answer arrives against a window that is already gone. That is
/// the entire reason `Block` exists as a variant distinct from `BlockAndShow`.
#[test]
fn the_block_decision_does_not_raise_a_second_dialog() {
    use arm_app::commands::Decision;

    assert!(
        !Decision::Block.shows_dialog(),
        "the confirmation is already up; raising another one stacks two dialogs \
         on one document"
    );
}

/// The live case, and the other way the branch can go wrong: mapping this to "do
/// not show" leaves a dirty quit prevented with **no dialog at all**, so the app
/// silently refuses to close on every quit path — including macOS Cmd+Q — and the
/// user's only remaining exit is a force-kill, which discards exactly the unsaved
/// work the guard exists to protect.
#[test]
fn the_block_and_show_decision_raises_the_confirmation() {
    use arm_app::commands::Decision;

    assert!(
        Decision::BlockAndShow.shows_dialog(),
        "a dirty close/quit must put the confirmation on screen, or the app just \
         refuses to close with no explanation"
    );
}

/// Erika E1, second half: the *other* action decision the F4 extraction left in
/// the binary — whether the answer the user gave re-issues the close/quit.
///
/// [`apply_dialog_answer`] returned `()`, so it recorded the latch but decided
/// nothing; `main.rs` then gated the re-issue on a bare `if !discard { return; }`
/// that no test could reach. The action behind that gate is `w.destroy()` or
/// `app.exit(0)` — the act of throwing the document away — so inverting the `!`
/// made **Cancel** destroy the unsaved work, with `cargo test`, clippy, fmt,
/// vitest and the whole e2e suite still green (WebDriver cannot answer a native
/// GTK dialog, see `the_discard_confirmation_is_native_in_the_default_build`).
///
/// [`resolve_discard_dialog`] now owns the gate, so `guard_blocks_quit` holds no
/// branch about it at all and this test is the thing that pins it.
#[test]
fn confirming_the_discard_runs_the_discard_action() {
    use arm_app::commands::{CloseGuardState, resolve_discard_dialog};
    use std::sync::Mutex;

    let guard = Mutex::new(CloseGuardState {
        dirty: true,
        showing: true,
        ..CloseGuardState::default()
    });
    let mut discarded = false;

    resolve_discard_dialog(&guard, true, || discarded = true);

    assert!(
        discarded,
        "the user pressed Discard, so the close/quit must be re-issued — \
         without this the app simply refuses to close and the answer is ignored"
    );
    let state = guard.lock().expect("close guard lock poisoned");
    assert!(!state.showing, "the dialog is no longer on screen");
    assert!(
        state.confirmed,
        "without the latch the re-issued close/quit would raise a second dialog"
    );
}

/// The row where getting the sign wrong destroys the document: Cancel must leave
/// the work alone AND must not run the discard action. The state assertions
/// below duplicate `cancelling_the_discard_keeps_the_unsaved_work` deliberately —
/// all four of them stayed true under the inversion Erika found, so the load-
/// bearing assertion is the first one.
#[test]
fn cancelling_the_discard_must_not_run_the_discard_action() {
    use arm_app::commands::{CloseGuardState, Decision, guard_decision, resolve_discard_dialog};
    use std::sync::Mutex;

    let guard = Mutex::new(CloseGuardState {
        dirty: true,
        showing: true,
        ..CloseGuardState::default()
    });
    let mut discarded = false;

    resolve_discard_dialog(&guard, false, || discarded = true);

    assert!(
        !discarded,
        "Cancel must never re-issue the close/quit — that inversion turns the \
         button that protects the user's work into the one that destroys it"
    );
    let mut state = guard.lock().expect("close guard lock poisoned");
    assert!(!state.showing, "the dialog is no longer on screen");
    assert!(!state.confirmed, "Cancel must never latch a discard");
    assert!(state.dirty, "the edits are still unsaved");
    assert_eq!(
        guard_decision(&mut state),
        Decision::BlockAndShow,
        "the next quit must ask again"
    );
}

/// The discard action must run with the close-guard lock **released**. This is
/// not hygiene: the action is `app.exit(0)`, which fires `RunEvent::ExitRequested`
/// synchronously, which re-enters `guard_blocks_quit`, which locks this very
/// mutex. Running it under the lock deadlocks the app on the one gesture that is
/// supposed to close it — so the release is part of the contract this seam owns,
/// not an implementation detail of the caller.
///
/// `try_lock` rather than `lock`, because a regression here must fail the test
/// rather than hang the suite.
#[test]
fn the_discard_action_runs_with_the_close_guard_lock_released() {
    use arm_app::commands::{CloseGuardState, resolve_discard_dialog};
    use std::sync::Mutex;

    let guard = Mutex::new(CloseGuardState {
        dirty: true,
        showing: true,
        ..CloseGuardState::default()
    });
    let mut lock_was_free = false;

    resolve_discard_dialog(&guard, true, || {
        lock_was_free = guard.try_lock().is_ok();
    });

    assert!(
        lock_was_free,
        "the discard action re-enters the close guard (app.exit(0) fires \
         ExitRequested synchronously), so holding the lock across it deadlocks \
         the quit it is meant to perform"
    );
}

/// C3b: the New/Open discard confirmation is the SAME native dialog the
/// close/quit guard shows, and `native_discard_confirmation_enabled()` is the
/// single switch deciding whether this build owns one.
///
/// The polarity is the inverse of [`e2e_file_override`]'s, and deliberately so.
/// There the seam is the *affordance* (a path override), so the shipped build
/// must not have it. Here the seam is the *bypass*: the shipped build is the one
/// that must act — it always shows the real dialog and never hands the decision
/// back to the webview — while the `e2e-testing` build declines, because no
/// WebDriver capability available to this project can dismiss a native GTK
/// dialog (the dirty-quit specs leave one open for the app's whole lifetime,
/// which is why they are pinned as the tail spec of their file). This test runs
/// under the plain `cargo test -p arm-app` gate, which is exactly the build
/// users receive.
#[cfg(not(feature = "e2e-testing"))]
#[test]
fn the_discard_confirmation_is_native_in_the_default_build() {
    assert!(
        arm_app::commands::native_discard_confirmation_enabled(),
        "the shipped build must confirm a discard with the native dialog; \
         returning `None` there would hand a load-bearing data-loss guard to a \
         fallback that exists only so the e2e suite can answer it"
    );
}

/// Mirror of the above, proving the gate actually opens rather than staying
/// permanently shut — compiled WITH the `e2e-testing` feature (exactly what
/// `ui/e2e/wdio.conf.js` passes to `cargo tauri build --no-bundle --features
/// e2e-testing`), the native confirmation must stand down, or every spec's
/// `returnToStartScreen()` setup hangs on a dialog WebDriver cannot answer.
#[cfg(feature = "e2e-testing")]
#[test]
fn the_discard_confirmation_stands_down_under_the_e2e_feature() {
    assert!(
        !arm_app::commands::native_discard_confirmation_enabled(),
        "under `e2e-testing` the backend must return no native answer, so the \
         frontend falls back to the in-app prompt the specs can click"
    );
}

/// C3b, trap 2: the New/Open confirmation must not ride on
/// `CloseGuardState::confirmed`. That flag is a one-shot LATCH — set in
/// `main.rs`'s dialog callback so the re-issued close/quit belonging to the same
/// confirmed action passes straight through — and reusing it for a NON-terminal
/// action would suppress every later confirmation in the session, silently
/// discarding the user's work on the second New.
///
/// What actually enforces that is the signature: `confirm_discard` takes no
/// `State<'_, AppState>`, so it cannot reach the latch at all. So the signature
/// is what this test reads.
///
/// **Erika F6.** This used to assert `latched.confirmed` one line after setting
/// it, plus a verbatim copy of the build-property assertion in
/// `the_discard_confirmation_is_native_in_the_default_build` — a tautology and a
/// duplicate, neither of which mentioned the relationship the name claims, and
/// both of which stayed green under the very change they were named for. The
/// behavioural half (that a second New really does ask again) remains the
/// frontend's `state.svelte.test.ts`, which confirms twice in one session.
#[test]
fn a_latched_close_guard_cannot_suppress_a_discard_confirmation() {
    let source = fs::read_to_string(repo_root().join("crates/arm-app/src/commands.rs")).unwrap();
    let after = source
        .split_once("pub async fn confirm_discard(")
        .expect("confirm_discard must still be declared in commands.rs")
        .1;
    let signature = after
        .split_once(')')
        .expect("confirm_discard's parameter list must close")
        .0;

    // The extraction really found the parameter list, so a pass below cannot be
    // vacuous.
    assert!(
        signature.contains("labels"),
        "expected confirm_discard's parameters, got: {signature}"
    );
    assert!(
        !signature.contains("State"),
        "confirm_discard must not take managed state: reading \
         `CloseGuardState::confirmed` there would let one confirmed quit silence \
         every later New/Open in the session, discarding the user's work without \
         asking. Got: {signature}"
    );
}

/// The command name is a string literal in `ui/src/lib/ipc.ts` and a function
/// name in `commands.rs`, with nothing but this test holding them together —
/// the same gap `main.rs`'s `the_shadow_script_invokes_the_registered_request_close_command`
/// closes for the window-close shadow. A rename on one side would leave New/Open
/// invoking a command that does not exist; the frontend would swallow the
/// rejection and fall back to the in-app prompt, so the app would keep working
/// while quietly shipping the seam's fallback as its production dialog.
#[test]
fn the_frontend_invokes_the_registered_discard_confirmation_command() {
    let ipc = fs::read_to_string(repo_root().join("ui/src/lib/ipc.ts")).unwrap();
    assert!(
        ipc.contains("invoke('confirm_discard'"),
        "ui/src/lib/ipc.ts must invoke the `confirm_discard` command by that \
         exact name"
    );

    let main_rs = fs::read_to_string(repo_root().join("crates/arm-app/src/main.rs")).unwrap();
    assert!(
        main_rs.contains("commands::confirm_discard"),
        "`confirm_discard` must be registered in main.rs's invoke_handler, or \
         the frontend's call can never reach it"
    );
}

/// K5/VA5: `ARM_E2E_FILE` must be inert unless the crate is built with the
/// `e2e-testing` Cargo feature. Without that gate this override compiled
/// unconditionally into the exact release binary end users install, letting
/// anything that can set an env var before launch silently redirect
/// Save/Open away from the native dialog with no user-facing confirmation.
/// This test runs under the plain `cargo test -p arm-app` gate (no features
/// enabled), which is exactly the build users receive.
#[cfg(not(feature = "e2e-testing"))]
#[test]
fn e2e_file_override_is_compiled_out_of_the_default_build() {
    // SAFETY: no other test in this binary reads or writes ARM_E2E_FILE, so
    // there is no cross-test race on this process-global.
    unsafe { std::env::set_var("ARM_E2E_FILE", "/nonexistent/should-not-be-honored") };
    let result = arm_app::commands::e2e_file_override();
    unsafe { std::env::remove_var("ARM_E2E_FILE") };
    assert!(
        result.is_none(),
        "ARM_E2E_FILE must not be honored unless the `e2e-testing` feature is \
         enabled at compile time; this test builds without it, matching the \
         binary shipped to users"
    );
}

/// Sibling of the above for the Markdown-export seam.
#[cfg(not(feature = "e2e-testing"))]
#[test]
fn e2e_export_file_override_is_compiled_out_of_the_default_build() {
    // SAFETY: no other test in this binary reads or writes ARM_E2E_EXPORT_FILE.
    unsafe {
        std::env::set_var(
            "ARM_E2E_EXPORT_FILE",
            "/nonexistent/should-not-be-honored.md",
        )
    };
    let result = arm_app::commands::e2e_export_file_override();
    unsafe { std::env::remove_var("ARM_E2E_EXPORT_FILE") };
    assert!(
        result.is_none(),
        "ARM_E2E_EXPORT_FILE must not be honored unless the `e2e-testing` \
         feature is enabled at compile time"
    );
}

/// Mirror of the two tests above, proving the gate actually opens rather than
/// just staying permanently shut: compiled WITH the `e2e-testing` feature
/// (exactly what `ui/e2e/wdio.conf.js` / `wdio.portable.conf.js` pass to
/// `cargo tauri build --no-bundle --features e2e-testing`), the overrides
/// must still work, or the e2e suite's 35 specs plus the portable-layout run
/// lose their save/load/export seam entirely.
#[cfg(feature = "e2e-testing")]
#[test]
fn e2e_file_override_is_honored_when_the_feature_is_enabled() {
    unsafe { std::env::set_var("ARM_E2E_FILE", "/nonexistent/honored-path.json") };
    let result = arm_app::commands::e2e_file_override();
    unsafe { std::env::remove_var("ARM_E2E_FILE") };
    assert_eq!(
        result,
        Some(PathBuf::from("/nonexistent/honored-path.json"))
    );
}

#[cfg(feature = "e2e-testing")]
#[test]
fn e2e_export_file_override_is_honored_when_the_feature_is_enabled() {
    unsafe { std::env::set_var("ARM_E2E_EXPORT_FILE", "/nonexistent/honored-path.md") };
    let result = arm_app::commands::e2e_export_file_override();
    unsafe { std::env::remove_var("ARM_E2E_EXPORT_FILE") };
    assert_eq!(result, Some(PathBuf::from("/nonexistent/honored-path.md")));
}

/// C6, both seams. `activate_menu_item` and `installed_menu` exist so a
/// WebDriver spec can press a native menu item and read the menu the OS was
/// actually given — neither of which is in the DOM, permanently (C3a). They
/// must be inert in the shipped build for the same reason `request_exit` is
/// (`main.rs`): a seam that drives the app's document actions has no business
/// existing in the binary users install, however benign the local threat model.
/// This test runs under the plain `cargo test -p arm-app` gate, which is
/// exactly that build.
#[cfg(not(feature = "e2e-testing"))]
#[test]
fn the_menu_test_seams_are_compiled_out_of_the_default_build() {
    assert!(
        !arm_app::commands::menu_test_seams_enabled(),
        "activate_menu_item / installed_menu must be inert unless built with \
         the `e2e-testing` feature, or the shipped binary exposes an IPC path \
         that can run New/Open/Save/Save As/Export/Settings"
    );
}

/// Mirror of the above, proving the gate opens rather than staying permanently
/// shut — compiled WITH the feature `ui/e2e/wdio.conf.js` passes to
/// `cargo tauri build --no-bundle`, or the menu specs have no seam to drive.
#[cfg(feature = "e2e-testing")]
#[test]
fn the_menu_test_seams_act_when_the_feature_is_enabled() {
    assert!(arm_app::commands::menu_test_seams_enabled());
}

/// The seam is worth nothing unless it fires the SAME handler the OS fires.
/// `arm_app::menu::forward_menu_action` is that one handler: `main.rs`'s
/// `on_menu_event` calls it and so does `activate_menu_item`. A second `emit`
/// anywhere would be a parallel path, and a spec driving it would prove only
/// that the parallel path works.
#[test]
fn the_os_menu_handler_and_the_activation_seam_share_one_dispatch_path() {
    let main_rs = fs::read_to_string(repo_root().join("crates/arm-app/src/main.rs")).unwrap();
    let commands_rs =
        fs::read_to_string(repo_root().join("crates/arm-app/src/commands.rs")).unwrap();

    assert!(
        main_rs.contains("menu::forward_menu_action"),
        "main.rs's on_menu_event must dispatch through \
         `arm_app::menu::forward_menu_action`"
    );
    assert!(
        commands_rs.contains("forward_menu_action"),
        "`activate_menu_item` must dispatch through the same \
         `forward_menu_action`, not emit the event a second way"
    );
    assert!(
        !main_rs.contains("MENU_ACTION_EVENT"),
        "main.rs must no longer emit the menu event itself; that line moved \
         into `forward_menu_action` so the seam and the OS cannot drift"
    );
}

/// The two command names are string literals on the WebdriverIO side and
/// function names in `commands.rs`, with nothing but this test holding them
/// together — the same gap
/// `the_frontend_invokes_the_registered_discard_confirmation_command` closes
/// for `confirm_discard`. A rename would leave the suite invoking a command
/// that does not exist, and its rejection would surface as a timeout on an
/// assertion about something else entirely.
///
/// Both the shared harness and the spec are read, because C7 moved the
/// activation call: `activateMenuItem` is `helpers.js`'s now, since
/// `runDocumentAction` drives the whole suite through it, while `installed_menu`
/// is still read only by the menu describe. Which of the two files holds which
/// literal is not the contract — that a registered command name exists on the
/// WebdriverIO side at all is.
#[test]
fn the_menu_spec_invokes_the_registered_menu_seam_commands() {
    let spec = fs::read_to_string(repo_root().join("ui/e2e/specs/app-shell.e2e.js")).unwrap()
        + &fs::read_to_string(repo_root().join("ui/e2e/helpers.js")).unwrap();
    let main_rs = fs::read_to_string(repo_root().join("crates/arm-app/src/main.rs")).unwrap();

    for command in ["activate_menu_item", "installed_menu"] {
        assert!(
            spec.contains(&format!("'{command}'")),
            "app-shell.e2e.js or helpers.js must invoke the `{command}` command \
             by that exact name"
        );
        assert!(
            main_rs.contains(&format!("commands::{command}")),
            "`{command}` must be registered in main.rs's invoke_handler, or the \
             spec's call can never reach it"
        );
    }
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

/// Every checked-in `examples/*.json` fixture must actually load and validate
/// through the real production path — the same loader and validator the app
/// uses, against the same shipped `rules/` the app ships. Scans the directory
/// rather than naming files, so adding a future example needs no test change
/// (full-audit finding V20).
#[test]
fn every_example_save_parses_and_validates() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let examples_dir = repo_root().join("examples");
    let mut checked = 0;
    let mut entries: Vec<PathBuf> = fs::read_dir(&examples_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    entries.sort();

    let names = load_catalogue_names_from_dir(&rules_dir(), &ruleset).unwrap();
    for path in entries {
        let entity = load_entity_from_path(
            &path,
            arm_rules::DEFAULT_SAGA_YEAR,
            Some(&ruleset),
            Some(&names),
        )
        .unwrap_or_else(|e| panic!("{} failed to load: {e}", path.display()))
        .entity;
        let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
        assert!(
            result.is_valid(),
            "{} failed to validate: {:?}",
            path.display(),
            result.issues
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "examples/ must contain at least one *.json fixture"
    );
}

/// C8 / Trap 2. `examples/` is deliberately **not** uniform, and this test is why.
///
/// `companion_sample.json` had to be regenerated at the current schema (20, as of
/// F1): it is the fixture
/// `save_then_load_round_trips_with_byte_stable_canonical_json` compares a save
/// against, so a stale version there makes the round trip a non-identity. The other
/// three are left at schema 16 on purpose — regenerating all four would leave the
/// migration with no real checked-in input at all, only hand-written string literals
/// in the test modules, and the thing most worth proving about a migration is that it
/// works on a file somebody actually wrote.
#[test]
fn the_examples_keep_a_genuine_pre_migration_fixture() {
    let pre_migration = repo_root().join("examples/grog_sample.json");
    let raw = fs::read_to_string(&pre_migration).unwrap();
    assert!(
        raw.contains("\"schema_version\": 16"),
        "grog_sample.json is the checked-in pre-17 fixture; do not regenerate it"
    );
    assert!(
        !raw.contains("saga_year"),
        "a pre-17 save records no saga year — that is the point of the fixture"
    );

    // Opened with an Iberia default, it becomes an Iberia character: the year comes
    // from the caller, never from a constant.
    let (ruleset, names) = shipped_ruleset_and_names();
    let migrated = load_entity_from_path(&pre_migration, 1197, Some(&ruleset), Some(&names))
        .unwrap()
        .entity;
    assert_eq!(migrated.saga_year, 1197);
    assert_eq!(migrated.schema_version, arm_rules::SCHEMA_VERSION);

    // And the current fixture is genuinely current, so the round-trip test above is
    // comparing like with like.
    let current = fs::read_to_string(repo_root().join("examples/companion_sample.json")).unwrap();
    assert!(current.contains("\"schema_version\": 21"), "got {current}");
    assert!(current.contains("\"saga_year\": 1220"), "got {current}");
}

// --- CV2: arm-app reads the catalogue name files (design note § 5.3, § 10) ---

/// `load_catalogue_names_from_dir` must read BOTH shipped
/// `i18n/{en,de}/parameter_catalogue.json` files against the real, shipped
/// ruleset's catalogues, regardless of the app's active UI language.
#[test]
fn load_catalogue_names_reads_both_locales_from_the_shipped_rules_dir() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;

    let names = load_catalogue_names_from_dir(&rules_dir(), &ruleset).unwrap();

    let latin_names = names
        .get(&Id::new("language.latin"))
        .unwrap_or_else(|| panic!("language.latin must have resolved names, got {names:?}"));
    assert!(latin_names.iter().any(|n| n == "Latin"));
    assert!(latin_names.iter().any(|n| n == "Latein"));
}

/// The portable Linux layout: `pick_rules_dir` resolves the real directory
/// (skipping the nonexistent system resource path), and the catalogue-names
/// loader must work identically against whatever directory that resolves to,
/// not just a directly-named one.
#[test]
fn load_catalogue_names_works_through_the_portable_fallback_directory() {
    let empty = tempfile::tempdir().unwrap();
    let resolved = pick_rules_dir(&[empty.path().to_path_buf(), rules_dir()])
        .expect("the real rules dir must be picked over the empty candidate");
    let ruleset = load_ruleset_from_dir(&resolved, "en").unwrap().ruleset;

    let names = load_catalogue_names_from_dir(&resolved, &ruleset).unwrap();

    assert!(
        names.contains_key(&Id::new("organization.order_of_hermes")),
        "got {names:?}"
    );
}

// --- CV7: catalogue value names reach the FRONTEND's own i18n map (design
// § 2.3/§ 6.4, § 10) -----------------------------------------------------

/// `load_catalogue_names_from_dir` (above) is a migration-only, both-locales
/// index — it never reaches the frontend. The picker instead needs a
/// catalogue id to resolve through the SAME `LocalizedRuleset.i18n` map every
/// other id already does, in the ACTIVE language alone, so `displayName`
/// needs no second code path. Red-checkpoint protocol, phase 1:
/// `merge_catalogue_display_names` is a no-op stub, so this fails until phase
/// 2 wires it in.
#[test]
fn load_ruleset_localizes_catalogue_value_names_in_english() {
    let localized = load_ruleset_from_dir(&rules_dir(), "en").unwrap();

    assert_eq!(
        localized
            .i18n
            .get(&Id::new("language.latin"))
            .map(|e| e.name.as_str()),
        Some("Latin"),
        "expected the English catalogue name merged into the frontend's own i18n map"
    );
}

/// Same fixture, German — proving the merge is language-aware, not a single
/// hardcoded name.
#[test]
fn load_ruleset_localizes_catalogue_value_names_in_german() {
    let localized = load_ruleset_from_dir(&rules_dir(), "de").unwrap();

    assert_eq!(
        localized
            .i18n
            .get(&Id::new("language.latin"))
            .map(|e| e.name.as_str()),
        Some("Latein"),
        "expected the German catalogue name merged into the frontend's own i18n map"
    );
}

// --- CV7: the removal flow's engine operation (design § 5.5, § 10) ---------

/// The Virtue/Flaw removal flow calls `unlink_ability_parameters` BEFORE
/// removing the source selection, so a bought Ability `Linked` to it keeps
/// its last resolvable value as free text rather than going dangling. Red-
/// checkpoint protocol, phase 1: `unlink_ability_parameters_loaded` is a
/// clone-only stub, so this fails until phase 2 calls the real engine
/// operation.
#[test]
fn unlink_ability_parameters_converts_a_linked_ability_score_to_text() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut entity = sample_entity();
    entity.selections = vec![Selection::with_params(
        Id::new("virtue.craft_guild_training"),
        BTreeMap::from([("guild".to_string(), Id::new("Smiths' Guild of Verdi"))]),
    )];
    let mut lore = AbilityScore::new(Id::new("ability.organization_lore"), 1);
    lore.parameter = Some(AbilityParameterValue::Linked {
        item: Id::new("virtue.craft_guild_training"),
        param: "guild".to_string(),
    });
    entity.ability_scores = vec![lore];

    let updated = unlink_ability_parameters_loaded(
        &entity,
        &ruleset,
        &Id::new("virtue.craft_guild_training"),
    );

    assert_eq!(
        updated.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("Smiths' Guild of Verdi")),
        "the linked Ability score must convert to Text holding the last resolvable value"
    );
}

/// D79 (`docs/vf-audit/decisions.md`): the frontend must not reconstruct a
/// per-spell Casting Total client-side from the three grid figures (base /
/// within-focus / within-potent-field) — `halve(a) + halve(b) != halve(a + b)`
/// in general, so summing two already-halved grid deltas is wrong whenever the
/// cell is Deficient. `derived_totals` instead carries one
/// `spell_casting_totals` entry per `entity.spells` row, index-aligned,
/// computed by the engine's own `spell_casting_total` (which already combines
/// both markers correctly). This is the IPC boundary's own test: the real
/// `derived_totals` Tauri command (`commands.rs::derived_totals`) is a thin
/// pass-through to `arm_rules::derived_totals`, exercised directly here
/// against the real shipped ruleset.
#[test]
fn derived_totals_spell_casting_totals_are_index_aligned_with_entity_spells() {
    let ruleset = load_ruleset_from_dir(&rules_dir(), "en").unwrap().ruleset;
    let mut magus = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("magus"),
        arm_rules::RulesetRef::new(Id::new(RULESET_ID), RULESET_VERSION),
    );
    // Creo 12 / Ignem 15, nothing else: base Casting Total 27 for
    // spell.pilum_of_fire (the same baseline the engine's own
    // `d79_potent_magic.rs` suite uses).
    magus.art_scores = vec![
        ArtScore::new(Id::new("art.creo"), 12),
        ArtScore::new(Id::new("art.ignem"), 15),
    ];
    magus.selections = vec![Selection::with_params(
        Id::new("virtue.potent_magic_major"),
        BTreeMap::from([("field".to_string(), Id::new("fire"))]),
    )];
    let mut unmarked = SpellSelection::new(Id::new("spell.pilum_of_fire"));
    unmarked.within_potent_field = false;
    let mut marked = SpellSelection::new(Id::new("spell.pilum_of_fire"));
    marked.within_potent_field = true;
    magus.spells = vec![unmarked, marked];

    let totals = derived_totals(&magus, &ruleset);
    assert_eq!(
        totals.spell_casting_totals.len(),
        2,
        "one entry per entity.spells row, index-aligned: {:?}",
        totals.spell_casting_totals
    );
    let unmarked_total =
        totals.spell_casting_totals[0].expect("spell.pilum_of_fire is in the shipped catalogue");
    let marked_total =
        totals.spell_casting_totals[1].expect("spell.pilum_of_fire is in the shipped catalogue");
    assert_eq!(
        marked_total - unmarked_total,
        6,
        "row 1 (within_potent_field: true) must add Major Potent Magic's +6 over row 0 \
         (marked {marked_total}, unmarked {unmarked_total})"
    );
}
