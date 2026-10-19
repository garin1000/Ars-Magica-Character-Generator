// Display preflight for the e2e suite.
//
// The suite drives a real GUI binary, so it needs an X display. WebdriverIO
// used to supply one on its own: `@wdio/local-runner` (via `@wdio/xvfb`) checked
// `DISPLAY` and, when it was unset (ssh session, cron, an agent session), spawned
// the worker through `xvfb-run --auto-servernum`. That wrapper is no longer used
// here: on this machine (and any distro with the same `xvfb-run` fix), the
// script closes fd 3 before exec'ing the wrapped command
// (`DISPLAY=:$N XAUTHORITY=$AUTHFILE "$@" 3>&-`, `/usr/bin/xvfb-run`) — and fd 3
// is exactly the IPC channel Node's `fork()` gives a worker
// (`stdio: [..., 'ipc']`), so every worker died at its first `process.send` with
// `write EINVAL`. See `xvfb.js` for the replacement: the suite starts exactly
// one Xvfb of its own in `onPrepare` (`xvfb.js`'s `ensureOwnDisplay`) and
// disables WebdriverIO's own wrapping (`autoXvfb: false`,
// `wdio.shared.conf.js`), rather than relying on `xvfb-run` at all.
//
// This module still runs first, in the launcher process, because the harness
// needs the `Xvfb` binary itself to exist before it tries to spawn one — on a
// headless box without the `xvfb` package, failing immediately with an
// install hint beats dying later inside WebDriver with an unrelated-looking
// error.

import { execFileSync } from 'node:child_process';

const isSet = (value) => typeof value === 'string' && value.trim() !== '';

/**
 * Check that the run can get a display before the suite starts.
 *
 * Both conditions deliberately mirror what `@wdio/xvfb`'s own `shouldRun` used
 * to check (and what `xvfb.js`'s `shouldStartOwnXvfb` checks now), so this can
 * never reject a run that could proceed, or pass one that cannot. It looks at
 * `DISPLAY` only (a Wayland session without XWayland exports none, and takes
 * the Xvfb path too), and only on Linux (macOS and Windows drive a native
 * webview and have no X display at all).
 *
 * @param {Record<string, string | undefined>} env
 * @param {{ xvfbAvailable: boolean, platform?: string }} host
 * @returns {string | null} an explanatory message, or null when the run can proceed
 */
export function preflightDisplay(env, { xvfbAvailable, platform = process.platform }) {
  if (platform !== 'linux') return null;
  if (isSet(env.DISPLAY) || xvfbAvailable) return null;

  return (
    'No display (DISPLAY is unset) and Xvfb was not found, so the e2e suite ' +
    'has nothing to render into. Install the xvfb package (Debian/Ubuntu: ' +
    'sudo apt install xvfb), or run from a desktop session.'
  );
}

/**
 * Whether `Xvfb` is on PATH — the suite spawns it directly (`xvfb.js`), rather
 * than going through the `xvfb-run` wrapper.
 *
 * @returns {boolean}
 */
export function hasXvfb() {
  try {
    execFileSync('which', ['Xvfb'], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}
