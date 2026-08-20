import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// The Details tab reads the shared store singleton (ruleset + entity + the
// engine-derived scores) and the Fluent bundle. The store schedules a debounced
// revalidate over the Tauri IPC bridge; mock the bridge so nothing reaches a
// backend. Harness mirrors AgingStep.test.ts.
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
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import CharacterDetails from './CharacterDetails.svelte';

/**
 * A minimal localized ruleset carrying one profile per capability the tab reads.
 * `is_magus` is the flag every gate keys off — never the type id — so a future
 * spell-casting type gets the same treatment without a code change.
 */
function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {
        grog: { id: 'grog', budget: { virtue_points: 3, flaw_points: 3 } },
        magus: { id: 'magus', budget: { virtue_points: 10, flaw_points: 10 }, is_magus: true },
      },
      abilities: {},
      arts: {},
      houses: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(typeId: string): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: typeId,
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
  store.effective = null;
  store.derived = null;
}

/** Render the tab to an HTML string (node env, no DOM). */
function html(): string {
  return render(CharacterDetails, { props: {} }).body;
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity('grog');
});

// "You can perform Longevity Rituals for others, even for non-magi."
// Source: Ars Magica - Definitive Edition (Core Rules).md:10672.
//
// The editor's only home for a ritual was the Possessions tab, which is
// magus-gated (App.svelte), so a grog or companion holding an externally-made
// ritual could enter its bonus in the guided aging step and then never see it
// again in the editor — while the bonus is a term of its aging total.
describe('CharacterDetails and the Longevity Ritual (slice 6b8c)', () => {
  it('offers a non-magus the ritual beside its aging surface', () => {
    const body = html();
    expect(has(body, 'aging-panel')).toBe(true);
    expect(has(body, 'longevity-add')).toBe(true);
  });

  it('takes the bonus and the source of a ritual made for a grog', () => {
    store.entity.longevity_ritual = { source: 'external', bonus: 4, focus: '' };
    const body = html();
    expect(has(body, 'longevity-bonus')).toBe(true);
    expect(has(body, 'longevity-source-external')).toBe(true);
    // Only the Creo Corpus SUGGESTION is magus-gated (derived.rs); a grog gets
    // no hint, and none is invented here.
    expect(has(body, 'longevity-hint')).toBe(false);
  });

  it('leaves a magus its one existing home on the Possessions tab', () => {
    // Two on-screen homes for one ritual in the same view is the trap the guided
    // step's own comment names; a magus keeps the Possessions tab and this tab
    // stays out of it.
    resetEntity('magus');
    expect(has(html(), 'longevity-add')).toBe(false);
  });
});
