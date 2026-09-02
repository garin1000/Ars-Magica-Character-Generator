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
import { e2eLogDir, sharedWdioConfig } from './wdio.shared.conf.js';

const dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dirname, '../..');

export const config = {
  ...sharedWdioConfig(portableApp),
  specs: [path.resolve(dirname, 'portable/**/*.e2e.js')],

  // Same repo-local log destination as the standard suite, for the same reason
  // the staged app itself lives under `tmp/`: nothing a run produces belongs
  // outside the repository.
  outputDir: e2eLogDir(repoRoot),

  // Build the production binary, then stage it — with `rules/` beside it — where
  // Tauri cannot mistake it for a dev build. The display check runs first and runs
  // here, the last point at which `DISPLAY` still reflects the real environment.
  onPrepare: () => {
    const problem = preflightDisplay(process.env, { xvfbRunAvailable: hasXvfbRun() });
    if (problem) throw new Error(problem);
    stagePortableApp();
  },

  // Deliberately narrower than the standard suite's `beforeSession` (V39): this
  // app is spawned WITHOUT ARM_E2E_FILE / ARM_E2E_EXPORT_FILE in its env, and
  // `stagePortableApp()` above builds with a plain `cargo tauri build --no-bundle`
  // — no `--features e2e-testing` — so the portable binary never compiles in
  // those save/load seams (see `wdio.conf.js` and `crates/arm-app/Cargo.toml`).
  // No spec under `portable/` exercises save, load, or Markdown export today as
  // a result. Supporting one would mean: building with `--features e2e-testing`
  // here too (as `wdio.conf.js`'s `onPrepare` does), passing the same two env
  // vars into the spawned tauri-driver below, and picking a `portable/`-local
  // fixture path — the standard suite's `e2eFile`/`e2eExportFile` (under
  // `os.tmpdir()`) could be reused as-is, since the seam itself is
  // layout-agnostic; only the build step and env wiring are missing here.
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
