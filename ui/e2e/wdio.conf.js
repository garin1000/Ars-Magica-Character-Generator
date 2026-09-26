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

import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { hasXvfbRun, preflightDisplay } from './display.js';
import { allDriverPorts, startWorkerDriver, workerConfigHome, workerSuffix } from './driver.js';
import { preflightDriverPorts } from './ports.js';
import {
  killDriverTree,
  pidRegistryDir,
  reapRegisteredDrivers,
  reapRegisteredDriversForCid,
  registerDriverPid,
  unregisterDriverPid,
} from './reap.js';
import { e2eLogDir, MAX_WORKERS, sharedWdioConfig } from './wdio.shared.conf.js';

// Xvfb offers no DRI3 extension and no GPU, so Mesa's EGL hardware probe fails
// and falls back to swrast on its own — noisy (`libEGL warning: DRI3 ...`) and
// discovered per worker rather than decided once. Pin the driver explicitly
// instead: llvmpipe is the same software rasterizer the fallback would have
// picked, just without the probe-then-fail step. `startWorkerDriver` spawns
// tauri-driver with `{ ...process.env, ...extraEnv }`, and tauri-driver spawns
// the app inheriting that same env, so setting this once here — before either
// process exists — reaches both, headful or under `xvfb-run`.
process.env.LIBGL_ALWAYS_SOFTWARE = '1';
process.env.GALLIUM_DRIVER = 'llvmpipe';

const dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(dirname, '../..');
const application = path.resolve(repoRoot, 'target/release/arm-app');

// Per-worker save/load target, so the flow stays deterministic and headless
// while two concurrent workers no longer round-trip the same file. Specs
// import this as a module constant and are evaluated inside the worker
// process, where `WDIO_WORKER_ID` is already set on `process.env` before any
// module loads (see `@wdio/local-runner`) — so `workerSuffix` here reads the
// right value with no plumbing through the config needed.
export const e2eFile = path.resolve(
  os.tmpdir(),
  `arm-e2e-character${workerSuffix(process.env)}.json`,
);

// Per-worker Markdown-export target, deliberately a separate seam from
// `e2eFile`: that one is the JSON save file the same specs round-trip, so
// sharing it would have an export clobber the document on disk.
export const e2eExportFile = path.resolve(
  os.tmpdir(),
  `arm-e2e-character${workerSuffix(process.env)}.md`,
);

let tauriDriver;
// `afterSession` is not handed `cid` (only `beforeSession` is — see
// `@wdio/runner`'s `afterSessionArgs`), so it is tracked here alongside the
// driver handle itself, the same way `tauriDriver` already is.
let currentCid;

export const config = {
  ...sharedWdioConfig(application),
  specs: [path.resolve(dirname, 'specs/**/*.e2e.js')],

  // Keep the full launcher + worker logs of a run. A suite this expensive should
  // never have to be re-run just to re-read output that scrolled past, so wdio
  // writes them here instead of only to the terminal — no shell redirect needed,
  // which is what makes this usable from an agent. Repo-local and gitignored
  // (`tmp/`), deliberately NOT the system temp dir: an artifact under `/tmp` is
  // invisible to `git status` and shared with every other project on the box.
  outputDir: e2eLogDir(repoRoot),

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
  onPrepare: async () => {
    const problem = preflightDisplay(process.env, { xvfbRunAvailable: hasXvfbRun() });
    if (problem) throw new Error(problem);

    // Refuses immediately, naming the port, if a stale driver from a previous
    // run is still holding one of the ports this run's workers would bind —
    // instead of this run connecting to it and hanging the full ~120s
    // session-creation timeout before an inscrutable `WebDriverError: timeout`
    // (docs/open-todos.md, "the e2e harness leaks its driver processes").
    const portProblem = await preflightDriverPorts(allDriverPorts(MAX_WORKERS));
    if (portProblem) throw new Error(portProblem);

    // The preflight above just proved every one of this run's ports is free,
    // which means any driver a *previous* run's pid registry still names is
    // already dead (whether cleanly reaped or a genuine orphan) — so it is
    // safe to discard stale bookkeeping here rather than ever attempting to
    // signal a pid that might since have been reused by an unrelated process.
    fs.rmSync(pidRegistryDir(repoRoot), { recursive: true, force: true });

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
  // export skip their native dialogs, and a per-worker XDG_CONFIG_HOME so
  // concurrent app instances do not race on the settings file
  // (`app.path().app_config_dir()`, `crates/arm-app/src/commands.rs`).
  //
  // This hook already runs inside the display WebdriverIO arranged — the real
  // one, or the `xvfb-run` wrapper around this worker — so `DISPLAY` is set
  // either way and everything spawned below (WebKitWebDriver, then the app)
  // inherits it. There is nothing to set up here beyond the driver itself.
  //
  // `startWorkerDriver` (driver.js) replaces a fixed port 4444 plus a
  // hardcoded 2s sleep: it gives this worker its own port pair and polls until
  // tauri-driver is actually listening. Mutating `config.port` here is safe —
  // `@wdio/runner` calls `beforeSession` with the live config and only creates
  // the session afterwards (`_initSession`).
  beforeSession: async (config, capabilities, specs, cid) => {
    tauriDriver = await startWorkerDriver(config, cid, {
      ARM_E2E_FILE: e2eFile,
      ARM_E2E_EXPORT_FILE: e2eExportFile,
      XDG_CONFIG_HOME: workerConfigHome(repoRoot, process.env),
    });
    // Recorded so the `onComplete`/`onWorkerEnd` safety net below can still
    // find and terminate this driver even if this worker crashes before its
    // own `afterSession` runs — see reap.js for why that needs a cross-process
    // registry rather than the `tauriDriver` handle itself.
    currentCid = cid;
    registerDriverPid(pidRegistryDir(repoRoot), cid, tauriDriver.pid);
  },
  afterSession: () => {
    if (!tauriDriver) return;
    killDriverTree(tauriDriver);
    unregisterDriverPid(pidRegistryDir(repoRoot), currentCid, tauriDriver.pid);
    tauriDriver = undefined;
  },
  // Runs in the launcher process, per worker, just after that worker exits —
  // including a crashed or forcibly-terminated one, which is exactly the case
  // `afterSession` above cannot cover. Scoped to *this* worker's own cid:
  // other workers can still be legitimately mid-session in the same shared
  // registry, so reaping the whole thing here (as tried first) kills a
  // sibling's still-running driver out from under it.
  onWorkerEnd: (cid) => {
    reapRegisteredDriversForCid(pidRegistryDir(repoRoot), cid);
  },
  // The final sweep, once every worker is done — safe to reap everything here
  // because nothing is still mid-session. A normal run leaves the registry
  // empty (each `afterSession` already cleared its own entry), so this only
  // ever does anything when a worker's teardown did not run.
  onComplete: () => {
    reapRegisteredDrivers(pidRegistryDir(repoRoot));
  },
};
