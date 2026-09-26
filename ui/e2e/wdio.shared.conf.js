// Shared WebdriverIO + tauri-driver settings for the standard and portable e2e
// configs (`wdio.conf.js`, `wdio.portable.conf.js`). Both drive a tauri-driver
// bridge (classic WebDriver) against the platform WebDriver session, so the
// runner, capability shape, connection settings, reporters, framework and
// timeouts are identical between them — only the spawned application, the specs
// glob, the `onPrepare` build/staging step, and whether the ARM_E2E_FILE /
// ARM_E2E_EXPORT_FILE seams are wired in differ (see the two callers). Extracted
// (V38) so these ~12-15 fields cannot drift when one config is tuned and the
// other is forgotten.

import path from 'node:path';

import { driverPorts } from './driver.js';

/**
 * How many spec files run at once.
 *
 * Each spec file gets its own WebDriver session — its own app launch and
 * teardown — and that launch is ~30s of the ~40s cycle, so the suite's wall
 * clock is almost entirely startup. The launches are independent (every spec
 * drives a fresh app from the start screen), so concurrency is the lever that
 * actually scales, and four is a deliberate middle: each worker carries a full
 * WebKit process, and the machine this runs on has 8 cores.
 *
 * This is only safe because every shared resource a second worker could
 * collide on has been given a per-worker identity — the tauri-driver and
 * native WebDriver ports (`driver.js`), the save/export fixtures, and the
 * app's own settings file (see each config's `beforeSession`).
 */
// Exported so each config's `onPrepare` can hand the same number to
// `allDriverPorts` (`driver.js`) for the port preflight — it must check every
// port this run's workers might actually bind, not just worker 0's.
export const MAX_WORKERS = 4;

/**
 * The fields both wdio configs share verbatim, plus the one `capabilities`
 * entry that differs only in which `application` it points `tauri:options` at.
 *
 * @param {string} application absolute path to the app binary this session drives
 * @returns {object} spreadable base config fields
 */
export function sharedWdioConfig(application) {
  return {
    runner: 'local',
    maxInstances: MAX_WORKERS,
    // Each spec file drives its own freshly launched app and passes in
    // isolation, but a run occasionally emits a transient
    // interactability/timing flake that wanders between specs run-to-run. One
    // retry cleanly absorbs those without masking a real, deterministic failure
    // (which fails both attempts). Note a retry re-pays the whole ~40s cycle.
    specFileRetries: 1,
    specFileRetriesDeferred: true,

    // Connect to tauri-driver (classic WebDriver) rather than auto-starting a
    // browser driver. tauri-driver forwards to the native WebKitWebDriver.
    //
    // This port is only the default for a lone worker: each config's
    // `beforeSession` overwrites `config.port` with the pair `driver.js`
    // assigns that worker before the session is created (the runner calls
    // `beforeSession` with the live config, then `_initSession` after it).
    hostname: '127.0.0.1',
    port: driverPorts(0).port,
    path: '/',

    capabilities: [
      {
        // Must track the top-level cap: a per-capability 1 silently pins the
        // whole run to one worker whatever `maxInstances` says.
        maxInstances: MAX_WORKERS,
        // tauri-driver does not implement WebDriver BiDi.
        'wdio:enforceWebDriverClassic': true,
        'tauri:options': { application },
      },
    ],
    reporters: ['spec'],
    framework: 'mocha',
    mochaOpts: { ui: 'bdd', timeout: 120000 },
    logLevel: 'info',
  };
}

/**
 * The repo-local, gitignored directory both configs write wdio's launcher +
 * worker logs to (`outputDir`). A suite this expensive should never have to be
 * re-run just to re-read output that scrolled past, and a path under the system
 * temp dir would be invisible to `git status` and shared with every other
 * project on the machine — see `wdio.conf.js` for the full rationale.
 *
 * @param {string} repoRoot absolute path to the repository root
 * @returns {string}
 */
export function e2eLogDir(repoRoot) {
  return path.resolve(repoRoot, 'tmp/e2e-logs');
}
