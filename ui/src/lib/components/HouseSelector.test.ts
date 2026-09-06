import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

import { SCHEMA_VERSION, store } from '../state.svelte';
import HouseSelector from './HouseSelector.svelte';

const HOUSE = 'house.jerbiton';
const VIRTUE_A = 'virtue.gentle_gift';
const VIRTUE_B = 'virtue.privileged_upbringing';
const OPEN_VIRTUE = 'virtue.affinity_with_art';
const FIXED_VIRTUE = 'virtue.hermetic_prestige';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      type_profiles: {},
      abilities: {},
      point_items: {
        [VIRTUE_A]: { id: VIRTUE_A, kind: 'virtue', magnitude: 'minor', categories: ['social'] },
        [VIRTUE_B]: { id: VIRTUE_B, kind: 'virtue', magnitude: 'minor', categories: ['social'] },
        [OPEN_VIRTUE]: {
          id: OPEN_VIRTUE,
          kind: 'virtue',
          magnitude: 'minor',
          categories: ['hermetic'],
        },
        [FIXED_VIRTUE]: {
          id: FIXED_VIRTUE,
          kind: 'virtue',
          magnitude: 'minor',
          categories: ['hermetic'],
        },
      },
      houses: {
        [HOUSE]: {
          id: HOUSE,
          lineage_type: 'true_lineage',
          grants: [
            { kind: 'fixed', item: FIXED_VIRTUE },
            {
              kind: 'choice',
              choice_key: 'house_choice',
              options: [{ ref: VIRTUE_A }, { ref: VIRTUE_B }],
            },
            {
              kind: 'open',
              choice_key: 'house_open',
              constraint: { kind: 'virtue', require_categories: ['hermetic'] },
            },
          ],
        },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      [HOUSE]: { name: 'Jerbiton' },
      [VIRTUE_A]: { name: 'The Gentle Gift' },
      [VIRTUE_B]: { name: 'Privileged Upbringing' },
      [OPEN_VIRTUE]: { name: 'Affinity with an Art' },
      [FIXED_VIRTUE]: { name: 'Hermetic Prestige' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    house: HOUSE,
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
  return render(HouseSelector, { props: {} }).body;
}

/** The `id` attribute of the element whose start tag also contains `marker`. */
function idOf(body: string, marker: string): string {
  const tag = new RegExp(`<[a-z]+[^>]*${marker}[^>]*>`).exec(body);
  if (!tag) throw new Error(`no element matching ${marker}`);
  const id = /\bid="([^"]+)"/.exec(tag[0]);
  if (!id) throw new Error(`element matching ${marker} has no id: ${tag[0]}`);
  return id[1];
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// S11/S12 (full-audit a11y): the choice- and open-grant `<select>`s in the House
// specialisation picker carried no accessible name at all — unlike the `fixed`
// grant case a few lines above, which shows a visible "Granted" span. Both
// selects now get that same visible, Fluent-sourced label, associated via
// `aria-labelledby` rather than a silent duplicate `aria-label`.
describe('HouseSelector grant selects (S11/S12)', () => {
  it('associates the choice-grant select with a visible "Granted" label', () => {
    const body = html();
    const select = /<select[^>]*data-testid="house-choice-house_choice"[^>]*>/.exec(body);
    expect(select).not.toBeNull();
    const labelledby = /aria-labelledby="([^"]+)"/.exec(select![0]);
    expect(labelledby).not.toBeNull();
    expect(idOf(body, `id="${labelledby![1]}"`)).toBe(labelledby![1]);
    // The referenced element actually carries the "Granted" text.
    const labelTag = new RegExp(`<[a-z]+[^>]*id="${labelledby![1]}"[^>]*>([^<]*)<`).exec(body);
    expect(labelTag![1]).toBe('Granted');
  });

  it('associates the open-grant select with a visible "Granted" label', () => {
    const body = html();
    const select = /<select[^>]*data-testid="house-open-house_open"[^>]*>/.exec(body);
    expect(select).not.toBeNull();
    const labelledby = /aria-labelledby="([^"]+)"/.exec(select![0]);
    expect(labelledby).not.toBeNull();
    const labelTag = new RegExp(`<[a-z]+[^>]*id="${labelledby![1]}"[^>]*>([^<]*)<`).exec(body);
    expect(labelTag![1]).toBe('Granted');
  });

  it('gives the choice and open grants distinct label ids', () => {
    const body = html();
    const choiceSelect = /<select[^>]*data-testid="house-choice-house_choice"[^>]*>/.exec(body)![0];
    const openSelect = /<select[^>]*data-testid="house-open-house_open"[^>]*>/.exec(body)![0];
    const choiceLabelledby = /aria-labelledby="([^"]+)"/.exec(choiceSelect)![1];
    const openLabelledby = /aria-labelledby="([^"]+)"/.exec(openSelect)![1];
    expect(choiceLabelledby).not.toBe(openLabelledby);
  });
});

