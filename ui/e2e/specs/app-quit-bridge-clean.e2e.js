// End-to-end: E5 (full-audit test-adequacy round 1, CRITICAL), no-unsaved-changes half
// — the companion case to `app-quit-bridge-dirty.e2e.js` (read that file's header for
// the full context on why this exists and what it does and does not prove). Drives the
// real binary.
//
// `guard_blocks_quit` (`crates/arm-app/src/main.rs`) returns `false` — allow — when the
// entity is not dirty, so `request_exit` must actually call `app.exit(0)` and end the
// process. Without this half, a "fix" that made `request_exit` an unconditional no-op
// (rather than actually routing to `guard_blocks_quit`/`app.exit(0)`) would pass the
// dirty-side spec vacuously — the app would never quit either way — and never be
// caught.
//
// OWN FILE, single test: a clean quit ends this app instance, so nothing may run after
// it in the same worker.

import { browser } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

describe('RunEvent::ExitRequested bridge — no unsaved changes', () => {
  it('a clean entity actually quits on a request_exit IPC call', async () => {
    // Creating a character is not itself a dirty edit ("a freshly created
    // character is not dirty" — `ui/src/lib/state.svelte.ts`), so no confirmation
    // dialog stands in the way.
    await startCharacter('grog');

    // No dialog blocks it, so the process should actually exit — and the WebDriver
    // session riding on it along with it. The teardown can land *during* this call
    // rather than after it, exactly as in `window-close-bridge-clean.e2e.js`: the
    // session dies while `execute` is still in flight, so the call itself rejects.
    // That rejection is not a test failure, it IS the evidence the quit happened.
    let sessionGone = false;
    try {
      await browser.execute(() => window.__TAURI_INTERNALS__.invoke('request_exit'));
    } catch {
      sessionGone = true;
    }

    if (!sessionGone) {
      // The exit can also complete just after the call returns, so fall back to
      // polling until the session stops answering.
      await browser.waitUntil(
        async () => {
          try {
            await browser.getWindowHandles();
            return false;
          } catch {
            return true;
          }
        },
        {
          timeout: 10000,
          timeoutMsg: 'request_exit did not quit the app when the document was clean',
        },
      );
    }
  });
});
