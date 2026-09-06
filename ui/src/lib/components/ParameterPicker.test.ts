import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  Ability,
  Art,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  ParameterDef,
  PointItem,
  Selection,
} from '../types';

// ParameterPicker reads the shared store singleton (the ruleset's Art / point-item
// catalogues, the entity's own rows) and the Fluent bundle. The store schedules a
// debounced revalidate over the Tauri IPC bridge; mock the bridge so nothing reaches
// a backend. Harness mirrors MagusMinimumAbilities.test.ts.
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
import ParameterPicker from './ParameterPicker.svelte';
import VirtueFlawTab from './VirtueFlawTab.svelte';
import WizardStep from './WizardStep.svelte';

/** Two Techniques and two Forms, so a domain that filters can be caught filtering. */
const ARTS: Record<string, Art> = {
  'art.creo': { id: 'art.creo', art_type: 'technique' } as unknown as Art,
  'art.rego': { id: 'art.rego', art_type: 'technique' } as unknown as Art,
  'art.aquam': { id: 'art.aquam', art_type: 'form' } as unknown as Art,
  'art.ignem': { id: 'art.ignem', art_type: 'form' } as unknown as Art,
};

/** One plain ability and one parameterized one, so instance handling is exercised. */
const ABILITIES: Record<string, Ability> = {
  'ability.awareness': { id: 'ability.awareness', category: 'general' },
  'ability.stealth': { id: 'ability.stealth', category: 'general' },
  'ability.area_lore': { id: 'ability.area_lore', category: 'general', parameter: 'area' },
};

function pointItem(id: string, parameters: ParameterDef[]): PointItem {
  return {
    id,
    kind: 'virtue',
    magnitude: 'minor',
    categories: ['hermetic'],
    classification: 'narrative',
    entity_kinds: ['character'],
    parameters,
  } as unknown as PointItem;
}

/** Every domain under test, one catalogue item each, so no branch shares a fixture. */
const ITEMS: Record<string, PointItem> = {
  'virtue.deft_form': pointItem('virtue.deft_form', [{ key: 'form', type: 'ref', domain: 'form' }]),
  'flaw.deficient_technique': pointItem('flaw.deficient_technique', [
    { key: 'technique', type: 'ref', domain: 'technique' },
  ]),
  'virtue.item_domain_probe': pointItem('virtue.item_domain_probe', [
    { key: 'item', type: 'ref', domain: 'item' },
  ]),
  'virtue.ways_of_the_land': pointItem('virtue.ways_of_the_land', [
    { key: 'land', type: 'ref', domain: 'text' },
  ]),
  'virtue.puissant_ability': pointItem('virtue.puissant_ability', [
    { key: 'ability', type: 'ref', domain: 'ability' },
  ]),
  // The `enumerated` domain declares its own closed list, so the option set comes
  // from the DATA, not from any catalogue the store holds. Three values, so a
  // taken one can be seen greyed while others stay offered.
  'virtue.folk_magic': pointItem('virtue.folk_magic', [
    {
      key: 'category',
      type: 'ref',
      domain: 'enumerated',
      values: ['folk_magic.abjuration', 'folk_magic.divination', 'folk_magic.healing'],
    },
  ]),
};

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: ITEMS,
      type_profiles: {
        magus: {
          id: 'magus',
          budget: { virtue_points: 10, flaw_points: 10 },
          is_magus: true,
          gift_policy: 'required',
          creation_phases: [],
        },
      },
      abilities: ABILITIES,
      arts: ARTS,
      spells: {},
      houses: {},
      mythic_types: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      'art.creo': { name: 'Creo' },
      'art.rego': { name: 'Rego' },
      'art.aquam': { name: 'Aquam' },
      'art.ignem': { name: 'Ignem' },
      'virtue.deft_form': { name: 'Deft {form}' },
      'flaw.deficient_technique': { name: 'Deficient {technique}' },
      'virtue.item_domain_probe': { name: 'Probe {item}' },
      'virtue.ways_of_the_land': { name: 'Ways Of The {land}' },
      'virtue.puissant_ability': { name: 'Puissant {ability}' },
      'ability.awareness': { name: 'Awareness' },
      'ability.stealth': { name: 'Stealth' },
      'ability.area_lore': { name: '{area} Lore' },
      'virtue.folk_magic': { name: 'Folk Magic {category}' },
      'folk_magic.abjuration': { name: 'Abjuration' },
      'folk_magic.divination': { name: 'Divination' },
      'folk_magic.healing': { name: 'Healing' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
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
  store.effective = null;
  store.result = { issues: [] };
}

