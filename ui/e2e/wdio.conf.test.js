import { spawn } from 'node:child_process';
import { EventEmitter } from 'node:events';
import fs from 'node:fs';
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
