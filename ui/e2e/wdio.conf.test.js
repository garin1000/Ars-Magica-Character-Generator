import { spawn, spawnSync } from 'node:child_process';
import { EventEmitter } from 'node:events';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  e2eBuildReusable,
  frontendIsFresh,
  rustInputsFingerprint,
  writeE2eBuildStamp,
} from './build-freshness.js';
import {
  allDriverPorts,
  driverPorts,
  workerConfigHome,
  workerIndexFromCid,
  workerSettingsFile,
} from './driver.js';
import { preflightDriverPorts } from './ports.js';
import { pidRegistryDir } from './reap.js';
import { config as portableConfig } from './wdio.portable.conf.js';
import {
  config as standardConfig,
  e2eExportFile,
  e2eFile,
  SPEC_UPDATE_SETUPS,
} from './wdio.conf.js';
import { config as reuseConfig } from './wdio.reuse.conf.js';
import { ensureOwnDisplay } from './xvfb.js';

// A fixed, recognizable stand-in pid — `registerDriverPid`/`killDriverTree`
// need a real number to write a registry filename and to assert the kill
// signal's target against.
const MOCK_DRIVER_PID = 987654;

// Mocked so `beforeSession` can be exercised without launching a real
// tauri-driver or hitting a real socket: `spawn` records the invocation, and
// `net.connect` fires `connect` on the next microtask so `waitForPort`
// resolves immediately instead of polling a port nothing is listening on.
// `vi.mock` is hoisted above these imports by vitest regardless of where it
// is written, so `spawn` above is already the mocked function.
// `execFileSync` is `display.js`'s `which xvfb-run` probe; a no-op answers
// "found", so `onPrepare`'s display preflight passes on a box without a display.
vi.mock('node:child_process', () => ({
  spawn: vi.fn(() => ({ kill: vi.fn(), pid: MOCK_DRIVER_PID })),
  spawnSync: vi.fn(),
  execFileSync: vi.fn(),
}));
// `onPrepare` probes the driver ports before it builds; with `node:net` mocked
// every port would read as occupied, so the preflight reports them all free.
vi.mock('./ports.js', () => ({
  preflightDriverPorts: vi.fn(async () => null),
}));
// The build-freshness decisions are unit-tested in `build-freshness.test.js`;
// here they are fixed per test, so the configs' wiring can be checked without
// hashing the real repository or writing the real stamps.
vi.mock('./build-freshness.js', async (importOriginal) => ({
  ...(await importOriginal()),
  frontendIsFresh: vi.fn(() => false),
  e2eBuildReusable: vi.fn(() => false),
  rustInputsFingerprint: vi.fn(() => 'rust-fingerprint'),
  writeE2eBuildStamp: vi.fn(),
}));
vi.mock('node:net', () => ({
  default: {
    connect: vi.fn(() => {
      const emitter = new EventEmitter();
      emitter.destroy = () => {};
      queueMicrotask(() => emitter.emit('connect'));
      return emitter;
    }),
  },
}));
// The own-Xvfb lifecycle itself (spawning, parsing -displayfd, teardown) is
// unit-tested in `xvfb.test.js`. Here it is fixed to "nothing to start" by
// default, so these tests exercise the configs' own wiring against it rather
// than needing a display or a real spawn; individual tests below override it
// with `mockResolvedValueOnce` to check what happens when there IS one to stop.
vi.mock('./xvfb.js', async (importOriginal) => ({
  ...(await importOriginal()),
  ensureOwnDisplay: vi.fn(async () => null),
}));

// An e2e run is expensive — a release build plus a full drive of the real
// binary. Its output is therefore worth keeping rather than re-running the
// suite to recover a line that scrolled past, so both configs set wdio's
// `outputDir`. The location is the load-bearing part: it must be the
// repo-local, gitignored `tmp/`, never the system temp dir. A path under
// `/tmp` puts the artifact outside the repo, where `git status` cannot see it
// and it is shared with every other project on the machine.
//
// (The save/load fixture the specs round-trip is the one deliberate exception:
// it goes through `os.tmpdir()` because the app's own native file dialogs need
// an OS-native target. That is committed, intentional, and unrelated to logs.)
const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const expectedLogDir = path.resolve(repoRoot, 'tmp/e2e-logs');

describe.each([
  ['standard', standardConfig],
  ['portable', portableConfig],
])('%s wdio config log output', (_name, config) => {
  it('writes run logs to the repo-local tmp/', () => {
    expect(config.outputDir).toBe(expectedLogDir);
  });

  it('does not write run logs to the system temp dir', () => {
    expect(config.outputDir.startsWith(os.tmpdir())).toBe(false);
  });
});

// `xvfb-run` on this machine closes fd 3 (Node's IPC channel to a forked
// worker) before exec'ing the wrapped command, killing every worker at its
// first `process.send` (see `xvfb.js`'s header). `onPrepare` now starts its own
// Xvfb instead (`ensureOwnDisplay`), so WebdriverIO's own wrapping must be off,
// or the two would race to supply a display.
describe.each([
  ['standard', standardConfig],
  ['portable', portableConfig],
])('%s wdio config autoXvfb', (_name, config) => {
  it("disables WebdriverIO's own Xvfb wrapping", () => {
    expect(config.autoXvfb).toBe(false);
  });
});

