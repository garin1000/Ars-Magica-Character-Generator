import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// `client`: a focus trap, focus return, Escape and the order of two IPC calls
// behind a click all need a live instance (CLAUDE.md, "Frontend test
// environments").
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
import UpdatePrompt from './UpdatePrompt.svelte';

let target: HTMLElement;
let app: Record<string, unknown> | undefined;
let invoker: HTMLButtonElement;

function resetUpdateCheck(): void {
  const check = store.updateCheck;
  check.promptOpen = false;
  check.available = null;
  check.dismissed = false;
  check.enabled = false;
  check.checkedThisSession = false;
}

/** Let queued promise callbacks and timers at zero run. */
async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync();
}

beforeEach(() => {
  store.lang = 'en';
  store.error = null;
  resetUpdateCheck();
  vi.mocked(ipc.answerUpdatePrompt).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.checkForUpdate).mockReset().mockResolvedValue(null);

  invoker = document.createElement('button');
  invoker.textContent = 'whatever had focus at launch';
  document.body.appendChild(invoker);

  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(UpdatePrompt, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  invoker?.remove();
  resetUpdateCheck();
  store.error = null;
});

function openFromInvoker(): void {
  invoker.focus();
  store.updateCheck.promptOpen = true;
  flushSync();
}

function prompt(): HTMLElement {
  const element = target.querySelector<HTMLElement>('[data-testid="update-prompt"]');
  expect(element).toBeTruthy();
  return element!;
}

function button(testid: string): HTMLButtonElement {
  const element = target.querySelector<HTMLButtonElement>(`[data-testid="${testid}"]`);
  expect(element).toBeTruthy();
  return element!;
}

function press(key: string, shiftKey = false): void {
  document.activeElement?.dispatchEvent(
    new KeyboardEvent('keydown', { key, shiftKey, bubbles: true, cancelable: true }),
  );
  flushSync();
}

describe('the update prompt focus', () => {
  // "Don't check" is the safe default: a stray Enter must not send anything.
  it('moves focus onto the decline answer when it opens', () => {
    openFromInvoker();

    expect(document.activeElement).toBe(button('update-prompt-decline'));
  });

  it('wraps Tab from the last control back to the first', () => {
    openFromInvoker();
    const controls = [...prompt().querySelectorAll<HTMLElement>('button')];
    expect(controls.length).toBe(2);

    controls[1].focus();
    press('Tab');

    expect(document.activeElement).toBe(controls[0]);
  });

  it('wraps Shift+Tab from the first control back to the last', () => {
    openFromInvoker();
    const controls = [...prompt().querySelectorAll<HTMLElement>('button')];

    controls[0].focus();
    press('Tab', true);

    expect(document.activeElement).toBe(controls[1]);
  });

  it('returns focus to whatever had it once answered', async () => {
    openFromInvoker();

    button('update-prompt-decline').click();
    await settle();

    expect(store.updateCheck.promptOpen).toBe(false);
    expect(document.activeElement).toBe(invoker);
  });
});

describe('answering the update prompt', () => {
  it('declines on Escape, recording the answer', async () => {
    openFromInvoker();

    press('Escape');
    await settle();

    expect(ipc.answerUpdatePrompt).toHaveBeenCalledWith(false);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
    expect(store.updateCheck.promptOpen).toBe(false);
  });

  it('records a no from its button and checks nothing', async () => {
    openFromInvoker();

    button('update-prompt-decline').click();
    await settle();

    expect(ipc.answerUpdatePrompt).toHaveBeenCalledWith(false);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
  });

  it('writes a yes before it checks', async () => {
    openFromInvoker();
    let finishWrite: () => void = () => {};
    vi.mocked(ipc.answerUpdatePrompt).mockReturnValue(
      new Promise<void>((resolve) => {
        finishWrite = resolve;
      }),
    );

    button('update-prompt-enable').click();
    await settle();
    expect(ipc.answerUpdatePrompt).toHaveBeenCalledWith(true);
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();

    finishWrite();
    await settle();

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
  });

  it('reports a failed write on the error banner', async () => {
    openFromInvoker();
    vi.mocked(ipc.answerUpdatePrompt).mockRejectedValue({ kind: 'io' });

    button('update-prompt-enable').click();
    await settle();

    expect(store.error).toEqual({ kind: 'io' });
    expect(ipc.checkForUpdate).not.toHaveBeenCalled();
  });

  it('leaves the document clean', async () => {
    const dirtyBefore = store.dirty;
    openFromInvoker();

    button('update-prompt-enable').click();
    await settle();

    expect(ipc.answerUpdatePrompt).toHaveBeenCalledTimes(1);
    expect(store.dirty).toBe(dirtyBefore);
    expect(store.dirty).toBe(false);
  });
});
