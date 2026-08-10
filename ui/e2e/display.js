// Display preflight for the e2e suite.
//
// The suite drives a real GUI binary, so it needs an X display. WebdriverIO
// supplies one on its own: `@wdio/local-runner` checks `DISPLAY` and, when it is
// unset (ssh session, cron, CI), spawns the worker through
// `xvfb-run --auto-servernum` — so the run is headful on a desktop and headless
// everywhere else, from the same `npm run test:e2e`. Nothing here needs to start
// an X server; doing so would be dead code, because by the time a hook runs in
// the worker the wrapper has already set `DISPLAY`.
//
// The one case WebdriverIO handles poorly is a headless box *without* the xvfb
// package: it logs a warning, continues, and the run then dies deep inside
// WebDriver with an unrelated-looking error. This module makes that failure
// immediate and actionable instead.

import { execFileSync } from 'node:child_process';

const isSet = (value) => typeof value === 'string' && value.trim() !== '';

/**
 * Check that the run can get a display before the suite starts.
 *
 * Both conditions deliberately mirror `@wdio/xvfb`'s own `shouldRun`, so the
 * check can never reject a run the framework would have handled — or pass one it
 * will fail. It looks at `DISPLAY` only (a Wayland session without XWayland
 * exports none, and takes the Xvfb path too), and only on Linux (macOS and
 * Windows drive a native webview and have no X display at all).
 *
 * @param {Record<string, string | undefined>} env
 * @param {{ xvfbRunAvailable: boolean, platform?: string }} host
 * @returns {string | null} an explanatory message, or null when the run can proceed
 */
export function preflightDisplay(env, { xvfbRunAvailable, platform = process.platform }) {
  if (platform !== 'linux') return null;
  if (isSet(env.DISPLAY) || xvfbRunAvailable) return null;

  return (
    'No display (DISPLAY is unset) and xvfb-run was not found, so the e2e suite ' +
    'has nothing to render into. Install the xvfb package (Debian/Ubuntu: ' +
    'sudo apt install xvfb), or run from a desktop session.'
  );
}

/**
 * Whether `xvfb-run` is on PATH. Split out so `preflightDisplay` stays pure.
 *
 * @returns {boolean}
 */
export function hasXvfbRun() {
  try {
    execFileSync('which', ['xvfb-run'], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}
