//! A3 (D83.16): the command shims and the menu glue, driven through a real
//! Tauri app on Tauri's `MockRuntime` (the `test` feature, enabled for this
//! crate's tests only — `Cargo.toml` `[dev-dependencies]`).
//!
//! Everything else in `tests/commands.rs` calls the webview-free `*_loaded`
//! helpers directly, so the `#[tauri::command]` layer above them — the cached
//! ruleset lookup, the `NotLoaded` refusal, the argument hand-over — and the
//! muda glue in `menu.rs` were reached by no test at any level but e2e. A
//! `State<'_, AppState>` can only come out of a running app's managed state,
//! which is what the mock app provides. Commands that take an `AppHandle` are
//! typed against the real `Wry` runtime and stay out of reach here.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use arm_app::commands::{self, AppState, CloseGuardLabels, Decision, guard_decision};
use arm_app::error::AppError;
use arm_app::menu::{
    self, InstalledMenuItem, InstalledMenuSection, MENU_ACTION_EVENT, MenuEntry, MenuFlags,
    MenuLabels, MenuSection, Platform,
};
use arm_app::ruleset_io::{
    AgingApplication, AgingProjection, AgingReversion, ChildhoodApplication, load_ruleset_from_dir,
    validate_loaded,
};
use arm_rules::{AbilityParameterValue, AbilityScore, Entity, Id, Selection, ValidationMode};
use pretty_assertions::assert_eq;
use tauri::test::{MockRuntime, mock_app};
use tauri::{App, Listener, Manager};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sample_entity() -> Entity {
    let json = std::fs::read_to_string(repo_root().join("examples/companion_sample.json")).unwrap();
    serde_json::from_str(&json).unwrap()
}

/// A mock app whose managed [`AppState`] holds no ruleset — the state every
/// launch starts in until `load_ruleset` succeeds.
fn app_without_ruleset() -> App<MockRuntime> {
    let app = mock_app();
    app.manage(AppState::default());
    app
}

/// A mock app whose managed [`AppState`] caches the shipped English ruleset,
/// exactly as `load_ruleset` leaves it.
fn app_with_ruleset() -> App<MockRuntime> {
    let app = app_without_ruleset();
    let localized = load_ruleset_from_dir(&repo_root().join("rules"), "en").unwrap();
    *app.state::<AppState>().ruleset.write().unwrap() = Some(localized);
    app
}

fn is_not_loaded<T: std::fmt::Debug>(result: Result<T, AppError>) -> bool {
    matches!(result, Err(AppError::NotLoaded))
}

/// Every command that reads the cached ruleset refuses with
/// [`AppError::NotLoaded`] before one is loaded — never a panic, never an
/// answer computed against nothing. The frontend maps that one variant to its
/// "rules not loaded" message, so a command that reported anything else would
/// surface as an unexplained failure.
#[test]
fn every_ruleset_backed_command_refuses_before_a_ruleset_is_loaded() {
    let app = app_without_ruleset();
    let state = || app.state::<AppState>();
    let entity = sample_entity();

    assert!(is_not_loaded(commands::validate_entity(
        entity.clone(),
        ValidationMode::Enforced,
        state()
    )));
    assert!(is_not_loaded(commands::effective_scores(
        entity.clone(),
        state()
    )));
    assert!(is_not_loaded(commands::apply_childhood_package(
        entity.clone(),
        "childhood.athletic".to_string(),
        BTreeMap::new(),
        state()
    )));
    assert!(is_not_loaded(commands::unlink_ability_parameters(
        entity.clone(),
        "virtue.craft_guild_training".to_string(),
        state()
    )));
    assert!(is_not_loaded(commands::aging_preview(
        entity.clone(),
        40,
        10,
        BTreeMap::new(),
        None,
        state()
    )));
    assert!(is_not_loaded(commands::aging_apply(
        entity.clone(),
        40,
        10,
        BTreeMap::new(),
        None,
        state()
    )));
    assert!(is_not_loaded(commands::aging_revert(
        entity.clone(),
        40,
        state()
    )));
    assert!(is_not_loaded(commands::derived_totals(entity, state())));
}

/// Validation and the two score read-outs answer from the cached ruleset with
/// exactly what the engine computes for it.
#[test]
fn the_read_commands_answer_from_the_cached_ruleset() {
    let app = app_with_ruleset();
    let entity = sample_entity();
    let ruleset = app
        .state::<AppState>()
        .ruleset
        .read()
        .unwrap()
        .clone()
        .unwrap()
        .ruleset;

    let validated =
        commands::validate_entity(entity.clone(), ValidationMode::Enforced, app.state()).unwrap();
    assert_eq!(
        validated,
        validate_loaded(&entity, &ruleset, ValidationMode::Enforced)
    );

    let effective = commands::effective_scores(entity.clone(), app.state()).unwrap();
    assert_eq!(
        serde_json::to_value(effective).unwrap(),
        serde_json::to_value(arm_app::effective_dto::effective_scores_loaded(
            &entity, &ruleset
        ))
        .unwrap()
    );

    let totals = commands::derived_totals(entity.clone(), app.state()).unwrap();
    assert_eq!(totals, arm_rules::derived_totals(&entity, &ruleset));
}

