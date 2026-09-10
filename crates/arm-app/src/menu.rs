//! The native application menu.
//!
//! Split deliberately in two. [`menu_model`] answers "which sections, which
//! items, in which order, with which text and which enabled state" as a **pure
//! value**, so the whole arrangement is unit-testable with no window, no GTK
//! context and no Tauri runtime (`crates/arm-app/tests/menu.rs` exercises all
//! three desktops from Linux). [`build_menu`] is the muda glue that turns that
//! value into a real [`tauri::menu::Menu`], and does nothing else — in
//! particular it authors no text, which
//! `the_menu_glue_authors_no_text_of_its_own` keeps true.
//!
//! **No user-facing string originates here** (CLAUDE.md). Every label arrives
//! in [`MenuLabels`], resolved from Fluent by the frontend, exactly as
//! `CloseGuardLabels` already does for the discard-confirmation dialog — and
//! because it is a struct of required fields rather than a lookup map, adding
//! an item is a compile error until the frontend supplies its text. The
//! frontend also rebuilds the menu when the UI language changes, so a runtime
//! language switch retitles it.

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Manager, Runtime};

pub const SECTION_APP: &str = "menu.app";
pub const SECTION_FILE: &str = "menu.file";
pub const SECTION_EDIT: &str = "menu.edit";
pub const SECTION_WINDOW: &str = "menu.window";

pub const ACTION_NEW: &str = "menu.new";
pub const ACTION_OPEN: &str = "menu.open";
pub const ACTION_SAVE: &str = "menu.save";
pub const ACTION_SAVE_AS: &str = "menu.save-as";
pub const ACTION_EXPORT: &str = "menu.export";
pub const ACTION_SETTINGS: &str = "menu.settings";

/// Every action id the menu can emit. The frontend maps each to the very store
/// action its toolbar button calls; see `ui/src/lib/menu.ts`.
pub const ACTION_IDS: &[&str] = &[
    ACTION_NEW,
    ACTION_OPEN,
    ACTION_SAVE,
    ACTION_SAVE_AS,
    ACTION_EXPORT,
    ACTION_SETTINGS,
];

/// The Tauri event a chosen menu item is announced on. A menu click is handled
/// in Rust but *acted on* in the frontend, which owns the document, so the id
/// is forwarded rather than reimplemented here.
///
/// The frontend half of this name lives in `ui/src/lib/menu.ts`; they are two
/// independently-typed literals with nothing tying them together, so
/// `the_frontend_listens_on_the_event_rust_emits` pins them, in the same shape
/// as `main.rs`'s `REQUEST_CLOSE_COMMAND` check.
pub const MENU_ACTION_EVENT: &str = "menu://action";

/// Which desktop the menu is being built for.
///
/// An explicit value rather than `cfg!` scattered through [`menu_model`], so
/// the macOS and Windows arrangements are testable from a Linux developer
/// machine and a CI runner instead of only being compiled there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOs,
    Windows,
    /// Linux and the BSDs — every desktop whose menu backend is GTK.
    Other,
}

impl Platform {
    pub const ALL: &'static [Platform] = &[Platform::MacOs, Platform::Windows, Platform::Other];

    /// The desktop this binary was compiled for.
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Other
        }
    }
}

/// Localized menu text, resolved from Fluent by the frontend and deserialized
/// verbatim. Rust authors none of it.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuLabels {
    /// Title of the macOS application menu — the app's own name.
    pub app: String,
    pub file: String,
    pub edit: String,
    pub window: String,
    pub new: String,
    pub open: String,
    pub save: String,
    pub save_as: String,
    pub export: String,
    pub settings: String,
    pub quit: String,
    pub services: String,
    pub hide: String,
    pub hide_others: String,
    pub show_all: String,
    pub undo: String,
    pub redo: String,
    pub cut: String,
    pub copy: String,
    pub paste: String,
    pub select_all: String,
    pub minimize: String,
    pub fullscreen: String,
    pub close_window: String,
}

/// Whether each document action may run right now.
///
/// A native menu reaches neither the toolbar buttons' `disabled` attribute nor
/// the window-level keydown gate, so the store's single availability predicate
/// is mirrored here as data and rendered as the items' enabled state.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuFlags {
    pub new: bool,
    pub open: bool,
    pub save: bool,
    pub save_as: bool,
    pub export: bool,
    pub settings: bool,
}

