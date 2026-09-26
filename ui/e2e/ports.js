// Port preflight for the e2e suite.
//
// The harness leaks its driver processes on some runs (docs/open-todos.md):
// `tauri-driver` and the `WebKitWebDriver` it spawns can outlive an exiting
// run and keep holding their ports. A later run does not start its own
// driver on an occupied port — it connects to the stale one, and
// `POST /session` then hangs for the full ~120s session-creation timeout
// before failing with a `WebDriverError: timeout` that reads like
// infrastructure flakiness rather than what it is.
//
// This preflight converts that into an immediate, accurate refusal: checked
// in `onPrepare`, before either wdio config spawns its own tauri-driver.

import net from 'node:net';

const DEFAULT_TIMEOUT_MS = 300;

/**
 * Whether something is already listening on `port` on localhost. Resolves
 * quickly either way — an immediate `connect` when something is listening, an
 * immediate refusal when nothing is — so this can never itself contribute to
 * a long hang; `timeoutMs` only guards the rare case of a port that neither
 * accepts nor refuses (e.g. filtered).
 *
 * @param {number} port
 * @param {{ connect?: (port: number, timeoutMs: number) => Promise<boolean>, timeoutMs?: number }} [options]
 * @returns {Promise<boolean>}
 */
export function isPortOccupied(port, options = {}) {
  const { connect = tcpProbe, timeoutMs = DEFAULT_TIMEOUT_MS } = options;
  return connect(port, timeoutMs);
}

/**
 * One TCP connect attempt against localhost, resolved as a plain boolean —
 * never rejects, so the caller never needs a catch.
 *
 * @param {number} port
 * @param {number} timeoutMs
 * @returns {Promise<boolean>}
 */
function tcpProbe(port, timeoutMs) {
  return new Promise((resolve) => {
    const socket = net.connect({ host: '127.0.0.1', port });
    const settle = (result) => {
      clearTimeout(timer);
      socket.removeAllListeners();
      socket.destroy();
      resolve(result);
    };
    const timer = setTimeout(() => settle(false), timeoutMs);
    socket.once('connect', () => settle(true));
    socket.once('error', () => settle(false));
  });
}

/**
 * Preflight refusal for the ports this run is about to start its own
 * tauri-driver(s) on. Checks each port in order and stops at the first one
 * already bound.
 *
 * @param {number[]} ports
 * @param {{ connect?: (port: number, timeoutMs: number) => Promise<boolean>, timeoutMs?: number }} [options]
 * @returns {Promise<string | null>} an explanatory message naming the port, or
 *   `null` when every port is free and the run may proceed
 */
export async function preflightDriverPorts(ports, options = {}) {
  for (const port of ports) {
    if (await isPortOccupied(port, options)) {
      return (
        `e2e preflight: port ${port} is already bound, so this run cannot start its own ` +
        `tauri-driver there. A stale tauri-driver/WebKitWebDriver left over from a previous ` +
        `e2e run is the usual cause — find it (\`ps aux | grep -E "tauri-driver|WebKitWebDriver"\`) ` +
        `and stop those pids, then re-run the suite.`
      );
    }
  }
  return null;
}
