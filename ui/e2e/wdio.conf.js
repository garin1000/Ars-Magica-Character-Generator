// WebdriverIO + tauri-driver e2e config. Drives the REAL release binary through
// the platform WebDriver (WebKitWebDriver on Linux), exercising true IPC and the
// bundled rules resources.
//
// Requirements to run:
//   - `tauri-driver`        (cargo install tauri-driver)
//   - `WebKitWebDriver`     (Linux: apt install webkit2gtk-driver)
//   - a display (use `xvfb-run` in headless CI)
//
// The native save/load dialogs can't be driven by WebDriver, so the app reads
// the ARM_E2E_FILE seam (see crates/arm-app/src/commands.rs) for a fixed path.

import { spawn, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dirname, '../..');
const application = path.resolve(repoRoot, 'target/release/arm-app');

// Fixed save/load target so the flow is deterministic and headless.
export const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

let tauriDriver;

export const config = {
  runner: 'local',
  specs: [path.resolve(dirname, 'specs/**/*.e2e.js')],
  maxInstances: 1,

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

  // Build the PRODUCTION binary (embedded frontend assets, no dev server), then
  // stage the rules resources beside it. `cargo tauri build` runs in production
  // mode and skips installers with --no-bundle; a plain `cargo build` would run
  // the app in dev mode (loading the Vite dev URL). The app resolves rules via
  // `BaseDirectory::Resource`, which for a non-bundled binary is the exe dir, so
  // the resources are copied to `target/release/rules`.
  onPrepare: () => {
    const build = spawnSync('cargo', ['tauri', 'build', '--no-bundle'], {
      cwd: path.resolve(repoRoot, 'crates/arm-app'),
      stdio: 'inherit',
    });
    if (build.status !== 0) throw new Error('cargo tauri build failed');

    const destRules = path.resolve(repoRoot, 'target/release/rules');
    for (const sub of ['core', 'i18n']) {
      fs.cpSync(path.resolve(repoRoot, 'rules', sub), path.resolve(destRules, sub), {
        recursive: true,
      });
    }
  },

  // tauri-driver bridges WebDriver to the platform webdriver. The spawned app
  // inherits ARM_E2E_FILE so save/load skip the native dialog.
  beforeSession: () =>
    new Promise((resolve) => {
      tauriDriver = spawn(path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'), [], {
        stdio: [null, process.stdout, process.stderr],
        env: { ...process.env, ARM_E2E_FILE: e2eFile },
      });
      // Give tauri-driver a moment to bind port 4444 before WDIO connects.
      setTimeout(resolve, 2000);
    }),
  afterSession: () => {
    if (tauriDriver) tauriDriver.kill();
  },
};