// The suite's wall clock is dominated by one ~30s app launch per spec file, and
// those launches are independent — each spec drives its own app from the start
// screen. Running them concurrently is therefore the one lever that scales,
// and it is only safe because every per-worker collision has been removed:
// the driver ports (`driver.js`), the save/export fixtures, and the app's
// settings file.
describe.each([
  ['standard', standardConfig],
  ['portable', portableConfig],
])('%s wdio config concurrency', (_name, config) => {
  it('runs more than one worker', () => {
    expect(config.maxInstances).toBeGreaterThan(1);
  });

  // A per-capability cap of 1 silently pins the run to one worker no matter
  // what the top-level `maxInstances` says, which is exactly the trap that
  // makes a "parallel" suite quietly stay serial.
  it('does not re-pin concurrency to 1 on the capability', () => {
    expect(config.capabilities[0].maxInstances).toBe(config.maxInstances);
  });
});

// `e2eFile`/`e2eExportFile` are read as module constants by the specs, which
// run *inside* the worker process (`WDIO_WORKER_ID` is set on that process's
// env before its modules ever load — see `@wdio/local-runner`). Two workers
// racing the standard suite must therefore land on different fixture paths, or
// one worker's save/export would clobber the other's mid-run.
describe('per-worker fixture isolation', () => {
  const originalWorkerId = process.env.WDIO_WORKER_ID;

  afterEach(() => {
    if (originalWorkerId === undefined) delete process.env.WDIO_WORKER_ID;
    else process.env.WDIO_WORKER_ID = originalWorkerId;
    vi.resetModules();
  });

  it('gives two different workers distinct e2eFile and e2eExportFile paths', async () => {
    vi.resetModules();
    process.env.WDIO_WORKER_ID = '0-1';
    const workerOne = await import('./wdio.conf.js');

    vi.resetModules();
    process.env.WDIO_WORKER_ID = '0-2';
    const workerTwo = await import('./wdio.conf.js');

    expect(workerOne.e2eFile).not.toBe(workerTwo.e2eFile);
    expect(workerOne.e2eExportFile).not.toBe(workerTwo.e2eExportFile);
  });
});

// The heart of A1: `beforeSession` must delegate to `driver.js`'s
// `startWorkerDriver` rather than spawning tauri-driver on the fixed default
// port and sleeping 2s. Verified against `spawn`'s actual call rather than by
// reading source, per the plan's "confirm by reading, do not assume".
const uiVersion = JSON.parse(
  fs.readFileSync(path.resolve(repoRoot, 'ui/package.json'), 'utf8'),
).version;
const updateFeed = path.resolve(repoRoot, 'ui/e2e/fixtures/update-feed-newer.json');
const invalidFeed = path.resolve(repoRoot, 'ui/e2e/fixtures/update-feed-invalid.json');

// The shipped check's whole-request timeout
// (`crates/arm-app/src/update_check.rs::FetchLimits::production`). A stall spec's
// hold must outlast it, or it would not stand in for a check that hangs.
const PRODUCTION_TIMEOUT_MS = 10000;

// The specs that hold the check back to prove the app stays usable, and quits
// promptly, while it hangs.
const STALL_SPECS = ['update-check-stall.e2e.js', 'update-check-stall-dirty-quit.e2e.js'];

/** A spec path in the `file://` URL form wdio hands `beforeSession`. */
function specUrl(basename) {
  return `file://${path.resolve(repoRoot, 'ui/e2e/specs', basename)}`;
}

function plantSettings(home, settings) {
  fs.mkdirSync(path.dirname(workerSettingsFile(home)), { recursive: true });
  fs.writeFileSync(workerSettingsFile(home), JSON.stringify(settings));
}

function readSettings(home) {
  return JSON.parse(fs.readFileSync(workerSettingsFile(home), 'utf8'));
}

