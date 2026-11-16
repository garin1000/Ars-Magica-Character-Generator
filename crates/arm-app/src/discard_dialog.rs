//! The F5 native discard-confirmation dialog ("discard unsaved changes?"),
//! built directly per platform instead of through `tauri-plugin-dialog`/rfd's
//! shared message-dialog path (`main.rs::guard_blocks_quit`,
//! `commands::confirm_discard`). `CLAUDE.md`'s "Unsaved-changes guard" names
//! this a mandatory product behaviour; this module exists because Enter —
//! the default on every desktop — silently discarded the user's work instead
//! of cancelling.
//!
//! **Why not the plugin.** Confirmed by reading the pinned versions this
//! workspace actually resolves (`Cargo.lock`):
//! - Windows: `rfd-0.16.0/src/backend/win_cid/message_dialog.rs:156`
//!   hardcodes `nDefaultButton: 0`, which matches none of the dialog's own
//!   custom button ids, so by `TaskDialogIndirect`'s own documented fallback
//!   the FIRST button in `pButtons` — Discard — becomes the Enter default.
//! - macOS: `rfd-0.16.0/src/backend/macos/message_dialog.rs:62-64` adds
//!   `NSAlert` buttons in the order the `MessageButtons` variant lists them,
//!   and `NSAlert`'s own Return-key default is always the FIRST button
//!   added — so whichever label comes first in `OkCancelCustom(a, b)` is the
//!   default, regardless of which one means Discard.
//! - Linux: `rfd-0.16.0/src/backend/gtk3/message_dialog.rs` never calls GTK's
//!   `set_default_response` at all, so the toolkit's own fallback (tab order),
//!   not this app, decides.
//!
//! Swapping the label order is not a fix: `tauri-plugin-dialog-2.7.1/src/desktop.rs:236-239`
//! maps a plain `rfd::MessageDialogResult::Cancel` (Esc, the window's close
//! button) to the dialog's SECOND label, so reordering the labels to fix the
//! default would silently flip what Esc does instead.
//!
//! **The fix.** Build the dialog directly per platform, so each one's own
//! default-button mechanism can be pointed at Cancel: a direct
//! `gtk::MessageDialog` with `set_default_response(Cancel)` on Linux, a direct
//! `TaskDialogIndirect` call with `nDefaultButton` set to the Cancel button's
//! own id on Windows, and rfd called directly with the labels ordered
//! `[cancel, discard]` on macOS, so `NSAlert`'s "first button added" default
//! lands on Cancel — with the result mapped by hand rather than through the
//! plugin.
//!
//! Every platform backend funnels its own native answer through exactly one
//! of the `*_to_outcome` functions below into a single [`Outcome`], so the
//! callback `show_discard_dialog` will hand to its caller never has to know
//! which platform answered — any platform error, a destroyed dialog, or any
//! response that is not an exact match for the Discard button/label all map
//! to [`Outcome::Cancel`], never [`Outcome::Discard`].
//!
//! Each mapping takes a plain, already-cross-platform type — `i32` for the
//! Windows button id, `rfd::MessageDialogResult` (a type that compiles on
//! every target, since `tauri-plugin-dialog` already resolves it everywhere) —
//! rather than an opaque platform handle, specifically so it is unit-testable
//! on every developer machine and not only its own OS: the same reasoning
//! `menu.rs`'s `Platform` argument already uses for `accelerator_for`. The GTK
//! mapping is the one exception, gated to Linux, because `gtk::ResponseType`
//! genuinely only exists there — and this machine IS Linux, so it still
//! compiles and runs here.
//!
//! Return to `tauri-plugin-dialog`/rfd once rfd supports choosing the default
//! button and the plugin maps Esc to Cancel — rfd hardcodes
//! `nDefaultButton: 0`, still true as of 0.18.0 (open-todos row 62).

use crate::commands::CloseGuardLabels;

use tauri::Manager;

/// The user's answer to a native discard-confirmation dialog, decoupled from
/// any one platform's own button/response vocabulary. Every per-platform
/// backend in this module (see the module doc) funnels its own native result
/// through exactly one of the `*_to_outcome` functions below before handing it
/// to `show_discard_dialog`'s callback, so the caller —
/// `commands::confirm_discard` / `main.rs::guard_blocks_quit` — never
/// branches on a platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The user chose to discard the unsaved changes.
    Discard,
    /// The user chose to keep them — or the dialog could not say otherwise: a
    /// platform error, a destroyed dialog, or any response that is not an
    /// exact match for the Discard button/label all map here, never to
    /// `Discard`. See each `*_to_outcome` function's own doc comment.
    Cancel,
}

