//! Thin `#[tauri::command]` shims. They resolve the rules directory and managed
//! state, then delegate to the webview-free logic in [`crate::ruleset_io`].

use std::sync::{Mutex, RwLock, RwLockReadGuard};

use arm_rules::{Characteristic, Entity, Id, LocalizedRuleset, ValidationMode, ValidationResult};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, MessageDialogButtons, MessageDialogKind};

use arm_rules::DerivedTotals;

use crate::effective_dto::{self, EffectiveScores};
use crate::error::AppError;
use crate::menu::{MenuFlags, MenuLabels};
use crate::ruleset_io;
use crate::ruleset_io::{AgingApplication, AgingProjection, AgingReversion, ChildhoodApplication};
use crate::settings;

/// Holds the parsed, localized ruleset so validation does not re-read and
/// re-check the rules files on every keystroke. `None` until `load_ruleset`
/// succeeds.
///
/// The whole [`LocalizedRuleset`] is cached, not just its mechanics: the
/// Markdown export needs the localized display names, and re-reading the i18n
/// files per export would duplicate the loader for no gain.
#[derive(Default)]
pub struct AppState {
    /// The loaded ruleset, or `None` until [`load_ruleset`] succeeds.
    pub ruleset: RwLock<Option<LocalizedRuleset>>,
    /// Every catalogue value's display name, **both locales at once**, loaded
    /// alongside `ruleset` by [`load_ruleset`] — CV4's dependency for
    /// [`arm_rules::load_entity_migrating`]'s catalogue-matching fold (design
    /// § 5.6). Deliberately not folded into `ruleset`/`LocalizedRuleset`, which
    /// only ever carries one active language's text at a time; migration must
    /// recognize a value regardless of which language the player was using when
    /// they typed it.
    pub catalogue_names: RwLock<Option<std::collections::BTreeMap<Id, Vec<String>>>>,
    /// Mirror of the frontend's unsaved-changes state, so the window-close and
    /// app-quit handlers can prompt before discarding without a round-trip.
    pub close_guard: Mutex<CloseGuardState>,
}

/// Backend view of the close/quit guard, kept in sync by [`update_close_guard`].
#[derive(Default)]
pub struct CloseGuardState {
    /// Whether the entity has unsaved edits (mirrored from the frontend store).
    pub dirty: bool,
    /// Localized dialog strings, supplied by the frontend so no user-facing text
    /// lives in Rust.
    pub labels: CloseGuardLabels,
    /// A confirmation dialog is currently open; guards against stacking a second
    /// one when the user triggers close/quit again while it is showing.
    pub showing: bool,
    /// The user chose to discard, so the re-issued close/quit must pass through.
    pub confirmed: bool,
}

/// Localized strings for the "discard unsaved changes?" dialog. Deserialized
/// verbatim from the frontend; Rust never authors these.
#[derive(Default, Clone, serde::Deserialize)]
pub struct CloseGuardLabels {
    /// The confirmation dialog's title.
    pub title: String,
    /// The confirmation dialog's body text.
    pub message: String,
    /// Label for the button that discards unsaved changes and proceeds.
    pub discard: String,
    /// Label for the button that cancels the close/quit.
    pub cancel: String,
}

impl CloseGuardState {
    /// Mirrors a fresh dirty-state report from the frontend, clearing the
    /// one-shot discard-confirmation latch (`confirmed`).
    ///
    /// `confirmed` exists only to let a close/quit re-issued as part of the
    /// SAME confirmed discard pass straight through `guard_blocks_quit`
    /// (`main.rs`) without a second dialog. It must not outlive that one
    /// action: `update_close_guard` fires on every dirty-state transition the
    /// frontend reports, so clearing it here means a stale `true` can never
    /// reach a LATER, unrelated close/quit — the only path back to an
    /// interactive window (dock reactivation, a future "new document" flow)
    /// necessarily reports a fresh dirty state first, and that report is
    /// exactly this call. Nothing re-arms `confirmed` in between: after
    /// `w.destroy()`/`app.exit(0)` runs, that window's webview is gone and can
    /// issue no further IPC, so this reset can never race the very
    /// close/quit it is meant to let through.
    pub fn report_dirty_state(&mut self, dirty: bool, labels: CloseGuardLabels) {
        self.dirty = dirty;
        self.labels = labels;
        self.confirmed = false;
    }
}

/// What a pending close or quit must do about the unsaved-changes guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Nothing to confirm — let the close/quit proceed.
    Allow,
    /// Refuse the close/quit; a confirmation is already on screen.
    Block,
    /// Refuse the close/quit and put the confirmation up.
    BlockAndShow,
}

impl Decision {
    /// Whether the pending close/quit must be prevented — i.e. whether the
    /// caller has to call `prevent_close`/`prevent_exit`.
    ///
    /// This mapping lives here, not in `main.rs`'s `guard_blocks_quit`, for the
    /// same reason [`guard_decision`] does (Erika E1): as a `match` in the
    /// binary, `Block => true` was covered by nothing at any level — each dirty
    /// e2e spec issues exactly one close/quit, so the second one this variant
    /// exists for was never exercised, and flipping it to `false` closed the
    /// window out from under the open confirmation with every gate green.
    pub fn blocks(self) -> bool {
        match self {
            Decision::Allow => false,
            Decision::Block | Decision::BlockAndShow => true,
        }
    }

    /// Whether this decision calls for putting the confirmation on screen.
    ///
    /// The second of the two halves of the no-second-dialog contract, and it
    /// lives here for the same reason [`Self::blocks`] does (Erika E1, round 3).
    /// [`Decision::Block`] exists precisely so a close/quit arriving while the
    /// confirmation is already up is refused **without stacking another one**;
    /// `guard_decision` owns the half that decides it, and this owns the half
    /// that acts on it. Spelled as a comparison in `main.rs::guard_blocks_quit`
    /// it was reachable from nothing: that file has no unit seam, and each dirty
    /// e2e spec issues exactly one close/quit, so "blocked" and "blocked with no
    /// second dialog" looked identical from outside — as does "blocked with no
    /// dialog at all", which silently refuses every quit path including Cmd+Q.
    pub fn shows_dialog(self) -> bool {
        match self {
            Decision::Allow | Decision::Block => false,
            Decision::BlockAndShow => true,
        }
    }
}

