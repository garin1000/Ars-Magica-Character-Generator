import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { CombatLine, Entity, LocalizedRuleset } from '../types';

// The section reads the shared store singleton (ruleset i18n + the Fluent
// bundle). The store schedules a debounced revalidate over the Tauri IPC
// bridge; mock the bridge so nothing reaches a backend. Mirrors
// DerivedTotalsPanel.test.ts.
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
import DerivedCombatSection from './DerivedCombatSection.svelte';

function installRuleset(): void {
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
    },
    i18n: { 'equipment.long_sword': { name: 'Long Sword' } },
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
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

/** A combat line whose three optional cells are all absent. */
function unarmed(): CombatLine {
  return {
    weapon: 'equipment.long_sword',
    ability: 'ability.single_weapon',
    initiative: 2,
    attack: null,
    defense: 5,
    damage: null,
    range: null,
  };
}

/** Render the section to an HTML string (node env, no DOM). */
function html(lines: CombatLine[]): string {
  return render(DerivedCombatSection, {
    props: { d: { combat: lines } as never },
  }).body;
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// Sabine 7 (round-1 audit): the three optional cells rendered a literal em dash
// as the app's "not applicable" wording — a user-facing string with no Fluent
// key, which CLAUDE.md forbids outright, and the only such site in the tree. It
// is also an accessibility defect: NVDA announces a bare em dash as nothing, so
// a blind user cannot tell an N/A cell from one that failed to render.
describe('DerivedCombatSection — the not-applicable cell (Sabine 7)', () => {
  it.each(['en', 'de'] as const)('renders the localized N/A wording, not a dash (%s)', (lang) => {
    store.lang = lang;
    const body = html([unarmed()]);
    const notApplicable = store.t('derived-not-applicable');
    // The key must resolve — `translate` echoes the key back otherwise.
    expect(notApplicable).not.toBe('derived-not-applicable');
    // Attack, Damage and Range are all absent on this line.
    expect(body.split(notApplicable).length - 1).toBe(3);
    expect(body).not.toContain('—');
  });

  // German convention for `n/a` is `n/v`
  // (rules/source/de/translation-tables/uebersetzungsregeln.md).
  it('uses the German n/v convention', () => {
    store.lang = 'de';
    expect(store.t('derived-not-applicable')).toBe('n/v');
  });

  it('still shows a present value rather than the placeholder', () => {
    const body = html([{ ...unarmed(), attack: 7, damage: 9, range: 0 }]);
    expect(body).not.toContain(store.t('derived-not-applicable'));
  });
});

// Sabine 9 (round-1 audit), this table only: it has a header ROW and a header
// COLUMN, so with no `scope` the cell→header association is not inferable and a
// screen reader reads bare numbers.
describe('DerivedCombatSection — table semantics (Sabine 9)', () => {
  it('scopes every column header to its column and every row header to its row', () => {
    const body = html([unarmed()]);
    // `<th`, not `<thead`: the lookahead keeps the section's own `<thead>` out.
    const headers = body.match(/<th(?=[\s>])[^>]*>/g) ?? [];
    // Five labelled column headers plus the empty corner cell, then one row
    // header per combat line.
    expect(headers.length).toBe(7);
    expect(headers.filter((h) => h.includes('scope="col"')).length).toBe(6);
    expect(headers.filter((h) => h.includes('scope="row"')).length).toBe(1);
    expect(headers.every((h) => h.includes('scope='))).toBe(true);
  });
});
