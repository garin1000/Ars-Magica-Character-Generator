import { describe, expect, it } from 'vitest';

import {
  abilityDisplayName,
  abilityLabel,
  abilityXpSpent,
  balance,
  characteristicPointsUsed,
  displayName,
  groupAbilitiesByCategory,
  groupByCategory,
  maxAbilityScore,
  paramValueUsage,
} from './derive';
import type { Ability, CharacteristicRules, Entity, LocalizedRuleset, PointItem } from './types';

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

/** A localized ruleset with the given items and (optionally) i18n + profiles. */
function makeRuleset(
  items: PointItem[],
  opts: {
    i18n?: LocalizedRuleset['i18n'];
    profiles?: LocalizedRuleset['ruleset']['type_profiles'];
  } = {},
): LocalizedRuleset {
  const point_items: Record<string, PointItem> = {};
  for (const it of items) point_items[it.id] = it;
  return {
    ruleset: {
      id: 'test',
      version: '1',
      point_items,
      type_profiles: opts.profiles ?? {},
    },
    i18n: opts.i18n ?? {},
  };
}

function entity(refs: string[], typeId = 'companion'): Entity {
  return {
    schema_version: 1,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: typeId,
    selections: refs.map((ref) => ({ ref })),
  };
}

// --- balance() --------------------------------------------------------------

describe('balance', () => {
  const profiles = {
    companion: {
      id: 'companion',
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      creation_phases: [],
    },
  };

  it('sums virtue, flaw and boon points by magnitude', () => {
    const ruleset = makeRuleset(
      [
        item({ id: 'v.minor', kind: 'virtue', magnitude: 'minor' }),
        item({ id: 'v.major', kind: 'virtue', magnitude: 'major' }),
        item({ id: 'f.major', kind: 'flaw', magnitude: 'major' }),
        item({ id: 'f.free', kind: 'flaw', magnitude: 'free' }),
        item({ id: 'b.boon', kind: 'boon', magnitude: 'minor' }),
      ],
      { profiles },
    );
    // virtue side: 1 (minor v) + 3 (major v) + 1 (minor boon) = 5
    // flaw side: 3 (major f) + 0 (free f) = 3
    const result = balance(ruleset, entity(['v.minor', 'v.major', 'f.major', 'f.free', 'b.boon']));
    expect(result.virtuePoints).toBe(5);
    expect(result.flawPoints).toBe(3);
    expect(result.virtueBudget).toBe(10);
    expect(result.flawBudget).toBe(10);
  });

  it('counts a hook on the flaw side (mirrors engine is_positive)', () => {
    const ruleset = makeRuleset([item({ id: 'h.minor', kind: 'hook', magnitude: 'minor' })], {
      profiles,
    });
    const result = balance(ruleset, entity(['h.minor']));
    expect(result.virtuePoints).toBe(0);
    expect(result.flawPoints).toBe(1);
  });

  it('skips selections whose item ref is unknown', () => {
    const ruleset = makeRuleset([item({ id: 'v.minor', kind: 'virtue', magnitude: 'minor' })], {
      profiles,
    });
    const result = balance(ruleset, entity(['v.minor', 'does.not.exist']));
    expect(result.virtuePoints).toBe(1);
    expect(result.flawPoints).toBe(0);
  });

  it('reports a zero budget when the type profile is missing', () => {
    const ruleset = makeRuleset([item({ id: 'v.minor', magnitude: 'minor' })]);
    const result = balance(ruleset, entity(['v.minor'], 'unknown_type'));
    expect(result.virtueBudget).toBe(0);
    expect(result.flawBudget).toBe(0);
    // points are still summed even without a profile
    expect(result.virtuePoints).toBe(1);
  });
});

// --- displayName() ----------------------------------------------------------

