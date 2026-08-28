import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity } from '../types';

// The step is a composition over the shared store singleton. The store schedules
// a debounced revalidate over the Tauri IPC bridge; mock the bridge so nothing
// reaches a backend. Harness mirrors AgeFields.test.ts.
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
import AgingStep from './AgingStep.svelte';

/** A grog: no House, no Arts — the type the guided aging step is hardest for. */
function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'grog',
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

/** Render the step to an HTML string (node env, no DOM). */
function html(): string {
  return render(AgingStep, { props: {} }).body;
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

describe('AgingStep (slice 6b6c)', () => {
  it('carries the Longevity Ritual on the aging step, which no other step can reach', () => {
    // The ritual's bonus is a term of the AGING TOTAL
    // (Core Rules.md:16567-16569), but its editor home is the Possessions tab,
    // which is magus-gated (App.svelte) and which no CreationPhase maps to. So
    // without this mount a guided character cannot enter the bonus at all.
    const body = html();
    expect(has(body, 'aging-step')).toBe(true);
    expect(has(body, 'longevity-add')).toBe(true);
  });

  it('takes the bonus of a ritual a grog did not make himself', () => {
    // "You can perform Longevity Rituals for others, even for non-magi."
    // Source: Ars Magica - Definitive Edition (Core Rules).md:10672 — so the
    // panel must work for a non-magus. Only the Creo Corpus SUGGESTION is
    // magus-gated (derived.rs), and a grog simply gets no hint.
    store.entity.longevity_ritual = { source: 'external', bonus: 7, focus: '' };
    const body = html();
    expect(has(body, 'longevity-bonus')).toBe(true);
    expect(has(body, 'longevity-source-external')).toBe(true);
    // No engine hint for a non-magus, and none invented here.
    expect(has(body, 'longevity-hint')).toBe(false);
  });

  it('still mounts the age and the aging surface it shares with the editor', () => {
    // The ritual joins the step; it does not displace what was already there.
    const body = html();
    expect(has(body, 'age-input')).toBe(true);
    expect(has(body, 'aging-panel')).toBe(true);
  });
});
