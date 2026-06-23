import { describe, expect, it } from 'vitest';

import { balance, displayName, groupByCategory } from './derive';
import type { Entity, LocalizedRuleset, PointItem } from './types';

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
});

// --- groupByCategory() ------------------------------------------------------

describe('groupByCategory', () => {
  it('groups items by category, sorting groups and items by id', () => {
    const ruleset = makeRuleset([
      item({ id: 'virtue.b_general', category: 'general' }),
      item({ id: 'virtue.a_general', category: 'general' }),
      item({ id: 'virtue.z_hermetic', category: 'hermetic' }),
      item({ id: 'virtue.m_hermetic', category: 'hermetic' }),
    ]);
    const groups = groupByCategory(ruleset);

    expect(groups.map((g) => g.category)).toEqual(['general', 'hermetic']);
    expect(groups[0].items.map((i) => i.id)).toEqual(['virtue.a_general', 'virtue.b_general']);
    expect(groups[1].items.map((i) => i.id)).toEqual(['virtue.m_hermetic', 'virtue.z_hermetic']);
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
