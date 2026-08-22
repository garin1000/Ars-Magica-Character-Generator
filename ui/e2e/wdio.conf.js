// WebdriverIO + tauri-driver e2e config. Drives the REAL release binary through
// the platform WebDriver (WebKitWebDriver on Linux), exercising true IPC and the
// bundled rules resources.
//
// Requirements to run:
//   - `tauri-driver`        (cargo install tauri-driver)
//   - `WebKitWebDriver`     (Linux: apt install webkit2gtk-driver)
//   - a display — the desktop session if there is one (the run is then headful
//     and watchable), otherwise Xvfb (apt install xvfb), which WebdriverIO
//     starts for us. See `display.js` for the whole story.
//
// The native save/load dialogs can't be driven by WebDriver, so the app reads
// the ARM_E2E_FILE seam (see crates/arm-app/src/commands.rs) for a fixed path.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { hasXvfbRun, preflightDisplay } from './display.js';

const dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dirname, '../..');
const application = path.resolve(repoRoot, 'target/release/arm-app');

// Fixed save/load target so the flow is deterministic and headless.
export const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

// Fixed Markdown-export target, deliberately a separate seam from `e2eFile`:
// that one is the JSON save file the same specs round-trip, so sharing it would
// have an export clobber the document on disk.
export const e2eExportFile = path.resolve(os.tmpdir(), 'arm-e2e-character.md');

let tauriDriver;

export const config = {
  runner: 'local',
  specs: [path.resolve(dirname, 'specs/**/*.e2e.js')],
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

  // Keep the full launcher + worker logs of a run. A suite this expensive should
  // never have to be re-run just to re-read output that scrolled past, so wdio
  // writes them here instead of only to the terminal — no shell redirect needed,
  // which is what makes this usable from an agent. Repo-local and gitignored
  // (`tmp/`), deliberately NOT the system temp dir: an artifact under `/tmp` is
  // invisible to `git status` and shared with every other project on the box.
  outputDir: path.resolve(repoRoot, 'tmp/e2e-logs'),

  // Build the PRODUCTION binary (embedded frontend assets, no dev server), then
  // stage the rules resources beside it. `cargo tauri build` runs in production
  // mode and skips installers with --no-bundle; a plain `cargo build` would run
  // the app in dev mode (loading the Vite dev URL). Here the binary lives under
  // target/ (a cargo output dir), so `BaseDirectory::Resource` resolves to the
  // exe dir and finds `target/release/rules`. NOTE: this is NOT the portable
  // path — a binary shipped outside target/ resolves Resource to /usr/lib/<name>
  // and relies on load_ruleset's exe-dir fallback (see commands.rs).
  //
  // The display check runs first, and runs *here* rather than in a session hook:
  // onPrepare is the launcher process, the last point at which `DISPLAY` still
  // reflects the real environment — every worker below it is already inside
  // WebdriverIO's `xvfb-run` wrapper when headless.
  onPrepare: () => {
    const problem = preflightDisplay(process.env, { xvfbRunAvailable: hasXvfbRun() });
    if (problem) throw new Error(problem);

    // `--features e2e-testing` is what compiles in the ARM_E2E_FILE /
    // ARM_E2E_EXPORT_FILE seams this same suite relies on below (see
    // beforeSession). They are off by default (crates/arm-app/Cargo.toml,
    // K5/VA5 security review fix) so a plain release build never carries
    // them — this is the one build step allowed to turn them on.
    const build = spawnSync(
      'cargo',
      ['tauri', 'build', '--no-bundle', '--features', 'e2e-testing'],
      {
        cwd: path.resolve(repoRoot, 'crates/arm-app'),
        stdio: 'inherit',
      },
    );
    if (build.status !== 0) throw new Error('cargo tauri build failed');

    const destRules = path.resolve(repoRoot, 'target/release/rules');
    for (const sub of ['core', 'i18n']) {
      fs.cpSync(path.resolve(repoRoot, 'rules', sub), path.resolve(destRules, sub), {
        recursive: true,
      });
    }
  },

  // tauri-driver bridges WebDriver to the platform webdriver. The spawned app
  // inherits ARM_E2E_FILE and ARM_E2E_EXPORT_FILE so save/load and the Markdown
  // export skip their native dialogs.
  //
  // This hook already runs inside the display WebdriverIO arranged — the real
  // one, or the `xvfb-run` wrapper around this worker — so `DISPLAY` is set
  // either way and everything spawned below (WebKitWebDriver, then the app)
  // inherits it. There is nothing to set up here beyond the driver itself.
  beforeSession: () =>
    new Promise((resolve) => {
      tauriDriver = spawn(path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'), [], {
        stdio: [null, process.stdout, process.stderr],
        env: { ...process.env, ARM_E2E_FILE: e2eFile, ARM_E2E_EXPORT_FILE: e2eExportFile },
      });
      // Give tauri-driver a moment to bind port 4444 before WDIO connects.
      setTimeout(resolve, 2000);
    }),
  afterSession: () => {
    if (tauriDriver) tauriDriver.kill();
  },
};