// G12 (full-audit test-adequacy): HouseSelector had no test file beyond the
// S11/S12 a11y fix above — its three grant kinds (fixed/choice/open) and the
// empty/no-house state were exercised only via the slow e2e layer
// (`houses.e2e.js`). These assert the same branches at the fast SSR layer.
describe('HouseSelector grant kinds and selection state', () => {
  it('shows no-house-selected empty state when the entity has no house', () => {
    store.entity.house = undefined;
    const body = html();
    expect(body).toContain('data-testid="house-selector"');
    expect(body).not.toContain('data-testid="house-granted-');
    expect(body).toContain(store.t('house-none-selected'));
  });

  it("renders the selected house's description and a fixed grant's item name", () => {
    const body = html();
    expect(body).toContain(`data-testid="house-granted-${FIXED_VIRTUE}"`);
    expect(body).toContain('Hermetic Prestige');
  });

  it('reflects a stored choice-grant pick as the selected option', () => {
    store.entity.house_choices = { house_choice: { ref: VIRTUE_B } };
    const body = html();
    const select = /<select[^>]*data-testid="house-choice-house_choice"[\s\S]*?<\/select>/.exec(
      body,
    )![0];
    // VIRTUE_B is options[1], so pickedIndex resolves to 1: that <option> alone
    // carries `selected` (Svelte SSR marks the chosen <option>, not a `value`
    // attribute on the <select> itself).
    expect(select).toMatch(/<option value="1" selected/);
    expect(select).not.toMatch(/<option value="0" selected/);
  });

  it('leaves the choice-grant select unpicked (-1) when nothing is stored yet', () => {
    const body = html();
    const select = /<select[^>]*data-testid="house-choice-house_choice"[\s\S]*?<\/select>/.exec(
      body,
    )![0];
    expect(select).toMatch(/<option value="-1" selected/);
  });

  it('reflects a stored open-grant pick as the selected option', () => {
    store.entity.house_choices = { house_open: { ref: OPEN_VIRTUE } };
    const body = html();
    const select = /<select[^>]*data-testid="house-open-house_open"[\s\S]*?<\/select>/.exec(
      body,
    )![0];
    expect(select).toMatch(new RegExp(`<option value="${OPEN_VIRTUE}" selected`));
  });

  it('offers only items eligible under the open grant constraint', () => {
    // VIRTUE_A/VIRTUE_B are 'social' category; only FIXED_VIRTUE/OPEN_VIRTUE are
    // 'hermetic', the constraint this house's open grant requires.
    const body = html();
    const select = /<select[^>]*data-testid="house-open-house_open"[\s\S]*?<\/select>/.exec(
      body,
    )![0];
    expect(select).toContain('Affinity with an Art');
    expect(select).not.toContain('The Gentle Gift');
    expect(select).not.toContain('Privileged Upbringing');
  });
});

// max_total slice: the open-grant menu must not re-offer an item already at
// its total ceiling — the engine's `too_many_selections` validator would
// reject the pick the instant it landed. Mirrors houses.e2e.js's Ex
// Miscellanea case (an item with ZERO copies must stay offered), which the
// second test below guards directly.
describe('HouseSelector open grant respects max_total', () => {
  function capOpenVirtue(maxTotal: number): void {
    store.ruleset!.ruleset.point_items[OPEN_VIRTUE] = {
      ...store.ruleset!.ruleset.point_items[OPEN_VIRTUE],
      max_total: maxTotal,
    };
  }

  function openSelect(body: string): string {
    return /<select[^>]*data-testid="house-open-house_open"[\s\S]*?<\/select>/.exec(body)![0];
  }

  it('drops an item whose bought copies already reached max_total', () => {
    capOpenVirtue(1);
    store.entity.selections = [{ ref: OPEN_VIRTUE }];
    expect(openSelect(html())).not.toContain('Affinity with an Art');
  });

  it('still offers it while the character holds zero copies', () => {
    capOpenVirtue(1);
    expect(openSelect(html())).toContain('Affinity with an Art');
  });

  // Self-exclusion: once resolved, this OPEN grant's own pick is itself folded
  // into `effective.granted_selections` (mirroring the engine's
  // `resolve_grants`). Without excluding it from the cap count, an item whose
  // max_total is reached BY THIS VERY PICK ALONE would vanish from its own
  // `<select>`'s option list, leaving the control showing no selection even
  // though the pick is still stored.
  it('keeps the slot’s own current pick offered even though it alone reaches max_total', () => {
    capOpenVirtue(1);
    store.entity.house_choices = { house_open: { ref: OPEN_VIRTUE } };
    store.effective = {
      granted_selections: [{ ref: OPEN_VIRTUE }],
    } as unknown as EffectiveScores;
    expect(openSelect(html())).toContain('Affinity with an Art');
  });
});
