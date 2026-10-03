import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// A `client` test, not `ssr`: the behaviour under test is the birth-year field's
// `oninput` / `onchange` LISTENERS firing on a live element, keystroke by keystroke,
// and the value the live input shows afterwards. SSR dispatches no events.
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

import { SCHEMA_VERSION, store } from '../state.svelte';
import IdentityFields from './IdentityFields.svelte';

const BIRTH_YEAR = '[data-testid="identity-birth-year"]';

/** A ruleset whose aging block states the shipped app maximum age (slice A1). */
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

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'grog',
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
  };
  store.effective = null;
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

function mountField(): HTMLInputElement {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(IdentityFields, { target });
  flushSync();
  const input = target.querySelector<HTMLInputElement>(BIRTH_YEAR);
  expect(input).toBeTruthy();
  return input!;
}

/** Type `text` one key at a time, appending to whatever the field shows, as a user does. */
function typeKeys(input: HTMLInputElement, text: string): void {
  for (const key of text) {
    input.value = input.value + key;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
  }
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

// Slice A1 bounds the birth year below at saga year - max_age (1220 - 500 = 720).
// Every PREFIX of an ordinary year lies under that bound ("1", "11", "119"), so a
// clamp applied on each keystroke snaps the first digit to 720 and the rest append
// to it: typing 1190 stored 720190. The bound is applied when the value is
// committed, never mid-typing.
describe('IdentityFields birth year typed key by key (slice A1)', () => {
  it('takes a four-digit year typed one key at a time', () => {
    const input = mountField();
    typeKeys(input, '1190');
    expect(store.entity.birth_year).toBe(1190);
    expect(input.value).toBe('1190');
  });

  it('still holds a committed year below the bound to saga year - max_age', () => {
    const input = mountField();
    typeKeys(input, '1');
    commit(input);
    expect(store.entity.birth_year).toBe(720);
    expect(input.value).toBe('720');
  });
});
