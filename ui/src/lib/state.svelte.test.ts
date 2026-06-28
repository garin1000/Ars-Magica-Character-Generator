import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Ability, Entity, LocalizedRuleset, PointItem } from './types';

// The AppStore methods under test are synchronous; they only *schedule* a
// debounced revalidate via setTimeout, which calls into the Tauri IPC bridge.
// Mock that bridge so the timer (if it ever fires) is inert, and use fake
// timers so it never fires mid-assertion. We assert on entity state directly.
vi.mock('./ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi
    .fn()
    .mockResolvedValue({ ability_bonuses: [], characteristic_caps: {}, characteristic_floors: {} }),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
}));

// Import the singleton after the mock is registered.
import { store } from './state.svelte';

// --- Fixtures ---------------------------------------------------------------

function item(overrides: Partial<PointItem> & Pick<PointItem, 'id'>): PointItem {
  return {
    kind: 'virtue',
    magnitude: 'minor',
    category: 'general',
    entity_kinds: ['character'],
    ...overrides,
  };
}

function ability(id: string, parameter?: string): Ability {
  return { id, category: 'general', ...(parameter ? { parameter } : {}) };
}

/** Build a localized ruleset from items + abilities and install it on the store. */
function installRuleset(items: PointItem[], abilities: Ability[] = []): LocalizedRuleset {
  const point_items: Record<string, PointItem> = {};
  for (const it of items) point_items[it.id] = it;
  const abilityMap: Record<string, Ability> = {};
  for (const a of abilities) abilityMap[a.id] = a;
  const ruleset: LocalizedRuleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items,
      type_profiles: {},
      abilities: abilityMap,
      // Engine-derived taxonomy the real backend ships on every Ruleset payload.
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
    },
    i18n: {},
  };
  store.ruleset = ruleset;
  return ruleset;
}

/** Reset the shared singleton's entity to a clean character before each test. */
function resetEntity(): void {
  store.entity = {
    schema_version: 2,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  resetEntity();
  installRuleset([]);
});

afterEach(() => {
  // Drop any pending debounced revalidate without running it, then restore.
  vi.clearAllTimers();
  vi.useRealTimers();
});

// --- setType() --------------------------------------------------------------

describe('setType', () => {
  it('switches the entity type id in place, keeping selections', () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.addSelection('virtue.plain');
    store.setType('magus');
    expect(store.entity.type_id).toBe('magus');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  it('is a no-op when the type is unchanged', () => {
    const before = store.entity;
    store.setType('companion');
    expect(store.entity).toBe(before);
    expect(store.entity.type_id).toBe('companion');
  });
});

// --- addSelection() ---------------------------------------------------------

describe('addSelection', () => {
  it('dedups a plain (non-repeatable) item: adding twice yields one row', () => {
    installRuleset([item({ id: 'virtue.plain' })]);
    store.addSelection('virtue.plain');
    store.addSelection('virtue.plain');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.plain' }]);
  });

  it('allows a parameterized item to appear several times', () => {
    installRuleset([
      item({
        id: 'virtue.great',
        parameters: [{ key: 'characteristic', type: 'ref', domain: 'characteristic' }],
      }),
    ]);
    store.addSelection('virtue.great');
    store.addSelection('virtue.great');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.great' }, { ref: 'virtue.great' }]);
  });

  it('allows an item with max_per_target > 1 to appear several times', () => {
    installRuleset([item({ id: 'virtue.stacks', max_per_target: 3 })]);
    store.addSelection('virtue.stacks');
    store.addSelection('virtue.stacks');
    expect(store.entity.selections).toHaveLength(2);
  });

  it('treats an unknown ref as non-repeatable (max one row)', () => {
    installRuleset([]);
    store.addSelection('virtue.unknown');
    store.addSelection('virtue.unknown');
    expect(store.entity.selections).toEqual([{ ref: 'virtue.unknown' }]);
  });
});

// --- removeSelectionAt() ----------------------------------------------------

describe('removeSelectionAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    installRuleset([
      item({ id: 'v.a', max_per_target: 5 }),
      item({ id: 'v.b', max_per_target: 5 }),
      item({ id: 'v.c', max_per_target: 5 }),
    ]);
    store.addSelection('v.a');
    store.addSelection('v.b');
    store.addSelection('v.c');
    store.removeSelectionAt(1);
    expect(store.entity.selections.map((s) => s.ref)).toEqual(['v.a', 'v.c']);
  });

  it('is a no-op for an out-of-range index', () => {
    installRuleset([item({ id: 'v.a' })]);
    store.addSelection('v.a');
    store.removeSelectionAt(5);
    expect(store.entity.selections.map((s) => s.ref)).toEqual(['v.a']);
  });
});

