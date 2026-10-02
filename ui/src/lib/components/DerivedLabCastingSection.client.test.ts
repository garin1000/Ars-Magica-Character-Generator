import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CastingTotal, Entity, LabTotal, LocalizedRuleset } from '../types';

// A `client` test, deliberately. The addend breakdown only ever reaches the DOM
// through the `tooltip` action (`actions.ts::tooltip`), whose `show()` runs on
// `focusin` off a `document.createElement` call — `svelte/server`'s `render()`
// never executes an action, so the joined text is invisible to a string-based
// `ssr` test. Same reasoning (and harness) as DerivedTotalsPanel.client.test.ts.
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

import { store } from '../state.svelte';
import { formatSigned } from '../derive';
import DerivedLabCastingSection from './DerivedLabCastingSection.svelte';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      aura_modifier_min: -50,
      aura_modifier_max: 10,
    },
    i18n: {
      'art.creo': { name: 'Creo' },
      'art.ignem': { name: 'Ignem' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: 11,
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
  };
}

function labTotal(): LabTotal {
  return {
    technique: 'art.creo',
    form: 'art.ignem',
    addends: [],
    total: 30,
    within_focus: null,
    deficient: false,
    enchanting: 30,
    unusable: false,
  };
}

// A Method Caster (`casting_total_mod +3`, `formulaic_ritual` scope) with the
// ritual Abilities, so all three addend lists the engine surfaces carry a term
// and each list's absence is separately visible.
function castingTotal(): CastingTotal {
  return {
    technique: 'art.creo',
    form: 'art.ignem',
    addends: [
      { label: 'technique', value: 10 },
      { label: 'form', value: 8 },
      { label: 'stamina', value: 2 },
      { label: 'encumbrance', value: -1 },
      { label: 'aura', value: 3 },
    ],
    ritual_addends: [
      { label: 'artes_liberales', value: 2 },
      { label: 'philosophiae', value: 1 },
    ],
    casting_mod_addends: [
      { label: 'casting_mod_formulaic', value: 3 },
      { label: 'casting_mod_ritual', value: 3 },
      { label: 'casting_mod_spontaneous', value: 0 },
    ],
    formulaic: 25,
    ritual: 28,
    spontaneous_fatiguing: 12,
    spontaneous_non_fatiguing: 6,
    within_focus: null,
    non_standard: {
      voice_penalty: -5,
      gesture_penalty: -2,
      silent: 20,
      still: 23,
      silent_and_still: 18,
      deft_form: false,
    },
    deficient: false,
    unusable: false,
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

/** Mount the section and open the casting row header's breakdown tooltip. */
function castingRowTooltipText(cell: CastingTotal): string {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(DerivedLabCastingSection, {
    target,
    props: { d: { lab_totals: [labTotal()], casting_totals: [cell] } as never },
  });
  flushSync();

  const rowHeader = target.querySelector(
    '[data-testid="derived-casting-total"] th[scope="row"]',
  ) as HTMLElement;
  expect(rowHeader, 'the casting row header should render').not.toBeNull();
  rowHeader.dispatchEvent(new Event('focusin', { bubbles: true }));
  flushSync();

  const tip = document.querySelector('[data-testid="tooltip-text"]');
  expect(tip, 'focusing the casting row header should open its breakdown tooltip').not.toBeNull();
  return tip!.textContent ?? '';
}

// Gerda round-4 finding 3 (the UI half, handed over from the engine slice), with
// Sabine's presentation call: the row `<th>` heads four cells holding four
// different totals, so its tooltip was never going to equal any one of them.
// It must therefore read as "here is every term that feeds this row" — which
// means naming every addend list the engine surfaces for the cell, not just the
// shared one. `ritual_addends` and `casting_mod_addends` were computed,
// serialized and typed by nobody's renderer; the Lab tooltip one element above
// has always been complete because `lab.rs::lab_totals` folds its `lab_mod` into
// the single `addends` list.
describe('DerivedLabCastingSection — the casting breakdown tooltip', () => {
  it('names every term the engine surfaces for the cell, not just the shared addends', () => {
    const cell = castingTotal();
    const text = castingRowTooltipText(cell);

    for (const addend of [...cell.addends, ...cell.ritual_addends, ...cell.casting_mod_addends]) {
      const label = store.t(`derived-addend-${addend.label}`);
      expect(text, `the breakdown should name the "${addend.label}" term`).toContain(
        `${label} ${formatSigned(addend.value)}`,
      );
    }
  });

  it('renders no raw addend slug — every term comes through Fluent', () => {
    const cell = castingTotal();
    const text = castingRowTooltipText(cell);

    for (const addend of [...cell.addends, ...cell.ritual_addends, ...cell.casting_mod_addends]) {
      const key = `derived-addend-${addend.label}`;
      expect(store.t(key), `${key} must resolve to a label, not echo its own key`).not.toBe(key);
      expect(text, `the breakdown must not render the raw "${key}" slug`).not.toContain(key);
    }
  });
});
