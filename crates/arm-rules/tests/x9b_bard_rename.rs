//! X9b (`docs/vf-audit/corrections.md` F-16, `docs/vf-audit/phase-2-plan.md`
//! row X9b) — `virtue.rard` is a scanno: ArMDE:3476 prints the heading
//! `#### Rard`, but the book's own index (`ArMDE:24289`, `| Bard (Virtue) |
//! [71](#bard) |`) and `rules/source/de/translation-tables/tugenden-fehler.md:243`
//! (`| Bard | Barde | ... |`) both read "Bard". The English `name` is wrong
//! and so is the **id**: `virtue.rard` must become `virtue.bard`.
//!
//! `source.anchor` does **NOT** change — it stays `"rard"` because the
//! *heading itself* is still spelled `#### Rard` (confirmed: ArMDE:3476), and
//! Guard A (`rules_source_provenance.rs`) compares the anchor against the
//! heading, not the corrected name. Name and id track the truth; the anchor
//! tracks the book. The German heading has no scanno (`Ars Magica Definitive
//! Edition Basisregeln.md:3476` already reads `#### Barde`), so the German
//! anchor (`rules/i18n/de/source_anchors.json`, currently keyed
//! `"virtue.rard"`) already holds the correct anchor `"barde"` — only its
//! **key** needs to move to `"virtue.bard"` alongside the id rename.
//!
//! This file is the PHASE-1 red checkpoint: it asserts the rename and its
//! migration, all still red because phase 2 (the data edit + the migration
//! fold body) has not landed. No `rules/*.json` edit and no migration fold
//! happen in this slice — seeing these tests fail for the right reason is the
//! point.
//!
//! Every place a V/F id can appear on [`Entity`] is covered: the bought
//! `selections` list, the three resolved-pick maps (`house_choices`,
//! `mythic_choices`, `warping_choices` — exactly the containers
//! `migration.rs::fold_legacy_being_params` already walks for the same
//! reason), and an `AbilityScore.parameter`'s `Linked.item` (the "declaring
//! item" a parameterized Ability can point at, `migration.rs::
//! fold_dangling_and_ambiguous_links`'s territory). `virtue.rard` grants only
//! a Local Reputation and declares no parameter of its own, so nothing in the
//! shipped data exercises the `Linked` or param-`Item`-domain cases today —
//! but a hand-edited save is this project's declared hostile-input surface
//! (`CLAUDE.md` → "This is a DESKTOP APPLICATION"), so the migration must
//! still rename the id wherever it is found, not only where today's data
//! happens to put it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use arm_rules::migration::{SCHEMA_VERSION, load_entity_migrating};
use arm_rules::ruleset::{LocalizedRuleset, Ruleset, RulesetSources};
use arm_rules::types::*;
use arm_rules::validation::DEFAULT_SAGA_YEAR;

/// `crates/arm-rules` (this crate's manifest dir) -> the repository root.
/// Duplicated from `citation_support::repo_root` rather than sharing it via
/// `mod citation_support;`: that module's dead-code lint only balances out
/// because its two existing consumers (`rulebook_citations.rs`,
/// `source_citations.rs`) between them use every function it exports — a
/// third consumer using only a subset makes `cargo clippy --all-targets`
/// flag the rest as dead code **in this binary's own compilation unit**
/// (each integration test is a separate crate). Three small helpers are
/// cheaper to duplicate than to reshape shared infrastructure for.
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

const EN_VF: &str = include_str!("../../../rules/i18n/en/virtues_flaws.json");
const DE_VF: &str = include_str!("../../../rules/i18n/de/virtues_flaws.json");

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

/// A ruleset carrying no catalogued point items at all — the migration tests
/// below exercise [`load_entity_migrating`]'s raw-JSON folds, not the shipped
/// catalogue, and need no real data to do it (same shape as
/// `migration.rs::tests::empty_ruleset`, duplicated here because integration
/// test binaries cannot share a sibling binary's private helpers).
fn empty_ruleset() -> Ruleset {
    Ruleset::from_sources(RulesetSources {
        point_items: "[]",
        type_profiles: "[]",
        ..RulesetSources::default()
    })
    .expect("an empty ruleset loads")
}

fn empty_catalogue_names() -> BTreeMap<Id, Vec<String>> {
    BTreeMap::new()
}

// --- 1. The data: virtue.bard exists, virtue.rard does not, anchor unmoved ---

