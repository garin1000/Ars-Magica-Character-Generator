import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { describe, expect, it, vi } from 'vitest';

import {
  driverPorts,
  waitForPort,
  workerConfigHome,
  workerIndexFromCid,
  workerSuffix,
} from './driver.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');

// The e2e suite's per-spec startup dominates its wall clock, and the two things
// standing between it and parallel workers are both here: a tauri-driver bound
// to one fixed port, and a hardcoded sleep standing in for readiness. These
// helpers are pure (or injectable) precisely so the port arithmetic and the
// polling loop can be proven without launching a driver.

describe('workerIndexFromCid', () => {
  it.each([
    ['0-0', 0],
    ['0-3', 3],
    ['0-42', 42],
    ['1-7', 7],
  ])('reads the worker number out of cid %s', (cid, expected) => {
    expect(workerIndexFromCid(cid)).toBe(expected);
  });

  // The launcher process has no cid, and `wdio.conf.js` is evaluated there too.
  // Falling back to 0 keeps a single-worker run on the historical port.
  it.each([undefined, null, '', 'nonsense'])('falls back to 0 for %s', (cid) => {
    expect(workerIndexFromCid(cid)).toBe(0);
  });
});

describe('driverPorts', () => {
  it('puts worker 0 on the historical 4444', () => {
    expect(driverPorts(0).port).toBe(4444);
  });

  it('gives tauri-driver and the native driver distinct ports', () => {
    const { port, nativePort } = driverPorts(0);
    expect(nativePort).not.toBe(port);
  });

  // The whole point: two concurrent workers must not collide on either port.
  it('never overlaps two workers on any port', () => {
    const seen = new Set();
    for (let index = 0; index < 64; index += 1) {
      const { port, nativePort } = driverPorts(index);
      expect(seen.has(port), `port ${port} reused at worker ${index}`).toBe(false);
      expect(seen.has(nativePort), `port ${nativePort} reused at worker ${index}`).toBe(false);
      seen.add(port);
      seen.add(nativePort);
    }
  });

  it('stays inside the valid TCP range even for an absurd worker count', () => {
    for (const index of [0, 99, 1000, 100000]) {
      const { port, nativePort } = driverPorts(index);
      expect(port).toBeGreaterThan(1024);
      expect(nativePort).toBeGreaterThan(1024);
      expect(port).toBeLessThan(65536);
      expect(nativePort).toBeLessThan(65536);
    }
  });
});

describe('workerSuffix', () => {
  // Specs import the fixture paths as module constants and run inside the
  // worker process, so deriving the suffix from WDIO_WORKER_ID is what keeps
  // two parallel workers from clobbering each other's save file.
  it('is empty when there is no worker id, keeping the single-worker path', () => {
    expect(workerSuffix({})).toBe('');
  });

  it('is derived from WDIO_WORKER_ID when wdio set one', () => {
    expect(workerSuffix({ WDIO_WORKER_ID: '0-3' })).toBe('-0-3');
  });

  it('distinguishes two workers', () => {
    expect(workerSuffix({ WDIO_WORKER_ID: '0-1' })).not.toBe(
      workerSuffix({ WDIO_WORKER_ID: '0-2' }),
    );
  });
});

describe('waitForPort', () => {
  it('resolves as soon as the port accepts a connection', async () => {
    const connect = vi.fn().mockResolvedValue(true);
    await expect(waitForPort(4444, { connect, timeoutMs: 1000, intervalMs: 1 })).resolves.toBe(
      undefined,
    );
    expect(connect).toHaveBeenCalledTimes(1);
  });

  it('keeps polling until the port comes up', async () => {
    const connect = vi
      .fn()
      .mockResolvedValueOnce(false)
      .mockResolvedValueOnce(false)
      .mockResolvedValue(true);
    await waitForPort(4444, { connect, timeoutMs: 1000, intervalMs: 1 });
    expect(connect).toHaveBeenCalledTimes(3);
  });

  // A driver that never binds must fail loudly rather than hand WDIO a dead
  // port and produce an inscrutable session error 30s later.
  it('rejects with the port in the message once the timeout passes', async () => {
    const connect = vi.fn().mockResolvedValue(false);
    await expect(waitForPort(4599, { connect, timeoutMs: 20, intervalMs: 1 })).rejects.toThrow(
      /4599/,
    );
  });
});

describe('workerConfigHome', () => {
  // The app resolves its settings file through `app_config_dir()`, which on
  // Linux hangs off XDG_CONFIG_HOME — one shared path unless each worker gets
  // its own, so two concurrent workers would otherwise race on the same file.
  it('creates and returns a directory that exists', () => {
    const dir = workerConfigHome(repoRoot, { WDIO_WORKER_ID: '0-1' });
    expect(fs.existsSync(dir)).toBe(true);
    expect(fs.statSync(dir).isDirectory()).toBe(true);
  });

  it('gives two workers distinct directories', () => {
    const a = workerConfigHome(repoRoot, { WDIO_WORKER_ID: '0-1' });
    const b = workerConfigHome(repoRoot, { WDIO_WORKER_ID: '0-2' });
    expect(a).not.toBe(b);
    expect(fs.existsSync(a)).toBe(true);
    expect(fs.existsSync(b)).toBe(true);
  });

  // Repo-local `tmp/`, never the system temp dir — see CLAUDE.md and
  // CLAUDE.local.md on scratch artifacts.
  it('resolves under the repo-local tmp/, never the system temp dir', () => {
    const dir = workerConfigHome(repoRoot, { WDIO_WORKER_ID: '0-1' });
    expect(dir.startsWith(path.resolve(repoRoot, 'tmp'))).toBe(true);
  });

  it('falls back to a stable directory when there is no worker id', () => {
    const dir = workerConfigHome(repoRoot, {});
    expect(fs.existsSync(dir)).toBe(true);
    expect(dir.startsWith(path.resolve(repoRoot, 'tmp'))).toBe(true);
  });
});