describe.each([
  ['standard', () => standardConfig],
  ['portable', () => portableConfig],
])('%s config beforeSession', (name, getConfig) => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    fs.rmSync(workerSettingsFile(workerConfigHome(repoRoot, process.env)), { force: true });
  });

  it("spawns tauri-driver on this worker's own port pair, not the fixed default", async () => {
    const config = getConfig();
    const cid = '0-5';
    const { port, nativePort } = driverPorts(workerIndexFromCid(cid));

    await config.beforeSession(config, [{}], [], cid);

    expect(spawn).toHaveBeenCalledTimes(1);
    const [driverPath, args] = spawn.mock.calls[0];
    expect(driverPath).toContain('tauri-driver');
    expect(args).toEqual(['--port', String(port), '--native-port', String(nativePort)]);
    expect(config.port).toBe(port);
  });

  it('passes a per-worker XDG_CONFIG_HOME so concurrent app instances do not race on settings', async () => {
    const config = getConfig();
    await config.beforeSession(config, [{}], [], '0-6');

    const [, , options] = spawn.mock.calls[0];
    const expectedHome = workerConfigHome(repoRoot, process.env);
    expect(options.env.XDG_CONFIG_HOME).toBe(expectedHome);
  });

  // tao reads the colour scheme from the desktop portal over the session bus,
  // blocking up to 5 s per call; headless, the user bus's portal never answers
  // and every launch stalled ~30 s (docs/test-performance-2026-09.md §3). An
  // address with no socket behind it makes every bus client fail at once.
  describe('session bus', () => {
    const inheritedBus = process.env.DBUS_SESSION_BUS_ADDRESS;

    afterEach(() => {
      if (inheritedBus === undefined) delete process.env.DBUS_SESSION_BUS_ADDRESS;
      else process.env.DBUS_SESSION_BUS_ADDRESS = inheritedBus;
    });

    function spawnedBusAddress() {
      const [, , options] = spawn.mock.calls[0];
      return options.env.DBUS_SESSION_BUS_ADDRESS;
    }

    it('overrides an inherited session bus address', async () => {
      process.env.DBUS_SESSION_BUS_ADDRESS = 'unix:path=/inherited-user-bus';
      const config = getConfig();
      await config.beforeSession(config, [{}], [], '0-14');

      expect(spawnedBusAddress()).not.toBe('unix:path=/inherited-user-bus');
    });

    it('points the app at a unix socket that does not exist', async () => {
      delete process.env.DBUS_SESSION_BUS_ADDRESS;
      const config = getConfig();
      await config.beforeSession(config, [{}], [], '0-15');

      const address = spawnedBusAddress();
      expect(address).toEqual(expect.stringMatching(/^unix:path=\//));
      expect(fs.existsSync(address.slice('unix:path='.length))).toBe(false);
    });
  });

  // M6.6: the update-check consent prompt makes the whole shell `inert`, so an
  // ordinary spec must launch with the prompt already answered — "Don't check",
  // for the version that is about to run — and must never be handed a feed.
  it('seeds a "Don\'t check" answer before an ordinary spec, keeping other settings', async () => {
    const config = getConfig();
    const home = workerConfigHome(repoRoot, process.env);
    plantSettings(home, { lang: 'de', update_check: true, update_check_asked_version: '0.0.1' });

    await config.beforeSession(config, [{}], [specUrl('app-shell.e2e.js')], '0-9');

    expect(readSettings(home)).toEqual({
      lang: 'de',
      update_check: false,
      update_check_asked_version: uiVersion,
    });
    const [, , options] = spawn.mock.calls[0];
    expect(options.env.ARM_E2E_UPDATE_FEED).toBeUndefined();
    expect(options.env.ARM_E2E_UPDATE_STALL_MS).toBeUndefined();
  });

  if (name === 'standard') {
    // The consent spec is the one exception: a genuine first start (its worker's
    // config home emptied, so a retry starts over too), and the newer-release
    // feed the `e2e-testing` build reads instead of GitHub.
    it('gives the consent spec a first start and the newer-release feed', async () => {
      const config = getConfig();
      const home = workerConfigHome(repoRoot, process.env);
      plantSettings(home, { update_check: false, update_check_asked_version: uiVersion });

      await config.beforeSession(config, [{}], [specUrl('update-check.e2e.js')], '0-10');

      expect(fs.readdirSync(home)).toEqual([]);
      const [, , options] = spawn.mock.calls[0];
      expect(options.env.ARM_E2E_UPDATE_FEED).toBe(updateFeed);
      expect(options.env.ARM_E2E_UPDATE_STALL_MS).toBeUndefined();
      expect(options.env.XDG_CONFIG_HOME).toBe(home);
    });

    // The stall specs: the check must run at launch with no prompt in the way
    // ("Check" already answered for the running version), read the newer-release
    // feed, and be held back longer than the shipped timeout before it reads it —
    // by exactly the hold the spec itself measures against.
    it.each(STALL_SPECS)(
      'gives %s an enabled check, the feed, and a hold past the production timeout',
      async (spec) => {
        const config = getConfig();
        const home = workerConfigHome(repoRoot, process.env);
        plantSettings(home, {
          lang: 'de',
          update_check: false,
          update_check_asked_version: '0.0.1',
        });

        await config.beforeSession(config, [{}], [specUrl(spec)], '0-13');

        expect(readSettings(home)).toEqual({
          lang: 'de',
          update_check: true,
          update_check_asked_version: uiVersion,
        });
        const [, , options] = spawn.mock.calls[0];
        expect(options.env.ARM_E2E_UPDATE_FEED).toBe(updateFeed);
        expect(options.env.ARM_E2E_UPDATE_STALL_MS).toBe(String(SPEC_UPDATE_SETUPS[spec].stallMs));
        expect(Number(options.env.ARM_E2E_UPDATE_STALL_MS)).toBeGreaterThan(PRODUCTION_TIMEOUT_MS);
      },
    );

    // E6 (full-audit round 1): a spec proving a malformed feed stays silent must
    // get its own feed file, named by `feed` in `SPEC_UPDATE_SETUPS`, rather than
    // the newer-release one every other update-check spec shares — and no stall,
    // since this spec measures an absence that would otherwise be ambiguous with
    // "still checking".
    it('gives the invalid-feed spec an enabled check and its own malformed feed', async () => {
      const config = getConfig();
      const home = workerConfigHome(repoRoot, process.env);
      plantSettings(home, {
        update_check: false,
        update_check_asked_version: '0.0.1',
      });

      await config.beforeSession(
        config,
        [{}],
        [specUrl('update-check-invalid-feed.e2e.js')],
        '0-16',
      );

      expect(readSettings(home)).toEqual({
        update_check: true,
        update_check_asked_version: uiVersion,
      });
      const [, , options] = spawn.mock.calls[0];
      expect(options.env.ARM_E2E_UPDATE_FEED).toBe(invalidFeed);
      expect(options.env.ARM_E2E_UPDATE_STALL_MS).toBeUndefined();
    });

    it('ships an invalid feed whose tag_name is not a semver release tag', () => {
      expect(JSON.parse(fs.readFileSync(invalidFeed, 'utf8'))).toEqual({
        tag_name: 'not-a-version',
        draft: false,
        prerelease: false,
      });
    });

    it('ships a feed announcing a release newer than any real one', () => {
      expect(JSON.parse(fs.readFileSync(updateFeed, 'utf8'))).toEqual({
        tag_name: 'v99.0.0',
        draft: false,
        prerelease: false,
      });
    });

    it('seams in the save/export fixtures for the app to inherit', async () => {
      const config = getConfig();
      await config.beforeSession(config, [{}], [], '0-7');

      const [, , options] = spawn.mock.calls[0];
      expect(options.env.ARM_E2E_FILE).toBe(e2eFile);
      expect(options.env.ARM_E2E_EXPORT_FILE).toBe(e2eExportFile);
    });
  }

  if (name === 'portable') {
    // The portable smoke check deliberately never reaches the dirty-quit /
    // save-load native-dialog seams: it builds without `--features
    // e2e-testing`, so an app spawned with these vars would ignore them, but
    // shipping them anyway would misdocument what this config actually does.
    it('does not seam in the save/export fixtures', async () => {
      const config = getConfig();
      await config.beforeSession(config, [{}], [], '0-8');

      const [, , options] = spawn.mock.calls[0];
      expect(options.env.ARM_E2E_FILE).toBeUndefined();
      expect(options.env.ARM_E2E_EXPORT_FILE).toBeUndefined();
    });
  }
});