#[test]
fn virtue_bard_exists_with_correct_en_and_de_names() {
    let rs = shipped_ruleset();
    let old_id = Id::new("virtue.rard");
    let new_id = Id::new("virtue.bard");

    assert!(
        rs.item(&new_id).is_some(),
        "virtue.bard must exist in the shipped catalogue (F-16): the id tracks \
         the corrected name, not the scanno"
    );
    assert!(
        rs.item(&old_id).is_none(),
        "virtue.rard must no longer exist once F-16 lands — the id is renamed, \
         not duplicated"
    );

    let en = LocalizedRuleset::new(shipped_ruleset(), EN_VF).expect("EN i18n loads");
    assert_eq!(
        en.display_name(&new_id),
        Some("Bard"),
        "the English name is the scanno \"Rard\" today (ArMDE:3476); the book's \
         own index (ArMDE:24289) reads \"Bard\""
    );

    let de = LocalizedRuleset::new(shipped_ruleset(), DE_VF).expect("DE i18n loads");
    assert_eq!(
        de.display_name(&new_id),
        Some("Barde"),
        "the German name was already correct under the old id (tugenden-fehler.md:243); \
         it must survive the rename unchanged under virtue.bard"
    );
}

#[test]
fn virtue_bard_keeps_the_rard_anchor_because_the_heading_still_misspells_it() {
    let rs = shipped_ruleset();
    let item = rs
        .item(&Id::new("virtue.bard"))
        .expect("virtue.bard must exist (see virtue_bard_exists_with_correct_en_and_de_names)");
    let source = item
        .source
        .as_ref()
        .expect("virtue.bard must still carry a source reference");

    assert_eq!(
        source.anchor, "rard",
        "source.anchor must stay \"rard\": the heading is still '#### Rard' \
         (ArMDE:3476), and Guard A compares the anchor against the heading, not \
         the corrected name — do NOT change the anchor"
    );
    assert_eq!(
        source.lines,
        LineRange::new(3476, 3478),
        "the cited range is unaffected by the id/name rename"
    );
    assert_eq!(
        source.file, "Ars Magica - Definitive Edition (Core Rules).md",
        "the source file is unaffected by the id/name rename"
    );
}

#[test]
fn german_source_anchor_key_moves_to_virtue_bard() {
    let raw: serde_json::Value =
        serde_json::from_str(include_str!("../../../rules/i18n/de/source_anchors.json"))
            .expect("source_anchors.json is valid JSON");
    let obj = raw
        .as_object()
        .expect("source_anchors.json is a JSON object");

    assert!(
        !obj.contains_key("virtue.rard"),
        "rules/i18n/de/source_anchors.json must not key the old id any more"
    );
    let entry = obj
        .get("virtue.bard")
        .unwrap_or_else(|| panic!("rules/i18n/de/source_anchors.json must key the new id"));
    assert_eq!(
        entry.get("anchor").and_then(|v| v.as_str()),
        Some("barde"),
        "the German anchor is unaffected by the id rename: the German heading was \
         never misspelled (Basisregeln.md:3476 already reads '#### Barde')"
    );
    assert_eq!(
        entry.get("file").and_then(|v| v.as_str()),
        Some("Ars Magica Definitive Edition Basisregeln.md")
    );
}

// --- 2. No stale reference to the old id survives under rules/ -------------

#[test]
fn no_file_under_rules_still_names_virtue_rard() {
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
        if content.contains("virtue.rard") {
            offenders.push(relative(path));
        }
    }
    assert!(
        offenders.is_empty(),
        "virtue.rard must not appear anywhere under rules/ once F-16 lands \
         (id renamed to virtue.bard); still cited in: {offenders:?}"
    );
}

/// `examples/` holds the sample saves exercised by the UI/e2e fixtures.
/// None reference `virtue.rard` today (checked directly), so there is nothing
/// to migrate there — this test locks that in rather than asserting a rename
/// that has no carrier.
#[test]
fn no_example_save_names_virtue_rard() {
    let examples_dir = repo_root().join("examples");
    let mut files = Vec::new();
    source_files_in(&examples_dir, &["json"], &mut files);
    assert!(!files.is_empty(), "examples/ must contain sample saves");
    for path in &files {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            !content.contains("virtue.rard"),
            "{} must not reference virtue.rard",
            relative(path)
        );
    }
}

// --- 3. Migration: an old save's virtue.rard selection becomes virtue.bard -

/// The target this bump had to reach was 21 (X9b's virtue.rard -> virtue.bard
/// id rename, phase-2-plan.md § 1: "X9b ... bump"). L1b has since moved the
/// constant to 22 for the Dead/Living Language move, so this tracks the current
/// value; X9b's own contribution is still the 20 -> 21 step.
#[test]
fn schema_version_target_is_one_past_todays() {
    assert_eq!(
        SCHEMA_VERSION, 22,
        "L1b bumps SCHEMA_VERSION past X9b's 21 for the language move; today's build \
         is still at {SCHEMA_VERSION}"
    );
}

