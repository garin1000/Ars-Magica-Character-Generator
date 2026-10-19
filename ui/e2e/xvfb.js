// Own-Xvfb management for the e2e suite.
//
// WebdriverIO (`@wdio/local-runner` → `@wdio/xvfb`) used to supply a display on
// its own: when `DISPLAY` is unset on Linux it wraps the forked worker process
// in `xvfb-run --auto-servernum -- node …`. On this machine, since a distro
// upgrade, `/usr/bin/xvfb-run` closes fd 3 before exec'ing the wrapped command
// (`DISPLAY=:$N XAUTHORITY=$AUTHFILE "$@" 3>&-`) — and fd 3 is exactly the IPC
// channel Node's `fork()` gives a worker (`stdio: [..., 'ipc']`). So every
// worker died at its very first `process.send` with `write EINVAL`, 7s in,
// all 17 spec files failing before a single test ran.
//
// The fix stops asking WebdriverIO for a display at all (`autoXvfb: false`,
// `wdio.shared.conf.js`) and starts exactly one Xvfb of its own in `onPrepare`
// — the launcher process, before any worker is forked, which is the only place
// setting `DISPLAY` here is guaranteed to reach every worker (a worker-side
// hook is already too late: `@wdio/local-runner` decides and forks before any
// hook of its own runs — see `display.js`'s and `README.md`'s "Headful and
// headless"). `-displayfd` has Xvfb itself pick a free server number and
// report it back once it is actually ready to accept connections, which is
// both simpler and race-free compared to scanning `/tmp/.X*-lock` by hand —
// the same approach `xvfb-run` itself takes, just one layer further out.
//
// With a display already present (`DISPLAY` set — a desktop session), none of
// this runs and behaviour is exactly as before: headful, on the real display.

import { spawn } from 'node:child_process';

const isSet = (value) => typeof value === 'string' && value.trim() !== '';

/**
 * The screen Xvfb opens unless told otherwise — the same resolution and depth
 * `xvfb-run` has always used by default (`XVFBARGS="-screen 0 1280x1024x24"`
 * in `/usr/bin/xvfb-run`), so switching away from the wrapper changes nothing
 * about what the suite renders into.
 */
const DEFAULT_SCREEN = { width: 1280, height: 1024, depth: 24 };

/**
 * Whether the suite should start its own Xvfb rather than rely on whatever
 * display is already there.
 *
 * Mirrors `display.js`'s `preflightDisplay` (and what `@wdio/xvfb`'s own
 * `shouldRun` used to check) exactly: Linux only (macOS/Windows drive a native
 * webview and never set `DISPLAY`), and only when `DISPLAY` is unset or blank
 * — a Wayland-only session exports no `DISPLAY` either, and takes this path
 * too.
 *
 * @param {Record<string, string | undefined>} env
 * @param {string} [platform]
 * @returns {boolean}
 */
export function shouldStartOwnXvfb(env, platform = process.platform) {
  return platform === 'linux' && !isSet(env.DISPLAY);
}

/**
 * The argv Xvfb is spawned with. `-displayfd 3` has it write the display
 * number it picked to fd 3 once ready, rather than taking one on the command
 * line — see the file header for why that is preferred over picking one
 * ourselves. `-nolisten tcp` matches `xvfb-run`'s default (no network-exposed
 * X11).
 *
 * @param {{ width?: number, height?: number, depth?: number }} [screen]
 * @returns {string[]}
 */
export function buildXvfbArgs(screen = {}) {
  const { width, height, depth } = { ...DEFAULT_SCREEN, ...screen };
  return ['-displayfd', '3', '-screen', '0', `${width}x${height}x${depth}`, '-nolisten', 'tcp'];
}

/**
 * Parses what Xvfb wrote to its `-displayfd` pipe: a bare decimal number
 * (optionally followed by a newline). Anything else means Xvfb did not behave
 * the way this module expects, and is a loud failure rather than a silently
 * wrong `DISPLAY`.
 *
 * @param {string} output
 * @returns {number}
 */
export function parseDisplayNumber(output) {
  const trimmed = output.trim();
  if (!/^\d+$/.test(trimmed)) {
    throw new Error(`Xvfb's -displayfd wrote an unexpected value: ${JSON.stringify(output)}`);
  }
  return Number(trimmed);
}

