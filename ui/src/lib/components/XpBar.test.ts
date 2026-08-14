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
  // The general pool is exactly the engine's `base_general` (`effective.rs`): for a
  // life-stage character the block that may fund anything — APPRENTICESHIP for a
  // magus, whose 240 points buy Arts as well as Abilities (Core Rules.md:2435), and
  // later life for anyone who serves none — and the typed `xp_pool` for a
  // directly-entered one. A non-zero apprenticeship block is what makes the character
  // a magus, so it is the discriminator here too. The bar reads the pool off
  // `xp_general_pool` rather than re-deriving it, so this must set that field.
  const pool = lifeStage
    ? lifeStage.apprenticeship_xp || lifeStage.later_life_xp
    : (store.entity.xp_pool ?? 0);
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
    xp_general_pool: pool,
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
    // No Gauntlet to live past: the years after one are a magus-only block, so for
    // everyone else the Gauntlet age is simply the age later life runs to.
    gauntlet_age: 5 + years,
    post_gauntlet_years: 0,
    post_gauntlet_points: 0,
    post_gauntlet_spell_levels: 0,
    post_gauntlet_xp: 0,
  };
}

/**
 * A life-stage budget for a guided MAGUS: fifteen years of apprenticeship earning the
 * 240 points that "can be spent on Arts or Abilities"
 * (Ars Magica - Definitive Edition (Core Rules).md:2435), which makes apprenticeship
 * the general pool — and a later life that stops at the Gauntlet (five years for a
 * magus of 25), which is a restricted, Abilities-only pool instead.
 */
function magusBudget(years = 5, rate = 15): LifeStageBudget {
  return {
    ...budget(years, rate),
    apprenticeship_years: 15,
    apprenticeship_xp: 240,
    // Standing at its Gauntlet: childhood + later life + the fifteen years of
    // apprenticeship, with no year after it yet.
    gauntlet_age: 5 + years + 15,
  };
}

/**
 * A guided magus a while past its Gauntlet: `years × 30` less `seasons × 10`, with
 * `spellLevels` of those points taken as spells and the rest as experience. That
 * experience joins apprenticeship in the general pool, which is why the caller must
 * raise `xp_general_pool` to match.
 */
function pastGauntletBudget(years: number, seasons = 0, spellLevels = 0): LifeStageBudget {
  const points = years * 30 - Math.min(seasons, 3 * years) * 10;
  return {
    ...magusBudget(),
    post_gauntlet_years: years,
    post_gauntlet_points: points,
    post_gauntlet_spell_levels: spellLevels,
    post_gauntlet_xp: points - spellLevels,
  };
}

/**
 * The life-stage rules the bar reads the per-year rate off. The rate is DATA — the
 * bar must never carry the 30 as a literal.
 */
function installPostApprenticeshipRules(pointsPerYear = 30): void {
  store.ruleset!.ruleset.life_stages = {
    childhood: {
      years: 5,
      native_language_ability: 'ability.living_language',
      native_language_xp: 75,
      spread_xp: 45,
      spread_abilities: [],
    },
    later_life: { xp_per_year: 15 },
    post_apprenticeship: {
      points_per_year: pointsPerYear,
      lab_season_cost: 10,
      max_charged_lab_seasons_per_year: 3,
    },
  };
}

/** The later-life block as the engine emits it for a magus: restricted, Abilities only. */
function laterLifePool(amount: number, used = 0): EffectiveScores['restricted_xp_pools'] {
  return [
    {
      amount,
      used,
      categories: ['general', 'academic'],
      origin: { kind: 'life_stage', block: 'later_life' },
    },
  ];
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

describe('XpBar under a guided magus plan (slice 6b4)', () => {
  it('shows the apprenticeship block as the pool the spend is charged against', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, laterLifePool(75), magusBudget());
    const body = html();
    // Apprenticeship is the general pool for a magus — not a grand total of 240 + 75 +
    // childhood, or `used / total` and Available would stop closing.
    expect(element(body, 'xp-pool-total').text).toBe('240');
    expect(element(body, 'xp-available').text).toContain('240');
  });

  it('names the apprenticeship block with its years and its experience', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, laterLifePool(75), magusBudget());
    const { text } = element(html(), 'life-stage-apprenticeship');
    expect(text).toContain('15');
    expect(text).toContain('240');
    expect(text).toContain('Apprenticeship');
    expect(text).not.toContain('−');
  });

  it('omits the apprenticeship line for a character who serves none', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, [], budget(10, 15));
    const body = html();
    // Zero magus branching in the component: the line is gated on the block itself, so
    // the guided companion bar is exactly what 6b3b shipped.
    expect(has(body, 'life-stage-apprenticeship')).toBe(false);
    expect(element(body, 'xp-pool-total').text).toBe('150');
    expect(() => element(body, 'life-stage-later-life')).not.toThrow();
  });

  it('keeps the later-life row beside it, labelled and never as its slug', () => {
    resetEntity(0);
    installPlan();
    setEffective(20, laterLifePool(75, 20), magusBudget());
    const body = html();
    // The restricted row shows the spend; the `life-stage-later-life` line above says
    // WHY (5 × 15 = 75). Both name it in words.
    // Fluent isolates each interpolated value, so used and amount are asserted apart.
    const later = element(body, 'restricted-xp-0');
    expect(later.text).toContain('20');
    expect(later.text).toContain('75');
    expect(later.text).toContain('Later life');
    expect(later.text).toContain('Abilities only');
    expect(later.text).not.toContain('later_life');
    expect(element(body, 'life-stage-later-life').text).toContain('75');
  });

  it('gives the Arts instance the identical guided shape', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, laterLifePool(75), magusBudget());
    const body = html('art-');
    // Apprenticeship experience buys Arts as well (`:2435`), so the Arts bar shows the
    // same 240 — the two instances differ only in their testid prefix.
    expect(element(body, 'art-xp-pool-total').text).toBe('240');
    expect(element(body, 'art-life-stage-apprenticeship').text).toContain('240');
    expect(element(body, 'art-restricted-xp-0').text).toContain('Later life');
  });
});

