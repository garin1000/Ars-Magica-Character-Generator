import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { DerivedTotals, Entity, LocalizedRuleset } from '../types';

// The addend breakdown only ever reaches the DOM through the `tooltip` action
// (actions.ts), which is entirely client-side: `show()` runs on `focusin`, off
// a `document.createElement` call, so `svelte/server`'s `render()` never
// executes it and a string-based SSR test cannot see the joined text at all —
// this belongs in the `client` project. Harness mirrors SpellTab.client.test.ts.
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
import DerivedTotalsPanel from './DerivedTotalsPanel.svelte';

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
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: 11,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

function derivedFixture(overrides: Partial<DerivedTotals> = {}): DerivedTotals {
  return {
    is_magus: false,
    lab_totals: [],
    casting_totals: [],
    penetration: [],
    magic_resistance: [],
    combat: [],
    soak: { addends: [], total: 0 },
    encumbrance: { load: 0, burden: 0, total: 0 },
    fatigue: [],
    wounds: [],
    size: 0,
    decrepitude_score: 0,
    warping_score: 0,
    warping_points: 0,
    surfaced_modifiers: [],
    ...overrides,
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

// S14 (full-audit i18n): the addend-breakdown tooltip joined its parts with a
// hardcoded `', '` literal instead of routing through Fluent, unlike the
// identical-shape join in derive.ts's `restrictedPoolLabel`
// (`restricted-xp-list-separator`). A hardcoded separator cannot be
// retargeted for a language whose list convention differs.
describe('DerivedTotalsPanel addend breakdown separator (S14)', () => {
  it("joins the soak breakdown through Fluent's derived-addend-list-separator, not a hardcoded ', '", () => {
    store.derived = derivedFixture({
      soak: {
        addends: [
          { label: 'stamina', value: 2 },
          { label: 'encumbrance', value: -1 },
        ],
        total: 1,
      },
    });

    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(DerivedTotalsPanel, { target });
    flushSync();

    const soakDd = target.querySelector('[data-testid="derived-soak"]') as HTMLElement;
    const soakDt = soakDd.previousElementSibling as HTMLElement;
    soakDt.dispatchEvent(new Event('focusin', { bubbles: true }));
    flushSync();

    const tip = document.querySelector('[data-testid="tooltip-text"]');
    expect(tip).not.toBeNull();
    expect(tip!.textContent).toBe(
      `${store.t('derived-addend-stamina')} +2${store.t('derived-addend-list-separator')} ${store.t('derived-addend-encumbrance')} -1`,
    );
  });
});