/// A menu item the OS implements. Exhaustive `match` in [`build_menu`], so
/// adding a role is a compile error until it is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredefinedRole {
    Services,
    Hide,
    HideOthers,
    ShowAll,
    Quit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Minimize,
    Fullscreen,
    CloseWindow,
}

/// One entry of a submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    Separator,
    /// An item this app handles itself, announced on [`MENU_ACTION_EVENT`].
    Action {
        id: String,
        label: String,
        enabled: bool,
    },
    Predefined {
        role: PredefinedRole,
        label: String,
    },
}

/// One top-level submenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuSection {
    pub id: String,
    pub label: String,
    pub items: Vec<MenuEntry>,
}

fn action(id: &str, label: &str, enabled: bool) -> MenuEntry {
    MenuEntry::Action {
        id: id.to_string(),
        label: label.to_string(),
        enabled,
    }
}

fn predefined(role: PredefinedRole, label: &str) -> MenuEntry {
    MenuEntry::Predefined {
        role,
        label: label.to_string(),
    }
}

fn section(id: &str, label: &str, items: Vec<MenuEntry>) -> MenuSection {
    MenuSection {
        id: id.to_string(),
        label: label.to_string(),
        items,
    }
}

/// The document actions every desktop's File menu opens with.
fn document_actions(labels: &MenuLabels, flags: &MenuFlags) -> Vec<MenuEntry> {
    vec![
        action(ACTION_NEW, &labels.new, flags.new),
        action(ACTION_OPEN, &labels.open, flags.open),
        MenuEntry::Separator,
        action(ACTION_SAVE, &labels.save, flags.save),
        action(ACTION_SAVE_AS, &labels.save_as, flags.save_as),
        MenuEntry::Separator,
        action(ACTION_EXPORT, &labels.export, flags.export),
    ]
}

/// The Edit submenu, which is entirely OS-implemented.
fn edit_section(labels: &MenuLabels) -> MenuSection {
    section(
        SECTION_EDIT,
        &labels.edit,
        vec![
            predefined(PredefinedRole::Undo, &labels.undo),
            predefined(PredefinedRole::Redo, &labels.redo),
            MenuEntry::Separator,
            predefined(PredefinedRole::Cut, &labels.cut),
            predefined(PredefinedRole::Copy, &labels.copy),
            predefined(PredefinedRole::Paste, &labels.paste),
            predefined(PredefinedRole::SelectAll, &labels.select_all),
        ],
    )
}

/// The Window submenu. `CloseWindow` is a close *request*, so it reaches the
/// unsaved-changes guard in `main.rs` like the title-bar X does.
fn window_section(labels: &MenuLabels) -> MenuSection {
    section(
        SECTION_WINDOW,
        &labels.window,
        vec![
            predefined(PredefinedRole::Minimize, &labels.minimize),
            predefined(PredefinedRole::Fullscreen, &labels.fullscreen),
            MenuEntry::Separator,
            predefined(PredefinedRole::CloseWindow, &labels.close_window),
        ],
    )
}

