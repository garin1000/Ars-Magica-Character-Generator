import { spawn } from 'node:child_process';
import { EventEmitter } from 'node:events';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { driverPorts, workerConfigHome, workerIndexFromCid } from './driver.js';
import { config as portableConfig } from './wdio.portable.conf.js';
import { config as standardConfig, e2eExportFile, e2eFile } from './wdio.conf.js';

// Mocked so `beforeSession` can be exercised without launching a real
// tauri-driver or hitting a real socket: `spawn` records the invocation, and
// `net.connect` fires `connect` on the next microtask so `waitForPort`
// resolves immediately instead of polling a port nothing is listening on.
// `vi.mock` is hoisted above these imports by vitest regardless of where it
// is written, so `spawn` above is already the mocked function.
vi.mock('node:child_process', () => ({
  spawn: vi.fn(() => ({ kill: vi.fn() })),
  spawnSync: vi.fn(),
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
describe.each([
  ['standard', () => standardConfig],
  ['portable', () => portableConfig],
])('%s config beforeSession', (name, getConfig) => {
  beforeEach(() => {
    vi.clearAllMocks();
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

  if (name === 'standard') {
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
