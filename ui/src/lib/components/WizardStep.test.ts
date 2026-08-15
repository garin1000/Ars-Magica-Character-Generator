import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { CreationPhase, EffectiveScores, Entity, LocalizedRuleset } from '../types';

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
      // The ruleset ships life-stage rules, so the Abilities step offers the funding
      // choice — the wizard mounts the very component the editor tab does.
      life_stages: {
        childhood: {
          years: 5,
          native_language_ability: 'ability.living_language',
          native_language_xp: 75,
          spread_xp: 45,
          spread_abilities: ['ability.athletics'],
        },
        later_life: { xp_per_year: 15 },
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
    ['aging', 'aging-step'],
    ['review', 'wizard-review-hint'],
  ];

  for (const [phase, testid] of expected) {
    it(`mounts the ${phase} surface`, () => {
      expect(body(phase)).toContain(`data-testid="${testid}"`);
    });
  }

  // AbilityTab carries the life-stage funding panel, so mounting it in the wizard
  // puts the choice in both flows at once: a save made in the wizard stays editable
  // in the editor because they are one surface, not two.
  it('offers the life-stage funding panel on the abilities step', () => {
    expect(body('abilities')).toContain('data-testid="life-stage-panel"');
  });

  // And the Hermetic minimums beside it, from the same mount: the wizard's magus has
  // to see what the Order demands before it can be finished.
  it('shows the magus minimums checklist on the abilities step', () => {
    store.effective = {
      magus_minimum_abilities: [
        {
          ability: 'ability.parma_magica',
          min_score: 1,
          score: 0,
          met: false,
          requirement: 'required',
        },
      ],
    } as unknown as EffectiveScores;
    expect(body('abilities')).toContain('data-testid="magus-minimums"');
  });

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