// U6 (docs/open-todos.md, "the e2e harness leaks its driver processes"):
// `afterSession` must actually terminate the driver `beforeSession` spawned.
//
// `beforeSession` is called with `@wdio/runner`'s own `this._config` — built by
// `deepmerge`-ing this file's exported `config` into the runner's defaults
// (`@wdio/config`'s `ConfigParser`) — which is a *different object* from this
// module's `export const config`. The portable config used to store the
// handle as `config.tauriDriver`, a property of whatever object
// `beforeSession` received; `afterSession` takes no parameter and reads the
// bare identifier `config`, resolved by lexical scope to this file's
// `export const config` — never the merged object `beforeSession` actually
// mutated. So the property read back `undefined` and `.kill()` was never
// called. Passing a shallow copy (not the same reference) as the
// `beforeSession` parameter reproduces that mismatch faithfully; passing the
// literal `config` object (as the `beforeSession` describe block above does,
// to check `config.port` mutation) would not, since same-reference args are
// exactly what makes the old bug invisible.
describe.each([
  ['standard', () => standardConfig],
  ['portable', () => portableConfig],
])('%s config afterSession', (_name, getConfig) => {
  let killSpy;

  beforeEach(() => {
    vi.clearAllMocks();
    killSpy = vi.spyOn(process, 'kill').mockImplementation(() => true);
    // Decontaminates a legacy artifact: the "beforeSession" describe block
    // above passes the literal shared `config` object (not a copy) into
    // `beforeSession`, so a leftover `config.tauriDriver` from one of those
    // tests could otherwise leak into these — irrelevant to the fixed
    // implementation (which never sets that property at all), but without
    // this the reverted, buggy version's assertions become run-order
    // dependent rather than reliably red.
    delete getConfig().tauriDriver;
  });

  afterEach(() => {
    killSpy.mockRestore();
    fs.rmSync(pidRegistryDir(repoRoot), { recursive: true, force: true });
  });

  it('signals the process group of the driver beforeSession spawned', async () => {
    const config = getConfig();
    await config.beforeSession({ ...config }, [{}], [], '0-11');

    config.afterSession();

    expect(killSpy).toHaveBeenCalledWith(-MOCK_DRIVER_PID, 'SIGTERM');
  });

  it('clears the pid registry entry once the driver is reaped', async () => {
    const config = getConfig();
    await config.beforeSession({ ...config }, [{}], [], '0-12');
    const registeredEntry = `0-12-${MOCK_DRIVER_PID}`;
    expect(fs.readdirSync(pidRegistryDir(repoRoot))).toContain(registeredEntry);

    config.afterSession();

    expect(fs.readdirSync(pidRegistryDir(repoRoot))).not.toContain(registeredEntry);
  });
});

