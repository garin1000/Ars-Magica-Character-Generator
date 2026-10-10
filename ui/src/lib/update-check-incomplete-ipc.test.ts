import { describe, expect, it, vi } from 'vitest';

import type { LocalizedRuleset } from './types';

// `ssr`. About eighty test files mock `./ipc` with an object-literal factory that
// predates M6.6 and so lacks the update commands; vitest throws as soon as a
// missing export is read. `AppStore.init()` starts the update check on every
// mount, so that throw must be swallowed, never surfaced: the factories stay as
// they are, and this file is the one that proves they may.
vi.mock('./ipc', () => ({
  readSettings: vi.fn().mockResolvedValue({
    default_saga_year: 1220,
    lang: null,
    theme: null,
    update_check: null,
    update_check_asked_version: null,
    validation_mode: null,
  }),
  writeSettings: vi.fn().mockResolvedValue(undefined),
  loadRuleset: vi.fn().mockResolvedValue({
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
  } as unknown as LocalizedRuleset),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
}));

import * as ipc from './ipc';
import { store } from './state.svelte';
import { UpdateCheck } from './update-check.svelte';

/** Let every queued microtask run. */
async function settle(): Promise<void> {
  for (let i = 0; i < 10; i += 1) await Promise.resolve();
}

describe('a bridge without the update commands', () => {
  it('starts without a throw and without an error', async () => {
    const setError = vi.fn();
    const check = new UpdateCheck({ setError });

    await expect(check.start()).resolves.toBeUndefined();

    expect(setError).not.toHaveBeenCalled();
    expect(check.promptOpen).toBe(false);
  });

  it('leaves the launch itself untouched', async () => {
    store.error = null;

    await store.init();
    await settle();

    expect(vi.mocked(ipc.loadRuleset)).toHaveBeenCalled();
    expect(store.error).toBeNull();
    expect(store.updateCheck.promptOpen).toBe(false);
  });
});
