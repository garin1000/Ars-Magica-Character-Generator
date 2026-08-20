// WebdriverIO + tauri-driver config for the PORTABLE-layout smoke check.
//
// Same driver, same production binary, one difference that matters: the app runs
// from `tmp/portable/` instead of `target/release/`, which is what forces
// `load_ruleset`'s exe-dir fallback to do the work Tauri's `BaseDirectory::
// Resource` does under `target/` (see `stage-portable.js` for the whole story).
// The ordinary suite cannot cover this — it runs the binary where cargo left it,
// where the fallback is never reached — so this config exists to close that gap.
//
// NOT part of `npm run test:e2e`: it needs the staging step first, and the
// specs live outside `specs/` so the standard config's glob cannot pick them up.
//
//   cd ui && npm run test:e2e:portable
//
// Requirements are the standard suite's: `tauri-driver`, `WebKitWebDriver`, and a
// display — the desktop session, or the Xvfb WebdriverIO starts when `DISPLAY` is
// unset.

import { spawn } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { hasXvfbRun, preflightDisplay } from './display.js';
import { portableApp, stagePortableApp } from './stage-portable.js';

const dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dirname, '../..');

export const config = {
  runner: 'local',
  specs: [path.resolve(dirname, 'portable/**/*.e2e.js')],
  maxInstances: 1,
  specFileRetries: 1,
  specFileRetriesDeferred: true,

  hostname: '127.0.0.1',
  port: 4444,
  path: '/',

  capabilities: [
    {
      maxInstances: 1,
      'wdio:enforceWebDriverClassic': true,
      'tauri:options': { application: portableApp },
    },
  ],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: { ui: 'bdd', timeout: 120000 },
  logLevel: 'info',

  // Same repo-local log destination as the standard suite, for the same reason
  // the staged app itself lives under `tmp/`: nothing a run produces belongs
  // outside the repository.
  outputDir: path.resolve(repoRoot, 'tmp/e2e-logs'),

  // Build the production binary, then stage it — with `rules/` beside it — where
  // Tauri cannot mistake it for a dev build. The display check runs first and runs
  // here, the last point at which `DISPLAY` still reflects the real environment.
  onPrepare: () => {
    const problem = preflightDisplay(process.env, { xvfbRunAvailable: hasXvfbRun() });
    if (problem) throw new Error(problem);
    stagePortableApp();
  },

  beforeSession: () =>
    new Promise((resolve) => {
      config.tauriDriver = spawn(path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'), [], {
        stdio: [null, process.stdout, process.stderr],
      });
      // Give tauri-driver a moment to bind port 4444 before WDIO connects.
      setTimeout(resolve, 2000);
    }),
  afterSession: () => {
    if (config.tauriDriver) config.tauriDriver.kill();
  },
};
