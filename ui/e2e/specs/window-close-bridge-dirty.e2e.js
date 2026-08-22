// End-to-end: the window.close() bypass fix (N1 / round-2 GA2, E2), unsaved-changes
// half. Drives the real binary.
//
// WebKitGTK and WebView2 (wry's Linux and Windows backends) hard-wire the JS-visible
// `window.close()` DOM method to destroy the native window directly, bypassing
// `WindowEvent::CloseRequested` — and therefore the unsaved-changes guard — entirely
// (see `crates/arm-app/src/main.rs`'s `window_close_bridge_plugin` doc comment). The
// fix shadows `window.close` so it invokes the `request_close` command instead, which
// calls the real `WebviewWindow::close()` and is subject to the same confirmation as
// every other close path.
//
// This is NOT a real window-manager close (title-bar click, Alt+F4) — WebDriver
// cannot generate that event — but it does not need to be: `window.close()` called
// from page script is exactly the call the shadow intercepts, and is reachable from
// any future frontend code, which is the entire point of the fix.
//
// OWN FILE, single test: with unsaved edits, `guard_blocks_quit` raises a native
// GTK confirmation dialog that WebDriver cannot dismiss (native dialogs cannot be
// driven by WebDriver — see `ui/e2e/wdio.conf.js`'s note on save/load). That dialog
// is left open for the rest of this worker's app instance, so nothing may run after
// this test in the same file. Each spec file gets its own freshly launched app (see
// `helpers.js`), so that is not a problem across files.

import { $, browser, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

const NAME_INPUT = '[data-testid="identity-name"]';
const TAB_BAR = '[role="tablist"]';

describe('window.close() bridge — unsaved changes', () => {
  it('a dirty entity survives window.close() called from page script', async () => {
    await startCharacter('companion');
    await $(NAME_INPUT).setValue('Bridge test dirty');

    // Before the shadow existed, this call destroyed the window immediately, with
    // no chance for the guard — or this assertion — to run at all.
    await browser.execute(() => window.close());

    // The shadow invokes `request_close`, which calls the real
    // `WebviewWindow::close()` -> `WindowEvent::CloseRequested` -> `guard_blocks_quit`,
    // which raises the native confirmation dialog and calls `prevent_close()` while
    // the entity is dirty. Poll rather than a fixed sleep: confirm the window stays
    // alive and the edit survives once the IPC round trip has had time to land.
    await browser.waitUntil(async () => (await $(NAME_INPUT).getValue()) === 'Bridge test dirty', {
      timeout: 5000,
      interval: 250,
      timeoutMsg:
        'the edit did not survive window.close() with unsaved changes — the window may have been destroyed',
    });
    expect(await $(TAB_BAR).isExisting()).toBe(true);
  });
});
