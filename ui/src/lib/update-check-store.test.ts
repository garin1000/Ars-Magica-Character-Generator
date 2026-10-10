import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { PersistedSettings } from './ipc';
import type { AppError, LocalizedRuleset } from './types';

// `ssr`: how `AppStore` hosts the update check (M6.6) — the launch order, the
// menu gate while the prompt is up, the error banner for failed writes, and the
// unsaved-changes guard. Plain store state, no component.
vi.mock('./ipc', () => ({
  readSettings: vi.fn(),
  writeSettings: vi.fn(),
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  updateCheckStatus: vi.fn(),
  answerUpdatePrompt: vi.fn(),
  checkForUpdate: vi.fn(),
  openReleasePage: vi.fn(),
}));

import * as ipc from './ipc';
import { store } from './state.svelte';

function persisted(overrides: Partial<PersistedSettings> = {}): PersistedSettings {
  return {
    default_saga_year: 1220,
    lang: null,
    theme: null,
    update_check: null,
    update_check_asked_version: null,
    validation_mode: null,
    ...overrides,
  };
}

const localized = {
  ruleset: {
    id: 'test',
    version: '1',
    point_items: {},
    type_profiles: {},
    abilities: {},
    magnitude_points: { free: 0, minor: 1, major: 3 },
    ability_category_order: ['general'],
    art_type_order: ['technique', 'form'],
  },
  i18n: {},
} as unknown as LocalizedRuleset;

const ioError = { kind: 'io' } as unknown as AppError;

/** Let every queued microtask run. */
async function settle(): Promise<void> {
  for (let i = 0; i < 10; i += 1) await Promise.resolve();
}

function resetUpdateCheck(): void {
  const check = store.updateCheck;
  check.promptOpen = false;
  check.available = null;
  check.dismissed = false;
  check.enabled = false;
  check.checkedThisSession = false;
}

beforeEach(() => {
  vi.mocked(ipc.readSettings).mockReset().mockResolvedValue(persisted());
  vi.mocked(ipc.writeSettings).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.loadRuleset).mockReset().mockResolvedValue(localized);
  vi.mocked(ipc.updateCheckStatus)
    .mockReset()
    .mockResolvedValue({ prompt_due: false, enabled: false });
  vi.mocked(ipc.answerUpdatePrompt).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.checkForUpdate).mockReset().mockResolvedValue(null);
  vi.mocked(ipc.openReleasePage).mockReset().mockResolvedValue(undefined);
  store.error = null;
  store.view = 'start';
  resetUpdateCheck();
});

afterEach(() => {
  resetUpdateCheck();
  store.error = null;
});

describe('the launch', () => {
  it('asks for the update status only after the settings are read', async () => {
    const order: string[] = [];
    vi.mocked(ipc.readSettings).mockImplementation(async () => {
      order.push('settings');
      return persisted();
    });
    vi.mocked(ipc.updateCheckStatus).mockImplementation(async () => {
      order.push('status');
      return { prompt_due: false, enabled: false };
    });

    await store.init();
    await settle();

    expect(order).toEqual(['settings', 'status']);
  });

  it('does not hold the ruleset load back for the update check', async () => {
    // A status that never answers: `init` must finish regardless.
    vi.mocked(ipc.updateCheckStatus).mockReturnValue(new Promise(() => {}));

    await store.init();

    expect(ipc.updateCheckStatus).toHaveBeenCalledTimes(1);
    expect(ipc.loadRuleset).toHaveBeenCalledTimes(1);
    expect(store.ruleset).toEqual(localized);
  });

  it('opens the prompt the status asks for', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: true, enabled: false });

    await store.init();
    await settle();

    expect(store.updateCheck.promptOpen).toBe(true);
  });

  it('knows the persisted consent from the settings file', async () => {
    vi.mocked(ipc.readSettings).mockResolvedValue(persisted({ update_check: true }));

    await store.loadSettings();

    expect(store.updateCheck.enabled).toBe(true);
  });
});

describe('the menu while the prompt is open', () => {
  it('withholds every action, on every screen', () => {
    store.updateCheck.promptOpen = true;

    for (const view of ['start', 'editor', 'wizard'] as const) {
      store.view = view;
      for (const action of ['new', 'open', 'save', 'saveAs', 'export', 'settings'] as const) {
        expect(store.documentActionEnabled(action), `${view}/${action}`).toBe(false);
      }
      expect(Object.values(store.menuFlags()).some(Boolean), view).toBe(false);
    }
  });

  it('gives the actions back once the prompt is answered', async () => {
    store.updateCheck.promptOpen = true;

    await store.updateCheck.answerPrompt(false);

    expect(store.documentActionEnabled('open')).toBe(true);
    expect(store.documentActionEnabled('settings')).toBe(true);
  });
});

describe('write failures', () => {
  it('reports a failed prompt answer on the error banner', async () => {
    store.updateCheck.promptOpen = true;
    vi.mocked(ipc.answerUpdatePrompt).mockRejectedValue(ioError);

    await store.updateCheck.answerPrompt(true);

    expect(store.error).toEqual(ioError);
  });

  it('reports a failed toggle write on the error banner', async () => {
    vi.mocked(ipc.writeSettings).mockRejectedValue(ioError);

    await store.updateCheck.enable();

    expect(store.error).toEqual(ioError);
  });

  it('keeps a failed check off the error banner', async () => {
    vi.mocked(ipc.updateCheckStatus).mockResolvedValue({ prompt_due: false, enabled: true });
    vi.mocked(ipc.checkForUpdate).mockRejectedValue(ioError);

    await store.updateCheck.start();

    expect(ipc.checkForUpdate).toHaveBeenCalledTimes(1);
    expect(store.error).toBeNull();
  });
});

// The unsaved-changes guard is a mandatory product behavior (CLAUDE.md). The
// consent is an app preference, never document state, so nothing about it may
// make the open character look edited.
describe('the unsaved-changes guard', () => {
  it('stays clean through every update-check answer and toggle', async () => {
    await store.init();
    await settle();
    expect(store.dirty).toBe(false);
    vi.mocked(ipc.checkForUpdate).mockResolvedValue('99.0.0');

    store.updateCheck.promptOpen = true;
    await store.updateCheck.answerPrompt(true);
    store.updateCheck.promptOpen = true;
    await store.updateCheck.answerPrompt(false);
    await store.updateCheck.enable();
    await store.updateCheck.disable();
    store.updateCheck.dismiss();

    expect(ipc.answerUpdatePrompt).toHaveBeenCalledTimes(2);
    expect(ipc.writeSettings).toHaveBeenCalledTimes(2);
    expect(store.dirty).toBe(false);
  });
});
