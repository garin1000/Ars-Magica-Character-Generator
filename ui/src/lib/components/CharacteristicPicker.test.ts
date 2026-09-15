import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import {
  CHARACTERISTICS,
  type Entity,
  type EffectiveScores,
  type LocalizedRuleset,
} from '../types';

import { SCHEMA_VERSION, store } from '../state.svelte';
import CharacteristicPicker from './CharacteristicPicker.svelte';

/** A minimal localized ruleset with the standard ±3 characteristic cost table. */
function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      characteristic_rules: {
        start_points: 7,
        base_max: 3,
        base_min: -3,
        costs: [-3, -2, -1, 0, 1, 2, 3].map((score) => ({ score, cost: Math.abs(score) })),
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

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
    ability_funding: 'pool',
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
  store.effective = null;
}

function html(): string {
  return render(CharacteristicPicker, { props: {} }).body;
}

/** Fluent isolates interpolated values with bidi marks; strip them for text matching. */
function clean(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// S10 (full-audit a11y): the eight Characteristic description `<input>`s all
// shared the same `placeholder` ("Description") and carried no accessible name
// of their own — a screen-reader user tabbing through them heard "Description,
// edit text" eight times with no way to tell Strength's field from Stamina's.
// Each now gets a distinct, Fluent-sourced `aria-label` naming its own
// Characteristic.
describe('CharacteristicPicker description fields (S10)', () => {
  it('gives each description input a distinct accessible name naming its Characteristic', () => {
    const body = html();
    for (const characteristic of CHARACTERISTICS) {
      const input = new RegExp(`<input[^>]*data-testid="char-desc-${characteristic}"[^>]*>`).exec(
        body,
      );
      expect(input, `no description input for ${characteristic}`).not.toBeNull();
      expect(input![0]).toMatch(/aria-label="[^"]+"/);
    }
    // Every label is distinct — Strength's input must not be reachable by Stamina's name.
    const names = CHARACTERISTICS.map((characteristic) => {
      const tag = new RegExp(`<input[^>]*data-testid="char-desc-${characteristic}"[^>]*>`).exec(
        body,
      )![0];
      return /aria-label="([^"]+)"/.exec(tag)![1];
    });
    expect(new Set(names).size).toBe(CHARACTERISTICS.length);
  });

  it("names Strength's field with the localized Characteristic name, not its slug", () => {
    const body = html();
    const input = /<input[^>]*data-testid="char-desc-str"[^>]*>/.exec(body)![0];
    expect(input).toContain('Strength');
    expect(input).not.toMatch(/aria-label="[^"]*\bstr\b[^"]*"/);
  });
});

/** The start tag of the element whose attributes contain `marker`. */
function tagContaining(body: string, marker: string): string {
  const tag = new RegExp(`<[a-z]+[^>]*${marker}[^>]*>`).exec(body);
  if (!tag) throw new Error(`no element matching ${marker}`);
  return tag[0];
}