describe('displayName', () => {
  it('returns the i18n name when no params are needed', () => {
    const ruleset = makeRuleset([], { i18n: { 'virtue.keen_vision': { name: 'Keen Vision' } } });
    expect(displayName(ruleset, 'virtue.keen_vision')).toBe('Keen Vision');
  });

  it('substitutes a present param placeholder', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(displayName(ruleset, 'virtue.puissant', { ability: 'Awareness' })).toBe(
      'Puissant Awareness',
    );
  });

  it('leaves the placeholder intact when the param is missing', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(displayName(ruleset, 'virtue.puissant', {})).toBe('Puissant {ability}');
    expect(displayName(ruleset, 'virtue.puissant')).toBe('Puissant {ability}');
  });

  it('uses the placeholder-label resolver for a missing param', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(displayName(ruleset, 'virtue.puissant', undefined, (key) => `(${key})`)).toBe(
      'Puissant (ability)',
    );
  });

  it('prefers a present param over the placeholder-label resolver', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.puissant': { name: 'Puissant {ability}' } },
    });
    expect(
      displayName(ruleset, 'virtue.puissant', { ability: 'Awareness' }, (key) => `(${key})`),
    ).toBe('Puissant Awareness');
  });

  it('falls back to the ref when there is no i18n entry', () => {
    const ruleset = makeRuleset([], { i18n: {} });
    expect(displayName(ruleset, 'virtue.unknown')).toBe('virtue.unknown');
  });

  it('resolves a present param value through resolveValue (slug -> label)', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.great': { name: 'Great {characteristic}' } },
    });
    const resolve = (_key: string, value: string) =>
      value === 'characteristic.per' ? 'Perception' : value;
    expect(
      displayName(
        ruleset,
        'virtue.great',
        { characteristic: 'characteristic.per' },
        undefined,
        resolve,
      ),
    ).toBe('Great Perception');
  });

  it('does not call resolveValue for an empty param (uses placeholder hint)', () => {
    const ruleset = makeRuleset([], {
      i18n: { 'virtue.great': { name: 'Great {characteristic}' } },
    });
    const resolve = () => 'should not be used';
    expect(displayName(ruleset, 'virtue.great', {}, (key) => `(${key})`, resolve)).toBe(
      'Great (characteristic)',
    );
  });
});

// --- paramValueUsage() ------------------------------------------------------

describe('paramValueUsage', () => {
  const sel = (ref: string, value?: string) => ({
    ref,
    params: value ? { characteristic: value } : undefined,
  });

  it('counts other selections of the same item per value, excluding the row itself', () => {
    const selections = [
      sel('virtue.great', 'characteristic.per'),
      sel('virtue.great', 'characteristic.per'),
      sel('virtue.great', 'characteristic.str'),
      sel('virtue.other', 'characteristic.per'), // different item, ignored
    ];
    // From row 0's perspective: one other 'per' (row 1) and one 'str' (row 2).
    const usage = paramValueUsage(selections, 'virtue.great', 'characteristic', 0);
    expect(usage.get('characteristic.per')).toBe(1);
    expect(usage.get('characteristic.str')).toBe(1);
  });

  it('ignores rows with no value for the key', () => {
    const selections = [sel('virtue.great'), sel('virtue.great', 'characteristic.per')];
    const usage = paramValueUsage(selections, 'virtue.great', 'characteristic', 0);
    expect(usage.get('characteristic.per')).toBe(1);
    expect(usage.size).toBe(1);
  });
});

// --- groupByCategory() ------------------------------------------------------

describe('groupByCategory', () => {
  it('groups items by category, sorting items by localized name (not id)', () => {
    // ids are in one order; localized names invert it within each group, so a
    // name-based sort must reorder them.
    const ruleset = makeRuleset(
      [
        item({ id: 'virtue.a_general', category: 'general' }),
        item({ id: 'virtue.b_general', category: 'general' }),
        item({ id: 'virtue.m_hermetic', category: 'hermetic' }),
        item({ id: 'virtue.z_hermetic', category: 'hermetic' }),
      ],
      {
        i18n: {
          'virtue.a_general': { name: 'Zeal' },
          'virtue.b_general': { name: 'Affinity' },
          'virtue.m_hermetic': { name: 'Verditius' },
          'virtue.z_hermetic': { name: 'Bonisagus' },
        },
      },
    );
    const groups = groupByCategory(ruleset);

    expect(groups.map((g) => g.category)).toEqual(['general', 'hermetic']);
    // Sorted by name: Affinity < Zeal, Bonisagus < Verditius.
    expect(groups[0].items.map((i) => i.id)).toEqual(['virtue.b_general', 'virtue.a_general']);
    expect(groups[1].items.map((i) => i.id)).toEqual(['virtue.z_hermetic', 'virtue.m_hermetic']);
  });

  it('falls back to id ordering when names are absent', () => {
    const ruleset = makeRuleset([
      item({ id: 'virtue.b_general', category: 'general' }),
      item({ id: 'virtue.a_general', category: 'general' }),
    ]);
    expect(groupByCategory(ruleset)[0].items.map((i) => i.id)).toEqual([
      'virtue.a_general',
      'virtue.b_general',
    ]);
  });

  it('returns an empty array for an empty ruleset', () => {
    expect(groupByCategory(makeRuleset([]))).toEqual([]);
  });

  it('keeps only items whose kind is in the given filter', () => {
    const ruleset = makeRuleset([
      item({ id: 'virtue.a', kind: 'virtue', category: 'general' }),
      item({ id: 'boon.b', kind: 'boon', category: 'general' }),
      item({ id: 'flaw.c', kind: 'flaw', category: 'general' }),
      item({ id: 'hook.d', kind: 'hook', category: 'general' }),
    ]);

    const virtues = groupByCategory(ruleset, ['virtue', 'boon']);
    expect(virtues.flatMap((g) => g.items.map((i) => i.id))).toEqual(['boon.b', 'virtue.a']);

    const flaws = groupByCategory(ruleset, ['flaw', 'hook']);
    expect(flaws.flatMap((g) => g.items.map((i) => i.id))).toEqual(['flaw.c', 'hook.d']);
  });
});

