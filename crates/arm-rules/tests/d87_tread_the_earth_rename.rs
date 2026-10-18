//! D87 (Norbert, 2026-10-10): the book's own heading for this spell reads
//! "Sense the Feet That **Tread** the Earth" (ArMDE:15412, confirmed by reading
//! the file directly) — the catalogue instead carried a scanno, "**Thread**",
//! and so does the id: `spell.sense_the_feet_that_thread_the_earth` must
//! become `spell.sense_the_feet_that_tread_the_earth`. Both the English display
//! name and the id are corrected; the German name was already right
//! ("Die Schritte auf der Erde spüren" — `Ars Magica Definitive Edition
//! Basisregeln.md:15410`, matching `zauber-nach-form.md:395`), so only its key
//! moves.
//!
//! `source.anchor` does **NOT** change: the P8-3 re-sync already relocated it
//! to `sense-the-feet-that-tread-the-earth` ahead of this rename (recorded as
//! an `ANCHOR_OVERRIDES` exception in `source_relocation.rs`, because the old
//! heading's own wording changed, not just its position) — that anchor already
//! matches the current heading, so this rename touches only the id/name layer.
//!
//! Scope, by contrast with the `virtue.rard` -> `virtue.bard` precedent
//! (`x9b_bard_rename.rs`): a spell id lives in exactly ONE place on [`Entity`]
//! — `Entity::spells[].spell` (`types.rs::SpellSelection::spell`) — never in a
//! `Selection`, a resolved-pick map, or a `Linked` ability parameter (those are
//! all point-item ids; nothing in the shipped data or the type system lets a
//! spell declare a parameter or be targeted by one). So the migration fold here
//! walks one field, not four.
//!
//! This file is the RED checkpoint: everything below fails for the right
//! reason until the catalogue data is renamed and the migration fold lands.

use std::path::{Path, PathBuf};

use arm_rules::migration::{SCHEMA_VERSION, load_entity_migrating};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::DEFAULT_SAGA_YEAR;

/// `crates/arm-rules` (this crate's manifest dir) -> the repository root.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Renders `path` relative to [`repo_root`] for a failure message.
fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .unwrap_or(path)
        .display()
        .to_string()
}

/// Every file under `dir` (recursively) whose extension is one of `extensions`.
fn source_files_in(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            source_files_in(&path, extensions, out);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| extensions.contains(&ext))
        {
            out.push(path);
        }
    }
}

const EN_SPELLS: &str = include_str!("../../../rules/i18n/en/spells.json");
const DE_SPELLS: &str = include_str!("../../../rules/i18n/de/spells.json");

fn shipped_ruleset() -> Ruleset {
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
        aging: Some(include_str!("../../../rules/core/aging.json")),
        parameter_catalogues: Some(include_str!(
            "../../../rules/core/parameter_catalogues.json"
        )),
        ..RulesetSources::default()
    })
    .expect("shipped core ruleset loads")
}

/// A ruleset carrying no catalogued data at all — the migration tests below
/// exercise [`load_entity_migrating`]'s raw-JSON fold, not the shipped
/// catalogue.
fn empty_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        point_items: "[]",
        type_profiles: "[]",
        ..RulesetSources::default()
    })
    .expect("an empty ruleset loads")
}

fn empty_catalogue_names() -> std::collections::BTreeMap<Id, Vec<String>> {
    std::collections::BTreeMap::new()
}

const OLD_ID: &str = "spell.sense_the_feet_that_thread_the_earth";
const NEW_ID: &str = "spell.sense_the_feet_that_tread_the_earth";

// --- 1. The data: the new id exists with the corrected names, the old one is gone ---

#[test]
fn spell_tread_exists_with_correct_en_and_de_names() {
    let rs = shipped_ruleset();
    let old_id = Id::new(OLD_ID);
    let new_id = Id::new(NEW_ID);

    assert!(
        rs.spell(&new_id).is_some(),
        "spell.sense_the_feet_that_tread_the_earth must exist in the shipped \
         catalogue (D87): the id tracks the book's own heading, not the scanno"
    );
    assert!(
        rs.spell(&old_id).is_none(),
        "spell.sense_the_feet_that_thread_the_earth must no longer exist once \
         D87 lands — the id is renamed, not duplicated"
    );

    let en = LocalizedRuleset::new(shipped_ruleset(), EN_SPELLS).expect("EN i18n loads");
    assert_eq!(
        en.display_name(&new_id),
        Some("Sense the Feet That Tread the Earth"),
        "the English name is the scanno \"Thread\" today; the book's own \
         heading (ArMDE:15412) reads \"Tread\""
    );

    let de = LocalizedRuleset::new(shipped_ruleset(), DE_SPELLS).expect("DE i18n loads");
    assert_eq!(
        de.display_name(&new_id),
        Some("Die Schritte auf der Erde spüren"),
        "the German name was already correct under the old id (it has no \
         thread/tread ambiguity); it must survive the rename unchanged under \
         the new id"
    );
}

