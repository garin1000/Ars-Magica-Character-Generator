import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

// The Spells tab reads the shared store singleton (ruleset catalogue + entity +
// engine-derived effective scores) and the Fluent bundle. The store schedules a
// debounced revalidate over the Tauri IPC bridge; mock the bridge so nothing
// reaches a backend. Harness mirrors SpellBudgetBar.test.ts.
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
import SpellTab from './SpellTab.svelte';

const SPELL = 'spell.true_rest_of_the_injured_brute';

/** A minimal localized ruleset: two Arts, one catalogue spell, the Ability
 * advancement table (which prices a Mastery score) and one mastery ability. */
function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      advancement: [
        { score: 1, total_xp: 5 },
        { score: 2, total_xp: 15 },
      ],
      arts: {
        'art.creo': { id: 'art.creo', art_type: 'technique' },
        'art.animal': { id: 'art.animal', art_type: 'form' },
      },
      spells: {
        [SPELL]: { id: SPELL, technique: 'art.creo', form: 'art.animal', level: 20 },
      },
      spell_mastery_abilities: {
        'spell_mastery_ability.quick_casting': {
          id: 'spell_mastery_ability.quick_casting',
          repeatable: true,
        },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      'art.creo': { name: 'Creo', abbreviation: 'Cr' },
      'art.animal': { name: 'Animal', abbreviation: 'An' },
      [SPELL]: { name: 'True Rest of the Injured Brute' },
      'spell_mastery_ability.quick_casting': { name: 'Quick Casting' },
    },
  } as unknown as LocalizedRuleset;
}

/** A magus knowing the one catalogue spell, mastered at 1 (so the mastery
 * spinner AND the special-ability picker both render on the row). */
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
    spells: [{ spell: SPELL, mastery: 1 }],
  };
  store.effective = {
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
    spell_levels_used: 20,
    spell_levels_budget: 120,
    spell_levels_profile_base: 120,
  } as unknown as EffectiveScores;
  store.result = null;
}

/** Render the tab to an HTML string (node env, no DOM). */
function html(): string {
  return render(SpellTab, { props: {} }).body;
}

/** The full outer HTML of the element carrying a data-testid — nesting-aware, so
 * a container's own closing tag is not mistaken for a nested child's. */
function outer(body: string, testid: string): string {
  const open = new RegExp(`<([a-z]+)[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!open) throw new Error(`no element with data-testid="${testid}"`);
  const tags = new RegExp(`</?${open[1]}\\b[^>]*>`, 'gi');
  tags.lastIndex = open.index;
  let depth = 0;
  let tag: RegExpExecArray | null;
  while ((tag = tags.exec(body))) {
    depth += tag[0].startsWith('</') ? -1 : 1;
    if (depth === 0) return body.slice(open.index, tag.index + tag[0].length);
  }
  throw new Error(`unclosed element with data-testid="${testid}"`);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// Spell names are long ("True Rest of the Injured Brute (CrAn 20)") and the row
// also carries a mastery spinner and a special-ability picker. Side by side the
// three squeeze the name down to one word per line, so the two mastery controls
// share ONE stacked column and the name keeps the rest of the row's width.
describe('SpellTab selected-row layout', () => {
  it('stacks the mastery score and the mastery special abilities in one block', () => {
    const block = outer(html(), `spell-mastery-block-${SPELL}-0`);
    expect(block).toContain(`data-testid="spell-mastery-${SPELL}-0"`);
    expect(block).toContain(`data-testid="spell-mastery-abilities-${SPELL}-0"`);
  });

  it('keeps the spell name outside the mastery block, ahead of it in the row', () => {
    const body = html();
    expect(outer(body, `spell-mastery-block-${SPELL}-0`)).not.toContain('spell-name-');
    expect(body.indexOf(`data-testid="spell-name-${SPELL}-0"`)).toBeLessThan(
      body.indexOf(`data-testid="spell-mastery-block-${SPELL}-0"`),
    );
  });
});
