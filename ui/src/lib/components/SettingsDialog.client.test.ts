import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// A `client` test, not `ssr`, and it could not be anything else. A focus trap is
// made of three things SSR has none of: a live `document.activeElement`, keydown
// listeners that actually fire, and `$effect` bodies that actually run. An SSR test
// of "Tab wraps to the first control" would assert against a string, find the string
// unchanged, and report green having exercised nothing at all.
//
// Accessibility defects are rated HIGH in this project, and this is the app's first
// modal with a real trap: `DiscardPrompt` gets containment only by accident, from
// `App.svelte`'s `inert` on the shell, and `ConfirmPrompt` has none whatsoever.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn().mockResolvedValue(null),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  readSettings: vi.fn().mockResolvedValue({
    lang: null,
    saga_year: 1220,
    theme: null,
    validation_mode: null,
  }),
  writeSettings: vi.fn().mockResolvedValue(undefined),
  deriveAge: vi.fn().mockResolvedValue({ age: null, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(null),
}));

import * as ipc from '../ipc';
import { store } from '../state.svelte';
import SettingsDialog from './SettingsDialog.svelte';

let target: HTMLElement;
let app: Record<string, unknown> | undefined;
let invoker: HTMLButtonElement;

beforeEach(() => {
  store.lang = 'en';
  store.theme = 'auto';
  store.mode = 'enforced';
  store.settingsOpen = false;
  vi.mocked(ipc.writeSettings).mockClear();

  // The element the dialog was opened FROM — a stand-in for the header button or
  // whatever the keyboard happened to be on when the menu item was chosen.
  invoker = document.createElement('button');
  invoker.textContent = 'open settings';
  document.body.appendChild(invoker);

  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(SettingsDialog, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  invoker?.remove();
  store.settingsOpen = false;
});

/** Open the dialog from `invoker`, the way a real click would. */
function openFromInvoker(): void {
  invoker.focus();
  store.openSettings();
  flushSync();
}

function dialog(): HTMLElement {
  const element = target.querySelector<HTMLElement>('[data-testid="settings-dialog"]');
  expect(element).toBeTruthy();
  return element!;
}

/** Every control the trap must cycle through, in DOM order. */
function focusables(): HTMLElement[] {
  return [...dialog().querySelectorAll<HTMLElement>('select, button')];
}

function press(key: string, shiftKey = false): void {
  document.activeElement?.dispatchEvent(
    new KeyboardEvent('keydown', { key, shiftKey, bubbles: true, cancelable: true }),
  );
  flushSync();
}

describe('the settings dialog focus trap', () => {
  it('moves focus into the dialog when it opens', () => {
    openFromInvoker();

    expect(dialog().contains(document.activeElement)).toBe(true);
  });

  it('wraps Tab from the last control back to the first', () => {
    openFromInvoker();
    const controls = focusables();
    expect(controls.length).toBeGreaterThan(1);

    controls[controls.length - 1].focus();
    press('Tab');

    expect(document.activeElement).toBe(controls[0]);
  });

  it('wraps Shift+Tab from the first control back to the last', () => {
    openFromInvoker();
    const controls = focusables();

    controls[0].focus();
    press('Tab', true);

    expect(document.activeElement).toBe(controls[controls.length - 1]);
  });

  it('leaves a Tab in the middle of the dialog to the browser', () => {
    // The trap only closes the loop at its two ends. Calling `preventDefault` on
    // every Tab would mean reimplementing the browser's own focus order, which is
    // how a trap starts skipping controls.
    openFromInvoker();
    const controls = focusables();
    controls[0].focus();

    const event = new KeyboardEvent('keydown', { key: 'Tab', bubbles: true, cancelable: true });
    controls[0].dispatchEvent(event);
    flushSync();

    expect(event.defaultPrevented).toBe(false);
  });

  it('closes on Escape', () => {
    openFromInvoker();

    press('Escape');

    expect(store.settingsOpen).toBe(false);
  });

  it('returns focus to whatever opened it', () => {
    openFromInvoker();
    expect(document.activeElement).not.toBe(invoker);

    store.closeSettings();
    flushSync();

    expect(document.activeElement).toBe(invoker);
  });

  it('renders nothing at all while closed', () => {
    expect(target.querySelector('[data-testid="settings-dialog"]')).toBeNull();
  });
});

describe('the settings dialog controls', () => {
  /** Choose `value` in the dialog's `testid` select, as a user would. */
  function choose(testid: string, value: string): void {
    const select = dialog().querySelector<HTMLSelectElement>(`[data-testid="${testid}"]`);
    expect(select).toBeTruthy();
    select!.value = value;
    select!.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
  }

  it('applies and persists a chosen theme', () => {
    openFromInvoker();

    choose('theme-select', 'light');

    expect(store.theme).toBe('light');
    expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ theme: 'light' });
  });

  it('applies and persists a chosen validation mode', () => {
    openFromInvoker();

    choose('mode-select', 'advisory');

    expect(store.mode).toBe('advisory');
    expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ validation_mode: 'advisory' });
  });

  it('applies and persists a chosen language', () => {
    openFromInvoker();

    choose('language-select', 'de');

    expect(store.lang).toBe('de');
    expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ lang: 'de' });
  });

  it('closes from its own Close button', () => {
    openFromInvoker();

    dialog().querySelector<HTMLButtonElement>('[data-testid="settings-close"]')!.click();
    flushSync();

    expect(store.settingsOpen).toBe(false);
  });
});