/// A pre-X9b save (schema 20, the version every build up to and including
/// this one writes) holding a BOUGHT `virtue.rard` selection must load as
/// `virtue.bard`, with the version stamped to the new one.
#[test]
fn a_bought_virtue_rard_selection_migrates_to_virtue_bard() {
    let old = r#"{
      "schema_version": 20,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "companion",
      "ability_funding": "pool",
      "saga_year": 1197,
      "selections": [{ "ref": "virtue.rard" }]
    }"#;
    let loaded = load_entity_migrating(
        old,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    )
    .expect("a schema-20 save with a bought virtue.rard must still load");

    assert_eq!(
        loaded.entity.selections,
        vec![Selection::new(Id::new("virtue.bard"))],
        "a bought virtue.rard selection must migrate to virtue.bard"
    );
    assert_eq!(
        loaded.entity.schema_version, SCHEMA_VERSION,
        "the id rename is a genuine value move, so the version must be stamped \
         to the new SCHEMA_VERSION, exactly like every other meaning-changing \
         fold in load_entity_migrating"
    );
}

/// The same rename, but via each of the three resolved-pick maps
/// (`house_choices`, `mythic_choices`, `warping_choices`) instead of the
/// bought `selections` list — the three containers
/// `fold_legacy_being_params` already walks for the same structural reason.
#[test]
fn a_granted_virtue_rard_choice_migrates_in_every_resolved_pick_map() {
    for field in ["house_choices", "mythic_choices", "warping_choices"] {
        let old = format!(
            r#"{{
              "schema_version": 20,
              "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
              "entity_kind": "character",
              "type_id": "companion",
              "ability_funding": "pool",
              "saga_year": 1197,
              "{field}": {{ "slot": {{ "ref": "virtue.rard" }} }}
            }}"#
        );
        let loaded = load_entity_migrating(
            &old,
            DEFAULT_SAGA_YEAR,
            &empty_ruleset(),
            &empty_catalogue_names(),
        )
        .unwrap_or_else(|e| {
            panic!("a schema-20 save with {field}.slot = virtue.rard must still load: {e}")
        });

        let map = match field {
            "house_choices" => &loaded.entity.house_choices,
            "mythic_choices" => &loaded.entity.mythic_choices,
            "warping_choices" => &loaded.entity.warping_choices,
            _ => unreachable!(),
        };
        assert_eq!(
            map.get("slot").map(|s| s.item_ref.clone()),
            Some(Id::new("virtue.bard")),
            "{field}'s \"slot\" pick must migrate virtue.rard -> virtue.bard"
        );
        assert_eq!(
            loaded.entity.schema_version, SCHEMA_VERSION,
            "{field}'s rename must stamp the new SCHEMA_VERSION"
        );
    }
}

/// `AbilityParameterValue::Linked { item, .. }` names a "declaring item" by
/// id (design CV5) — the one remaining place on `Entity` a V/F id can hide,
/// reached via an `AbilityScore`'s own parameter rather than a `Selection`.
/// No shipped data links to `virtue.rard` today (it declares no parameter of
/// its own), but a hand-edited save is this project's hostile-input surface,
/// so a stale `Linked.item` must still be renamed.
///
/// The bought `virtue.rard` selection carries a synthetic `guild` param purely
/// so `fold_dangling_and_ambiguous_links` (which runs right after the rename
/// fold, in the same load) resolves this Linked value to `Resolved(Some(_))`
/// and leaves it as `Linked` — a target with no value of its own resolves to
/// `Resolved(None)`, which that fold treats exactly like a dangling link and
/// converts to `Text`, which is correct separate engine behavior this test is
/// not about.
#[test]
fn a_linked_ability_parameter_naming_virtue_rard_migrates_too() {
    let old = r#"{
      "schema_version": 20,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "companion",
      "ability_funding": "pool",
      "saga_year": 1197,
      "selections": [{ "ref": "virtue.rard", "params": { "guild": "Verditius Smiths" } }],
      "ability_scores": [
        { "ability": "ability.organization_lore", "score": 1,
          "parameter": { "item": "virtue.rard", "param": "guild" } }
      ]
    }"#;
    let loaded = load_entity_migrating(
        old,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    )
    .expect("a schema-20 save with a Linked.item of virtue.rard must still load");

    let parameter = loaded
        .entity
        .ability_scores
        .first()
        .and_then(|score| score.parameter.clone())
        .expect("the ability score must keep its parameter");
    assert_eq!(
        parameter,
        AbilityParameterValue::Linked {
            item: Id::new("virtue.bard"),
            param: "guild".to_string(),
        },
        "a Linked parameter naming virtue.rard as its declaring item must be \
         renamed to virtue.bard, same as the selection itself"
    );
    assert_eq!(loaded.entity.schema_version, SCHEMA_VERSION);
}

