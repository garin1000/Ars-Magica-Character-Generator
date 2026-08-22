import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, Entity, LocalizedRuleset } from '../types';

// The panel reads the shared store singleton (entity + engine-derived totals)
// and the Fluent bundle. The store schedules a debounced revalidate over the
// Tauri IPC bridge; mock the bridge so nothing reaches a backend. Mirrors
// SpellBudgetBar.test.ts / XpBar.test.ts.
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
  };
}

/** A minimal, complete DerivedTotals fixture; overrides layer on top. */
function derivedFixture(overrides: Partial<DerivedTotals> = {}): DerivedTotals {
  return {
    is_magus: true,
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

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(DerivedTotalsPanel, { props: {} }).body;
}

/** The text content of the single element carrying `testid`, tags stripped. */
function textOf(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i').exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[1]
    .replace(/<[^>]*>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('DerivedTotalsPanel longevity read-out', () => {
  // E4 (round-1 audit): the panel negates the stored bonus into an aging-roll
  // modifier and documents that a zero bonus must read "0", never "-0" — this
  // is the load-bearing call site formatSigned(-0) exists to protect.
  it('shows a zero longevity bonus as "0", never "-0"', () => {
    store.derived = derivedFixture({
      longevity: { source: 'self_made', bonus: 0, entered: true, bronze_cord: 0 },
    });
    const text = textOf(html(), 'derived-longevity');
    expect(text).toContain('0');
    expect(text).not.toContain('-0');
  });

  it('negates a positive stored bonus into a negative aging-roll modifier with an ASCII hyphen', () => {
    store.derived = derivedFixture({
      longevity: { source: 'self_made', bonus: 10, entered: true, bronze_cord: 0 },
    });
    const text = textOf(html(), 'derived-longevity');
    expect(text).toContain('-10');
    // ASCII hyphen-minus (U+002D), never the mathematical minus (U+2212).
    expect(text).not.toContain('−');
  });

  it('shows the not-entered hint when no ritual has been recorded yet', () => {
    store.derived = derivedFixture({
      longevity: { source: 'external', bonus: 0, entered: false, bronze_cord: 0 },
    });
    const text = textOf(html(), 'derived-longevity');
    expect(text).toContain(store.t('derived-longevity-not-entered'));
  });
});

describe('DerivedTotalsPanel loading state', () => {
  it('shows a loading placeholder before the engine totals arrive', () => {
    store.derived = null;
    const body = html();
    expect(body).toContain(store.t('loading'));
  });
});
