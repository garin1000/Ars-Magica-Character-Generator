import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset, ValidationIssue } from '../types';

// N3b (D84.2 follow-up, Norbert 2026-10-04, option b): the engine's ordinary
// validation pass reports `saga_year_before_birth_year` whenever the stored birth
// year is after the saga year, so a loaded save holding such a pair says so. That
// makes validation the ONE source of the warning in the panel: the age derivation's
// own copy of it is no longer shown beside it, or the same finding would appear
// twice.
//
// `ssr` project: the panel is server-rendered from the store after the mocked IPC
// answers have landed; nothing needs a live component.
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
  readSettings: vi.fn(),
  writeSettings: vi.fn(),
  deriveAge: vi.fn().mockResolvedValue({ age: 1, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(0),
}));

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import ValidationPanel from './ValidationPanel.svelte';

const CODE = 'saga_year_before_birth_year';

/** The engine's advisory for a birth year after the saga year (validation/saga.rs). */
function notBornYet(sagaYear: number, birthYear: number): ValidationIssue {
  return {
    severity: 'warning',
    code: CODE,
    phase: 'concept',
    args: { saga_year: String(sagaYear), birth_year: String(birthYear) },
  };
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

/** Open `entity` as a saved document, through the real load path. */
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

/** How many rows of the saga advisory the panel renders, for one step or all. */
function sagaRows(phase?: 'concept'): number {
  const body = render(ValidationPanel, { props: phase ? { phase } : {} }).body;
  return [...body.matchAll(new RegExp(`data-code="${CODE}"`, 'g'))].length;
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
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
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
  vi.mocked(ipc.validateEntity).mockReset().mockResolvedValue({ issues: [] });
  vi.mocked(ipc.deriveAge).mockReset().mockResolvedValue({ age: 1, issues: [] });
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
  store.result = null;
});

describe('the saga-year advisory has one source: validation (N3b)', () => {
  it('shows a loaded save whose birth year is after the saga year exactly once', async () => {
    vi.mocked(ipc.validateEntity).mockResolvedValue({ issues: [notBornYet(1220, 1250)] });

    await openDocument(character(1250, 0));

    expect(sagaRows()).toBe(1);
    expect(sagaRows('concept')).toBe(1);
  });

  it('does not show the age derivation’s copy of the advisory beside validation’s', async () => {
    // The derivation still returns its advisory in its payload (the `derive_age`
    // command is unchanged). Whatever it says, the panel shows what validation
    // says about the stored pair — here, nothing.
    await openDocument(character(1190, 30));
    vi.mocked(ipc.deriveAge).mockResolvedValue({ age: 30, issues: [notBornYet(1220, 1250)] });
    vi.mocked(ipc.validateEntity).mockResolvedValue({ issues: [] });

    store.setBirthYear(1190);
    await vi.runAllTimersAsync();

    expect(sagaRows()).toBe(0);
  });

  it('shows the advisory once even when the derivation reports a different pair', async () => {
    await openDocument(character(1250, 0));
    vi.mocked(ipc.deriveAge).mockResolvedValue({ age: 0, issues: [notBornYet(1220, 1251)] });
    vi.mocked(ipc.validateEntity).mockResolvedValue({ issues: [notBornYet(1220, 1250)] });

    store.setBirthYear(1190);
    await vi.runAllTimersAsync();

    expect(sagaRows()).toBe(1);
  });
});

// The message used to add "the age reads 0 until one of the two is changed". That was
// true only of the derivation's clamped answer: a loaded save, or a saga year moved
// back under the birth year (which rewrites neither value, D3.3), keeps whatever age
// it stores. Now that validation raises the warning for exactly those cases, the
// message states only the fact.
describe('the saga-year advisory states only the fact (N3b)', () => {
  it.each([
    [
      'en',
      'The saga year (1220) is before the birth year (1250), so the character is not born yet.',
    ],
    [
      'de',
      'Das Jahr der Saga (1220) liegt vor dem Geburtsjahr (1250), der Charakter ist also noch nicht geboren.',
    ],
  ] as const)('%s', (lang, expected) => {
    store.lang = lang;
    expect(store.t(`issue-${CODE}`, { saga_year: '1220', birth_year: '1250' })).toBe(expected);
  });
});
