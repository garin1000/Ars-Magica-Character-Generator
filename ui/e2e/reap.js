// Reaping the driver process(es) an e2e run spawned (docs/open-todos.md,
// "the e2e harness leaks its driver processes"). Two halves.
//
// `killDriverTree`/`reapDriverTrees` terminate a live `ChildProcess` handle a
// config still holds a reference to — the normal `afterSession` path. They
// signal the *process group*, not just the pid: `startWorkerDriver`
// (`driver.js`) spawns tauri-driver with `detached: true`, which on POSIX
// makes it the leader of a new group, and any child it spawns —
// WebKitWebDriver — inherits that same group id. A plain `child.kill()` only
// reaches tauri-driver itself; if tauri-driver does not forward the signal to
// WebKitWebDriver before exiting, the grandchild is orphaned and keeps
// holding its port. Signalling the group reaches both without ever needing
// WebKitWebDriver's own pid.
//
// The pid registry (`registerDriverPid`/`unregisterDriverPid`/
// `reapRegisteredDrivers`/`reapRegisteredDriversForCid`) exists because
// `onComplete` cannot hold either of those handles at all: per
// `@wdio/runner`, `onPrepare`/`onComplete` run in the launcher process, while
// `beforeSession`/`afterSession` run inside each worker — a separate OS
// process with its own memory. The only channel between them is the
// filesystem, so `beforeSession` records the pid it just spawned and
// `afterSession` erases that record the moment it reaps the child cleanly.
//
// Entries are scoped by wdio's `cid` (`"0-3"`), not just pid, because the
// registry is one shared directory across every *concurrent* worker.
// `reapRegisteredDrivers` sweeps the whole thing and is safe only from
// `onComplete`, which fires once, after every worker has already exited.
// `onWorkerEnd` fires per worker, while *other* workers can still be
// mid-session — sweeping the whole registry there was tried and caught live:
// the first spec to finish killed every other concurrently-running spec's
// driver out from under it (a cascade of `ECONNREFUSED`).
// `reapRegisteredDriversForCid` scopes the sweep to the one worker that just
// ended, which is what makes calling it from `onWorkerEnd` safe.

import fs from 'node:fs';
import path from 'node:path';

/**
 * Terminates one driver process tree.
 *
 * Tries the process group first (`-pid`), which is what reaches a
 * WebKitWebDriver child tauri-driver spawned without cleaning up itself; on
 * `ESRCH` the group (and everything in it) is already gone, so there is
 * nothing left to do. Any other failure (e.g. this process was not a group
 * leader) falls back to signalling the pid directly.
 *
 * @param {{ pid?: number | null } | null | undefined} child
 * @param {{ signal?: NodeJS.Signals, killFn?: (pid: number, signal: NodeJS.Signals) => void }} [options]
 */
export function killDriverTree(child, options = {}) {
  const { signal = 'SIGTERM', killFn = process.kill } = options;
  if (!child || child.pid == null) return;

  if (signalOrAlreadyGone(killFn, -child.pid, signal)) return;
  signalOrAlreadyGone(killFn, child.pid, signal);
}

/**
 * @param {(pid: number, signal: NodeJS.Signals) => void} killFn
 * @param {number} pid
 * @param {NodeJS.Signals} signal
 * @returns {boolean} true when nothing further needs trying — either the
 *   signal was delivered, or the target is already gone
 */
function signalOrAlreadyGone(killFn, pid, signal) {
  try {
    killFn(pid, signal);
    return true;
  } catch (err) {
    return err?.code === 'ESRCH';
  }
}

/**
 * Terminates exactly the tracked handles — never a handle not in this list.
 * Skips `null`/`undefined` entries so a caller can pass a list that may hold
 * a not-yet-spawned or already-cleared slot without filtering it first.
 *
 * @param {Array<{ pid?: number | null } | null | undefined>} children
 * @param {{ signal?: NodeJS.Signals, killFn?: (pid: number, signal: NodeJS.Signals) => void }} [options]
 */
export function reapDriverTrees(children, options = {}) {
  for (const child of children) {
    killDriverTree(child, options);
  }
}

/**
 * The repo-local, gitignored directory the pid registry for one e2e run lives
 * in. Never the system temp dir — see CLAUDE.md/CLAUDE.local.md on scratch
 * artifacts.
 *
 * @param {string} repoRoot
 * @returns {string}
 */