/**
 * Resolves with the display number once `stream` (Xvfb's `-displayfd` pipe)
 * reports one, or rejects if Xvfb closes the pipe first (it died before
 * becoming ready), errors, or nothing arrives within `timeoutMs`.
 *
 * @param {import('node:stream').Readable} stream
 * @param {{ timeoutMs?: number }} [options]
 * @returns {Promise<number>}
 */
export function readDisplayNumber(stream, { timeoutMs = 10000 } = {}) {
  return new Promise((resolve, reject) => {
    let buffer = '';
    const timer = setTimeout(() => {
      cleanup();
      reject(new Error(`Xvfb did not report a display number within ${timeoutMs}ms`));
    }, timeoutMs);
    const cleanup = () => {
      clearTimeout(timer);
      stream.removeListener('data', onData);
      stream.removeListener('error', onError);
      stream.removeListener('close', onClose);
    };
    const onData = (chunk) => {
      buffer += chunk.toString('utf8');
      if (!buffer.includes('\n')) return;
      cleanup();
      try {
        resolve(parseDisplayNumber(buffer));
      } catch (err) {
        reject(err);
      }
    };
    const onError = (err) => {
      cleanup();
      reject(err);
    };
    const onClose = () => {
      cleanup();
      reject(new Error('Xvfb exited before reporting a display number'));
    };
    stream.on('data', onData);
    stream.once('error', onError);
    stream.once('close', onClose);
  });
}

/**
 * Starts exactly one Xvfb and resolves once it is ready to accept
 * connections.
 *
 * @param {object} [options]
 * @param {typeof spawn} [options.spawnFn] injected by tests
 * @param {{ width?: number, height?: number, depth?: number }} [options.screen]
 * @param {number} [options.timeoutMs]
 * @returns {Promise<{ display: string, pid: number, stop: () => void }>}
 */
export function startOwnXvfb({ spawnFn = spawn, screen, timeoutMs } = {}) {
  return new Promise((resolve, reject) => {
    const child = spawnFn('Xvfb', buildXvfbArgs(screen), {
      // fd 2 (stderr) inherited so a startup failure is visible in the run's
      // own log (`outputDir`, see `wdio.shared.conf.js`); fd 3 is the
      // `-displayfd` pipe this module reads the display number from.
      stdio: ['ignore', 'ignore', 'inherit', 'pipe'],
    });
    let settled = false;

    const stop = () => {
      if (!child.killed) child.kill('SIGTERM');
    };

    const onSpawnError = (err) => {
      if (settled) return;
      settled = true;
      stop();
      reject(err);
    };
    child.once('error', onSpawnError);

    readDisplayNumber(child.stdio[3], { timeoutMs }).then(
      (displayNumber) => {
        if (settled) return;
        settled = true;
        child.removeListener('error', onSpawnError);
        resolve({ display: `:${displayNumber}`, pid: child.pid, stop });
      },
      (err) => {
        if (settled) return;
        settled = true;
        child.removeListener('error', onSpawnError);
        stop();
        reject(err);
      },
    );
  });
}

/**
 * The one entry point both wdio configs' `onPrepare` call: starts this
 * suite's own Xvfb when (and only when) {@link shouldStartOwnXvfb} says the
 * run needs one, and sets `DISPLAY` on `env` so every worker forked after this
 * resolves inherits it — `env` is `process.env` in production, injected in
 * tests.
 *
 * @param {object} [options]
 * @param {Record<string, string | undefined>} [options.env]
 * @param {string} [options.platform]
 * @param {typeof spawn} [options.spawnFn]
 * @param {{ width?: number, height?: number, depth?: number }} [options.screen]
 * @returns {Promise<{ display: string, pid: number, stop: () => void } | null>}
 */
export async function ensureOwnDisplay({
  env = process.env,
  platform = process.platform,
  spawnFn,
  screen,
} = {}) {
  if (!shouldStartOwnXvfb(env, platform)) return null;
  const handle = await startOwnXvfb({ spawnFn, screen });
  env.DISPLAY = handle.display;
  return handle;
}