/** Render the picker for one catalogue item's parameter list. */
function pickerBody(ref: string, index = 0, params?: Record<string, string>): string {
  const selection: Selection = params ? { ref, params } : { ref };
  return render(ParameterPicker, {
    props: { selection, index, params: ITEMS[ref].parameters! },
  }).body;
}

/**
 * Render the picker in GRANT-PICK mode: `index === -1` and a `commit` callback,
 * exactly as HouseSelector/MythicCompanionTypeSelector/CharacterDetails mount it
 * for an open-grant/warping-owed fill (see the doc comment above `usage()` in
 * ParameterPicker.svelte). `commit` is never invoked under SSR (no interaction
 * fires), so a no-op stands in for the real store mutator.
 */
function pickerBodyGrant(ref: string, params: Record<string, string>, idSuffix = 'grant'): string {
  const selection: Selection = { ref, params };
  return render(ParameterPicker, {
    props: {
      selection,
      index: -1,
      params: ITEMS[ref].parameters!,
      idSuffix,
      commit: () => {},
    },
  }).body;
}

/** The `<option>` whose visible text is `label`, or null when it is not offered. */
function optionByText(select: string, label: string): string | null {
  return (
    [...select.matchAll(/<option[^>]*>([\s\S]*?)<\/option>/g)].find(
      (m) =>
        m[1]
          .replace(/<[^>]*>/g, '')
          .replace(/[⁦-⁩]/g, '')
          .trim() === label,
    )?.[0] ?? null
  );
}

/** The whole `<select data-testid="…">…</select>` element, or null when absent. */
function selectFor(body: string, testid: string): string | null {
  const re = new RegExp(`<select[^>]*data-testid="${testid}"[^>]*>[\\s\\S]*?</select>`, 'i');
  return re.exec(body)?.[0] ?? null;
}

/** Whether an `<input>` carries the testid — i.e. the fall-through text branch fired. */
function hasInput(body: string, testid: string): boolean {
  return new RegExp(`<input[^>]*data-testid="${testid}"`, 'i').test(body);
}

/** Visible option texts, in document order — the empty prompt included. */
function optionTexts(select: string): string[] {
  return [...select.matchAll(/<option[^>]*>([\s\S]*?)<\/option>/g)].map((m) =>
    m[1]
      .replace(/<[^>]*>/g, '')
      .replace(/[⁦-⁩]/g, '')
      .trim(),
  );
}

/** The `aria-label` of an element's opening tag. */
function ariaLabel(element: string): string {
  return /aria-label="([^"]*)"/.exec(element)?.[1] ?? '';
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  store.result = null;
});

