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
/// GA2/round-2: the runtime claim here — that the shadowed `window.close()`
/// actually reaches `WindowEvent::CloseRequested` and is blocked while dirty —
/// still has no unit seam: this is window/webview-level Tauri glue with no
/// library-crate hook (`main.rs` is a binary, and `guard_blocks_quit` below has
/// the same gap), and exercising it for real needs a live window, which needs
/// Tauri's mock runtime (the `test` feature, not enabled in this crate — adding
/// it is a larger step than this fix). That half is closed by an e2e spec
/// instead: `ui/e2e/specs/window-close-bridge-dirty.e2e.js` and
/// `window-close-bridge-clean.e2e.js` drive the real release binary through
/// `browser.execute(() => window.close())`, the same call the shadow below
/// intercepts, against both a dirty and a clean document.
///
/// The one part of this glue that IS plain data — that the injected script
/// invokes the command that is actually registered — is unit-tested in this
/// file's own `tests` module: [`REQUEST_CLOSE_COMMAND`] is the single name both
/// [`window_close_shadow_script`] and `request_close`'s own `stringify!` check
/// read, so a rename of one without the other fails a fast test instead of
/// silently reopening the bypass.
fn window_close_bridge_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("window-close-bridge")
        .js_init_script(window_close_shadow_script())
        .build()
}

/// The Tauri command name the shadow script below invokes. Named once so
/// [`window_close_shadow_script`] and `request_close`'s registration can never
/// drift apart without a test noticing (see
/// `the_shadow_script_invokes_the_registered_request_close_command`).
const REQUEST_CLOSE_COMMAND: &str = "request_close";

/// The JS that shadows the DOM's built-in `window.close()` (see
/// [`window_close_bridge_plugin`]'s doc comment for why this exists and how it
/// is installed). Broken out of the plugin builder so it can be inspected
/// directly by a unit test with no Tauri runtime involved.
fn window_close_shadow_script() -> String {
    format!(
        "window.close = function () {{ \
           if (window.__TAURI_INTERNALS__) {{ \
             window.__TAURI_INTERNALS__.invoke('{REQUEST_CLOSE_COMMAND}'); \
           }} \
         }};"
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    // GA2/E2 (round 2): the JS-visible shadow and the registered Tauri command it
    // invokes are two independently-typed literals with nothing tying them
    // together. A rename of `request_close` with a missed edit to the JS string
    // would silently reopen the N1 bypass — the shadow would install fine and
    // invoke a command that no longer exists, and nothing but an e2e run (or a
    // user hitting Alt+F4) would notice. This is the one part of the bridge that
    // is pure data and testable without Tauri's mock runtime (see the doc comment
    // on `window_close_bridge_plugin` for why the rest is not); it does not, and
    // cannot, verify that the real `window.close()` call actually reaches
    // `WindowEvent::CloseRequested` — that is the e2e spec's job.
    #[test]
    fn the_shadow_script_invokes_the_registered_request_close_command() {
        assert_eq!(REQUEST_CLOSE_COMMAND, stringify!(request_close));

        let script = window_close_shadow_script();
        assert!(script.contains("window.close = function"));
        assert!(script.contains("__TAURI_INTERNALS__"));
        assert!(script.contains(&format!("invoke('{REQUEST_CLOSE_COMMAND}')")));
    }
}