impl Outcome {
    /// Whether this outcome discards the unsaved changes — the single `bool`
    /// `commands::apply_dialog_answer`/`resolve_discard_dialog` already
    /// consume, so wiring `show_discard_dialog`'s callback into them needs no
    /// second decision layer.
    pub fn discards(self) -> bool {
        matches!(self, Outcome::Discard)
    }
}

/// Shows the F5 discard-confirmation dialog without blocking the calling
/// thread, and hands the user's answer to `callback` **exactly once** — on
/// every path, including a platform error, which always resolves to
/// [`Outcome::Cancel`] (see the module doc) and never silently drops the
/// callback. Both call sites — `main.rs::guard_blocks_quit` (window close /
/// `Cmd+Q`) and `commands::confirm_discard` (New/Open) — share this one entry
/// point, so the dialog is confirmed exactly one way whatever triggered it,
/// same as before this module existed.
pub fn show_discard_dialog(
    app: &tauri::AppHandle,
    labels: CloseGuardLabels,
    callback: impl FnOnce(Outcome) + Send + 'static,
) {
    #[cfg(target_os = "linux")]
    {
        show_discard_dialog_gtk(app, labels, callback);
    }
    #[cfg(target_os = "windows")]
    {
        show_discard_dialog_windows(app, labels, callback);
    }
    #[cfg(target_os = "macos")]
    {
        show_discard_dialog_macos(app, labels, callback);
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = (app, labels);
        callback(Outcome::Cancel);
    }
}

/// A boxed, one-shot discard-dialog callback, held by the GTK backend until it
/// has an [`Outcome`] to hand it.
#[cfg(target_os = "linux")]
type DiscardCallback = Box<dyn FnOnce(Outcome) + Send>;

/// [`DiscardCallback`], shared so a dispatch-failure path can still invoke it
/// if the closure meant to call it never runs at all (see
/// [`show_discard_dialog_gtk`]).
#[cfg(target_os = "linux")]
type SharedDiscardCallback = std::sync::Arc<std::sync::Mutex<Option<DiscardCallback>>>;

/// Linux backend: a modal, transient `gtk::MessageDialog` with Cancel as the
/// default response (see the module doc for why this bypasses rfd's GTK3
/// backend, which never sets one at all).
#[cfg(target_os = "linux")]
fn show_discard_dialog_gtk(
    app: &tauri::AppHandle,
    labels: CloseGuardLabels,
    callback: impl FnOnce(Outcome) + Send + 'static,
) {
    use std::sync::{Arc, Mutex};

    use gtk::prelude::*;

    // `run_on_main_thread`'s closure must be `Send`, which a plain `Rc`/`RefCell`
    // is not — this is shared so the dispatch-failure branch below can still
    // invoke the callback if the closure below never runs at all.
    let callback: SharedDiscardCallback = Arc::new(Mutex::new(Some(Box::new(callback))));
    let finish = Arc::clone(&callback);
    let main_thread_app = app.clone();

    let dispatched = app.run_on_main_thread(move || {
        // `gtk_window()`'s own doc: "this type can only be used on the main
        // thread" — which this closure, run via `run_on_main_thread`, is.
        let parent = main_thread_app
            .get_webview_window("main")
            .and_then(|window| window.gtk_window().ok());

        let dialog = gtk::MessageDialog::new(
            parent.as_ref(),
            gtk::DialogFlags::MODAL | gtk::DialogFlags::DESTROY_WITH_PARENT,
            gtk::MessageType::Warning,
            gtk::ButtonsType::None,
            &labels.message,
        );
        dialog.set_title(&labels.title);
        dialog.add_button(&labels.discard, gtk::ResponseType::Accept);
        dialog.add_button(&labels.cancel, gtk::ResponseType::Cancel);
        // The one line rfd's GTK3 backend never calls at all (module doc) —
        // Cancel, not Discard, is now the Enter/Space default.
        dialog.set_default_response(gtk::ResponseType::Cancel);

        // GTK's own `GtkDialog` turns the window's close button (and Escape,
        // since no accelerator overrides it) into a `response` signal carrying
        // `ResponseType::DeleteEvent` by default, so this one handler covers
        // every dismissal path without a separate `delete-event` connection.
        dialog.connect_response(move |dialog, response| {
            // SAFETY: nothing on `dialog` is queried after this call — the
            // handler returns immediately afterwards.
            unsafe {
                dialog.destroy();
            }
            let callback = finish
                .lock()
                .expect("discard dialog callback lock poisoned")
                .take();
            if let Some(callback) = callback {
                callback(gtk_response_to_outcome(response, gtk::ResponseType::Accept));
            }
        });
        dialog.show();
    });

    if dispatched.is_err() {
        // `run_on_main_thread` itself failed (the module doc's "any platform
        // error" rule) — the closure above never ran, so nothing else will
        // ever call this callback.
        let callback = callback
            .lock()
            .expect("discard dialog callback lock poisoned")
            .take();
        if let Some(callback) = callback {
            callback(Outcome::Cancel);
        }
    }
}

