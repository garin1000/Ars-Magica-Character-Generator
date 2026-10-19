import { EventEmitter } from 'node:events';

import { describe, expect, it, vi } from 'vitest';

import {
  buildXvfbArgs,
  ensureOwnDisplay,
  parseDisplayNumber,
  readDisplayNumber,
  shouldStartOwnXvfb,
  startOwnXvfb,
} from './xvfb.js';

// `xvfb-run` on this machine closes fd 3 — Node's IPC channel to a forked
// worker — before exec'ing the wrapped command, so every `@wdio/local-runner`
// worker died at `process.send` with `write EINVAL`. This module replaces
// WebdriverIO's own `xvfb-run` wrapping with exactly one Xvfb this suite
// starts and stops itself, in the launcher process (`onPrepare`/`onComplete`),
// before any worker is forked. See `README.md` and `CLAUDE.local.md`.

describe('shouldStartOwnXvfb', () => {
  const linux = 'linux';

  it('is false when a display is already there', () => {
    expect(shouldStartOwnXvfb({ DISPLAY: ':0' }, linux)).toBe(false);
  });

  it('is true on Linux with no DISPLAY', () => {
    expect(shouldStartOwnXvfb({}, linux)).toBe(true);
  });

  it('treats a blank DISPLAY as no display', () => {
    expect(shouldStartOwnXvfb({ DISPLAY: '   ' }, linux)).toBe(true);
  });

  // A Wayland-only session exports no DISPLAY either, and takes the same path.
  it('is true on a Wayland-only session', () => {
    expect(shouldStartOwnXvfb({ WAYLAND_DISPLAY: 'wayland-0' }, linux)).toBe(true);
  });

  // macOS/Windows drive a native webview and never set DISPLAY; never start an
  // X server there.
  it.each(['darwin', 'win32'])('is always false off Linux (%s)', (platform) => {
    expect(shouldStartOwnXvfb({}, platform)).toBe(false);
  });
});

describe('buildXvfbArgs', () => {
  it('matches xvfb-run default screen (1280x1024x24) with no args listened on tcp', () => {
    expect(buildXvfbArgs()).toEqual([
      '-displayfd',
      '3',
      '-screen',
      '0',
      '1280x1024x24',
      '-nolisten',
      'tcp',
    ]);
  });

  it('lets a custom screen size through', () => {
    expect(buildXvfbArgs({ width: 800, height: 600, depth: 16 })).toEqual([
      '-displayfd',
      '3',
      '-screen',
      '0',
      '800x600x16',
      '-nolisten',
      'tcp',
    ]);
  });

  it('always asks Xvfb to pick its own free display number via -displayfd 3', () => {
    expect(buildXvfbArgs()).toEqual(expect.arrayContaining(['-displayfd', '3']));
  });
});

describe('parseDisplayNumber', () => {
  it('parses a bare number', () => {
    expect(parseDisplayNumber('99')).toBe(99);
  });

  it('parses a number with a trailing newline', () => {
    expect(parseDisplayNumber('102\n')).toBe(102);
  });

  it('rejects anything that is not a plain decimal number', () => {
    expect(() => parseDisplayNumber('')).toThrow();
    expect(() => parseDisplayNumber('abc')).toThrow();
    expect(() => parseDisplayNumber(':99')).toThrow();
  });
});

describe('readDisplayNumber', () => {
  function fakeStream() {
    return new EventEmitter();
  }

  it('resolves once the stream reports a number', async () => {
    const stream = fakeStream();
    const result = readDisplayNumber(stream, { timeoutMs: 1000 });
    stream.emit('data', Buffer.from('99\n'));
    await expect(result).resolves.toBe(99);
  });

  it('buffers chunks until a newline arrives', async () => {
    const stream = fakeStream();
    const result = readDisplayNumber(stream, { timeoutMs: 1000 });
    stream.emit('data', Buffer.from('1'));
    stream.emit('data', Buffer.from('03\n'));
    await expect(result).resolves.toBe(103);
  });

  it('rejects if the stream closes before reporting anything', async () => {
    const stream = fakeStream();
    const result = readDisplayNumber(stream, { timeoutMs: 1000 });
    stream.emit('close');
    await expect(result).rejects.toThrow(/exited/);
  });

  it('rejects on a stream error', async () => {
    const stream = fakeStream();
    const result = readDisplayNumber(stream, { timeoutMs: 1000 });
    stream.emit('error', new Error('boom'));
    await expect(result).rejects.toThrow('boom');
  });

  it('rejects once the timeout passes with nothing reported', async () => {
    const stream = fakeStream();
    await expect(readDisplayNumber(stream, { timeoutMs: 10 })).rejects.toThrow(/did not report/);
  });
});