// G12 (full-audit test-adequacy): CharacteristicPicker had no test file beyond
// the S10 a11y fix above. It carries real logic (point-buy budget, per-
// characteristic cap/floor clamping, the effective-score badge, the Size
// readout) that a regression could silently break with nothing catching it.
describe('CharacteristicPicker point-buy budget and spinners', () => {
  it('reads used/budget from the engine, not a local re-derivation', () => {
    store.effective = {
      characteristic_points_used: 3,
      characteristic_points_granted: 2,
    } as unknown as EffectiveScores;
    const body = html();
    // start_points (7) + granted (2) = 9.
    expect(body).toContain('3');
    expect(body).toContain('9');
  });

  it('disables decrement at the floor and increment at the cap', () => {
    store.entity.characteristics = { str: -3, sta: 3 } as Entity['characteristics'];
    const body = html();
    expect(tagContaining(body, 'data-testid="char-dec-str"')).toContain('disabled');
    expect(tagContaining(body, 'data-testid="char-inc-str"')).not.toContain('disabled');
    expect(tagContaining(body, 'data-testid="char-inc-sta"')).toContain('disabled');
    expect(tagContaining(body, 'data-testid="char-dec-sta"')).not.toContain('disabled');
  });

  it('takes the buyable range from the engine rather than re-deriving it', () => {
    // The engine, not this component, owns the per-characteristic buy range
    // (VA1-style separation), so whatever cap it reports must drive the
    // increment button's disabled state — even one wider than the cost table.
    store.entity.characteristics = { str: 3 } as Entity['characteristics'];
    store.effective = {
      characteristic_caps: { str: 5 },
    } as unknown as EffectiveScores;
    const body = html();
    expect(tagContaining(body, 'data-testid="char-inc-str"')).not.toContain('disabled');
  });

  it('keeps the spinner at +3 under Great Characteristic and shows the point as a badge', () => {
    // Great (Characteristic) grants the point rather than unlocking a purchase
    // (ArMDE:3989), so the engine reports an unchanged +3 cap and a +4 effective
    // score. The spinner must therefore stay capped at the bought +3, with the
    // fourth point surfacing in the effective badge — exactly how Giant Blood's
    // free +1 already renders.
    store.entity.characteristics = { str: 3 } as Entity['characteristics'];
    store.effective = {
      characteristic_caps: { str: 3 },
      characteristic_effective: { str: 4 },
      characteristic_bonuses: [{ characteristic: 'str', bonus: 1 }],
    } as unknown as EffectiveScores;
    const body = html();
    expect(tagContaining(body, 'data-testid="char-inc-str"')).toContain('disabled');
    expect(body).toContain('data-testid="char-effective-str"');
    expect(clean(body)).toContain(clean(store.t('effective-score-from', { score: '+4' })));
  });

  it('shows the effective-score badge only when it differs from the bought score', () => {
    store.entity.characteristics = { str: 1 } as Entity['characteristics'];
    expect(html()).not.toContain('data-testid="char-effective-str"');

    store.effective = {
      characteristic_effective: { str: 0 },
    } as unknown as EffectiveScores;
    const withDrop = html();
    expect(withDrop).toContain('data-testid="char-effective-str"');
    // Sabine 5 (full-audit round 4): asserted through the KEY, not against a
    // literal `→ 0`. The arrow is user-facing text — it carries the base→effective
    // relationship the three sibling badges leave to colour alone — so it belongs
    // in Fluent like the `&` in `derived-combat-and` and the `n/a` in
    // `derived-not-applicable`. A literal here passed whether the glyph came
    // through the bundle or was written into the markup, which is how it stayed
    // hardcoded.
    const badge = store.t('effective-score-from', { score: '0' });
    expect(badge).not.toBe('effective-score-from');
    // `clean` on both sides: Fluent wraps a placeable in FSI/PDI isolation marks,
    // and the rendered markup carries them just as the bundle's own output does.
    expect(clean(withDrop)).toContain(clean(badge));
  });

  it('shows the Size readout only when it is off 0', () => {
    expect(html()).not.toContain('data-testid="characteristic-size"');

    store.effective = { size: 1 } as unknown as EffectiveScores;
    expect(html()).toContain('data-testid="characteristic-size"');
  });
});

// Sabine 3 (full-audit round 1): `use:tooltip` opens on `focusin` as well as
// `mouseenter`, so the rules text it holds is keyboard-ready by construction — but
// only if its host can take focus, and both hosts here were bare `<span>`s. The
// characteristic description and the bought→effective breakdown were mouse-only.
//
// `ssr`, not `client`: `tabindex` is static markup, and the outcome it produces —
// focus opening the popup — is proved once for the whole action in
// `ArtGrid.client.test.ts`. Re-mounting a component per site to re-test the shared
// action would test the action five times and the markup not at all.
describe('CharacteristicPicker tooltip hosts are keyboard-reachable (Sabine 3)', () => {
  /** The start tag of the first element whose attributes contain `marker`. */
  function tagContaining(body: string, marker: string): string {
    const tag = new RegExp(`<[a-z]+[^>]*${marker}[^>]*>`).exec(body);
    if (!tag) throw new Error(`no element whose start tag contains ${marker}`);
    return tag[0];
  }

  it('puts the characteristic name in the tab order', () => {
    expect(tagContaining(html(), 'class="spinner-label"')).toContain('tabindex="0"');
  });

  it('puts the effective-score badge in the tab order', () => {
    store.entity.characteristics = { str: 1 } as Entity['characteristics'];
    store.effective = {
      characteristic_effective: { str: 0 },
    } as unknown as EffectiveScores;

    expect(tagContaining(html(), 'data-testid="char-effective-str"')).toContain('tabindex="0"');
  });
});