/// Windows backend: `TaskDialogIndirect` on a dedicated thread, with
/// `nDefaultButton` pointed at the Cancel button's own id (see the module doc
/// for why this bypasses rfd, which hardcodes `nDefaultButton: 0`).
#[cfg(target_os = "windows")]
fn show_discard_dialog_windows(
    app: &tauri::AppHandle,
    labels: CloseGuardLabels,
    callback: impl FnOnce(Outcome) + Send + 'static,
) {
    let owner = SendHwnd(
        app.get_webview_window("main")
            .and_then(|window| window.hwnd().ok())
            .map_or(std::ptr::null_mut(), |hwnd| hwnd.0),
    );

    // A dedicated thread: `TaskDialogIndirect` pumps its own nested message
    // loop while the dialog is open, so this must never run on the caller's
    // own thread (the module doc's "never block the calling thread" rule) —
    // the same reason `confirm_discard`'s predecessor never called
    // `blocking_show` from the main thread either.
    std::thread::spawn(move || {
        // Rebind the whole wrapper first: edition-2021 closures capture only the
        // fields they use, and `owner.0` alone (a raw pointer) is not `Send`.
        let owner = owner;
        let outcome = task_dialog_outcome(owner.0, &labels);
        callback(outcome);
    });
}

/// A raw Win32 window handle, wrapped so it can cross into the dedicated
/// thread [`show_discard_dialog_windows`] spawns. A `HWND` is an opaque
/// numeric identifier the OS accepts from any thread — Win32 explicitly
/// supports using a window handle cross-thread — so sending the pointer value
/// itself is sound even though `*mut c_void` is not `Send` by default; rfd's
/// own `MessageDialog` makes the same call for its `RawWindowHandle` (see its
/// `unsafe impl Send for MessageDialog`).
#[cfg(target_os = "windows")]
struct SendHwnd(windows_sys::Win32::Foundation::HWND);

#[cfg(target_os = "windows")]
unsafe impl Send for SendHwnd {}

