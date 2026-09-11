//! The app-settings file: the small, non-mechanical preferences the app itself
//! owns — the saga year (guided-creation-review-2026-08 #25 / D3.1, Slice 12) and,
//! since C4, the UI language, the palette and the validation mode.
//!
//! None of the four is character state and none is a rule, so they live neither on
//! the entity nor in the ruleset but in a small settings file this crate owns.
//! `arm-rules` gains nothing from any of it: the engine holds the saga-year
//! arithmetic and the `ValidationMode` taxonomy, never the IO — and it holds no
//! notion of a theme at all.

use std::fs;
use std::path::PathBuf;

use arm_app::settings::{self, SettingsPatch};
use arm_rules::ValidationMode;

/// A patch naming exactly one field, which is how every caller writes.
fn saga_year(year: i32) -> SettingsPatch {
    SettingsPatch {
        saga_year: Some(year),
        ..SettingsPatch::default()
    }
}

fn theme(theme: &str) -> SettingsPatch {
    SettingsPatch {
        theme: Some(theme.to_owned()),
        ..SettingsPatch::default()
    }
}

#[test]
fn a_written_setting_reads_back() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);

    settings::write_settings(
        &path,
        &SettingsPatch {
            lang: Some("de".to_owned()),
            saga_year: Some(1230),
            theme: Some("light".to_owned()),
            validation_mode: Some(ValidationMode::Advisory),
        },
    )
    .unwrap();

    let read = settings::read_settings(Some(&path));
    assert_eq!(read.lang.as_deref(), Some("de"));
    assert_eq!(read.saga_year, 1230);
    assert_eq!(read.theme.as_deref(), Some("light"));
    assert_eq!(read.validation_mode, Some(ValidationMode::Advisory));
}

#[test]
fn writing_one_setting_preserves_every_other_setting() {
    // The file is READ-MODIFY-WRITE, never rebuilt from the field being changed.
    // Rebuilding was harmless while the saga year was the only key and silently
    // destroying the moment a second one existed: choosing a saga year would have
    // wiped the theme and the language the user had picked, with no error, no
    // banner and no undo — the silent-data-corruption class CLAUDE.md rates
    // highest.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);

    settings::write_settings(
        &path,
        &SettingsPatch {
            lang: Some("de".to_owned()),
            saga_year: Some(1220),
            theme: Some("light".to_owned()),
            validation_mode: Some(ValidationMode::Silent),
        },
    )
    .unwrap();
    settings::write_settings(&path, &saga_year(1230)).unwrap();

    let read = settings::read_settings(Some(&path));
    assert_eq!(read.saga_year, 1230, "the field being written did not land");
    assert_eq!(
        read.lang.as_deref(),
        Some("de"),
        "the language was destroyed"
    );
    assert_eq!(
        read.theme.as_deref(),
        Some("light"),
        "the theme was destroyed"
    );
    assert_eq!(
        read.validation_mode,
        Some(ValidationMode::Silent),
        "the validation mode was destroyed"
    );
}

#[test]
fn writing_creates_the_settings_directory_on_first_run() {
    // First launch: the per-user config directory may not exist yet, and the write
    // must make it rather than fail.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("config").join(settings::SETTINGS_FILE_NAME);

    settings::write_settings(&path, &saga_year(1201)).unwrap();
    assert_eq!(settings::read_settings(Some(&path)).saga_year, 1201);
}

#[test]
fn a_missing_settings_file_reads_as_the_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    assert!(!path.exists());

    // Silently, never as an error: this runs at launch, and a first launch has no
    // settings file at all.
    let read = settings::read_settings(Some(&path));
    assert_eq!(read.saga_year, arm_rules::DEFAULT_SAGA_YEAR);
    // The other three have no default HERE. The saga year's lives in the engine
    // (a rules value); the language's and the validation mode's are the frontend
    // store's existing initial state, and the theme's is `auto` — also the
    // frontend's, deliberately, since a palette is not an Ars Magica rule and
    // `arm-rules` must never learn what one is. Reporting "unset" rather than
    // inventing a value here is what keeps each default in exactly one place.
    assert_eq!(read.lang, None);
    assert_eq!(read.theme, None);
    assert_eq!(read.validation_mode, None);
}