// --- characteristicPointsUsed() ---------------------------------------------

describe('characteristicPointsUsed', () => {
  const rules: CharacteristicRules = {
    start_points: 7,
    costs: [
      { score: 3, cost: 6 },
      { score: 2, cost: 3 },
      { score: 1, cost: 1 },
      { score: 0, cost: 0 },
      { score: -1, cost: -1 },
      { score: -2, cost: -3 },
      { score: -3, cost: -6 },
    ],
  };

  it('nets spends against gains', () => {
    // Int +3 (6) + Per +1 (1) + Pre -3 (-6) + Com -1 (-1) + Qik +2 (3) + Str +2 (3) + Dex +1 (1) = 7
    expect(
      characteristicPointsUsed(rules, {
        int: 3,
        per: 1,
        pre: -3,
        com: -1,
        qik: 2,
        str: 2,
        dex: 1,
      }),
    ).toBe(7);
  });

  it('ignores out-of-range scores (contributes 0)', () => {
    expect(characteristicPointsUsed(rules, { str: 4 })).toBe(0);
  });

  it('returns 0 without rules or scores', () => {
    expect(characteristicPointsUsed(undefined, { int: 3 })).toBe(0);
    expect(characteristicPointsUsed(rules, undefined)).toBe(0);
  });
});

// --- abilityXpSpent() -------------------------------------------------------

describe('abilityXpSpent', () => {
  const advancement = [
    { score: 1, total_xp: 5 },
    { score: 2, total_xp: 15 },
    { score: 3, total_xp: 30 },
  ];

  it('sums total XP per whole bought score', () => {
    expect(abilityXpSpent(advancement, [{ score: 3 }, { score: 2 }])).toBe(45);
  });

  it('treats score 0 as no XP and skips unknown scores', () => {
    expect(abilityXpSpent(advancement, [{ score: 0 }, { score: 9 }])).toBe(0);
  });
});

// --- groupAbilitiesByCategory() ---------------------------------------------

describe('groupAbilitiesByCategory', () => {
  function withAbilities(
    abilities: Ability[],
    i18n: LocalizedRuleset['i18n'] = {},
  ): LocalizedRuleset {
    const map: Record<string, Ability> = {};
    for (const a of abilities) map[a.id] = a;
    return {
      ruleset: { id: 't', version: '1', point_items: {}, type_profiles: {}, abilities: map },
      i18n,
    };
  }

  it('groups by category in book order, sorting each group by localized name', () => {
    const groups = groupAbilitiesByCategory(
      withAbilities(
        [
          { id: 'ability.magic_theory', category: 'arcane' },
          { id: 'ability.swim', category: 'general' },
          { id: 'ability.awareness', category: 'general' },
          { id: 'ability.second_sight', category: 'supernatural' },
        ],
        {
          // German names: "Schwimmen" < "Aufmerksamkeit"? No — A < S, so Awareness
          // (Aufmerksamkeit) sorts before Swim (Schwimmen).
          'ability.swim': { name: 'Schwimmen' },
          'ability.awareness': { name: 'Aufmerksamkeit' },
        },
      ),
    );
    expect(groups.map((g) => g.category)).toEqual(['general', 'arcane', 'supernatural']);
    expect(groups[0].abilities.map((a) => a.id)).toEqual(['ability.awareness', 'ability.swim']);
  });

  it('sorts by name even when it inverts id order', () => {
    const groups = groupAbilitiesByCategory(
      withAbilities(
        [
          { id: 'ability.awareness', category: 'general' },
          { id: 'ability.bargain', category: 'general' },
        ],
        {
          'ability.awareness': { name: 'Wachsamkeit' },
          'ability.bargain': { name: 'Feilschen' },
        },
      ),
    );
    // Feilschen < Wachsamkeit, so bargain comes first despite the id order.
    expect(groups[0].abilities.map((a) => a.id)).toEqual(['ability.bargain', 'ability.awareness']);
  });

  it('is empty when the ruleset has no abilities', () => {
    expect(groupAbilitiesByCategory(withAbilities([]))).toEqual([]);
  });
});