/// The anchor was already relocated ahead of this rename (P8-3's
/// `ANCHOR_OVERRIDES` exception in `source_relocation.rs`), so it does not move
/// again here — only the id/name layer changes.
#[test]
fn spell_tread_keeps_its_already_relocated_anchor() {
    let rs = shipped_ruleset();
    let spell = rs.spell(&Id::new(NEW_ID)).expect(
        "the renamed spell must exist (see spell_tread_exists_with_correct_en_and_de_names)",
    );
    let source = spell
        .source
        .as_ref()
        .expect("the spell must still carry a source reference");

    assert_eq!(
        source.anchor, "sense-the-feet-that-tread-the-earth",
        "the anchor was already relocated by the P8-3 re-sync, ahead of this \
         id rename; it must not change again"
    );
    assert_eq!(
        source.lines,
        LineRange::new(15412, 15415),
        "the cited range is unaffected by the id/name rename"
    );
    assert_eq!(
        source.file, "Ars Magica - Definitive Edition (Core Rules).md",
        "the source file is unaffected by the id/name rename"
    );
}

#[test]
fn german_source_anchor_key_moves_to_the_new_spell_id() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/de/source_anchors.json"))
            .expect("source_anchors.json is valid JSON");
    let obj = raw
        .as_object()
        .expect("source_anchors.json is a JSON object");

    assert!(
        !obj.contains_key(OLD_ID),
        "rules/i18n/de/source_anchors.json must not key the old id any more"
    );
    let entry = obj
        .get(NEW_ID)
        .unwrap_or_else(|| panic!("rules/i18n/de/source_anchors.json must key the new id"));
    assert_eq!(
        entry.get("anchor").and_then(|v| v.as_str()),
        Some("die-schritte-auf-der-erde-spüren"),
        "the German anchor is unaffected by the id rename"
    );
    assert_eq!(
        entry.get("file").and_then(|v| v.as_str()),
        Some("Ars Magica Definitive Edition Basisregeln.md")
    );
}

// --- 2. No stale reference to the old id survives under rules/ -------------

#[test]
fn no_file_under_rules_still_names_the_old_spell_id() {
    let rules_dir = repo_root().join("rules");
    let mut files = Vec::new();
    source_files_in(&rules_dir, &["json", "md"], &mut files);
    assert!(
        files.len() > 50,
        "the walk must have found a plausible number of files under rules/ \
         ({} found) — otherwise this test would pass vacuously",
        files.len()
    );

    let mut offenders = Vec::new();
    for path in &files {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if content.contains(OLD_ID) {
            offenders.push(relative(path));
        }
    }
    assert!(
        offenders.is_empty(),
        "{OLD_ID} must not appear anywhere under rules/ once D87 lands (id \
         renamed to {NEW_ID}); still cited in: {offenders:?}"
    );
}

/// `examples/` holds the sample saves exercised by the UI/e2e fixtures. None
/// reference the old id today (checked directly), so there is nothing to
/// migrate there — this test locks that in rather than asserting a rename
/// that has no carrier.
#[test]
fn no_example_save_names_the_old_spell_id() {
    let examples_dir = repo_root().join("examples");
    let mut files = Vec::new();
    source_files_in(&examples_dir, &["json", "armc", "armcov"], &mut files);
    assert!(!files.is_empty(), "examples/ must contain sample saves");
    for path in &files {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            !content.contains(OLD_ID),
            "{} must not reference {OLD_ID}",
            relative(path)
        );
    }
}

// --- 3. Migration: an old save's known spell becomes the new id ------------

/// The target this bump had to reach was 24, one past N4b's 23. Row 56 has
/// since moved the constant to 25 for the lab-year counts, so this tracks the
/// current value; D87's own contribution is still the 23 -> 24 step.
#[test]
fn schema_version_target_is_one_past_todays() {
    assert_eq!(
        SCHEMA_VERSION, 25,
        "D87 and row 56 bump SCHEMA_VERSION past N4b's 23; today's build is \
         at {SCHEMA_VERSION}"
    );
}

/// A pre-D87 save (schema 23, the version every build up to and including this
/// one writes) holding the old spell id as a known spell must load as the new
/// id, with the version stamped to the new one.
#[test]
fn a_known_spell_migrates_to_the_new_id() {
    let old = format!(
        r#"{{
      "schema_version": 23,
      "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "pool",
      "saga_year": 1197,
      "spells": [{{ "spell": "{OLD_ID}" }}]
    }}"#
    );
    let loaded = load_entity_migrating(
        &old,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    )
    .expect("a schema-23 save with the old spell id must still load");

    assert_eq!(
        loaded.entity.spells,
        vec![SpellSelection::new(Id::new(NEW_ID))],
        "a known spell using the old id must migrate to the new id"
    );
    assert_eq!(
        loaded.entity.schema_version, SCHEMA_VERSION,
        "the id rename is a genuine value move, so the version must be \
         stamped to the new SCHEMA_VERSION, exactly like the virtue.rard -> \
         virtue.bard fold"
    );
}

