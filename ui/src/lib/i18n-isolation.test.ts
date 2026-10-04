import { describe, expect, it } from 'vitest';

import { buildBundle, translate } from './i18n';

// Try-out finding N8: text copied out of the app window carried Fluent's
// invisible bidi-isolation marks (U+2066-U+2069, FSI/PDI around every
// placeable), so a pasted read-out looked like "Later life (ages ⁨5⁩-...".
// Both shipped locales are left-to-right, so the marks buy nothing. The
// formatted strings are asserted raw here — never stripped — because the
// marks themselves are the defect.
const BIDI_ISOLATION_MARKS = /[⁦-⁩]/;

describe('Fluent placeables carry no bidi-isolation marks', () => {
  // Four placeables, two of them adjacent (`{ $min }{ $qualifier }`).
  const minimum = { ability: 'Parma Magica', min: '1', qualifier: '', score: '0' };
  const totals = { spent: '75', total: '75' };

  it.each([
    ['en', 'magus-minimum-met', minimum, 'Parma Magica 1 is met: this character has 0.'],
    ['en', 'xp-total', totals, 'XP: spent 75 of 75'],
    ['de', 'magus-minimum-met', minimum, 'Parma Magica 1 ist erfüllt: dieser Charakter hat 0.'],
    ['de', 'xp-total', totals, 'EP: 75 von 75 ausgegeben'],
  ] as const)('%s %s formats to plain text', (lang, key, args, expected) => {
    const message = translate(buildBundle(lang), key, args);
    expect(message).not.toMatch(BIDI_ISOLATION_MARKS);
    expect(message).toBe(expected);
  });
});