// --- maxAbilityScore() ------------------------------------------------------

describe('maxAbilityScore', () => {
  it('returns 0 for undefined or empty advancement', () => {
    expect(maxAbilityScore(undefined)).toBe(0);
    expect(maxAbilityScore([])).toBe(0);
  });

  it('returns the highest score the table can price', () => {
    expect(maxAbilityScore([{ score: 1 }, { score: 5 }, { score: 3 }])).toBe(5);
  });
});

// --- abilityDisplayName() ---------------------------------------------------

describe('abilityDisplayName', () => {
  function withAbilityNames(
    abilities: Ability[],
    i18n: LocalizedRuleset['i18n'],
  ): LocalizedRuleset {
    const map: Record<string, Ability> = {};
    for (const a of abilities) map[a.id] = a;
    return {
      ruleset: { id: 't', version: '1', point_items: {}, type_profiles: {}, abilities: map },
      i18n,
    };
  }

  const placeholder = (key: string) => `(${key})`;

  it('interpolates the parameter value for a parameterized ability', () => {
    const ruleset = withAbilityNames(
      [{ id: 'ability.area_lore', category: 'general', parameter: 'area' }],
      {
        'ability.area_lore': { name: '{area} Lore' },
      },
    );
    expect(abilityDisplayName(ruleset, 'ability.area_lore', 'Rhine', placeholder)).toBe(
      'Rhine Lore',
    );
  });

  it('uses the placeholder label when the param value is empty', () => {
    const ruleset = withAbilityNames(
      [{ id: 'ability.area_lore', category: 'general', parameter: 'area' }],
      {
        'ability.area_lore': { name: '{area} Lore' },
      },
    );
    expect(abilityDisplayName(ruleset, 'ability.area_lore', '', placeholder)).toBe('(area) Lore');
    expect(abilityDisplayName(ruleset, 'ability.area_lore', null, placeholder)).toBe('(area) Lore');
  });

  it('returns the plain name for a non-parameterized ability', () => {
    const ruleset = withAbilityNames([{ id: 'ability.awareness', category: 'general' }], {
      'ability.awareness': { name: 'Awareness' },
    });
    expect(abilityDisplayName(ruleset, 'ability.awareness', 'ignored', placeholder)).toBe(
      'Awareness',
    );
  });
});

// --- abilityLabel() ---------------------------------------------------------

describe('abilityLabel', () => {
  function withAbilityNames(
    abilities: Ability[],
    i18n: LocalizedRuleset['i18n'],
  ): LocalizedRuleset {
    const map: Record<string, Ability> = {};
    for (const a of abilities) map[a.id] = a;
    return {
      ruleset: { id: 't', version: '1', point_items: {}, type_profiles: {}, abilities: map },
      i18n,
    };
  }

  const placeholder = (key: string) => `(${key})`;

  it('appends the supernatural marker for a Supernatural Ability', () => {
    const ruleset = withAbilityNames([{ id: 'ability.second_sight', category: 'supernatural' }], {
      'ability.second_sight': { name: 'Second Sight' },
    });
    expect(abilityLabel(ruleset, 'ability.second_sight', undefined, placeholder, '*')).toBe(
      'Second Sight*',
    );
  });

  it('does not mark a non-supernatural ability', () => {
    const ruleset = withAbilityNames([{ id: 'ability.awareness', category: 'general' }], {
      'ability.awareness': { name: 'Awareness' },
    });
    expect(abilityLabel(ruleset, 'ability.awareness', undefined, placeholder, '*')).toBe(
      'Awareness',
    );
  });
});
