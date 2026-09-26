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

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { hasXvfbRun, preflightDisplay } from './display.js';
import { allDriverPorts, startWorkerDriver, workerConfigHome } from './driver.js';
import { preflightDriverPorts } from './ports.js';
import {
  killDriverTree,
  pidRegistryDir,
  reapRegisteredDrivers,
  reapRegisteredDriversForCid,
  registerDriverPid,
  unregisterDriverPid,
} from './reap.js';
import { portableApp, stagePortableApp } from './stage-portable.js';
import { e2eLogDir, MAX_WORKERS, sharedWdioConfig } from './wdio.shared.conf.js';

// Same fix as wdio.conf.js: Xvfb has no DRI3, so Mesa's hardware probe fails
// and falls back to swrast noisily. Pin llvmpipe up front instead — this
// config launches the app through the same `startWorkerDriver` env chain
// (driver.js), so setting it here reaches tauri-driver and the staged app too.
process.env.LIBGL_ALWAYS_SOFTWARE = '1';
process.env.GALLIUM_DRIVER = 'llvmpipe';

const dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dirname, '../..');

let tauriDriver;
// `afterSession` is not handed `cid` (only `beforeSession` is — see
// `@wdio/runner`'s `afterSessionArgs`), so it is tracked here alongside the
// driver handle itself, the same way `tauriDriver` already is.
let currentCid;

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
  onPrepare: async () => {
    const problem = preflightDisplay(process.env, { xvfbRunAvailable: hasXvfbRun() });
    if (problem) throw new Error(problem);

    // Same reasoning as the standard config's onPrepare: refuse immediately,
    // naming the port, rather than hang the session-creation timeout against a
    // stale driver (docs/open-todos.md, "the e2e harness leaks its driver
    // processes" — this config's own teardown bug, fixed below, is exactly
    // what produced the stale drivers that finding was written from).
    const portProblem = await preflightDriverPorts(allDriverPorts(MAX_WORKERS));
    if (portProblem) throw new Error(portProblem);
    fs.rmSync(pidRegistryDir(repoRoot), { recursive: true, force: true });

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
  //
  // It still needs the port and settings-file isolation the standard suite
  // gets from `startWorkerDriver`/`workerConfigHome` (driver.js) — just not the
  // file seams above.
  //
  // The handle used to live on `config.tauriDriver` — a property of the
  // object `beforeSession` receives as its first argument. That object is
  // `@wdio/runner`'s own `this._config`, built by `deepmerge`-ing this file's
  // exported `config` into the runner's defaults, so it is a *different*
  // object from the one this module exports. `afterSession` below takes no
  // parameters and reads the bare identifier `config`, which JS resolves via
  // lexical scope to this file's `export const config` — never the merged
  // object `beforeSession` actually mutated. So `config.tauriDriver` inside
  // `afterSession` was always `undefined`, `.kill()` was never called, and
  // this config's tauri-driver (and the WebKitWebDriver it spawns) leaked on
  // every run (docs/open-todos.md, "the e2e harness leaks its driver
  // processes"). A plain module-scope variable — the standard config's
  // `wdio.conf.js` pattern — sidesteps the object-identity question entirely.
  beforeSession: async (config, capabilities, specs, cid) => {
    tauriDriver = await startWorkerDriver(config, cid, {
      XDG_CONFIG_HOME: workerConfigHome(repoRoot, process.env),
    });
    currentCid = cid;
    registerDriverPid(pidRegistryDir(repoRoot), cid, tauriDriver.pid);
  },
  afterSession: () => {
    if (!tauriDriver) return;
    killDriverTree(tauriDriver);
    unregisterDriverPid(pidRegistryDir(repoRoot), currentCid, tauriDriver.pid);
    tauriDriver = undefined;
  },
  // Scoped to *this* worker's own cid — see reap.js: other workers can still
  // be legitimately mid-session in the same shared registry, and sweeping the
  // whole thing here (tried first) killed a sibling's still-running driver
  // out from under it.
  onWorkerEnd: (cid) => {
    reapRegisteredDriversForCid(pidRegistryDir(repoRoot), cid);
  },
  // Safe to reap everything here: nothing is still mid-session once
  // `onComplete` runs.
  onComplete: () => {
    reapRegisteredDrivers(pidRegistryDir(repoRoot));
  },
};
