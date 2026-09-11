//! The native application menu's *model* — the part that is plain data and can
//! therefore be tested without a window, a GTK context or a Tauri runtime.
//!
//! `arm_app::menu::menu_model` answers "which sections, which items, in which
//! order, with which text and which enabled state" as a pure value; the muda
//! glue that turns that value into a real `tauri::menu::Menu` is the only piece
//! left untested here, and `the_menu_glue_authors_no_text_of_its_own` below
//! keeps it from growing text of its own.

use std::collections::BTreeSet;

use arm_app::menu::{
    ACTION_EXPORT, ACTION_IDS, ACTION_NEW, ACTION_OPEN, ACTION_SAVE, ACTION_SAVE_AS,
    ACTION_SETTINGS, InstalledMenuItem, InstalledMenuSection, MENU_ACTION_EVENT, MenuEntry,
    MenuFlags, MenuLabels, MenuSection, Platform, PredefinedRole, SECTION_APP, SECTION_EDIT,
    SECTION_FILE, SECTION_WINDOW, is_menu_action_id, menu_model,
};

/// A [`MenuLabels`] whose every field carries a unique, recognisable sentinel,
/// plus the set of those sentinels.
///
/// Built by round-tripping through JSON rather than by naming the fields, so
/// adding a label to the struct extends the fixture automatically — a new field
/// nobody wired into the menu then fails
/// [`every_label_field_reaches_at_least_one_platform`] instead of quietly
/// shipping as dead weight.
fn sentinel_labels() -> (MenuLabels, BTreeSet<String>) {
    let mut value = serde_json::to_value(MenuLabels::default()).expect("labels serialize");
    let object = value.as_object_mut().expect("labels are a JSON object");
    let mut sentinels = BTreeSet::new();
    for (field, slot) in object.iter_mut() {
        let sentinel = format!("<<{field}>>");
        sentinels.insert(sentinel.clone());
        *slot = serde_json::Value::String(sentinel);
    }
    let labels: MenuLabels = serde_json::from_value(value).expect("labels deserialize");
    (labels, sentinels)
}

/// Every piece of user-facing text the model carries: submenu titles and item
/// texts alike.
fn texts_in(model: &[MenuSection]) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for section in model {
        found.insert(section.label.clone());
        for item in &section.items {
            match item {
                MenuEntry::Separator => {}
                MenuEntry::Action { label, .. } | MenuEntry::Predefined { label, .. } => {
                    found.insert(label.clone());
                }
            }
        }
    }
    found
}

// CLAUDE.md: no user-facing string may be hardcoded in Rust, and a raw id must
// never be rendered as a label. The menu is the easiest place in the whole app
// to break that — muda takes plain `&str` text — and an English-only menu passes
// every other gate, because nothing else looks. This is the test that looks.
#[test]
fn no_menu_label_is_authored_in_rust() {
    let (labels, sentinels) = sentinel_labels();
    let flags = MenuFlags::default();

    for platform in Platform::ALL {
        for text in texts_in(&menu_model(*platform, &labels, &flags)) {
            assert!(
                sentinels.contains(&text),
                "{platform:?} menu renders {text:?}, which the caller never supplied — \
                 every label must come from MenuLabels"
            );
        }
    }
}

// The other half of the same contract: a label field the menu never reads is a
// string the frontend resolves and translates for nothing, and — worse — a
// reader's evidence that some item is localized when it is not.
#[test]
fn every_label_field_reaches_at_least_one_platform() {
    let (labels, sentinels) = sentinel_labels();
    let flags = MenuFlags::default();

    let mut used = BTreeSet::new();
    for platform in Platform::ALL {
        used.extend(texts_in(&menu_model(*platform, &labels, &flags)));
    }

    let unused: Vec<&String> = sentinels.difference(&used).collect();
    assert!(
        unused.is_empty(),
        "MenuLabels fields nothing renders: {unused:?}"
    );
}

/// Every action id `platform`'s menu offers, in menu order.
fn action_ids(model: &[MenuSection]) -> Vec<String> {
    let mut ids = Vec::new();
    for section in model {
        for item in &section.items {
            if let MenuEntry::Action { id, .. } = item {
                ids.push(id.clone());
            }
        }
    }
    ids
}

/// Every predefined role `platform`'s menu asks the OS for.
fn predefined_roles(model: &[MenuSection]) -> Vec<PredefinedRole> {
    let mut roles = Vec::new();
    for section in model {
        for item in &section.items {
            if let MenuEntry::Predefined { role, .. } = item {
                roles.push(*role);
            }
        }
    }
    roles
}