// U6 round two: a real e2e run caught `onWorkerEnd` sweeping the *whole*
// shared pid registry the instant the first worker's spec finished, killing
// every other concurrently-running worker's still-legitimate driver (a
// cascade of `ECONNREFUSED` across every other spec). `onWorkerEnd` must
// touch only the cid it is told just ended.
describe.each([
  ['standard', () => standardConfig],
  ['portable', () => portableConfig],
])('%s config onWorkerEnd', (_name, getConfig) => {
  afterEach(() => {
    fs.rmSync(pidRegistryDir(repoRoot), { recursive: true, force: true });
  });

  it("reaps only the ended worker's own entry, never a sibling still mid-session", () => {
    const config = getConfig();
    const dir = pidRegistryDir(repoRoot);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(path.resolve(dir, '0-1-1001'), '');
    fs.writeFileSync(path.resolve(dir, '0-2-2002'), '');
    const killSpy = vi.spyOn(process, 'kill').mockImplementation(() => true);

    config.onWorkerEnd('0-1', 0, [], 0);

    expect(killSpy).toHaveBeenCalledWith(-1001, 'SIGTERM');
    expect(killSpy).not.toHaveBeenCalledWith(-2002, 'SIGTERM');
    expect(fs.readdirSync(dir)).toEqual(['0-2-2002']);
    killSpy.mockRestore();
  });
});

// `@wdio/cli` sets `NODE_ENV=test` in the launcher when it is unset, `onPrepare`
// runs there, and Vite keeps an inherited `NODE_ENV` — so a build that merely
// inherits the launcher's env embeds a development frontend, not the one that
// ships (docs/test-performance-2026-09.md §3a). vitest sets the same `test`.
// The spawn is made to fail so the hook stops right after it, before staging.
describe.each([
  ['standard', () => standardConfig],
  ['portable', () => portableConfig],
])('%s config onPrepare', (_name, getConfig) => {
  // A current frontend, so the cargo build is the one spawn (see "frontend build").
  beforeEach(() => {
    vi.clearAllMocks();
    spawnSync.mockReturnValue({ status: 1 });
    frontendIsFresh.mockReturnValue(true);
  });

  afterEach(() => {
    spawnSync.mockReset();
    frontendIsFresh.mockReturnValue(false);
  });

  it('builds the release binary with a production frontend', async () => {
    await expect(getConfig().onPrepare()).rejects.toThrow(/cargo tauri build failed/);

    expect(spawnSync).toHaveBeenCalledTimes(1);
    const [command, args, options] = spawnSync.mock.calls[0];
    expect([command, ...args.slice(0, 2)]).toEqual(['cargo', 'tauri', 'build']);
    expect(options.env?.NODE_ENV).toBe('production');
    expect(options.env?.PATH).toBe(process.env.PATH);
  });

  it("checks the ports of exactly the run's worker count", async () => {
    const config = getConfig();
    await expect(config.onPrepare()).rejects.toThrow();

    expect(preflightDriverPorts).toHaveBeenCalledWith(allDriverPorts(config.maxInstances));
  });
});

// `xvfb-run` on this machine closes fd 3 before exec'ing the wrapped command,
// killing every worker's IPC channel (see `xvfb.js`'s header). `onPrepare` now
// asks `ensureOwnDisplay` for a display itself, before anything else, and must
// tear down whatever it started if a later preparation step fails — the
// success path (kept alive through the whole run, stopped only in
// `onComplete`) is covered in "standard config e2e build" below, where a full
// successful `onPrepare` is already exercised.
describe.each([
  ['standard', () => standardConfig],
  ['portable', () => portableConfig],
])('%s config own-Xvfb lifecycle', (_name, getConfig) => {
  beforeEach(() => {
    vi.clearAllMocks();
    spawnSync.mockReturnValue({ status: 1 });
    frontendIsFresh.mockReturnValue(true);
  });

  afterEach(() => {
    spawnSync.mockReset();
    frontendIsFresh.mockReturnValue(false);
  });

  it('asks for a display before anything else in onPrepare', async () => {
    await expect(getConfig().onPrepare()).rejects.toThrow();

    expect(ensureOwnDisplay).toHaveBeenCalledTimes(1);
  });

  it('stops an own Xvfb it started when a later preparation step fails', async () => {
    const stop = vi.fn();
    ensureOwnDisplay.mockResolvedValueOnce({ display: ':123', pid: 4321, stop });

    await expect(getConfig().onPrepare()).rejects.toThrow();

    expect(stop).toHaveBeenCalledTimes(1);
  });
});

/** The `--config` override a build spawn carries, parsed, or `undefined`. */
function configOverride(args) {
  const at = args.indexOf('--config');
  return at === -1 ? undefined : JSON.parse(args[at + 1]);
}