/// The whole of the close/quit guard's decision, as a pure function of the
/// mirrored state — `main.rs`'s `guard_blocks_quit` adds nothing to it but the
/// dialog.
///
/// **It lives here rather than there because there is nowhere to test it there**
/// (Erika F4). `main.rs` is a binary, this crate does not enable Tauri's `test`
/// feature, and a native GTK dialog is not something WebDriver can answer — so
/// the e2e suite reached two of these rows and the other four were unreachable
/// at every level. They are the rows that matter most: the `confirmed`
/// pass-through and the `showing` no-second-dialog branch are the difference
/// between one confirmation and two, and the dialog answers are the difference
/// between Cancel keeping the user's work and Cancel destroying it. The
/// unsaved-changes guard is a mandatory product behaviour (`CLAUDE.md`), and a
/// mandatory behaviour with an untestable decision is one edit from silently
/// inverting.
///
/// Arming the `showing` latch is part of the decision, not of the presentation:
/// it is what makes [`Decision::Block`] reachable, and a caller that forgot to
/// set it would stack a second dialog on the next Alt+F4.
pub fn guard_decision(guard: &mut CloseGuardState) -> Decision {
    if !guard.dirty || guard.confirmed {
        return Decision::Allow;
    }
    if guard.showing {
        return Decision::Block;
    }
    guard.showing = true;
    Decision::BlockAndShow
}

/// Records the user's answer to the discard confirmation: the dialog is down
/// either way, and only a confirmed discard latches the re-issued close/quit
/// through [`guard_decision`].
///
/// Cancel deliberately changes nothing else — the edits are still unsaved and
/// still unconfirmed, so the next quit asks again. That asymmetry is the guard's
/// entire safety property; inverting it would make the button that protects the
/// document the one that discards it.
pub fn apply_dialog_answer(guard: &mut CloseGuardState, discard: bool) {
    guard.showing = false;
    if discard {
        guard.confirmed = true;
    }
}

/// Records the user's answer to the discard confirmation and re-issues the
/// close/quit — by calling `on_discard` — if and only if they chose to discard.
///
/// This is the guard's second *action* decision, and it lives here for the same
/// reason [`Decision::blocks`] does (Erika E1). `main.rs` used to spell it as a
/// bare `if !discard { return; }` around `w.destroy()`/`app.exit(0)`, i.e. around
/// the act of throwing the document away, in a binary with no unit seam — so the
/// one inversion that makes **Cancel** destroy the user's work was asserted
/// nowhere, at any level. Taking the action as a closure rather than returning a
/// "re-issue?" flag is deliberate: a flag leaves the branch in the caller, which
/// is exactly where nothing can reach it.
///
/// **The lock is released before `on_discard` runs**, and that is load-bearing
/// rather than tidy: `on_discard` is `app.exit(0)`, which fires
/// `RunEvent::ExitRequested` synchronously, which re-enters `guard_blocks_quit`
/// and locks this same mutex. See
/// `the_discard_action_runs_with_the_close_guard_lock_released`.
pub fn resolve_discard_dialog(
    guard: &Mutex<CloseGuardState>,
    discard: bool,
    on_discard: impl FnOnce(),
) {
    {
        let mut state = guard.lock().expect("close guard lock poisoned");
        apply_dialog_answer(&mut state, discard);
    }
    if discard {
        on_discard();
    }
}

/// Mirrors the frontend's dirty flag and dialog strings into managed state for
/// the close/quit guard (see `main.rs`).
#[tauri::command]
pub fn update_close_guard(dirty: bool, labels: CloseGuardLabels, state: State<'_, AppState>) {
    let mut guard = state.close_guard.lock().expect("close guard lock poisoned");
    guard.report_dirty_state(dirty, labels);
}

/// Installs (or reinstalls) the native application menu.
///
/// The frontend supplies every label — resolved from Fluent, exactly as
/// [`update_close_guard`] does for the discard dialog — and the enabled state
/// of each document action, which it reads from the one predicate
/// `store.runDocumentAction` also reads. Rust contributes the menu's shape, its
/// platform placement and — since C7 — each item's keyboard accelerator, and
/// nothing else (see [`crate::menu`]).
///
/// Called again on every UI-language switch and whenever that predicate
/// changes, because a menu carries the text and the enabled state it was built
/// with until it is replaced.
#[tauri::command]
pub fn set_app_menu(labels: MenuLabels, flags: MenuFlags, app: AppHandle) -> Result<(), AppError> {
    crate::menu::install_menu(&app, &labels, &flags).map_err(|e| AppError::Menu {
        message: e.to_string(),
    })
}

/// The chords the webview must answer to itself on this desktop. Empty
/// everywhere but Windows; see [`crate::menu::menu_shortcuts`].
#[tauri::command]
pub fn menu_shortcuts() -> Vec<crate::menu::MenuShortcut> {
    crate::menu::menu_shortcuts(crate::menu::Platform::current())
}

/// Locks the cached ruleset for reading. Every command that needs a loaded
/// ruleset starts here, one line before [`require_loaded`].
///
/// Split into two helpers rather than one "give me the `&LocalizedRuleset`"
/// function: `RwLockReadGuard` has no stable `.map()` on this toolchain, so a
/// single helper cannot hand back a reference into a guard it dropped on
/// return. Naming the guard at the call site keeps it alive exactly as long as
/// the reference [`require_loaded`] derives from it — the same lifetime shape
/// the old, duplicated two-liner had, just written once.
fn ruleset_guard<'a>(
    state: &'a State<'_, AppState>,
) -> RwLockReadGuard<'a, Option<LocalizedRuleset>> {
    state.ruleset.read().expect("ruleset lock poisoned")
}

/// The other half of the pair `ruleset_guard` starts: turns "no ruleset
/// loaded yet" into the one [`AppError::NotLoaded`] every command reports it
/// with, so that mapping can only be expressed in one place.
fn require_loaded(ruleset: Option<&LocalizedRuleset>) -> Result<&LocalizedRuleset, AppError> {
    ruleset.ok_or(AppError::NotLoaded)
}

