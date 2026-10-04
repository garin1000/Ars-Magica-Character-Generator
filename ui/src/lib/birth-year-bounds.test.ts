import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from './types';

// N3 (try-out 2026-10-04, Norbert): "I can just enter digits in front of 720, becoming
// 1190720. Age is then 0." The birth year had a lower bound (saga year - max_age,
// slice A1) and none above but the i32 width, so any year after the saga year was
// stored and the engine derived age 0 from it. Decided (D84.2): the latest birth year
// is saga year - 1 — age at least 1, the same floor `setAge` keeps.
//
// `ssr` project: store logic only, nothing mounted.
vi.mock('./ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
  confirmDiscard: vi.fn().mockResolvedValue(true),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  readSettings: vi.fn(),
  writeSettings: vi.fn(),
  deriveAge: vi.fn().mockResolvedValue({ age: 1, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(0),
  unlinkAbilityParameters: vi.fn().mockImplementation((entity: Entity) => Promise.resolve(entity)),
}));

import * as ipc from './ipc';
import { I32_MAX, I32_MIN } from './clamp';
import { SCHEMA_VERSION, store } from './state.svelte';

/** A ruleset with an aging block stating `maxAge`, or none at all for `null`. */
function installRuleset(maxAge: number | null): void {
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
      ...(maxAge == null
        ? {}
        : {
            aging: {
              start_age: 35,
              age_divisor: 10,
              apparent_age_increase_min: 3,
              living_conditions: [],
              outcomes: [],
              max_age: maxAge,
            },
          }),
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
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
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  installRuleset(500);
  resetEntity();
  vi.mocked(ipc.deriveAge).mockClear();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('the latest birth year is saga year - 1 (N3)', () => {
  it('states the latest birth year the input accepts', () => {
    expect(store.latestBirthYear).toBe(1219);
    store.setSagaYear(1197);
    expect(store.latestBirthYear).toBe(1196);
  });

  it('clamps a year after the saga year down to saga year - 1', () => {
    store.setBirthYear(1190720);
    expect(store.entity.birth_year).toBe(1219);
    store.setBirthYear(3e9);
    expect(store.entity.birth_year).toBe(1219);
  });

  it('clamps the saga year itself, since a character born then is of age 0', () => {
    store.setBirthYear(1220);
    expect(store.entity.birth_year).toBe(1219);
  });

  it('leaves the latest year itself, and an ordinary one, alone', () => {
    store.setBirthYear(1219);
    expect(store.entity.birth_year).toBe(1219);
    store.setBirthYear(1190);
    expect(store.entity.birth_year).toBe(1190);
  });

  it('derives the age from the clamped year, never age 0 from a typed future year', async () => {
    store.setBirthYear(1250);
    await vi.runAllTimersAsync();
    expect(vi.mocked(ipc.deriveAge)).toHaveBeenCalledWith(1220, 1219, 500);
    expect(vi.mocked(ipc.deriveAge)).not.toHaveBeenCalledWith(1220, 1250, 500);
  });

  it('keeps the lower bound at saga year - max_age (slice A1)', () => {
    store.setBirthYear(100);
    expect(store.entity.birth_year).toBe(720);
  });

  it('bounds above even when the ruleset states no maximum age', () => {
    installRuleset(null);
    store.setBirthYear(3e9);
    expect(store.entity.birth_year).toBe(1219);
    store.setBirthYear(-3e9);
    expect(store.entity.birth_year).toBe(I32_MIN);
  });

  it('stays inside the i32 width at the bottom edge of the saga year', () => {
    installRuleset(null);
    store.setSagaYear(I32_MIN);
    store.setBirthYear(0);
    expect(store.entity.birth_year).toBe(I32_MIN);
    store.setSagaYear(I32_MAX);
    store.setBirthYear(3e9);
    expect(store.entity.birth_year).toBe(I32_MAX - 1);
  });

  it('still clears on an emptied field rather than clamping', () => {
    store.setBirthYear(1190);
    store.setBirthYear(null);
    expect(store.entity.birth_year).toBeNull();
  });
});