// tauri-cli hands `--config` to cargo as the TAURI_CONFIG env var, and
// tauri-build reruns on any change to it (`cargo:rerun-if-env-changed`), which
// recompiles arm-app (~45 s). So a target dir must see the SAME `--config` on
// every build. The e2e build, alone in `target/e2e`, always removes
// `beforeBuildCommand` (a JSON merge-patch `null`) and runs the production
// frontend build itself, only when `ui/dist` is not current
// (`build-freshness.js`). The portable build shares `target/release` with the
// gate's plain `cargo tauri build`, so it must build exactly like the gate.
describe('standard config frontend build', () => {
  let log;

  beforeEach(() => {
    vi.clearAllMocks();
    spawnSync.mockReturnValue({ status: 0 });
    log = vi.spyOn(console, 'log').mockImplementation(() => {});
    vi.spyOn(fs, 'cpSync').mockImplementation(() => {});
  });

  afterEach(() => {
    spawnSync.mockReset();
    frontendIsFresh.mockReturnValue(false);
    log.mockRestore();
    fs.cpSync.mockRestore();
  });

  function cargoCalls() {
    return spawnSync.mock.calls.filter(([command]) => command === 'cargo');
  }

  function npmCalls() {
    return spawnSync.mock.calls.filter(([command]) => command === 'npm');
  }

  it('always removes beforeBuildCommand, so TAURI_CONFIG never changes in target/e2e', async () => {
    frontendIsFresh.mockReturnValue(false);
    await standardConfig.onPrepare();
    frontendIsFresh.mockReturnValue(true);
    await standardConfig.onPrepare();

    const overrides = cargoCalls().map(([, args]) => configOverride(args));
    expect(overrides).toEqual([
      { build: { beforeBuildCommand: null } },
      { build: { beforeBuildCommand: null } },
    ]);
  });

  it('builds the production frontend first when ui/dist is not current', async () => {
    frontendIsFresh.mockReturnValue(false);
    await standardConfig.onPrepare();

    expect(npmCalls()).toHaveLength(1);
    const [, args, options] = npmCalls()[0];
    expect(args).toEqual(['run', 'build']);
    expect(options.cwd).toBe(path.resolve(repoRoot, 'ui'));
    expect(options.env.NODE_ENV).toBe('production');
    expect(spawnSync.mock.calls.map(([command]) => command)).toEqual(['npm', 'cargo']);
  });

  it('skips the frontend build when ui/dist is current, saying so', async () => {
    frontendIsFresh.mockReturnValue(true);
    await standardConfig.onPrepare();

    expect(npmCalls()).toHaveLength(0);
    expect(log).toHaveBeenCalledWith(expect.stringMatching(/frontend/i));
  });

  it('stops before cargo when the frontend build fails', async () => {
    frontendIsFresh.mockReturnValue(false);
    spawnSync.mockReturnValue({ status: 1 });
    await expect(standardConfig.onPrepare()).rejects.toThrow(/frontend build failed/);

    expect(cargoCalls()).toHaveLength(0);
  });
});

describe('portable config frontend build', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    spawnSync.mockReturnValue({ status: 1 });
  });

  afterEach(() => {
    spawnSync.mockReset();
    frontendIsFresh.mockReturnValue(false);
  });

  it.each([false, true])(
    'builds exactly like the gate, with no --config (frontend current: %s)',
    async (fresh) => {
      frontendIsFresh.mockReturnValue(fresh);
      await expect(portableConfig.onPrepare()).rejects.toThrow(/cargo tauri build failed/);

      expect(spawnSync).toHaveBeenCalledTimes(1);
      const [command, args] = spawnSync.mock.calls[0];
      expect([command, ...args]).toEqual(['cargo', 'tauri', 'build', '--no-bundle']);
    },
  );
});

