import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { AgingReadout, EffectiveScores, Entity, LocalizedRuleset } from '../types';

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

/**
 * Everything the four gated blocks need before they render: a Living Conditions
 * table on the ruleset and an aging read-out (with owed years) on the effective
 * scores. Without it only the record and the ritual stand, which is enough for the
 * tests above but not for a statement about the surface's ORDER.
 */
function installAgingSurface(): void {
  const localized = store.ruleset!;
  localized.ruleset.aging = {
    start_age: 35,
    age_divisor: 10,
    apparent_age_increase_min: 3,
    living_conditions: [
      { id: 'living_condition.castle', modifier: 1, cumulative: false },
      { id: 'living_condition.leper_colony', modifier: -1, cumulative: true },
    ],
    outcomes: [],
    crisis: { rows: [], die: { min: 1, max: 10 } },
  } as unknown as NonNullable<LocalizedRuleset['ruleset']['aging']>;
  localized.i18n['living_condition.castle'] = { name: 'Live in a castle' };
  localized.i18n['living_condition.leper_colony'] = { name: 'Live in a leper colony' };

  const aging: AgingReadout = {
    first_roll_age: 36,
    begins_after_age: 35,
    schedule: [{ age: 36, year: null, recorded: false }],
    rolls_owed: 1,
    rolls_recorded: 0,
    age_modifier: 4,
    living_conditions_modifier: 0,
    longevity_modifier: 0,
    trait_modifier: 0,
    longevity_clamp_active: false,
    fixed_total: 4,
  };
  store.effective = { ...(store.effective ?? {}), aging } as unknown as EffectiveScores;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(AgingPanel, { props: {} }).body;
}

/** Where a testid first appears in the markup — i.e. its place in reading order. */
function positionOf(body: string, testid: string): number {
  const at = body.indexOf(`data-testid="${testid}"`);
  expect(at, `no element with data-testid="${testid}"`).toBeGreaterThanOrEqual(0);
  return at;
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

// manual-testing-findings-2026-09-03 #22/#24/#25: the surface read as a band of
// columns with the log orphaned in a full-width row far below the fold, and the
// Longevity Ritual buried past even that — so a player never found either. The
// order a year is actually resolved in is schedule → living conditions → roll →
// log → ritual, and since every block is now a full-width row of the grid (app.css),
// DOM order IS the order on screen and the order the keyboard walks. Asserted here
// rather than in the children because the composition is what owns it.
describe('AgingPanel reading order', () => {
  it('reads schedule, living conditions, roll, log, then the ritual', () => {
    installAgingSurface();
    const body = html();
    const at = (testid: string): number => positionOf(body, testid);

    expect(at('aging-schedule')).toBeLessThan(at('living-conditions'));
    expect(at('living-conditions')).toBeLessThan(at('aging-calculator'));
    expect(at('aging-calculator')).toBeLessThan(at('aging-log-block'));
    // The per-year log comes BEFORE the running totals it explains: the log is what
    // the player has just written to and needs to see without scrolling, while the
    // apparent age and the Aging Points are the engine's own bookkeeping.
    expect(at('aging-log-block')).toBeLessThan(at('apparent-age-input'));
    // And the ritual closes the surface, as the term subtracted from every total.
    expect(at('apparent-age-input')).toBeLessThan(at('longevity-add'));
  });
});

// S17 (full-audit a11y): AgingSchedulePanel, LivingConditionsPicker,
// AgingRollCalculator and AgingRecordPanel — every block this panel composes —
// opened at <h3 class="detail-label"> directly under the app's single <h1>,
// with no <h2> between (a heading hierarchy gap). This shared composition
// mounts on BOTH the editor's Aging tab and the wizard's aging step (the
// file's own comment: "composed once ... so the two flows can never drift"),
// so fixing it here fixes both surfaces in one place. Reuses `tab-aging`, the
// same label App.svelte's tab button already carries.
//
// This mounts inside a `.character-details` CSS GRID (provided by AgingStep /
// App.svelte, both wrapping <AgingPanel /> in `<section class="panel
// character-details">`) where every child is its own cell — the added <h2>
// needs the same full-width span AgingSchedulePanel etc. never needed for
// themselves, since `.character-details .aging-panel { display: contents }`
// promotes THIS panel's children (including the new h2) straight into that
// grid.
describe('AgingPanel heading hierarchy (S17)', () => {
  it('opens with an h2 naming the tab, ahead of every h3 subsection', () => {
    const body = html();
    const h2 = body.indexOf('<h2');
    const h3 = body.indexOf('<h3');
    expect(h2).toBeGreaterThanOrEqual(0);
    // LongevityPanel's own h3 always renders unconditionally, so this holds
    // even with the grog ruleset/entity fixture installed above.
    expect(h3).toBeGreaterThan(h2);
    expect(body).toMatch(/<h2[^>]*>Aging<\/h2>/);
  });

  it('spans the h2 across every column of the character-details grid', () => {
    const body = html();
    const h2Tag = /<h2[^>]*>/.exec(body)![0];
    const cls = /class="([^"]*)"/.exec(h2Tag)?.[1] ?? '';
    expect(cls).not.toBe('');
    let found = false;
    for (const token of cls.split(/\s+/)) {
      const rule = new RegExp(
        `\\.character-details\\s+\\.${token}\\s*\\{[^}]*grid-column:\\s*1\\s*/\\s*-1`,
      ).exec(appCss);
      if (rule) found = true;
    }
    expect(found).toBe(true);
  });
});
