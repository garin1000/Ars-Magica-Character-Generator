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
    ['characteristics', 'characteristic-points'],
    ['virtues_flaws', 'balance'],
    ['experience', 'life-stage-panel'],
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

  // Slice 2 (#11): the funding panel is the `experience` step's own surface, and the
  // `abilities` step is the Available/Selected lists alone — three concerns in one
  // bounded flex column is what collapsed both lists in an 800px window.
  it('mounts the ExperienceStep for the experience phase', () => {
    const markup = body('experience');
    expect(markup).toContain('data-testid="life-stage-panel"');
    expect(markup).toContain('data-testid="ability-funding-life_stages"');
  });

  it('keeps the funding panel off the abilities step', () => {
    expect(body('abilities')).not.toContain('data-testid="life-stage-panel"');
  });

  // The step has to offer the funding choice it asks for. Under flat (pool) funding
  // the panel renders only the two radios — the total itself is `XpBar`'s input — so
  // without a bar here `wizard-guidance-experience` promises "enter one total
  // yourself" beside no field to enter it in, and the step can never record
  // anything for a pool-funded character. #14's life-stage chips land on this step
  // too, so the bar belongs here for both funding modes.
  it('mounts the experience budget bar so the pool total can be entered', () => {
    // The editable field, not the read-only `xp-pool-total` span beside it: the
    // point is that a pool-funded character can answer this step at all.
    expect(body('experience')).toContain('data-testid="xp-pool"');
  });

  // #1: the read-only `type` step is gone. The step table is exhaustive over
  // `CreationPhase` by `satisfies`, so the only way to show the entry is absent is to
  // ask for it — which now finds nothing to mount rather than a blank step.
  it('has no step for a removed type phase', () => {
    expect(() => body('type' as CreationPhase)).toThrow();
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

  // --- 6b8b: the per-step guidance note ------------------------------------

  function guidance(phase: CreationPhase): string {
    const markup = /data-testid="wizard-guidance"[^>]*>([\s\S]*?)<\/p>/.exec(body(phase));
    if (!markup) throw new Error(`no guidance on the ${phase} step`);
    return markup[1]
      .replace(/<[^>]*>/g, '')
      .replace(/[⁨⁩]/g, '')
      .trim();
  }

  // Every step says what is decided on it — including the closing one, which
  // decides nothing and says so.
  for (const [phase] of expected) {
    it(`explains what the ${phase} step is for`, () => {
      const note = guidance(phase);
      expect(note.length).toBeGreaterThan(0);
      // The one thing a label may never do: render its own key back at the player,
      // which is exactly what a locale missing the line would produce.
      expect(note).not.toContain('wizard-guidance');
    });
  }

  it('takes the Characteristic allowance from the ruleset rather than stating it', () => {
    expect(guidance('characteristics')).toContain('7');
    store.ruleset!.ruleset.characteristic_rules!.start_points = 9;
    expect(guidance('characteristics')).toContain('9');
  });

  it("takes the Virtue and Flaw budget from the character type's profile", () => {
    store.ruleset!.ruleset.type_profiles.magus.budget = { virtue_points: 20, flaw_points: 10 };
    const note = guidance('virtues_flaws');
    expect(note).toContain('20');
    expect(note).toContain('10');
  });

  it('localizes the note to German', () => {
    store.lang = 'de';
    const note = guidance('concept');
    expect(note).not.toContain('wizard-guidance-concept');
    expect(note).toContain('Konzept');
  });
});
