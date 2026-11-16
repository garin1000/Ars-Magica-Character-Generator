import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

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
    saga_year: 1220,
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

// manual-testing-findings-2026-11-15 #9 (F9), phase 2 fold-in: Norbert's "true for
// other fields too" names the apparent-age and Aging-Points fields on this very
// panel. Same mechanism as `AgeFields.client.test.ts`/`AgingRollCalculator.client.test.ts`:
// each `oninput` forwards an out-of-range value on every keystroke, and the plain
// `value={...}` binding snaps the field back to the clamped number before typing
// finishes. Table-driven over the two fields, which share the same pattern and only
// differ in bound and store path.
function installAgingRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      aging: {
        start_age: 35,
        age_divisor: 10,
        apparent_age_increase_min: 3,
        living_conditions: [],
        outcomes: [],
        max_age: 500,
      },
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function typeInto(input: HTMLInputElement, text: string): void {
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

/** Commit the field, as leaving it (or Enter) does. */
function commitField(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

describe('AgingRecordPanel number fields stay within range while typing (F9)', () => {
  const cases: {
    name: string;
    testid: string;
    setup: () => void;
    read: () => number | undefined;
    typed: string;
    blockedAt: number;
    committed: number;
  }[] = [
    {
      name: 'apparent age',
      testid: 'apparent-age-input',
      setup: () => {
        installAgingRuleset();
        store.entity.apparent_age = 100;
      },
      read: () => store.entity.apparent_age ?? undefined,
      typed: '600',
      blockedAt: 100,
      committed: 500,
    },
    {
      name: 'Aging Points (sta)',
      testid: 'aging-points-sta',
      setup: () => {
        store.entity.aging_points = { sta: 10 };
      },
      read: () => store.entity.aging_points?.sta,
      typed: '300',
      blockedAt: 10,
      committed: 255,
    },
  ];

  it.each(cases)(
    '$name keeps an out-of-range value out of the store, and clamps only on commit',
    ({ setup, testid, read, typed, blockedAt, committed }) => {
      setup();
      target = document.createElement('div');
      document.body.appendChild(target);
      app = mount(AgingRecordPanel, { target });
      flushSync();
      const input = target.querySelector<HTMLInputElement>(`[data-testid="${testid}"]`);
      expect(input, `no field ${testid}`).toBeTruthy();

      typeInto(input!, typed);
      expect(input!.value, 'typing is never fought').toBe(typed);
      expect(read(), 'no write past the bound until the edit is committed').toBe(blockedAt);

      commitField(input!);
      expect(read()).toBe(committed);
      expect(input!.value).toBe(String(committed));
    },
  );
});