// The `e2e-testing` binary gets its own cargo target dir, so it never
// overwrites the plain `target/release/arm-app` the gate built — the seams
// hazard of docs/test-performance-2026-09.md §5 — and each of the two stays a
// cargo no-op while its inputs are unchanged.
describe('standard config e2e build', () => {
  const e2eRelease = path.resolve(repoRoot, 'target/e2e/release');
  const originalReuse = process.env.ARM_E2E_REUSE_BUILD;
  let cpSync;
  let log;

  // A current frontend, so the cargo build is the one spawn (see "frontend build").
  beforeEach(() => {
    vi.clearAllMocks();
    spawnSync.mockReturnValue({ status: 0 });
    frontendIsFresh.mockReturnValue(true);
    cpSync = vi.spyOn(fs, 'cpSync').mockImplementation(() => {});
    log = vi.spyOn(console, 'log').mockImplementation(() => {});
    delete process.env.ARM_E2E_REUSE_BUILD;
  });

  afterEach(() => {
    spawnSync.mockReset();
    frontendIsFresh.mockReturnValue(false);
    e2eBuildReusable.mockReturnValue(false);
    cpSync.mockRestore();
    log.mockRestore();
    if (originalReuse === undefined) delete process.env.ARM_E2E_REUSE_BUILD;
    else process.env.ARM_E2E_REUSE_BUILD = originalReuse;
  });

  function rulesCopyDestinations() {
    return cpSync.mock.calls.map(([, dest]) => dest);
  }

  it('drives the binary in target/e2e/release', () => {
    expect(standardConfig.capabilities[0]['tauri:options'].application).toBe(
      path.resolve(e2eRelease, 'arm-app'),
    );
  });

  it('builds the e2e-testing binary into target/e2e with a production frontend', async () => {
    await standardConfig.onPrepare();

    expect(spawnSync).toHaveBeenCalledTimes(1);
    const [, args, options] = spawnSync.mock.calls[0];
    expect(args).toEqual(expect.arrayContaining(['--features', 'e2e-testing']));
    expect(options.env.CARGO_TARGET_DIR).toBe(path.resolve(repoRoot, 'target/e2e'));
    expect(options.env.NODE_ENV).toBe('production');
  });

  it('copies rules/core and rules/i18n beside the e2e binary', async () => {
    await standardConfig.onPrepare();

    expect(rulesCopyDestinations()).toEqual([
      path.resolve(e2eRelease, 'rules/core'),
      path.resolve(e2eRelease, 'rules/i18n'),
    ]);
  });

  // The Rust inputs are fingerprinted before cargo starts, so an edit made
  // while it compiles is never stamped as built.
  it('records a successful build for a later reuse', async () => {
    await standardConfig.onPrepare();

    expect(writeE2eBuildStamp).toHaveBeenCalledWith(repoRoot, 'rust-fingerprint');
    expect(rustInputsFingerprint.mock.invocationCallOrder[0]).toBeLessThan(
      spawnSync.mock.invocationCallOrder[0],
    );
  });

  it('records nothing for a failed build', async () => {
    spawnSync.mockReturnValue({ status: 1 });
    await expect(standardConfig.onPrepare()).rejects.toThrow(/cargo tauri build failed/);

    expect(writeE2eBuildStamp).not.toHaveBeenCalled();
  });

  // Opt-in only, and only while the last e2e build is still current.
  it('skips cargo only with ARM_E2E_REUSE_BUILD=1 and a current build', async () => {
    e2eBuildReusable.mockReturnValue(true);
    await standardConfig.onPrepare();
    expect(spawnSync).toHaveBeenCalledTimes(1);

    process.env.ARM_E2E_REUSE_BUILD = '1';
    e2eBuildReusable.mockReturnValue(false);
    await standardConfig.onPrepare();
    expect(spawnSync).toHaveBeenCalledTimes(2);

    e2eBuildReusable.mockReturnValue(true);
    await standardConfig.onPrepare();
    expect(spawnSync).toHaveBeenCalledTimes(2);
  });

  it('says loudly that it reused the binary, and still stages the rules', async () => {
    process.env.ARM_E2E_REUSE_BUILD = '1';
    e2eBuildReusable.mockReturnValue(true);
    await standardConfig.onPrepare();

    expect(log).toHaveBeenCalledWith(expect.stringMatching(/ARM_E2E_REUSE_BUILD/));
    expect(rulesCopyDestinations()).toEqual([
      path.resolve(e2eRelease, 'rules/core'),
      path.resolve(e2eRelease, 'rules/i18n'),
    ]);
  });

  // `npm run test:e2e:reuse`: the same suite, reuse on, with no environment
  // prefix needed — so it works from any shell and from an agent.
  it('has a reuse config that reuses a current build without the variable', async () => {
    expect(reuseConfig.specs).toEqual(standardConfig.specs);
    e2eBuildReusable.mockReturnValue(true);
    await reuseConfig.onPrepare();

    expect(spawnSync).not.toHaveBeenCalled();
  });

  // The own-Xvfb failure path is covered above ("own-Xvfb lifecycle"); this is
  // the success path, only exercisable here where `onPrepare` is already set up
  // to complete without throwing.
  it('keeps a started own Xvfb running through a successful prepare, stopping it only on completion', async () => {
    const stop = vi.fn();
    ensureOwnDisplay.mockResolvedValueOnce({ display: ':123', pid: 4321, stop });

    await standardConfig.onPrepare();
    expect(stop).not.toHaveBeenCalled();

    standardConfig.onComplete();
    expect(stop).toHaveBeenCalledTimes(1);
  });
});

// `ARM_E2E_WORKERS` overrides how many spec files run at once. The configs read
// it when they load, so each case loads them afresh.
describe('ARM_E2E_WORKERS', () => {
  const original = process.env.ARM_E2E_WORKERS;

  afterEach(() => {
    if (original === undefined) delete process.env.ARM_E2E_WORKERS;
    else process.env.ARM_E2E_WORKERS = original;
    vi.resetModules();
  });

  async function loadConfigsWith(value) {
    vi.resetModules();
    if (value === undefined) delete process.env.ARM_E2E_WORKERS;
    else process.env.ARM_E2E_WORKERS = value;
    const childProcess = await import('node:child_process');
    const ports = await import('./ports.js');
    const standard = (await import('./wdio.conf.js')).config;
    const portable = (await import('./wdio.portable.conf.js')).config;
    return { childProcess, ports, configs: [standard, portable] };
  }

  it('sets the worker count of both configs, on the capability too', async () => {
    const { configs } = await loadConfigsWith('6');
    for (const config of configs) {
      expect(config.maxInstances).toBe(6);
      expect(config.capabilities[0].maxInstances).toBe(6);
    }
  });

  it('has the port preflight check every port the overridden count binds', async () => {
    const { childProcess, ports, configs } = await loadConfigsWith('7');
    childProcess.spawnSync.mockReturnValue({ status: 1 });
    for (const config of configs) {
      ports.preflightDriverPorts.mockClear();
      await expect(config.onPrepare()).rejects.toThrow();
      expect(ports.preflightDriverPorts).toHaveBeenCalledWith(allDriverPorts(7));
    }
  });

  it('keeps the default when unset or empty', async () => {
    const unset = (await loadConfigsWith(undefined)).configs[0].maxInstances;
    const empty = (await loadConfigsWith('')).configs[0].maxInstances;
    expect(unset).toBeGreaterThan(1);
    expect(empty).toBe(unset);
  });

  // A typo must not silently fall back to some other count.
  it.each(['0', '-2', 'six', '2.5', '4x'])('refuses %j with a message naming it', async (value) => {
    await expect(loadConfigsWith(value).then(() => 'loaded')).rejects.toThrow(/ARM_E2E_WORKERS/);
  });
});