describe('ParameterPicker domain branches (slice 7, #4)', () => {
  it('renders a Forms-only select for a form-domain parameter', () => {
    const select = selectFor(pickerBody('virtue.deft_form'), 'param-virtue.deft_form-form-0');
    expect(select).not.toBeNull();
    // Localized Form names, never the `art.ignem` slug: the value attribute may carry
    // the id, the text a player reads may not.
    expect(optionTexts(select!)).toEqual(['Form', 'Aquam', 'Ignem']);
    expect(optionTexts(select!).join(' ')).not.toContain('art.');
    // A Technique is not a legal Deft (Form) target and must not be offered at all.
    expect(select!).not.toContain('art.creo');
    expect(select!).not.toContain('art.rego');
  });

  it('renders a Techniques-only select for a technique-domain parameter', () => {
    const select = selectFor(
      pickerBody('flaw.deficient_technique'),
      'param-flaw.deficient_technique-technique-0',
    );
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toEqual(['Technique', 'Creo', 'Rego']);
    expect(select!).not.toContain('art.ignem');
    expect(select!).not.toContain('art.aquam');
  });

  it('renders a select, not a text input, for an item-domain parameter', () => {
    const body = pickerBody('virtue.item_domain_probe');
    const testid = 'param-virtue.item_domain_probe-item-0';
    // The domain resolves against the point-item registry, so a typed string could
    // only ever be an internal slug — the enum is exhaustive and this branch exists
    // for it even though no shipped catalogue entry uses it yet.
    expect(hasInput(body, testid)).toBe(false);
    const select = selectFor(body, testid);
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toContain('Item');
    expect(optionTexts(select!).join(' ')).not.toContain('virtue.');
  });

  it('renders a text input only for the text domain', () => {
    const body = pickerBody('virtue.ways_of_the_land');
    const testid = 'param-virtue.ways_of_the_land-land-0';
    // `text` references no registry, so free text is correct here and nowhere else.
    expect(hasInput(body, testid)).toBe(true);
    expect(selectFor(body, testid)).toBeNull();
  });

  it('names each control in the active language, never by its param key', () => {
    store.lang = 'de';
    const form = selectFor(pickerBody('virtue.deft_form'), 'param-virtue.deft_form-form-0');
    expect(ariaLabel(form!)).toBe('Form');
    const technique = selectFor(
      pickerBody('flaw.deficient_technique'),
      'param-flaw.deficient_technique-technique-0',
    );
    expect(ariaLabel(technique!)).toBe('Technik');
    const item = selectFor(
      pickerBody('virtue.item_domain_probe'),
      'param-virtue.item_domain_probe-item-0',
    );
    expect(ariaLabel(item!)).toBe('Gegenstand');
    const category = selectFor(
      pickerBody('virtue.folk_magic'),
      'param-virtue.folk_magic-category-0',
    );
    expect(ariaLabel(category!)).toBe('Kategorie');
  });
});

// Folk Magic's spell category and the three (Beings) classes are closed lists the
// rulebook prints in full, declared on the parameter itself. The picker's option
// set is therefore the DATA's list — nothing narrows a catalogue.
describe('ParameterPicker enumerated domain', () => {
  const TESTID = 'param-virtue.folk_magic-category-0';

  it('offers exactly the values the parameter declares, in order', () => {
    const body = pickerBody('virtue.folk_magic');
    // Free text was the old control and is exactly what this domain replaces.
    expect(hasInput(body, TESTID)).toBe(false);
    const select = selectFor(body, TESTID);
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toEqual(['Category', 'Abjuration', 'Divination', 'Healing']);
  });

  it('labels each option through the rules i18n, never as its raw id', () => {
    const select = selectFor(pickerBody('virtue.folk_magic'), TESTID);
    // The value attribute carries the id; the text a player reads must not.
    expect(select!).toContain('value="folk_magic.abjuration"');
    expect(optionTexts(select!).join(' ')).not.toContain('folk_magic.');
  });

  it('greys out a value another selection of the same item already holds', () => {
    store.entity.selections = [
      { ref: 'virtue.folk_magic', params: { category: 'folk_magic.healing' } },
      { ref: 'virtue.folk_magic' },
    ];
    const select = selectFor(
      pickerBody('virtue.folk_magic', 1),
      'param-virtue.folk_magic-category-1',
    );
    expect(optionByText(select!, 'Healing')).toContain('disabled');
    expect(optionByText(select!, 'Abjuration')).not.toContain('disabled');
  });

  it('counts a GRANTED copy against the same list', () => {
    // The greying path is domain-agnostic and grant-aware: `usage()` merges bought
    // rows with `store.effective.granted_selections`, so a House-granted category
    // must disappear from a bought row's menu exactly as a bought one does.
    store.entity.selections = [{ ref: 'virtue.folk_magic' }];
    store.effective = {
      granted_selections: [
        { ref: 'virtue.folk_magic', params: { category: 'folk_magic.healing' } },
      ],
    } as unknown as EffectiveScores;
    const select = selectFor(pickerBody('virtue.folk_magic'), TESTID);
    expect(optionByText(select!, 'Healing')).toContain('disabled');
    expect(optionByText(select!, 'Divination')).not.toContain('disabled');
  });
});