/// The menu `platform` should show.
///
/// The three arrangements differ for reasons of platform convention and, on
/// GTK, of what muda can actually render:
///
/// * **macOS** — Settings and Quit belong in the application menu, never in
///   File. The predefined Quit is the same item tao's own default menu
///   installs, so replacing that menu does not move Cmd+Q off
///   `RunEvent::ExitRequested` and away from the unsaved-changes guard.
/// * **Windows** — Settings goes in File. There is deliberately **no** Quit:
///   muda's Windows backend implements the predefined Quit as
///   `PostQuitMessage(0)` (`muda-0.19.3/src/platform_impl/windows/mod.rs:1223`),
///   which ends the message loop instead of raising a close request, so it
///   would walk straight past the guard. Window → Close Window posts `WM_CLOSE`
///   (`:1220`) and is guarded, so that is the offered way out.
/// * **Linux/BSD** — File only. muda's GTK backend renders just Separator,
///   Copy, Cut, Paste, SelectAll and About and silently drops every other
///   predefined item (`muda-0.19.3/src/platform_impl/gtk/mod.rs:36-43`), and
///   even those four are inert unless tauri's `linux-libxdo` feature is
///   enabled, which it is not. An Edit menu there would be four dead entries
///   and a Window menu would be empty.
pub fn menu_model(platform: Platform, labels: &MenuLabels, flags: &MenuFlags) -> Vec<MenuSection> {
    let mut file_items = document_actions(labels, flags);
    if platform != Platform::MacOs {
        file_items.push(MenuEntry::Separator);
        file_items.push(action(ACTION_SETTINGS, &labels.settings, flags.settings));
    }
    let file = section(SECTION_FILE, &labels.file, file_items);

    match platform {
        Platform::MacOs => vec![
            section(
                SECTION_APP,
                &labels.app,
                vec![
                    action(ACTION_SETTINGS, &labels.settings, flags.settings),
                    MenuEntry::Separator,
                    predefined(PredefinedRole::Services, &labels.services),
                    predefined(PredefinedRole::Hide, &labels.hide),
                    predefined(PredefinedRole::HideOthers, &labels.hide_others),
                    predefined(PredefinedRole::ShowAll, &labels.show_all),
                    MenuEntry::Separator,
                    predefined(PredefinedRole::Quit, &labels.quit),
                ],
            ),
            file,
            edit_section(labels),
            window_section(labels),
        ],
        Platform::Windows => vec![file, edit_section(labels), window_section(labels)],
        Platform::Other => vec![file],
    }
}

/// Turn a [`menu_model`] value into a real menu. Pure glue: it copies text, it
/// never writes any.
pub fn build_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    model: &[MenuSection],
) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(manager)?;
    for section in model {
        let submenu = Submenu::new(manager, &section.label, true)?;
        for entry in &section.items {
            match entry {
                MenuEntry::Separator => {
                    submenu.append(&PredefinedMenuItem::separator(manager)?)?;
                }
                MenuEntry::Action { id, label, enabled } => {
                    submenu.append(&MenuItem::with_id(
                        manager,
                        id,
                        label,
                        *enabled,
                        None::<&str>,
                    )?)?;
                }
                MenuEntry::Predefined { role, label } => {
                    submenu.append(&predefined_item(manager, *role, label)?)?;
                }
            }
        }
        menu.append(&submenu)?;
    }
    Ok(menu)
}

/// Build the menu this platform should show and make it the app's, replacing
/// whatever was installed before. Called again on every language switch and on
/// every change of the document-action gate.
pub fn install_menu<R: Runtime>(
    app: &tauri::AppHandle<R>,
    labels: &MenuLabels,
    flags: &MenuFlags,
) -> tauri::Result<()> {
    let model = menu_model(Platform::current(), labels, flags);
    let menu = build_menu(app, &model)?;
    app.set_menu(menu)?;
    Ok(())
}

fn predefined_item<R: Runtime, M: Manager<R>>(
    manager: &M,
    role: PredefinedRole,
    label: &str,
) -> tauri::Result<PredefinedMenuItem<R>> {
    let text = Some(label);
    match role {
        PredefinedRole::Services => PredefinedMenuItem::services(manager, text),
        PredefinedRole::Hide => PredefinedMenuItem::hide(manager, text),
        PredefinedRole::HideOthers => PredefinedMenuItem::hide_others(manager, text),
        PredefinedRole::ShowAll => PredefinedMenuItem::show_all(manager, text),
        PredefinedRole::Quit => PredefinedMenuItem::quit(manager, text),
        PredefinedRole::Undo => PredefinedMenuItem::undo(manager, text),
        PredefinedRole::Redo => PredefinedMenuItem::redo(manager, text),
        PredefinedRole::Cut => PredefinedMenuItem::cut(manager, text),
        PredefinedRole::Copy => PredefinedMenuItem::copy(manager, text),
        PredefinedRole::Paste => PredefinedMenuItem::paste(manager, text),
        PredefinedRole::SelectAll => PredefinedMenuItem::select_all(manager, text),
        PredefinedRole::Minimize => PredefinedMenuItem::minimize(manager, text),
        PredefinedRole::Fullscreen => PredefinedMenuItem::fullscreen(manager, text),
        PredefinedRole::CloseWindow => PredefinedMenuItem::close_window(manager, text),
    }
}
