import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, Familiar, LocalizedRuleset } from '../types';

// N2 (try-out 2026-10-04): every number field bound one-way to a clamping store
// setter showed what was typed rather than what was stored whenever the clamp left
// the stored value unchanged. One representative per component family proves the
// shared `commitStored` write-back is wired there: aging, familiar, the XP and spell
// budgets, and the saga year (whose emptied field the setter ignores outright).
//
// A `client` test: `input`/`change` listeners on live elements and the text the live
// field shows afterwards. SSR dispatches no events and never runs an action.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  confirmDiscard: vi.fn().mockResolvedValue(true),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  readSettings: vi.fn(),
  writeSettings: vi.fn(),
  deriveAge: vi.fn().mockResolvedValue({ age: null, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(null),
}));

import { U32_MAX } from '../clamp';
import { SCHEMA_VERSION, store } from '../state.svelte';
import AgingRecordPanel from './AgingRecordPanel.svelte';
import FamiliarPanel from './FamiliarPanel.svelte';
import SagaYearField from './SagaYearField.svelte';
import SpellBudgetBar from './SpellBudgetBar.svelte';
import XpBar from './XpBar.svelte';

function installRuleset(): void {
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

const familiar: Familiar = {
  name: 'Corax',
  animal: 'raven',
  might: null,
  characteristics: {},
  size: 0,
  personality_traits: [],
  cord_gold: 5,
  cord_silver: 0,
  cord_bronze: 0,
  powers: [],
};

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: U32_MAX,
    ability_funding: 'pool',
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
    apparent_age: 500,
    aging_points: { sta: 255 },
    spell_levels_override: U32_MAX,
    familiar: { ...familiar },
  };
  store.effective = null;
  store.derived = null;
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

function mountField(component: Component, testid: string): HTMLInputElement {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(component, { target });
  flushSync();
  const input = target.querySelector<HTMLInputElement>(`[data-testid="${testid}"]`);
  expect(input, `no field ${testid}`).toBeTruthy();
  return input!;
}

/** Type `text` one key at a time, appending to whatever the field shows. */
function typeKeys(input: HTMLInputElement, text: string): void {
  for (const key of text) {
    input.value = input.value + key;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
  }
}

/** Empty the field, as select-all plus Delete does. */
function clearField(input: HTMLInputElement): void {
  input.value = '';
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

describe('number fields show the stored value once an edit is committed (N2)', () => {
  it('aging: the apparent age at the maximum age', () => {
    const input = mountField(AgingRecordPanel, 'apparent-age-input');
    typeKeys(input, '0');
    expect(input.value).toBe('5000');

    commit(input);
    expect(store.entity.apparent_age).toBe(500);
    expect(input.value).toBe('500');
  });

  it('aging: Aging Points at the u8 ceiling', () => {
    const input = mountField(AgingRecordPanel, 'aging-points-sta');
    typeKeys(input, '0');

    commit(input);
    expect(store.entity.aging_points?.sta).toBe(255);
    expect(input.value).toBe('255');
  });

  it('familiar: a cord at its rules maximum of +5', () => {
    const input = mountField(FamiliarPanel, 'familiar-cord-gold');
    typeKeys(input, '5');
    expect(input.value).toBe('55');

    commit(input);
    expect(store.entity.familiar?.cord_gold).toBe(5);
    expect(input.value).toBe('5');
  });

  it('XP: the experience pool at the u32 ceiling', () => {
    const input = mountField(XpBar, 'xp-pool');
    typeKeys(input, '9');

    commit(input);
    expect(store.entity.xp_pool).toBe(U32_MAX);
    expect(input.value).toBe(String(U32_MAX));
  });

  it('spell budget: the levels override at the u32 ceiling', () => {
    const input = mountField(SpellBudgetBar, 'spell-levels-base');
    typeKeys(input, '9');

    commit(input);
    expect(store.entity.spell_levels_override).toBe(U32_MAX);
    expect(input.value).toBe(String(U32_MAX));
  });

  // `setSagaYear` ignores an emptied field (the saga year is never null), so the
  // store kept 1220 while the field stayed blank.
  it('saga year: an emptied field shows the year the document still holds', () => {
    const input = mountField(SagaYearField, 'saga-year-input');
    clearField(input);
    expect(store.entity.saga_year).toBe(1220);

    commit(input);
    expect(input.value).toBe('1220');
  });
});