describe('startOwnXvfb', () => {
  function fakeChild() {
    const child = new EventEmitter();
    child.pid = 5555;
    child.killed = false;
    child.kill = vi.fn(() => {
      child.killed = true;
    });
    child.stdio = [null, null, null, new EventEmitter()];
    return child;
  }

  it('spawns Xvfb with -displayfd 3 and resolves the display once it reports', async () => {
    const child = fakeChild();
    const spawnFn = vi.fn(() => child);

    const promise = startOwnXvfb({ spawnFn, timeoutMs: 1000 });
    child.stdio[3].emit('data', Buffer.from('77\n'));
    const handle = await promise;

    expect(spawnFn).toHaveBeenCalledWith('Xvfb', buildXvfbArgs(), expect.any(Object));
    expect(handle.display).toBe(':77');
    expect(handle.pid).toBe(5555);
  });

  it('stop() kills the spawned process', async () => {
    const child = fakeChild();
    const spawnFn = vi.fn(() => child);
    const promise = startOwnXvfb({ spawnFn, timeoutMs: 1000 });
    child.stdio[3].emit('data', Buffer.from('77\n'));
    const handle = await promise;

    handle.stop();

    expect(child.kill).toHaveBeenCalledWith('SIGTERM');
  });

  it('stop() is a no-op if the process is already dead', async () => {
    const child = fakeChild();
    const spawnFn = vi.fn(() => child);
    const promise = startOwnXvfb({ spawnFn, timeoutMs: 1000 });
    child.stdio[3].emit('data', Buffer.from('77\n'));
    const handle = await promise;
    child.killed = true;

    expect(() => handle.stop()).not.toThrow();
    expect(child.kill).not.toHaveBeenCalled();
  });

  it('rejects and kills the half-started process if Xvfb fails to spawn', async () => {
    const child = fakeChild();
    const spawnFn = vi.fn(() => child);
    const promise = startOwnXvfb({ spawnFn, timeoutMs: 1000 });
    child.emit('error', new Error('ENOENT'));

    await expect(promise).rejects.toThrow('ENOENT');
    expect(child.kill).toHaveBeenCalledWith('SIGTERM');
  });
});

describe('ensureOwnDisplay', () => {
  function fakeChild() {
    const child = new EventEmitter();
    child.pid = 5555;
    child.killed = false;
    child.kill = vi.fn(() => {
      child.killed = true;
    });
    child.stdio = [null, null, null, new EventEmitter()];
    return child;
  }

  it('does nothing and returns null when a display is already there', async () => {
    const spawnFn = vi.fn();
    const env = { DISPLAY: ':0' };

    const handle = await ensureOwnDisplay({ env, platform: 'linux', spawnFn });

    expect(handle).toBeNull();
    expect(spawnFn).not.toHaveBeenCalled();
    expect(env.DISPLAY).toBe(':0');
  });

  it('starts Xvfb and sets DISPLAY on the given env when there is none', async () => {
    const child = fakeChild();
    const spawnFn = vi.fn(() => child);
    const env = {};

    const promise = ensureOwnDisplay({ env, platform: 'linux', spawnFn });
    child.stdio[3].emit('data', Buffer.from('55\n'));
    const handle = await promise;

    expect(handle.display).toBe(':55');
    expect(env.DISPLAY).toBe(':55');
  });

  it('never starts Xvfb off Linux', async () => {
    const spawnFn = vi.fn();
    const env = {};

    const handle = await ensureOwnDisplay({ env, platform: 'darwin', spawnFn });

    expect(handle).toBeNull();
    expect(spawnFn).not.toHaveBeenCalled();
    expect(env.DISPLAY).toBeUndefined();
  });
});
