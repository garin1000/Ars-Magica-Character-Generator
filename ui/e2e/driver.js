// Per-worker tauri-driver plumbing, shared by the standard and portable e2e
// configs.
//
// Two things used to make the suite strictly serial, and both live here now.
//
// 1. **A fixed port.** `beforeSession` span tauri-driver on 4444 and
//    `afterSession` killed it, so a second concurrent worker collided with the
//    first. `driverPorts` gives every worker its own pair — the intermediary
//    port WDIO talks to, and the native WebDriver port tauri-driver puts
//    WebKitWebDriver on.
// 2. **A hardcoded `setTimeout(resolve, 2000)`** standing in for "the driver has
//    bound its port". Paid 43 times, that is ~1.4 minutes of sleeping, and it is
//    simultaneously too long on a fast machine and a coin flip on a loaded one.
//    `waitForPort` polls instead, so the wait is as short as it can be and a
//    driver that never binds fails with a message that names the port rather
//    than an inscrutable session error half a minute later.
//
// Everything here is pure or takes its side effect by injection, which is what
// lets `driver.test.js` prove the arithmetic and the polling loop without
// launching anything.

import { spawn } from 'node:child_process';
import fs from 'node:fs';
import net from 'node:net';
import os from 'node:os';
import path from 'node:path';

/** The port worker 0 uses — the one the suite has always used. */
const BASE_PORT = 4444;

/**
 * Ports per worker: one for tauri-driver, one for the native WebDriver it
 * spawns.
 */
const PORTS_PER_WORKER = 2;

/**
 * How many distinct worker slots the range covers before wrapping.
 *
 * WDIO's cid counts *spec files*, not concurrent workers — a 43-spec run
 * reaches `0-42` even at `maxInstances: 1` — so the index climbs far past the
 * number of drivers ever alive at once. Wrapping keeps the range bounded well
 * inside the ephemeral-port floor while leaving far more slots than any
 * plausible `maxInstances`, and the climb is a feature in itself: consecutive
 * spec files land on different ports, so a driver still shutting down cannot
 * collide with the next one starting.
 */
const WORKER_SLOTS = 100;

/**
 * The worker number out of a WDIO cid (`"0-3"` → `3`).
 *
 * The launcher process has no cid and evaluates the config too, so anything
 * unparseable reads as worker 0 — which keeps a single-worker run on the
 * historical port.
 *
 * @param {string | undefined | null} cid
 * @returns {number}
 */
export function workerIndexFromCid(cid) {
  const match = /-(\d+)$/.exec(cid ?? '');
  return match ? Number(match[1]) : 0;
}

/**
 * The port pair for a worker.
 *
 * @param {number} workerIndex
 * @returns {{ port: number, nativePort: number }}
 */
export function driverPorts(workerIndex) {
  const slot = ((workerIndex % WORKER_SLOTS) + WORKER_SLOTS) % WORKER_SLOTS;
  const port = BASE_PORT + slot * PORTS_PER_WORKER;
  return { port, nativePort: port + 1 };
}

/**
 * The suffix that makes a per-worker file name, or `''` for a lone worker.
 *
 * Specs import the save/export fixture paths as module constants and are
 * evaluated *inside* the worker process, so reading WDIO's own
 * `WDIO_WORKER_ID` here is enough to keep two parallel workers off each other's
 * files — no plumbing through the config required.
 *
 * @param {Record<string, string | undefined>} env
 * @returns {string}
 */
export function workerSuffix(env) {
  const id = env.WDIO_WORKER_ID;
  return id ? `-${id}` : '';
}

/**
 * Resolves once something is listening on `port`, or rejects on timeout.
 *
 * @param {number} port
 * @param {object} [options]
 * @param {(port: number) => Promise<boolean>} [options.connect] probe, injected by tests
 * @param {number} [options.timeoutMs]
 * @param {number} [options.intervalMs]
 * @returns {Promise<void>}
 */
export async function waitForPort(port, options = {}) {
  const { connect = tcpProbe, timeoutMs = 30000, intervalMs = 25 } = options;
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    if (await connect(port)) return;
    if (Date.now() >= deadline) {
      throw new Error(`tauri-driver did not bind port ${port} within ${timeoutMs}ms`);
    }
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
}

/**
 * A per-worker `XDG_CONFIG_HOME`, created on demand.
 *
 * The app persists its settings (the saga year today) under
 * `app_config_dir()`, which on Linux hangs off `XDG_CONFIG_HOME` — one path
 * shared by every instance. Two concurrent workers would race on that file, so
 * each gets its own tree.
 *
 * A welcome side effect: the suite stops writing the developer's real settings
 * file. `saga-year.e2e.js` used to do exactly that and restore it afterwards;
 * now it starts from a guaranteed-clean directory instead of from whatever the
 * machine happened to hold, which is both safer and more deterministic.
 *
 * @param {string} repoRoot
 * @param {Record<string, string | undefined>} env
 * @returns {string} absolute path, guaranteed to exist
 */
export function workerConfigHome(repoRoot, env) {
  const dir = path.resolve(repoRoot, `tmp/e2e-config${workerSuffix(env) || '-0'}`);
  fs.mkdirSync(dir, { recursive: true });
  return dir;
}

/**
 * Spawns this worker's tauri-driver, points the config at it, and resolves once
 * it is actually listening.
 *
 * Mutating `config.port` here is load-bearing and safe: the runner hands
 * `beforeSession` the live config object and only calls `_initSession` after
 * the hook resolves (`@wdio/runner`), so the session is created against this
 * worker's port rather than the shared default.
 *
 * @param {object} config the live WDIO config, mutated in place
 * @param {string | undefined} cid
 * @param {Record<string, string>} [extraEnv] passed down to the spawned app
 * @returns {Promise<import('node:child_process').ChildProcess>}
 */
export async function startWorkerDriver(config, cid, extraEnv = {}) {
  const { port, nativePort } = driverPorts(workerIndexFromCid(cid));
  const child = spawn(
    path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
    ['--port', String(port), '--native-port', String(nativePort)],
    {
      stdio: [null, process.stdout, process.stderr],
      env: { ...process.env, ...extraEnv },
    },
  );
  config.port = port;
  // Replaces a hardcoded 2s sleep: as short as the machine allows, and a loud
  // failure naming the port if the driver never comes up.
  await waitForPort(port);
  return child;
}

/**
 * One TCP connect attempt against localhost. Resolves `true` when the port
 * accepts, `false` on any refusal — never rejects, so the caller's loop stays
 * a plain condition.
 *
 * @param {number} port
 * @returns {Promise<boolean>}
 */
function tcpProbe(port) {
  return new Promise((resolve) => {
    const socket = net.connect({ host: '127.0.0.1', port });
    const settle = (result) => {
      socket.destroy();
      resolve(result);
    };
    socket.once('connect', () => settle(true));
    socket.once('error', () => settle(false));
  });
}
