import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

// The spell-budget bar reads the shared store singleton (entity + engine-derived
// effective scores) and the Fluent bundle. The store schedules a debounced
// revalidate over the Tauri IPC bridge; mock the bridge so nothing reaches a
// backend. Mirrors XpBar.test.ts — the two bars are the same shape.
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
import SpellBudgetBar from './SpellBudgetBar.svelte';

/** A minimal localized ruleset: the bar only needs the advancement table. */
function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      // Ability advancement table, which prices a Mastery Ability score.
      advancement: [
        { score: 1, total_xp: 5 },
        { score: 2, total_xp: 15 },
      ],
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: 11,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
}

/** Install the engine-derived spell-levels budget figures. */
function setEffective(used: number, budget: number, extra: Partial<EffectiveScores> = {}): void {
  store.effective = {
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
    spell_levels_used: used,
    spell_levels_budget: budget,
    spell_levels_profile_base: 120,
    ...extra,
  } as unknown as EffectiveScores;
}

/** Render the bar to an HTML string (node env, no DOM). */
function html(): string {
  return render(SpellBudgetBar, { props: {} }).body;
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
  installRuleset();
  resetEntity();
  setEffective(30, 120);
});

describe('SpellBudgetBar layout (mirrors the XP pool bar)', () => {
  it('renders the base as the bracketed editable total, like the XP pool input', () => {
    store.entity.spell_levels_override = 150;
    const { open } = element(html(), 'spell-levels-base');
    expect(open).toMatch(/<input/i);
    expect(open).toMatch(/value="150"/);
    // The default is data-driven: the type profile's base, never a UI literal.
    expect(open).toMatch(/placeholder="120"/);
  });

  it('renders the read-only used figure on the spell-levels-used testid', () => {
    setEffective(45, 120);
    const { text } = element(html(), 'spell-levels-used');
    expect(text).toContain('45');
  });

  it('renders an Available line with the remaining levels', () => {
    setEffective(45, 120);
    const { text } = element(html(), 'spell-levels-available');
    expect(text).toContain('Available');
    expect(text).toContain('75');
  });
});

describe('SpellBudgetBar V/F bonus (spent first, like a restricted XP pool)', () => {
  it('lists a positive bonus as a used/amount entry and charges it before the base', () => {
    // Skilled Parens: base 120 + 30 = a 150 budget, of which 45 is used. The bonus
    // covers its 30 first, so only 15 lands on the base.
    setEffective(45, 150, { spell_levels_bonus: 30 } as Partial<EffectiveScores>);
    const body = html();
    const { text } = element(body, 'spell-levels-bonus');
    expect(text).toContain('30');
    // The used figure is the BASE charge, and Available closes against it: 120 - 15.
    expect(element(body, 'spell-levels-used').text).toContain('15');
    expect(element(body, 'spell-levels-available').text).toContain('105');
  });

  it('keeps unspent bonus levels in the bonus entry rather than in Available', () => {
    // Only 10 of the 30 bonus levels are used, so nothing is charged to the base
    // and Available is the untouched base — the same way unspent restricted XP is
    // not counted in the XP bar's Available.
    setEffective(10, 150, { spell_levels_bonus: 30 } as Partial<EffectiveScores>);
    const body = html();
    expect(element(body, 'spell-levels-used').text).toContain('0');
    expect(element(body, 'spell-levels-available').text).toContain('120');
    expect(element(body, 'spell-levels-bonus').text).toContain('10');
  });

  it('charges a negative modifier (Weak Parens) to the base, keeping the line closed', () => {
    // Base 120 - 30 = a 90 budget with 45 levels chosen: the penalty takes the
    // first 30 off the base, so used reads 75 and Available 45 (= 90 - 45).
    setEffective(45, 90, { spell_levels_bonus: -30 } as Partial<EffectiveScores>);
    const body = html();
    const { text } = element(body, 'spell-levels-bonus');
    // ASCII hyphen-minus, never U+2212.
    expect(text).toContain('-30');
    expect(text).not.toContain('−');
    expect(element(body, 'spell-levels-used').text).toContain('75');
    expect(element(body, 'spell-levels-available').text).toContain('45');
  });

  it('omits the bonus entry when no V/F touches the spell-levels budget', () => {
    setEffective(45, 120, { spell_levels_bonus: 0 } as Partial<EffectiveScores>);
    expect(() => element(html(), 'spell-levels-bonus')).toThrow();
  });
});