/// Loads the ruleset for `lang`, caches it in managed state, and returns the
/// localized snapshot (mechanics + display text) for the frontend.
#[tauri::command]
pub fn load_ruleset(
    lang: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<LocalizedRuleset, AppError> {
    // Where the rules live depends on how the app was launched. For a dev run or
    // an installed bundle, `BaseDirectory::Resource` points at the right place;
    // for a portable Linux build it resolves to a nonexistent system path
    // (`/usr/lib/<name>`), so the directory next to the executable is the real
    // location. Offer both and let `pick_rules_dir` choose whichever exists.
    let mut candidates = Vec::new();
    if let Ok(resource) = app.path().resolve("rules", BaseDirectory::Resource) {
        candidates.push(resource);
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
    {
        candidates.push(exe_dir.join("rules"));
    }

    let rules_dir = ruleset_io::pick_rules_dir(&candidates).ok_or_else(|| AppError::Io {
        message: format!(
            "no valid rules directory found; looked in: {}",
            candidates
                .iter()
                .map(|c| describe_rejected_candidate(c))
                .collect::<Vec<_>>()
                .join("; ")
        ),
    })?;

    // Report before propagating (E4). The frontend gets the same error either
    // way and renders its localized sentence plus a technical-detail disclosure;
    // this is the second surface, for a binary launched from a terminal, where
    // the whole diagnostic list is free and needs no UI at all. See
    // `error.rs::AppError::write_ruleset_diagnostics` for why its text is
    // English on purpose.
    let localized =
        ruleset_io::load_ruleset_from_dir(&rules_dir, &lang).map_err(AppError::reported)?;

    // Both locales, always, independent of `lang` (design § 5.3/§ 5.6): migration
    // must recognize a catalogue value's name regardless of which language the
    // player was using when they typed it. A failure here does not fail the whole
    // ruleset load — the picker/migration simply see no catalogue names yet, the
    // same "not populated" shape `AppState::catalogue_names` starts in — since a
    // player already relies on the ruleset itself being usable.
    let catalogue_names =
        ruleset_io::load_catalogue_names_from_dir(&rules_dir, &localized.ruleset).ok();

    *state.ruleset.write().expect("ruleset lock poisoned") = Some(localized.clone());
    *state
        .catalogue_names
        .write()
        .expect("catalogue names lock poisoned") = catalogue_names;

    Ok(localized)
}

/// Names exactly why a candidate directory was rejected by `pick_rules_dir`
/// (V9): either it does not exist at all, or it exists but is missing one or
/// more of the required core rules files — named individually, via
/// [`ruleset_io::missing_core_files`], rather than surfacing only a raw
/// "file not found" for whichever file `load_ruleset_from_dir` happened to
/// read first.
fn describe_rejected_candidate(dir: &std::path::Path) -> String {
    if !dir.is_dir() {
        return format!("{} (directory does not exist)", dir.display());
    }
    let missing = ruleset_io::missing_core_files(dir);
    format!("{} (missing: {})", dir.display(), missing.join(", "))
}

/// Validates an entity against the loaded ruleset under the given mode.
#[tauri::command]
pub fn validate_entity(
    entity: Entity,
    mode: ValidationMode,
    state: State<'_, AppState>,
) -> Result<ValidationResult, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(ruleset_io::validate_loaded(&entity, &ruleset.ruleset, mode))
}

/// Computes the effective-score bonuses (Puissant Ability, Great Characteristic)
/// the entity's virtues grant, for the frontend to display alongside the base
/// scores. Uses the same engine path as validation.
#[tauri::command]
pub fn effective_scores(
    entity: Entity,
    state: State<'_, AppState>,
) -> Result<EffectiveScores, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(effective_dto::effective_scores_loaded(
        &entity,
        &ruleset.ruleset,
    ))
}

/// Applies a Sample Childhood package to the character, returning either the
/// character it becomes or the reasons it could not be applied (as localizable
/// [`arm_rules::ValidationIssue`]s — see [`ChildhoodApplication`]).
///
/// `slot_values` answers the package's parameter slots (`area_a`, `language`, …),
/// keyed by slot; JS supplies it as `slotValues`, as Tauri's camelCase argument
/// convention requires.
#[tauri::command]
pub fn apply_childhood_package(
    entity: Entity,
    package_id: String,
    slot_values: std::collections::BTreeMap<String, String>,
    state: State<'_, AppState>,
) -> Result<ChildhoodApplication, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(ruleset_io::apply_childhood_package_loaded(
        &entity,
        &Id::new(package_id),
        &slot_values,
        &ruleset.ruleset,
    ))
}

/// Converts every bought Ability score `Linked` to `removed_item`'s own
/// parameter into free text holding its last resolvable value (design § 5.5,
/// `docs/vf-audit/design-cv-catalogued-values.md`), returning the whole
/// updated entity. The frontend's Virtue/Flaw removal/clear flow calls this
/// BEFORE applying the change that would otherwise strand the link (CV7).
#[tauri::command]
pub fn unlink_ability_parameters(
    entity: Entity,
    removed_item: String,
    state: State<'_, AppState>,
) -> Result<Entity, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(ruleset_io::unlink_ability_parameters_loaded(
        &entity,
        &ruleset.ruleset,
        &Id::new(removed_item),
    ))
}

/// Reads one year's aging roll without writing anything: the AGING TOTAL the typed
/// `die` makes at `age`, and the row it lands on.
///
/// The die is player input the entity must never store, so the calculator asks the
/// engine rather than keeping the number on the character. A refusal comes back as
/// localizable [`arm_rules::ValidationIssue`]s (see [`AgingProjection`]), never as
/// an error.
///
/// The Crisis rides on this same call rather than on one of its own: it exists only
/// because this year's row demanded it, and its total counts the Decrepitude this
/// year raises. `distribution` and `crisisDie` are therefore the very arguments
/// [`aging_apply`] takes, so the preview shows exactly what Apply will write.
#[tauri::command]
pub fn aging_preview(
    entity: Entity,
    age: u32,
    die: i32,
    distribution: std::collections::BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
    state: State<'_, AppState>,
) -> Result<AgingProjection, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(ruleset_io::aging_preview_loaded(
        &entity,
        &ruleset.ruleset,
        age,
        die,
        &distribution,
        crisis_die,
    ))
}

