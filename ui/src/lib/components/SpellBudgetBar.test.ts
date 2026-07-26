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
  it('renders the budget as the editable override input, with the profile base as placeholder', () => {
    store.entity.spell_levels_override = 150;
    const { open } = element(html(), 'spell-levels-override');
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

  it('renders the engine-effective budget as a read-only figure, not the override input', () => {
    // The effective budget (base + virtue bonuses, e.g. Skilled Parens +30) is
    // engine-owned, so it is displayed rather than edited; only its base is.
    store.entity.spell_levels_override = 80;
    setEffective(45, 110);
    const { open, text } = element(html(), 'spell-levels-budget');
    expect(text).toContain('110');
    expect(open).not.toMatch(/<input/i);
  });

  it('renders an Available line with the remaining levels', () => {
    setEffective(45, 120);
    const { text } = element(html(), 'spell-levels-available');
    expect(text).toContain('Available');
    expect(text).toContain('75');
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
