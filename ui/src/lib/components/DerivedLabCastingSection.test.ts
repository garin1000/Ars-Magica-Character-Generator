import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { CastingTotal, Entity, LabTotal, LocalizedRuleset } from '../types';

// An `ssr` test, deliberately: everything asserted here is static rendered
// markup — which element a number sits in, and which header it is scoped to.
// Nothing needs a live instance, an `$effect` or a DOM event, so the fast
// project is the right one. (The Technique/Form `<select>`s bind to the store,
// but no assertion below depends on a binding round-trip.)
//
// The section reads the shared store singleton (ruleset i18n + the Fluent
// bundle), which schedules a debounced revalidate over the Tauri IPC bridge;
// mock the bridge so nothing reaches a backend. Mirrors
// DerivedCombatSection.test.ts.
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
  };
}

function castingTotal(overrides: Partial<CastingTotal> = {}): CastingTotal {
  return {
    technique: 'art.creo',
    form: 'art.ignem',
    addends: [],
    ritual_addends: [],
    casting_mod_addends: [],
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
      ...(overrides.non_standard ?? {}),
    },
    deficient: false,
    ...overrides,
  };
}

/** Render the section to an HTML string (node env, no DOM). */
function html(casting: CastingTotal = castingTotal()): string {
  return render(DerivedLabCastingSection, {
    props: { d: { lab_totals: [labTotal()], casting_totals: [casting] } as never },
  }).body;
}

/** The `<table class="derived-table casting">` element's markup, that table only. */
function castingTable(body: string): string {
  const start = body.indexOf('<table');
  const end = body.indexOf('</table>');
  expect(start, 'the casting table should render').toBeGreaterThan(-1);
  expect(end).toBeGreaterThan(start);
  return body.slice(start, end);
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// Sabine 9 (round-1 audit), the Lab/Casting half. This table has a header ROW
// and a header COLUMN and carried no `scope` at all, so the cell→header
// association is not inferable and a screen reader reads bare numbers.
//
// ORDER MATTERS, and that is the whole point of this block. The `.non-standard`
// row was filed INTO the table while not lying on its axes: under the
// "Formulaic" column header it rendered "No voice: 20", under "Ritual" it
// rendered "No gestures: 23", and a `colspan="2"` cell spanned the two
// spontaneous columns. Adding `scope` first would have made that WORSE — the
// screen reader would then confidently announce "Formulaic — No voice: 20", a
// relationship the rules do not have. So the row moves out first, into a
// description list whose terms actually name what the numbers are, and only
// then do the remaining headers get scoped.
describe('DerivedLabCastingSection — table semantics (Sabine 9)', () => {
  it('scopes every column header to its column and every row header to its row', () => {
    const table = castingTable(html());
    // `<th`, not `<thead`: the lookahead keeps the section's own `<thead>` out.
    const headers = table.match(/<th(?=[\s>])[^>]*>/g) ?? [];
    // Four labelled column headers plus the empty corner cell (it heads the
    // row-label column), then one row header for the Casting line.
    expect(headers.length).toBe(6);
    expect(headers.filter((h) => h.includes('scope="col"')).length).toBe(5);
    expect(headers.filter((h) => h.includes('scope="row"')).length).toBe(1);
    expect(headers.every((h) => h.includes('scope='))).toBe(true);
  });

  it('scopes the within-focus row header too when that row renders', () => {
    const table = castingTable(
      html(
        castingTotal({
          within_focus: {
            focus_art: 8,
            formulaic: 33,
            ritual: 36,
            spontaneous_fatiguing: 16,
            spontaneous_non_fatiguing: 8,
          },
        }),
      ),
    );
    const headers = table.match(/<th(?=[\s>])[^>]*>/g) ?? [];
    expect(headers.length).toBe(7);
    expect(headers.filter((h) => h.includes('scope="row"')).length).toBe(2);
    expect(headers.every((h) => h.includes('scope='))).toBe(true);
  });
});

describe('DerivedLabCastingSection — the non-standard casting figures (Sabine 9)', () => {
  it('keeps the non-standard figures out of the four-column casting table', () => {
    // The defect itself: three numbers that belong to no column sitting under
    // column headers that name four cast types.
    const table = castingTable(html());
    for (const key of ['derived-cast-silent', 'derived-cast-still', 'derived-cast-silent-still']) {
      expect(table).not.toContain(store.t(key));
    }
    expect(table).not.toContain('colspan');
    // Every body row is now genuinely four cells wide behind its row header.
    const bodyRows = table.slice(table.indexOf('<tbody')).match(/<tr(?=[\s>])/g) ?? [];
    expect(bodyRows.length).toBe(1);
  });

  it('names each non-standard figure with a term of its own', () => {
    const body = html();
    const list = body.slice(body.indexOf('data-testid="derived-cast-non-standard"'));
    // A description list: each number gets a <dt> that says what it is, rather
    // than borrowing an unrelated column header.
    for (const [key, value] of [
      ['derived-cast-silent', '20'],
      ['derived-cast-still', '23'],
      ['derived-cast-silent-still', '18'],
    ] as const) {
      const label = store.t(key);
      expect(label).not.toBe(key);
      expect(list).toContain(`<dt>${label}</dt>`);
      expect(list).toContain(`<dd>${value}</dd>`);
    }
  });

  it('still flags Deft Form on the group heading', () => {
    const body = html(castingTotal({ non_standard: { deft_form: true } as never }));
    expect(body).toContain(store.t('derived-deft-form'));
  });
});