// manual-testing-findings-2026-09-03 #5: the ability domain used to offer ONLY the
// abilities already on the sheet, while abilities are bought on a later step — so
// Puissant Ability could not be completed where it is taken and the wizard deadlocked.
// Puissant Ability is "choose one Ability" with no requirement that a score exists
// (Ars Magica - Definitive Edition (Core Rules).md:4814-4816), exactly like the `art`
// domain, which never required the Art to be on the sheet.
describe('ParameterPicker ability domain (manual-testing-findings-2026-09-03 #5)', () => {
  const TESTID = 'param-virtue.puissant_ability-ability-0';

  it('offers the whole ability catalogue, not only the abilities the character has', () => {
    const select = selectFor(pickerBody('virtue.puissant_ability'), TESTID);
    expect(select).not.toBeNull();
    // No ability_scores at all, and every catalogue ability is still offered.
    expect(optionTexts(select!)).toContain('Awareness');
    expect(optionTexts(select!)).toContain('Stealth');
    expect(optionTexts(select!).join(' ')).not.toContain('ability.');
  });

  it('offers a parameterized ability before any instance of it exists', () => {
    const select = selectFor(pickerBody('virtue.puissant_ability'), TESTID);
    // The name template's placeholder stands in for the unfilled area.
    expect(optionTexts(select!)).toContain('(Area) Lore');
  });

  it("still lists the character's own instances of a parameterized ability", () => {
    store.entity.ability_scores = [
      { ability: 'ability.area_lore', score: 2, parameter: 'Brandenburg' },
    ] as Entity['ability_scores'];
    const select = selectFor(pickerBody('virtue.puissant_ability'), TESTID);
    expect(optionTexts(select!)).toContain('Brandenburg Lore');
    // …and the generic entry stays, so a second area can still be chosen.
    expect(optionTexts(select!)).toContain('(Area) Lore');
  });

  it('lists an owned plain ability exactly once', () => {
    store.entity.ability_scores = [
      { ability: 'ability.awareness', score: 2 },
    ] as Entity['ability_scores'];
    const select = selectFor(pickerBody('virtue.puissant_ability'), TESTID);
    expect(optionTexts(select!).filter((t) => t === 'Awareness')).toHaveLength(1);
  });

  // max_per_target is value-keyed, so it must keep working now that the values
  // include catalogue entries no ability row backs.
  it('disables a target another selection of the same item already claims', () => {
    store.entity.selections = [
      { ref: 'virtue.puissant_ability', params: { ability: 'ability.awareness' } },
      { ref: 'virtue.puissant_ability' },
    ];
    // Second row, so its test id carries index 1.
    const select = selectFor(
      pickerBody('virtue.puissant_ability', 1),
      'param-virtue.puissant_ability-ability-1',
    );
    expect(optionByText(select!, 'Awareness')).toContain('disabled');
    expect(optionByText(select!, 'Stealth')).not.toContain('disabled');
  });

  it('leaves a parameterized ability open once one of its instances is claimed', () => {
    store.entity.ability_scores = [
      { ability: 'ability.area_lore', score: 2, parameter: 'Brandenburg' },
    ] as Entity['ability_scores'];
    store.entity.selections = [
      {
        ref: 'virtue.puissant_ability',
        params: { ability: 'ability.area_lore', area: 'Brandenburg' },
      },
      { ref: 'virtue.puissant_ability' },
    ];
    // Second row, so its test id carries index 1.
    const select = selectFor(
      pickerBody('virtue.puissant_ability', 1),
      'param-virtue.puissant_ability-ability-1',
    );
    expect(optionByText(select!, 'Brandenburg Lore')).toContain('disabled');
    expect(optionByText(select!, '(Area) Lore')).not.toContain('disabled');
  });

  // Choosing the generic entry leaves the instance discriminator unset, which the
  // engine reports as `missing_param` — so the picker must offer somewhere to put it,
  // or the catalogue entry would be a new dead end.
  it('asks for the instance value once a parameterized target is chosen', () => {
    const body = pickerBody('virtue.puissant_ability', 0, { ability: 'ability.area_lore' });
    const testid = 'param-virtue.puissant_ability-area-0';
    expect(hasInput(body, testid)).toBe(true);
    const input = /<input[^>]*data-testid="param-virtue\.puissant_ability-area-0"[^>]*>/.exec(
      body,
    )![0];
    // Named for assistive tech through the param-label key, never the raw `area` slug.
    expect(ariaLabel(input)).toBe('Area');
  });

  it('asks for no instance value on a plain target', () => {
    const body = pickerBody('virtue.puissant_ability', 0, { ability: 'ability.awareness' });
    expect(body).not.toContain('param-virtue.puissant_ability-area-0');
  });
});

