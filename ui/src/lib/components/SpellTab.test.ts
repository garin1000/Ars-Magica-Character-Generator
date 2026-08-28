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
      ritual_min_level: 20,
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
    ability_funding: 'pool',
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

// --- 6b8c: who owns the budget bar ------------------------------------------

// Every other input surface takes its budget bar from whoever mounts it — the
// editor's tab in `App.svelte`, the wizard's step through `WizardStep`'s phase
// table. The Spells surface used to be the one exception, mounting
// `SpellBudgetBar` itself, which made the step table's `bar` column read "—" for
// `spells` alone and hid a whole-character budget inside the picker.
describe('SpellTab budget-bar ownership (slice 6b8c)', () => {
  it('mounts no budget bar of its own', () => {
    const body = html();
    expect(body).not.toContain('data-testid="spell-levels-used"');
    expect(body).not.toContain('data-testid="spell-levels-available"');
  });

  it('still renders both picker regions', () => {
    const body = html();
    expect(body).toContain('data-testid="available-title"');
    expect(body).toContain('data-testid="spell-list"');
  });
});

// VA2 (tmp/review/review-round-1-viktor-app.md): the Ritual level floor is read
// from the engine-surfaced `ruleset.ritual_min_level` (mirrors
// `crates/arm-rules/src/spell.rs`'s `RITUAL_MIN_LEVEL` constant), not a local
// literal. The fixture's `ritual_min_level: 20` above matches the shipped
// engine value, so most of these pin the real threshold; the dedicated
// "sources the ritual floor from the engine" test below sets a
// non-canonical value (25) specifically to prove the component tracks the
// engine's number rather than a hardcoded 20 that would happen to agree with
// it.
describe('SpellTab ritual minimum learnable level (VA2)', () => {
  const RITUAL = 'spell.test_general_ritual';
  const ORDINARY = 'spell.test_general_ordinary';

  function installGeneralSpells(): void {
    store.ruleset!.ruleset.spells = {
      ...store.ruleset!.ruleset.spells,
      [RITUAL]: { id: RITUAL, technique: 'art.creo', form: 'art.animal', ritual: true },
      [ORDINARY]: { id: ORDINARY, technique: 'art.creo', form: 'art.animal' },
    };
    store.ruleset!.i18n[RITUAL] = { name: 'Test General Ritual' };
    store.ruleset!.i18n[ORDINARY] = { name: 'Test General Ordinary' };
  }

  it('blocks a General Ritual when fewer than 20 levels remain', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 100;
    store.effective!.spell_levels_budget = 119; // remaining = 19
    const body = html();
    expect(outer(body, `add-${RITUAL}`)).toMatch(/aria-disabled="true"/);
  });

  it('allows a General Ritual once exactly 20 levels remain', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 100;
    store.effective!.spell_levels_budget = 120; // remaining = 20
    const body = html();
    expect(outer(body, `add-${RITUAL}`)).toMatch(/aria-disabled="false"/);
  });

  it('allows an ordinary General spell with only 1 level remaining', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 119;
    store.effective!.spell_levels_budget = 120; // remaining = 1
    const body = html();
    expect(outer(body, `add-${ORDINARY}`)).toMatch(/aria-disabled="false"/);
  });

  it('blocks an ordinary General spell with 0 levels remaining', () => {
    installGeneralSpells();
    store.effective!.spell_levels_used = 120;
    store.effective!.spell_levels_budget = 120; // remaining = 0
    const body = html();
    expect(outer(body, `add-${ORDINARY}`)).toMatch(/aria-disabled="true"/);
  });

  it('sources the ritual floor from the engine, not a local constant', () => {
    installGeneralSpells();
    // A non-canonical floor: if the component still hardcoded 20, 24
    // remaining levels would satisfy it and this would render enabled.
    store.ruleset!.ruleset.ritual_min_level = 25;
    store.effective!.spell_levels_used = 100;
    store.effective!.spell_levels_budget = 124; // remaining = 24, below the engine's 25
    expect(outer(html(), `add-${RITUAL}`)).toMatch(/aria-disabled="true"/);

    store.effective!.spell_levels_budget = 125; // remaining = 25, meets the engine's floor
    expect(outer(html(), `add-${RITUAL}`)).toMatch(/aria-disabled="false"/);
  });
});
