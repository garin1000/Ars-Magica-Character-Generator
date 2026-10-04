import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset, ValidationIssue } from '../types';

// N3 (try-out 2026-10-04, Norbert): "I can just enter digits in front of 720, becoming
// 1190720. Age is then 0." The birth year is bounded above at saga year - 1 (D84.2):
// the field carries that `max`, a keystroke past it never reaches the store, and the
// commit clamps and shows the stored year. A save that already holds a later year (or
// a saga year moved back under it) keeps it, with the engine's age-0 advisory, until
// the user actually edits the field.
//
// A `client` test: the `input`/`change`/`focus`/`blur` listeners on a live element,
// the text the live field shows, and `store.dirty` read after them.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  confirmDiscard: vi.fn().mockResolvedValue(true),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  readSettings: vi.fn(),
  writeSettings: vi.fn(),
  deriveAge: vi.fn().mockResolvedValue({ age: null, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(null),
}));

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import IdentityFields from './IdentityFields.svelte';

const BIRTH_YEAR = '[data-testid="identity-birth-year"]';

/** The engine's advisory for a birth year after the saga year (validation/mod.rs). */
const NOT_BORN_YET: ValidationIssue = {
  severity: 'warning',
  code: 'saga_year_before_birth_year',
  phase: 'concept',
  args: { saga_year: '1220', birth_year: '1250' },
} as ValidationIssue;

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      aging: {
        start_age: 35,
        age_divisor: 10,
        apparent_age_increase_min: 3,
        living_conditions: [],
        outcomes: [],
        max_age: 500,
      },
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function character(birthYear: number | null, age: number | null): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'grog',
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
    age,
    birth_year: birthYear,
  };
}

/** Open `entity` as a saved document, so `dirty` starts false against it. */
async function openDocument(entity: Entity): Promise<void> {
  vi.mocked(ipc.loadEntity).mockResolvedValue({
    path: '/saga/grog.armc',
    entity,
    migrated_aging_characteristics: [],
  });
  const opening = store.open();
  if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
  await opening;
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  vi.mocked(ipc.deriveAge).mockClear();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  store.sagaIssues = [];
});

function mountField(): HTMLInputElement {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(IdentityFields, { target });
  flushSync();
  const input = target.querySelector<HTMLInputElement>(BIRTH_YEAR);
  expect(input).toBeTruthy();
  return input!;
}

/** Put `values` into the field one keystroke at a time, each the field's whole text. */
function keystrokes(input: HTMLInputElement, values: string[]): void {
  for (const value of values) {
    input.value = value;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
  }
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

describe('IdentityFields birth year is bounded above at saga year - 1 (N3)', () => {
  it('states saga year - 1 as the field maximum', async () => {
    await openDocument(character(1190, 30));
    const input = mountField();
    expect(input.max).toBe('1219');
  });

  it('keeps digits typed in front of 720 out of the store, and clamps them on commit', async () => {
    await openDocument(character(720, 500));
    const input = mountField();

    // Norbert's keystrokes: 1, 1, 9, 0 typed in front of the 720 the field showed.
    keystrokes(input, ['1720', '11720', '119720', '1190720']);
    expect(store.entity.birth_year, 'no keystroke past the bound reaches the store').toBe(720);

    commit(input);
    expect(store.entity.birth_year).toBe(1219);
    expect(input.value).toBe('1219');
  });

  it('clamps a committed year after the saga year to saga year - 1', async () => {
    await openDocument(character(1190, 30));
    const input = mountField();

    keystrokes(input, ['1250']);
    commit(input);
    expect(store.entity.birth_year).toBe(1219);
    expect(input.value).toBe('1219');
  });

  // A save written before D84.2, or a saga year moved back under the birth year,
  // holds a year the setter would now clamp. Visiting the field without typing is
  // not an edit: nothing is rewritten, the document stays clean, and the engine's
  // age-0 advisory stays up to say what is wrong.
  it('leaves a loaded later birth year alone when the field is focused and left untouched', async () => {
    await openDocument(character(1250, 0));
    store.sagaIssues = [NOT_BORN_YET];
    const input = mountField();
    expect(input.value).toBe('1250');

    input.dispatchEvent(new FocusEvent('focus'));
    commit(input);
    input.dispatchEvent(new FocusEvent('blur'));
    await Promise.resolve();
    flushSync();

    expect(store.entity.birth_year).toBe(1250);
    expect(store.entity.age).toBe(0);
    expect(input.value).toBe('1250');
    expect(store.sagaIssues).toEqual([NOT_BORN_YET]);
    expect(vi.mocked(ipc.deriveAge)).not.toHaveBeenCalled();
    expect(store.dirty).toBe(false);
  });
});
