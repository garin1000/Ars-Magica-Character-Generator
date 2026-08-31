// End-to-end: E5 (full-audit test-adequacy round 1, CRITICAL) — RunEvent::ExitRequested
// (macOS Cmd+Q / app-level quit) coverage. Drives the real binary.
//
// window-close-bridge-*.e2e.js cover WindowEvent::CloseRequested, but that is a
// structurally different Tauri event from RunEvent::ExitRequested — the one Cmd+Q
// actually raises (see `crates/arm-app/src/main.rs`'s `request_exit` doc comment for
// the full story: window.close() always resolves to CloseRequested, never
// ExitRequested, so the window-close bridge cannot be reused to reach this path). No
// WebDriver capability available to this project can synthesize a real OS-level quit
// signal either, so this spec drives `request_exit` — a narrow, e2e-only IPC command
// (gated behind the `e2e-testing` Cargo feature this suite already builds with, see
// `wdio.conf.js`) that runs the exact same `guard_blocks_quit` call the real
// `RunEvent::ExitRequested` handler makes, triggered by IPC instead of a real quit
// signal.
//
// OWN FILE, single test: with unsaved edits, `guard_blocks_quit` raises a native GTK
// confirmation dialog that WebDriver cannot dismiss (see `window-close-bridge-dirty
// .e2e.js`'s header for the same reasoning). That dialog is left open for the rest of
// this worker's app instance, so nothing may run after this test in the same file.

import { $, browser, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

const NAME_INPUT = '[data-testid="identity-name"]';
const TAB_BAR = '[role="tablist"]';

describe('RunEvent::ExitRequested bridge — unsaved changes', () => {
  it('a dirty entity survives a request_exit IPC call', async () => {
    await startCharacter('companion');
    await $(NAME_INPUT).setValue('Quit bridge test dirty');

    // Invokes the same low-level call `window_close_shadow_script` uses
    // (`withGlobalTauri` is false, so there is no `window.__TAURI__` convenience
    // global — see `main.rs`'s `window_close_bridge_plugin` doc comment).
    await browser.execute(() => window.__TAURI_INTERNALS__.invoke('request_exit'));

    // `guard_blocks_quit` raises the native confirmation dialog and calls
    // `prevent_exit()` while the entity is dirty, so the process — and this
    // WebDriver session riding on it — must survive. Poll rather than a fixed
    // sleep: confirm the app stays alive and the edit survives once the IPC round
    // trip has had time to land.
    await browser.waitUntil(
      async () => (await $(NAME_INPUT).getValue()) === 'Quit bridge test dirty',
      {
        timeout: 5000,
        interval: 250,
        timeoutMsg:
          'the edit did not survive request_exit with unsaved changes — the app may have quit',
      },
    );
    expect(await $(TAB_BAR).isExisting()).toBe(true);
  });
});
