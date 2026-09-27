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

function pointItem(
  id: string,
  parameters: ParameterDef[],
  categories = ['hermetic'],
  tainted = false,
): PointItem {
  return {
    id,
    kind: 'virtue',
    magnitude: 'minor',
    categories,
    classification: 'narrative',
    entity_kinds: ['character'],
    tainted,
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
  // The same domain narrowed to a category (`require_categories`): the engine
  // resolves a value outside it to `unknown_param_value`, so offering the whole
  // registry would offer an illegal choice.
  'virtue.narrowed_probe': pointItem('virtue.narrowed_probe', [
    { key: 'item', type: 'ref', domain: 'item', require_categories: ['supernatural'] },
  ]),
  // D34: `allow_ids` additive to `require_categories` — False Power's own
  // shape. `virtue.diedne_probe` stands in for Diedne Magic (named outright,
  // ArMDE:6082, but `hermetic`, outside the required category);
  // `virtue.other_hermetic_probe` stands in for the 55 OTHER Hermetic Virtues
  // the bare category reading wrongly admitted and D34 closes off.
  'virtue.allow_ids_probe': pointItem('virtue.allow_ids_probe', [
    {
      key: 'item',
      type: 'ref',
      domain: 'item',
      require_categories: ['supernatural'],
      allow_ids: ['virtue.diedne_probe'],
    },
  ]),
  'virtue.diedne_probe': pointItem('virtue.diedne_probe', []),
  'virtue.other_hermetic_probe': pointItem('virtue.other_hermetic_probe', []),
  // The one catalogue item of the required category, so a filter can be caught
  // filtering — every other fixture item is `hermetic`.
  'virtue.second_sight': pointItem('virtue.second_sight', [], ['supernatural']),
  // False Power's shape: the target must be a Virtue the character HOLDS
  // (`require_possessed`) and must not already be Infernal (`forbid_tainted`) —
  // ArMDE:6096. The engine refuses either, so the menu must not offer
  // them.
  'flaw.possessed_probe': pointItem('flaw.possessed_probe', [
    { key: 'virtue', type: 'ref', domain: 'item', require_possessed: true, forbid_tainted: true },
  ]),
  // Held but already Infernal — offered by neither the narrowed nor the
  // possession filter, and the control for `forbid_tainted`.
  'virtue.demonic_blood': pointItem('virtue.demonic_blood', [], ['supernatural'], true),
  // Reachable only as a House grant, so "possessed" can be seen counting a row
  // the player never bought.
  'virtue.granted_gift': pointItem('virtue.granted_gift', [], ['supernatural']),
  'virtue.ways_of_the_land': pointItem('virtue.ways_of_the_land', [
    { key: 'land', type: 'ref', domain: 'text' },
  ]),
  // D35: Simple Student's shape — a bounded integer count, 1-2.
  'virtue.simple_student_probe': pointItem('virtue.simple_student_probe', [
    { key: 'years', type: { number: { min: 1, max: 2 } }, domain: 'number' },
  ]),
  'virtue.puissant_ability': pointItem('virtue.puissant_ability', [
    { key: 'ability', type: 'ref', domain: 'ability' },
  ]),
  // Two axes, exactly as the shipped entry has them (ArMDE:3909). The
  // `enumerated` spell category declares its own closed list, so its option set
  // comes from the DATA and not from any catalogue the store holds — three
  // values, so a taken one can be seen greyed while others stay offered. The
  // Realm its magic is aligned to is the closed `realm` taxonomy instead.
  'virtue.folk_magic': pointItem('virtue.folk_magic', [
    {
      key: 'category',
      type: 'ref',
      domain: 'enumerated',
      values: ['folk_magic.abjuration', 'folk_magic.divination', 'folk_magic.healing'],
    },
    {
      key: 'realm',
      type: 'ref',
      domain: 'realm',
      at_most_one_of: [['realm.divine', 'realm.infernal']],
    },
  ]),
  // Row 19 "taken as": Sufi (ArMDE:5083) "either as a Minor Social Status
  // Virtue or a Minor
  // Supernatural Virtue" — the `category` domain's own values are a subset of
  // the item's `categories`, labelled through the SAME `category-<id>` Fluent
  // family the V/F badge uses, never through rules i18n.
  'virtue.sufi': pointItem('virtue.sufi', [
    {
      key: 'taken_as',
      type: 'ref',
      domain: 'category',
      values: ['social_status', 'supernatural'],
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
          hermetically_trained: true,
          order_member: true,
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
      'virtue.narrowed_probe': { name: 'Narrowed Probe {item}' },
      'virtue.allow_ids_probe': { name: 'Allow Ids Probe {item}' },
      'virtue.diedne_probe': { name: 'Diedne Probe' },
      'virtue.other_hermetic_probe': { name: 'Other Hermetic Probe' },
      'virtue.second_sight': { name: 'Second Sight' },
      'flaw.possessed_probe': { name: 'Possessed Probe' },
      'virtue.demonic_blood': { name: 'Demonic Blood' },
      'virtue.granted_gift': { name: 'Granted Gift' },
      'virtue.ways_of_the_land': { name: 'Ways Of The {land}' },
      'virtue.simple_student_probe': { name: 'Simple Student Probe' },
      'virtue.puissant_ability': { name: 'Puissant {ability}' },
      'ability.awareness': { name: 'Awareness' },
      'ability.stealth': { name: 'Stealth' },
      'ability.area_lore': { name: '{area} Lore' },
      'virtue.folk_magic': { name: 'Folk Magic {category}' },
      'folk_magic.abjuration': { name: 'Abjuration' },
      'folk_magic.divination': { name: 'Divination' },
      'folk_magic.healing': { name: 'Healing' },
      'virtue.sufi': { name: 'Sufi' },
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
    saga_year: 1220,
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

/** The whole `<input data-testid="…">` element (self-closing), or null when absent. */
function inputFor(body: string, testid: string): string | null {
  const re = new RegExp(`<input[^>]*data-testid="${testid}"[^>]*>`, 'i');
  return re.exec(body)?.[0] ?? null;
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

  it('offers only items of the required category when the parameter narrows the domain', () => {
    const select = selectFor(
      pickerBody('virtue.narrowed_probe'),
      'param-virtue.narrowed_probe-item-0',
    );
    expect(select).not.toBeNull();
    // `require_categories: ['supernatural']` — the engine refuses anything else
    // as `unknown_param_value`, so the menu must not offer it either. All three
    // supernatural fixture items are offered, held or not and Tainted or not:
    // this parameter narrows by category ALONE.
    expect(optionTexts(select!)).toEqual(['Item', 'Demonic Blood', 'Granted Gift', 'Second Sight']);
  });

  // D34: `allow_ids` is additive to `require_categories`, read straight off the
  // ruleset data (`param.allow_ids`), never re-derived in TypeScript. False
  // Power's own shape: `require_categories: ['supernatural']` widened by two
  // named ids the category axis cannot reach.
  it('additionally offers whitelisted ids the category alone would refuse (D34)', () => {
    const select = selectFor(
      pickerBody('virtue.allow_ids_probe'),
      'param-virtue.allow_ids_probe-item-0',
    );
    expect(select).not.toBeNull();
    // The three supernatural fixtures resolve via `require_categories` alone;
    // Diedne Probe resolves ONLY via `allow_ids` despite being `hermetic`.
    expect(optionTexts(select!)).toEqual([
      'Item',
      'Demonic Blood',
      'Diedne Probe',
      'Granted Gift',
      'Second Sight',
    ]);
    // The whitelist is closed, not a fourth admitted category: an arbitrary
    // OTHER Hermetic Virtue outside both `require_categories` and `allow_ids`
    // must not be offered — the exact over-permission D34 closes.
    expect(optionTexts(select!)).not.toContain('Other Hermetic Probe');
  });

  it('offers only Virtues the character holds when the parameter requires possession', () => {
    // False Power taints "one of the character's Supernatural Virtues", taken
    // "once for each appropriate Supernatural Virtue that the character
    // possesses" (ArMDE:6096). Both non-offers matter: Demonic Blood is
    // HELD but already Infernal, and Granted Gift is un-held here.
    store.entity.selections = [{ ref: 'virtue.second_sight' }, { ref: 'virtue.demonic_blood' }];
    const select = selectFor(
      pickerBody('flaw.possessed_probe'),
      'param-flaw.possessed_probe-virtue-0',
    );
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toEqual(['Virtue', 'Second Sight']);
  });

  it('counts a granted row as possessed in the picker', () => {
    // The engine reads possession off the grants-inclusive `present_ids`, so a
    // House-granted Virtue is a legal target and must be offered as one.
    store.effective = {
      granted_selections: [{ ref: 'virtue.granted_gift' }],
    } as unknown as EffectiveScores;
    const select = selectFor(
      pickerBody('flaw.possessed_probe'),
      'param-flaw.possessed_probe-virtue-0',
    );
    expect(optionTexts(select!)).toEqual(['Virtue', 'Granted Gift']);
  });

  it('leaves an un-narrowed item-domain parameter offering the whole registry', () => {
    // The filter is per parameter, not global: the probe that declares no
    // `require_categories`, `require_possessed` or `forbid_tainted` still lists
    // the supernatural item, the Tainted one, and an item nobody holds.
    const select = selectFor(
      pickerBody('virtue.item_domain_probe'),
      'param-virtue.item_domain_probe-item-0',
    );
    expect(optionTexts(select!)).toContain('Second Sight');
    expect(optionTexts(select!)).toContain('Demonic Blood');
    expect(optionTexts(select!)).toContain('Granted Gift');
    expect(optionTexts(select!).length).toBeGreaterThan(2);
  });

  it('renders a text input only for the text domain', () => {
    const body = pickerBody('virtue.ways_of_the_land');
    const testid = 'param-virtue.ways_of_the_land-land-0';
    // `text` references no registry, so free text is correct here and nowhere else.
    expect(hasInput(body, testid)).toBe(true);
    expect(selectFor(body, testid)).toBeNull();
  });

  it('renders a bounded number input, not a select or a plain text box, for the number domain', () => {
    const body = pickerBody('virtue.simple_student_probe');
    const testid = 'param-virtue.simple_student_probe-years-0';
    const input = inputFor(body, testid);
    expect(input).not.toBeNull();
    expect(input).toContain('type="number"');
    expect(input).toContain('min="1"');
    expect(input).toContain('max="2"');
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

// B7 (row 12): Folk Magic's magic "is aligned to" a supernatural realm
// (ArMDE:3909), and the four Realms are a closed engine taxonomy with
// Fluent labels of their own — so the picker offers them from `REALMS`, labelled
// through `realm-<id>`, and never a text box in which only an internal slug
// would validate.
describe('ParameterPicker realm domain (row 12)', () => {
  const TESTID = 'param-virtue.folk_magic-realm-0';

  it('offers the four Realms, labelled via realm-<id>, never as a text box', () => {
    const body = pickerBody('virtue.folk_magic');
    expect(hasInput(body, TESTID)).toBe(false);
    const select = selectFor(body, TESTID);
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toEqual(['Realm', 'Magic', 'Faerie', 'Divine', 'Infernal']);
    // The value attribute carries the id; the text a player reads must not.
    expect(select!).toContain('value="realm.divine"');
    expect(optionTexts(select!).join(' ')).not.toContain('realm.');
  });

  it('names the control and its options in the active language', () => {
    store.lang = 'de';
    const select = selectFor(pickerBody('virtue.folk_magic'), TESTID);
    expect(ariaLabel(select!)).toBe('Sphäre');
    // Article-free, as the rulebook enumerates the four Sphären
    // (`Basisregeln.md:2960`) — round-1 audit, Sabine 6.
    expect(optionTexts(select!)).toContain('Göttlich');
  });

  // ":3919 — you can align it to the same Realm as before or pick a different
  // one". Greying a Realm another copy holds would forbid what that sentence
  // explicitly permits, so neither axis is greyed on its own: a target is the
  // whole parameter tuple, and these two copies differ in their category.
  it('leaves both axes open when another copy differs in the other one', () => {
    store.entity.selections = [
      {
        ref: 'virtue.folk_magic',
        params: { category: 'folk_magic.healing', realm: 'realm.magic' },
      },
      {
        ref: 'virtue.folk_magic',
        params: { category: 'folk_magic.abjuration', realm: 'realm.faerie' },
      },
    ];
    const body = pickerBody('virtue.folk_magic', 1, {
      category: 'folk_magic.abjuration',
      realm: 'realm.faerie',
    });
    const realms = selectFor(body, 'param-virtue.folk_magic-realm-1');
    expect(optionByText(realms!, 'Magic')).not.toContain('disabled');
    const categories = selectFor(body, 'param-virtue.folk_magic-category-1');
    expect(optionByText(categories!, 'Healing')).not.toContain('disabled');
  });

  // …and the target that IS already taken whole still greys out, so the row
  // above is a real distinction rather than greying simply having been removed.
  it('greys a value another copy holds when every other axis matches too', () => {
    store.entity.selections = [
      {
        ref: 'virtue.folk_magic',
        params: { category: 'folk_magic.healing', realm: 'realm.magic' },
      },
      { ref: 'virtue.folk_magic', params: { realm: 'realm.magic' } },
    ];
    const body = pickerBody('virtue.folk_magic', 1, { realm: 'realm.magic' });
    const categories = selectFor(body, 'param-virtue.folk_magic-category-1');
    expect(optionByText(categories!, 'Healing')).toContain('disabled');
    expect(optionByText(categories!, 'Abjuration')).not.toContain('disabled');
  });
});

// Row 19 "taken as": the `category` domain's menu is the item's own declared
// `values` (a subset of `categories`, enforced at load), but options are
// labelled through the `category-<id>` Fluent family rather than rules i18n —
// the sharp edge the comment on the `category` domain branch in
// `ParameterPicker.svelte` exists to prevent, since a bare category slug has no
// rules-i18n entry of its own.
describe('ParameterPicker category domain (row 19 "taken as")', () => {
  const TESTID = 'param-virtue.sufi-taken_as-0';

  it('offers exactly the values the parameter declares, labelled via category-<id>', () => {
    const body = pickerBody('virtue.sufi');
    // A `text` input was never this domain's control; the enumerated-style
    // select is.
    expect(hasInput(body, TESTID)).toBe(false);
    const select = selectFor(body, TESTID);
    expect(select).not.toBeNull();
    expect(optionTexts(select!)).toEqual(['Taken as', 'Social Status', 'Supernatural']);
  });

  it('labels each option through category-<id>, never as its raw slug', () => {
    const select = selectFor(pickerBody('virtue.sufi'), TESTID);
    expect(select!).toContain('value="social_status"');
    expect(optionTexts(select!).join(' ')).not.toContain('social_status');
    expect(optionTexts(select!).join(' ')).not.toContain('supernatural');
  });

  it('names the control via param-label-taken_as, in the active language', () => {
    store.lang = 'de';
    const select = selectFor(pickerBody('virtue.sufi'), TESTID);
    expect(ariaLabel(select!)).toBe('Gewählt als');
    expect(optionTexts(select!)).toEqual(['Gewählt als', 'Sozialer Status', 'Übernatürlich']);
  });

  it('greys out a reading another selection of the same item already holds', () => {
    store.entity.selections = [
      { ref: 'virtue.sufi', params: { taken_as: 'social_status' } },
      { ref: 'virtue.sufi' },
    ];
    const select = selectFor(pickerBody('virtue.sufi', 1), 'param-virtue.sufi-taken_as-1');
    expect(optionByText(select!, 'Social Status')).toContain('disabled');
    expect(optionByText(select!, 'Supernatural')).not.toContain('disabled');
  });
});

// manual-testing-findings-2026-09-03 #5: the ability domain used to offer ONLY the
// abilities already on the sheet, while abilities are bought on a later step — so
// Puissant Ability could not be completed where it is taken and the wizard deadlocked.
// Puissant Ability is "choose one Ability" with no requirement that a score exists
// (ArMDE:4814-4816), exactly like the `art`
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
      { ability: 'ability.area_lore', score: 2, parameter: { text: 'Brandenburg' } },
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
      { ability: 'ability.area_lore', score: 2, parameter: { text: 'Brandenburg' } },
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
