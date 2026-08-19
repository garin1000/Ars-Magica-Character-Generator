import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import {
  CHARACTERISTICS,
  type EffectiveScores,
  type Entity,
  type LocalizedRuleset,
} from '../types';

// The panel reads the shared store singleton (the entity's apparent age, aging
// points and aging log, the engine's Decrepitude score) and the Fluent bundle.
// The store schedules a debounced revalidate over the Tauri IPC bridge; mock the
// bridge so nothing reaches a backend. Harness mirrors LifeStagePanel.test.ts.
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
import AgingRecordPanel from './AgingRecordPanel.svelte';

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
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
  store.effective = null;
}

/** The engine's derived Decrepitude score, which the read-out reports. */
function setDecrepitude(score: number): void {
  store.effective = {
    ...(store.effective ?? {}),
    decrepitude_score: score,
  } as unknown as EffectiveScores;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(AgingRecordPanel, { props: {} }).body;
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

/** The visible text of the element carrying a data-testid, tags stripped. */
function text(body: string, testid: string): string {
  const opening = new RegExp(`<([a-z]+)[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!opening) throw new Error(`no element with data-testid="${testid}"`);
  const inner = body.slice(opening.index + opening[0].length);
  const closing = new RegExp(`</${opening[1]}>`, 'i').exec(inner);
  return (closing ? inner.slice(0, closing.index) : inner)
    .replace(/<[^>]*>/g, ' ')
    .replace(/[⁨⁩]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * A minimal localized ruleset, so a logged Crisis row resolves to its name in
 * `rules/i18n/<lang>/aging.json` rather than printing its id.
 */
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
    i18n: { 'crisis.minor_illness': { name: 'Minor illness' } },
  } as unknown as LocalizedRuleset;
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  resetEntity();
});

describe('AgingRecordPanel (slice 6b6b)', () => {
  it('keeps every recorded-aging control the details tab had', () => {
    // The whole recorded-aging surface, as CharacterDetails carried it before the
    // extraction: an entry in the log, aging points, a Decrepitude score.
    store.entity.apparent_age = 38;
    store.entity.aging_points = { sta: 2 };
    store.entity.aging_log = [{ year: 1220, effect: 'A hard winter' }];
    setDecrepitude(1);
    const body = html();

    expect(has(body, 'aging-record')).toBe(true);
    expect(has(body, 'apparent-age-input')).toBe(true);
    expect(has(body, 'decrepitude-readout')).toBe(true);
    expect(has(body, 'decrepitude-effect-input')).toBe(true);
    expect(has(body, 'aging-points-list')).toBe(true);
    for (const characteristic of CHARACTERISTICS) {
      expect(has(body, `aging-points-${characteristic}`)).toBe(true);
    }
    expect(has(body, 'aging-points-note')).toBe(true);
    expect(has(body, 'aging-log-list')).toBe(true);
    expect(has(body, 'aging-log-year-0')).toBe(true);
    expect(has(body, 'aging-log-effect-0')).toBe(true);
    expect(has(body, 'aging-log-remove-0')).toBe(true);
    expect(has(body, 'aging-log-add')).toBe(true);
  });

  it('says the log is empty when no year is recorded', () => {
    const body = html();
    expect(has(body, 'aging-log-empty')).toBe(true);
    expect(has(body, 'aging-log-year-0')).toBe(false);
    // Decrepitude is hidden at 0 — a score of zero is not a state to report.
    expect(has(body, 'decrepitude-readout')).toBe(false);
  });

  it('reads a resolved Crisis back off the log entry that recorded it', () => {
    // The four crisis fields are the whole record of what the Crisis Table was
    // asked and what it answered (`:16621`, `:16624-16632`). Without them on
    // screen a resolved Crisis is invisible the moment the calculator is closed.
    installRuleset();
    store.entity.aging_log = [
      {
        year: 1220,
        age: 40,
        effect: '',
        die: 9,
        total: 13,
        crisis: true,
        crisis_die: 10,
        crisis_total: 15,
        crisis_row: 'crisis.minor_illness',
        crisis_severity: 'minor',
      },
    ];
    const body = html();
    const crisis = text(body, 'aging-log-crisis-0');
    // The row's text is rules data keyed by its id; the severity goes through
    // Fluent. Neither is ever rendered as its slug.
    expect(crisis).toContain('Minor illness');
    expect(crisis).toContain('15');
    expect(crisis).toContain('10');
    expect(crisis).not.toContain('crisis.minor_illness');
    expect(body).not.toContain('crisis_severity');
  });

  it('tells a Crisis owed and unrolled apart from a resolved one', () => {
    // Three states, not two: no Crisis, one the table demanded that nobody has
    // rolled, and one resolved. A `crisis` with no row is the middle state.
    installRuleset();
    store.entity.aging_log = [
      { year: 1220, age: 40, effect: '', die: 9, total: 13, crisis: true },
      { year: 1221, age: 41, effect: '', die: 4, total: 8 },
    ];
    const body = html();
    expect(text(body, 'aging-log-crisis-0').length).toBeGreaterThan(0);
    expect(has(body, 'aging-log-crisis-1')).toBe(false);
  });

  it('labels every control through Fluent, never as a raw slug', () => {
    const body = html();
    expect(body).toContain('Apparent age');
    expect(body).toContain('Aging');
    // Characteristic rows are named in words, never by their slugs.
    expect(body).toContain('Stamina');
    expect(body).not.toMatch(/>\s*sta\s*</);
    expect(body).not.toMatch(/>\s*apparent_age\s*</);
  });
});