/// Builds and shows the `TaskDialogIndirect` dialog and maps its result (or a
/// failing `HRESULT`) to an [`Outcome`]. Split out from
/// [`show_discard_dialog_windows`] so the FFI call site is small and easy to
/// read in full.
#[cfg(target_os = "windows")]
fn task_dialog_outcome(
    owner: windows_sys::Win32::Foundation::HWND,
    labels: &CloseGuardLabels,
) -> Outcome {
    use windows_sys::Win32::UI::Controls::{
        TASKDIALOG_BUTTON, TASKDIALOGCONFIG, TDF_ALLOW_DIALOG_CANCELLATION, TaskDialogIndirect,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::IDCANCEL;

    // The Discard button's own id — anything other than `IDCANCEL`, since
    // `TaskDialogIndirect` returns `IDCANCEL` itself for Escape/the window's
    // close button when no custom button claims that id (see
    // `windows_button_to_outcome`'s doc comment).
    const DISCARD_BUTTON_ID: i32 = 1000;

    let title = to_wide(&labels.title);
    let message = to_wide(&labels.message);
    let discard_label = to_wide(&labels.discard);
    let cancel_label = to_wide(&labels.cancel);

    let buttons = [
        TASKDIALOG_BUTTON {
            nButtonID: DISCARD_BUTTON_ID,
            pszButtonText: discard_label.as_ptr(),
        },
        TASKDIALOG_BUTTON {
            nButtonID: IDCANCEL,
            pszButtonText: cancel_label.as_ptr(),
        },
    ];

    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        hwndParent: owner,
        dwFlags: TDF_ALLOW_DIALOG_CANCELLATION,
        pszWindowTitle: title.as_ptr(),
        pszContent: message.as_ptr(),
        cButtons: buttons.len() as u32,
        pButtons: buttons.as_ptr(),
        // The Cancel button is the default (Enter-activated) button — the
        // entire point of this module (see the module doc): rfd's own
        // `TaskDialogIndirect` call hardcodes `nDefaultButton: 0`, which
        // matches no custom button id and so falls back to the FIRST button,
        // Discard.
        nDefaultButton: IDCANCEL,
        ..Default::default()
    };

    let mut selected_button = 0i32;
    // SAFETY: `config`, `buttons`, and every wide-string buffer the pointers
    // in both reference (`title`, `message`, `discard_label`, `cancel_label`)
    // stay alive for the whole call — none are moved or dropped before
    // `TaskDialogIndirect` returns. The two other out-parameters are null:
    // this dialog has no radio buttons and no verification checkbox.
    let hr = unsafe {
        TaskDialogIndirect(
            &config,
            &mut selected_button,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    if hr < 0 {
        // Any platform error maps to Cancel (module doc) — never Discard.
        return Outcome::Cancel;
    }
    windows_button_to_outcome(selected_button, DISCARD_BUTTON_ID)
}

#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// macOS backend: rfd called DIRECTLY (not through `tauri-plugin-dialog`), on
/// a dedicated thread, with the labels ordered `[cancel, discard]` so
/// `NSAlert`'s own "first button added" default lands on Cancel (see the
/// module doc for why the plugin can't be used here).
#[cfg(target_os = "macos")]
fn show_discard_dialog_macos(
    app: &tauri::AppHandle,
    labels: CloseGuardLabels,
    callback: impl FnOnce(Outcome) + Send + 'static,
) {
    let app = app.clone();

    // A dedicated thread: `MessageDialog::show` blocks the calling thread
    // while it runs (rfd dispatches the actual `NSAlert` onto the main thread
    // internally and waits for the answer), and this must never block its own
    // caller (the module doc's "never block the calling thread" rule).
    std::thread::spawn(move || {
        let parent = app.get_webview_window("main");
        let mut dialog = rfd::MessageDialog::new()
            .set_title(labels.title)
            .set_description(labels.message)
            .set_level(rfd::MessageLevel::Warning)
            // Ordered [cancel, discard]: NSAlert's own Return-key default is
            // always the FIRST button added
            // (`rfd-0.16.0/src/backend/macos/message_dialog.rs:62-64`), which
            // is what makes Cancel — not Discard — the default here.
            .set_buttons(rfd::MessageButtons::OkCancelCustom(
                labels.cancel.clone(),
                labels.discard.clone(),
            ));
        if let Some(window) = parent.as_ref() {
            dialog = dialog.set_parent(window);
        }
        let result = dialog.show();
        callback(macos_result_to_outcome(result, &labels.discard));
    });
}

/// Maps a Windows `TaskDialogIndirect` button id to an [`Outcome`] (see the
/// module doc for why this dialog is built directly rather than through
/// `tauri-plugin-dialog`). Takes a plain `i32` rather than a `windows-sys`
/// type, so the mapping compiles — and is unit-tested — on every platform,
/// not only Windows.
///
/// `discard_button_id` is the id the caller assigned the Discard button in its
/// own `TASKDIALOG_BUTTON` array; everything else — including the id
/// `TaskDialogIndirect` returns for Escape, the window's close button, or any
/// other way of dismissing the dialog without clicking a button (`IDCANCEL`,
/// per MSDN, when that id is assigned to a button) — maps to
/// [`Outcome::Cancel`]. A caller-side `HRESULT` failure from
/// `TaskDialogIndirect` itself never reaches this function at all: it is
/// handled before this runs, by treating the call as Cancel directly (the
/// module doc's "any platform error" rule).
pub fn windows_button_to_outcome(button_id: i32, discard_button_id: i32) -> Outcome {
    if button_id == discard_button_id {
        Outcome::Discard
    } else {
        Outcome::Cancel
    }
}

/// Maps rfd's own `MessageDialogResult`, from calling rfd DIRECTLY on macOS
/// (see the module doc), to an [`Outcome`]. `discard_label` is the exact label
/// the caller gave the Discard button; `Custom(discard_label)` is the only
/// Discard case. Everything else — `Custom(cancel_label)` (NSAlert's second
/// button), a plain `Cancel` (NSAlert's own "nothing matched" fallback) `Ok`,
/// `Yes`, `No` — maps to [`Outcome::Cancel`], matching
/// `rfd-0.16.0/src/backend/macos/message_dialog.rs`'s own `dialog_result`,
/// whose final arm is `_ => MessageDialogResult::Cancel`.
pub fn macos_result_to_outcome(result: rfd::MessageDialogResult, discard_label: &str) -> Outcome {
    match result {
        rfd::MessageDialogResult::Custom(s) if s == discard_label => Outcome::Discard,
        _ => Outcome::Cancel,
    }
}

/// Maps a GTK `MessageDialog` response to an [`Outcome`] (Linux only — see the
/// module doc). `discard` is the `ResponseType` the caller gave the Discard
/// button when building the dialog; everything else — including
/// `ResponseType::Cancel` (the Cancel button itself), `ResponseType::DeleteEvent`
/// (the dialog's window closed, or Escape when `set_default_response` points
/// at Cancel) and `ResponseType::None` (used defensively for any backend
/// error) — maps to [`Outcome::Cancel`].
#[cfg(target_os = "linux")]
pub fn gtk_response_to_outcome(response: gtk::ResponseType, discard: gtk::ResponseType) -> Outcome {
    if response == discard {
        Outcome::Discard
    } else {
        Outcome::Cancel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Outcome::discards() -------------------------------------------

    #[test]
    fn the_discard_outcome_discards() {
        assert!(
            Outcome::Discard.discards(),
            "the Discard outcome must report that it discards the unsaved changes"
        );
    }

    #[test]
    fn the_cancel_outcome_never_discards() {
        assert!(
            !Outcome::Cancel.discards(),
            "the Cancel outcome must never report that it discards the unsaved changes"
        );
    }

    // --- windows_button_to_outcome --------------------------------------

    #[test]
    fn windows_the_discard_button_id_discards() {
        assert_eq!(windows_button_to_outcome(1000, 1000), Outcome::Discard);
    }

    #[test]
    fn windows_idcancel_cancels() {
        // IDCANCEL (2): what TaskDialogIndirect returns for Escape/the
        // window's close button, when the real implementation assigns that
        // id to the Cancel button (TDF_ALLOW_DIALOG_CANCELLATION).
        assert_eq!(windows_button_to_outcome(2, 1000), Outcome::Cancel);
    }

    #[test]
    fn windows_any_other_button_id_cancels() {
        assert_eq!(windows_button_to_outcome(0, 1000), Outcome::Cancel);
    }

    // --- macos_result_to_outcome -----------------------------------------

    #[test]
    fn macos_the_matching_custom_label_discards() {
        let result = rfd::MessageDialogResult::Custom("Discard".to_string());
        assert_eq!(macos_result_to_outcome(result, "Discard"), Outcome::Discard);
    }

    #[test]
    fn macos_the_cancel_label_cancels() {
        let result = rfd::MessageDialogResult::Custom("Cancel".to_string());
        assert_eq!(macos_result_to_outcome(result, "Discard"), Outcome::Cancel);
    }

    #[test]
    fn macos_a_plain_cancel_result_cancels() {
        assert_eq!(
            macos_result_to_outcome(rfd::MessageDialogResult::Cancel, "Discard"),
            Outcome::Cancel
        );
    }

    #[test]
    fn macos_an_unexpected_ok_result_cancels() {
        // Defensive: Ok should never occur for an OkCancelCustom dialog, but a
        // platform quirk must still never be read as Discard.
        assert_eq!(
            macos_result_to_outcome(rfd::MessageDialogResult::Ok, "Discard"),
            Outcome::Cancel
        );
    }

    // --- gtk_response_to_outcome (Linux only) ----------------------------

    #[cfg(target_os = "linux")]
    #[test]
    fn gtk_the_matching_response_discards() {
        assert_eq!(
            gtk_response_to_outcome(gtk::ResponseType::Accept, gtk::ResponseType::Accept),
            Outcome::Discard
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn gtk_the_cancel_response_cancels() {
        assert_eq!(
            gtk_response_to_outcome(gtk::ResponseType::Cancel, gtk::ResponseType::Accept),
            Outcome::Cancel
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn gtk_delete_event_cancels() {
        // The window's close button, or Escape when the dialog's default
        // response is Cancel: GTK reports DeleteEvent, not the default
        // response's own variant.
        assert_eq!(
            gtk_response_to_outcome(gtk::ResponseType::DeleteEvent, gtk::ResponseType::Accept),
            Outcome::Cancel
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn gtk_none_response_cancels() {
        assert_eq!(
            gtk_response_to_outcome(gtk::ResponseType::None, gtk::ResponseType::Accept),
            Outcome::Cancel
        );
    }
}
