import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, LocalizedRuleset, SurfacedModifier } from '../types';
import { store } from '../state.svelte';
import DerivedSurfacedModifiersSection from './DerivedSurfacedModifiersSection.svelte';

/** A minimal, complete DerivedTotals fixture; only `surfaced_modifiers` varies. */
function derivedFixture(surfaced_modifiers: SurfacedModifier[]): DerivedTotals {
  return {
    hermetically_trained: false,
    lab_totals: [],
    casting_totals: [],
    spell_casting_totals: [],
    penetration: [],
    magic_resistance: [],
    combat: [],
    soak: { addends: [], total: 0 },
    encumbrance: { load: 0, burden: 0, total: 0 },
    fatigue: [],
    wounds: [],
    size: 0,
    decrepitude_score: 0,
    warping_score: 0,
    warping_points: 0,
    surfaced_modifiers,
  };
}

function html(d: DerivedTotals): string {
  return render(DerivedSurfacedModifiersSection, { props: { d } }).body;
}

beforeEach(() => {
  store.lang = 'en';
  store.ruleset = null;
});

/** A minimal `LocalizedRuleset` naming one item, for source-attribution tests. */
function installRulesetNaming(id: string, name: string): void {
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
    i18n: { [id]: { name } },
  } as unknown as LocalizedRuleset;
}

// D55 (Q6, V/F audit Q-113): `amount: 0` used to double as "no magnitude" AND
// "halved", so a halving row rendered with nothing beside it — indistinguishable
// from a modifier of no size. `factor` now disambiguates: when it is present the
// row must render the factor's own Fluent string, never the raw `amount` number
// and never the bare slug (`half`) — that would be exactly the same "raw enum
// value rendered as a label" violation CLAUDE.md forbids for any other slug.
describe('DerivedSurfacedModifiersSection renders a halving factor as a word', () => {
  it('renders an ordinary amount as a signed number', () => {
    const body = html(derivedFixture([{ family: 'advancement', detail: 'taught', amount: 5 }]));
    expect(body).toContain('+5');
  });

  it('renders a factor-carrying row as "Halved", not "0" and not the slug', () => {
    const body = html(
      derivedFixture([{ family: 'advancement', detail: 'teaching', amount: 0, factor: 'half' }]),
    );
    expect(body).toContain('Halved');
    expect(body).not.toContain('>0<');
    expect(body).not.toMatch(/\bhalf\b/);
  });

  it('renders both a factor row and an ordinary amount row side by side', () => {
    const body = html(
      derivedFixture([
        { family: 'advancement', detail: 'authoring', amount: 0, factor: 'half' },
        { family: 'advancement', detail: 'taught', amount: -3 },
      ]),
    );
    expect(body).toContain('Halved');
    expect(body).toContain('-3');
  });
});

// D45/F-423: six shipped Flaws used to render the identical unattributed
// "Special casting: Circumstantial" line, with nothing to tell a character
// holding two of them apart. `source` names the granting item; the panel must
// show its localized rules name, never the raw id — the same rule that
// governs every other slug this component renders (`detailLabel`,
// `factorLabel`).
describe('DerivedSurfacedModifiersSection attributes a row to its source (D45, F-423)', () => {
  it('renders the source as its localized display name, not the raw id', () => {
    installRulesetNaming('flaw.environmental_magic_condition', 'Environmental Magic Condition');
    const body = html(
      derivedFixture([
        {
          family: 'special_casting',
          detail: 'circumstantial',
          amount: 0,
          source: 'flaw.environmental_magic_condition',
        },
      ]),
    );
    expect(body).toContain('Environmental Magic Condition');
    expect(body).not.toContain('flaw.environmental_magic_condition');
  });

  it('lets two carriers of the identical family+detail read as two distinct lines', () => {
    installRulesetNaming('flaw.environmental_magic_condition', 'Environmental Magic Condition');
    // A second family/detail-identical row from a different Flaw, installed by
    // hand since `installRulesetNaming` only names one item at a time.
    (store.ruleset as unknown as { i18n: Record<string, { name: string }> }).i18n[
      'flaw.deleterious_circumstances'
    ] = { name: 'Deleterious Circumstances' };
    const body = html(
      derivedFixture([
        {
          family: 'special_casting',
          detail: 'circumstantial',
          amount: 0,
          source: 'flaw.environmental_magic_condition',
        },
        {
          family: 'special_casting',
          detail: 'circumstantial',
          amount: 0,
          source: 'flaw.deleterious_circumstances',
        },
      ]),
    );
    expect(body).toContain('Environmental Magic Condition');
    expect(body).toContain('Deleterious Circumstances');
  });

  it('renders no attribution when the family carries none (HealthRoll)', () => {
    const body = html(
      derivedFixture([{ family: 'health_roll', detail: 'fatigue_roll', amount: 3 }]),
    );
    expect(body).not.toContain('derived-surfaced-source');
  });
});

// B5 (coordinator review, post-phase-1 design decision; D61 corrected the
// worked example from Poor Hearing to Poor Concentration — Poor Hearing's
// "rolls involving hearing" is sense-conditioned and stays text, see
// `docs/vf-audit/decisions.md` D61): a fixed-target `ability_roll_mod` row
// (Poor Concentration: -3 to Concentration) names its Ability through the
// STRUCTURED `ability` field, never through the free-text `detail` a
// parameter-based row (Academic Concentration) uses — a raw catalogue id
// must never reach a field this component renders verbatim (CLAUDE.md). The
// panel must resolve `ability` the same way every other catalogue id is
// resolved: through the ruleset's own i18n, not a hardcoded slug or the id
// itself.
describe('DerivedSurfacedModifiersSection renders a fixed-target ability_roll_mod (B5)', () => {
  it('renders the localized English Ability name, never the raw id', () => {
    store.lang = 'en';
    installRulesetNaming('ability.concentration', 'Concentration');
    const body = html(
      derivedFixture([
        {
          family: 'ability_roll',
          detail: '',
          amount: -3,
          source: 'flaw.poor_concentration',
          ability: 'ability.concentration',
        },
      ]),
    );
    expect(body).toContain('Concentration');
    expect(body).not.toContain('ability.concentration');
  });

  it('renders the localized German Ability name, never the raw id', () => {
    store.lang = 'de';
    installRulesetNaming('ability.concentration', 'Konzentration');
    const body = html(
      derivedFixture([
        {
          family: 'ability_roll',
          detail: '',
          amount: -3,
          source: 'flaw.poor_concentration',
          ability: 'ability.concentration',
        },
      ]),
    );
    expect(body).toContain('Konzentration');
    expect(body).not.toContain('ability.concentration');
  });
});