// --- setParamAt() -----------------------------------------------------------

describe('setParamAt', () => {
  it('sets a param on the targeted row only', () => {
    installRuleset([item({ id: 'v.great', max_per_target: 5 })]);
    store.addSelection('v.great');
    store.addSelection('v.great');
    store.setParamAt(0, 'characteristic', 'characteristic.per');
    expect(store.entity.selections[0]).toEqual({
      ref: 'v.great',
      params: { characteristic: 'characteristic.per' },
    });
    expect(store.entity.selections[1]).toEqual({ ref: 'v.great' });
  });

  it('merges into existing params rather than replacing them', () => {
    installRuleset([item({ id: 'v.x' })]);
    store.addSelection('v.x');
    store.setParamAt(0, 'a', '1');
    store.setParamAt(0, 'b', '2');
    expect(store.entity.selections[0].params).toEqual({ a: '1', b: '2' });
  });
});

// --- setAbilityBonusTarget() ------------------------------------------------

describe('setAbilityBonusTarget', () => {
  beforeEach(() => {
    installRuleset(
      [item({ id: 'virtue.puissant', max_per_target: 5 })],
      [ability('ability.awareness'), ability('ability.area_lore', 'area')],
    );
    store.addSelection('virtue.puissant');
  });

  it('stores the instance key for a parameterized ability', () => {
    store.setAbilityBonusTarget(0, 'ability.area_lore', 'Rhine');
    expect(store.entity.selections[0].params).toEqual({
      ability: 'ability.area_lore',
      area: 'Rhine',
    });
  });

  it('stores just the ability for a plain ability (no instance key)', () => {
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections[0].params).toEqual({ ability: 'ability.awareness' });
  });

  it('drops the stale instance key when switching to a plain ability', () => {
    // First point it at a parameterized instance...
    store.setAbilityBonusTarget(0, 'ability.area_lore', 'Rhine');
    expect(store.entity.selections[0].params).toHaveProperty('area', 'Rhine');
    // ...then switch the target to a plain ability: the params object is
    // rebuilt, so the stale `area` key is gone, not merged.
    store.setAbilityBonusTarget(0, 'ability.awareness');
    expect(store.entity.selections[0].params).toEqual({ ability: 'ability.awareness' });
    expect(store.entity.selections[0].params).not.toHaveProperty('area');
  });

  it('omits the instance key when a parameterized ability has no parameter value', () => {
    store.setAbilityBonusTarget(0, 'ability.area_lore', null);
    expect(store.entity.selections[0].params).toEqual({ ability: 'ability.area_lore' });
  });
});

// --- setCharacteristic() ----------------------------------------------------

describe('setCharacteristic', () => {
  it('stores a non-zero score', () => {
    store.setCharacteristic('int', 3);
    expect(store.entity.characteristics).toEqual({ int: 3 });
  });

  it('stores a negative score', () => {
    store.setCharacteristic('per', -2);
    expect(store.entity.characteristics).toEqual({ per: -2 });
  });

  it('removes the entry when the score is set to 0', () => {
    store.setCharacteristic('int', 3);
    store.setCharacteristic('per', 1);
    store.setCharacteristic('int', 0);
    expect(store.entity.characteristics).toEqual({ per: 1 });
    expect(store.entity.characteristics).not.toHaveProperty('int');
  });
});

// --- setXpPool() ------------------------------------------------------------

describe('setXpPool', () => {
  it('stores a positive integer', () => {
    store.setXpPool(45);
    expect(store.entity.xp_pool).toBe(45);
  });

  it('floors a fractional value', () => {
    store.setXpPool(45.9);
    expect(store.entity.xp_pool).toBe(45);
  });

  it('clamps non-positive and non-finite values to 0', () => {
    store.setXpPool(-5);
    expect(store.entity.xp_pool).toBe(0);
    store.setXpPool(0);
    expect(store.entity.xp_pool).toBe(0);
    store.setXpPool(Number.NaN);
    expect(store.entity.xp_pool).toBe(0);
  });
});

// --- addAbility() -----------------------------------------------------------

describe('addAbility', () => {
  it('adds a plain ability at score 0 and dedups a second add', () => {
    installRuleset([], [ability('ability.awareness')]);
    store.addAbility('ability.awareness');
    store.addAbility('ability.awareness');
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.awareness', score: 0 }]);
  });

  it('allows a parameterized ability to be added several times', () => {
    installRuleset([], [ability('ability.area_lore', 'area')]);
    store.addAbility('ability.area_lore');
    store.addAbility('ability.area_lore');
    expect(store.entity.ability_scores).toEqual([
      { ability: 'ability.area_lore', score: 0 },
      { ability: 'ability.area_lore', score: 0 },
    ]);
  });
});

