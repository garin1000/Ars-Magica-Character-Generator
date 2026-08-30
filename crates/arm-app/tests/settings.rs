//! The app-settings file that carries the saga year
//! (guided-creation-review-2026-08 #25 / D3.1, Slice 12).
//!
//! The saga year is saga state, not character state and not a rule, so it is
//! neither on the entity nor in the ruleset — it lives in a small settings file
//! this crate owns. `arm-rules` gains nothing from this slice: the engine holds the
//! arithmetic and the advisory, never the IO.

use std::fs;
use std::path::PathBuf;

use arm_app::settings;

#[test]
fn a_written_saga_year_reads_back() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);

    settings::write_saga_year(&path, 1230).unwrap();
    assert_eq!(settings::read_saga_year(Some(&path)), 1230);
}

#[test]
fn writing_creates_the_settings_directory_on_first_run() {
    // First launch: the per-user config directory may not exist yet, and the write
    // must make it rather than fail.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config").join(settings::SETTINGS_FILE_NAME);

    settings::write_saga_year(&path, 1201).unwrap();
    assert_eq!(settings::read_saga_year(Some(&path)), 1201);
}

#[test]
fn a_missing_settings_file_reads_as_the_default_saga_year() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    assert!(!path.exists());

    // Silently, never as an error: this runs at launch, and a first launch has no
    // settings file at all.
    assert_eq!(
        settings::read_saga_year(Some(&path)),
        arm_rules::DEFAULT_SAGA_YEAR
    );
}

#[test]
fn an_unreadable_settings_file_reads_as_the_default_saga_year() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    fs::write(&path, "{ this is not json").unwrap();

    assert_eq!(
        settings::read_saga_year(Some(&path)),
        arm_rules::DEFAULT_SAGA_YEAR
    );

    // A well-formed file that simply does not carry the key reads the same way.
    fs::write(&path, "{}").unwrap();
    assert_eq!(
        settings::read_saga_year(Some(&path)),
        arm_rules::DEFAULT_SAGA_YEAR
    );
}

#[test]
fn no_resolvable_settings_path_reads_as_the_default_saga_year() {
    // No config directory and no executable directory: the launch still succeeds.
    assert_eq!(settings::read_saga_year(None), arm_rules::DEFAULT_SAGA_YEAR);
}

#[test]
fn pick_settings_file_returns_the_first_candidate_that_exists() {
    // The same shape as `pick_rules_dir`, and for the same reason: a candidate list
    // is offered in order and whichever is really on disk wins, so a layout whose
    // preferred directory does not exist falls through instead of failing.
    let tmp = tempfile::tempdir().unwrap();
    let absent = tmp.path().join("nowhere").join("settings.json");
    let present = tmp.path().join(settings::SETTINGS_FILE_NAME);
    fs::write(&present, "{}").unwrap();

    assert_eq!(
        settings::pick_settings_file(&[absent.clone(), present.clone()]),
        Some(present)
    );
    assert_eq!(settings::pick_settings_file(&[absent]), None);
    assert_eq!(settings::pick_settings_file(&[]), None);
}

#[test]
fn storing_writes_to_the_file_that_already_exists() {
    // Whichever candidate the settings already live in keeps them; otherwise a
    // second file would appear and the two would disagree.
    let tmp = tempfile::tempdir().unwrap();
    let fresh = tmp.path().join("config").join(settings::SETTINGS_FILE_NAME);
    let existing = tmp.path().join(settings::SETTINGS_FILE_NAME);
    fs::write(&existing, "{}").unwrap();

    let written = settings::store_saga_year(&[fresh.clone(), existing.clone()], 1230).unwrap();
    assert_eq!(written, existing);
    assert!(!fresh.exists(), "a second settings file was started");
    assert_eq!(settings::read_saga_year(Some(&existing)), 1230);
}

#[test]
fn storing_falls_through_to_the_next_candidate_when_one_cannot_be_written() {
    // An installed build's executable directory is read-only for the user; the
    // per-user config directory is the one that takes the write. Modelled here by a
    // candidate whose parent is a plain file, which no platform lets us create.
    let tmp = tempfile::tempdir().unwrap();
    let blocked_parent = tmp.path().join("not-a-directory");
    fs::write(&blocked_parent, "").unwrap();
    let blocked = blocked_parent.join(settings::SETTINGS_FILE_NAME);
    let usable = tmp.path().join("config").join(settings::SETTINGS_FILE_NAME);

    let written = settings::store_saga_year(&[blocked, usable.clone()], 1230).unwrap();
    assert_eq!(written, usable);
    assert_eq!(settings::read_saga_year(Some(&usable)), 1230);
}

#[test]
fn storing_with_no_candidate_at_all_is_an_error() {
    // Distinct from reading: a failed *read* falls back silently because the launch
    // must proceed, while a failed *write* is a user action that reported nothing.
    assert!(settings::store_saga_year(&[], 1230).is_err());
}

#[test]
fn the_settings_file_is_canonical_json_a_human_can_edit() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    settings::write_saga_year(&path, 1230).unwrap();

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"saga_year\""), "got {text}");
    assert!(text.contains("1230"), "got {text}");
    assert!(text.ends_with('\n'), "no trailing newline: {text:?}");
}

#[test]
fn candidate_paths_are_offered_in_order_without_touching_the_disk() {
    // `settings_candidates` is the pure half of the resolution: given the
    // directories the app can resolve, it names the files to look for. Keeping it
    // pure is what makes every layout testable without a Tauri runtime.
    let config = PathBuf::from("/home/someone/.config/arm-char-gen");
    let exe_dir = PathBuf::from("/media/stick/arm-char-gen");

    assert_eq!(
        settings::settings_candidates(Some(&config), Some(&exe_dir)),
        vec![
            config.join(settings::SETTINGS_FILE_NAME),
            exe_dir.join(settings::SETTINGS_FILE_NAME),
        ],
        "the per-user config directory comes first; it resolves in every layout, \
         portable included, and is the one the user can always write"
    );
    assert_eq!(
        settings::settings_candidates(None, Some(&exe_dir)),
        vec![exe_dir.join(settings::SETTINGS_FILE_NAME)]
    );
    assert!(settings::settings_candidates(None, None).is_empty());
}