fn section<'a>(model: &'a [MenuSection], id: &str) -> Option<&'a MenuSection> {
    model.iter().find(|section| section.id == id)
}

fn model_for(platform: Platform, flags: &MenuFlags) -> Vec<MenuSection> {
    let (labels, _) = sentinel_labels();
    menu_model(platform, &labels, flags)
}

fn all_enabled() -> MenuFlags {
    MenuFlags {
        new: true,
        open: true,
        save: true,
        save_as: true,
        export: true,
        settings: true,
    }
}

// The whole point of the slice: every document action is reachable from the
// menu bar, on every desktop, in the order a File menu is read in. Since C3c
// removed the in-app toolbar this is no longer a second route to them — it is
// the primary one, the keyboard shortcuts being the other.
#[test]
fn the_file_menu_offers_every_document_action_on_every_platform() {
    for platform in Platform::ALL {
        let model = model_for(*platform, &all_enabled());
        let file = section(&model, SECTION_FILE)
            .unwrap_or_else(|| panic!("{platform:?} has no File section"));
        let ids: Vec<&str> = file
            .items
            .iter()
            .filter_map(|item| match item {
                MenuEntry::Action { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            &ids[..5],
            &[
                ACTION_NEW,
                ACTION_OPEN,
                ACTION_SAVE,
                ACTION_SAVE_AS,
                ACTION_EXPORT
            ],
            "{platform:?} File menu"
        );
    }
}

// Stated placement, not guessed: macOS puts Settings in the application menu,
// every other desktop puts it in File.
#[test]
fn settings_sits_under_the_app_menu_on_macos_and_under_file_elsewhere() {
    let mac = model_for(Platform::MacOs, &all_enabled());
    let app = section(&mac, SECTION_APP).expect("macOS has an application section");
    assert!(action_ids(std::slice::from_ref(app)).contains(&ACTION_SETTINGS.to_string()));
    let mac_file = section(&mac, SECTION_FILE).expect("macOS has a File section");
    assert!(!action_ids(std::slice::from_ref(mac_file)).contains(&ACTION_SETTINGS.to_string()));

    for platform in [Platform::Windows, Platform::Other] {
        let model = model_for(platform, &all_enabled());
        assert!(
            section(&model, SECTION_APP).is_none(),
            "{platform:?} must not grow a macOS-only application menu"
        );
        let file = section(&model, SECTION_FILE).expect("File section");
        assert!(
            action_ids(std::slice::from_ref(file)).contains(&ACTION_SETTINGS.to_string()),
            "{platform:?} must offer Settings under File"
        );
    }
}

// A native menu reaches neither the buttons' `disabled` nor the window-level
// keydown gate, so the store's single predicate has to arrive here as data —
// and it has to land on the right item.
#[test]
fn a_cleared_flag_disables_exactly_its_own_action() {
    for cleared in ACTION_IDS {
        let mut flags = all_enabled();
        match *cleared {
            ACTION_NEW => flags.new = false,
            ACTION_OPEN => flags.open = false,
            ACTION_SAVE => flags.save = false,
            ACTION_SAVE_AS => flags.save_as = false,
            ACTION_EXPORT => flags.export = false,
            ACTION_SETTINGS => flags.settings = false,
            other => panic!("unmapped action id {other}"),
        }

        for platform in Platform::ALL {
            for section in model_for(*platform, &flags) {
                for item in section.items {
                    if let MenuEntry::Action { id, enabled, .. } = item {
                        assert_eq!(
                            enabled,
                            id != *cleared,
                            "{platform:?}: clearing {cleared} left {id} enabled={enabled}"
                        );
                    }
                }
            }
        }
    }
}

// muda 0.19.3's GTK backend renders only Separator/Copy/Cut/Paste/SelectAll/
// About as predefined items and silently drops the rest, and even those four
// are inert unless tauri's `linux-libxdo` feature is on (which it is not). An
// Edit menu there would be three dead entries and a Window menu would be empty,
// so this desktop gets the File menu it can actually honour.
#[test]
fn the_linux_menu_asks_the_os_for_nothing_gtk_cannot_honour() {
    let model = model_for(Platform::Other, &all_enabled());

    assert_eq!(
        predefined_roles(&model),
        Vec::<PredefinedRole>::new(),
        "the GTK menu must contain no predefined item"
    );
    let ids: Vec<&str> = model.iter().map(|section| section.id.as_str()).collect();
    assert_eq!(ids, vec![SECTION_FILE]);
}

// The unsaved-changes guard is a MANDATORY product behavior (CLAUDE.md), and a
// Quit item is the one menu entry that can walk straight past it. macOS's
// predefined Quit is the very item tao's own default menu installs, so it keeps
// routing through `RunEvent::ExitRequested` exactly as Cmd+Q does today.
// Windows' predefined Quit is `PostQuitMessage(0)`
// (muda-0.19.3/src/platform_impl/windows/mod.rs:1223) — it ends the message
// loop rather than raising a close request, so it must never appear.
#[test]
fn only_macos_offers_quit_and_only_through_the_predefined_item() {
    let mac = model_for(Platform::MacOs, &all_enabled());
    let app = section(&mac, SECTION_APP).expect("macOS application section");
    assert_eq!(
        app.items.last(),
        Some(&MenuEntry::Predefined {
            role: PredefinedRole::Quit,
            label: "<<quit>>".to_string(),
        }),
    );

    for platform in [Platform::Windows, Platform::Other] {
        assert!(
            !predefined_roles(&model_for(platform, &all_enabled())).contains(&PredefinedRole::Quit),
            "{platform:?} must not offer a Quit that bypasses the unsaved-changes guard"
        );
    }
}

// Ids cross the IPC boundary and are matched by the frontend, so they are a
// contract: stable slugs, never text, never repeated.
#[test]
fn section_and_action_ids_are_unique_stable_slugs() {
    assert_eq!(
        ACTION_IDS,
        [
            "menu.new",
            "menu.open",
            "menu.save",
            "menu.save-as",
            "menu.export",
            "menu.settings"
        ]
    );
    assert_eq!(
        [SECTION_APP, SECTION_FILE, SECTION_EDIT, SECTION_WINDOW],
        ["menu.app", "menu.file", "menu.edit", "menu.window"]
    );

    for platform in Platform::ALL {
        let model = model_for(*platform, &all_enabled());
        let ids = action_ids(&model);
        let unique: BTreeSet<&String> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "{platform:?} repeats an action id");
        for id in &ids {
            assert!(
                ACTION_IDS.contains(&id.as_str()),
                "{platform:?} invents {id}"
            );
        }
    }
}

/// `ui/src/lib/menu.ts` — the frontend half of the menu contract, read as text
/// because the two halves are separate languages with nothing but this test
/// holding them together. Same shape as `main.rs`'s
/// `the_shadow_script_invokes_the_registered_request_close_command`.
fn frontend_menu_module() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../ui/src/lib/menu.ts");
    std::fs::read_to_string(path).expect("ui/src/lib/menu.ts is readable")
}

