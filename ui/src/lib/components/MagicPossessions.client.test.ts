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
  // The document-swap test below drives the real `AppStore.open()`, which asks
  // to discard the edits the test just made; without this the store falls back
  // to the in-app prompt and the promise never settles.
  confirmDiscard: vi.fn().mockResolvedValue(true),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
}));

import * as ipc from '../ipc';
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

/** A minimal magus carrying `aura`, and a `name` so two of them differ. */
function magus(aura: number, name: string): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    name,
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
    aura,
  };
}

function resetEntity(): void {
  store.entity = magus(0, 'Marcus');
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

  it('does not claim a range violation for an in-range entry that was merely truncated', () => {
    // The Magic Items tab's half of Gerda #3 (round 3): `clampInt` clamps AND
    // truncates, so a fractional but in-range entry must not raise the
    // out-of-range hint. The two entry points give the same feedback for the
    // same keystroke, so they are tested with the same keystroke.
    typeAura('2.5');

    expect(store.entity.aura).toBe(2);
    expect(hint()).toBeNull();
    const input = target.querySelector<HTMLInputElement>('[data-testid="aura-input"]');
    expect(input!.getAttribute('aria-describedby')).toBeNull();
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

  it('retires the hint when a DIFFERENT document is opened behind it', async () => {
    // Sabine #1 (full-audit round 3). The Magic Items tab stays selected across
    // a document swap and this panel is never recreated, so the hint — and the
    // input's `aria-describedby` — would otherwise assert that the newly opened
    // character's in-range aura had been adjusted.
    typeAura('12');
    expect(hint()).not.toBeNull();

    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/somewhere/quendalon.armc',
      entity: magus(3, 'Quendalon'),
      migrated_aging_characteristics: [],
    });
    await store.open();
    flushSync();

    expect(store.entity.aura).toBe(3);
    expect(hint()).toBeNull();
    const input = target.querySelector<HTMLInputElement>('[data-testid="aura-input"]');
    expect(input!.getAttribute('aria-describedby')).toBeNull();
  });
});
