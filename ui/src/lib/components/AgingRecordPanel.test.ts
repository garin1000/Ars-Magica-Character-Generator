import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import { CHARACTERISTICS, type EffectiveScores, type Entity } from '../types';

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