/// The field names of a serde struct, as the frontend must spell them.
fn serialized_fields<T: serde::Serialize>(value: &T) -> BTreeSet<String> {
    serde_json::to_value(value)
        .expect("serializes")
        .as_object()
        .expect("is a JSON object")
        .keys()
        .cloned()
        .collect()
}

/// The property names declared by a `export interface <name> {` block.
fn interface_fields(source: &str, name: &str) -> BTreeSet<String> {
    let header = format!("export interface {name} {{");
    let start = source
        .find(&header)
        .unwrap_or_else(|| panic!("menu.ts declares no {name}"))
        + header.len();
    let body = &source[start..];
    let end = body.find("\n}").expect("interface block is closed");
    body[..end]
        .lines()
        .filter_map(|line| line.trim().split_once(':'))
        .map(|(field, _)| field.trim().to_string())
        .filter(|field| !field.is_empty() && !field.starts_with("//") && !field.starts_with('*'))
        .collect()
}

// Rust and the frontend hold two independently-typed copies of one contract:
// the struct fields the labels arrive in, the ids the menu emits, and the event
// name they travel on. Nothing but this test connects them — a rename on either
// side would compile, pass every other gate, and produce a menu whose labels
// are all `undefined` or whose clicks reach nobody.
#[test]
fn the_frontend_and_rust_agree_on_the_menu_contract() {
    let source = frontend_menu_module();

    assert!(
        source.contains(&format!("'{MENU_ACTION_EVENT}'")),
        "menu.ts does not listen on {MENU_ACTION_EVENT}"
    );
    for id in ACTION_IDS {
        assert!(
            source.contains(&format!("'{id}'")),
            "menu.ts does not route the menu id {id}"
        );
    }
    assert_eq!(
        interface_fields(&source, "MenuLabels"),
        serialized_fields(&MenuLabels::default()),
        "MenuLabels has drifted between Rust and the frontend"
    );
}

