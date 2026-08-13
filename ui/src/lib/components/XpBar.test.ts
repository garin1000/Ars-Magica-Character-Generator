import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  Ability,
  AbilityCategory,
  EffectiveScores,
  Entity,
  LifeStageBudget,
  LifeStagePlan,
  LocalizedRuleset,
} from '../types';

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
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
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
  lifeStage: LifeStageBudget | null = null,
): void {
  // The general pool is later life's experience for a life-stage character and the
  // typed `xp_pool` for a directly-entered one — exactly the engine's `base_general`
  // (`effective.rs`), so a guided-mode test cannot assert against an impossible flow.
  const pool = lifeStage ? lifeStage.later_life_xp : (store.entity.xp_pool ?? 0);
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
    life_stage: lifeStage,
  } as unknown as EffectiveScores;
}

/**
 * A life-stage budget for a non-magus: `years × rate` funds later life, which for
 * a grog or companion is the general pool. No apprenticeship — only a magus serves
 * one.
 */
function budget(years: number, rate: number): LifeStageBudget {
  return {
    childhood_native_xp: 75,
    childhood_spread_xp: 45,
    later_life_years: years,
    later_life_rate: rate,
    later_life_xp: years * rate,
    apprenticeship_years: 0,
    apprenticeship_xp: 0,
  };
}

/** Put the entity in guided funding: a plan present IS the switch. */
function installPlan(plan: LifeStagePlan = {}): void {
  store.entity.life_stages = plan;
}

/** The two childhood blocks as the engine emits them: restricted, life-stage origin. */
function childhoodPools(nativeUsed = 0, spreadUsed = 0): EffectiveScores['restricted_xp_pools'] {
  return [
    {
      amount: 75,
      used: nativeUsed,
      origin: { kind: 'life_stage', block: 'childhood_native_language' },
    },
    {
      amount: 45,
      used: spreadUsed,
      origin: { kind: 'life_stage', block: 'childhood_spread' },
    },
  ];
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

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
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
    setEffective(30, [
      {
        amount: 50,
        used: 10,
        categories: ['martial'],
        origin: { kind: 'item', item: 'virtue.warrior' },
      },
    ]);
    const { text } = element(html(), 'restricted-xp-0');
    expect(text).toContain('Martial');
    expect(text).toContain('10');
    expect(text).toContain('50');
  });

  it('uses the art- testid prefix for the Arts instance', () => {
    setEffective(30, [
      {
        amount: 50,
        used: 10,
        categories: ['martial'],
        origin: { kind: 'item', item: 'virtue.warrior' },
      },
    ]);
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

describe('XpBar under a life-stage plan (slice 6b3b)', () => {
  it('replaces the editable pool input with a read-only later-life total', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, [], budget(10, 15));
    const body = html();
    // The engine forbids a plan AND a typed pool (life_stage_xp_pool_conflict), so
    // the input must be gone entirely — and a read-only span, not a disabled input,
    // so assistive tech does not announce an unusable control.
    expect(has(body, 'xp-pool')).toBe(false);
    const { open, text } = element(body, 'xp-pool-total');
    expect(open).toMatch(/<span/i);
    expect(open).not.toMatch(/<input/i);
    expect(text).toBe('150');
  });

  it('renders the later-life row as years, rate and total experience', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, [], budget(10, 15));
    const { text } = element(html(), 'life-stage-later-life');
    expect(text).toContain('10');
    expect(text).toContain('15');
    expect(text).toContain('150');
    // ASCII hyphen-minus only; nothing here is negative but no U+2212 may leak in.
    expect(text).not.toContain('−');
  });

  it('keeps the spent and available arithmetic against the derived later-life pool', () => {
    resetEntity(0);
    installPlan();
    setEffective(60, [], budget(10, 15)); // 150-point general pool, 60 spent
    const body = html();
    expect(element(body, 'xp-spent').text).toBe('60');
    expect(element(body, 'xp-available').text).toContain('90');
  });

  it('announces a missing budget with role=status when the plan yields none', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, [], null); // no age typed, or a ruleset without life-stage rules
    const body = html();
    const { open, text } = element(body, 'life-stage-no-budget');
    // It appears in response to an unrelated edit (the age), so its arrival must
    // be announced rather than silently rendered.
    expect(open).toMatch(/role="status"/);
    expect(text).toContain('No life-stage experience yet');
    expect(has(body, 'life-stage-later-life')).toBe(false);
  });

  it('offers the clear button only when a plan coexists with a typed pool', () => {
    resetEntity(40); // a hand-edited save carrying both
    installPlan();
    setEffective(0, [], budget(10, 15));
    const body = html();
    const { open, text } = element(body, 'xp-pool-clear');
    expect(open).toMatch(/<button/i);
    expect(text).toContain('Clear pool');
    // The hint explaining why the pool must go back to 0 is wired for assistive tech.
    expect(open).toMatch(/aria-describedby="xp-pool-clear-hint"/);
    expect(element(body, 'xp-pool-clear-hint').open).toMatch(/id="xp-pool-clear-hint"/);
  });

  it('omits the clear button when the plan carries no typed pool', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, [], budget(10, 15));
    const body = html();
    expect(has(body, 'xp-pool-clear')).toBe(false);
    expect(has(body, 'xp-pool-clear-hint')).toBe(false);
  });

  it('localizes the childhood blocks the engine already emits, with no new code', () => {
    resetEntity(0);
    installPlan();
    setEffective(20, childhoodPools(5, 15), budget(10, 15));
    const body = html();
    // Both blocks arrive as ordinary restricted pools with a life_stage origin, so
    // the existing restricted rows name them through the xp-pool-<block> keys.
    expect(element(body, 'restricted-xp-0').text).toContain('Native language');
    expect(element(body, 'restricted-xp-0').text).toContain('75');
    expect(element(body, 'restricted-xp-1').text).toContain('Early childhood');
    expect(element(body, 'restricted-xp-1').text).toContain('45');
  });

  it('gives the Arts instance the identical guided shape', () => {
    resetEntity(40);
    installPlan();
    setEffective(0, [], budget(10, 15));
    const body = html('art-');
    // Arts spend the SAME pool, so the Arts bar must not offer the forbidden input
    // either — this is correct, not an oversight to be "fixed" later.
    expect(has(body, 'art-xp-pool')).toBe(false);
    expect(element(body, 'art-xp-pool-total').text).toBe('150');
    expect(() => element(body, 'art-life-stage-later-life')).not.toThrow();
    expect(() => element(body, 'art-xp-pool-clear')).not.toThrow();
  });
});

describe('XpBar flat mode is untouched by the guided branch (slice 6b3b)', () => {
  it('keeps the editable pool input and adds no life-stage node without a plan', () => {
    resetEntity(200);
    setEffective(0, []);
    const body = html();
    // Four e2e specs drive this input; every guided node must stay behind the plan check.
    expect(element(body, 'xp-pool').open).toMatch(/<input/i);
    expect(has(body, 'xp-pool-total')).toBe(false);
    expect(has(body, 'xp-pool-clear')).toBe(false);
    expect(has(body, 'life-stage-later-life')).toBe(false);
    expect(has(body, 'life-stage-no-budget')).toBe(false);
  });
});
