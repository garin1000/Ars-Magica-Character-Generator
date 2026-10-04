import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import { selectionDisplayName, type Translate } from './derive';
import { buildBundle, translate, type Lang } from './i18n';
import type { LocalizedRuleset } from './types';

// T2 (try-out finding 23, decision C3): the Art-parameterized Virtues and Flaws,
// read from the SHIPPED rules i18n and the SHIPPED Fluent hint labels, in both
// locales, through the same `selectionDisplayName` a V/F row and the picker use.
//
// Unfilled, an entry reads as its book heading (EN) or its translation-table row
// (DE, D31: the table wins on a name). Filled, it reads as the chosen Art in the
// template's slot. The two Deficiencies are the case C3 decided: EN "Deficient
// Creo" / "Deficient Ignem", DE "Defizitäre Technik: Creo" / "Defizitäre Form:
// Ignem" (no adjective agreement with a Latin Art name), unfilled "Deficient
// Technique" / "Deficient Form" (ArMDE:5909, :5913) and "Defizitäre Technik" /
// "Defizitäre Form" (tugenden-fehler.md rows). The Form-apposition entries keep
// D57's uninflected "der Form, Ignem" when filled.

function shippedRulesI18n(lang: Lang, file: string): LocalizedRuleset['i18n'] {
  return JSON.parse(
    readFileSync(
      fileURLToPath(new URL(`../../../rules/i18n/${lang}/${file}`, import.meta.url)),
      'utf-8',
    ),
  ) as LocalizedRuleset['i18n'];
}

function shippedLocalized(lang: Lang): LocalizedRuleset {
  return {
    ruleset: { abilities: {} } as unknown as LocalizedRuleset['ruleset'],
    i18n: {
      ...shippedRulesI18n(lang, 'arts.json'),
      ...shippedRulesI18n(lang, 'virtues_flaws.json'),
    },
  };
}

/** The shipped Fluent bundle, with Fluent's bidi isolation marks removed. */
function shippedTranslate(lang: Lang): Translate {
  const bundle = buildBundle(lang);
  return (key, args) => translate(bundle, key, args).replace(/[⁨⁩]/g, '');
}

interface Case {
  id: string;
  key: string;
  art: string;
  filled: string;
  unfilled: string;
}

const EN: Case[] = [
  {
    id: 'flaw.deficient_technique',
    key: 'technique',
    art: 'art.creo',
    filled: 'Deficient Creo',
    unfilled: 'Deficient Technique',
  },
  {
    id: 'flaw.deficient_form',
    key: 'form',
    art: 'art.ignem',
    filled: 'Deficient Ignem',
    unfilled: 'Deficient Form',
  },
  {
    id: 'virtue.puissant_art',
    key: 'art',
    art: 'art.creo',
    filled: 'Puissant Creo',
    unfilled: 'Puissant Art',
  },
  {
    id: 'virtue.affinity_art',
    key: 'art',
    art: 'art.creo',
    filled: 'Affinity with Creo',
    unfilled: 'Affinity with Art',
  },
  {
    id: 'virtue.imbued_with_the_spirit_of_form',
    key: 'form',
    art: 'art.ignem',
    filled: 'Imbued with the Spirit of Ignem',
    unfilled: 'Imbued with the Spirit of (Form)',
  },
  {
    id: 'virtue.extractor_of_form_vis',
    key: 'form',
    art: 'art.ignem',
    filled: 'Extractor of Ignem Vis',
    unfilled: 'Extractor of (Form) Vis',
  },
  {
    id: 'virtue.master_of_form_creatures',
    key: 'form',
    art: 'art.animal',
    filled: 'Master of Animal Creatures',
    unfilled: 'Master of (Form) Creatures',
  },
  {
    id: 'flaw.hunger_for_form_magic',
    key: 'form',
    art: 'art.ignem',
    filled: 'Hunger for Ignem Magic',
    unfilled: 'Hunger for (Form) Magic',
  },
];

const DE: Case[] = [
  {
    id: 'flaw.deficient_technique',
    key: 'technique',
    art: 'art.creo',
    filled: 'Defizitäre Technik: Creo',
    unfilled: 'Defizitäre Technik',
  },
  {
    id: 'flaw.deficient_form',
    key: 'form',
    art: 'art.ignem',
    filled: 'Defizitäre Form: Ignem',
    unfilled: 'Defizitäre Form',
  },
  {
    id: 'virtue.puissant_art',
    key: 'art',
    art: 'art.creo',
    filled: 'Begabung in Creo',
    unfilled: 'Begabung in (Kunst)',
  },
  {
    id: 'virtue.affinity_art',
    key: 'art',
    art: 'art.creo',
    filled: 'Affinität zu Creo',
    unfilled: 'Affinität zu (Kunst)',
  },
  {
    id: 'virtue.imbued_with_the_spirit_of_form',
    key: 'form',
    art: 'art.ignem',
    filled: 'Durchdrungen vom Geist der Form, Ignem',
    unfilled: 'Durchdrungen vom Geist der (Form)',
  },
  {
    id: 'virtue.extractor_of_form_vis',
    key: 'form',
    art: 'art.ignem',
    filled: 'Vis-Gewinner der Form, Ignem',
    unfilled: 'Vis-Gewinner der (Form)',
  },
  {
    id: 'virtue.master_of_form_creatures',
    key: 'form',
    art: 'art.animal',
    filled: 'Meister der Animal-Kreaturen',
    unfilled: 'Meister der (Form-)Kreaturen',
  },
  {
    id: 'flaw.hunger_for_form_magic',
    key: 'form',
    art: 'art.ignem',
    filled: 'Hunger nach Ignem-Magie',
    unfilled: 'Hunger nach (Form-)Magie',
  },
];

describe('Art-parameterized V/F names (T2, finding 23)', () => {
  for (const [lang, cases] of [
    ['en', EN],
    ['de', DE],
  ] as const) {
    const localized = shippedLocalized(lang);
    const t = shippedTranslate(lang);

    for (const c of cases) {
      it(`${lang} ${c.id}: unfilled reads as the heading`, () => {
        expect(selectionDisplayName(localized, c.id, undefined, t)).toBe(c.unfilled);
      });

      it(`${lang} ${c.id}: a chosen Art fills the slot`, () => {
        expect(selectionDisplayName(localized, c.id, { [c.key]: c.art }, t)).toBe(c.filled);
      });
    }
  }
});
