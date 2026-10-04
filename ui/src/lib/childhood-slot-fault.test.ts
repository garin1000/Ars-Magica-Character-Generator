import { describe, expect, it } from 'vitest';

import { childhoodSlotFault } from './derive';
import type { ChildhoodPackage, LocalizedRuleset } from './types';

// N4a: the native language matches by catalogue value, not by spelling. The engine
// resolves a typed language against its catalogue names (case-folded and trimmed,
// `catalogue.rs::resolve_typed_instance`), so "arabic" and "Arabic" are one
// language. The picker's local check must agree, or it lets through a form the
// engine is bound to reject.
// Source: ArMDE:2378

/** Only what `childhoodSlotFault` reads: the childhood's native-language Ability. */
const localized = {
  ruleset: {
    life_stages: {
      childhood: { native_language_ability: 'ability.living_language' },
    },
  },
  i18n: {},
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
    const plan = { native_language: 'Arabic' };
    for (const language of ['arabic', ' ARABIC ']) {
      const slots = { area_a: 'Rhine', area_b: 'Provence', language };
      expect(childhoodSlotFault(localized, traveling, 'language', slots, plan)).toBe('native');
    }
  });

  it('faults two answers of one Ability that differ only in case as duplicates', () => {
    const plan = { native_language: 'Arabic' };
    const slots = { area_a: 'Rhine', area_b: ' rhine', language: 'Greek' };
    expect(childhoodSlotFault(localized, traveling, 'area_a', slots, plan)).toBe('duplicate');
    expect(childhoodSlotFault(localized, traveling, 'area_b', slots, plan)).toBe('duplicate');
  });

  it('still lets an Area Lore be named after the native language', () => {
    const plan = { native_language: 'German' };
    const slots = { area_a: 'german', area_b: 'Provence', language: 'Greek' };
    expect(childhoodSlotFault(localized, traveling, 'area_a', slots, plan)).toBeNull();
  });
});
