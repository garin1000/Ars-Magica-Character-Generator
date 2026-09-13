import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// An `ssr` test, deliberately: everything asserted here is static rendered
// markup — which words a read-out is composed of. Nothing needs a live
// instance, an `$effect` or a DOM event, so the fast project is the right one.
//
// The section reads the shared store singleton (the Fluent bundle), which
// schedules a debounced revalidate over the Tauri IPC bridge; mock the bridge
// so nothing reaches a backend. Mirrors DerivedLabCastingSection.test.ts.
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
import DerivedSummarySection from './DerivedSummarySection.svelte';

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

/** Render the section to an HTML string (node env, no DOM). */
function html(): string {
  return render(DerivedSummarySection, {
    props: {
      d: { size: 0, decrepitude_score: 0, warping_score: 3, warping_points: 12 } as never,
    },
  }).body;
}

/**
 * Fluent wraps every interpolated value in bidi isolation marks (FSI U+2068 /
 * PDI U+2069) so a right-to-left value cannot reorder the sentence around it.
 * They are invisible, they are correct, and they are not what these assertions
 * are about — strip them so an expectation can be written in readable text.
 */
function stripIsolation(s: string): string {
  return s.replace(/[⁨⁩]/g, '');
}

/** The text of the `derived-warping` cell, tags and isolation marks stripped. */
function warpingCell(body: string): string {
  const match = body.match(/data-testid="derived-warping"[^>]*>([\s\S]*?)<\/dd>/);
  expect(match, 'the warping read-out should render').not.toBeNull();
  return stripIsolation((match?.[1] ?? '').replace(/<[^>]*>/g, '')).trim();
}

beforeEach(() => {
  installRuleset();
  resetEntity();
});

// Sabine 8 (round-1 audit). The Summary panel rendered Warping as
// `{d.warping_score} ({d.warping_points})` → "3 (12)", with nothing saying that
// 3 is the SCORE and 12 the running POINTS count — hardcoded parentheses
// standing in for the missing words, which is also a user-facing string with no
// Fluent key.
//
// The labelled wording already ships and is already used for this very pair on
// the Details tab (`CharacterDetails.svelte`, `warping-readout`), so the defect
// is that ONE figure had TWO renderings in one app and the unlabelled one was
// the denser panel. That matters beyond tidiness: the Warping *score* is a
// threshold with consequences and the *points* are the accumulator creeping
// toward it, so which number is which is the whole reading.
describe('DerivedSummarySection — warping read-out (Sabine 8)', () => {
  it('labels score and points rather than juxtaposing them in parentheses', () => {
    store.lang = 'en';
    const cell = warpingCell(html());
    expect(cell).toBe('Score 3, 12 points');
    expect(cell).not.toContain('(');
  });

  it('uses the German wording under the German bundle', () => {
    store.lang = 'de';
    const cell = warpingCell(html());
    expect(cell).toBe('Wert 3, 12 Punkte');
    expect(cell).not.toContain('(');
  });

  it('renders the same string the Details tab renders, from one key', () => {
    // Guards the actual defect: two surfaces answering one question two ways.
    store.lang = 'en';
    expect(warpingCell(html())).toBe(
      stripIsolation(store.t('warping-readout', { score: String(3), points: String(12) })),
    );
  });
});
