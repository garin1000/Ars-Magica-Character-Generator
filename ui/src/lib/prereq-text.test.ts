import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import { describePrereq, resolveIssueArgs, type Translate } from './derive';
import { buildBundle, translate as formatMessage } from './i18n';
import type { Lang } from './i18n';
import type { EntityTypeProfile, LocalizedRuleset, PointItem, Prereq } from './types';

// I1 (try-out finding 4, after-deadline Q2): "Prerequisite not met" never said
// WHICH prerequisite. `describePrereq` renders a `Prereq` tree as a localized
// phrase — every variant, names instead of ids, `all`/`any`/`none` as localized
// joins — and `resolveIssueArgs` folds it into the three prerequisite findings.
//
// `ssr` project: pure text, no component.

/**
 * The SHIPPED catalogue and rules i18n, so every name below is the one a player
 * actually reads — including Offensive to (Beings), the case the finding names.
 */
function shippedRuleset(lang: Lang): LocalizedRuleset {
  const read = (path: string) =>
    JSON.parse(readFileSync(fileURLToPath(new URL(path, import.meta.url)), 'utf-8'));
  const items = read('../../../rules/core/virtues_flaws.json') as PointItem[];
  const profiles = read('../../../rules/core/character_types.json') as EntityTypeProfile[];
  return {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: Object.fromEntries(items.map((i) => [i.id, i])),
      type_profiles: Object.fromEntries(profiles.map((p) => [p.id, p])),
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      ...read(`../../../rules/i18n/${lang}/virtues_flaws.json`),
      ...read(`../../../rules/i18n/${lang}/houses.json`),
      ...read(`../../../rules/i18n/${lang}/abilities.json`),
      ...read(`../../../rules/i18n/${lang}/arts.json`),
    },
  } as unknown as LocalizedRuleset;
}

