import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity } from '../types';

// guided-creation-review-2026-08 #26 turned the decrepitude-effect control from an
// `<input type="text">` into a `<textarea rows="3">`, which changes the handler's
// event cast from `HTMLInputElement` to `HTMLTextAreaElement`. That the cast still
// reaches `store.setDecrepitudeEffect` cannot be observed under SSR — the rendered
// markup carries the same `value` either way and no handler ever runs — so this one
// assertion needs a live component in the `client` project. Harness mirrors
// SpellTab.client.test.ts.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
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
}));

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import AgingRecordPanel from './AgingRecordPanel.svelte';

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

// manual-testing-findings-2026-09-03 #23: the × on an engine-recorded row used to be a
// plain array filter — it deleted the RECORD and left that year's Aging Points and
// its year of apparent age applied to the character, so the sheet kept effects with
// nothing left to explain them. Pressing the real button is the only way to observe
// the whole chain (handler -> store -> `aging_revert`), so it belongs in the client
// project.
describe('AgingRecordPanel aging-log undo (#23)', () => {
  /** The engine-recorded row for the year rolled at age 40. */
  const recorded = {
    year: 1220,
    age: 40,
    effect: '',
    die: 9,
    total: 13,
    points: { sta: 1 },
    apparent_age_increased: true,
  };

  it('takes the year back off the character when its × is pressed', async () => {
    store.entity.aging_points = { sta: 1 };
    store.entity.apparent_age = 41;
    store.entity.aging_log = [recorded];

    // What `aging::revert_year` hands back for age 40 — the character before it.
    vi.mocked(ipc.agingRevert).mockResolvedValue({
      status: 'reverted',
      entity: { ...store.entity, aging_points: {}, apparent_age: null, aging_log: [] },
    });

    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(AgingRecordPanel, { target });
    flushSync();

    target.querySelector<HTMLButtonElement>('[data-testid="aging-log-remove-0"]')!.click();

    await vi.waitFor(() => expect(store.entity.aging_log).toEqual([]));
    // The row is gone AND so is everything the year did — the whole point.
    expect(store.entity.aging_points).toEqual({});
    expect(store.entity.apparent_age).toBeNull();
    expect(vi.mocked(ipc.agingRevert).mock.lastCall?.[1]).toBe(40);
  });
});

describe('AgingRecordPanel decrepitude narrative (#26)', () => {
  it('writes what is typed in the decrepitude textarea to the entity', () => {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(AgingRecordPanel, { target });
    flushSync();

    const field = target.querySelector<HTMLTextAreaElement>(
      '[data-testid="decrepitude-effect-input"]',
    );
    expect(field).not.toBeNull();
    expect(field!.tagName.toLowerCase()).toBe('textarea');

    field!.value = 'Deaf in one ear, and a limp since the winter of 1223.';
    field!.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();

    expect(store.entity.decrepitude_effect).toBe(
      'Deaf in one ear, and a limp since the winter of 1223.',
    );
  });
});
