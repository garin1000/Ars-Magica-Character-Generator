import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EffectiveScores, Entity, LifeStageBudget, LocalizedRuleset } from '../types';

// N7/N9 (try-out 2026-10-04): the later-life and post-Gauntlet chips state only the
// result, and the calculation behind each — `years × rate = XP`, and `years × rate -
// lab = points` — moves into the chip's tooltip. A `client` test: `use:tooltip` is an
// action, which the SSR renderer never runs, so only a mounted chip can show that the
// calculation is there and reachable by keyboard focus.
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
  confirmDiscard: vi.fn().mockResolvedValue(true),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import XpBar from './XpBar.svelte';

/** The life-stage rules, with the post-Gauntlet rate as DATA (never a literal 30). */
function installRuleset(pointsPerYear = 30): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
      life_stages: {
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
      },
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function installGuidedEntity(): void {
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
    ability_funding: 'life_stages',
    life_stages: { gauntlet_age: 25 },
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

/** A budget with `laterYears` of later life at 15, and the post-Gauntlet block given. */
function lifeStage(
  laterYears: number,
  apprenticeshipXp: number,
  post: { years: number; points: number; levels: number } = { years: 0, points: 0, levels: 0 },
): LifeStageBudget {
  return {
    childhood_native_xp: 75,
    childhood_spread_xp: 45,
    later_life_years: laterYears,
    later_life_rate: 15,
    later_life_xp: laterYears * 15,
    apprenticeship_years: apprenticeshipXp > 0 ? 15 : 0,
    apprenticeship_xp: apprenticeshipXp,
    gauntlet_age: 25,
    post_gauntlet_years: post.years,
    post_gauntlet_points: post.points,
    post_gauntlet_spell_levels: post.levels,
    post_gauntlet_xp: post.points - post.levels,
    truncated_training_years: 0,
    truncated_training_xp: 0,
    truncated_training_spell_levels: 0,
    truncated_training_post_span_years: 0,
    truncated_training_post_span_xp: 0,
  };
}

function setEffective(budget: LifeStageBudget, restricted: EffectiveScores['restricted_xp_pools']) {
  store.effective = {
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
    xp_total_demand: 0,
    xp_general_used: 0,
    xp_general_pool: budget.apprenticeship_xp || budget.later_life_xp,
    xp_max_flow: 0,
    restricted_xp_pools: restricted,
    life_stage: budget,
  } as unknown as EffectiveScores;
}

/** A magus's later life as the engine emits it: a restricted, Abilities-only pool. */
const laterLifePool: EffectiveScores['restricted_xp_pools'] = [
  {
    amount: 75,
    used: 0,
    categories: ['general', 'academic'],
    origin: { kind: 'life_stage', block: 'later_life' },
  },
];

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

function mountBar(): void {
  app = mount(XpBar, { target });
  flushSync();
}

function chip(testid: string): HTMLElement {
  const el = target.querySelector<HTMLElement>(`[data-testid="${testid}"]`);
  expect(el, `no chip ${testid}`).toBeTruthy();
  return el!;
}

/** The text of the tooltip the chip opens on keyboard focus, isolates stripped. */
function tooltipOnFocus(el: HTMLElement): string {
  el.focus();
  vi.advanceTimersByTime(500);
  flushSync();
  const describedBy = el.getAttribute('aria-describedby');
  expect(describedBy, 'the chip opened no tooltip').toBeTruthy();
  return (document.getElementById(describedBy!)?.textContent ?? '').replace(/[⁦-⁩]/g, '');
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  installGuidedEntity();
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  document.querySelectorAll('.tooltip-pop').forEach((pop) => pop.remove());
  store.effective = null;
  vi.useRealTimers();
});

describe('XpBar chip tooltips carry the calculation (N7/N9)', () => {
  it("puts later life's years × rate in the tooltip of a companion's chip", () => {
    setEffective(lifeStage(20, 0), []);
    mountBar();
    expect(tooltipOnFocus(chip('life-stage-later-life'))).toBe('20 × 15 = 300 XP');
  });

  it("puts later life's years × rate in the tooltip of a magus's restricted chip", () => {
    setEffective(lifeStage(5, 240), laterLifePool);
    mountBar();
    expect(tooltipOnFocus(chip('life-stage-later-life'))).toBe('5 × 15 = 75 XP');
  });

  it('puts the post-Gauntlet points and the lab deduction in that chip’s tooltip', () => {
    // 30 years × 30 = 900, less 5 charged lab seasons × 10 = 850 points.
    setEffective(lifeStage(5, 240, { years: 30, points: 850, levels: 120 }), laterLifePool);
    mountBar();
    expect(tooltipOnFocus(chip('life-stage-post-gauntlet'))).toBe(
      '30 × 30 - 50 for lab work = 850 points',
    );
  });

  it('reads the per-year rate off the ruleset rather than a literal 30', () => {
    installRuleset(20);
    // 30 years × 20 = 600, less 50 for lab work = 550 points.
    setEffective(lifeStage(5, 240, { years: 30, points: 550, levels: 0 }), laterLifePool);
    mountBar();
    expect(tooltipOnFocus(chip('life-stage-post-gauntlet'))).toBe(
      '30 × 20 - 50 for lab work = 550 points',
    );
  });

  it('localizes both tooltips to German', () => {
    store.lang = 'de';
    setEffective(lifeStage(5, 240, { years: 30, points: 850, levels: 120 }), laterLifePool);
    mountBar();
    expect(tooltipOnFocus(chip('life-stage-later-life'))).toBe('5 × 15 = 75 EP');
    // Focus moving on closes the first popup (`focusout`), so the next read is clean.
    expect(tooltipOnFocus(chip('life-stage-post-gauntlet'))).toBe(
      '30 × 30 - 50 für Laborarbeit = 850 Punkte',
    );
  });
});
