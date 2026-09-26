import net from 'node:net';

import { describe, expect, it } from 'vitest';

import { isPortOccupied, preflightDriverPorts } from './ports.js';

// The e2e harness leaks its driver processes (docs/open-todos.md), and the
// symptom is maximally misleading: a later run does not start its own
// tauri-driver on an occupied port, it connects to the stale one and hangs
// for the full 120s session-creation timeout before failing with
// `WebDriverError: timeout`. This preflight turns that into an immediate,
// accurate refusal naming the port, checked in `onPrepare` before either wdio
// config spawns its own driver.

/** Opens a real listener on an OS-assigned free port, for a real bind to probe. */
function listen() {
  return new Promise((resolve) => {
    const server = net.createServer();
    server.listen(0, '127.0.0.1', () => resolve(server));
  });
}

function close(server) {
  return new Promise((resolve) => server.close(resolve));
}

describe('isPortOccupied', () => {
  it('reports true for a port a real listener is bound to', async () => {
    const server = await listen();
    try {
      const { port } = server.address();
      await expect(isPortOccupied(port)).resolves.toBe(true);
    } finally {
      await close(server);
    }
  });

  it('reports false for a port nothing is listening on', async () => {
    const server = await listen();
    const { port } = server.address();
    await close(server);
    await expect(isPortOccupied(port)).resolves.toBe(false);
  });

  it('resolves quickly rather than hanging for a long timeout', async () => {
    const server = await listen();
    const { port } = server.address();
    await close(server);

    const started = Date.now();
    await isPortOccupied(port);
    expect(Date.now() - started).toBeLessThan(2000);
  });
});

describe('preflightDriverPorts', () => {
  it('refuses with a message naming the occupied port and the stale-driver cause', async () => {
    const server = await listen();
    try {
      const { port } = server.address();
      const message = await preflightDriverPorts([port]);
      expect(message).not.toBeNull();
      expect(message).toContain(String(port));
      expect(message).toMatch(/tauri-driver/);
      expect(message).toMatch(/stale/i);
    } finally {
      await close(server);
    }
  });

  it('passes (returns null) when every port is free', async () => {
    const server = await listen();
    const { port } = server.address();
    await close(server);

    await expect(preflightDriverPorts([port])).resolves.toBeNull();
  });

  it('checks every listed port, not just the first', async () => {
    const freeServer = await listen();
    const freePort = freeServer.address().port;
    await close(freeServer);

    const occupiedServer = await listen();
    try {
      const occupiedPort = occupiedServer.address().port;
      const message = await preflightDriverPorts([freePort, occupiedPort]);
      expect(message).toContain(String(occupiedPort));
    } finally {
      await close(occupiedServer);
    }
  });
});
