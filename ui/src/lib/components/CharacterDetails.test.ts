import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
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
    ability_funding: 'pool',
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

/** `app.css` as text, for the grid-spanning rule server-rendered markup cannot reveal. */
const appCss = readFileSync(fileURLToPath(new URL('../../app.css', import.meta.url)), 'utf-8');

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity('grog');
});

// Slice 3 (#28). The tab used to carry everything that had no other home: the
// aging cluster, the Longevity Ritual for a type with no Possessions tab, the
// Personality Traits and the Reputations. Each of those mirrors a wizard phase and
// now has its own tab, so Details keeps what does NOT — identity, the age the
// concept states, and the Warping/Twilight cluster no `CreationPhase` maps to.
describe('CharacterDetails after the tab split', () => {
  it('no longer carries the aging cluster or the Longevity Ritual', () => {
    const body = html();
    expect(has(body, 'aging-panel')).toBe(false);
    expect(has(body, 'aging-record')).toBe(false);
    expect(has(body, 'longevity-add')).toBe(false);
  });

  it('no longer carries Personality Traits or Reputations', () => {
    const body = html();
    expect(has(body, 'personality-add')).toBe(false);
    expect(has(body, 'reputation-empty')).toBe(false);
  });

  it('keeps a magus out of it just the same', () => {
    // The old arrangement gated the ritual on `!is_magus` so a magus would not get
    // two homes for one. With the panel gone from here the gate goes too, and
    // neither type sees a ritual on this tab.
    resetEntity('magus');
    expect(has(html(), 'longevity-add')).toBe(false);
  });

  it('keeps identity, age and the Warping/Twilight cluster', () => {
    const body = html();
    expect(has(body, 'identity-concept')).toBe(true);
    expect(has(body, 'age-input')).toBe(true);
    expect(has(body, 'warping-points-input')).toBe(true);
    expect(has(body, 'twilight-scars-list')).toBe(true);
  });
});

// S17 (full-audit a11y): IdentityFields, and the Warping/Twilight h3s further
// down, all opened directly under the app's single <h1> with no <h2> between
// (a heading hierarchy gap). The Details tab now gets its own <h2> (reusing
// `tab-details`, the same label App.svelte's tab button already carries).
//
// This section is a CSS GRID (`.character-details`, auto-fit columns) where
// every direct child is its own cell — a plain h2 would land in a single
// column-width cell instead of reading as a banner over the whole panel, so it
// carries a class the grid spans full width (`grid-column: 1 / -1`), the same
// technique `.aging-log-block` already uses for the same reason.
describe('CharacterDetails heading hierarchy (S17)', () => {
  it('opens with an h2 naming the tab, ahead of every h3 subsection', () => {
    const body = html();
    const h2 = body.indexOf('<h2');
    const h3 = body.indexOf('<h3');
    expect(h2).toBeGreaterThanOrEqual(0);
    expect(h3).toBeGreaterThan(h2);
    expect(body).toMatch(/<h2[^>]*>Details<\/h2>/);
  });

  it('spans the h2 across every column of the character-details grid', () => {
    const body = html();
    const h2Tag = /<h2[^>]*>/.exec(body)![0];
    const cls = /class="([^"]*)"/.exec(h2Tag)?.[1] ?? '';
    expect(cls).not.toBe('');
    for (const token of cls.split(/\s+/)) {
      const rule = new RegExp(
        `\\.character-details\\s+\\.${token}\\s*\\{[^}]*grid-column:\\s*1\\s*/\\s*-1`,
      ).exec(appCss);
      if (rule) return;
    }
    throw new Error(`no grid-column: 1 / -1 rule found for classes "${cls}" in app.css`);
  });
});
