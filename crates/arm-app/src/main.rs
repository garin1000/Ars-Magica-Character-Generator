// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arm_app::commands::{self, AppState};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(window_close_bridge_plugin())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_ruleset,
            commands::validate_entity,
            commands::effective_scores,
            commands::derived_totals,
            commands::apply_childhood_package,
            commands::aging_preview,
            commands::aging_apply,
            commands::aging_revert,
            commands::save_entity,
            commands::load_entity,
            commands::export_markdown,
            commands::export_label_keys,
            commands::update_close_guard,
            request_close,
        ])
        // Window-close gestures (title-bar X, Alt+F4, Cmd+W).
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && guard_blocks_quit(window.app_handle(), |app| {
                    if let Some(w) = app.get_webview_window("main") {
                        // destroy() force-closes without re-firing CloseRequested.
                        let _ = w.destroy();
                    }
                })
            {
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        // App-quit (notably macOS Cmd+Q), which bypasses window close entirely.
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event
                && guard_blocks_quit(app_handle, |app| app.exit(0))
            {
                api.prevent_exit();
            }
        });
}

/// N1: WebKitGTK and WebView2 (wry's Linux and Windows backends) hard-wire the
/// JS-visible `window.close()` DOM method to destroy the native window
/// *directly*, bypassing `WindowEvent::CloseRequested` — and therefore this
/// file's `on_window_event` guard below — entirely. Confirmed by reading wry
/// 0.55.1 itself, the version this workspace pins (`Cargo.lock`): the
/// webkitgtk backend runs `webview.connect_close(|w| unsafe { w.destroy() })`
/// (`wry-0.55.1/src/webkitgtk/mod.rs:460`) and the webview2 backend runs
/// `webview.add_WindowCloseRequested(.., || DestroyWindow(hwnd))`
/// (`wry-0.55.1/src/webview2/mod.rs:616-619`) — both unconditional, first line
/// of their handler setup, with no `WebViewAttributes` field or builder option
/// to opt out. (macOS's WKWebView backend never wires anything to this at
/// all — `window.close()` is already a no-op there, since wry implements no
/// `webViewDidClose:` WKUIDelegate method, and WebKit only invokes delegate
/// methods an app actually implements.) No UI path in this app calls
/// `window.close()` today (checked `App.svelte`'s `handleShortcut`), so this
/// is pre-emptive hardening of a latent hole, not a live regression fix.
///
/// The fix: shadow the JS-visible `window.close` before any page script can
/// run, so the engines' own native implementations — the ones wired to the
/// signals above — are never invoked in the first place (a reassigned global
/// function shadows the built-in; the native close path is only reachable by
/// actually calling it). The shadow instead invokes the `request_close`
/// command below, which calls the real [`tauri::WebviewWindow::close`] — whose
/// own doc comment states "It emits `WindowEvent::CloseRequested` first like a
/// user-initiated close request so you can intercept it", i.e. the exact
/// pipeline `on_window_event` already guards, confirmed independently by
/// reading `tauri-runtime-wry` 2.11.3: `Window::close()` sends
/// `WindowMessage::Close`, dispatched through `on_close_requested` — the same
/// function a title-bar click or Alt+F4 reaches.
///
/// Registered as a plugin's `js_init_script` (not a window-builder option)
/// because the main window is declared in `tauri.conf.json`, not built
/// manually in Rust; a plugin init script is Tauri's supported way to inject
/// script into a config-declared window, guaranteed to run before any page
/// script on all three backends (`tauri::plugin::Builder::js_init_script`'s
/// own doc: "before the HTML document has been parsed and before any other
/// script"). `withGlobalTauri` is `false` in `tauri.conf.json`, so the bridge
/// calls the low-level `window.__TAURI_INTERNALS__.invoke` the frontend's own
/// `@tauri-apps/api/core` wraps (confirmed in
/// `ui/node_modules/@tauri-apps/api/core.js`), not the convenience
/// `window.__TAURI__` global this app does not inject.
///
/// Not unit-testable the way the validation-layer fixes are: this is
/// window/webview-level Tauri glue with no library-crate seam (`main.rs` is a
/// binary, and `guard_blocks_quit` below has the same gap) — Tauri's mock
/// runtime needs the `test` feature, not enabled in this crate, and adding it
/// is a larger step than this fix. Needs an e2e spec (`ui/e2e/`, not owned by
/// this change) driving the real release binary — dirty an entity, run
/// `browser.execute(() => window.close())` as wave-1b agent C's now-deleted
/// probe did, and assert the app is still alive with the edit intact — to
/// close the loop this reasoning cannot close on its own.
fn window_close_bridge_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("window-close-bridge")
        .js_init_script(
            "window.close = function () { \
               if (window.__TAURI_INTERNALS__) { \
                 window.__TAURI_INTERNALS__.invoke('request_close'); \
               } \
             };"
            .to_string(),
        )
        .build()
}

/// Bridges a webview-initiated `window.close()` (shadowed by
/// [`window_close_bridge_plugin`]) to the real window close, so it is subject
/// to the same [`guard_blocks_quit`] confirmation as every other close path.
#[tauri::command]
fn request_close(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

/// Shared close/quit guard. If the entity has unsaved changes, shows a
/// discard-confirmation dialog (unless one is already open) and returns `true`
/// so the caller blocks the pending close/quit via the matching `prevent_*`.
/// When the user confirms discarding, `on_discard` re-issues the action.
///
/// Returns `false` (allow) when there are no unsaved changes or the user has
/// already confirmed — mirroring the frontend dirty flag pushed via
/// `update_close_guard`.
fn guard_blocks_quit<F>(app: &AppHandle, on_discard: F) -> bool
where
    F: FnOnce(&AppHandle) + Send + 'static,
{
    let state = app.state::<AppState>();
    let mut guard = state.close_guard.lock().expect("close guard lock poisoned");
    if !guard.dirty || guard.confirmed {
        return false;
    }
    if !guard.showing {
        guard.showing = true;
        let labels = guard.labels.clone();
        drop(guard);
        let app = app.clone();
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
        // behind it. Parenting IS the modality mechanism the dialog plugin offers.
        if let Some(window) = app.get_webview_window("main") {
            dialog = dialog.parent(&window);
        }
        dialog.show(move |discard| {
            {
                let state = app.state::<AppState>();
                let mut guard = state.close_guard.lock().expect("close guard lock poisoned");
                guard.showing = false;
                if !discard {
                    return;
                }
                // Let the re-issued close/quit pass straight through.
                guard.confirmed = true;
            }
            on_discard(&app);
        });
    }
    true
}
