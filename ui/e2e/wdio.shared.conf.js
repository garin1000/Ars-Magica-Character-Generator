// Shared WebdriverIO + tauri-driver settings for the standard and portable e2e
// configs (`wdio.conf.js`, `wdio.portable.conf.js`). Both drive a tauri-driver
// bridge (classic WebDriver) against the platform WebDriver session, so the
// runner, capability shape, connection settings, reporters, framework and
// timeouts are identical between them — only the spawned application, the specs
// glob, the `onPrepare` build/staging step, and whether the ARM_E2E_FILE /
// ARM_E2E_EXPORT_FILE seams are wired in differ (see the two callers). Extracted
// (V38) so these ~12-15 fields cannot drift when one config is tuned and the
// other is forgotten.

import fs from 'node:fs';
import path from 'node:path';

import { driverPorts } from './driver.js';

/**
 * The version of the app about to run, from `ui/package.json`. A Rust test
 * holds it equal to `CARGO_PKG_VERSION`, which is what the update check's
 * consent record is compared against (M6.6).
 *
 * @param {string} repoRoot absolute path to the repository root
 * @returns {string}
 */
export function appVersion(repoRoot) {
  return JSON.parse(fs.readFileSync(path.resolve(repoRoot, 'ui/package.json'), 'utf8')).version;
}

/**
 * How many spec files run at once, unless `ARM_E2E_WORKERS` says otherwise.
 *
 * Every spec drives a fresh app from the start screen, so the files are
 * independent and concurrency is the lever that scales. Measured on the 8-core
 * dev box (2026-10-11, docs/test-performance-2026-09.md): 4 workers 2:44, 6
 * 2:15, 8 2:09 for the test phase. 8 was not chosen: its first confirmation
 * run needed a retry (an interactability flake under load), and past 6 the
 * longest spec, not the worker count, sets the floor. On a smaller machine,
 * lower it with the variable.
 */
const DEFAULT_WORKERS = 6;

/**
 * The run's worker count.
 *
 * This is only safe because every shared resource a second worker could
 * collide on has been given a per-worker identity — the tauri-driver and
 * native WebDriver ports (`driver.js`), the save/export fixtures, and the
 * app's own settings file (see each config's `beforeSession`).
 */
// Exported so each config's `onPrepare` can hand the same number to
// `allDriverPorts` (`driver.js`) for the port preflight — it must check every
// port this run's workers might actually bind, not just worker 0's.
export const MAX_WORKERS = workerCount(process.env);

/**
 * The worker count: `ARM_E2E_WORKERS` when set to a positive integer, the
 * default when unset or empty. Anything else throws, so a typo never runs the
 * suite at some other count unnoticed.
 *
 * @param {Record<string, string | undefined>} env
 * @returns {number}
 */
function workerCount(env) {
  const value = env.ARM_E2E_WORKERS;
  if (value === undefined || value === '') return DEFAULT_WORKERS;
  if (!/^[1-9]\d*$/.test(value)) {
    throw new Error(`ARM_E2E_WORKERS must be a positive integer, got ${JSON.stringify(value)}`);
  }
  return Number(value);
}

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