/// The aging commands hand `age` and `die` through in their own places: the
/// shipped table answers a die of 10 at age 40 with a total of 14 (the age
/// modifier is +4), which a swapped pair could not produce — the same reading
/// `tests/commands.rs`'s `aging_preview_totals_the_typed_die_and_names_the_outcome`
/// pins one layer down. Apply then writes that year and revert takes it back
/// off, byte for byte.
#[test]
fn the_aging_commands_preview_apply_and_revert_one_year() {
    let app = app_with_ruleset();
    let mut entity = sample_entity();
    entity.age = Some(40);
    entity.aging_log.clear();
    entity.living_conditions.clear();
    entity.longevity_ritual = None;
    entity.normalize();
    let before = serde_json::to_string(&entity).unwrap();

    let AgingProjection::Previewed { total, .. } =
        commands::aging_preview(entity.clone(), 40, 10, BTreeMap::new(), None, app.state())
            .unwrap()
    else {
        panic!("the shipped ruleset carries aging rules");
    };
    assert_eq!((total.die, total.age_modifier, total.total), (10, 4, 14));

    let AgingApplication::Applied {
        entity: applied,
        total,
        ..
    } = commands::aging_apply(entity.clone(), 40, 10, BTreeMap::new(), None, app.state()).unwrap()
    else {
        panic!("a year the character owes and has not rolled applies");
    };
    assert_eq!(total.total, 14, "apply reads the same roll the preview did");
    assert_eq!(applied.aging_log.len(), 1);

    let AgingReversion::Reverted { entity: reverted } =
        commands::aging_revert((*applied).clone(), 40, app.state()).unwrap()
    else {
        panic!("the year just applied is recorded, so it reverts");
    };
    assert_eq!(serde_json::to_string(&*reverted).unwrap(), before);
}

/// The two entity-rewriting commands: a Sample Childhood applies its rows
/// through the cached ruleset ("Athletic Childhood: Athletics 2, Brawl 2,
/// Native Language 5, Swim 2", ArMDE:2384), and unlinking a removed Virtue's
/// parameter leaves its last value behind as free text.
#[test]
fn the_entity_rewriting_commands_answer_from_the_cached_ruleset() {
    let app = app_with_ruleset();

    let mut child = Entity::new(
        arm_rules::EntityKind::Character,
        Id::new("companion"),
        sample_entity().ruleset,
    );
    child.age = Some(25);
    child.ability_funding = arm_rules::AbilityFunding::LifeStages;
    child.life_stages = Some(arm_rules::LifeStagePlan {
        native_language: Some("German".to_string()),
        ..arm_rules::LifeStagePlan::default()
    });
    let ChildhoodApplication::Applied { entity: applied } = commands::apply_childhood_package(
        child,
        "childhood.athletic".to_string(),
        BTreeMap::new(),
        app.state(),
    )
    .unwrap() else {
        panic!("Athletic Childhood asks the player for nothing, so it applies");
    };
    assert_eq!(
        applied
            .ability_scores
            .iter()
            .map(|row| (row.ability.as_str(), row.score))
            .collect::<Vec<_>>(),
        vec![
            ("ability.athletics", 2),
            ("ability.brawl", 2),
            ("ability.living_language", 5),
            ("ability.swim", 2),
        ]
    );

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
    let unlinked = commands::unlink_ability_parameters(
        entity,
        "virtue.craft_guild_training".to_string(),
        app.state(),
    )
    .unwrap();
    assert_eq!(
        unlinked.ability_scores[0].parameter,
        Some(AbilityParameterValue::text("Smiths' Guild of Verdi"))
    );
}

fn discard_labels() -> CloseGuardLabels {
    CloseGuardLabels {
        title: "title".to_string(),
        message: "message".to_string(),
        discard: "discard".to_string(),
        cancel: "cancel".to_string(),
    }
}

/// The IPC half of the unsaved-changes guard (CLAUDE.md, mandatory product
/// behaviour): `update_close_guard` mirrors the frontend's dirty flag and
/// dialog text into managed state, and a fresh report clears a discard
/// confirmation left over from an earlier action, so the next close of a
/// dirty document asks again instead of passing straight through.
#[test]
fn update_close_guard_mirrors_the_report_and_rearms_the_confirmation() {
    let app = app_without_ruleset();
    app.state::<AppState>()
        .close_guard
        .lock()
        .unwrap()
        .confirmed = true;

    commands::update_close_guard(true, discard_labels(), app.state());

    let state = app.state::<AppState>();
    let mut guard = state.close_guard.lock().unwrap();
    assert!(guard.dirty);
    assert!(!guard.confirmed, "a fresh report clears the one-shot latch");
    assert_eq!(guard.labels.discard, "discard");
    assert_eq!(guard.labels.cancel, "cancel");
    assert_eq!(guard_decision(&mut guard), Decision::BlockAndShow);
}