export function pidRegistryDir(repoRoot) {
  return path.resolve(repoRoot, 'tmp/e2e-driver-pids');
}

/**
 * The registry filename for one entry: the pid trails after the last hyphen,
 * so `parseRegistryFileName` can split it back out even though `cid` itself
 * contains a hyphen (wdio's own format, `"0-3"`).
 *
 * @param {string} cid
 * @param {number} pid
 * @returns {string}
 */
function registryFileName(cid, pid) {
  return `${cid}-${pid}`;
}

/**
 * The inverse of `registryFileName`. Returns `null` for anything that is not
 * one of this registry's own entries, so a sweep can safely ignore stray
 * files without guessing.
 *
 * @param {string} name
 * @returns {{ cid: string, pid: number } | null}
 */
function parseRegistryFileName(name) {
  const lastDash = name.lastIndexOf('-');
  if (lastDash === -1) return null;
  const pid = Number(name.slice(lastDash + 1));
  if (!Number.isInteger(pid)) return null;
  return { cid: name.slice(0, lastDash), pid };
}

/**
 * Records that this run spawned `pid` for worker `cid`, so a sweep can still
 * find and terminate it later even if that worker crashes before its own
 * `afterSession` runs. Scoped by `cid` (not just `pid`) because the registry
 * directory is shared across every concurrently-running worker.
 *
 * @param {string} dir
 * @param {string} cid
 * @param {number} pid
 */
export function registerDriverPid(dir, cid, pid) {
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.resolve(dir, registryFileName(cid, pid)), '');
}

/**
 * Clears `pid`'s registry entry for worker `cid`. Call this once the driver
 * has actually been reaped, so a later sweep does not try to kill an
 * already-dead pid — or, worse, a different process the OS has since reused
 * that same pid for.
 *
 * @param {string} dir
 * @param {string} cid
 * @param {number} pid
 */
export function unregisterDriverPid(dir, cid, pid) {
  fs.rmSync(path.resolve(dir, registryFileName(cid, pid)), { force: true });
}

/**
 * The `onComplete` safety net: terminates every pid still listed in the
 * registry, across every worker, and clears their entries. Safe to call only
 * once every worker has already exited (i.e. from `onComplete`, never from
 * `onWorkerEnd`) — see `reapRegisteredDriversForCid` for why a mid-run sweep
 * must be scoped instead.
 *
 * @param {string} dir
 * @param {{ signal?: NodeJS.Signals, killFn?: (pid: number, signal: NodeJS.Signals) => void }} [options]
 * @returns {number[]} the pids reaped
 */
export function reapRegisteredDrivers(dir, options = {}) {
  let entries;
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return [];
  }

  const reaped = [];
  for (const entry of entries) {
    const parsed = parseRegistryFileName(entry);
    if (!parsed) continue;
    killDriverTree({ pid: parsed.pid }, options);
    unregisterDriverPid(dir, parsed.cid, parsed.pid);
    reaped.push(parsed.pid);
  }
  return reaped;
}

/**
 * The `onWorkerEnd` safety net: terminates only the pid(s) registered under
 * `cid` — the worker that just exited — and clears just those entries.
 * Deliberately narrower than `reapRegisteredDrivers`: `onWorkerEnd` fires
 * once per worker while *other* workers can still be legitimately mid-session
 * with their own drivers in this same shared registry, so sweeping
 * everything here would kill a sibling's still-running driver out from under
 * it (observed live: a cascade of `ECONNREFUSED` across every other spec the
 * instant the first one finished).
 *
 * @param {string} dir
 * @param {string} cid
 * @param {{ signal?: NodeJS.Signals, killFn?: (pid: number, signal: NodeJS.Signals) => void }} [options]
 * @returns {number[]} the pids reaped
 */
export function reapRegisteredDriversForCid(dir, cid, options = {}) {
  let entries;
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return [];
  }

  const reaped = [];
  for (const entry of entries) {
    const parsed = parseRegistryFileName(entry);
    if (!parsed || parsed.cid !== cid) continue;
    killDriverTree({ pid: parsed.pid }, options);
    unregisterDriverPid(dir, parsed.cid, parsed.pid);
    reaped.push(parsed.pid);
  }
  return reaped;
}