/// Applies one year's aging roll, returning the character it makes together with
/// the reading that made it.
///
/// `distribution` places the Aging Points the row leaves to the player, per
/// Characteristic; JS supplies it as `distribution` keyed by Characteristic slug.
/// Empty for a row that names its own Characteristics.
///
/// `crisisDie` is the Simple Die thrown at the Crisis Table for a year the aging
/// row sent there (`ArMDE:16621`); `null` records the Crisis as owed and unrolled,
/// which is a legitimate state rather than a refusal.
#[tauri::command]
pub fn aging_apply(
    entity: Entity,
    age: u32,
    die: i32,
    distribution: std::collections::BTreeMap<Characteristic, u8>,
    crisis_die: Option<i32>,
    state: State<'_, AppState>,
) -> Result<AgingApplication, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(ruleset_io::aging_apply_loaded(
        &entity,
        &ruleset.ruleset,
        age,
        die,
        &distribution,
        crisis_die,
    ))
}

/// Takes one applied aging year back off, exactly — a 25-roll pre-play catch-up
/// with no undo would not be shippable.
#[tauri::command]
pub fn aging_revert(
    entity: Entity,
    age: u32,
    state: State<'_, AppState>,
) -> Result<AgingReversion, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(ruleset_io::aging_revert_loaded(
        &entity,
        &ruleset.ruleset,
        age,
    ))
}

/// Computes the read-only play-stat totals (casting, lab, penetration, magic
/// resistance, combat, soak, encumbrance, fatigue, wounds, longevity,
/// decrepitude, warping) the character sheet displays. Pure engine path, mirrors
/// [`effective_scores`]; the frontend renders these numbers and computes no
/// mechanics itself.
#[tauri::command]
pub fn derived_totals(
    entity: Entity,
    state: State<'_, AppState>,
) -> Result<DerivedTotals, AppError> {
    let guard = ruleset_guard(&state);
    let ruleset = require_loaded(guard.as_ref())?;
    Ok(arm_rules::derived_totals(&entity, &ruleset.ruleset))
}

/// The settings files to look for, in preference order (see [`settings`] for why
/// the per-user config directory comes first and the executable's directory is only
/// a fallback).
fn settings_candidates(app: &AppHandle) -> Vec<std::path::PathBuf> {
    let config_dir = app.path().app_config_dir().ok();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf));
    settings::settings_candidates(config_dir.as_deref(), exe_dir.as_deref())
}

/// Everything the app persists between runs: the **default** saga year for new
/// documents (guided-creation-review-2026-08 #25; narrowed to a default by C8, when
/// the saga year itself moved onto the entity), and since C4 the UI language, the
/// palette and the validation mode.
///
/// **One command for all four, and the frontend reads it before it loads the
/// ruleset.** Rules display text is per-language, so the language has to be known
/// *before* the load, not beside it; a single settings read is what makes serializing
/// the two affordable, since it is one small local file rather than a round trip per
/// setting.
///
/// Infallible on purpose: it is read at launch, and a first launch has no settings
/// file. Everything that could go wrong reads as "not chosen" — with the default saga
/// year resolving to [`arm_rules::DEFAULT_SAGA_YEAR`], the one setting whose own
/// default is a rules value.
#[tauri::command]
pub fn read_settings(app: AppHandle) -> settings::Settings {
    let candidates = settings_candidates(&app);
    settings::read_settings(settings::pick_settings_file(&candidates).as_deref())
}

/// Persists the settings the patch names, leaving every other key alone. App state,
/// so it survives a relaunch — but it is not document state, so it neither touches
/// the entity nor dirties it.
#[tauri::command]
pub fn write_settings(patch: settings::SettingsPatch, app: AppHandle) -> Result<(), AppError> {
    settings::store_settings(&settings_candidates(&app), &patch).map(|_| ())
}

/// How old a character born in `birthYear` is in `sagaYear`, plus any advisory the
/// pair warrants (a saga year before the birth year clamps the age to 0).
///
/// The engine's [`arm_rules::age_in_saga_year`] does the work: the clamp policy and
/// the finding have one home, and the frontend computes no mechanics of its own.
///
/// `maxAge` is the ruleset's app maximum age (`AgingRules::max_age`), handed in by
/// the frontend from the ruleset it already holds; `None` when the ruleset states
/// none. The derived age clamps at it.
#[tauri::command]
pub fn derive_age(
    saga_year: i32,
    birth_year: i32,
    max_age: Option<u32>,
) -> arm_rules::AgeInSagaYear {
    arm_rules::age_in_saga_year(saga_year, birth_year, max_age)
}

/// Which year a character aged `age` in `sagaYear` was born in — the other view of
/// the same fact as [`derive_age`].
#[tauri::command]
pub fn derive_birth_year(saga_year: i32, age: u32) -> i32 {
    arm_rules::birth_year_in_saga_year(saga_year, age)
}

/// E2E seam: when set, save/load use this fixed path instead of opening a
/// native dialog. Native GTK dialogs can't be driven by WebDriver, so the
/// real-binary e2e sets this so the flow is deterministic and headless.
///
/// **Fixed (K5/VA5, security review wave 1).** This override used to be read
/// unconditionally, with no `#[cfg(debug_assertions)]` or Cargo feature gate,
/// so the exact binary shipped to end users honored it: anything able to set
/// this process's environment before launch (a modified shortcut, a wrapper
/// script) could silently redirect Save/Open to an attacker-chosen path with
/// no dialog and no user-facing confirmation.
///
/// A `#[cfg(debug_assertions)]` gate was not an option: `ui/e2e/README.md` is
/// explicit that the suite deliberately drives the real **release**-profile
/// binary end users run (`cargo tauri build --no-bundle`), where
/// `debug_assertions` is always `false` — that gate would silently break e2e
/// rather than close the gap. Instead this is gated behind the Cargo feature
/// `e2e-testing` (declared in `crates/arm-app/Cargo.toml`, off by default), so
/// [`e2e_file_override`] always returns `None` in the binary a plain
/// `cargo build --release` / `cargo tauri build` produces. Only
/// `ui/e2e/wdio.conf.js` and `ui/e2e/wdio.portable.conf.js` pass
/// `--features e2e-testing` to `cargo tauri build --no-bundle` before driving
/// the resulting binary, so the e2e suite keeps its deterministic seam
/// without it existing in the shipped artifact. See
/// `e2e_file_override_is_compiled_out_of_the_default_build` /
/// `e2e_file_override_is_honored_when_the_feature_is_enabled` in
/// `tests/commands.rs` for both halves of the gate.
#[cfg(feature = "e2e-testing")]
const E2E_FILE_ENV: &str = "ARM_E2E_FILE";

