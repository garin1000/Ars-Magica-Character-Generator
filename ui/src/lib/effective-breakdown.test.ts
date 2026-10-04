import { describe, expect, it } from 'vitest';

import { effectiveBreakdownTooltip, scoreSourceLines, type Translate } from './derive';
import { buildBundle, translate } from './i18n';
import type { LocalizedRuleset, ScoreSource } from './types';

// I3 (try-out finding 15): the effective-score badge on the Arts and Abilities
// tabs explains itself the way the Characteristics badge already does — the
// bought score, one line per SOURCE (the Virtue that contributed, by its localized
// name, with its signed amount), and the effective score. These two pure helpers
// are the shared shape all three badges build their tooltip from; `ssr` project,
// no component mounted.

/** A translator that echoes the key and its arguments, so an assertion can see
 *  exactly which message was asked for with which values. */
const echo: Translate = (key, args) =>
  `${key}(${Object.entries(args ?? {})
    .map(([k, v]) => `${k}=${v}`)
    .join(',')})`;

const localized = {
  ruleset: { point_items: {} },
  i18n: {
    'virtue.puissant_art': { name: 'Puissant {art}', name_unfilled: 'Puissant Art' },
    'virtue.elemental_magic': { name: 'Elemental Magic' },
    'flaw.example_penalty': { name: 'Example Penalty' },
  },
} as unknown as LocalizedRuleset;

describe('scoreSourceLines', () => {
  it('names each source by its localized item name with a signed amount', () => {
    const sources: ScoreSource[] = [
      { source: 'virtue.puissant_art', amount: 3 },
      { source: 'virtue.elemental_magic', amount: 2 },
    ];
    expect(scoreSourceLines(sources, localized, echo)).toEqual([
      'effective-tooltip-source(name=Puissant Art,amount=+3)',
      'effective-tooltip-source(name=Elemental Magic,amount=+2)',
    ]);
  });

  it('signs a negative contribution with the ASCII hyphen, via formatSigned', () => {
    const lines = scoreSourceLines(
      [{ source: 'flaw.example_penalty', amount: -1 }],
      localized,
      echo,
    );
    expect(lines).toEqual(['effective-tooltip-source(name=Example Penalty,amount=-1)']);
    expect(lines[0]).not.toContain('−');
  });

  it('never renders the raw item id as the name', () => {
    const lines = scoreSourceLines([{ source: 'virtue.puissant_art', amount: 3 }], localized, echo);
    expect(lines.join(' ')).not.toContain('virtue.puissant_art');
  });

  it('is empty for no sources', () => {
    expect(scoreSourceLines([], localized, echo)).toEqual([]);
  });
});

describe('effectiveBreakdownTooltip', () => {
  it('summarises bought and effective, then lists the source lines', () => {
    expect(
      effectiveBreakdownTooltip({ bought: '6', effective: '12' }, ['line a', 'line b'], echo),
    ).toEqual({
      text: 'effective-tooltip-summary(bought=6,effective=12)',
      listLabel: 'effective-tooltip-breakdown-label()',
      list: ['line a', 'line b'],
    });
  });
});

describe('effective breakdown messages exist in both locales', () => {
  const strip = (s: string) => s.replace(/[⁦-⁩]/g, '');

  it('renders the summary and a source line in English', () => {
    const en = buildBundle('en');
    const summary = translate(en, 'effective-tooltip-summary', { bought: '6', effective: '12' });
    expect(strip(summary)).toBe('Bought 6, effective 12.');
    expect(strip(translate(en, 'effective-tooltip-breakdown-label'))).toBe('Includes');
    expect(
      strip(translate(en, 'effective-tooltip-source', { name: 'Puissant Art', amount: '+3' })),
    ).toBe('Puissant Art +3');
  });

  it('renders the summary and a source line in German', () => {
    const de = buildBundle('de');
    const summary = translate(de, 'effective-tooltip-summary', { bought: '6', effective: '12' });
    expect(strip(summary)).toBe('Basiswert 6, effektiv 12.');
    expect(strip(translate(de, 'effective-tooltip-breakdown-label'))).toBe('Enthält');
    expect(
      strip(translate(de, 'effective-tooltip-source', { name: 'Elementarmagie', amount: '+3' })),
    ).toBe('Elementarmagie +3');
  });
});
