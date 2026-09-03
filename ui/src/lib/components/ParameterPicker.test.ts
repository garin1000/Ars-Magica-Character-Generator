import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Art, Entity, LocalizedRuleset, ParameterDef, PointItem, Selection } from '../types';

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
      abilities: {},
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
function pickerBody(ref: string, index = 0): string {
  const selection: Selection = { ref };
  return render(ParameterPicker, {
    props: { selection, index, params: ITEMS[ref].parameters! },
  }).body;
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