/// The `ARM_E2E_FILE` override path, when the `e2e-testing` feature is built in.
#[cfg(feature = "e2e-testing")]
pub fn e2e_file_override() -> Option<std::path::PathBuf> {
    std::env::var_os(E2E_FILE_ENV).map(std::path::PathBuf::from)
}

/// Always `None`: the shipped (non-`e2e-testing`) build has no override seam.
#[cfg(not(feature = "e2e-testing"))]
pub fn e2e_file_override() -> Option<std::path::PathBuf> {
    None
}

/// Asks the user whether to discard unsaved changes before a NON-TERMINAL
/// document action — New or Open — replaces the character being edited.
///
/// This is the same native dialog the close/quit guard shows (`main.rs`'s
/// `guard_blocks_quit`), so the app confirms a discard exactly one way whatever
/// triggered it. It used to confirm it two ways: a native dialog on close/quit
/// and an in-app Svelte modal on New/Open, which meant two dialogs to keep
/// worded alike, two focus behaviours, and only one of them looking like the
/// desktop it runs on.
///
/// **It deliberately touches no [`CloseGuardState`].** It takes no
/// `State<'_, AppState>`, so it structurally cannot: `confirmed` there is a
/// one-shot latch that lets a re-issued *close/quit* through without a second
/// dialog, and it is never reset until the frontend reports a fresh dirty state.
/// Riding a New or an Open on that latch would suppress every later
/// confirmation in the session and silently destroy the user's work on the
/// second New. Neither is `dirty` consulted: whether there is anything to
/// discard is the frontend's question, and it only calls this when there is.
///
/// `labels` carries every word, resolved from Fluent by the caller, exactly as
/// [`update_close_guard`] and [`set_app_menu`] do — Rust authors no user-facing
/// text.
///
/// Returns `Some(true)` to discard, `Some(false)` to cancel, and `None` when
/// this build has no native confirmation to offer, which is the test seam: see
/// [`native_discard_confirmation_enabled`].
#[tauri::command]
pub async fn confirm_discard(labels: CloseGuardLabels, app: AppHandle) -> Option<bool> {
    if !native_discard_confirmation_enabled() {
        return None;
    }
    let mut dialog = app
        .dialog()
        .message(labels.message)
        .title(labels.title)
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            labels.discard,
            labels.cancel,
        ));
    // Tie the confirmation to the window it is about, so it cannot be lost
    // behind it. Parenting IS the modality mechanism the dialog plugin offers —
    // the same call `guard_blocks_quit` makes for the close/quit dialog.
    if let Some(window) = app.get_webview_window("main") {
        dialog = dialog.parent(&window);
    }
    // `blocking_show` off the main thread, exactly as [`save_entity`] calls
    // `blocking_save_file`: an async command runs on Tauri's async runtime, not
    // on the UI thread, so blocking here waits for the answer without stalling
    // the event loop that has to draw the dialog.
    Some(dialog.blocking_show())
}

/// Whether this build owns a native discard confirmation. The single switch
/// [`confirm_discard`] consults, and the whole of the C3b test seam.
///
/// **The polarity is the inverse of [`e2e_file_override`]'s, on purpose.** There
/// the seam is an *affordance* (redirect a save away from its dialog), so the
/// shipped build must not have it. Here the seam is the *bypass*, so the shipped
/// build is the one that must act: it always shows the real dialog and never
/// hands a load-bearing data-loss decision back to the webview.
///
/// The `e2e-testing` build declines instead, because no WebDriver capability
/// available to this project can dismiss a native GTK dialog — proven, not
/// theoretical: the dirty-quit specs leave one open for the app's entire
/// lifetime, which is why they are pinned as the tail spec of their file. New is
/// reached from `returnToStartScreen()` in essentially every spec's setup
/// (`ui/e2e/helpers.js`), so without this the whole suite would hang on its
/// first `beforeEach`. Returning `None` sends the frontend to its in-app
/// fallback prompt, whose buttons WebDriver can click
/// (`ui/src/lib/components/DiscardPrompt.svelte`).
///
/// Both halves are proved: `the_discard_confirmation_is_native_in_the_default_build`
/// and `the_discard_confirmation_stands_down_under_the_e2e_feature` in
/// `tests/commands.rs`.
#[cfg(not(feature = "e2e-testing"))]
pub fn native_discard_confirmation_enabled() -> bool {
    true
}

#[cfg(feature = "e2e-testing")]
pub fn native_discard_confirmation_enabled() -> bool {
    false
}

/// E2E-only IPC seam: press a native menu item.
///
/// A native menu is drawn by the OS *outside* the webview, so WebDriver can
/// neither see it, click it, nor read it — C3a established that and it is
/// permanent. Without this the only thing proving a menu click does anything is
/// a client unit test calling `App.svelte`'s handler directly; nothing connects
/// "the OS activated `menu.save`" to "the document was saved" against the
/// shipped binary.
///
/// **It is the real dispatch path, not a parallel one.** The body is the single
/// line `main.rs`'s `on_menu_event` runs — [`crate::menu::forward_menu_action`]
/// — so the id travels the same event to the same frontend listener
/// (`runMenuAction` in `ui/App.svelte`), which routes it to
/// `store.runDocumentAction`. The availability gate is therefore untouched:
/// `store.documentActionEnabled` is still the one answer to "may this run right
/// now?" (C3a consolidated it there precisely so no second copy could exist),
/// and a disabled item activated through here is refused by it exactly as a
/// disabled item clicked by a user would never be delivered at all. The
/// e2e spec asserts that.
///
/// Ids are restricted to [`crate::menu::is_menu_action_id`] so this stays a way
/// to press one of six items rather than a way to publish an arbitrary string
/// on the app's event bus.
///
/// Inert unless the crate is built with the `e2e-testing` Cargo feature — see
/// [`menu_test_seams_enabled`], which mirrors `main.rs`'s `request_exit`.
#[tauri::command]
pub fn activate_menu_item(id: String, app: AppHandle) {
    if !menu_test_seams_enabled() || !crate::menu::is_menu_action_id(&id) {
        return;
    }
    crate::menu::forward_menu_action(&app, &id);
}

