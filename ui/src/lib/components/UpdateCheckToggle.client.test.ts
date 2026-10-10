import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// `client`: the Settings checkbox (M6.6, decision 3). The live `checked` state and
// the order of the write and the check behind a click need a mounted instance.
vi.mock('../ipc', () => ({
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  updateCheckStatus: vi.fn(),
  answerUpdatePrompt: vi.fn(),
  checkForUpdate: vi.fn(),
  openReleasePage: vi.fn(),
  writeSettings: vi.fn(),
}));

import * as ipc from '../ipc';
import { store } from '../state.svelte';
import UpdateCheckToggle from './UpdateCheckToggle.svelte';

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

function resetUpdateCheck(): void {
  const check = store.updateCheck;
  check.promptOpen = false;
  check.available = null;
  check.dismissed = false;
  check.enabled = false;
  check.checkedThisSession = false;
}

async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

function toggle(): HTMLInputElement {
  const element = target.querySelector<HTMLInputElement>('[data-testid="update-check-toggle"]');
  expect(element).toBeTruthy();
  return element!;
}

beforeEach(() => {
  store.lang = 'en';
  store.error = null;
  resetUpdateCheck();
  vi.mocked(ipc.writeSettings).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.checkForUpdate).mockReset().mockResolvedValue(null);

  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(UpdateCheckToggle, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  resetUpdateCheck();
  store.error = null;
});

describe('the update-check toggle', () => {
  it('shows the stored consent', () => {
    expect(toggle().checked).toBe(false);

    store.updateCheck.enabled = true;
    flushSync();

    expect(toggle().checked).toBe(true);
  });

  it('writes the consent before it checks', async () => {
    let finishWrite: () => void = () => {};
    vi.mocked(ipc.writeSettings).mockReturnValue(
      new Promise<void>((resolve) => {
        finishWrite = resolve;
      }),
    );

    toggle().click();
    await settle();
    expect(ipc.writeSettings).toHaveBeenCalledWith({ update_check: true });
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();

    finishWrite();
    await settle();

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(toggle().checked).toBe(true);
  });

  it('writes a refusal when turned off, and checks nothing', async () => {
    store.updateCheck.enabled = true;
    flushSync();

    toggle().click();
    await settle();

    expect(ipc.writeSettings).toHaveBeenCalledWith({ update_check: false });
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
    expect(toggle().checked).toBe(false);
  });

  it('reports a failed write and shows the state that stuck', async () => {
    vi.mocked(ipc.writeSettings).mockRejectedValue({ kind: 'io' });

    toggle().click();
    await settle();

    expect(store.error).toEqual({ kind: 'io' });
    expect(toggle().checked).toBe(false);
  });

  it('leaves the document clean', async () => {
    toggle().click();
    await settle();
    toggle().click();
    await settle();

    expect(ipc.writeSettings).toHaveBeenCalledTimes(2);
    expect(store.dirty).toBe(false);
  });
});
