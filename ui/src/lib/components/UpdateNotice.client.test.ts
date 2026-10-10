import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// `client`: the notice appearing once a real check answers, and its two buttons,
// need a live instance.
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
import UpdateNotice from './UpdateNotice.svelte';

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

function notice(): HTMLElement | null {
  return target.querySelector<HTMLElement>('[data-testid="update-notice"]');
}

function button(testid: string): HTMLButtonElement {
  const element = target.querySelector<HTMLButtonElement>(`[data-testid="${testid}"]`);
  expect(element, `a button with data-testid="${testid}"`).toBeTruthy();
  return element!;
}

beforeEach(() => {
  store.lang = 'en';
  store.error = null;
  resetUpdateCheck();
  vi.mocked(ipc.updateCheckStatus)
    .mockReset()
    .mockResolvedValue({ prompt_due: false, enabled: true });
  vi.mocked(ipc.checkForUpdate).mockReset().mockResolvedValue('99.0.0');
  vi.mocked(ipc.openReleasePage).mockReset().mockResolvedValue(undefined);

  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(UpdateNotice, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  resetUpdateCheck();
  store.error = null;
});

describe('the update notice', () => {
  it('appears once the check finds a newer release', async () => {
    expect(notice()).toBeNull();

    await store.updateCheck.start();
    await settle();

    expect(notice()).not.toBeNull();
    expect(notice()!.textContent).toContain('99.0.0');
  });

  it('hides for the session when dismissed', async () => {
    await store.updateCheck.start();
    await settle();
    expect(notice()).not.toBeNull();

    button('update-notice-dismiss').click();
    flushSync();

    expect(notice()).toBeNull();
  });

  it('opens the release page from its button', async () => {
    await store.updateCheck.start();
    await settle();

    button('update-notice-open').click();
    await settle();

    expect(ipc.openReleasePage).toHaveBeenCalledTimes(1);
  });

  it('stays away, silently, when the check fails', async () => {
    vi.mocked(ipc.checkForUpdate).mockRejectedValue({ kind: 'io' });

    await store.updateCheck.start();
    await settle();

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(notice()).toBeNull();
    expect(store.error).toBeNull();
  });

  it('stays away, silently, when the status cannot be read', async () => {
    vi.mocked(ipc.updateCheckStatus).mockRejectedValue({ kind: 'io' });

    await store.updateCheck.start();
    await settle();

    expect(ipc.updateCheckStatus).toHaveBeenCalledTimes(1);
    expect(notice()).toBeNull();
    expect(store.error).toBeNull();
  });
});