/// The two argument-free commands hand the engine's and the menu's own lists
/// across unchanged, so the frontend holds no copy of either.
#[test]
fn the_list_commands_hand_over_their_single_sources() {
    assert_eq!(
        commands::export_label_keys(),
        arm_rules::export::LABEL_KEYS
            .iter()
            .map(|key| key.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        commands::menu_shortcuts(),
        menu::menu_shortcuts(Platform::current())
    );
}

/// Every label distinct, so a label copied into the wrong item shows.
fn distinct_labels() -> MenuLabels {
    MenuLabels {
        app: "app".into(),
        file: "file".into(),
        edit: "edit".into(),
        window: "window".into(),
        new: "new".into(),
        open: "open".into(),
        save: "save".into(),
        save_as: "save as".into(),
        export: "export".into(),
        settings: "settings".into(),
        quit: "quit".into(),
        services: "services".into(),
        hide: "hide".into(),
        hide_others: "hide others".into(),
        show_all: "show all".into(),
        undo: "undo".into(),
        redo: "redo".into(),
        cut: "cut".into(),
        copy: "copy".into(),
        paste: "paste".into(),
        select_all: "select all".into(),
        minimize: "minimize".into(),
        fullscreen: "fullscreen".into(),
        close_window: "close window".into(),
    }
}

/// A mixed gate, so an enabled state copied from the wrong flag shows.
fn mixed_flags() -> MenuFlags {
    MenuFlags {
        new: true,
        open: false,
        save: true,
        save_as: false,
        export: true,
        settings: false,
    }
}

/// What reading the installed menu must report for `model`: the glue copies
/// every title, id and enabled state across and invents nothing.
fn as_installed(model: &[MenuSection]) -> Vec<InstalledMenuSection> {
    model
        .iter()
        .map(|section| InstalledMenuSection {
            title: section.label.clone(),
            items: section
                .items
                .iter()
                .map(|entry| match entry {
                    MenuEntry::Separator => InstalledMenuItem::Other {
                        title: String::new(),
                    },
                    MenuEntry::Action {
                        id, label, enabled, ..
                    } => InstalledMenuItem::Action {
                        id: id.clone(),
                        title: label.clone(),
                        enabled: *enabled,
                    },
                    MenuEntry::Predefined { label, .. } => InstalledMenuItem::Other {
                        title: label.clone(),
                    },
                })
                .collect(),
        })
        .collect()
}

/// `build_menu` turns each desktop's model into a real menu that reads back
/// exactly as the model says — every section, every item in order, each
/// action's id, text and enabled state, each OS item's text. macOS's model is
/// the one carrying the app-menu roles, so all three desktops are built, not
/// only the one running the test.
#[test]
fn a_built_menu_reads_back_as_its_model_on_every_desktop() {
    let app = mock_app();
    for &platform in Platform::ALL {
        let model = menu::menu_model(platform, &distinct_labels(), &mixed_flags());
        let built = menu::build_menu(&app, &model).unwrap();
        assert_eq!(
            menu::read_installed_menu(&built).unwrap(),
            as_installed(&model),
            "{platform:?}"
        );
    }
}

/// `install_menu` makes this desktop's menu the app's, replacing the one
/// installed before — the language switch's rebuild, which must not leave the
/// old text in place.
#[test]
fn install_menu_replaces_the_installed_menu() {
    let app = mock_app();
    let handle = app.handle();
    menu::install_menu(handle, &MenuLabels::default(), &MenuFlags::default()).unwrap();
    menu::install_menu(handle, &distinct_labels(), &mixed_flags()).unwrap();

    let installed = menu::read_installed_menu(&handle.menu().unwrap()).unwrap();
    assert_eq!(
        installed,
        as_installed(&menu::menu_model(
            Platform::current(),
            &distinct_labels(),
            &mixed_flags()
        ))
    );
}

/// A chosen item is announced on [`MENU_ACTION_EVENT`] carrying its id, which
/// is all the frontend's `runMenuAction` listens for.
#[test]
fn a_forwarded_menu_action_is_announced_with_its_id() {
    let app = mock_app();
    let heard = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&heard);
    app.listen_any(MENU_ACTION_EVENT, move |event| {
        sink.lock().unwrap().push(event.payload().to_string());
    });

    menu::forward_menu_action(app.handle(), menu::ACTION_SAVE);

    assert_eq!(*heard.lock().unwrap(), vec!["\"menu.save\"".to_string()]);
}