#[test]
fn an_unreadable_settings_file_reads_as_the_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    fs::write(&path, "{ this is not json").unwrap();

    assert_eq!(
        settings::read_settings(Some(&path)).saga_year,
        arm_rules::DEFAULT_SAGA_YEAR
    );

    // A well-formed file that simply does not carry the keys reads the same way.
    fs::write(&path, "{}").unwrap();
    let read = settings::read_settings(Some(&path));
    assert_eq!(read.saga_year, arm_rules::DEFAULT_SAGA_YEAR);
    assert_eq!(read.theme, None);
}

#[test]
fn one_unusable_value_does_not_cost_the_user_the_others() {
    // The settings file IS the trust boundary (CLAUDE.md): it is hand-editable, and
    // a build that wrote a key this one cannot parse would otherwise take the whole
    // document down with it — every OTHER setting read as unset, and then
    // overwritten with that reading on the next write. Each field is parsed on its
    // own, so a value only ever costs its own key.
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    fs::write(
        &path,
        "{\"lang\": \"de\", \"saga_year\": \"twelve thirty\", \
         \"theme\": \"light\", \"validation_mode\": \"whimsical\"}",
    )
    .unwrap();

    let read = settings::read_settings(Some(&path));
    assert_eq!(read.lang.as_deref(), Some("de"));
    assert_eq!(read.theme.as_deref(), Some("light"));
    // The two unusable ones fall back to their own default and nothing else does.
    assert_eq!(read.saga_year, arm_rules::DEFAULT_SAGA_YEAR);
    assert_eq!(read.validation_mode, None);
}

#[test]
fn no_resolvable_settings_path_reads_as_the_defaults() {
    // No config directory and no executable directory: the launch still succeeds.
    let read = settings::read_settings(None);
    assert_eq!(read.saga_year, arm_rules::DEFAULT_SAGA_YEAR);
    assert_eq!(read.lang, None);
    assert_eq!(read.theme, None);
    assert_eq!(read.validation_mode, None);
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

    let written =
        settings::store_settings(&[fresh.clone(), existing.clone()], &saga_year(1230)).unwrap();
    assert_eq!(written, existing);
    assert!(!fresh.exists(), "a second settings file was started");
    assert_eq!(settings::read_settings(Some(&existing)).saga_year, 1230);
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

    let written = settings::store_settings(&[blocked, usable.clone()], &saga_year(1230)).unwrap();
    assert_eq!(written, usable);
    assert_eq!(settings::read_settings(Some(&usable)).saga_year, 1230);
}

#[test]
fn storing_with_no_candidate_at_all_is_an_error() {
    // Distinct from reading: a failed *read* falls back silently because the launch
    // must proceed, while a failed *write* is a user action that reported nothing.
    assert!(settings::store_settings(&[], &saga_year(1230)).is_err());
}

#[test]
fn the_settings_file_is_canonical_json_a_human_can_edit() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    settings::write_settings(
        &path,
        &SettingsPatch {
            lang: Some("de".to_owned()),
            saga_year: Some(1230),
            theme: Some("light".to_owned()),
            validation_mode: Some(ValidationMode::Enforced),
        },
    )
    .unwrap();

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"saga_year\""), "got {text}");
    assert!(text.contains("1230"), "got {text}");
    assert!(text.ends_with('\n'), "no trailing newline: {text:?}");

    // Keys in a fixed, alphabetical order (CLAUDE.md's canonical serialization):
    // whichever setting the user changes, the file's shape is the same and the diff
    // is only ever the line that moved.
    let keys: Vec<usize> = [
        "\"lang\"",
        "\"saga_year\"",
        "\"theme\"",
        "\"validation_mode\"",
    ]
    .iter()
    .map(|key| {
        text.find(key)
            .unwrap_or_else(|| panic!("no {key} in {text}"))
    })
    .collect();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]), "got {text}");

    // The validation mode is written as the engine's own serde spelling, so the
    // file names the very value `validate_entity` already takes over IPC.
    assert!(text.contains("\"enforced\""), "got {text}");
}

#[test]
fn an_unset_value_is_left_out_of_the_file_entirely() {
    // Sparse, like every save this project writes: absent means "never chosen", and
    // that is a different fact from "chosen and happens to equal the default".
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(settings::SETTINGS_FILE_NAME);
    settings::write_settings(&path, &theme("dark")).unwrap();

    let text = fs::read_to_string(&path).unwrap();
    assert!(text.contains("\"theme\""), "got {text}");
    assert!(!text.contains("saga_year"), "got {text}");
    assert!(!text.contains("lang"), "got {text}");
    assert!(!text.contains("validation_mode"), "got {text}");
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
