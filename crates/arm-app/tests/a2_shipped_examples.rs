//! A2 (try-out finding 12): the rulebook's own example characters ship with the
//! app, in an `examples/` folder beside `rules/` — in the installers and in the
//! portable archives alike.
//!
//! **One source, one guard.** The examples are not hand-maintained copies. The
//! book templates under `crates/arm-rules/tests/fixtures/book_templates/` stay
//! the single source (their conformance tests in `book_templates.rs` pin them
//! against the book), and each shipped `examples/rulebook/<template>.armc` is
//! what the app itself makes of that template: opened through the app's own load
//! path (`ruleset_io.rs::load_entity_from_path`, migrations included) and written
//! back through its own save path (`ruleset_io.rs::save_entity_to_path`). So a
//! shipped example is exactly the file a user would get by opening the template
//! and pressing Save — current schema, catalogue ids, canonical key order — and
//! opening it shows no upgrade notice.
//!
//! The drift guard below regenerates every example in memory and compares bytes.
//! When it fails because the engine or a template changed, run
//!
//! ```text
//! cargo test -p arm-app --test a2_shipped_examples -- --ignored
//! ```
//!
//! which rewrites `examples/rulebook/` from the templates, and commit the result.
//!
//! `examples/*_sample.json` are test fixtures, not shipped examples, and are
//! deliberately left alone: tests scan `examples/*.json` non-recursively, and
//! `grog_sample.json` must stay a pre-migration save.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use arm_app::ruleset_io::{
    entity_extension, load_catalogue_names_from_dir, load_entity_from_path, load_ruleset_from_dir,
    save_entity_to_path, validate_loaded,
};
use arm_rules::validation::DEFAULT_SAGA_YEAR;
use arm_rules::{EntityKind, Id, LoadedEntity, Ruleset, SCHEMA_VERSION, ValidationMode};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Where the book templates live — the single source of every shipped example.
fn templates_dir() -> PathBuf {
    repo_root().join("crates/arm-rules/tests/fixtures/book_templates")
}

/// Where the shipped examples live in the repository. The bundle maps this
/// directory to `examples/` beside `rules/` (see
/// [`the_installers_ship_the_rulebook_examples_as_an_examples_folder`]).
fn shipped_dir() -> PathBuf {
    repo_root().join("examples/rulebook")
}

/// The real shipped ruleset and catalogue names, read from `rules/` exactly as
/// the app reads them at startup.
fn shipped_ruleset_and_names() -> (Ruleset, BTreeMap<Id, Vec<String>>) {
    let rules = repo_root().join("rules");
    let localized = load_ruleset_from_dir(&rules, "en").expect("shipped ruleset loads");
    let names = load_catalogue_names_from_dir(&rules, &localized.ruleset)
        .expect("shipped catalogue names load");
    (localized.ruleset, names)
}

/// Every book template's file stem (`magus_bonisagus`), sorted. Read from the
/// directory, so adding a template ships it without touching this file.
fn template_stems() -> Vec<String> {
    let mut stems: Vec<String> = fs::read_dir(templates_dir())
        .expect("book template directory exists")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| file_stem(&path))
        .collect();
    stems.sort();
    assert!(
        !stems.is_empty(),
        "no book templates found under {}",
        templates_dir().display()
    );
    stems
}

/// Book templates that are test fixtures only and are NOT shipped as examples.
///
/// `magus_criamon` is the six-spell transcription kept for DISAGREEMENT MAG2's
/// own test (`book_templates.rs::the_criamon_matches_the_book`): it stops 20 spell
/// levels short and carries a `spell_levels_unspent` advisory. The book's own
/// Criamon, all seven spells, ships as `magus_criamon_full` (D83.15).
const NOT_SHIPPED: &[&str] = &["magus_criamon"];

/// The templates that ship as examples: every template but [`NOT_SHIPPED`].
fn shipped_stems() -> Vec<String> {
    template_stems()
        .into_iter()
        .filter(|stem| !NOT_SHIPPED.contains(&stem.as_str()))
        .collect()
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .expect("UTF-8 file stem")
        .to_owned()
}

/// Opens a save through the app's own load path, migrations included.
fn open(path: &Path, ruleset: &Ruleset, names: &BTreeMap<Id, Vec<String>>) -> LoadedEntity {
    load_entity_from_path(path, DEFAULT_SAGA_YEAR, Some(ruleset), Some(names))
        .unwrap_or_else(|error| panic!("{} does not open: {error:?}", path.display()))
}

/// The file name a template ships under: its own stem, with the save extension
/// of the kind of entity it holds (`ruleset_io.rs::entity_extension`).
fn shipped_name(stem: &str, loaded: &LoadedEntity) -> String {
    format!("{stem}.{}", entity_extension(loaded.entity.entity_kind))
}

/// Whether `path` carries the save extension of any entity kind.
fn is_save_file(path: &Path) -> bool {
    let save_extensions = [
        entity_extension(EntityKind::Character),
        entity_extension(EntityKind::Covenant),
    ];
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| save_extensions.contains(&ext))
}

