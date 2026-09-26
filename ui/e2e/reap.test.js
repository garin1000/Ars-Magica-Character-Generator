import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  killDriverTree,
  pidRegistryDir,
  reapDriverTrees,
  reapRegisteredDrivers,
  reapRegisteredDriversForCid,
  registerDriverPid,
  unregisterDriverPid,
} from './reap.js';

// Reaping the driver process(es) this run spawned — never anything found by
// port or name, so a user's unrelated process is never touched. Two halves:
// `killDriverTree`/`reapDriverTrees` terminate handles a config still holds a
// live reference to (the normal `afterSession` path); the pid registry lets
// `onComplete` (which runs in a different OS process than the worker that
// spawned the driver — see @wdio/runner, onPrepare/onComplete run in the
// launcher, beforeSession/afterSession in the worker) reap whatever a crashed
// or aborted session left behind, without ever guessing by port or name.

describe('killDriverTree', () => {
  it('signals the process group of the given handle', () => {
    const killFn = vi.fn();
    killDriverTree({ pid: 4444 }, { killFn });
    expect(killFn).toHaveBeenCalledWith(-4444, 'SIGTERM');
  });

  it('does nothing for a handle with no pid', () => {
    const killFn = vi.fn();
    killDriverTree(null, { killFn });
    killDriverTree(undefined, { killFn });
    killDriverTree({}, { killFn });
    expect(killFn).not.toHaveBeenCalled();
  });

  it('treats "no such process" as already reaped, without falling back to a direct signal', () => {
    const killFn = vi.fn(() => {
      const err = new Error('no such process');
      err.code = 'ESRCH';
      throw err;
    });
    expect(() => killDriverTree({ pid: 4444 }, { killFn })).not.toThrow();
    expect(killFn).toHaveBeenCalledTimes(1);
  });

  it('falls back to signalling the pid directly if the group signal is refused for another reason', () => {
    const killFn = vi.fn((pid) => {
      if (pid < 0) {
        const err = new Error('operation not permitted');
        err.code = 'EPERM';
        throw err;
      }
    });
    killDriverTree({ pid: 4444 }, { killFn });
    expect(killFn).toHaveBeenNthCalledWith(1, -4444, 'SIGTERM');
    expect(killFn).toHaveBeenNthCalledWith(2, 4444, 'SIGTERM');
  });
});

describe('reapDriverTrees', () => {
  // The whole point of tracking handles explicitly rather than discovering
  // processes by port or name: an untracked handle — standing in for some
  // unrelated process the user happens to be running — must never be touched.
  it('terminates only the tracked handle, never an untracked one', () => {
    const killFn = vi.fn();
    const tracked = { pid: 111 };
    const untracked = { pid: 222 };

    reapDriverTrees([tracked], { killFn });

    expect(killFn).toHaveBeenCalledWith(-111, 'SIGTERM');
    expect(killFn).not.toHaveBeenCalledWith(-222, 'SIGTERM');
    expect(killFn).not.toHaveBeenCalledWith(222, 'SIGTERM');
  });

  it('terminates every handle in a list of several', () => {
    const killFn = vi.fn();
    reapDriverTrees([{ pid: 1 }, { pid: 2 }, { pid: 3 }], { killFn });
    expect(killFn).toHaveBeenCalledWith(-1, 'SIGTERM');
    expect(killFn).toHaveBeenCalledWith(-2, 'SIGTERM');
    expect(killFn).toHaveBeenCalledWith(-3, 'SIGTERM');
  });

  it('skips null/undefined entries without throwing', () => {
    const killFn = vi.fn();
    expect(() => reapDriverTrees([null, undefined, { pid: 5 }], { killFn })).not.toThrow();
    expect(killFn).toHaveBeenCalledTimes(1);
  });
});

