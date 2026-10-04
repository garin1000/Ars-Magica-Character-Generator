import { describe, expect, it } from 'vitest';

import { childhoodSlotFault } from './derive';
import type { ChildhoodPackage, LocalizedRuleset } from './types';

// N4a: the native language matches by catalogue value, not by spelling. The engine
// resolves a typed language against its catalogue names (case-folded and trimmed,
// `catalogue.rs::resolve_typed_instance`), so "arabic" and "Arabic" are one
// language. The picker's local check must agree, or it lets through a form the
// engine is bound to reject.
// N4b: the native language and a catalogued slot may also hold a value picked from
// the list (`{ id }`), which is the language its display name spells.
// Source: ArMDE:2378

/** What `childhoodSlotFault` reads: the native-language Ability and the language names. */
const localized = {
  ruleset: {
    life_stages: {
      childhood: { native_language_ability: 'ability.living_language' },
    },
  },
  i18n: {
    'language.arabic': { name: 'Arabic' },
    'language.greek': { name: 'Greek' },
  },
} as unknown as LocalizedRuleset;

/** Traveling Childhood as `rules/core/childhoods.json` ships it. */
const traveling: ChildhoodPackage = {
  id: 'childhood.traveling',
  entries: [
    { ability: 'ability.area_lore', score: 1, slot: 'area_a' },
    { ability: 'ability.area_lore', score: 1, slot: 'area_b' },
    { ability: 'ability.folk_ken', score: 2 },
    { ability: 'ability.living_language', score: 5, native: true },
    { ability: 'ability.living_language', score: 1, slot: 'language' },
    { ability: 'ability.survival', score: 2 },
  ],
};

describe('childhoodSlotFault compares languages as the engine resolves them', () => {
  it('faults a childhood language spelling the native language in another case', () => {
    const plan = { native_language: { text: 'Arabic' } };
    for (const language of ['arabic', ' ARABIC ']) {
      const slots = { area_a: 'Rhine', area_b: 'Provence', language };
      expect(childhoodSlotFault(localized, traveling, 'language', slots, plan)).toBe('native');
    }
  });

  it('faults two answers of one Ability that differ only in case as duplicates', () => {
    const plan = { native_language: { text: 'Arabic' } };
    const slots = { area_a: 'Rhine', area_b: ' rhine', language: 'Greek' };
    expect(childhoodSlotFault(localized, traveling, 'area_a', slots, plan)).toBe('duplicate');
    expect(childhoodSlotFault(localized, traveling, 'area_b', slots, plan)).toBe('duplicate');
  });

  it('still lets an Area Lore be named after the native language', () => {
    const plan = { native_language: { text: 'German' } };
    const slots = { area_a: 'german', area_b: 'Provence', language: 'Greek' };
    expect(childhoodSlotFault(localized, traveling, 'area_a', slots, plan)).toBeNull();
  });
});

describe('childhoodSlotFault compares picked languages by what they name (N4b)', () => {
  const areas = { area_a: 'Rhine', area_b: 'Provence' };

  it('faults a typed slot language naming the native language picked from the list', () => {
    const plan = { native_language: { id: 'language.arabic' } };
    const slots = { ...areas, language: ' arabic ' };
    expect(childhoodSlotFault(localized, traveling, 'language', slots, plan)).toBe('native');
  });

  it('faults a picked slot language repeating the typed or picked native language', () => {
    const slots = { ...areas, language: { id: 'language.arabic' } };
    for (const native_language of [{ text: 'Arabic' }, { id: 'language.arabic' }]) {
      expect(childhoodSlotFault(localized, traveling, 'language', slots, { native_language })).toBe(
        'native',
      );
    }
  });

  it('accepts a picked slot language other than the native one', () => {
    const plan = { native_language: { id: 'language.arabic' } };
    const slots = { ...areas, language: { id: 'language.greek' } };
    expect(childhoodSlotFault(localized, traveling, 'language', slots, plan)).toBeNull();
  });
});