/// The rename must preserve every other field of the spell selection — level,
/// mastery, parameter, mastery abilities, within-focus/within-potent-field —
/// not just the `spell` id itself.
#[test]
fn migrating_the_spell_id_keeps_its_other_fields() {
    let old = format!(
        r#"{{
      "schema_version": 23,
      "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "pool",
      "saga_year": 1197,
      "spells": [{{
        "spell": "{OLD_ID}",
        "mastery": 2,
        "mastery_abilities": ["spell_mastery_ability.quick_casting"],
        "within_focus": true
      }}]
    }}"#
    );
    let loaded = load_entity_migrating(
        &old,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    )
    .expect("a schema-23 save with the old spell id must still load");

    let spell = &loaded.entity.spells[0];
    assert_eq!(spell.spell, Id::new(NEW_ID));
    assert_eq!(spell.mastery, Some(2));
    assert_eq!(
        spell.mastery_abilities,
        vec![Id::new("spell_mastery_ability.quick_casting")]
    );
    assert!(spell.within_focus);
}

/// A save written by a build at D87's version (24: id already the new one) must
/// load with NO further change — no fold fires, so its version is not
/// re-stamped either.
#[test]
fn a_save_already_at_d87s_version_with_the_new_id_is_untouched() {
    let current = format!(
        r#"{{
      "schema_version": 24,
      "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "pool",
      "saga_year": 1197,
      "spells": [{{ "spell": "{NEW_ID}" }}]
    }}"#
    );
    let result = load_entity_migrating(
        &current,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    );
    assert!(
        result.is_ok(),
        "a save written at D87's version (24) must be accepted by this build \
         (SCHEMA_VERSION {SCHEMA_VERSION}): {result:?}"
    );
    let entity = result.unwrap().entity;
    assert_eq!(
        entity.spells,
        vec![SpellSelection::new(Id::new(NEW_ID))],
        "a save already holding the new id must be left exactly as written"
    );
    assert_eq!(
        entity.schema_version, 24,
        "the version must not be re-stamped"
    );
}

/// The standing invariant must still hold one past the NEW target version.
#[test]
fn a_save_one_past_the_new_target_version_is_still_refused() {
    let one_past = SCHEMA_VERSION + 1;
    let future = format!(
        r#"{{
      "schema_version": {one_past},
      "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "pool",
      "saga_year": 1197
    }}"#
    );
    let result = load_entity_migrating(
        &future,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    );
    assert!(
        result.is_err(),
        "schema_version {one_past} is one past the current target and must be \
         refused, not half-migrated"
    );
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains(&one_past.to_string()),
        "the refusal must name the version it read: {message}"
    );
}

/// Round-trip fidelity: a migrated entity re-serializes, and reloading that
/// output is byte-for-byte the same.
#[test]
fn a_migrated_spell_survives_a_save_and_reload_byte_identical() {
    let old = format!(
        r#"{{
      "schema_version": 23,
      "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
      "entity_kind": "character",
      "type_id": "magus",
      "ability_funding": "pool",
      "saga_year": 1197,
      "name": "Fiona",
      "spells": [{{ "spell": "{OLD_ID}" }}]
    }}"#
    );
    let migrated = load_entity_migrating(
        &old,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    )
    .expect("a schema-23 save with the old spell id must still load")
    .entity;
    assert_eq!(
        migrated.spells,
        vec![SpellSelection::new(Id::new(NEW_ID))],
        "see a_known_spell_migrates_to_the_new_id for the same assertion in \
         isolation"
    );

    let saved = serde_json::to_string_pretty(&migrated).unwrap();
    assert!(
        saved.contains(&format!("\"spell\": \"{NEW_ID}\"")),
        "{saved}"
    );
    assert!(!saved.contains(OLD_ID), "{saved}");

    let reloaded = load_entity_migrating(
        &saved,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    )
    .unwrap()
    .entity;
    assert_eq!(
        reloaded, migrated,
        "nothing was lost or invented on the second pass"
    );
    assert_eq!(
        serde_json::to_string_pretty(&reloaded).unwrap(),
        saved,
        "the second write must be byte-identical to the first"
    );
}

// --- 4. UI-side reference sweep (informational — none found today) ---------

#[test]
fn no_ui_source_file_names_the_old_spell_id() {
    let ui_src = repo_root().join("ui/src");
    let mut files = Vec::new();
    source_files_in(&ui_src, &["ts", "svelte", "css", "json"], &mut files);
    assert!(!files.is_empty(), "ui/src must contain source files");
    for path in &files {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            !content.contains(OLD_ID),
            "{} must not reference the old spell id",
            relative(path)
        );
    }
}