// The standard config lists its specs explicitly, longest first, so the long
// ones do not start last and leave workers idle at the end. A glob could never
// drop a new spec; a list can, so the list is held to the files on disk.
describe('standard config spec list', () => {
  const specsDir = path.resolve(repoRoot, 'ui/e2e/specs');
  const onDisk = fs
    .readdirSync(specsDir, { recursive: true })
    .filter((name) => name.endsWith('.e2e.js'))
    .map((name) => path.resolve(specsDir, name));

  it('names every spec file on disk, and nothing else', () => {
    expect([...standardConfig.specs].sort()).toEqual([...onDisk].sort());
  });

  it('names no spec file twice', () => {
    expect(new Set(standardConfig.specs).size).toBe(standardConfig.specs.length);
  });
});

// A2 (spec consolidation) merged 40 single-purpose spec files into 7, and the
// comments across the codebase that pointed a reader at "the e2e counterpart"
// kept naming the files that had gone. A dangling pointer is worse than no
// pointer: it reads as a promise that the behaviour is covered somewhere, and
// the reader who goes looking finds nothing and cannot tell whether the spec was
// renamed or the coverage was dropped. Round-1 audit finding 1d listed eight such
// names; this guard's first run found thirteen, across twenty-five sites — and one
// of them was hiding a behaviour (`.icon-btn`'s pointer-target size) that really
// had lost its e2e measurement in the merge with nobody noticing. That gap between
// what a hand audit saw and what the sweep sees is the reason this is a test.
//
// This is the same shape as `crates/arm-rules/tests/source_citations.rs`, which
// checks that a `file::symbol` cross-reference still resolves. Here the claim is
// narrower and purely mechanical: every `*.e2e.js` basename named in live code
// must be a spec file that exists.
//
// `docs/` IS DELIBERATELY NOT SCANNED, and that is project policy rather than an
// oversight (CLAUDE.md, "A cross-reference to another source file"): `docs/`
// holds dated historical records — implementation plans, reviews, findings
// sheets — where the name a file had at the time is part of the snapshot, and
// rewriting it to today's name would falsify the record.
describe('e2e spec citations', () => {
  // Non-spec harness modules (`helpers.js`, `wizard-walk.js`) live at the `e2e/`
  // root and are not spec files, so only these two directories count.
  const SPEC_DIRS = ['ui/e2e/specs', 'ui/e2e/portable'];
  // Live code only. `docs/` is excluded by being absent here — see above.
  const SCAN_ROOTS = ['ui/src', 'ui/e2e', 'crates'];
  const SCAN_EXTENSIONS = ['.js', '.ts', '.svelte', '.css', '.rs', '.md'];
  const SKIP_DIRS = new Set(['node_modules', 'target', 'dist', '.git']);

  // Starts on a word character, so a glob (`specs/**/*.e2e.js` in a wdio config,
  // `window-close-bridge-*.e2e.js` in a prose comment) matches nothing: a `*` is
  // not a claim that a file by that name exists.
  const CITATION = /[A-Za-z0-9_][A-Za-z0-9_.-]*\.e2e\.js/g;
  // `ex-<name>` is the codebase's explicit marker for a file the consolidation
  // removed — `wizard-flow.e2e.js` (ex-`tab-area.e2e.js`). It says out loud that
  // the name is historical, which is the opposite of a dangling pointer, so it
  // is allowed to name a file that no longer exists.
  const HISTORICAL_MARKER = /ex-`?$/;

  function filesUnder(dir) {
    const found = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        if (!SKIP_DIRS.has(entry.name)) found.push(...filesUnder(path.join(dir, entry.name)));
      } else if (SCAN_EXTENSIONS.includes(path.extname(entry.name))) {
        found.push(path.join(dir, entry.name));
      }
    }
    return found;
  }

  const realSpecs = new Set(
    SPEC_DIRS.flatMap((dir) =>
      fs.readdirSync(path.resolve(repoRoot, dir)).filter((name) => name.endsWith('.e2e.js')),
    ),
  );

  // The scan itself is asserted, not assumed: a roots list that silently stopped
  // matching anything would make every assertion below vacuously green.
  it('finds the spec files and the sources that cite them', () => {
    expect(realSpecs.size).toBeGreaterThan(0);
    const scanned = SCAN_ROOTS.flatMap((root) => filesUnder(path.resolve(repoRoot, root)));
    expect(scanned.length).toBeGreaterThan(0);
  });

  it('names only spec files that exist', () => {
    const dangling = [];
    for (const root of SCAN_ROOTS) {
      for (const file of filesUnder(path.resolve(repoRoot, root))) {
        const source = fs.readFileSync(file, 'utf8');
        for (const match of source.matchAll(CITATION)) {
          if (realSpecs.has(match[0])) continue;
          if (HISTORICAL_MARKER.test(source.slice(0, match.index))) continue;
          const line = source.slice(0, match.index).split('\n').length;
          dangling.push(`${path.relative(repoRoot, file)}:${line} cites ${match[0]}`);
        }
      }
    }
    // Collected rather than asserted one at a time, so a failure names every
    // offender instead of stopping at the first.
    expect(dangling).toEqual([]);
  });
});
