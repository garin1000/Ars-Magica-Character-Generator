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
    ability_funding: 'pool',
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

/**
 * Put the entity in guided funding. Since schema 16 that takes BOTH the stored mode
 * and the plan: a plan alone is inert data a pool-funded character may legitimately
 * carry, so setting only `life_stages` would leave the bar in its flat shape.
 */
function installPlan(plan: LifeStagePlan = {}): void {
  store.entity.ability_funding = 'life_stages';
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

/** Fluent isolates every interpolated value in bidi marks; strip them. */
function clean(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

/**
 * The life-stage chips in DOCUMENT order — which is the whole point of #14: the
 * bar's blocks must read as a chronology, and only their rendered order can say
 * whether they do.
 */
function chips(body: string, prefix = ''): { testid: string; text: string }[] {
  const re = new RegExp(
    `<span[^>]*data-testid="${prefix}(life-stage-[a-z-]+)"[^>]*>([\\s\\S]*?)</span>`,
    'gi',
  );
  return [...body.matchAll(re)].map((match) => ({
    testid: match[1],
    text: clean(match[2].replace(/<[^>]*>/g, '')).trim(),
  }));
}

/** The leading label of a chip: everything before its first figure or separator. */
function chipLabel(text: string): string {
  return text.split(/[:—(]/)[0].trim();
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

  // A pure CSS-class rename must not break a spec that only cares about the
  // overspend STATE, not its presentation — so the state is also exposed
  // through a semantic `data-overspent` attribute (arts.e2e.js reads this
  // instead of the `over-value`/`over` classes, finding E7).
  it('marks Available with a semantic data-overspent attribute when overspent', () => {
    resetEntity(20);
    setEffective(30, []); // available = -10
    const { open } = element(html(), 'xp-available');
    expect(open).toMatch(/data-overspent="true"/);
  });

  it('marks the used figure with a semantic data-overspent attribute when overspent', () => {
    resetEntity(10);
    setEffective(15, []); // available = -5
    const { open } = element(html(), 'xp-spent');
    expect(open).toMatch(/data-overspent="true"/);
  });

  it('reports data-overspent="false" on both figures when the pool covers the spend', () => {
    resetEntity(50);
    setEffective(30, []);
    expect(element(html(), 'xp-available').open).toMatch(/data-overspent="false"/);
    expect(element(html(), 'xp-spent').open).toMatch(/data-overspent="false"/);
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
    // Under life-stage funding the pools are derived from the stages, so an editable
    // total would offer a figure the engine never reads: the input must be gone
    // entirely — and a read-only span, not a disabled input, so assistive tech does
    // not announce an unusable control.
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

  it('offers no way to destroy a typed pool that a life-stage plan is not using', () => {
    // There used to be a "Clear pool" button here, and schema 16 removed its whole
    // reason to exist. It was added because the engine reported a plan beside a typed
    // pool as `life_stage_xp_pool_conflict`, an error the guided bar showed no field to
    // correct — so the button was the only escape. Schema 16 stores the funding mode
    // explicitly (#29) and RETIRED that finding: the pair is now the ORDINARY shape of
    // a character who typed a pool and then switched to the stages, and the pool is
    // merely inert rather than illegal.
    //
    // Keeping the button would have left a one-click destroyer of exactly the data
    // this slice exists to preserve, under a hint that asserted the retired rule
    // ("a pool entered by hand has to go back to 0"). The typed total is editable on
    // the flat side whenever the player switches back, so nothing is unreachable.
    resetEntity(40);
    installPlan();
    setEffective(0, [], budget(10, 15));
    const body = html();
    expect(has(body, 'xp-pool-clear')).toBe(false);
    expect(has(body, 'xp-pool-clear-hint')).toBe(false);
  });

  it('still offers no clear button when the plan carries no typed pool', () => {
    resetEntity(0);
    installPlan();
    setEffective(0, [], budget(10, 15));
    const body = html();
    expect(has(body, 'xp-pool-clear')).toBe(false);
    expect(has(body, 'xp-pool-clear-hint')).toBe(false);
  });

  it('localizes both childhood blocks under the one Early childhood chip', () => {
    resetEntity(0);
    installPlan();
    setEffective(20, childhoodPools(5, 15), budget(10, 15));
    const { text } = element(html(), 'life-stage-early-childhood');
    // Both blocks arrive as restricted pools with a life_stage origin, and #14 merges
    // them into ONE chip: the 75 for the native language and the 45 spread are one
    // block of the rules (Core Rules.md:2378), so they share one heading rather than
    // reading as two blocks — which is what put "Early childhood" on the spread row
    // while the block's real name was the heading it lacked.
    expect(text).toContain('Early childhood');
    expect(text).toContain('Native language');
    expect(text).toContain('5');
    expect(text).toContain('75');
    expect(text).toContain('15');
    expect(text).toContain('45');
    // Never the raw block slug.
    expect(text).not.toContain('childhood_native_language');
    expect(text).not.toContain('childhood_spread');
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
    // And no clear button on this instance either — it is gone from both (schema 16
    // retired the finding it existed to escape).
    expect(has(body, 'art-xp-pool-clear')).toBe(false);
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

  it('keeps the later-life block beside it, with its derivation and its spend in one chip', () => {
    resetEntity(0);
    installPlan();
    setEffective(20, laterLifePool(75, 20), magusBudget());
    // Before #14 this was TWO chips: a `life-stage-later-life` line saying WHY (5 ×
    // 15 = 75) and a separate restricted row saying how much of it was spent, both
    // labelled "Later life". One chip now carries both.
    // Fluent isolates each interpolated value, so the figures are asserted apart.
    const { text } = element(html(), 'life-stage-later-life');
    expect(text).toContain('Later life');
    expect(text).toContain('20');
    expect(text).toContain('75');
    expect(text).not.toContain('later_life');
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
    expect(element(body, 'art-life-stage-later-life').text).toContain('Later life');
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

// guided-creation-review-2026-08 #14. The bar used to show TWO chips both labelled
// "Later life" — the derivation (`5 × 15 = 75 XP`) and, separately, the restricted
// pool it forms (`Later life (Abilities only): 0 / 75`) — in an order that put later
// life AFTER apprenticeship, so the label read as life past the Gauntlet. The blocks
// are a chronology in the rules' own summary
// (Ars Magica - Definitive Edition (Core Rules).md:2213-2216, :2364), so the bar
// reads as one: one chip per block, in the order the character lived them.
describe('XpBar life-stage chronology (#14)', () => {
  /**
   * Every block a magus can have on screen at once: childhood's two, later life
   * (restricted for a magus), apprenticeship, and ten years past the Gauntlet.
   * Gauntlet age 25 and a childhood of five years put later life at ages 5-10 —
   * the very span the Darius example works through (`:2402`).
   */
  function installEveryBlock(): void {
    resetEntity(0);
    installPlan({ gauntlet_age: 25 });
    installPostApprenticeshipRules();
    setEffective(0, [...childhoodPools(), ...laterLifePool(75)], pastGauntletBudget(10));
    store.effective!.xp_general_pool = 240 + 300;
  }

  it('renders the life-stage blocks in chronological order', () => {
    installEveryBlock();
    const rendered = chips(html());
    expect(rendered.map((chip) => chip.testid)).toEqual([
      'life-stage-early-childhood',
      'life-stage-later-life',
      'life-stage-apprenticeship',
      'life-stage-post-gauntlet',
    ]);
    // And the labels the player actually reads, in that same order — the testids
    // could be right while the wording still said "As a magus" in the wrong place.
    expect(rendered.map((chip) => chipLabel(chip.text))).toEqual([
      'Early childhood',
      'Later life',
      'Apprenticeship',
      'After the Gauntlet',
    ]);
  });

  it('renders exactly one chip per life-stage block', () => {
    installEveryBlock();
    const body = html();
    // No label twice: the derivation and the restricted pool are now one chip, so
    // each block's name occurs exactly once in the whole bar.
    for (const label of ['Early childhood', 'Later life', 'Apprenticeship', 'After the Gauntlet']) {
      expect(clean(body.replace(/<[^>]*>/g, ' ')).split(label).length - 1).toBe(1);
    }
    // ...and the merged pools no longer appear a second time as generic restricted
    // rows. A life-stage block belongs to its own chip; only an ITEM-granted pool
    // (Educated, Warrior, Privileged Upbringing) still gets a `restricted-xp-N` row.
    expect(has(body, 'restricted-xp-0')).toBe(false);
  });

  it('merges each block figure with its restricted pool inside the one chip', () => {
    resetEntity(0);
    installPlan({ gauntlet_age: 25 });
    installPostApprenticeshipRules();
    // 20 spent out of childhood's 75, 15 of its 45, and 30 of later life's 75.
    setEffective(65, [...childhoodPools(20, 15), ...laterLifePool(75, 30)], magusBudget());
    const rendered = chips(html());
    const childhood = rendered.find((c) => c.testid === 'life-stage-early-childhood')!;
    // Both childhood sub-pools under the one Early childhood heading (`:2378`).
    expect(childhood.text).toContain('20');
    expect(childhood.text).toContain('75');
    expect(childhood.text).toContain('15');
    expect(childhood.text).toContain('45');
    expect(childhood.text).toContain('Native language');
    const later = rendered.find((c) => c.testid === 'life-stage-later-life')!;
    // The derivation AND the pool it forms, in one line: 5 × 15 = 75 XP, 30 spent.
    expect(later.text).toContain('75');
    expect(later.text).toContain('30');
  });

  it("labels later life with the character's own age span", () => {
    installEveryBlock();
    const later = chips(html()).find((c) => c.testid === 'life-stage-later-life')!;
    // Childhood runs to 5 (`life_stages.childhood.years`) and apprenticeship starts
    // at the Gauntlet age less its fifteen years, so later life is ages 5 to 10 —
    // exactly the Darius worked example (`:2402`).
    expect(later.text).toContain('ages 5-10');
    // ASCII hyphen-minus as the range separator, never U+2212.
    expect(later.text).not.toContain('−');
  });

  it("spans a guided companion's later life from childhood to its age", () => {
    resetEntity(0);
    installPlan();
    installPostApprenticeshipRules();
    // A companion of 25: no apprenticeship, so later life runs the whole way.
    setEffective(0, childhoodPools(), budget(20, 15));
    const later = chips(html()).find((c) => c.testid === 'life-stage-later-life')!;
    expect(later.text).toContain('ages 5-25');
  });

  it('drops the childhood chip when the plan has formed no childhood pool yet', () => {
    resetEntity(0);
    installPlan();
    installPostApprenticeshipRules();
    // No native language named and no spread pool: nothing to report about childhood.
    setEffective(0, [], budget(20, 15));
    expect(chips(html()).map((chip) => chip.testid)).toEqual(['life-stage-later-life']);
  });

  it('names childhood without its native-language pool before a language is chosen', () => {
    resetEntity(0);
    installPlan();
    installPostApprenticeshipRules();
    // The engine forms the native-language pool only once the language is named
    // (`effective/xp.rs`), so the spread can be the only childhood pool on screen.
    setEffective(0, [childhoodPools()[1]], budget(20, 15));
    const childhood = chips(html()).find((c) => c.testid === 'life-stage-early-childhood')!;
    expect(chipLabel(childhood.text)).toBe('Early childhood');
    expect(childhood.text).toContain('45');
    expect(childhood.text).not.toContain('Native language');
  });

  it('gives the Arts instance the identical chronology', () => {
    installEveryBlock();
    expect(chips(html('art-'), 'art-').map((chip) => chip.testid)).toEqual([
      'life-stage-early-childhood',
      'life-stage-later-life',
      'life-stage-apprenticeship',
      'life-stage-post-gauntlet',
    ]);
  });

  it('still lists an item-granted restricted pool as its own row', () => {
    resetEntity(0);
    installPlan();
    installPostApprenticeshipRules();
    setEffective(
      0,
      [
        ...childhoodPools(),
        {
          amount: 50,
          used: 10,
          categories: ['martial'],
          origin: { kind: 'item', item: 'virtue.warrior' },
        },
      ],
      budget(20, 15),
    );
    const body = html();
    // Indexed over the rows actually RENDERED, so `restricted-xp-0` still means
    // "the first restricted row on screen" — the life-stage blocks that used to
    // occupy indices 0 and 1 have moved into their own chronological chips.
    expect(has(body, 'restricted-xp-0')).toBe(true);
    expect(element(body, 'restricted-xp-0').text).toContain('Martial');
    expect(has(body, 'restricted-xp-1')).toBe(false);
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

  it('keeps the editable total on the typed pool, whatever the engine funds from', () => {
    resetEntity(200);
    setEffective(0, []);
    // The bracketed field is the number the player typed — `entity.xp_pool`, the
    // engine's own `base_general`. It is never overwritten by the pool the solve
    // funds from, which is that base plus any V/F contribution; the two are
    // reconciled by the bonus entry below, not by moving the field.
    store.effective!.xp_general_pool = 260;
    store.effective!.xp_general_bonus = 60;
    const body = html();
    expect(element(body, 'xp-pool').open).toMatch(/value="200"/);
  });
});

// A flat magus with Skilled Parens ("You gain an additional 60 experience points …
// during apprenticeship", Core Rules.md:4966) may spend 300 against a typed 240,
// and the engine grants exactly that. The bar used to charge the spend against the
// typed total alone, so the character read a negative Available with no error
// anywhere — a silently wrong read-out of a perfectly legal character.
describe('XpBar and the Virtue/Flaw pool bonus (slice 6b8c)', () => {
  /** A typed pool of `typed` raised by a signed V/F contribution, `used` spent. */
  function withBonus(typed: number, bonus: number, used: number): void {
    resetEntity(typed);
    setEffective(used, []);
    store.effective!.xp_general_pool = Math.max(typed + bonus, 0);
    store.effective!.xp_general_bonus = bonus;
    // The engine funds up to the pool and reports the rest as unfunded demand.
    store.effective!.xp_general_used = Math.min(used, store.effective!.xp_general_pool);
    store.effective!.xp_max_flow = store.effective!.xp_general_used;
  }

  it('spends the whole raised pool without reporting an overspend', () => {
    withBonus(240, 60, 300);
    const body = html();
    expect(element(body, 'xp-available').text).toContain('0');
    // Precise word-boundary match, not a bare substring: `data-overspent="false"`
    // itself contains the substring "over", so a plain `.not.toContain('over')`
    // would false-fail here.
    expect(element(body, 'xp-available').open).not.toMatch(/\bover\b/);
    expect(element(body, 'xp-available').open).toMatch(/data-overspent="false"/);
  });

  it('lists the bonus as a pool of its own, spent before the base', () => {
    withBonus(240, 60, 50);
    const bonus = element(html(), 'xp-bonus');
    // Drawn from the bonus first, exactly as a restricted pool is: 50 of 60.
    expect(bonus.text).toContain('50');
    expect(bonus.text).toContain('60');
    // ...so the base is untouched and fully available.
    expect(element(html(), 'xp-spent').text).toBe('0');
    expect(element(html(), 'xp-available').text).toContain('240');
  });

  it('still reports an overspend once the raised pool is exceeded', () => {
    withBonus(240, 60, 310);
    const body = html();
    expect(element(body, 'xp-available').text).toContain('-10');
    expect(element(body, 'xp-available').open).toContain('over');
  });

  it('charges a Weak Parens penalty to the base and shows it signed', () => {
    withBonus(240, -60, 60);
    const body = html();
    // 240 - (60 spent + 60 penalty) = 120, which is also pool (180) - used (60).
    expect(element(body, 'xp-spent').text).toBe('120');
    expect(element(body, 'xp-available').text).toContain('120');
    // ASCII hyphen, never U+2212 (formatSigned is the single source of truth).
    expect(element(body, 'xp-bonus').text).toContain('-60');
    expect(element(body, 'xp-bonus').text).not.toContain('−');
  });

  it('shows no bonus entry at all when no Virtue touches the pool', () => {
    resetEntity(200);
    setEffective(30, []);
    expect(has(html(), 'xp-bonus')).toBe(false);
  });

  it('raises the guided pool the same way, without a magus branch', () => {
    // Guided funding: the bracketed total is the life-stage block's own base, and
    // the bonus is listed beside it rather than folded into an unexplained number.
    resetEntity(0);
    installPlan();
    setEffective(0, [], magusBudget());
    store.effective!.xp_general_pool = 300;
    store.effective!.xp_general_bonus = 60;
    const body = html();
    expect(element(body, 'xp-pool-total').text).toBe('240');
    expect(element(body, 'xp-bonus').text).toContain('60');
    // Available is the BASE remainder: the unspent 60 stays in the bonus entry
    // rather than inflating it, exactly as unspent restricted XP does.
    expect(element(body, 'xp-available').text).toContain('240');
  });
});