/// E2E-only IPC seam: read back the menu Tauri has actually installed.
///
/// The companion to [`activate_menu_item`], and the only way to observe that
/// [`set_app_menu`] reached the OS at all. `crate::menu::menu_model` is unit
/// tested as a pure value, which can never show that a live language switch
/// *rebuilt* the installed menu — the failure C3a warned about, where a menu
/// built once at startup sits in English behind a German UI for the rest of the
/// session.
///
/// Reads [`tauri::AppHandle::menu`], i.e. the live object, never the model that
/// was handed to it; see [`crate::menu::read_installed_menu`] for what the API
/// can and cannot report. `None` means either that this build has no seam or
/// that no menu is installed — the spec's assertions distinguish the two,
/// because an absent menu fails them.
#[tauri::command]
pub fn installed_menu(app: AppHandle) -> Option<Vec<crate::menu::InstalledMenuSection>> {
    if !menu_test_seams_enabled() {
        return None;
    }
    crate::menu::read_installed_menu(&app.menu()?).ok()
}

/// Whether this build carries the two menu test seams above.
///
/// Same polarity and same reasoning as [`e2e_file_override`]'s: the seams are
/// *affordances* (drive the app, inspect its chrome), so the shipped build must
/// not have them, and only `ui/e2e/wdio.conf.js`'s
/// `--features e2e-testing` build does. Both halves are proved in
/// `tests/commands.rs`.
#[cfg(feature = "e2e-testing")]
pub fn menu_test_seams_enabled() -> bool {
    true
}

/// Always `false`: the shipped (non-`e2e-testing`) build has neither seam.
#[cfg(not(feature = "e2e-testing"))]
pub fn menu_test_seams_enabled() -> bool {
    false
}

/// Ties a file dialog to the app's main window, so it opens centred on the app and
/// can never be lost behind it. Parenting IS the mechanism here — the dialog plugin
/// exposes no modal or always-on-top flag. Falls back to an unparented dialog when
/// the window cannot be resolved (it is still better than no dialog at all).
fn parented_to_main_window(
    app: &AppHandle,
    dialog: FileDialogBuilder<tauri::Wry>,
) -> FileDialogBuilder<tauri::Wry> {
    match app.get_webview_window("main") {
        Some(window) => dialog.set_parent(&window),
        None => dialog,
    }
}

/// An opened document: the deserialized entity plus the file it came from, so
/// the frontend can track it as the "current file" for subsequent direct saves —
/// and whatever the load's schema migration rewrote on the way in.
///
/// **Named for what it is** (Viktor #8). It used to be `LoadedEntity`, which is
/// also the name of `arm_rules`'s migration outcome — two types in scope in this
/// one crate, meaning different things. This one is what [`load_entity`]
/// returns: a document the user opened. The serde field names `path` and
/// `entity` are unchanged, so the shape the frontend consumes is exactly what it
/// was.
#[derive(serde::Serialize)]
pub struct OpenedDocument {
    /// The filesystem path the entity was loaded from.
    pub path: String,
    /// The deserialized (and migrated) entity.
    pub entity: Entity,
    /// Characteristics whose legacy `aging_reductions` the load folded into
    /// `aging_points` — empty for a save that needed no migration.
    ///
    /// Carried to the frontend rather than printed (Viktor #4): a GUI binary
    /// started from a desktop launcher has no terminal, so the `eprintln!` this
    /// used to be reached nobody. The migration is lossy — the engine
    /// reconstructs the MINIMAL aging-point total that reproduces the recorded
    /// drops — and the next save makes it permanent, which is exactly the kind
    /// of silent rewrite the user is entitled to hear about.
    ///
    /// The Characteristics travel as themselves, never as a sentence: the
    /// frontend resolves the localized notice from them, as it does for every
    /// other engine output.
    pub migrated_aging_characteristics: Vec<Characteristic>,
    /// Parameterized Ability rows whose free-text value did NOT match any
    /// catalogue entry's name in either locale, and so stayed free text (CV4b,
    /// design § 5.5) — empty for a save with nothing unresolved.
    ///
    /// A previously working-by-luck authorization or restricted-pool funding
    /// can silently stop applying once a Literal instance is satisfied only by
    /// a `Catalogued` value (design § 4 rule 1), so this must reach the player,
    /// not stay an engine-only report (Viktor #4's aging precedent again). The
    /// Ability travels as its own id, never a sentence: the frontend resolves
    /// the localized name from it, exactly as `migrated_aging_characteristics`
    /// does.
    ///
    pub unresolved_catalogued_parameters: Vec<UnresolvedCatalogueParameter>,
    /// Parameterized Ability rows whose free-text value WAS recognized as a
    /// catalogue entry's name and folded into `Catalogued` (CV4b, design §
    /// 5.5) — empty for a save with nothing recognized. A positive counterpart
    /// to [`Self::unresolved_catalogued_parameters`]: "what you typed is now
    /// linked to its catalogue entry," so a rename or a future locale switch
    /// still resolves correctly.
    ///
    pub migrated_catalogued_parameters: Vec<MigratedCatalogueParameter>,
    /// Ability instances a pre-22 save stored under the sibling Ability whose
    /// catalogue no longer holds their value, moved by the load (L1b, try-out
    /// finding 6). Empty for a save that needed no move. Ids and scores only —
    /// both Abilities, the catalogue value, the moved score and, on a collision,
    /// the score already in place — so the frontend composes its own one-time
    /// localized notice, as it does for the CV4b reports above.
    pub moved_ability_parameters: Vec<arm_rules::migration::MovedAbilityParameter>,
}

