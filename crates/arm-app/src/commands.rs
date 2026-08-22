//! Thin `#[tauri::command]` shims. They resolve the rules directory and managed
//! state, then delegate to the webview-free logic in [`crate::ruleset_io`].

use std::sync::{Mutex, RwLock, RwLockReadGuard};

use arm_rules::{Characteristic, Entity, Id, LocalizedRuleset, ValidationMode, ValidationResult};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

use arm_rules::DerivedTotals;

use crate::error::AppError;
use crate::ruleset_io;
use crate::ruleset_io::{
    AgingApplication, AgingProjection, AgingReversion, ChildhoodApplication, EffectiveScores,
};

/// Holds the parsed, localized ruleset so validation does not re-read and
/// re-check the rules files on every keystroke. `None` until `load_ruleset`
/// succeeds.
///
/// The whole [`LocalizedRuleset`] is cached, not just its mechanics: the
/// Markdown export needs the localized display names, and re-reading the i18n
/// files per export would duplicate the loader for no gain.
#[derive(Default)]
pub struct AppState {
    pub ruleset: RwLock<Option<LocalizedRuleset>>,
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
    pub title: String,
    pub message: String,
    pub discard: String,
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

/// Mirrors the frontend's dirty flag and dialog strings into managed state for
/// the close/quit guard (see `main.rs`).
#[tauri::command]
pub fn update_close_guard(dirty: bool, labels: CloseGuardLabels, state: State<'_, AppState>) {
    let mut guard = state.close_guard.lock().expect("close guard lock poisoned");
    guard.report_dirty_state(dirty, labels);
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
            "rules directory not found; looked in: {}",
            candidates
                .iter()
                .map(|c| c.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    })?;

    let localized = ruleset_io::load_ruleset_from_dir(&rules_dir, &lang)?;

    *state.ruleset.write().expect("ruleset lock poisoned") = Some(localized.clone());

    Ok(localized)
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
    Ok(ruleset_io::effective_scores_loaded(
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
/// row sent there (`:16621`); `null` records the Crisis as owed and unrolled,
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

#[cfg(feature = "e2e-testing")]
pub fn e2e_file_override() -> Option<std::path::PathBuf> {
    std::env::var_os(E2E_FILE_ENV).map(std::path::PathBuf::from)
}

#[cfg(not(feature = "e2e-testing"))]
pub fn e2e_file_override() -> Option<std::path::PathBuf> {
    None
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
/// the frontend can track it as the "current file" for subsequent direct saves.
#[derive(serde::Serialize)]
pub struct LoadedEntity {
    pub path: String,
    pub entity: Entity,
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

    ruleset_io::save_entity_to_path(&entity, &target)?;
    Ok(Some(target.to_string_lossy().into_owned()))
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
#[tauri::command]
pub async fn load_entity(app: AppHandle) -> Result<Option<LoadedEntity>, AppError> {
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

    let entity = ruleset_io::load_entity_from_path(&path)?;
    Ok(Some(LoadedEntity {
        path: path.to_string_lossy().into_owned(),
        entity,
    }))
}
