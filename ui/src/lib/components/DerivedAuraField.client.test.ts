import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// A `client` test, not `ssr`, and it has to be: the event this component now
// gives feedback about is the CLAMP, which only happens when an `oninput`
// listener fires on a live element. SSR renders markup and never dispatches an
// event, so an `ssr` assertion after "type into the field" reports green with
// nothing having happened — which is exactly how the hint came to be unreachable
// (Gerda #1 / Sabine #2, full-audit round 2): every test of it rendered a STORED
// out-of-range value, a state no code path can produce any more.
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
  readSettings: vi.fn().mockResolvedValue({
    default_saga_year: 1220,
    lang: null,
    theme: null,
    validation_mode: null,
  }),
  writeSettings: vi.fn().mockResolvedValue(undefined),
  deriveAge: vi.fn().mockResolvedValue({ age: null, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(null),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import DerivedAuraField from './DerivedAuraField.svelte';

const AURA_MIN = -50;
const AURA_MAX = 10;

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
      aura_modifier_min: AURA_MIN,
      aura_modifier_max: AURA_MAX,
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

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
    xp_pool: 0,
    ability_funding: 'pool',
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    aura: 0,
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  store.view = 'editor';
  installRuleset();
  resetEntity();
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(DerivedAuraField, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

/** Type `value` into the aura field, as a player would. */
function typeAura(value: string): void {
  const input = target.querySelector<HTMLInputElement>('[data-testid="derived-aura-input"]');
  expect(input).toBeTruthy();
  input!.value = value;
  input!.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

function hint(): HTMLElement | null {
  return target.querySelector<HTMLElement>('[data-testid="derived-aura-out-of-range"]');
}

function auraInput(): HTMLInputElement {
  const input = target.querySelector<HTMLInputElement>('[data-testid="derived-aura-input"]');
  if (!input) throw new Error('no aura input');
  return input;
}

describe('DerivedAuraField: the clamp is never silent', () => {
  it('says nothing while the typed value is inside the rules range', () => {
    typeAura('3');
    expect(store.entity.aura).toBe(3);
    expect(hint()).toBeNull();
    expect(auraInput().getAttribute('aria-describedby')).toBeNull();
  });

  it('tells the player when their typed value was rewritten to the bound', () => {
    // The failure this fixes: a magus in an aura of 12 types 12, setAura clamps
    // to 10, the controlled binding rewrites the field under the caret, and
    // nothing on screen says a bound exists — while every derived read-out below
    // is computed from 10.
    typeAura('12');

    expect(store.entity.aura).toBe(AURA_MAX);
    const shown = hint();
    expect(shown).not.toBeNull();
    expect(shown!.textContent).toContain(String(AURA_MIN));
    expect(shown!.textContent).toContain(String(AURA_MAX));
    // Announced politely, so it does not talk over the keystroke in progress.
    expect(shown!.getAttribute('role')).toBe('status');
    // And associated with the input, so a screen-reader user tabbing to the
    // field learns about it at all (round 4, S2 — kept through the re-aim).
    expect(auraInput().getAttribute('aria-describedby')).toBe('derived-aura-out-of-range');
  });

  it('clamps and warns at the negative bound too', () => {
    typeAura('-999');
    expect(store.entity.aura).toBe(AURA_MIN);
    expect(hint()).not.toBeNull();
  });

  it('retires the hint once the next entry is legal', () => {
    typeAura('12');
    expect(hint()).not.toBeNull();

    typeAura('7');

    expect(store.entity.aura).toBe(7);
    expect(hint()).toBeNull();
    expect(auraInput().getAttribute('aria-describedby')).toBeNull();
  });

  it('treats a cleared field as no entry at all, not as a clamp', () => {
    // An empty input is `setAura(null)` — the store stores 0, which is in range
    // and is not the player being overridden.
    typeAura('');
    expect(store.entity.aura).toBe(0);
    expect(hint()).toBeNull();
  });
});
