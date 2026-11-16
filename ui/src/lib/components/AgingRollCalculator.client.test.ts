import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { AgingOutcome, AgingTotal } from '../ipc';
import type { AgingDraft } from '../state.svelte';
import type {
  AgingReadout,
  AgingScheduleYear,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
} from '../types';

// manual-testing-findings-2026-11-15 #9 (F9): "Gain sufficient Aging Points (in any
// Characteristics)" (ArMDE:16681) lets the player SPREAD the owed points, so the cap
// on any one field shrinks as the others fill up (manual-testing-findings #22). That
// cap is often tiny — one or two points — so a keystroke crossing it is the ordinary
// case, not an edge case: typing "2" into a field capped at 1 used to write 1 to the
// draft immediately (`setDistribution`'s own clamp) and the plain `value={...}`
// binding snapped the field back to "1" before the player had finished. The
// birth-year pattern (`IdentityFields.svelte::onBirthYearInput`) fixes it: `oninput`
// forwards only a value within [0, the field's own live cap], and the full typed text
// is clamped on commit.
//
// A `client` test: the `input`/`change` listeners on a live element and the text the
// live field shows between them, which SSR never dispatches.
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
  agingPreview: vi.fn().mockResolvedValue({ status: 'rejected', issues: [] }),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import AgingRollCalculator from './AgingRollCalculator.svelte';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      aging: {
        start_age: 35,
        age_divisor: 10,
        apparent_age_increase_min: 3,
        living_conditions: [],
        outcomes: [],
      },
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

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
  const schedule: AgingScheduleYear[] = [{ age: 40, year: null, recorded: false }];
  const aging: AgingReadout = {
    first_roll_age: 36,
    begins_after_age: 35,
    schedule,
    rolls_owed: 1,
    rolls_recorded: 0,
    age_modifier: 4,
    living_conditions_modifier: 0,
    longevity_modifier: 0,
    trait_modifier: 0,
    longevity_clamp_active: false,
    fixed_total: 4,
  };
  store.effective = { aging } as unknown as EffectiveScores;
}

function draft(over: Partial<AgingDraft> = {}): AgingDraft {
  return { age: 40, die: 10, distribution: {}, crisisDie: null, ...over };
}

function total(overrides: Partial<AgingTotal> = {}): AgingTotal {
  return {
    age: 40,
    die: 10,
    age_modifier: 4,
    living_conditions: { rows: [], from_table: 0, from_traits: 0, total: 0 },
    longevity_bonus: 0,
    trait_modifier: 0,
    uncapped_total: 14,
    total: 14,
    capped_by_longevity: false,
    ...overrides,
  };
}

function outcome(overrides: Partial<AgingOutcome> = {}): AgingOutcome {
  return {
    total: 14,
    apparent_age_increases: true,
    awards: [],
    crisis: false,
    ...overrides,
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function mountCalculator(): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(AgingRollCalculator, { target });
  flushSync();
}

function field(characteristic: string): HTMLInputElement {
  const input = target.querySelector<HTMLInputElement>(
    `[data-testid="aging-distribute-${characteristic}"]`,
  );
  expect(input, `no distribution field for ${characteristic}`).toBeTruthy();
  return input!;
}

function typeInto(input: HTMLInputElement, text: string): void {
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

describe('AgingRollCalculator distribution inputs stay within their live cap while typing (F9)', () => {
  it('keeps an over-cap value out of the draft, and clamps only on commit', () => {
    // 5 owed; `qik` and `sta` already hold 2 each, so `per` (nothing placed yet) is
    // capped at what both leave: 5 - 2 - 2 = 1 (manual-testing-findings #22).
    store.agingDraft = draft({ die: 9, distribution: { qik: 2, sta: 2 } });
    store.agingPreview = {
      total: total({ die: 9, total: 13 }),
      outcome: outcome({
        total: 13,
        awards: [{ target: { kind: 'next_decrepitude_level' }, points: 5 }],
        crisis: true,
      }),
      crisis: null,
    };
    mountCalculator();
    const input = field('per');
    expect(input.max).toBe('1');

    typeInto(input, '2');
    expect(input.value, 'typing is never fought').toBe('2');
    expect(
      store.agingDraft.distribution.per,
      'no write past the live cap until the edit is committed',
    ).toBeUndefined();

    commit(input);
    expect(store.agingDraft.distribution.per).toBe(1);
    expect(input.value).toBe('1');
  });
});
