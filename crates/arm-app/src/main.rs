// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arm_app::commands::{self, AppState};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::load_ruleset,
            commands::validate_entity,
            commands::effective_scores,
            commands::derived_totals,
            commands::save_entity,
            commands::load_entity,
            commands::update_close_guard,
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
        app.dialog()
            .message(labels.message)
            .title(labels.title)
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                labels.discard,
                labels.cancel,
            ))
            .show(move |discard| {
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
