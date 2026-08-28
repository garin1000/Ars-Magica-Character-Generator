import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// The panel is a composition over the shared store singleton. The store schedules
// a debounced revalidate over the Tauri IPC bridge; mock the bridge so nothing
// reaches a backend. Harness mirrors AgingStep.test.ts.
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
import AgingPanel from './AgingPanel.svelte';

/** One profile per capability the aging surfaces read; `is_magus` is the flag. */
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
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
  store.derived = null;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(AgingPanel, { props: {} }).body;
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

// Slice 3 (#28). The ritual's bonus is a term of the AGING TOTAL
// (Ars Magica - Definitive Edition (Core Rules).md:16567-16569), so its home is
// the aging surface. It used to be mounted by `AgingStep` instead of here, and
// separately by `MagicPossessions`, purely because the editor had no aging tab:
// putting it inside this panel would then have shown it on the editor's Details
// tab as well, giving a magus two on-screen homes for one ritual. The Aging tab
// removes that constraint, so the panel owns it — once, for every surface.
describe('AgingPanel and the Longevity Ritual', () => {
  it('renders the Longevity panel', () => {
    const body = html();
    expect(has(body, 'aging-panel')).toBe(true);
    expect(has(body, 'longevity-add')).toBe(true);
  });

  it('renders it for a magus too, which is now its only home', () => {
    // "You can perform Longevity Rituals for others, even for non-magi."
    // Source: Ars Magica - Definitive Edition (Core Rules).md:10672 — the panel
    // is not type-gated, and with the Possessions mount gone this is the one
    // place a magus edits its ritual.
    resetEntity('magus');
    expect(has(html(), 'longevity-add')).toBe(true);
  });

  it('takes the bonus and the source of an existing ritual', () => {
    store.entity.longevity_ritual = { source: 'external', bonus: 4, focus: '' };
    const body = html();
    expect(has(body, 'longevity-bonus')).toBe(true);
    expect(has(body, 'longevity-source-external')).toBe(true);
    // Only the Creo Corpus SUGGESTION is magus-gated (derived.rs); a grog gets no
    // hint, and none is invented here.
    expect(has(body, 'longevity-hint')).toBe(false);
  });

  it('keeps the record of what aging has already done', () => {
    // The ritual joins the panel; it does not displace what was already there.
    // The schedule, the conditions and the roll all gate on engine read-outs this
    // fixture has none of, so the record is the surface that always stands.
    expect(has(html(), 'aging-record')).toBe(true);
  });
});