// --- removeAbilityAt() ------------------------------------------------------

describe('removeAbilityAt', () => {
  it('removes only the row at the given index, keeping order', () => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.b', score: 2 },
      { ability: 'ability.c', score: 3 },
    ];
    store.removeAbilityAt(1);
    expect(store.entity.ability_scores).toEqual([
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.c', score: 3 },
    ]);
  });

  it('is a no-op for an out-of-range index', () => {
    store.entity.ability_scores = [{ ability: 'ability.a', score: 1 }];
    store.removeAbilityAt(5);
    expect(store.entity.ability_scores).toEqual([{ ability: 'ability.a', score: 1 }]);
  });
});

// --- adjustAbilityAt() ------------------------------------------------------

describe('adjustAbilityAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 2 },
      { ability: 'ability.b', score: 5 },
    ];
  });

  it('raises the score by a positive delta within range', () => {
    store.adjustAbilityAt(0, 2, 10);
    expect(store.entity.ability_scores![0].score).toBe(4);
    // other rows untouched
    expect(store.entity.ability_scores![1].score).toBe(5);
  });

  it('clamps to 0 when the delta would push below 0', () => {
    store.adjustAbilityAt(0, -5, 10);
    expect(store.entity.ability_scores![0].score).toBe(0);
  });

  it('clamps to max when the delta would push above max', () => {
    store.adjustAbilityAt(1, 99, 7);
    expect(store.entity.ability_scores![1].score).toBe(7);
  });

  it('is a no-op for an out-of-range index', () => {
    store.adjustAbilityAt(9, 3, 10);
    expect(store.entity.ability_scores!.map((a) => a.score)).toEqual([2, 5]);
  });
});

// --- setCharacteristicDescription() -----------------------------------------

describe('setCharacteristicDescription', () => {
  it('stores the (untrimmed) text when it is non-blank', () => {
    // The trim() is only an emptiness guard; the raw text is stored verbatim.
    store.setCharacteristicDescription('int', '  Sharp wit  ');
    expect(store.entity.characteristic_descriptions).toEqual({ int: '  Sharp wit  ' });
  });

  it('removes the entry when the text is blank or whitespace-only', () => {
    store.setCharacteristicDescription('int', 'Sharp wit');
    store.setCharacteristicDescription('per', 'Keen eyes');
    store.setCharacteristicDescription('int', '   ');
    expect(store.entity.characteristic_descriptions).toEqual({ per: 'Keen eyes' });
    expect(store.entity.characteristic_descriptions).not.toHaveProperty('int');
  });
});

// --- setAbilitySpecialtyAt() ------------------------------------------------

describe('setAbilitySpecialtyAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.a', score: 1 },
      { ability: 'ability.b', score: 2 },
    ];
  });

  it('stores a trimmed specialty on the targeted row only', () => {
    store.setAbilitySpecialtyAt(0, '  Birds  ');
    expect(store.entity.ability_scores![0]).toEqual({
      ability: 'ability.a',
      score: 1,
      specialty: 'Birds',
    });
    expect(store.entity.ability_scores![1]).toEqual({ ability: 'ability.b', score: 2 });
  });

  it('sets specialty to undefined for blank/whitespace input', () => {
    store.setAbilitySpecialtyAt(0, 'Birds');
    store.setAbilitySpecialtyAt(0, '   ');
    expect(store.entity.ability_scores![0].specialty).toBeUndefined();
  });
});

// --- setAbilityParameterAt() ------------------------------------------------

describe('setAbilityParameterAt', () => {
  beforeEach(() => {
    store.entity.ability_scores = [
      { ability: 'ability.area_lore', score: 1 },
      { ability: 'ability.area_lore', score: 2 },
    ];
  });

  it('stores a trimmed parameter on the targeted row only', () => {
    store.setAbilityParameterAt(1, '  Rhine  ');
    expect(store.entity.ability_scores![1]).toEqual({
      ability: 'ability.area_lore',
      score: 2,
      parameter: 'Rhine',
    });
    expect(store.entity.ability_scores![0]).toEqual({ ability: 'ability.area_lore', score: 1 });
  });

  it('sets parameter to undefined for blank input', () => {
    store.setAbilityParameterAt(0, 'Rhine');
    store.setAbilityParameterAt(0, '   ');
    expect(store.entity.ability_scores![0].parameter).toBeUndefined();
  });
});
