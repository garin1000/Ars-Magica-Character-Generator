import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, SurfacedModifier } from '../types';
import { store } from '../state.svelte';
import DerivedSurfacedModifiersSection from './DerivedSurfacedModifiersSection.svelte';

/** A minimal, complete DerivedTotals fixture; only `surfaced_modifiers` varies. */
function derivedFixture(surfaced_modifiers: SurfacedModifier[]): DerivedTotals {
  return {
    is_magus: false,
    lab_totals: [],
    casting_totals: [],
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
});

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
