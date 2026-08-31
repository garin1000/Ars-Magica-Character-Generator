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

  it('widens the buyable range past the table when the engine reports a raised cap/lowered floor', () => {
    // Great Characteristic / Poor Characteristic widen the per-characteristic
    // buy range past the ruleset's base ±3 — the engine, not this component,
    // owns that math (VA1-style separation), so a wider cap must lift the
    // increment button's disabled state past the table max.
    store.entity.characteristics = { str: 3 } as Entity['characteristics'];
    store.effective = {
      characteristic_caps: { str: 5 },
    } as unknown as EffectiveScores;
    const body = html();
    expect(tagContaining(body, 'data-testid="char-inc-str"')).not.toContain('disabled');
  });

  it('shows the effective-score badge only when it differs from the bought score', () => {
    store.entity.characteristics = { str: 1 } as Entity['characteristics'];
    expect(html()).not.toContain('data-testid="char-effective-str"');

    store.effective = {
      characteristic_effective: { str: 0 },
    } as unknown as EffectiveScores;
    const withDrop = html();
    expect(withDrop).toContain('data-testid="char-effective-str"');
    expect(clean(withDrop)).toMatch(/char-effective-str[^]*?→ 0/);
  });

  it('shows the Size readout only when it is off 0', () => {
    expect(html()).not.toContain('data-testid="characteristic-size"');

    store.effective = { size: 1 } as unknown as EffectiveScores;
    expect(html()).toContain('data-testid="characteristic-size"');
  });
});