/// One parameterized Ability instance whose stored value did not match any
/// catalogue entry's name in either locale, so it stayed free text — see
/// [`OpenedDocument::unresolved_catalogued_parameters`].
#[derive(serde::Serialize)]
pub struct UnresolvedCatalogueParameter {
    /// The parameterized Ability's id.
    pub ability: Id,
    /// The stored free-text value that matched no catalogue entry.
    pub text: String,
}

/// One parameterized Ability instance whose free-text value WAS recognized as
/// a catalogue entry's name and folded into `Catalogued` — see
/// [`OpenedDocument::migrated_catalogued_parameters`].
#[derive(serde::Serialize)]
pub struct MigratedCatalogueParameter {
    /// The parameterized Ability's id.
    pub ability: Id,
    /// The stored free-text value that was recognized.
    pub text: String,
    /// The catalogue entry id the free text was folded into.
    pub resolved: Id,
}

/// Builds the frontend-facing [`OpenedDocument`] from the engine's migration
/// outcome. Extracted so [`load_entity`]'s command body and this crate's own
/// tests share one conversion, rather than each hand-building the struct
/// literal and risking the two drifting apart.
pub fn opened_document(path: String, loaded: arm_rules::LoadedEntity) -> OpenedDocument {
    OpenedDocument {
        path,
        entity: loaded.entity,
        migrated_aging_characteristics: loaded.migrated_aging_characteristics,
        unresolved_catalogued_parameters: loaded
            .unresolved_catalogued_parameters
            .into_iter()
            .map(|(ability, text)| UnresolvedCatalogueParameter { ability, text })
            .collect(),
        migrated_catalogued_parameters: loaded
            .migrated_catalogued_parameters
            .into_iter()
            .map(|(ability, text, resolved)| MigratedCatalogueParameter {
                ability,
                text,
                resolved,
            })
            .collect(),
        moved_ability_parameters: loaded.moved_ability_parameters,
    }
}

/// Writes the entity as canonical JSON. When `path` is `Some`, writes straight to
/// that file with no dialog (a plain Save to the current file); when `None`,
/// prompts for a destination (Save As / first Save). Returns the written path, or
/// `None` if a prompt was cancelled.
#[tauri::command]
pub async fn save_entity(
    entity: Entity,
    path: Option<String>,
    app: AppHandle,
) -> Result<Option<String>, AppError> {
    let ext = ruleset_io::entity_extension(entity.entity_kind);
    let target = match path {
        // A known current file: write directly, no dialog. `ensure_extension` is a
        // no-op for a path that already carries one.
        Some(path) => ruleset_io::ensure_extension(std::path::PathBuf::from(path), ext),
        None => match e2e_file_override() {
            Some(path) => path,
            None => {
                let dialog = app
                    .dialog()
                    .file()
                    // Ars Magica character/covenant files first (the default save
                    // type), then JSON for interop, then an all-files fallback.
                    .add_filter("Ars Magica character", &["armc", "armcov"])
                    .add_filter("JSON", &["json"])
                    .add_filter("All files", &["*"])
                    .set_file_name(ruleset_io::default_file_name(entity.entity_kind));
                let Some(file) = parented_to_main_window(&app, dialog).blocking_save_file() else {
                    return Ok(None);
                };
                let chosen = file.into_path().map_err(|e| AppError::Io {
                    message: e.to_string(),
                })?;
                ruleset_io::ensure_extension(chosen, ext)
            }
        },
    };

    // Before the write, not after: a path the frontend cannot be told about is a
    // path it must not adopt as the current file, and writing first would leave a
    // document on disk the app then disowns (Gerda #5).
    let reported = ruleset_io::path_text(&target)?;
    ruleset_io::save_entity_to_path(&entity, &target)?;
    Ok(Some(reported))
}

/// E2E seam for the Markdown export, deliberately separate from [`E2E_FILE_ENV`]:
/// that one is a fixed `.json` path shared by save and open, so writing Markdown
/// through it would clobber the save file the same spec round-trips.
#[cfg(feature = "e2e-testing")]
const E2E_EXPORT_FILE_ENV: &str = "ARM_E2E_EXPORT_FILE";

/// Fixed (K5/VA5, see [`e2e_file_override`]): gated behind the same
/// `e2e-testing` Cargo feature, so it too is inert in the shipped binary.
#[cfg(feature = "e2e-testing")]
pub fn e2e_export_file_override() -> Option<std::path::PathBuf> {
    std::env::var_os(E2E_EXPORT_FILE_ENV).map(std::path::PathBuf::from)
}

/// Always `None`: the shipped (non-`e2e-testing`) build has no override seam.
#[cfg(not(feature = "e2e-testing"))]
pub fn e2e_export_file_override() -> Option<std::path::PathBuf> {
    None
}

