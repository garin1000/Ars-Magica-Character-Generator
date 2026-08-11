import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { CreationPhase, Entity, LocalizedRuleset } from '../types';

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
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import WizardStep from './WizardStep.svelte';

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {
        magus: {
          id: 'magus',
          budget: { virtue_points: 10, flaw_points: 10 },
          is_magus: true,
          gift_policy: 'required',
          creation_phases: [],
        },
      },
      abilities: {},
      arts: {},
      spells: {},
      houses: {},
      mythic_types: {},
      characteristic_rules: {
        start_points: 7,
        base_max: 3,
        base_min: -3,
        effective_max: 5,
        effective_min: -5,
        costs: [],
      },
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
    type_id: 'magus',
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
  store.result = { issues: [] };
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  store.result = null;
});

function body(phase: CreationPhase): string {
  return render(WizardStep, { props: { phase } }).body;
}

describe('WizardStep', () => {
  // Each phase mounts the same direct-entry surface the editor's tab uses — the
  // wizard adds orchestration, not a second set of inputs.
  const expected: [CreationPhase, string][] = [
    ['concept', 'identity-concept'],
    ['type', 'type-step-name'],
    ['characteristics', 'characteristic-points'],
    ['virtues_flaws', 'balance'],
    ['abilities', 'xp-spent'],
    ['arts', 'art-xp-spent'],
    ['spells', 'spell-levels-used'],
    ['house_specialisation', 'house-select'],
    ['mythic_type', 'mythic-type-select'],
    ['personality_reputations', 'personality-list'],
    ['review', 'wizard-review-hint'],
  ];

  for (const [phase, testid] of expected) {
    it(`mounts the ${phase} surface`, () => {
      expect(body(phase)).toContain(`data-testid="${testid}"`);
    });
  }

  it('shows both halves of the personality step, traits and reputations', () => {
    const markup = body('personality_reputations');
    expect(markup).toContain('data-testid="personality-list"');
    expect(markup).toContain('data-testid="reputation-list"');
  });

  // The rail carries the step's title. A heading here would nest under it, and
  // every region component already ships its own "Available"/"Selected" h2s.
  it('adds no heading of its own around a region surface', () => {
    const markup = body('virtues_flaws');
    const headings = [...markup.matchAll(/<h2[^>]*>([\s\S]*?)<\/h2>/g)].map((m) =>
      m[1].replace(/<[^>]*>/g, '').trim(),
    );
    expect(headings).not.toContain('Virtues & Flaws');
  });

  // The height chain `.tab-content > .vf-tab > .region-row` is what keeps the
  // Available list from collapsing; the wizard reproduces it rather than restyling.
  it('wraps a region surface in the editor tab-content chain', () => {
    expect(body('virtues_flaws')).toContain('class="vf-tab"');
  });

  it('wraps a long single-panel surface in the scrolling container', () => {
    expect(body('concept')).toContain('class="tab-scroll"');
  });
});
