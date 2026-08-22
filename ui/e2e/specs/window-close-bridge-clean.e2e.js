// End-to-end: the window.close() bypass fix (N1 / round-2 GA2, E2), no-unsaved-
// changes half — the companion case to `window-close-bridge-dirty.e2e.js` (read that
// file's header for the full context on why this exists and what it does and does
// not prove). Drives the real binary.
//
// `guard_blocks_quit` (`crates/arm-app/src/main.rs`) returns `false` — allow — when
// the entity is not dirty, so the shadowed `window.close()` must reach the real
// close uninterrupted. Without this half, a "fix" that made `request_close` an
// unconditional no-op (rather than actually routing to `WebviewWindow::close()`)
// would pass the dirty-side spec vacuously — the window would never disappear
// either way — and never be caught.
//
// OWN FILE, single test: a clean close ends this app instance, so nothing may run
// after it in the same worker.

import { browser } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

describe('window.close() bridge — no unsaved changes', () => {
  it('a clean entity actually closes on window.close() called from page script', async () => {
    // Creating a character is not itself a dirty edit ("a freshly created
    // character is not dirty" — `ui/src/lib/state.svelte.ts`), so no confirmation
    // dialog stands in the way.
    await startCharacter('grog');

    // No dialog blocks it and this single-window app has nothing to fall back to,
    // so the app — and the WebDriver session riding on it — should actually go
    // away. The teardown can land *during* this call rather than after it: the
    // session dies while `execute` is still in flight, so the call itself rejects
    // ("invalid session id" / "session deleted because of page crash or hang").
    // That rejection is not a test failure, it IS the evidence the close
    // happened, so it counts as success rather than propagating.
    let sessionGone = false;
    try {
      await browser.execute(() => window.close());
    } catch {
      sessionGone = true;
    }

    if (!sessionGone) {
      // The close can also complete just after the call returns, so fall back to
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
          timeoutMsg: 'window.close() did not close the window when the document was clean',
        },
      );
    }
  });
});