/// Writes the entity as a Markdown character sheet. When `path` is `Some`, writes
/// straight to that file; when `None`, prompts for a destination. Returns the
/// written path, or `None` if a prompt was cancelled.
///
/// `current_path` is the document's own save file, used ONLY to prefill the dialog
/// (name and starting directory) so an export lands next to and named after the
/// character file. It never becomes a write target — that is `path`'s job.
///
/// `labels` is the frontend's localized document chrome, keyed by the Fluent message
/// names [`export_label_keys`] lists — Rust authors none of the document's text.
#[tauri::command]
pub async fn export_markdown(
    entity: Entity,
    labels: std::collections::BTreeMap<String, String>,
    path: Option<String>,
    current_path: Option<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Option<String>, AppError> {
    // The destination is resolved BEFORE the ruleset lock is taken: a native dialog
    // blocks until the user dismisses it, and holding the read guard across it would
    // stall a concurrent language switch (`load_ruleset` wants the write guard) for
    // exactly as long as the dialog stays open.
    let target = match path {
        Some(path) => ruleset_io::ensure_extension(
            std::path::PathBuf::from(path),
            ruleset_io::MARKDOWN_EXTENSION,
        ),
        None => match e2e_export_file_override() {
            Some(path) => path,
            None => {
                let mut dialog = app
                    .dialog()
                    .file()
                    .add_filter("Markdown", &["md"])
                    .set_file_name(ruleset_io::markdown_file_name_for(
                        current_path.as_deref(),
                        entity.entity_kind,
                    ));
                // Open where the character file lives, so the export sits beside it.
                if let Some(parent) = ruleset_io::save_file_directory(current_path.as_deref()) {
                    dialog = dialog.set_directory(parent);
                }
                let Some(file) = parented_to_main_window(&app, dialog).blocking_save_file() else {
                    return Ok(None);
                };
                let chosen = file.into_path().map_err(|e| AppError::Io {
                    message: e.to_string(),
                })?;
                ruleset_io::ensure_extension(chosen, ruleset_io::MARKDOWN_EXTENSION)
            }
        },
    };

    // `export_markdown_to_path` (owned by `ruleset_io.rs`) does its own
    // `NotLoaded` check on the `Option`, so only the lock-acquisition half of
    // the pair applies here.
    let guard = ruleset_guard(&state);
    ruleset_io::export_markdown_to_path(&entity, guard.as_ref(), &labels, &target)?;
    Ok(Some(target.to_string_lossy().into_owned()))
}

/// The document-chrome label keys the Markdown export needs, so the frontend
/// resolves exactly those against its Fluent bundle and there is one source of
/// truth for the list (the engine's).
#[tauri::command]
pub fn export_label_keys() -> Vec<String> {
    arm_rules::export::LABEL_KEYS
        .iter()
        .map(|key| key.to_string())
        .collect()
}

/// Prompts for a file and deserializes the entity from it, returning it paired
/// with its path. Returns `None` if the dialog was cancelled.
///
/// **Gains a `NotLoaded` guard at CV4** (design § 5.6): opening a save now needs
/// a loaded ruleset to migrate against (recognizing a catalogue value's name),
/// which this command never needed before. The `Option<&Ruleset>` read here is
/// threaded straight through to [`ruleset_io::load_entity_from_path`] with no
/// gate of its own, exactly mirroring `export_markdown_to_path` just above:
/// that function does its own `NotLoaded` check on the `Option`, so only the
/// lock-acquisition half of the pair applies here — `None` reports
/// [`AppError::NotLoaded`] rather than panicking or reading a stale ruleset.
#[tauri::command]
pub async fn load_entity(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<OpenedDocument>, AppError> {
    // `load_entity_from_path` (owned by `ruleset_io.rs`) does its own
    // `NotLoaded` check on the `Option`, so only the lock-acquisition half of
    // the pair applies here (mirrors `export_markdown_to_path` above).
    let guard = ruleset_guard(&state);
    let ruleset = guard.as_ref().map(|localized| &localized.ruleset);
    let catalogue_names_guard = state
        .catalogue_names
        .read()
        .expect("catalogue names lock poisoned");

    let path = match e2e_file_override() {
        Some(path) => path,
        None => {
            let dialog = app
                .dialog()
                .file()
                // Accept our own extensions and .json, plus an all-files fallback
                // so a character file is never hidden by the filter.
                .add_filter("Ars Magica character", &["armc", "armcov", "json"])
                .add_filter("All files", &["*"]);
            let Some(file) = parented_to_main_window(&app, dialog).blocking_pick_file() else {
                return Ok(None);
            };
            file.into_path().map_err(|e| AppError::Io {
                message: e.to_string(),
            })?
        }
    };

    // Before the read, for the same reason the save checks before the write: the
    // frontend keeps this path as the current file, so a path it cannot be told
    // about verbatim must be refused rather than mangled (Gerda #5).
    let reported = ruleset_io::path_text(&path)?;

    // A pre-schema-17 save carries no saga year of its own, and the honest value for
    // it is the one the user has configured for new documents — read here, because
    // `arm-rules` has no filesystem and cannot.
    let default_saga_year = read_settings(app).default_saga_year;
    let loaded = ruleset_io::load_entity_from_path(
        &path,
        default_saga_year,
        ruleset,
        catalogue_names_guard.as_ref(),
    )?;
    Ok(Some(opened_document(reported, loaded)))
}

#[cfg(test)]
mod tests {
    use super::{describe_rejected_candidate, require_loaded};
    use crate::error::AppError;
    use std::fs;

    /// [`require_loaded`] is the one place `None` (no `load_ruleset` call yet)
    /// becomes [`AppError::NotLoaded`] — coverage slice, round 3: the function
    /// had no test of any kind before, only its callers' happy paths.
    #[test]
    fn require_loaded_turns_none_into_not_loaded() {
        let err = require_loaded(None).unwrap_err();
        assert!(matches!(err, AppError::NotLoaded), "got {err:?}");
    }

    #[test]
    fn require_loaded_passes_through_a_present_ruleset() {
        let localized = crate::ruleset_io::load_ruleset_from_dir(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules"),
            "en",
        )
        .unwrap();
        let got = require_loaded(Some(&localized)).unwrap();
        assert_eq!(got.ruleset.id, localized.ruleset.id);
    }

    /// [`describe_rejected_candidate`] (V9) must name exactly why a candidate
    /// rules directory was rejected: either it is absent, or it exists but is
    /// missing specific required files — never a bare "not found".
    #[test]
    fn describe_rejected_candidate_names_a_nonexistent_directory() {
        let description =
            describe_rejected_candidate(std::path::Path::new("/does/not/exist/at/all"));
        assert!(
            description.contains("directory does not exist"),
            "got: {description}"
        );
    }

    #[test]
    fn describe_rejected_candidate_names_the_missing_files_of_a_partial_directory() {
        let stale = tempfile::tempdir().unwrap();
        fs::create_dir_all(stale.path().join("core")).unwrap();
        fs::write(stale.path().join("core/character_types.json"), "[]").unwrap();

        let description = describe_rejected_candidate(stale.path());

        assert!(description.contains("missing:"), "got: {description}");
        assert!(
            description.contains("core/ruleset.json"),
            "got: {description}"
        );
        assert!(
            !description.contains("character_types.json"),
            "a present file must not be listed as missing, got: {description}"
        );
    }
}
