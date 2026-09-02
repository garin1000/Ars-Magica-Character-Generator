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
    maxInstances: 1,
    // Specs run serially against one shared app instance and each passes in
    // isolation, but the shared session occasionally emits a transient
    // interactability/timing flake that wanders between specs run-to-run. One
    // retry cleanly absorbs those without masking a real, deterministic failure
    // (which fails both attempts).
    specFileRetries: 1,
    specFileRetriesDeferred: true,

    // Connect to tauri-driver (classic WebDriver) rather than auto-starting a
    // browser driver. tauri-driver listens on 4444 and forwards to the native
    // WebKitWebDriver.
    hostname: '127.0.0.1',
    port: 4444,
    path: '/',

    capabilities: [
      {
        maxInstances: 1,
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