/// Writes the shipped form of one template into `out_dir`, returning the path
/// written: the template opened by the app and saved by the app.
fn generate(
    stem: &str,
    out_dir: &Path,
    ruleset: &Ruleset,
    names: &BTreeMap<Id, Vec<String>>,
) -> PathBuf {
    let template = templates_dir().join(format!("{stem}.json"));
    let loaded = open(&template, ruleset, names);
    let out = out_dir.join(shipped_name(stem, &loaded));
    save_entity_to_path(&loaded.entity, &out)
        .unwrap_or_else(|error| panic!("{} cannot be written: {error:?}", out.display()));
    out
}

/// The shipped path of every shipped template, in template order — derived from
/// the generation itself, so the extension is never retyped here.
fn shipped_paths(ruleset: &Ruleset, names: &BTreeMap<Id, Vec<String>>) -> Vec<(String, PathBuf)> {
    let scratch = tempfile::tempdir().unwrap();
    shipped_stems()
        .into_iter()
        .map(|stem| {
            let generated = generate(&stem, scratch.path(), ruleset, names);
            let name = generated.file_name().unwrap().to_owned();
            (stem, shipped_dir().join(name))
        })
        .collect()
}

/// Fails with one message naming every example that is not on disk, so a single
/// run says what is missing rather than stopping at the first.
fn assert_all_shipped(paths: &[(String, PathBuf)]) {
    let missing: Vec<String> = paths
        .iter()
        .filter(|(_, path)| !path.is_file())
        .map(|(_, path)| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(
        missing.is_empty(),
        "these rulebook examples are not shipped in examples/rulebook/ (run `cargo test \
         -p arm-app --test a2_shipped_examples -- --ignored` to generate them): {missing:?}"
    );
}

/// The advisory findings each template's own conformance test in
/// `book_templates.rs` pins, mirrored here: an example handed to users must show
/// exactly what its book template shows. A template absent from this list is
/// warning-free. Errors are never expected — every template is Enforced-valid.
fn expected_warnings(stem: &str) -> BTreeSet<String> {
    let codes: &[&str] = match stem {
        // B3 and T1: two Personality Flaws on a grog.
        "grog_berserker" | "grog_tough_guy" => &["too_many_personality_flaws"],
        // V3: the template shows aging outcomes without the per-year log.
        "grog_grizzled_veteran" => &["aging_rolls_pending"],
        _ => &[],
    };
    codes.iter().map(|code| (*code).to_owned()).collect()
}

/// The drift guard: each shipped example is byte-for-byte what the app makes of
/// its template today, and nothing else is in the folder.
#[test]
fn every_book_template_ships_as_the_save_the_app_writes_for_it() {
    let (ruleset, names) = shipped_ruleset_and_names();
    let paths = shipped_paths(&ruleset, &names);
    assert_all_shipped(&paths);

    let scratch = tempfile::tempdir().unwrap();
    let drifted: Vec<String> = paths
        .iter()
        .filter(|(stem, shipped)| {
            let generated = generate(stem, scratch.path(), &ruleset, &names);
            fs::read(generated).unwrap() != fs::read(shipped).unwrap()
        })
        .map(|(stem, _)| stem.clone())
        .collect();
    assert!(
        drifted.is_empty(),
        "these shipped examples no longer match their book template (regenerate with \
         `cargo test -p arm-app --test a2_shipped_examples -- --ignored`): {drifted:#?}"
    );

    // Only save files are counted, so a licence or notice file may sit beside
    // the examples and ship with them.
    let expected: BTreeSet<PathBuf> = paths.into_iter().map(|(_, path)| path).collect();
    let on_disk: BTreeSet<PathBuf> = fs::read_dir(shipped_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| is_save_file(path))
        .collect();
    assert_eq!(
        on_disk, expected,
        "examples/rulebook/ must hold exactly one generated example per shipped book \
         template (every template not in NOT_SHIPPED)"
    );
}

/// The exclusion list names real templates only, so a renamed or deleted
/// template cannot leave a stale entry that silently excludes nothing.
#[test]
fn every_excluded_template_exists() {
    let templates = template_stems();
    for stem in NOT_SHIPPED {
        assert!(
            templates.iter().any(|t| t == stem),
            "NOT_SHIPPED names {stem}, which is not a book template"
        );
    }
}

/// The examples are transcriptions of the rulebook's own statblocks, so their
/// Ars Magica Open License (CC BY-SA 4.0) attribution travels with them: a
/// `NOTICE.md` in the folder the bundle and the portable staging both copy.
#[test]
fn the_shipped_examples_carry_their_attribution_notice() {
    let notice = shipped_dir().join("NOTICE.md");
    assert!(
        notice.is_file(),
        "examples/rulebook/NOTICE.md is missing; the rulebook examples would ship \
         without their CC BY-SA attribution"
    );
    let text = fs::read_to_string(notice).unwrap();
    for required in ["Ars Magica Open License", "CC BY-SA 4.0", "Atlas Games"] {
        assert!(
            text.contains(required),
            "examples/rulebook/NOTICE.md must name {required:?}"
        );
    }
}

/// Opening a shipped example is a clean open: current schema, and not one of the
/// load-time notices a migrated or hand-typed save would raise (the CV4 "were
/// recognized from what you typed" line the raw templates trigger, finding 12).
#[test]
fn every_shipped_example_opens_in_the_current_schema_without_a_notice() {
    let (ruleset, names) = shipped_ruleset_and_names();
    let paths = shipped_paths(&ruleset, &names);
    assert_all_shipped(&paths);

    for (stem, path) in &paths {
        let raw: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(
            raw["schema_version"],
            serde_json::json!(SCHEMA_VERSION),
            "{stem} is not stored in the current schema"
        );

        let loaded = open(path, &ruleset, &names);
        assert!(
            loaded.migrated_aging_characteristics.is_empty()
                && loaded.migrated_catalogued_parameters.is_empty()
                && loaded.unresolved_catalogued_parameters.is_empty()
                && loaded.dangling_links.is_empty()
                && loaded.ambiguous_links.is_empty()
                && loaded.moved_ability_parameters.is_empty(),
            "{stem} raises a load notice: {loaded:#?}"
        );
    }
}

/// Saving an opened example without editing it writes the same bytes back, so
/// a shipped example is already in the app's canonical form.
#[test]
fn a_shipped_example_saves_back_unchanged() {
    let (ruleset, names) = shipped_ruleset_and_names();
    let paths = shipped_paths(&ruleset, &names);
    assert_all_shipped(&paths);

    let scratch = tempfile::tempdir().unwrap();
    for (stem, path) in &paths {
        let loaded = open(path, &ruleset, &names);
        let resaved = scratch.path().join(path.file_name().unwrap());
        save_entity_to_path(&loaded.entity, &resaved).unwrap();
        assert!(
            fs::read(&resaved).unwrap() == fs::read(path).unwrap(),
            "{stem} changes when opened and saved again"
        );
    }
}

/// Every shipped example is valid in Enforced mode (no error at all), and its
/// advisories are exactly the ones its book template's own test pins.
#[test]
fn every_shipped_example_validates_as_its_book_template_does() {
    let (ruleset, names) = shipped_ruleset_and_names();
    let paths = shipped_paths(&ruleset, &names);
    assert_all_shipped(&paths);

    for (stem, path) in &paths {
        let entity = open(path, &ruleset, &names).entity;
        let result = validate_loaded(&entity, &ruleset, ValidationMode::Enforced);
        let errors: BTreeSet<String> = result.errors().map(|i| i.code.clone()).collect();
        let warnings: BTreeSet<String> = result.warnings().map(|i| i.code.clone()).collect();
        assert_eq!(errors, BTreeSet::new(), "{stem} is not Enforced-valid");
        assert_eq!(
            warnings,
            expected_warnings(stem),
            "{stem} shows advisories its book template does not"
        );
    }
}

/// The installers carry the examples beside `rules/`: `bundle.resources` maps the
/// repository's `examples/rulebook/` to `examples/`. The installers are built
/// from nothing else (`.github/workflows/release.yml` runs a plain
/// `cargo tauri build`), so this mapping is the whole of the installer side.
#[test]
fn the_installers_ship_the_rulebook_examples_as_an_examples_folder() {
    let config: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("crates/arm-app/tauri.conf.json")).unwrap(),
    )
    .unwrap();
    let resources = config["bundle"]["resources"]
        .as_object()
        .expect("bundle.resources should be a map of source -> destination");
    assert_eq!(
        resources.get("../../examples/rulebook"),
        Some(&serde_json::json!("examples")),
        "tauri.conf.json bundle.resources must map ../../examples/rulebook to examples"
    );
}

/// Rewrites `examples/rulebook/` from the book templates. Not part of the suite:
/// run it on purpose when the drift guard above says the examples are stale,
///
/// ```text
/// cargo test -p arm-app --test a2_shipped_examples -- --ignored
/// ```
///
/// and commit what it writes. Removes every save file there first, so an example
/// whose template was removed does not linger; anything else in the folder (a
/// licence or notice file) is left alone.
#[test]
#[ignore = "rewrites examples/rulebook/; run on purpose with -- --ignored"]
fn regenerate_shipped_examples() {
    let (ruleset, names) = shipped_ruleset_and_names();
    let out = shipped_dir();
    fs::create_dir_all(&out).unwrap();
    for entry in fs::read_dir(&out).unwrap() {
        let path = entry.unwrap().path();
        if is_save_file(&path) {
            fs::remove_file(path).unwrap();
        }
    }
    for stem in shipped_stems() {
        generate(&stem, &out, &ruleset, &names);
    }
}