describe('ParameterPicker on both mounts (slice 7, #4 — shared component)', () => {
  // ParameterPicker is shared, so #4 was an edit-mode bug as much as a wizard one.
  // `VirtueFlawTab` is the editor's Virtues & Flaws tab; `WizardStep` mounts the very
  // same component for the `virtues_flaws` phase. Both must show the Forms select.
  beforeEach(() => {
    store.entity.selections = [{ ref: 'virtue.deft_form' }];
  });

  it('renders the Forms select on the editor tab', () => {
    const select = selectFor(
      render(VirtueFlawTab, { props: {} }).body,
      'param-virtue.deft_form-form-0',
    );
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toEqual(['Form', 'Aquam', 'Ignem']);
  });

  it('renders the Forms select on the wizard step', () => {
    const select = selectFor(
      render(WizardStep, { props: { phase: 'virtues_flaws' } }).body,
      'param-virtue.deft_form-form-0',
    );
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toEqual(['Form', 'Aquam', 'Ignem']);
  });
});

// The UI half of the max_total slice's engine fix (`too_many_selections` is now
// grant-aware): `usage()`/`usedAbilityTargets` used to read ONLY
// `entity.selections`, so a House-granted Puissant Ignem never counted against
// a bought Puissant Art's per-target cap — the same illegal state
// `validate_duplicate_selections` now catches on the engine side, reachable
// through the UI a moment before revalidation caught up.
describe('ParameterPicker folds granted rows into per-target usage (grant-awareness)', () => {
  afterEach(() => {
    store.effective = null;
  });

  it('disables an art-domain target already claimed by a GRANTED copy of the same item', () => {
    store.effective = {
      granted_selections: [{ ref: 'virtue.deft_form', params: { form: 'art.ignem' } }],
    } as unknown as EffectiveScores;
    const select = selectFor(pickerBody('virtue.deft_form', 0), 'param-virtue.deft_form-form-0');
    expect(optionByText(select!, 'Ignem')).toContain('disabled');
    expect(optionByText(select!, 'Aquam')).not.toContain('disabled');
  });

  it('disables an ability-domain target already claimed by a GRANTED copy of the same item', () => {
    store.effective = {
      granted_selections: [
        { ref: 'virtue.puissant_ability', params: { ability: 'ability.awareness' } },
      ],
    } as unknown as EffectiveScores;
    const select = selectFor(
      pickerBody('virtue.puissant_ability', 0),
      'param-virtue.puissant_ability-ability-0',
    );
    expect(optionByText(select!, 'Awareness')).toContain('disabled');
    expect(optionByText(select!, 'Stealth')).not.toContain('disabled');
  });

  // A grant pick passes `index === -1`, which excludes nothing from
  // `entity.selections` — so without excluding the pick from the GRANTED list
  // too, it would count against its own current target and greys out its own
  // value the instant it is set (see the comment above `usage()`).
  it('does not grey out a grant pick’s own current target against itself', () => {
    store.effective = {
      granted_selections: [{ ref: 'virtue.deft_form', params: { form: 'art.ignem' } }],
    } as unknown as EffectiveScores;
    const select = selectFor(
      pickerBodyGrant('virtue.deft_form', { form: 'art.ignem' }),
      'param-virtue.deft_form-form-grant',
    );
    expect(optionByText(select!, 'Ignem')).not.toContain('disabled');
    expect(optionByText(select!, 'Aquam')).not.toContain('disabled');
  });

  // A DIFFERENT granted copy (not this grant pick's own) must still count —
  // self-exclusion removes only the ONE occurrence matching this pick's value.
  it('still disables a target a DIFFERENT granted copy of the same item claims', () => {
    store.effective = {
      granted_selections: [
        { ref: 'virtue.deft_form', params: { form: 'art.ignem' } },
        { ref: 'virtue.deft_form', params: { form: 'art.aquam' } },
      ],
    } as unknown as EffectiveScores;
    const select = selectFor(
      pickerBodyGrant('virtue.deft_form', { form: 'art.ignem' }),
      'param-virtue.deft_form-form-grant',
    );
    expect(optionByText(select!, 'Ignem')).not.toContain('disabled');
    expect(optionByText(select!, 'Aquam')).toContain('disabled');
  });
});