describe('pid registry', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arm-e2e-reap-test-'));

  afterEach(() => {
    fs.rmSync(dir, { recursive: true, force: true });
    fs.mkdirSync(dir, { recursive: true });
  });

  it('pidRegistryDir resolves under the repo-local tmp/, never the system temp dir', () => {
    const repoRoot = '/repo';
    expect(pidRegistryDir(repoRoot)).toBe(path.resolve(repoRoot, 'tmp/e2e-driver-pids'));
  });

  it('registerDriverPid creates an entry that reapRegisteredDrivers finds and clears', () => {
    registerDriverPid(dir, '0-1', 9001);
    const killFn = vi.fn();

    const reaped = reapRegisteredDrivers(dir, { killFn });

    expect(reaped).toEqual([9001]);
    expect(killFn).toHaveBeenCalledWith(-9001, 'SIGTERM');
    expect(fs.readdirSync(dir)).toEqual([]);
  });

  it('unregisterDriverPid removes an entry before it is ever reaped', () => {
    registerDriverPid(dir, '0-1', 9002);
    unregisterDriverPid(dir, '0-1', 9002);
    const killFn = vi.fn();

    const reaped = reapRegisteredDrivers(dir, { killFn });

    expect(reaped).toEqual([]);
    expect(killFn).not.toHaveBeenCalled();
  });

  it('reaps only the pids actually registered, never an unrelated one', () => {
    registerDriverPid(dir, '0-1', 111);
    const killFn = vi.fn();

    reapRegisteredDrivers(dir, { killFn });

    expect(killFn).not.toHaveBeenCalledWith(-222, 'SIGTERM');
  });

  it('is a no-op, not a throw, when the registry directory does not exist yet', () => {
    const missing = path.join(dir, 'does-not-exist');
    expect(() => reapRegisteredDrivers(missing)).not.toThrow();
    expect(reapRegisteredDrivers(missing)).toEqual([]);
  });
});

// This is the bug a live e2e run actually caught (U6): `onWorkerEnd` fires
// once per worker, in the launcher, the moment that worker's session ends —
// but while several *other* workers are still concurrently mid-session, each
// with its own driver legitimately registered in the same shared directory.
// Reaping the *whole* registry from `onWorkerEnd` (as `reapRegisteredDrivers`
// does, correctly, from `onComplete` once every worker is done) kills every
// still-running sibling's driver out from under it. Observed directly: the
// first spec to finish triggered a cascade of `ECONNREFUSED` across every
// other concurrently-running spec. `reapRegisteredDriversForCid` scopes the
// sweep to one worker's own entries, which is what makes it safe to call
// mid-run.
describe('reapRegisteredDriversForCid', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'arm-e2e-reap-cid-test-'));

  afterEach(() => {
    fs.rmSync(dir, { recursive: true, force: true });
    fs.mkdirSync(dir, { recursive: true });
  });

  it("reaps only the ended worker's own driver, never a concurrently-running sibling's", () => {
    registerDriverPid(dir, '0-1', 1001); // the worker that just ended
    registerDriverPid(dir, '0-2', 2002); // still running
    registerDriverPid(dir, '0-3', 3003); // still running
    const killFn = vi.fn();

    const reaped = reapRegisteredDriversForCid(dir, '0-1', { killFn });

    expect(reaped).toEqual([1001]);
    expect(killFn).toHaveBeenCalledWith(-1001, 'SIGTERM');
    expect(killFn).not.toHaveBeenCalledWith(-2002, 'SIGTERM');
    expect(killFn).not.toHaveBeenCalledWith(-3003, 'SIGTERM');
    // The siblings' registrations must still be there for their own eventual
    // afterSession/onWorkerEnd to reap.
    const remaining = fs.readdirSync(dir);
    expect(remaining).toHaveLength(2);
  });

  it('is a no-op for a cid with nothing registered', () => {
    registerDriverPid(dir, '0-2', 2002);
    const killFn = vi.fn();

    expect(reapRegisteredDriversForCid(dir, '0-9', { killFn })).toEqual([]);
    expect(killFn).not.toHaveBeenCalled();
    expect(fs.readdirSync(dir)).toHaveLength(1);
  });

  it('is a no-op, not a throw, when the registry directory does not exist yet', () => {
    const missing = path.join(dir, 'does-not-exist');
    expect(() => reapRegisteredDriversForCid(missing, '0-1')).not.toThrow();
  });
});
