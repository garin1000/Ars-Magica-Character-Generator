import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

import { SCHEMA_VERSION, store } from '../state.svelte';
import MythicCompanionTypeSelector from './MythicCompanionTypeSelector.svelte';

const TYPE = 'mythic_type.nephilim';
const VIRTUE_A = 'virtue.divine_guidance';
const VIRTUE_B = 'virtue.true_faith';
const OPEN_VIRTUE = 'virtue.blessing_of_the_divine';
const FLAW_DEFAULT = 'flaw.wrathful';
const FLAW_SUBSTITUTE = 'flaw.pride';
const FIXED_VIRTUE = 'virtue.mythic_blood';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      type_profiles: {},
      abilities: {},
      point_items: {
        [VIRTUE_A]: {
          id: VIRTUE_A,
          kind: 'virtue',
          magnitude: 'minor',
          categories: ['supernatural'],
        },
        [VIRTUE_B]: {
          id: VIRTUE_B,
          kind: 'virtue',
          magnitude: 'minor',
          categories: ['supernatural'],
        },
        [OPEN_VIRTUE]: {
          id: OPEN_VIRTUE,
          kind: 'virtue',
          magnitude: 'minor',
          categories: ['divine'],
        },
        [FLAW_DEFAULT]: {
          id: FLAW_DEFAULT,
          kind: 'flaw',
          magnitude: 'minor',
          categories: ['personality'],
        },
        [FLAW_SUBSTITUTE]: {
          id: FLAW_SUBSTITUTE,
          kind: 'flaw',
          magnitude: 'minor',
          categories: ['personality'],
        },
        [FIXED_VIRTUE]: {
          id: FIXED_VIRTUE,
          kind: 'virtue',
          magnitude: 'minor',
          categories: ['supernatural'],
        },
      },
      mythic_companion_types: {
        [TYPE]: {
          id: TYPE,
          grants: [
            { kind: 'fixed', item: FIXED_VIRTUE },
            {
              kind: 'choice',
              choice_key: 'mythic_choice',
              options: [{ ref: VIRTUE_A }, { ref: VIRTUE_B }],
            },
            {
              kind: 'open',
              choice_key: 'mythic_open',
              constraint: { kind: 'virtue', require_categories: ['divine'] },
            },
          ],
          required_flaws: [
            {
              default: { ref: FLAW_DEFAULT },
              constraint: { kind: 'flaw', require_categories: ['personality'] },
            },
          ],
        },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      [TYPE]: { name: 'Nephilim' },
      [VIRTUE_A]: { name: 'Divine Guidance' },
      [VIRTUE_B]: { name: 'True Faith' },
      [OPEN_VIRTUE]: { name: 'Blessing of the Divine' },
      [FLAW_DEFAULT]: { name: 'Wrathful' },
      [FLAW_SUBSTITUTE]: { name: 'Pride' },
      [FIXED_VIRTUE]: { name: 'Mythic Blood' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'mythic_companion',
    mythic_type: TYPE,
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
  return render(MythicCompanionTypeSelector, { props: {} }).body;
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

/** The `aria-labelledby` id and the visible text of the element it names. */
function labelledbyText(body: string, testid: string): { id: string; text: string } {
  const select = new RegExp(`<select[^>]*data-testid="${testid}"[^>]*>`).exec(body);
  if (!select) throw new Error(`no select ${testid}`);
  const labelledby = /aria-labelledby="([^"]+)"/.exec(select[0]);
  if (!labelledby) throw new Error(`select ${testid} has no aria-labelledby`);
  const labelTag = new RegExp(`<[a-z]+[^>]*id="${labelledby[1]}"[^>]*>([^<]*)<`).exec(body);
  if (!labelTag) throw new Error(`no element with id ${labelledby[1]}`);
  return { id: labelledby[1], text: labelTag[1] };
}

// S19/S20/S21 (full-audit a11y): three grant/required-Flaw selects each sit next
// to a visible span (the "Granted"/"Required Flaw" label the fixed-grant case
// already uses) that carried no programmatic association at all — a
// screen-reader user landing on any of them heard only "combo box". Each span
// now gets an `id` and the neighbouring select an `aria-labelledby` pointing at
// it.
describe('MythicCompanionTypeSelector grant/required-flaw selects (S19/S20/S21)', () => {
  it('associates the choice-grant select with the visible "Granted" label (S19)', () => {
    const body = html();
    const { text } = labelledbyText(body, 'mythic-choice-mythic_choice');
    expect(text).toBe('Granted');
  });

  it('associates the open-grant select with the visible "Granted" label (S20)', () => {
    const body = html();
    const { text } = labelledbyText(body, 'mythic-open-mythic_open');
    expect(text).toBe('Granted');
  });

  it('associates the required-Flaw select with the visible "Required Flaw" label (S21)', () => {
    const body = html();
    const { text } = labelledbyText(body, `mythic-required-flaw-${FLAW_DEFAULT}`);
    expect(text).toBe('Required Flaw');
  });

  it('gives all three selects distinct label ids', () => {
    const body = html();
    const ids = [
      labelledbyText(body, 'mythic-choice-mythic_choice').id,
      labelledbyText(body, 'mythic-open-mythic_open').id,
      labelledbyText(body, `mythic-required-flaw-${FLAW_DEFAULT}`).id,
    ];
    expect(new Set(ids).size).toBe(3);
  });
});

// G12 (full-audit test-adequacy): MythicCompanionTypeSelector had no test file
// beyond the S19/S20/S21 a11y fix above. Its type-picker empty state, fixed
// grant, and the required-Flaw default-vs-substitute logic were exercised only
// via the slow e2e layer (`mythic-companion.e2e.js`).
describe('MythicCompanionTypeSelector type selection and grants', () => {
  it('shows no grants/required-Flaws section when no type is selected', () => {
    store.entity.mythic_type = undefined;
    const body = html();
    expect(body).toContain('data-testid="mythic-type-selector"');
    expect(body).not.toContain('data-testid="mythic-granted-');
    expect(body).not.toContain('mythic-required-flaw-');
  });

  it("renders the selected type's fixed grant", () => {
    const body = html();
    expect(body).toContain(`data-testid="mythic-granted-${FIXED_VIRTUE}"`);
    expect(body).toContain('Mythic Blood');
  });

  it('defaults the required-Flaw select to the rules default when none is bought', () => {
    const body = html();
    const select = new RegExp(
      `<select[^>]*data-testid="mythic-required-flaw-${FLAW_DEFAULT}"[\\s\\S]*?</select>`,
    ).exec(body)![0];
    expect(select).toMatch(new RegExp(`<option value="${FLAW_DEFAULT}" selected`));
  });

  it('shows the bought eligible Flaw as the current pick, not the rules default', () => {
    // The player bought Pride instead of the default Wrathful — Pride satisfies
    // the same "personality" constraint, so it must show as the current pick.
    store.entity.selections = [{ ref: FLAW_SUBSTITUTE }];
    const body = html();
    const select = new RegExp(
      `<select[^>]*data-testid="mythic-required-flaw-${FLAW_DEFAULT}"[\\s\\S]*?</select>`,
    ).exec(body)![0];
    expect(select).toMatch(new RegExp(`<option value="${FLAW_SUBSTITUTE}" selected`));
  });
});