/** Fluent wraps each interpolated value in bidi isolates; strip them for reading. */
function plain(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

function translator(lang: Lang): Translate {
  const bundle = buildBundle(lang);
  return (key, args) => formatMessage(bundle, key, args);
}

function describeIn(lang: Lang, prereq: Prereq): string {
  return plain(describePrereq(prereq, shippedRuleset(lang), translator(lang)));
}

const GIFT: Prereq = { kind: 'has', value: 'virtue.the_gift' };
const GENTLE: Prereq = { kind: 'has', value: 'virtue.gentle_gift' };
const JERBITON: Prereq = { kind: 'house', value: 'house.jerbiton' };
const TRAINED: Prereq = { kind: 'hermetically_trained' };

// Every leaf variant, as [prereq, English, German]. A raw id, slug or Fluent key
// in any cell would mean a leaf fell through to a fallback.
const LEAVES: [string, Prereq, string, string][] = [
  ['has', GIFT, 'The Gift', 'Die Gabe'],
  ['house', JERBITON, 'House Jerbiton', 'Haus Jerbiton'],
  [
    'ability_min',
    { kind: 'ability_min', value: { ability: 'ability.magic_theory', score: 3 } },
    'Magic Theory 3',
    'Magietheorie 3',
  ],
  ['art_min', { kind: 'art_min', value: { art: 'art.creo', score: 5 } }, 'Creo 5', 'Creo 5'],
  ['hermetically_trained', TRAINED, 'Hermetic training', 'hermetische Ausbildung'],
  [
    'order_member',
    { kind: 'order_member' },
    'membership in the Order of Hermes',
    'Mitgliedschaft im Orden des Hermes',
  ],
  ['is_companion', { kind: 'is_companion' }, 'being a companion', 'Gefährte zu sein'],
  ['is_grog', { kind: 'is_grog' }, 'being a grog', 'Grog zu sein'],
  [
    'has_category',
    { kind: 'has_category', value: 'supernatural' },
    'a Virtue or Flaw (Supernatural)',
    'eine Tugend oder ein Fehler (Übernatürlich)',
  ],
  [
    'age_min',
    { kind: 'age_min', value: 40 },
    'an age of at least 40',
    'ein Alter von mindestens 40',
  ],
  [
    'has_category_at_magnitude',
    {
      kind: 'has_category_at_magnitude',
      value: { category: 'supernatural', magnitude: 'major', item_kind: 'virtue' },
    },
    'a Virtue (Supernatural, at least Major)',
    'eine Tugend (Übernatürlich, mindestens Groß)',
  ],
  ['character_type', { kind: 'character_type', value: 'magus' }, 'being a Magus', 'Magus zu sein'],
  [
    'characteristic_min',
    { kind: 'characteristic_min', value: { characteristic: 'characteristic.pre', score: 1 } },
    'Presence of at least 1',
    'Präsenz mindestens 1',
  ],
  [
    'ability_category_score_min',
    { kind: 'ability_category_score_min', value: { category: 'supernatural', score: 1 } },
    'an Ability (Supernatural) with a score of at least 1',
    'eine Fertigkeit (Übernatürlich) mit einem Wert von mindestens 1',
  ],
  [
    'any_art_min',
    { kind: 'any_art_min', value: { score: 1 } },
    'any Art with a score of at least 1',
    'eine beliebige Kunst mit einem Wert von mindestens 1',
  ],
];

describe('describePrereq — leaves', () => {
  it.each(LEAVES)('renders %s in English', (_kind, prereq, en) => {
    expect(describeIn('en', prereq)).toBe(en);
  });

  it.each(LEAVES)('renders %s in German', (_kind, prereq, _en, de) => {
    expect(describeIn('de', prereq)).toBe(de);
  });

  // F-556: `virtue.domestic_animal` gates on a type id no shipped profile carries,
  // so there is no `type-<id>` label to show — and the id itself must not leak.
  it('names a character type no shipped profile offers without printing its id', () => {
    const prereq: Prereq = { kind: 'character_type', value: 'character_type.domestic_animal' };
    expect(describeIn('en', prereq)).toBe('a character type not available here');
    expect(describeIn('de', prereq)).toBe('ein hier nicht verfügbarer Charaktertyp');
  });

  // A negative threshold keeps its ASCII hyphen (CLAUDE.md), never U+2212.
  it('renders a negative Characteristic threshold with an ASCII hyphen', () => {
    const prereq: Prereq = {
      kind: 'characteristic_min',
      value: { characteristic: 'characteristic.pre', score: -2 },
    };
    expect(describeIn('en', prereq)).toBe('Presence of at least -2');
    expect(describeIn('de', prereq)).toBe('Präsenz mindestens -2');
    expect(describeIn('en', prereq)).not.toContain('−');
  });
});

describe('describePrereq — compounds', () => {
  it('joins two conditions with "and" / "und"', () => {
    const prereq: Prereq = { kind: 'all', value: [GIFT, JERBITON] };
    expect(describeIn('en', prereq)).toBe('The Gift and House Jerbiton');
    expect(describeIn('de', prereq)).toBe('Die Gabe und Haus Jerbiton');
  });

  it('lists three conditions with the separator before the final "and"', () => {
    const prereq: Prereq = { kind: 'all', value: [GIFT, JERBITON, TRAINED] };
    expect(describeIn('en', prereq)).toBe('The Gift, House Jerbiton and Hermetic training');
    expect(describeIn('de', prereq)).toBe('Die Gabe, Haus Jerbiton und hermetische Ausbildung');
  });

  it('joins alternatives with "or" / "oder"', () => {
    const prereq: Prereq = { kind: 'any', value: [GIFT, GENTLE] };
    expect(describeIn('en', prereq)).toBe('The Gift or Gentle Gift');
    expect(describeIn('de', prereq)).toBe('Die Gabe oder Sanfte Gabe');
  });

  it('renders a one-child "none" as "without …"', () => {
    const prereq: Prereq = { kind: 'none', value: [GIFT] };
    expect(describeIn('en', prereq)).toBe('without The Gift');
    expect(describeIn('de', prereq)).toBe('ohne Die Gabe');
  });

  it('renders a several-child "none" as "none of: …"', () => {
    const prereq: Prereq = { kind: 'none', value: [GIFT, GENTLE] };
    expect(describeIn('en', prereq)).toBe('none of: The Gift, Gentle Gift');
    expect(describeIn('de', prereq)).toBe('keines von: Die Gabe, Sanfte Gabe');
  });

  it('renders a one-child "all" or "any" as the child alone', () => {
    expect(describeIn('en', { kind: 'all', value: [GIFT] })).toBe('The Gift');
    expect(describeIn('en', { kind: 'any', value: [JERBITON] })).toBe('House Jerbiton');
  });

  // Without grouping, "The Gift or Gentle Gift and Hermetic training" could be
  // read either way round; a nested compound is bracketed so it cannot.
  it('brackets a compound nested inside another compound', () => {
    const prereq: Prereq = {
      kind: 'all',
      value: [{ kind: 'any', value: [GIFT, GENTLE] }, TRAINED],
    };
    expect(describeIn('en', prereq)).toBe('(The Gift or Gentle Gift) and Hermetic training');
    expect(describeIn('de', prereq)).toBe('(Die Gabe oder Sanfte Gabe) und hermetische Ausbildung');
  });

  it('brackets three levels of nesting, all/any/none', () => {
    const prereq: Prereq = {
      kind: 'any',
      value: [{ kind: 'all', value: [JERBITON, { kind: 'none', value: [GIFT, GENTLE] }] }, TRAINED],
    };
    expect(describeIn('en', prereq)).toBe(
      '(House Jerbiton and (none of: The Gift, Gentle Gift)) or Hermetic training',
    );
    expect(describeIn('de', prereq)).toBe(
      '(Haus Jerbiton und (keines von: Die Gabe, Sanfte Gabe)) oder hermetische Ausbildung',
    );
  });

  // The rules directory is a trust boundary: a tree deeper than the engine
  // accepts must not recurse without bound in the UI either.
  it('stops at PREREQ_MAX_DEPTH instead of recursing without bound', () => {
    let deep: Prereq = GIFT;
    for (let i = 0; i < 200; i += 1) deep = { kind: 'all', value: [deep, TRAINED] };
    const text = describeIn('en', deep);
    expect(text).toContain('…');
    expect(text).not.toContain('The Gift');
  });
});

describe('describePrereq — shipped entries', () => {
  // ArMDE:6530: "Characters with The Gift may take this Flaw only if they have
  // the Gentle Gift" — data: none-of(The Gift) OR Gentle Gift.
  it('names what Offensive to (Beings) requires', () => {
    const tree = (lang: Lang) =>
      shippedRuleset(lang).ruleset.point_items['flaw.offensive_to_beings'].prerequisites!;
    expect(describeIn('en', tree('en'))).toBe('(without The Gift) or Gentle Gift');
    expect(describeIn('de', tree('de'))).toBe('(ohne Die Gabe) oder Sanfte Gabe');
  });

  it('names both halves of Broken Vessel’s requirement', () => {
    const tree = shippedRuleset('en').ruleset.point_items['flaw.broken_vessel'].prerequisites!;
    expect(describeIn('en', tree)).toBe(
      'an Ability (Supernatural) with a score of at least 1 or any Art with a score of at least 1',
    );
  });
});

describe('the prerequisite findings name the requirement', () => {
  /** The finding as a player reads it: engine args → resolved labels → the real bundle. */
  function render(lang: Lang, code: string, item: string): string {
    const t = translator(lang);
    const resolved = resolveIssueArgs(shippedRuleset(lang), { item }, t, code);
    return plain(t(`issue-${code}`, resolved));
  }

  it('says what Offensive to (Beings) on a magus is missing', () => {
    expect(render('en', 'prereq_not_met', 'flaw.offensive_to_beings')).toBe(
      'Prerequisite not met for Offensive to (Beings). Requires: (without The Gift) or Gentle Gift.',
    );
    expect(render('de', 'prereq_not_met', 'flaw.offensive_to_beings')).toBe(
      'Voraussetzung für Abstoßend für (Wesen) nicht erfüllt. Erfordert: (ohne Die Gabe) oder Sanfte Gabe.',
    );
  });

  it('names the requirement in the could-not-check-yet warning too', () => {
    expect(render('en', 'prereq_unevaluated', 'flaw.offensive_to_beings')).toBe(
      'Prerequisite for Offensive to (Beings) could not be checked yet. Requires: (without The Gift) or Gentle Gift.',
    );
  });

  // The advisory finding reads the item's OWN hedged tree, not `prerequisites`.
  it('names the hedged requirement in the advisory warning', () => {
    expect(render('en', 'advisory_prereq_not_met', 'flaw.vendetta')).toBe(
      'Prerequisite for Vendetta is not normally met. Requires: House Verditius.',
    );
    expect(render('de', 'advisory_prereq_not_met', 'flaw.vendetta')).toBe(
      'Voraussetzung für Vendetta ist normalerweise nicht erfüllt. Erfordert: Haus Verditius.',
    );
  });

  // An item the ruleset does not know (a stale save, a renamed id) has no tree
  // to describe: the sentence must still close cleanly, with no `{$requirement}`.
  it('leaves the clause out when the item carries no tree', () => {
    expect(render('en', 'prereq_not_met', 'virtue.nonesuch')).toBe(
      'Prerequisite not met for virtue.nonesuch.',
    );
  });
});