describe('XpBar for a magus past its Gauntlet (slice 6b5)', () => {
  /** A magus of 55 gauntleted at 25: 30 years, 6 charged lab seasons, 120 spell levels. */
  function installPastGauntlet(): void {
    resetEntity(0);
    installPlan({
      gauntlet_age: 25,
      post_gauntlet_lab_seasons: 6,
      post_gauntlet_spell_levels: 120,
    });
    installPostApprenticeshipRules();
    setEffective(0, laterLifePool(75), pastGauntletBudget(30, 6, 120));
    // Those years' experience joins apprenticeship in the general pool (the engine's
    // `base_general`), so the total the bar charges against is 240 + 720.
    store.effective!.xp_general_pool = 240 + 720;
  }

  it('names the years past the Gauntlet with the rate, the lab deduction and the experience', () => {
    installPastGauntlet();
    const { text } = element(html(), 'life-stage-post-gauntlet');
    // 30 years × 30 = 900, less 6 charged seasons × 10 = 840 points, 120 of them
    // taken as levels of spells, leaving 720 XP.
    expect(text).toContain('30');
    expect(text).toContain('60');
    expect(text).toContain('840');
    expect(text).toContain('720');
    expect(text).not.toContain('post_gauntlet');
    // ASCII hyphen-minus only; nothing here is negative but no U+2212 may leak in.
    expect(text).not.toContain('−');
  });

  it('reads the per-year rate off the ruleset rather than a literal 30', () => {
    installPastGauntlet();
    installPostApprenticeshipRules(20);
    const { text } = element(html(), 'life-stage-post-gauntlet');
    expect(text).toContain('20');
  });

  it('omits the line for a magus standing at its Gauntlet', () => {
    resetEntity(0);
    installPlan();
    installPostApprenticeshipRules();
    setEffective(0, laterLifePool(75), magusBudget());
    const body = html();
    // Gated on the block's own years, exactly like the apprenticeship line — so the
    // component still needs no notion of a magus.
    expect(has(body, 'life-stage-post-gauntlet')).toBe(false);
    expect(() => element(body, 'life-stage-apprenticeship')).not.toThrow();
  });

  it('omits the line for a guided companion', () => {
    resetEntity(0);
    installPlan();
    installPostApprenticeshipRules();
    setEffective(0, [], budget(10, 15));
    expect(has(html(), 'life-stage-post-gauntlet')).toBe(false);
  });

  it('gives the Arts instance the identical line', () => {
    installPastGauntlet();
    const body = html('art-');
    // Those points buy Arts as readily as Abilities (Core Rules.md:2471), so the Arts
    // bar carries the same row — the two instances differ only in their testid prefix.
    expect(element(body, 'art-life-stage-post-gauntlet').text).toContain('720');
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
    expect(has(body, 'life-stage-apprenticeship')).toBe(false);
    expect(has(body, 'life-stage-post-gauntlet')).toBe(false);
    expect(has(body, 'life-stage-no-budget')).toBe(false);
  });

  it('charges the typed pool, not xp_general_pool, when the two differ', () => {
    resetEntity(200);
    setEffective(0, []);
    // In flat mode the editable total IS `entity.xp_pool`: the engine's own
    // `base_general` there. Reading `xp_general_pool` unconditionally would be the
    // tempting over-simplification — and would silently shift the figure whenever a
    // Skilled Parens bonus is folded in, breaking the exact arithmetic
    // `arts.e2e.js` drives against this very input.
    store.effective!.xp_general_pool = 999;
    const body = html();
    expect(element(body, 'xp-pool').open).toMatch(/value="200"/);
    expect(element(body, 'xp-available').text).toContain('200');
  });
});