describe('SpellBudgetBar post-Gauntlet levels (read-only, already earned)', () => {
  it('lists the levels a magus past its Gauntlet bought and keeps the base at the profile figure', () => {
    // 35 years a magus with 300 of the points taken as levels of spells: the
    // engine's budget is 120 + 300, so the base field still means the 120.
    setEffective(0, 420, { spell_levels_life_stage: 300 } as Partial<EffectiveScores>);
    const body = html();
    expect(element(body, 'spell-levels-post-gauntlet').text).toContain('300');
    // The editable base is untouched: its placeholder is still the profile's 120.
    expect(element(body, 'spell-levels-base').open).toMatch(/placeholder="120"/);
    // Earned levels are spendable, so Available is the whole budget.
    expect(element(body, 'spell-levels-used').text).toContain('0');
    expect(element(body, 'spell-levels-available').text).toContain('420');
  });

  it('charges a spend against the base and the post-Gauntlet levels as one', () => {
    setEffective(150, 420, { spell_levels_life_stage: 300 } as Partial<EffectiveScores>);
    const body = html();
    expect(element(body, 'spell-levels-used').text).toContain('150');
    expect(element(body, 'spell-levels-available').text).toContain('270');
  });

  it('shows base, V/F bonus and post-Gauntlet levels as three separate parts', () => {
    // Skilled Parens 30 on top of 120 + 300 = the engine's 450 budget, 45 spent.
    setEffective(45, 450, {
      spell_levels_bonus: 30,
      spell_levels_life_stage: 300,
    } as Partial<EffectiveScores>);
    const body = html();
    expect(element(body, 'spell-levels-base').open).toMatch(/placeholder="120"/);
    expect(element(body, 'spell-levels-bonus').text).toContain('30');
    expect(element(body, 'spell-levels-post-gauntlet').text).toContain('300');
    // The bonus is still spent first, so only 15 lands on the unconditional side.
    expect(element(body, 'spell-levels-used').text).toContain('15');
    expect(element(body, 'spell-levels-available').text).toContain('405');
  });

  it('omits the line for a magus standing at its Gauntlet', () => {
    // Gated on the number alone, exactly like the XP bar's life-stage lines — no
    // is_magus test, no type branch — so this also covers every non-magus.
    setEffective(45, 120, { spell_levels_life_stage: 0 } as Partial<EffectiveScores>);
    expect(() => element(html(), 'spell-levels-post-gauntlet')).toThrow();
  });

  it('omits the line when the engine reports no life-stage figure at all', () => {
    setEffective(45, 120);
    expect(() => element(html(), 'spell-levels-post-gauntlet')).toThrow();
  });
});

describe('SpellBudgetBar overspend styling (same behavior as the XP pool)', () => {
  it('marks Available with the over class and shows a negative value when overspent', () => {
    setEffective(130, 120); // available = -10
    const { open, text } = element(html(), 'spell-levels-available');
    // over drives the bold-red styling on the label and the value.
    expect(open).toMatch(/class="[^"]*\bover\b[^"]*"/);
    expect(text).toContain('Available');
    // ASCII hyphen-minus, never U+2212.
    expect(text).toContain('-10');
    expect(text).not.toContain('−');
  });

  it('marks the used figure with the over-value class when overspent', () => {
    setEffective(130, 120);
    const { open, text } = element(html(), 'spell-levels-used');
    expect(open).toMatch(/class="[^"]*\bover-value\b[^"]*"/);
    expect(text).toContain('130');
  });

  it('marks neither the used figure nor Available when within budget', () => {
    setEffective(30, 120);
    expect(element(html(), 'spell-levels-used').open).not.toMatch(/\bover-value\b/);
    expect(element(html(), 'spell-levels-available').open).not.toMatch(/\bover\b/);
  });
});

// GF2 (tmp/review/review-round-2-gerda-frontend.md): XpBar.svelte gained a
// semantic `data-overspent` attribute in round 1 specifically so e2e specs
// stop coupling to a CSS class name (see XpBar.test.ts). SpellBudgetBar has
// the identical overspend concept on the identical shape (derive.ts documents
// spellLevelAllocation/generalXpAllocation as deliberate "twins") but never
// received it — this closes that gap so the two bars are consistent.
describe('SpellBudgetBar semantic overspend attribute (mirrors XpBar, GF2)', () => {
  it('marks Available with a semantic data-overspent attribute when overspent', () => {
    setEffective(130, 120); // available = -10
    const { open } = element(html(), 'spell-levels-available');
    expect(open).toMatch(/data-overspent="true"/);
  });

  it('marks the used figure with a semantic data-overspent attribute when overspent', () => {
    setEffective(130, 120);
    const { open } = element(html(), 'spell-levels-used');
    expect(open).toMatch(/data-overspent="true"/);
  });

  it('reports data-overspent="false" on both figures when within budget', () => {
    setEffective(30, 120);
    expect(element(html(), 'spell-levels-used').open).toMatch(/data-overspent="false"/);
    expect(element(html(), 'spell-levels-available').open).toMatch(/data-overspent="false"/);
  });

  it('marks the mastery read-out with a semantic data-overspent attribute when mastery XP is overspent', () => {
    // Mastery score 2 costs table(2) = 15 against a 10-point pool.
    setEffective(30, 120, { spell_mastery_xp: 10 } as Partial<EffectiveScores>);
    store.entity.spells = [{ spell: 'spell.a', mastery: 2 }];
    const { open } = element(html(), 'spell-mastery-info');
    expect(open).toMatch(/data-overspent="true"/);
  });

  it('reports data-overspent="false" on the mastery read-out when within its pool', () => {
    setEffective(30, 120, { spell_mastery_xp: 50 } as Partial<EffectiveScores>);
    store.entity.spells = [{ spell: 'spell.a', mastery: 2 }];
    const { open } = element(html(), 'spell-mastery-info');
    expect(open).toMatch(/data-overspent="false"/);
  });
});

describe('SpellBudgetBar mastery read-out', () => {
  it('shows the mastery pool when the character has mastery XP', () => {
    setEffective(30, 120, { spell_mastery_xp: 20 } as Partial<EffectiveScores>);
    const { text } = element(html(), 'spell-mastery-info');
    expect(text).toContain('20');
  });

  it('omits the mastery read-out when there is no pool and no floor', () => {
    setEffective(30, 120);
    expect(() => element(html(), 'spell-mastery-info')).toThrow();
  });
});
