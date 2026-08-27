import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// The Magic Items tab reads the shared store singleton (entity + ruleset) and the
// Fluent bundle. The store schedules a debounced revalidate over the Tauri IPC
// bridge; mock the bridge so nothing reaches a backend. Mirrors
// DerivedTotalsPanel.test.ts / FamiliarPanel.test.ts.
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

/** Installs a ruleset, with the round-3 aura bound fields by default. */
function installRuleset(overrides: Partial<LocalizedRuleset['ruleset']> = {}): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      aura_modifier_min: -50,
      aura_modifier_max: 10,
      ...overrides,
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
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
    devices: [],
  };
  store.derived = null;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(MagicPossessions, { props: {} }).body;
}

/** The single element carrying a given data-testid, tags stripped, plus its opening tag. */
function element(body: string, testid: string): { open: string; text: string } {
  const re = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i');
  const match = re.exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  const openMatch = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  return { open: openMatch![0], text: match[1].replace(/<[^>]*>/g, '').trim() };
}

function hasElement(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`, 'i').test(body);
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('MagicPossessions aura bound (round 3, Task 3)', () => {
  // G3/Task 3: the input used to carry min="-2147483648" max="2147483647" — the
  // full i32 range — instead of the engine's actual rules bound, so the browser
  // never hinted at the real range and a wildly out-of-range entry was silently
  // rewritten only later, at save.
  it('bounds the aura input to the engine-surfaced rules range, not the raw i32 range', () => {
    const body = html();
    const input = element(body, 'aura-input').open;
    expect(input).toContain('min="-50"');
    expect(input).toContain('max="10"');
    expect(input).not.toContain('-2147483648');
    expect(input).not.toContain('2147483647');
  });

  it('reads the bound from whatever the ruleset payload carries, not a hardcoded pair', () => {
    installRuleset({ aura_modifier_min: -7, aura_modifier_max: 4 });
    const body = html();
    const input = element(body, 'aura-input').open;
    expect(input).toContain('min="-7"');
    expect(input).toContain('max="4"');
  });

  it('shows no out-of-range hint for a legal aura value', () => {
    store.entity.aura = 3;
    const body = html();
    expect(hasElement(body, 'aura-out-of-range')).toBe(false);
  });

  it('shows an out-of-range hint when the stored aura is beyond the engine bound', () => {
    // A hand-edited or stale save can carry an out-of-range aura before the next
    // save re-normalizes it (Entity::normalize is not called on load) — the input
    // itself does not clamp on render, so this must be reachable and visible.
    store.entity.aura = 999;
    const body = html();
    expect(hasElement(body, 'aura-out-of-range')).toBe(true);
  });

  it('shows an out-of-range hint for a value below the minimum too', () => {
    store.entity.aura = -999;
    const body = html();
    expect(hasElement(body, 'aura-out-of-range')).toBe(true);
  });

  // Round 4, S2: the hint used to be visually adjacent to the aura input with no
  // programmatic association, so a screen-reader user tabbing to the input never
  // learned their value was out of range.
  it('associates the out-of-range hint with the aura input via aria-describedby', () => {
    store.entity.aura = 999;
    const body = html();
    const input = element(body, 'aura-input').open;
    expect(input).toContain('aria-describedby="aura-out-of-range"');
  });

  it('does not describe the input when the aura is in range', () => {
    store.entity.aura = 3;
    const body = html();
    const input = element(body, 'aura-input').open;
    expect(input).not.toContain('aria-describedby');
  });

  it('announces the hint as a polite live region, not an interrupting one', () => {
    // role="status" is implicitly aria-live="polite" — the hint fires while the
    // player is still typing, so an assertive region would talk over them.
    store.entity.aura = 999;
    const body = html();
    const hint = element(body, 'aura-out-of-range').open;
    expect(hint).toContain('role="status"');
  });
});

// Slice 3 (#28). This tab was the magus's editor home for a Longevity Ritual only
// because the editor had no aging tab: the ritual's bonus is a term of the AGING
// TOTAL (Ars Magica - Definitive Edition (Core Rules).md:16567-16569), not a
// possession. With a real Aging tab the panel moves into `AgingPanel`, and it must
// leave here — two on-screen homes for one ritual is the trap the special case was
// invented to dodge in the first place.
describe('MagicPossessions after the Aging tab took the ritual (Slice 3)', () => {
  it('no longer renders the Longevity panel', () => {
    const body = html();
    expect(hasElement(body, 'longevity-add')).toBe(false);
    expect(hasElement(body, 'longevity-bonus')).toBe(false);
  });

  it('still renders an existing ritual nowhere on this tab', () => {
    // A stored ritual is what would betray a leftover mount: the empty state's Add
    // button is easy to lose sight of, the populated editor is not.
    store.entity.longevity_ritual = { source: 'self_made', bonus: 6, focus: '' };
    const body = html();
    expect(hasElement(body, 'longevity-bonus')).toBe(false);
    expect(hasElement(body, 'longevity-source-self_made')).toBe(false);
  });

  it('keeps the possessions that really are possessions', () => {
    const body = html();
    expect(hasElement(body, 'aura-input')).toBe(true);
    expect(hasElement(body, 'device-add')).toBe(true);
  });
});
