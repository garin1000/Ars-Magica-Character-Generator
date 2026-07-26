import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Ability, AbilityCategory, EffectiveScores, Entity, LocalizedRuleset } from '../types';

// The XP bar reads the shared store singleton (entity + engine-derived effective
// scores) and the Fluent bundle. The store schedules a debounced revalidate over
// the Tauri IPC bridge; mock the bridge so nothing tries to reach a backend.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
}));

import { store } from '../state.svelte';
import XpBar from './XpBar.svelte';

function ability(id: string, category: AbilityCategory = 'martial'): Ability {
  return { id, category };
}

/** Install a minimal localized ruleset carrying the abilities the pools cite. */
function installRuleset(abilities: Ability[] = []): void {
  const abilityMap: Record<string, Ability> = {};
  for (const a of abilities) abilityMap[a.id] = a;
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: abilityMap,
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(pool: number): void {
  store.entity = {
    schema_version: 11,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: pool,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

/**
 * Installs effective scores for a TOTAL demand, deriving the rest exactly as the
 * engine's `xp_allocation` max-flow does — so a test can never assert against a
 * state the engine cannot produce.
 *
 * Critically, `xp_general_used` is a flow bounded by the pool: however far the
 * demand overshoots, it stops AT the pool and the unfundable remainder shows up as
 * `total_demand - max_flow`. Deriving it here (rather than letting each test pass a
 * free-hand `generalUsed`) is what keeps the overspend tests honest — an earlier
 * version of this helper accepted `generalUsed > pool`, which made the overspend
 * assertions pass against input the app can never reach.
 */
function setEffective(
  totalDemand: number,
  restricted: EffectiveScores['restricted_xp_pools'],
): void {
  const pool = store.entity.xp_pool ?? 0;
  const restrictedUsed = restricted.reduce((sum, p) => sum + p.used, 0);
  // Restricted pools are drained first (the engine's two-phase flow), so the
  // general pool funds the remainder — up to its size, never beyond.
  const generalUsed = Math.min(Math.max(totalDemand - restrictedUsed, 0), pool);
  store.effective = {
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
    xp_total_demand: totalDemand,
    xp_general_used: generalUsed,
    xp_max_flow: restrictedUsed + generalUsed,
    restricted_xp_pools: restricted,
  } as unknown as EffectiveScores;
}

/** Render the bar to an HTML string (node env, no DOM). */
function html(prefix = ''): string {
  return render(XpBar, { props: { prefix } }).body;
}

/** The single element carrying a given data-testid, with its class attribute. */
function element(body: string, testid: string): { open: string; text: string } {
  const re = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i');
  const match = re.exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  const openMatch = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  return { open: openMatch![0], text: match[1].replace(/<[^>]*>/g, '').trim() };
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset([ability('ability.great_weapon', 'martial')]);
  resetEntity(30);
  setEffective(30, []);
});

describe('XpBar layout data (Issue G)', () => {
  it('renders the general pool total as the editable xp-pool input carrying the value', () => {
    resetEntity(200);
    setEffective(0, []);
    const body = html();
    const { open } = element(body, 'xp-pool');
    // Editable field with the entity's pool as its value.
    expect(open).toMatch(/<input/i);
    expect(open).toMatch(/value="200"/);
  });

  it('renders the read-only general used figure on the xp-spent testid', () => {
    resetEntity(50);
    setEffective(30, []);
    const { text } = element(html(), 'xp-spent');
    expect(text).toContain('30');
  });

  it('renders each restricted sub-budget as used/amount with its localized label', () => {
    setEffective(30, [{ amount: 50, used: 10, categories: ['martial'] }]);
    const { text } = element(html(), 'restricted-xp-0');
    expect(text).toContain('Martial');
    expect(text).toContain('10');
    expect(text).toContain('50');
  });

  it('uses the art- testid prefix for the Arts instance', () => {
    setEffective(30, [{ amount: 50, used: 10, categories: ['martial'] }]);
    const body = html('art-');
    expect(() => element(body, 'art-xp-pool')).not.toThrow();
    expect(() => element(body, 'art-xp-spent')).not.toThrow();
    expect(() => element(body, 'art-xp-available')).not.toThrow();
    expect(() => element(body, 'art-restricted-xp-0')).not.toThrow();
  });
});

describe('XpBar negative-available regression (Issue G(a))', () => {
  it('marks Available with the over class and shows label+value when the pool is overspent', () => {
    resetEntity(20);
    setEffective(30, []); // general used exceeds the 20-point pool -> available = -10
    const { open, text } = element(html(), 'xp-available');
    // The over class drives the bold-red styling on both label and value.
    expect(open).toMatch(/class="[^"]*\bover\b[^"]*"/);
    // Label and value both live inside the one xp-available element.
    expect(text).toContain('Available');
    // ASCII hyphen-minus, never U+2212.
    expect(text).toContain('-10');
    expect(text).not.toContain('−');
  });

  it('does not mark Available as over when the pool covers the spend', () => {
    resetEntity(50);
    setEffective(30, []);
    const { open } = element(html(), 'xp-available');
    expect(open).not.toMatch(/\bover\b/);
  });

  it('marks the used figure with the over-value class when the pool is overspent', () => {
    resetEntity(10);
    setEffective(15, []); // used 15 exceeds the 10-point pool -> available = -5
    const { open, text } = element(html(), 'xp-spent');
    // over-value drives the red (non-bold) styling on the used figure; the bold
    // treatment stays on the Available value alone.
    expect(open).toMatch(/class="[^"]*\bover-value\b[^"]*"/);
    expect(text).toContain('15');
  });

  it('does not mark the used figure as over-value when the pool covers the spend', () => {
    resetEntity(50);
    setEffective(30, []);
    const { open } = element(html(), 'xp-spent');
    expect(open).not.toMatch(/\bover-value\b/);
  });
});
