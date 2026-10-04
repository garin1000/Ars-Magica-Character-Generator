import { describe, expect, it } from 'vitest';

import { spellGroupCapTooltip, type Translate } from './derive';
import { buildBundle, translate } from './i18n';
import type { Lang } from './i18n';
import type { SpellLevelCap } from './types';

// N1 (try-out 2026-10-04): the Spells tab's Te/Fo group header tooltip shows
// the Magical Focus and Potent Magic caps beside the plain spell-level cap,
// read from the engine's grid row (`SpellLevelCap`'s marked figures, present
// only while the Virtue is held). `ssr` project, no component mounted.

/** The real bundle, with Fluent's bidi isolation marks (U+2066-2069) stripped
 *  so the assertion reads the visible text. */
function translator(lang: Lang): Translate {
  const bundle = buildBundle(lang);
  return (key, args) => translate(bundle, key, args).replace(/[⁦-⁩]/g, '');
}

const plain: SpellLevelCap = {
  technique: 'art.creo',
  form: 'art.ignem',
  range_beyond_touch: false,
  cap: 17,
};

describe('spellGroupCapTooltip', () => {
  it('has no tooltip without a grid row', () => {
    expect(spellGroupCapTooltip(undefined, translator('en'))).toBeUndefined();
  });

  it('shows only the plain cap when neither Virtue is held', () => {
    expect(spellGroupCapTooltip(plain, translator('en'))).toBe('Spell-level cap: 17');
  });

  it('adds the within-Magical-Focus cap when a Magical Focus is held', () => {
    expect(spellGroupCapTooltip({ ...plain, within_focus_cap: 20 }, translator('en'))).toBe(
      'Spell-level cap: 17 · within Magical Focus: 20',
    );
  });

  it('adds the within-Potent-Magic-field cap when Potent Magic is held', () => {
    expect(spellGroupCapTooltip({ ...plain, within_potent_field_cap: 20 }, translator('en'))).toBe(
      'Spell-level cap: 17 · within Potent Magic field: 20',
    );
  });

  const both: SpellLevelCap = {
    ...plain,
    within_focus_cap: 20,
    within_potent_field_cap: 20,
    within_focus_and_potent_field_cap: 23,
  };

  it('adds all three marked caps when both Virtues are held (EN)', () => {
    expect(spellGroupCapTooltip(both, translator('en'))).toBe(
      'Spell-level cap: 17 · within Magical Focus: 20 · within Potent Magic field: 20' +
        ' · within Magical Focus and Potent Magic field: 23',
    );
  });

  it('adds all three marked caps when both Virtues are held (DE)', () => {
    expect(spellGroupCapTooltip(both, translator('de'))).toBe(
      'Zaubergrenze: 17 · im Magischen Fokus: 20 · im Bereich der Potenten Magie: 20' +
        ' · im Magischen Fokus und im Bereich der Potenten Magie: 23',
    );
  });

  it('signs a negative cap with the ASCII hyphen', () => {
    const text = spellGroupCapTooltip(
      { ...plain, cap: -3, within_focus_cap: -1 },
      translator('en'),
    );
    expect(text).toBe('Spell-level cap: -3 · within Magical Focus: -1');
    expect(text).not.toContain('−');
  });
});
