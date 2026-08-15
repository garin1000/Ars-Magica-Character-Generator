import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

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

import { SCHEMA_VERSION, store } from '../state.svelte';
import WizardReview from './WizardReview.svelte';

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
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
  };
  store.result = { issues: [] };
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  store.result = null;
});

describe('WizardReview', () => {
  it('reports a clean character when nothing is outstanding', () => {
    const body = render(WizardReview).body;
    expect(body).toContain('data-testid="wizard-review-clean"');
  });

  // The unfiltered panel is the point of this step: it is the only place a
  // finding no creation phase owns — equipment, Might, Warping — is shown.
  it('shows the findings no step of the flow could have shown', () => {
    store.result = {
      issues: [
        {
          severity: 'error',
          code: 'unknown_equipment',
          phase: 'review',
          args: { item: 'weapon.nonesuch' },
        },
      ],
    };
    const body = render(WizardReview).body;
    expect(body).toContain('data-code="unknown_equipment"');
    expect(body).not.toContain('data-testid="wizard-review-clean"');
  });

  // Legal is not complete: the gate only catches errors, so an empty-but-legal
  // phase walks through. Saying so here is the honest version of that gap.
  it('warns that a legal character may still be incomplete', () => {
    expect(render(WizardReview).body).toContain('data-testid="wizard-review-incomplete"');
  });

  it('points out that the editor holds the surfaces the flow never visits', () => {
    expect(render(WizardReview).body).toContain('data-testid="wizard-review-hint"');
  });

  it('localizes to German without echoing a key', () => {
    store.lang = 'de';
    const body = render(WizardReview).body;
    expect(body).not.toContain('wizard-review-hint<');
    expect(body).toMatch(/Charakter|Editor|Überprüfung/);
  });
});
