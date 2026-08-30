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
import CharacteristicPicker from './CharacteristicPicker.svelte';
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
    ability_funding: 'pool',
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

  // guided-creation-review-2026-08 #19: the spell-levels base is a fixed rules grant
  // (Core Rules.md:2215), so the wizard shows it and does not offer it for editing.
  // The divergence is carried by an explicit prop through the existing `barProps`
  // seam — the same seam that already carries `XpBar`'s testid prefix — and never by
  // the bar sniffing which flow mounted it.
  it('passes readonlyBase to the spells bar', () => {
    const open = /<[^>]*data-testid="spell-levels-base"[^>]*>/.exec(body('spells'));
    expect(open).not.toBeNull();
    expect(open![0]).not.toMatch(/<input/i);
    expect(open![0]).toMatch(/<span/i);
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

  // --- Slice 9 (#27): one panel, two mounts, one centring ------------------

  // The editor mounts `CharacteristicPicker` as a direct child of `.tab-panel`,
  // which centres its children; the wizard always wraps a step body in `.vf-tab`,
  // which is `width: 100%` and stretches them. So the panel was centred in one
  // surface and left-aligned in the other. The fix is that the panel centres
  // ITSELF (`.char-panel`'s auto inline margins, pinned in `app.css.test.ts`), and
  // this is the guard that both mounts really carry that one class rather than one
  // of them growing a centring wrapper of its own.
  it('carries the self-centring characteristics panel in both mounts', () => {
    const panelTag = (markup: string): string => {
      const open = /<[^>]*data-testid="char-panel"[^>]*>/.exec(markup);
      if (!open) throw new Error('no element with data-testid="char-panel"');
      return open[0];
    };

    const wizardMount = body('characteristics');
    const editorMount = render(CharacteristicPicker).body;

    expect(panelTag(wizardMount)).toContain('char-panel');
    // Byte-identical opening tags: the centring cannot depend on which surface
    // mounted the picker, because there is only one element to carry it.
    expect(panelTag(wizardMount)).toBe(panelTag(editorMount));

    // And the wizard mount really is the stretching wrapper the editor has not
    // got — the divergence the shared class has to overcome.
    expect(wizardMount).toContain('class="vf-tab"');
    expect(editorMount).not.toContain('class="vf-tab"');
  });

  it('localizes the note to German', () => {
    store.lang = 'de';
    const note = guidance('concept');
    expect(note).not.toContain('wizard-guidance-concept');
    expect(note).toContain('Konzept');
  });

  // --- Slice 12 (#24): age has ONE canonical home ---------------------------

  /**
   * How many times a test-id is rendered across the whole wizard — every phase the
   * step table declares. The point of the count is that no reader can tell from one
   * step alone whether the age is offered twice; only the sum can.
   */
  function countAcrossSteps(testid: string): number {
    let seen = 0;
    for (const [phase] of expected) {
      seen += [...body(phase).matchAll(new RegExp(`data-testid="${testid}"`, 'g'))].length;
    }
    return seen;
  }

  it('mounts the identity fields and the age together on the concept step', () => {
    // Mirrors the editor, where `CharacterDetails` mounts `IdentityFields` and then
    // `AgeFields` immediately after: the age belongs beside the birth year it is now
    // linked to, not on a step three phases later.
    const markup = body('concept');
    expect(markup).toContain('data-testid="identity-concept"');
    expect(markup).toContain('data-testid="identity-birth-year"');
    expect(markup).toContain('data-testid="age-input"');
  });

  it('gives a pool-funded character exactly one age input in the whole wizard', () => {
    store.entity.ability_funding = 'pool';
    store.entity.age = 30;
    // This is the assertion that makes dropping the aging step's copy safe: under
    // flat funding no other surface offers an age at all, so if Concept ever lost
    // its field the wizard would have none.
    expect(countAcrossSteps('age-input')).toBe(1);
    expect(countAcrossSteps('life-stage-age-input')).toBe(0);
  });

  it('gives a life-stage-funded character one editable age and one read-only echo', () => {
    store.entity.ability_funding = 'life_stages';
    store.entity.life_stages = {};
    store.entity.age = 30;
    // A magus carries two ages, not one: the Experience step still needs to show the
    // age its Gauntlet age is measured against, but it is not a second place to
    // change it.
    expect(countAcrossSteps('age-input')).toBe(1);
    expect(countAcrossSteps('age-readout')).toBe(1);
    expect(countAcrossSteps('life-stage-age-input')).toBe(0);
  });

  it('renders the age cap note exactly once, beside the ability lists', () => {
    store.effective = { age_ability_cap: 5 } as unknown as EffectiveScores;
    store.entity.age = 30;
    try {
      // It used to render on two surfaces at once. Its home is the step whose lists
      // the cap actually constrains.
      expect(countAcrossSteps('age-cap-note')).toBe(1);
      expect(body('abilities')).toContain('data-testid="age-cap-note"');
    } finally {
      store.effective = null;
    }
  });
});