// Building the menu is the one thing `set_app_menu` can fail at, and it fails
// where nothing else in this app does — in the window system, not in a file.
// Reusing `AppError::Io` would have the banner tell the user a file could not
// be read; the frontend maps `kind` straight to `error-<kind>`, so the kind IS
// the message it picks.
#[test]
fn a_menu_failure_reports_under_its_own_kind() {
    let error = arm_app::error::AppError::Menu {
        message: "gtk said no".to_string(),
    };
    let json = serde_json::to_value(&error).expect("AppError serializes");

    assert_eq!(
        json.get("kind").and_then(|kind| kind.as_str()),
        Some("menu")
    );
    assert!(error.to_string().contains("gtk said no"));
}

/// `menu.rs`, from the muda glue down — the one stretch of the module that
/// `no_menu_label_is_authored_in_rust` cannot reach, because it needs a window
/// to run.
fn glue_source() -> &'static str {
    let source = include_str!("../src/menu.rs");
    let start = source
        .find("pub fn build_menu")
        .expect("menu.rs still defines build_menu");
    &source[start..]
}

// `no_menu_label_is_authored_in_rust` proves the MODEL invents no text; this
// covers the remaining stretch, which hands that text to muda and is the one
// piece no unit test can execute (a real `Menu` needs a window and a GTK
// context). muda takes plain `&str`, so a single stray literal there is all it
// would take to ship an unlocalizable item — and every other gate would stay
// green. Comment lines are exempt; live code there has no business holding a
// string at all.
#[test]
fn the_menu_glue_authors_no_text_of_its_own() {
    for (offset, line) in glue_source().lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") || code.starts_with("#[") {
            continue;
        }
        assert!(
            !code.contains('"'),
            "menu.rs glue line {} holds a string literal: {line}",
            offset + 1
        );
    }
}

// C6, seam 1. `activate_menu_item` forwards an id onto the app's event bus —
// the very line the OS handler runs — so the set of ids it will forward has to
// be the set the menu can actually emit. Without this guard the seam is a
// general-purpose "emit anything on `menu://action`" hole rather than a way to
// press one of six items, and a typo'd id in a spec would look like a silently
// ignored click instead of a mistake.
#[test]
fn the_activation_seam_accepts_exactly_the_ids_the_menu_can_emit() {
    for id in ACTION_IDS {
        assert!(
            is_menu_action_id(id),
            "{id} is on the menu but the activation seam would not forward it"
        );
    }

    // A section id is a real menu id and still not an action: choosing a
    // submenu opens it, it never reaches `on_menu_event`.
    for section in [SECTION_APP, SECTION_FILE, SECTION_EDIT, SECTION_WINDOW] {
        assert!(
            !is_menu_action_id(section),
            "{section} is a submenu title, not an action the frontend can run"
        );
    }
    assert!(!is_menu_action_id("menu.nonsense"));
    assert!(!is_menu_action_id(""));
}

// C6, seam 2. The read-back is consumed by a WebdriverIO spec through JSON, so
// its wire shape is a contract with a file the compiler never sees. Pinned here
// in the same spirit as `the_frontend_and_rust_agree_on_the_menu_contract`.
//
// `Other` is not an evasion, it is the API's limit stated honestly: tauri 2.11.3
// exposes only `id()` and `text()` on a `PredefinedMenuItem` — no `is_enabled()`
// and no way to ask which predefined role it plays — so a separator and an
// OS-implemented item can be reported by their text and their position, and by
// nothing else. Their muda-generated numeric ids are not carried either: they
// are a per-process counter, so they would differ on every run and make the
// value uncomparable.
#[test]
fn the_installed_menu_reads_back_in_the_shape_the_spec_parses() {
    let section = InstalledMenuSection {
        title: "Datei".to_string(),
        items: vec![
            InstalledMenuItem::Action {
                id: ACTION_SAVE.to_string(),
                title: "Speichern".to_string(),
                enabled: false,
            },
            InstalledMenuItem::Other {
                title: String::new(),
            },
        ],
    };

    assert_eq!(
        serde_json::to_value(&section).expect("the read-back serializes"),
        serde_json::json!({
            "title": "Datei",
            "items": [
                { "kind": "action", "id": "menu.save", "title": "Speichern", "enabled": false },
                { "kind": "other", "title": "" },
            ],
        })
    );
}
