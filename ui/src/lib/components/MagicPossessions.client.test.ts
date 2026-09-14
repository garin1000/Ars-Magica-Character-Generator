import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// A `client` test, not `ssr`: the aura hint reports the CLAMP, which only
// happens when the `oninput` listener fires on a live element. SSR never
// dispatches an event, so the assertion would report green with nothing having
// happened. The Magic Items tab is the SECOND aura entry point — the Derived
// Totals tab's `DerivedAuraField` is the other — and the two must give the same
// feedback for the same keystroke (Gerda #1 / Sabine #2, full-audit round 2).
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
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import MagicPossessions from './MagicPossessions.svelte';

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
    spells: [],
    devices: [],
    aura: 0,
  };
  store.derived = null;
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
  app = mount(MagicPossessions, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function typeAura(value: string): void {
  const input = target.querySelector<HTMLInputElement>('[data-testid="aura-input"]');
  expect(input).toBeTruthy();
  input!.value = value;
  input!.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

function hint(): HTMLElement | null {
  return target.querySelector<HTMLElement>('[data-testid="aura-out-of-range"]');
}

describe('MagicPossessions: the aura clamp is never silent', () => {
  it('says nothing while the typed value is inside the rules range', () => {
    typeAura('3');
    expect(store.entity.aura).toBe(3);
    expect(hint()).toBeNull();
  });

  it('tells the player when their typed value was rewritten to the bound', () => {
    typeAura('12');

    expect(store.entity.aura).toBe(AURA_MAX);
    const shown = hint();
    expect(shown).not.toBeNull();
    expect(shown!.textContent).toContain(String(AURA_MAX));
    expect(shown!.getAttribute('role')).toBe('status');
    const input = target.querySelector<HTMLInputElement>('[data-testid="aura-input"]');
    expect(input!.getAttribute('aria-describedby')).toBe('aura-out-of-range');
  });

  it('retires the hint once the next entry is legal', () => {
    typeAura('-999');
    expect(hint()).not.toBeNull();

    typeAura('-4');

    expect(store.entity.aura).toBe(-4);
    expect(hint()).toBeNull();
  });
});