/// A save already written by a build at the new version (id already
/// `virtue.bard`, `schema_version` already the new target) must load with NO
/// further change — the fold is dispatch-on-presence of the OLD id, so
/// finding none is a no-op, exactly like every other fold in
/// `load_entity_migrating`. Phrased as `assert!` on the `Result`/equality
/// throughout (never a bare `.unwrap()`) so today's failure — this save's
/// claimed version (21) is refused because `SCHEMA_VERSION` has not been
/// bumped yet — reports as a clear assertion failure, not a stray panic.
#[test]
fn a_save_already_at_the_new_version_with_virtue_bard_is_untouched() {
    let current = r#"{
      "schema_version": 21,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "companion",
      "ability_funding": "pool",
      "saga_year": 1197,
      "selections": [{ "ref": "virtue.bard" }]
    }"#;
    let result = load_entity_migrating(
        current,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    );
    assert!(
        result.is_ok(),
        "a save already claiming the new SCHEMA_VERSION (21) must be accepted \
         once X9b bumps the constant; today it is refused because the build's \
         own SCHEMA_VERSION is still {SCHEMA_VERSION}: {result:?}"
    );
    let entity = result.unwrap().entity;
    assert_eq!(
        entity.selections,
        vec![Selection::new(Id::new("virtue.bard"))],
        "a save already holding virtue.bard must be left exactly as written"
    );
    assert_eq!(
        entity.schema_version, 21,
        "the version must not be re-stamped"
    );
}

/// The standing invariant (`migration.rs::
/// a_save_from_a_future_schema_is_refused_rather_than_half_migrated`) must
/// still hold one past the NEW target version: a save from 22 is refused
/// exactly as a save from `SCHEMA_VERSION + 1` is refused today. This is
/// "green on arrival" (the refusal already holds against any number above
/// today's 20) and is kept as a lock against the bump silently widening the
/// accepted range further than intended. Written against `SCHEMA_VERSION + 1`
/// since L1b's 21 -> 22 bump made the literal 22 a version this build reads.
#[test]
fn a_save_one_past_the_new_target_version_is_still_refused() {
    let one_past = SCHEMA_VERSION + 1;
    let future = format!(
        r#"{{
      "schema_version": {one_past},
      "ruleset": {{ "id": "arm5-core", "version": "2024.1" }},
      "entity_kind": "character",
      "type_id": "companion",
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
        "schema_version {one_past} is one past the current target and must be refused, \
         not half-migrated"
    );
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains(&one_past.to_string()),
        "the refusal must name the version it read: {message}"
    );
}

/// Round-trip fidelity (`CLAUDE.md` rates this top severity): a migrated
/// entity re-serializes, and reloading that output is byte-for-byte the
/// same — no further drift on a second pass. Mirrors `migration.rs::
/// a_migrated_saga_year_survives_a_save_and_reload_unchanged`.
#[test]
fn a_migrated_bard_selection_survives_a_save_and_reload_byte_identical() {
    let old = r#"{
      "schema_version": 20,
      "ruleset": { "id": "arm5-core", "version": "2024.1" },
      "entity_kind": "character",
      "type_id": "companion",
      "ability_funding": "pool",
      "saga_year": 1197,
      "name": "Fiona",
      "selections": [{ "ref": "virtue.rard" }]
    }"#;
    let migrated = load_entity_migrating(
        old,
        DEFAULT_SAGA_YEAR,
        &empty_ruleset(),
        &empty_catalogue_names(),
    )
    .expect("a schema-20 save with a bought virtue.rard must still load")
    .entity;
    assert_eq!(
        migrated.selections,
        vec![Selection::new(Id::new("virtue.bard"))],
        "see a_bought_virtue_rard_selection_migrates_to_virtue_bard for the same \
         assertion in isolation"
    );

    let saved = serde_json::to_string_pretty(&migrated).unwrap();
    assert!(saved.contains(r#""ref": "virtue.bard""#), "{saved}");
    assert!(!saved.contains("virtue.rard"), "{saved}");

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

/// `ui/src` carries no reference to the scanno today (checked directly with
/// `grep -rni rard ui/src`, zero hits). This test locks that in; if a future
/// change adds a hardcoded reference it would be a hardcoded-id/string
/// violation independent of X9b, and this test would catch it either way.
#[test]
fn no_ui_source_file_names_rard() {
    let ui_src = repo_root().join("ui/src");
    let mut files = Vec::new();
    source_files_in(&ui_src, &["ts", "svelte", "css", "json"], &mut files);
    assert!(!files.is_empty(), "ui/src must contain source files");
    for path in &files {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            !content.to_lowercase().contains("rard"),
            "{} must not reference the virtue.rard/virtue.bard id or the \"Rard\" \
             scanno",
            relative(path)
        );
    }
}
