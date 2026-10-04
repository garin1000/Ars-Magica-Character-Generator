import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

import { buildBundle, translate } from './i18n';
import type { ParameterDomain } from './types';

// The real locale sources, loaded the same way i18n.ts loads them, so these
// tests exercise the shipped German .ftl rather than a synthetic bundle.
const ftlSources = import.meta.glob('../../../locales/*/main.ftl', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function sourceForLang(lang: string): string {
  const entry = Object.entries(ftlSources).find(([path]) => path.includes(`/locales/${lang}/`));
  if (!entry) throw new Error(`missing locale: ${lang}`);
  return entry[1];
}

// Message ids declared at column 0 (`key = value`). Ignores comments, blank
// lines, indented continuations, attribute lines (`.attr =`), and terms (`-id`).
function messageKeys(src: string): Set<string> {
  const keys = new Set<string>();
  for (const line of src.split('\n')) {
    const match = /^([A-Za-z][\w-]*)\s*=/.exec(line);
    if (match) keys.add(match[1]);
  }
  return keys;
}

// A message's full value: the text after its `=`, plus any indented continuation
// lines that belong to it. Read from the raw `.ftl` rather than through the
// bundle so an assertion about WORDING needs no placeable arguments.
function messageValue(src: string, key: string): string {
  const lines = src.split('\n');
  const head = lines.findIndex((line) => new RegExp(`^${key}\\s*=`).test(line));
  if (head === -1) throw new Error(`missing message: ${key}`);
  const value = [lines[head].replace(/^[^=]*=/, '')];
  for (const line of lines.slice(head + 1)) {
    if (!/^\s+\S/.test(line)) break;
    value.push(line);
  }
  return value.join('\n');
}

// E9 (full-audit round 2). Every `$name` a message's VALUE references, per key —
// including indented continuation and attribute lines, which carry placeables
// too, and including selector references (`{ $count ->` ), which are variable
// references like any other. Sets, not sequences: German word order is free to
// differ, and must stay free to.
function messageVariables(src: string): Map<string, Set<string>> {
  const vars = new Map<string, Set<string>>();
  let current: Set<string> | null = null;
  for (const line of src.split('\n')) {
    const declaration = /^([A-Za-z][\w-]*)\s*=(.*)$/.exec(line);
    if (declaration) {
      current = new Set();
      vars.set(declaration[1], current);
    } else if (!/^\s+\S/.test(line)) {
      // A blank line, a comment, or a term — whatever message was open ends here.
      current = null;
      continue;
    }
    if (!current) continue;
    for (const [, name] of line.matchAll(/\$([A-Za-z][\w-]*)/g)) current.add(name);
  }
  return vars;
}

describe('German UI bundle', () => {
  it('builds the real German Fluent bundle and resolves known keys to German', () => {
    const de = buildBundle('de');
    // A real German string (added for the spell picker) resolves, not echoed back.
    expect(translate(de, 'spell-already-taken-reason')).toBe('Bereits ausgewählt');
    // A key present in both locales resolves to a value (never the key itself).
    expect(translate(de, 'spell-budget-reason')).not.toBe('spell-budget-reason');
  });

  // The guided life-stage funding chrome is where a wrong German word is most
  // visible, so its terms are pinned to the rulebook's own wording
  // (Basisregeln.md:2364-2394) instead of merely existing.
  it('names the life-stage funding chrome as the German rulebook does', () => {
    const de = buildBundle('de');
    expect(translate(de, 'ability-funding-life_stages')).toBe('Lebensabschnitte');
    expect(translate(de, 'native-language-label')).toBe('Muttersprache');
    expect(translate(de, 'childhood-label')).toBe('Beispielhafte Kindheit');
    // A magus is built through its life stages too (slice 6b4), so the guided chrome
    // names apprenticeship and the Gauntlet exactly as the German rulebook does
    // (Basisregeln.md:2433, :2449 — Lehrlingszeit, Lehrlingsprüfung). The prose that
    // carried this term went with manual-testing-findings #21; the FIELD LABEL is
    // what has to keep it, since it is now the only place the word is read.
    expect(translate(de, 'life-stage-gauntlet-age-label')).toContain('Lehrlingsprüfung');
    // The two checklist headings are the rulebook's own (Basisregeln.md:2437, :2451).
    expect(translate(de, 'magus-minimums-label')).toBe('Mindestfertigkeiten');
    expect(translate(de, 'magus-recommended-label')).toBe('Empfohlene Mindestfertigkeiten');
    // Fertigkeiten, never "Fähigkeiten": the glossary's word for an Ability.
    expect(translate(de, 'magus-minimums-summary', { unmet: '3', total: '7' })).not.toContain(
      'Fähigkeiten',
    );
    // Whole sentences per status, so neither row leans on colour alone. `qualifier`
    // is the trailing "(Dead Language)" note, empty for a requirement that names
    // no exemplar — and never absent, because an unresolved variable renders as a
    // literal `{$qualifier}` in the sentence. (It used to THROW; `translate` now
    // collects the error and returns the partial instead — see i18n.ts.)
    const row = { ability: 'Parma Magica', min: '1', score: '0', qualifier: '' };
    expect(translate(de, 'magus-minimum-met', row)).toContain('erfüllt');
    expect(translate(de, 'magus-minimum-unmet', row)).toContain('nicht erfüllt');
  });

  // Sabine 4 (full-audit round 4). A control and the validation messages that
  // refer to the same object must share a word, or the user cannot map the error
  // onto anything on screen. English always did — "Special abilities" against
  // "Mastery ability" / "Mastery special abilities" — while German set
  // "Besondere Fähigkeiten" against "Meisterschaftsfähigkeit" and shared no word
  // at all. The repo's own glossary settles which side moves:
  // `rules/source/de/translation-tables/grundbegriffe.md:671` gives the full form
  // "Besondere Fähigkeiten gemeisterter Zauber" and names
  // "Meisterschaftsfähigkeit(en)" the sanctioned short form for running text —
  // which is exactly what a one-line control label is. So the label adopts the
  // errors' word, not the reverse.
  //
  // Pinned as a shared STEM rather than as five fixed strings: what must hold is
  // the PAIRING. Either side may be rewritten later, but not apart from the other.
  it('names Spell Mastery abilities with one term in the control and in its errors', () => {
    // A word stem, not a whole word: German compounds and inflects it
    // (…fähigkeit/…fähigkeiten) and English pluralizes it.
    const stems: Record<string, string> = { en: 'abilit', de: 'Meisterschaftsfähigkeit' };
    for (const [lang, stem] of Object.entries(stems)) {
      const src = sourceForLang(lang);
      for (const key of [
        // The control: the chip row's label, and the add-`<select>`'s own
        // `aria-label` — which is what a screen-reader user hears.
        'spell-mastery-abilities-label',
        'spell-mastery-ability-add',
        // Every issue the engine can raise about that control's contents.
        'issue-unknown_mastery_ability',
        'issue-too_many_mastery_abilities',
        'issue-duplicate_mastery_ability',
      ]) {
        expect(
          messageValue(src, key),
          `${lang}/${key} does not name the object the way its siblings do`,
        ).toContain(stem);
      }
    }
  });

  // Slice 2 (#1, #11): the read-only `type` step is gone and the `experience` step
  // took the funding choice off the Abilities step. This pins that the removed
  // phase's keys left with it, so no key names a phase the engine no longer has.
  //
  // manual-testing-findings #21 then removed the whole `wizard-guidance-*` family,
  // so its absence is asserted here too: a surviving key would be a dead string
  // nothing renders, which a later reader would take for live copy.
  it('keys the experience phase, and neither the removed type phase nor any guidance', () => {
    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      expect(keys).toContain('phase-experience');
      expect(keys).not.toContain('phase-type');
      expect([...keys].filter((key) => key.startsWith('phase-type-'))).toEqual([]);
      expect([...keys].filter((key) => key.startsWith('wizard-guidance-'))).toEqual([]);
    }
  });

  // guided-creation-review-2026-08 #14: the life-stage read-outs became one chip per
  // block, keyed `xp-pool-block-*`, and the three `life-stage-*` chip keys they
  // replaced are retired. Both halves matter — a surviving old key is a dead string
  // that a later reader will take for a live one, and a missing new key renders as
  // its own slug.
  it('keys one xp-pool block chip per life stage and retires the old chip keys', () => {
    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      for (const key of [
        'xp-pool-block-early-childhood',
        'xp-pool-block-early-childhood-spread-only',
        'xp-pool-block-later-life',
        'xp-pool-block-later-life-restricted',
        'xp-pool-block-apprenticeship',
        'xp-pool-block-after-gauntlet',
      ]) {
        expect(keys, `${lang} is missing ${key}`).toContain(key);
      }
      for (const retired of [
        'life-stage-later-life',
        'life-stage-apprenticeship',
        'life-stage-post-gauntlet',
      ]) {
        expect(keys, `${lang} still carries the retired ${retired}`).not.toContain(retired);
      }
      // The block-slug keys STAY: `resolveIssueArgValue` names an unspent-experience
      // warning's `origin` through them, and that is not the bar's chip.
      for (const slugKey of [
        'xp-pool-childhood_native_language',
        'xp-pool-childhood_spread',
        'xp-pool-later_life',
        'xp-pool-apprenticeship',
      ]) {
        expect(keys, `${lang} dropped the issue-arg key ${slugKey}`).toContain(slugKey);
      }
    }
  });

  // #14's fourth decision: "After the Gauntlet" replaces "As a magus" on the two
  // LABELS that name the block, and nowhere else — the six prose strings that say
  // "years as a magus" read correctly and keep it. The German term is the glossary's
  // (`rules/source/de/translation-tables/grundbegriffe.md:57` — Lehrlingsprüfung).
  it('names the post-Gauntlet block after the Gauntlet, in both bars', () => {
    const en = buildBundle('en');
    const de = buildBundle('de');
    for (const key of ['xp-pool-block-after-gauntlet', 'spell-levels-post-gauntlet']) {
      const args = { years: '10', rate: '30', lab: '0', points: '300', xp: '300', levels: '300' };
      expect(translate(en, key, args)).toContain('After the Gauntlet');
      expect(translate(en, key, args)).not.toContain('As a magus');
      expect(translate(de, key, args)).toContain('Lehrlingsprüfung');
      expect(translate(de, key, args)).not.toContain('Als Magus');
    }
    // The one surviving piece of prose on that panel keeps "as a magus"/"als Magus":
    // it is a description of a read-only state, not the block's label.
    expect(translate(en, 'life-stage-post-gauntlet-no-years-note')).toContain('as a magus');
    expect(translate(de, 'life-stage-post-gauntlet-no-years-note')).toContain('als Magus');
  });

  // The German block names are the rulebook's own, at the mirrored lines
  // (Basisregeln.md:2213 Frühe Kindheit, :2214 Späteres Leben, :2215 Lehrlingszeit).
  it('names the life-stage blocks as the German rulebook does', () => {
    const de = buildBundle('de');
    const span = { from: '5', to: '10', years: '5', rate: '15', xp: '75', used: '0', amount: '75' };
    expect(
      translate(de, 'xp-pool-block-early-childhood', {
        nativeUsed: '0',
        nativeAmount: '75',
        spreadUsed: '0',
        spreadAmount: '45',
      }),
    ).toContain('Frühe Kindheit');
    expect(translate(de, 'xp-pool-block-later-life-restricted', span)).toContain('Späteres Leben');
    expect(translate(de, 'xp-pool-block-apprenticeship', { years: '15', xp: '240' })).toContain(
      'Lehrlingszeit',
    );
  });

  // guided-creation-review-2026-08 #30: two new warning codes. A code with no
  // `issue-<code>` message renders as its own slug, which is the very thing
  // CLAUDE.md forbids — and locale parity alone would not catch it, because a code
  // missing from BOTH locales is perfectly symmetrical.
  it('names both unspent-budget findings in both locales', () => {
    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      expect(keys, `${lang} is missing issue-general_xp_unspent`).toContain(
        'issue-general_xp_unspent',
      );
      expect(keys, `${lang} is missing issue-spell_levels_unspent`).toContain(
        'issue-spell_levels_unspent',
      );
    }
  });

  // #30's wording rule: `restricted_xp_unspent` may say the points are wasted
  // (childhood's blocks are spend-or-lose), but the Core Rules make no such
  // statement about the general pool or the 120 levels of spells. So these two
  // messages count and stop — asserted, because "factual" is the requirement and a
  // later editor "improving" the copy would silently invent a rule.
  it.each(['en', 'de'])('states the unspent budgets as a bare count (%s)', (lang) => {
    const bundle = buildBundle(lang as 'en' | 'de');
    const clean = (s: string) => s.replace(/[⁦-⁩]/g, '');
    const xp = clean(
      translate(bundle, 'issue-general_xp_unspent', { pool: '240', used: '40', unspent: '200' }),
    );
    const levels = clean(
      translate(bundle, 'issue-spell_levels_unspent', {
        budget: '120',
        used: '60',
        unspent: '60',
      }),
    );
    for (const message of [xp, levels]) {
      // The count reaches the sentence…
      expect(message).toMatch(/\b(200|60)\b/);
      // …and the wasted-points claim of the restricted sibling does not.
      expect(message.toLowerCase()).not.toMatch(/wast|verfall|verlor|verlier/);
    }
    // The sibling that IS allowed to say it still does, so this is a real contrast
    // rather than a vacuous check on strings that never mention waste anyway.
    const restricted = clean(
      translate(bundle, 'issue-restricted_xp_unspent', {
        origin: 'Educated',
        amount: '50',
        used: '0',
        unspent: '50',
      }),
    );
    expect(restricted.toLowerCase()).toMatch(/wast|verfall/);
  });

  // manual-testing-findings-2026-09-03 #10 (reopened): `mythic_companion` is a
  // real Virtue category — `### Mythic Companion, Free` is one of the headings the
  // `## List of Virtues` index groups by (ArMDE:3329). A category with no
  // `category-<id>` message renders as its own slug in the badge, the filter
  // dropdown and the exported Type cell, and parity alone would not catch it
  // because a key missing from BOTH locales is perfectly symmetrical.
  it('names the Mythic Companion Virtue category in both locales', () => {
    for (const lang of ['en', 'de']) {
      expect(messageKeys(sourceForLang(lang))).toContain('category-mythic_companion');
    }
    // The German term is the rulebook's own heading (Basisregeln.md:3329) and the
    // glossary's (translation-tables/grundbegriffe.md:83) — and the same string the
    // character-type label already uses, so one concept reads one way everywhere.
    const de = buildBundle('de');
    expect(translate(de, 'category-mythic_companion')).toBe('Mythischer Gefährte');
    expect(translate(de, 'category-mythic_companion')).toBe(translate(de, 'type-mythic_companion'));
  });

  // F7/D83.7: "You must take the Free Virtue defining which type of Mythic
  // Companion you are" (ArMDE:2846). The engine reports an unchosen type as an
  // error, so the message may not soften it to a "should". The German names the
  // character type with the glossary's term (translation-tables/grundbegriffe.md:83).
  it('says a Mythic Companion must choose a type, in both locales', () => {
    const en = translate(buildBundle('en'), 'issue-mythic_type_unset');
    expect(en).toMatch(/\bmust\b/);
    expect(en).not.toMatch(/\bshould\b/);
    const de = translate(buildBundle('de'), 'issue-mythic_type_unset');
    expect(de).toMatch(/\bmuss\b/);
    expect(de).not.toMatch(/\bsollte\b/);
    expect(de).toContain('Mythischer Gefährte');
  });

  // R6/D83.7: a bought status Virtue is an error, and the Available list greys it
  // out with a reason. Both strings exist in both locales — a key missing from
  // both is perfectly symmetrical, so parity alone would not catch it — and the
  // error names the Virtue it reports.
  it('names the bought status-Virtue finding and its picker reason in both locales', () => {
    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      expect(keys, `${lang}`).toContain('issue-mythic_status_virtue_bought');
      expect(keys, `${lang}`).toContain('vf-blocked-mythic-status');
    }
    for (const lang of ['en', 'de'] as const) {
      const message = translate(buildBundle(lang), 'issue-mythic_status_virtue_bought', {
        item: 'Devil Child',
      }).replace(/[⁦-⁩]/g, '');
      expect(message).toContain('Devil Child');
    }
  });

  // B7 (ArMDE:3919): a new validation code, `exclusive_param_values`.
  // A code with no `issue-<code>` message renders as its own slug, and parity
  // alone would not catch it — a code missing from BOTH locales is perfectly
  // symmetrical. The message must also stay free of any realm name: which
  // values exclude each other is rules DATA (`at_most_one_of`), so a locale
  // that spelled out "Divine"/"Infernal" would freeze one item's group into a
  // string every other item's group would then read wrongly.
  it('names the exclusive-values finding in both locales, without naming a Realm', () => {
    for (const lang of ['en', 'de']) {
      expect(messageKeys(sourceForLang(lang))).toContain('issue-exclusive_param_values');
    }
    for (const lang of ['en', 'de'] as const) {
      const message = translate(buildBundle(lang), 'issue-exclusive_param_values', {
        item: 'Folk Magic',
        key: 'Realm',
        count: '2',
      }).replace(/[⁦-⁩]/g, '');
      expect(message).toContain('2');
      expect(message.toLowerCase()).not.toMatch(/divine|infernal|göttlich|infernal/);
    }
  });

  // E5 (ArMDE:6482, open-todos row 29): a new validation code,
  // `too_many_for_param_value`. Unlike `exclusive_param_values` above it DOES
  // name the offending value — there is exactly one of it, and "twice for
  // something" is not a finding a player can act on — so both locales must
  // interpolate `$value` alongside the key, the count and the cap.
  it('names the per-parameter-value cap in both locales, naming the value', () => {
    for (const lang of ['en', 'de']) {
      expect(messageKeys(sourceForLang(lang))).toContain('issue-too_many_for_param_value');
    }
    for (const lang of ['en', 'de'] as const) {
      const message = translate(buildBundle(lang), 'issue-too_many_for_param_value', {
        item: 'Necessary Aura',
        key: lang === 'de' ? 'Fertigkeit' : 'Ability',
        value: 'Awareness',
        count: '2',
        max: '1',
      }).replace(/[⁦-⁩]/g, '');
      expect(message).toContain('Awareness');
      expect(message).toContain('2');
      expect(message).toContain(lang === 'de' ? 'Fertigkeit' : 'Ability');
    }
  });

  // D42/D70/D74: the three realm-association findings, plus X10b's
  // `banked_xp_at_or_above_next_level` — none of the four had a spot-check
  // case, so a future placeholder/param-name drift on any of them would pass
  // silently. Same pattern as `issue-too_many_for_param_value` above: each
  // code's own `$`-params rendered with real values in both locales.
  it('names the three realm-association findings and the banked-XP finding in both locales', () => {
    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      for (const code of [
        'issue-realm_changed_default',
        'issue-realm_unset_subset',
        'issue-realm_override_invalid',
        'issue-banked_xp_at_or_above_next_level',
      ]) {
        expect(keys, `${lang} is missing ${code}`).toContain(code);
      }
    }
    for (const lang of ['en', 'de'] as const) {
      const bundle = buildBundle(lang);
      const clean = (s: string) => s.replace(/[⁦-⁩]/g, '');

      const changedDefault = clean(
        translate(bundle, 'issue-realm_changed_default', { item: 'Faerie Blood' }),
      );
      expect(changedDefault).toContain('Faerie Blood');

      const unsetSubset = clean(
        translate(bundle, 'issue-realm_unset_subset', { item: 'Manifest Sin' }),
      );
      expect(unsetSubset).toContain('Manifest Sin');

      const overrideInvalid = clean(
        translate(bundle, 'issue-realm_override_invalid', {
          item: 'Faerie Blood',
          value: 'realm.infernal',
        }),
      );
      expect(overrideInvalid).toContain('Faerie Blood');
      expect(overrideInvalid).toContain('realm.infernal');

      const bankedXp = clean(
        translate(bundle, 'issue-banked_xp_at_or_above_next_level', {
          banked: '10',
          needed: '5',
        }),
      );
      expect(bankedXp).toContain('10');
      expect(bankedXp).toContain('5');
    }
  });

  // E2 (open-todos row 24): `unknown_param_value` is the finding a player meets
  // when a save's typed realm word no longer resolves, and it names the DOMAIN
  // the value failed in. The engine emits that as the `ParameterDomain` enum's
  // serialized name, so without a label per variant the German message read
  // "unbekannten realm-Wert" — an English slug inside a German sentence, the
  // very thing CLAUDE.md forbids. `Record<ParameterDomain, ...>` is what keeps
  // this honest: a new engine variant mirrored into the frontend union is a
  // type error here until it is labelled, so no variant can slip through
  // unlabelled the way `realm` did.
  const PARAM_DOMAINS: Record<ParameterDomain, true> = {
    ability: true,
    art: true,
    technique: true,
    form: true,
    characteristic: true,
    item: true,
    enumerated: true,
    category: true,
    realm: true,
    text: true,
    number: true,
    spell: true,
    ability_category: true,
  };

  it('labels every parameter domain in both locales', () => {
    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      for (const domain of Object.keys(PARAM_DOMAINS)) {
        expect(keys, `${lang} is missing param-domain-${domain}`).toContain(
          `param-domain-${domain}`,
        );
      }
    }
    // The German term is the glossary's (translation-tables/sphären-mächte.md:16,
    // grundbegriffe.md:97 — Realm → Sphäre), the same word `param-label-realm`
    // and the `realm-<id>` family already use.
    expect(translate(buildBundle('de'), 'param-domain-realm')).toBe('Sphäre');
  });

  // The other half of row 24: whatever the player typed must reach the sentence
  // intact — that IS the mechanism by which the choice is not lost — while the
  // domain beside it arrives as a word, never as its slug.
  it.each(['en', 'de'])('shows the typed value back, and no domain slug (%s)', (lang) => {
    const bundle = buildBundle(lang as 'en' | 'de');
    const clean = (s: string) => s.replace(/[⁦-⁩]/g, '');
    const message = clean(
      translate(bundle, 'issue-unknown_param_value', {
        item: 'Bound to (Realm)',
        key: translate(bundle, 'param-label-realm'),
        value: 'Feenreich',
        domain: translate(bundle, 'param-domain-realm'),
      }),
    );
    expect(message).toContain('Feenreich');
    expect(message).not.toContain('realm');
  });

  // Round-1 audit, Sabine 6: the four German Realm labels were a mixed
  // register — two bare terms (Magie, Fee) and two carrying a definite article
  // (Das Göttliche, Das Infernale). They are `<option>` labels and they are
  // interpolated into slots (`might-effective = Effektive Macht: { $realm }
  // { $score }`, and the `..., {realm}` apposition every German V/F name
  // template uses), where the article produced "Effektive Macht: Das Göttliche
  // 15". The rulebook's own four-term enumeration is article-free
  // (`Basisregeln.md:2960`: "mit einer der vier Sphären verbunden: Magie, Fee,
  // Infernal und Göttlich"), and CLAUDE.md's standalone-label rule wants the
  // uninflected form. The glossary (`translation-tables/sphären-mächte.md:20`)
  // offers "Das Göttliche / Göttliche Sphäre"; neither is a standalone label,
  // so the rulebook's enumeration is what this follows.
  it('names the four Realms in the rulebook’s own article-free register', () => {
    const de = buildBundle('de');
    expect(translate(de, 'realm-magic')).toBe('Magie');
    expect(translate(de, 'realm-faerie')).toBe('Fee');
    expect(translate(de, 'realm-divine')).toBe('Göttlich');
    expect(translate(de, 'realm-infernal')).toBe('Infernal');
    // The whole point: no label carries an article the slot cannot absorb.
    for (const realm of ['magic', 'faerie', 'divine', 'infernal']) {
      expect(translate(de, `realm-${realm}`)).not.toMatch(/^(Der|Die|Das) /);
    }
  });

  // Round-1 audit, Sabine 5: an Ability is a **Fertigkeit**. The glossary maps
  // it that way and the German rulebook says *Übernatürliche Fertigkeiten*
  // throughout (`Basisregeln.md:1065, :1067, :2315, :2872`), which is why the
  // rest of the bundle already reads Mindestfertigkeiten /
  // Fertigkeitskategorie. Two strings about the Gift's one free Ability said
  // *Fähigkeit* instead — the very word `magus-minimums-summary` is already
  // asserted not to contain, so the repo treats it as an error everywhere but
  // here.
  //
  // Deliberately NOT covered by this test, because both are correct German
  // rather than the game term: `characteristic-desc-int` / `-per` use
  // *Fähigkeit* in its ordinary sense ("the ability to analyse"), and
  // `spell-mastery-abilities-label` is the rulebook's own heading verbatim
  // (`Basisregeln.md:9524` — "Besondere Fähigkeiten gemeisterter Zauber").
  //
  // F3 (tmp/ftl-rules-audit.md) dropped the "Gift's one free Ability" clause, so
  // the picker tooltip no longer names the game term at all. It must still never
  // say Fähigkeit; only the issue message is held to say Fertigkeit.
  it('calls an Ability a Fertigkeit wherever it names the game term', () => {
    const de = buildBundle('de');
    for (const key of ['ability-requires-virtue', 'issue-supernatural_ability_requires_virtue']) {
      const message = translate(de, key, { ability: 'Zweites Gesicht' });
      expect(message, `${key} still says Fähigkeit`).not.toContain('Fähigkeit');
    }
    expect(
      translate(de, 'issue-supernatural_ability_requires_virtue', { ability: 'Zweites Gesicht' }),
    ).toContain('Fertigkeit');
  });

  // Round-1 audit, Sabine 4: the German Fatigue ladder is the rulebook's own
  // six-term enumeration, given verbatim at
  // `Ars Magica Definitive Edition Basisregeln.md:17127` ("Ausgeruht, Außer
  // Atem, Erschöpft, Müde, Betäubt und Bewusstlos") and in the glossary
  // (`translation-tables/grundbegriffe.md:173` — Fresh → Ausgeruht). Four of the
  // five keys already matched and only `fresh` said `Frisch`, a word that
  // appears nowhere in the German rulebook as a Fatigue tier — which is what
  // makes it an oversight rather than a choice. All five are pinned, so the next
  // one to drift is caught too.
  it('names every Fatigue level as the German rulebook does', () => {
    const de = buildBundle('de');
    expect(translate(de, 'derived-fatigue-fresh')).toBe('Ausgeruht');
    expect(translate(de, 'derived-fatigue-winded')).toBe('Außer Atem');
    expect(translate(de, 'derived-fatigue-weary')).toBe('Erschöpft');
    expect(translate(de, 'derived-fatigue-tired')).toBe('Müde');
    expect(translate(de, 'derived-fatigue-dazed')).toBe('Betäubt');
  });

  // Round-1 audit, Sabine 11: `validate_category_caps` composes its issue code
  // from a category slug read out of `rules/core/character_types.json`
  // (`caps.rs` — `too_many_{category}_{noun}`, or `too_many_major_…` for a
  // Major-only cap). The four codes the shipped data produces all have keys, but
  // nothing pinned that — and CLAUDE.md explicitly blesses adding a cap as a
  // DATA-ONLY change, which is precisely the edit that would print a raw code
  // into the validation panel. Parity alone cannot catch it: a code missing from
  // both locales is perfectly symmetrical.
  //
  // The list is walked, never spelled out, so a fifth cap is covered the day it
  // is added rather than the day someone remembers this test.
  it('names every category cap the shipped character types declare, in both locales', () => {
    interface CategoryCap {
      category: string;
      max: number;
      major_only?: boolean;
    }
    interface TypeProfile {
      id: string;
      budget: { flaw_category_caps?: CategoryCap[]; virtue_category_caps?: CategoryCap[] };
    }
    const profiles: TypeProfile[] = JSON.parse(
      readFileSync(
        fileURLToPath(new URL('../../../rules/core/character_types.json', import.meta.url)),
        'utf-8',
      ),
    );

    /** The code exactly as `caps.rs` composes it. */
    const codeFor = (cap: CategoryCap, noun: string) =>
      cap.major_only
        ? `too_many_major_${cap.category}_${noun}`
        : `too_many_${cap.category}_${noun}`;

    const codes = new Set<string>();
    for (const profile of profiles) {
      for (const cap of profile.budget.flaw_category_caps ?? []) codes.add(codeFor(cap, 'flaws'));
      for (const cap of profile.budget.virtue_category_caps ?? [])
        codes.add(codeFor(cap, 'virtues'));
    }
    // A guard over an empty set would pass vacuously.
    expect(codes.size).toBeGreaterThan(0);

    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      for (const code of codes) {
        expect(keys, `${lang} is missing issue-${code}`).toContain(`issue-${code}`);
      }
    }
  });

  // B2 (D41, ArMDE:2816): `CategoryCap.min`/`min_hard` (B1, landed) is the
  // floor twin of the ceiling test above, composed by `caps.rs` the same way
  // (`too_few_<category>_<noun>`, or `too_few_major_…` for a Major-only cap).
  // Walked from the shipped `character_types.json`, never spelled out, on the
  // identical reasoning: a floor is DATA (CLAUDE.md), so adding one must not
  // silently print a raw code into the validation panel. The guard against an
  // empty set is deliberately load-bearing here, not just defensive: B2's own
  // data (a `social_status` row with `min: 1`) has not landed yet, so today
  // this set IS empty and the test fails for exactly that reason.
  it('names every category floor the shipped character types declare, in both locales', () => {
    interface CategoryCap {
      category: string;
      min?: number;
      major_only?: boolean;
    }
    interface TypeProfile {
      id: string;
      budget: { flaw_category_caps?: CategoryCap[]; virtue_category_caps?: CategoryCap[] };
    }
    const profiles: TypeProfile[] = JSON.parse(
      readFileSync(
        fileURLToPath(new URL('../../../rules/core/character_types.json', import.meta.url)),
        'utf-8',
      ),
    );

    /** The code exactly as `caps.rs`'s floor half composes it. */
    const codeFor = (cap: CategoryCap, noun: string) =>
      cap.major_only ? `too_few_major_${cap.category}_${noun}` : `too_few_${cap.category}_${noun}`;

    const codes = new Set<string>();
    for (const profile of profiles) {
      for (const cap of profile.budget.flaw_category_caps ?? [])
        if (cap.min !== undefined) codes.add(codeFor(cap, 'flaws'));
      for (const cap of profile.budget.virtue_category_caps ?? [])
        if (cap.min !== undefined) codes.add(codeFor(cap, 'virtues'));
    }
    // A guard over an empty set would pass vacuously — and today it IS empty
    // (D41's `social_status` floor has not landed), so this is this test's
    // own red.
    expect(codes.size).toBeGreaterThan(0);

    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      for (const code of codes) {
        expect(keys, `${lang} is missing issue-${code}`).toContain(`issue-${code}`);
      }
    }
  });

  // E2 (V/F audit Q-32): a new `AdvancementSource` variant, `authoring`, is
  // rendered through `derived-detail-authoring` (`DerivedSurfacedModifiersSection.svelte`)
  // exactly like every other scalar source. A code with no `derived-detail-<id>`
  // message renders as its own slug, which parity alone would not catch, since a
  // key missing from both locales is symmetrical.
  it('names the authoring advancement source in both locales', () => {
    for (const lang of ['en', 'de']) {
      expect(
        messageKeys(sourceForLang(lang)),
        `${lang} is missing derived-detail-authoring`,
      ).toContain('derived-detail-authoring');
    }
    // The German rulebook's own verb for writing a book (Basisregeln.md:6296,
    // "aus einem von dir verfassten Buch"), matching the gerund-noun register
    // `derived-detail-teaching` ("Unterrichten") already uses.
    expect(translate(buildBundle('de'), 'derived-detail-authoring')).toBe('Verfassen');
  });

  it('has full message-key parity between English and German', () => {
    // A missing German key silently falls back to English (or the key) at
    // runtime, so drift is invisible without this check — the same class of gap
    // that hid the missing German spell descriptions.
    const en = messageKeys(sourceForLang('en'));
    const de = messageKeys(sourceForLang('de'));
    const missingInDe = [...en].filter((key) => !de.has(key)).sort();
    const missingInEn = [...de].filter((key) => !en.has(key)).sort();
    expect({ missingInDe, missingInEn }).toEqual({ missingInDe: [], missingInEn: [] });
  });

  // E9 (full-audit round 2): key parity alone is not locale parity. Since
  // `translate` (i18n.ts) calls `formatPattern` in its three-argument form, an
  // unresolved variable no longer throws — @fluent/bundle's resolver returns
  // `FluentNone("$name")`, which renders as the literal `{$name}` INSIDE the
  // sentence, and `translate` discards the collected errors by design. Its doc
  // comment justifies discarding them on the grounds that the i18n tests catch
  // authoring mistakes at build time; this is the test that makes that true for
  // this class. Without it, a German value referencing a variable no caller
  // passes ships a raw identifier into the German UI — the "never render a raw
  // identifier as a user-facing label" invariant, breached in the one locale
  // nothing else checks — with vitest, eslint, prettier, svelte-check and
  // `cargo tauri build` all green.
  it('references the same variables per key in English and German', () => {
    const en = messageVariables(sourceForLang('en'));
    const de = messageVariables(sourceForLang('de'));
    // A guard over an empty map would pass vacuously.
    expect([...en.values()].filter((set) => set.size > 0).length).toBeGreaterThan(0);

    const mismatches: Record<string, { en: string[]; de: string[] }> = {};
    for (const [key, enVars] of en) {
      const deVars = de.get(key);
      // Key parity is the test above's job; judge only keys both locales carry.
      if (!deVars) continue;
      const enSorted = [...enVars].sort();
      const deSorted = [...deVars].sort();
      if (enSorted.join('|') !== deSorted.join('|')) {
        mismatches[key] = { en: enSorted, de: deSorted };
      }
    }
    expect(mismatches).toEqual({});
  });

  // Round-1 audit (orchestrator, from Sabine's @Gerda tag): `formatPattern` was
  // called in its TWO-argument form, which makes `@fluent/bundle` **throw** on a
  // resolution error rather than return a partial string. Every `store.t()` call
  // site is unguarded and many sit inside `$derived`, so one message
  // interpolating a variable its caller omitted would take down the whole
  // render — the app going blank because a *label* could not be built. No
  // reachable trigger exists in the shipped data; this is defence in depth, and
  // a renderable sentence with one empty slot beats a blank window either way.
  it('renders what it can instead of throwing when an argument is missing', () => {
    const en = buildBundle('en');
    // `magus-minimum-met` interpolates ability/min/score/qualifier; omit two.
    const args = { ability: 'Parma Magica', min: '1' };
    expect(() => translate(en, 'magus-minimum-met', args)).not.toThrow();
    const message = translate(en, 'magus-minimum-met', args).replace(/[⁦-⁩]/g, '');
    // The arguments that DID resolve still reach the sentence.
    expect(message).toContain('Parma Magica');
    expect(message).toContain('1');
  });

  // E6 (round-1 audit): the "never render a raw slug as a user-facing label"
  // invariant depends on translate() falling back to the key itself when the
  // message is missing, but every other test here exercises that fallback only
  // indirectly (through call sites that happen to resolve). Pin the branch
  // directly.
  it('falls back to the key itself for a message the bundle does not have', () => {
    const en = buildBundle('en');
    expect(translate(en, 'totally-bogus-key-xyz')).toBe('totally-bogus-key-xyz');
  });

  // guided-creation-review-2026-08 #22: `aging-total-formula` named exactly the
  // book's three terms while the total it states is the engine's sum of all of them,
  // so a character with an aging-roll Virtue or Flaw read a sentence that did not add
  // up. The sibling `aging-total-parts` already carried the term, which is what makes
  // it an oversight — so this pins BOTH halves: the argument reaches the sentence, and
  // it is worded exactly as the sibling words it rather than diverging from it.
  it.each(['en', 'de'])('names the Virtue/Flaw term in the aging total formula (%s)', (lang) => {
    const bundle = buildBundle(lang as 'en' | 'de');
    // Fluent wraps interpolated values in bidi isolation marks; strip them.
    const clean = (s: string) => s.replace(/[⁦-⁩]/g, '');
    const args = {
      die: '+8',
      age: '+4',
      conditions: '0',
      longevity: '0',
      traits: '-1',
      fixed: '+3',
    };
    const formula = clean(translate(bundle, 'aging-total-formula', args));
    const parts = clean(translate(bundle, 'aging-total-parts', args));

    // The sibling's own wording for the term, read out of the sibling itself.
    const sibling = /-1\s*\(([^)]+)\)/.exec(parts);
    expect(sibling).not.toBeNull();
    expect(formula).toContain(`-1 (${sibling![1]})`);
  });

  // One Rust enum value, two render sites: the editor radio
  // (`longevity-source-<v>`) and the Totals read-out (`derived-longevity-<v>`).
  // English renders both identically, so a divergence shows up only in German —
  // the same value named two ways across two tabs of one character.
  it.each(['en', 'de'])('renders each longevity source the same in both panels (%s)', (lang) => {
    const bundle = buildBundle(lang as 'en' | 'de');
    for (const source of ['self_made', 'external']) {
      expect(translate(bundle, `derived-longevity-${source}`)).toBe(
        translate(bundle, `longevity-source-${source}`),
      );
    }
  });

  // UI review 2026-09-30 #3: `param-label-company` used to say "Kompanie" — a
  // military reading — while Educated (Vernacular)'s own German summary
  // already names the same referent (a merchant/trading company)
  // "Unternehmen". Pins the two texts to the SAME term so a player does not
  // see one word in the picker and a different one in the Virtue's own text.
  it('names "company" the same way the picker label and Educated (Vernacular) do', () => {
    const virtuesFlaws = JSON.parse(
      readFileSync(
        fileURLToPath(new URL('../../../rules/i18n/de/virtues_flaws.json', import.meta.url)),
        'utf-8',
      ),
    ) as Record<string, { summary?: string }>;
    const summary = virtuesFlaws['virtue.educated_vernacular']?.summary;
    expect(summary).toBeDefined();
    const label = translate(buildBundle('de'), 'param-label-company');
    expect(summary).toContain(label);
  });
});

// DE .ftl audit (tmp/ftl-audit.md, 2026-10-03): D1-D3, S1-S6 and the pre-existing
// items 1-6. Each message is formatted through the real German bundle and pinned
// to its whole target sentence, because the defects are in the wording itself.
//
// Every `issue-*` arg is passed as a STRING, as the app passes it: the engine's
// `ValidationIssue::args` is one string per key, and `resolveIssueArgs` (derive.ts)
// returns `Record<string, string>`. @fluent/bundle only selects a plural category
// (`[one]`) for a FluentNumber, so a selector on an issue arg would always fall to
// `*[other]` in the shipped app. That is why the count-bearing fixes below
// (D2, item 1) are worded to read correctly at every count, including 1.
describe('German UI bundle, DE .ftl audit fixes', () => {
  const de = buildBundle('de');
  // Fluent wraps each interpolated value in bidi isolation marks; strip them.
  const say = (key: string, args?: Record<string, string>) =>
    translate(de, key, args).replace(/[⁦-⁩]/g, '');

  it('D1: points the blocked wizard at the mode labels the app really shows', () => {
    expect(say('wizard-blocked-hint')).toBe(
      'Behebe die Fehler dieses Schritts, um fortzufahren, oder stelle die Prüfung unter „Einstellungen“ auf „Hinweise“.',
    );
    // The hint and the controls it names must agree, whichever side moves later.
    expect(say('wizard-blocked-hint')).toContain(`„${say('mode-advisory')}“`);
    expect(say('wizard-blocked-hint')).toContain(`„${say('settings-title')}“`);
  });

  it('D2: states the lab-season limit correctly for one year as a magus', () => {
    // Wording since F1 (tmp/ftl-rules-audit.md): `max` is the seasons the years
    // hold, four a year, not the seasons that can be charged.
    expect(
      say('issue-life_stage_lab_seasons_out_of_range', { seasons: '5', max: '4', years: '1' }),
    ).toBe('5 Quartale Laborarbeit sind mehr als die 4 Quartale in den Jahren als Magus (1).');
    expect(
      say('issue-life_stage_lab_seasons_out_of_range', { seasons: '9', max: '8', years: '2' }),
    ).toBe('9 Quartale Laborarbeit sind mehr als die 8 Quartale in den Jahren als Magus (2).');
  });

  // Same defect as D2, outside the audit: LifeStagePanel.svelte passes
  // `String(budget.post_gauntlet_years)`, so a selector could not help here either.
  it('states the post-Gauntlet summary correctly for one year as a magus', () => {
    expect(
      say('life-stage-post-gauntlet-summary', { years: '1', points: '30', xp: '30', levels: '0' }),
    ).toBe('Jahre als Magus: 1; 30 Punkte = 30 EP + Zauberstufen (0)');
  });

  it('D3: never guesses the grammatical gender of a Virtue or Flaw name', () => {
    expect(say('issue-realm_changed_default', { item: 'Verfluchte Täuschung' })).toBe(
      'Verfluchte Täuschung weicht von der üblichen Sphäre ab; bitte bestätigen, dass dies beabsichtigt ist.',
    );
    expect(say('issue-realm_unset_subset', { item: 'Offenbarte Sünde' })).toBe(
      'Offenbarte Sünde benötigt eine Sphäre aus der eingeschränkten Liste dieses Eintrags; ein Rückfall auf Magie ist hier nicht möglich.',
    );
  });

  it('S1: names the Reputation source without hyphen-compounding a data name', () => {
    const source = { source: 'Außenseiter (Groß)' };
    expect(say('reputation-level-increment', source)).toBe(
      'Reputation von Außenseiter (Groß) erhöhen',
    );
    expect(say('reputation-level-decrement', source)).toBe(
      'Reputation von Außenseiter (Groß) verringern',
    );
  });

  it('S2: calls the Focus Power budget a Vorrat, as the rest of the bundle does', () => {
    expect(say('issue-over_focus_points', { used: '12', budget: '10', over: '2' })).toBe(
      'Fokussierte Mächte verbrauchen 12 Punkte, über dem Vorrat von 10 (um 2).',
    );
  });

  it('S3: names the Ability Category parameter as its domain and filter do', () => {
    expect(say('param-label-class')).toBe('Fertigkeitskategorie');
    expect(say('param-label-class')).toBe(say('param-domain-ability_category'));
  });

  it('S4: capitalises Sozialer Status as the category label does', () => {
    expect(say('issue-too_few_social_status_virtues', { count: '0', min: '1' })).toBe(
      'Zu wenige Tugenden oder Fehler des Sozialen Status (0 von min. 1).',
    );
    expect(say('issue-too_many_social_status_virtues', { item: 'Ritter', other: 'Bauer' })).toBe(
      'Mehr als eine Tugend oder ein Fehler des Sozialen Status gewählt (Ritter, Bauer).',
    );
  });

  it('S5: says the character has no Magical Focus, not that none is held', () => {
    expect(
      say('issue-spell_within_focus_without_magical_focus', { spell: 'Ball des Abyssalen Feuers' }),
    ).toBe(
      'Ball des Abyssalen Feuers ist als im Fokus markiert, aber der Charakter hat keinen Magischen Fokus; die Markierung hat keine Wirkung.',
    );
  });

  it('S6: says a spell above the cap is allowed within the Magical Focus', () => {
    expect(say('spell-cap-within-focus-reason', { cap: '15' })).toBe(
      'Über deiner Zaubergrenze (15); im Rahmen deines Magischen Fokus erlaubt',
    );
    expect(say('spell-add-within-focus-tooltip', { cap: '15' })).toBe(
      'Im Rahmen deines Magischen Fokus erlaubt (Grenze 15)',
    );
  });

  it('item 1: states owed Warping choices correctly at a count of 1', () => {
    expect(say('issue-warping_owed_minor_flaws', { count: '1' })).toBe(
      'Noch offene Kleine Fehler aus der Verzerrung: 1.',
    );
    expect(say('issue-warping_owed_supernatural_virtues', { count: '1' })).toBe(
      'Noch offene Übernatürliche Kleine Tugenden aus der Verzerrung: 1.',
    );
    expect(say('issue-warping_owed_major_flaws', { count: '1' })).toBe(
      'Noch offene Große Fehler aus der Verzerrung: 1.',
    );
    expect(say('issue-warping_owed_major_flaws', { count: '3' })).toBe(
      'Noch offene Große Fehler aus der Verzerrung: 3.',
    );
  });

  it('item 2: never assumes a neuter item in the Warping-fill finding', () => {
    expect(
      say('issue-warping_fill_ineligible', { choice_key: 'Kleiner Fehler 1', item: 'Verhexung' }),
    ).toBe(
      'Die Verzerrungswahl Kleiner Fehler 1 fällt auf Verhexung; dieser Eintrag gewährt selbst Verzerrung und kann keinen Verzerrungsplatz füllen.',
    );
  });

  it('item 3: addresses the user with du in the export error', () => {
    expect(say('error-export', { missing: 'Tugenden' })).toBe(
      'Das Charakterblatt konnte nicht exportiert werden: In der aktuellen Sprache fehlt Text für Tugenden. Versuche es mit Englisch als Sprache erneut, oder melde dies als Fehler.',
    );
  });

  it('item 4: quotes data values with German typographic quotes', () => {
    expect(say('issue-unknown_living_condition', { condition: 'Palast' })).toBe(
      'Der Lebensumstand „Palast“ entspricht keiner Zeile der Lebensumstände-Tabelle und geht daher nicht in den Alterungswurf ein.',
    );
    expect(say('issue-living_conditions_conflict', { condition: 'Palast', other: 'Hütte' })).toBe(
      'Die Lebensumstände „Palast“ und „Hütte“ schließen einander aus, es kann also nur einer davon gelten.',
    );
    expect(say('issue-unknown_equipment', { item: 'Lanze' })).toBe(
      'Ausrüstung „Lanze“ passt zu keiner Waffe, keinem Schild und keiner Rüstung.',
    );
  });

  it('item 5: capitalises Große/Kleine in the Virtue and Flaw cap findings', () => {
    const cap = { count: '2', max: '1' };
    expect(say('issue-too_many_major_virtues', cap)).toBe(
      'Zu viele Große Tugenden (2 von max. 1).',
    );
    expect(say('issue-too_many_major_hermetic_virtues', cap)).toBe(
      'Zu viele Große Hermetische Tugenden (2 von max. 1).',
    );
    expect(say('issue-too_many_major_flaws', cap)).toBe('Zu viele Große Fehler (2 von max. 1).');
    expect(say('issue-too_many_minor_flaws', cap)).toBe('Zu viele Kleine Fehler (2 von max. 1).');
    expect(say('issue-too_many_major_personality_flaws', cap)).toBe(
      'Zu viele Große Persönlichkeitsfehler (2 von max. 1).',
    );
  });

  it('item 6: writes Zaubermeisterschaft as the glossary does', () => {
    expect(say('spell-mastery-increment', { name: 'Pilum des Feuers' })).toBe(
      'Zaubermeisterschaft für Pilum des Feuers erhöhen',
    );
    expect(say('spell-mastery-decrement', { name: 'Pilum des Feuers' })).toBe(
      'Zaubermeisterschaft für Pilum des Feuers verringern',
    );
  });

  // The mechanism behind D2/item 1, pinned so a later edit cannot reintroduce a
  // plural selector that the shipped app would never take: no `issue-*` message
  // in either locale selects on an argument.
  it('puts no plural selector in any issue message, since issue args are strings', () => {
    // The library behaviour this rests on: the same count selects `[one]` as a
    // number and falls to `*[other]` as a string.
    expect(say('warping-owed-minor-flaws', { count: '1' })).toBe('1 Kleine Fehler');
    expect(translate(de, 'warping-owed-minor-flaws', { count: 1 }).replace(/[⁦-⁩]/g, '')).toBe(
      '1 Kleiner Fehler',
    );
    for (const lang of ['en', 'de']) {
      const src = sourceForLang(lang);
      const selecting = [...messageKeys(src)].filter(
        (key) => key.startsWith('issue-') && /\{\s*\$[\w-]+\s*->/.test(messageValue(src, key)),
      );
      expect(selecting, `${lang} issue messages with a selector`).toEqual([]);
    }
  });
});

// EN messages that rendered "1 levels", "1 values" … at a count of 1. Args are
// passed exactly as the app passes them: STRINGS for every `issue-*` message
// (`derive.ts::resolveIssueArgs`), and a NUMBER for the Aging outcome, which
// AgingRollCalculator.svelte passes raw.
// A string arg can never select `[one]`, so each fix is worded to read correctly
// at every count (an "(s)" hedge, or the count moved out of the noun's way).
describe('English UI bundle, wording at a count of 1', () => {
  const en = buildBundle('en');
  const say = (key: string, args?: Record<string, string | number>) =>
    translate(en, key, args).replace(/[⁦-⁩]/g, '');

  it('states a one-point Decrepitude-and-Crisis outcome correctly', () => {
    expect(say('aging-outcome-decrepitude_and_crisis', { points: 1 })).toBe(
      '1 Aging Point(s) — enough to reach the next level of Decrepitude — and a Crisis.',
    );
  });

  it('states a single lab season without years as a magus correctly', () => {
    expect(say('issue-life_stage_lab_seasons_without_years', { seasons: '1' })).toBe(
      '1 lab season(s) recorded, but this character has no years as a magus to work them in.',
    );
  });

  it('states a one-level spell split beyond the points correctly', () => {
    expect(
      say('issue-life_stage_spell_level_split_exceeds_points', { levels: '1', points: '0' }),
    ).toBe(
      'Taking 1 level(s) of spells out of the years as a magus is more than those years grant: they are worth 0 points, to be divided between experience and levels of spells.',
    );
  });

  // An Art at 0 needs 1 XP for its next level, so banked 1 / needed 1 is reachable.
  it('states one banked experience point against one needed correctly', () => {
    expect(
      say('issue-banked_xp_at_or_above_next_level', { art: 'Creo', banked: '1', needed: '1' }),
    ).toBe(
      'Banked experience points (1) are already enough to raise this score — the next level needs only 1.',
    );
  });

  it('states one Mastery special ability above a Mastery score of 0 correctly', () => {
    expect(
      say('issue-too_many_mastery_abilities', {
        spell: 'Pilum of Fire',
        chosen: '1',
        mastery: '0',
      }),
    ).toBe(
      'Pilum of Fire has more Mastery special abilities (1) than its Mastery score of 0 allows (one per level).',
    );
  });

  it('states a single named parameter value correctly', () => {
    expect(
      say('issue-wrong_param_count', {
        item: 'Restricted Learning',
        key: 'Abilities',
        count: '1',
        expected: '5',
      }),
    ).toBe('Restricted Learning names 1 value(s) for Abilities, but exactly 5 are required.');
  });

  // LifeStagePanel.svelte passes every figure as a string. The points are 30 a
  // year less 10 per lab season, so never 1; the years and levels can be.
  it('states one year as a magus and one level of spells correctly', () => {
    expect(
      say('life-stage-post-gauntlet-summary', { years: '1', points: '30', xp: '29', levels: '1' }),
    ).toBe('Years as a magus: 1; 30 points = 29 XP + 1 level(s) of spells');
  });

  // DerivedSummarySection.svelte and CharacterDetails.svelte pass strings.
  it('states a single Warping Point correctly', () => {
    expect(say('warping-readout', { score: '0', points: '1' })).toBe('Score 0, Points 1');
  });

  it('states a single Confidence Point correctly', () => {
    expect(say('confidence-readout', { score: '1', points: '1' })).toBe('Score 1, Points 1');
  });
});

// The German twins of the EN messages above, which rendered „1 Werte“,
// „1 Alterungspunkte“ … the same way. Fixed count-neutrally, as in e66c0ce: the
// count moves out of the noun's way, since a string arg can never select `[one]`.
describe('German UI bundle, wording at a count of 1', () => {
  const de = buildBundle('de');
  const say = (key: string, args?: Record<string, string | number>) =>
    translate(de, key, args).replace(/[⁦-⁩]/g, '');

  it('states a one-point Decrepitude-and-Crisis outcome correctly', () => {
    expect(say('aging-outcome-decrepitude_and_crisis', { points: 1 })).toBe(
      'Alterungspunkte: 1 — genug für die nächste Stufe Gebrechlichkeit — und eine Krise.',
    );
  });

  it('states a single lab season without years as a magus correctly', () => {
    expect(say('issue-life_stage_lab_seasons_without_years', { seasons: '1' })).toBe(
      'Eingetragene Quartale Laborarbeit: 1; dieser Charakter hat aber keine Jahre als Magus, in denen sie stattfinden könnten.',
    );
  });

  it('states a one-level spell split beyond the points correctly', () => {
    expect(
      say('issue-life_stage_spell_level_split_exceeds_points', { levels: '1', points: '0' }),
    ).toBe(
      'Die aus den Jahren als Magus genommenen Zauberstufen (1) übersteigen, was diese Jahre gewähren: Sie sind 0 Punkte wert, die zwischen Erfahrung und Zauberstufen aufzuteilen sind.',
    );
  });

  it('states one banked experience point against one needed correctly', () => {
    expect(
      say('issue-banked_xp_at_or_above_next_level', { art: 'Creo', banked: '1', needed: '1' }),
    ).toBe(
      'Die angesparten Erfahrungspunkte (1) reichen bereits aus, um diesen Wert zu steigern — die nächste Stufe braucht nur 1.',
    );
  });

  it('states one Mastery special ability above a Mastery score of 0 correctly', () => {
    expect(
      say('issue-too_many_mastery_abilities', {
        spell: 'Pilum des Feuers',
        chosen: '1',
        mastery: '0',
      }),
    ).toBe(
      'Pilum des Feuers hat mehr Meisterschaftsfähigkeiten (1), als der Meisterschaftswert von 0 erlaubt (eine je Stufe).',
    );
  });

  it('states a single named parameter value correctly', () => {
    expect(
      say('issue-wrong_param_count', {
        item: 'Eingeschränktes Lernen',
        key: 'Fertigkeiten',
        count: '1',
        expected: '5',
      }),
    ).toBe(
      'Eingeschränktes Lernen: Für Fertigkeiten sind genau 5 Werte erforderlich, genannt sind 1.',
    );
  });

  it('states one year as a magus and one level of spells correctly', () => {
    expect(
      say('life-stage-post-gauntlet-summary', { years: '1', points: '30', xp: '29', levels: '1' }),
    ).toBe('Jahre als Magus: 1; 30 Punkte = 29 EP + Zauberstufen (1)');
  });

  it('states a single Warping Point correctly', () => {
    expect(say('warping-readout', { score: '0', points: '1' })).toBe('Wert 0, Punkte 1');
  });

  it('states a single Confidence Point correctly', () => {
    expect(say('confidence-readout', { score: '1', points: '1' })).toBe('Wert 1, Punkte 1');
  });
});

// FTL rules-claim audit (tmp/ftl-rules-audit.md, 2026-10-03): messages that
// claimed something the engine does not do. One case per key and locale, every
// arg a STRING as `derive.ts::resolveIssueArgs` passes it, pinned to the whole
// target sentence.
describe('UI bundles, rules-claim audit fixes', () => {
  const bundles = { en: buildBundle('en'), de: buildBundle('de') };
  const say = (lang: 'en' | 'de', key: string, args?: Record<string, string>) =>
    translate(bundles[lang], key, args).replace(/[⁦-⁩]/g, '');

  const cases: {
    finding: string;
    lang: 'en' | 'de';
    key: string;
    args?: Record<string, string>;
    want: string;
  }[] = [
    // F2: Great/Poor (Characteristic) never move the buy cap or floor
    // (`effective/characteristic.rs::characteristic_cap`, ArMDE:3987-3989); the
    // only shipped cap-lowerer is Uninspirational (ArMDE:6919-6921).
    {
      finding: 'F2',
      lang: 'en',
      key: 'issue-characteristic_above_cap',
      args: { characteristic: 'Presence', score: '1', cap: '0' },
      want: 'Characteristic Presence score 1 exceeds its maximum of 0.',
    },
    {
      finding: 'F2',
      lang: 'de',
      key: 'issue-characteristic_above_cap',
      args: { characteristic: 'Präsenz', score: '1', cap: '0' },
      want: 'Eigenschaft Präsenz mit Wert 1 überschreitet ihr Maximum von 0.',
    },
    {
      finding: 'F2',
      lang: 'en',
      key: 'issue-characteristic_below_floor',
      args: { characteristic: 'Presence', score: '-4', floor: '-3' },
      want: 'Characteristic Presence score -4 is below its minimum of -3.',
    },
    {
      finding: 'F2',
      lang: 'de',
      key: 'issue-characteristic_below_floor',
      args: { characteristic: 'Präsenz', score: '-4', floor: '-3' },
      want: 'Eigenschaft Präsenz mit Wert -4 liegt unter ihrem Minimum von -3.',
    },
    // F3: a magus has no free Gift slot (ArMDE:2874,
    // `effective/reputation_and_caps.rs::supernatural_free_slots`), and the
    // tooltip shows only when no slot is free.
    {
      finding: 'F3',
      lang: 'en',
      key: 'ability-requires-virtue',
      want: 'Requires a granting Virtue',
    },
    {
      finding: 'F3',
      lang: 'de',
      key: 'ability-requires-virtue',
      want: 'Erfordert eine verleihende Tugend',
    },
    {
      finding: 'F3',
      lang: 'en',
      key: 'issue-supernatural_ability_requires_virtue',
      args: { ability: 'Second Sight' },
      want: 'Second Sight is a Supernatural Ability and requires a granting Virtue.',
    },
    {
      finding: 'F3',
      lang: 'de',
      key: 'issue-supernatural_ability_requires_virtue',
      args: { ability: 'Zweites Gesicht' },
      want: 'Zweites Gesicht ist eine Übernatürliche Fertigkeit und erfordert eine verleihende Tugend.',
    },
    // F4: a Mythic Companion's Flaws fund twice their points (ArMDE:2844,
    // `validation/balance.rs::validate_balance` via `funded`), so the sentence
    // must not state 1:1. 7 Virtue / 3 Flaw fires at both rates.
    {
      finding: 'F4',
      lang: 'en',
      key: 'issue-unbalanced_virtues',
      args: { virtue_points: '7', flaw_points: '3' },
      want: 'Virtue points (7) exceed what your Flaw points (3) can fund.',
    },
    {
      finding: 'F4',
      lang: 'de',
      key: 'issue-unbalanced_virtues',
      args: { virtue_points: '7', flaw_points: '3' },
      want: 'Tugendpunkte (7) übersteigen, was deine Fehlerpunkte (3) finanzieren können.',
    },
    // F8: `spent` is the flow solve's whole demand, Abilities + Arts + Spell
    // Mastery (`validation/magus.rs::validate_xp_pool`, `effective/xp.rs::build_spends`).
    {
      finding: 'F8',
      lang: 'en',
      key: 'issue-not_enough_xp',
      args: { spent: '300', pool: '240', shortfall: '60' },
      want: 'Abilities, Arts and Spell Mastery need 300 XP in total, more than the 240 XP available for them.',
    },
    {
      finding: 'F8',
      lang: 'de',
      key: 'issue-not_enough_xp',
      args: { spent: '300', pool: '240', shortfall: '60' },
      want: 'Fertigkeiten, Künste und Zaubermeisterschaft brauchen insgesamt 300 EP, mehr als die 240 EP, die dafür verfügbar sind.',
    },
    // F9: duplicates are keyed on (Ability, instance parameter), never on the
    // specialty (`validation/scores.rs::validate_abilities`).
    {
      finding: 'F9',
      lang: 'en',
      key: 'issue-duplicate_ability',
      args: { ability: 'Brawl', count: '2' },
      want: 'Brawl is listed 2 times.',
    },
    {
      finding: 'F9',
      lang: 'de',
      key: 'issue-duplicate_ability',
      args: { ability: 'Raufen', count: '2' },
      want: 'Raufen ist 2-mal aufgeführt.',
    },
    // L1: the cap is the highest level a magus can LEARN (ArMDE:2465), worded as
    // `issue-spell_level_exceeds_cap` already words it. DE's neutral
    // „Zaubergrenze“ makes no casting claim and stays (pinned by S6 above).
    {
      finding: 'L1',
      lang: 'en',
      key: 'spell-cap-reason',
      args: { cap: '15' },
      want: 'Above the highest level you can learn (15)',
    },
    {
      finding: 'L1',
      lang: 'en',
      key: 'spell-cap-within-focus-reason',
      args: { cap: '15' },
      want: 'Above the highest level you can learn (15); fits within your Magical Focus',
    },
    // L2: the Flaw bars Technique+Form combinations (ArMDE:6292), and a spell
    // trips it by touching one (`effective/spell.rs::spell_touches_barred_combination`).
    {
      finding: 'L2',
      lang: 'en',
      key: 'issue-spell_uses_incompatible_arts',
      args: { spell: 'Pilum of Fire' },
      want: 'Pilum of Fire uses a Technique and Form combination that Incompatible Arts forbids.',
    },
    {
      finding: 'L2',
      lang: 'de',
      key: 'issue-spell_uses_incompatible_arts',
      args: { spell: 'Pilum des Feuers' },
      want: 'Pilum des Feuers verwendet eine Kombination aus Technik und Form, die der Fehler Unvereinbare Künste verbietet.',
    },
    // F1: a year holds four lab seasons, of which at most three are charged
    // (ArMDE:2482, `life_stage.rs::charged_lab_seasons`), so `max` counts the
    // seasons the years HOLD (`validation/life_stage.rs::validate_post_gauntlet_choices`),
    // not the seasons that can be charged.
    {
      finding: 'F1',
      lang: 'en',
      key: 'issue-life_stage_lab_seasons_out_of_range',
      args: { seasons: '200', max: '140', years: '35' },
      want: '200 lab seasons is more than the 140 seasons in 35 year(s) as a magus.',
    },
    {
      finding: 'F1',
      lang: 'de',
      key: 'issue-life_stage_lab_seasons_out_of_range',
      args: { seasons: '200', max: '140', years: '35' },
      want: '200 Quartale Laborarbeit sind mehr als die 140 Quartale in den Jahren als Magus (35).',
    },
  ];

  it.each(cases)('$finding: $lang/$key states what the engine does', (c) => {
    expect(say(c.lang, c.key, c.args)).toBe(c.want);
  });
});

// T1 (post-deadline round, 2026-10-03): wording the try-out and the
// after-deadline answers asked for.
describe('UI bundles, post-deadline wording fixes', () => {
  // Try-out finding 19 / ftl-audit S7: the Incompatible Arts picker labels each
  // parameter group "Combination N" / „Kombination N“ (`param-group-label`), so
  // the error about two identical groups must use the same word, or the player
  // cannot map it onto anything on screen. Read out of the label itself, so the
  // pairing holds whichever side is reworded later.
  it.each(['en', 'de'])(
    'calls a repeated parameter group a combination, as the picker does (%s)',
    (lang) => {
      const src = sourceForLang(lang);
      const word = messageValue(src, 'param-group-label')
        .replace(/\{\s*\$n\s*\}/, '')
        .trim()
        .toLowerCase();
      expect(word).toBe(lang === 'en' ? 'combination' : 'kombination');
      const message = messageValue(src, 'issue-param_groups_not_distinct').toLowerCase();
      expect(message).toContain(word);
      expect(message).not.toMatch(lang === 'en' ? /\bgroup/ : /gruppe/);
    },
  );

  // After-deadline answer 6: „Angelegt“ fits armour, but a weapon is „geführt“,
  // and the one loadout state covers both.
  it('names the wielded loadout for weapons and armour alike in German', () => {
    expect(translate(buildBundle('de'), 'equipment-loadout-wielded')).toBe('Geführt / Angelegt');
  });
});

// R3b (D83.3): the Potent Magic twins of D81.5's "add within focus" strings —
// the Spells picker's reasons, actions, accessible names and tooltips for a
// spell that fits only within the Potent Magic field, or only within the
// Magical Focus and the Potent Magic field together. Worded after the
// within-focus strings; DE term „Bereich der Potenten Magie“ as in
// `spell-within-potent-field-label`.
describe('UI bundles, Potent Magic picker actions (R3b)', () => {
  const bundles = { en: buildBundle('en'), de: buildBundle('de') };
  const say = (lang: 'en' | 'de', key: string, args?: Record<string, string>) =>
    translate(bundles[lang], key, args).replace(/[⁦-⁩]/g, '');

  const cases: {
    lang: 'en' | 'de';
    key: string;
    args?: Record<string, string>;
    want: string;
  }[] = [
    {
      lang: 'en',
      key: 'spell-cap-within-potent-field-reason',
      args: { cap: '15' },
      want: 'Above the highest level you can learn (15); fits within your Potent Magic field',
    },
    {
      lang: 'de',
      key: 'spell-cap-within-potent-field-reason',
      args: { cap: '15' },
      want: 'Über deiner Zaubergrenze (15); im Bereich deiner Potenten Magie erlaubt',
    },
    {
      lang: 'en',
      key: 'spell-cap-within-focus-and-potent-field-reason',
      args: { cap: '15' },
      want: 'Above the highest level you can learn (15); fits within your Magical Focus and Potent Magic field together',
    },
    {
      lang: 'de',
      key: 'spell-cap-within-focus-and-potent-field-reason',
      args: { cap: '15' },
      want: 'Über deiner Zaubergrenze (15); im Rahmen deines Magischen Fokus und im Bereich deiner Potenten Magie zusammen erlaubt',
    },
    {
      lang: 'en',
      key: 'spell-add-within-potent-field',
      want: 'Add within Potent Magic field',
    },
    {
      lang: 'de',
      key: 'spell-add-within-potent-field',
      want: 'Im Bereich der Potenten Magie hinzufügen',
    },
    {
      lang: 'en',
      key: 'spell-add-within-potent-field-label',
      args: { name: 'Pilum of Fire (CrIg 20)' },
      want: 'Add Pilum of Fire (CrIg 20) within Potent Magic field',
    },
    {
      lang: 'de',
      key: 'spell-add-within-potent-field-label',
      args: { name: 'Pilum des Feuers (CrIg 20)' },
      want: 'Pilum des Feuers (CrIg 20) im Bereich der Potenten Magie hinzufügen',
    },
    {
      lang: 'en',
      key: 'spell-add-within-potent-field-tooltip',
      args: { cap: '15' },
      want: 'Fits within your Potent Magic field (cap 15)',
    },
    {
      lang: 'de',
      key: 'spell-add-within-potent-field-tooltip',
      args: { cap: '15' },
      want: 'Im Bereich deiner Potenten Magie erlaubt (Grenze 15)',
    },
    {
      lang: 'en',
      key: 'spell-add-within-focus-and-potent-field',
      want: 'Add within focus and Potent Magic field',
    },
    {
      lang: 'de',
      key: 'spell-add-within-focus-and-potent-field',
      want: 'Im Fokus und im Bereich der Potenten Magie hinzufügen',
    },
    {
      lang: 'en',
      key: 'spell-add-within-focus-and-potent-field-label',
      args: { name: 'Pilum of Fire (CrIg 20)' },
      want: 'Add Pilum of Fire (CrIg 20) within focus and Potent Magic field',
    },
    {
      lang: 'de',
      key: 'spell-add-within-focus-and-potent-field-label',
      args: { name: 'Pilum des Feuers (CrIg 20)' },
      want: 'Pilum des Feuers (CrIg 20) im Fokus und im Bereich der Potenten Magie hinzufügen',
    },
    {
      lang: 'en',
      key: 'spell-add-within-focus-and-potent-field-tooltip',
      args: { cap: '15' },
      want: 'Fits within your Magical Focus and Potent Magic field together (cap 15)',
    },
    {
      lang: 'de',
      key: 'spell-add-within-focus-and-potent-field-tooltip',
      args: { cap: '15' },
      want: 'Im Rahmen deines Magischen Fokus und im Bereich deiner Potenten Magie zusammen erlaubt (Grenze 15)',
    },
  ];

  it.each(cases)('$lang/$key', (c) => {
    expect(say(c.lang, c.key, c.args)).toBe(c.want);
  });
});

// L3 (try-out finding 13): the CV4 load notices read "Latin were recognized"
// for a single item. `$items` is one pre-joined string (`derive.ts`), so no
// selector can see the count: the wording must read correctly for one item and
// for several. Pinned with ONE item, the case that broke, args as derive.ts
// composes them from the `-item` messages.
describe('UI bundles, catalogue load notices at a single item (L3)', () => {
  const bundles = { en: buildBundle('en'), de: buildBundle('de') };
  const say = (lang: 'en' | 'de', key: string, args?: Record<string, string>) =>
    translate(bundles[lang], key, args).replace(/[⁦-⁩]/g, '');

  const cases: {
    lang: 'en' | 'de';
    key: string;
    args: Record<string, string>;
    want: string;
  }[] = [
    {
      lang: 'en',
      key: 'migrated-catalogued-parameter-notice',
      args: { items: 'Dead Language: "Latin" → Latin' },
      want: 'Dead Language: "Latin" → Latin — recognized from what you typed and now linked to the catalogue.',
    },
    {
      lang: 'de',
      key: 'migrated-catalogued-parameter-notice',
      args: { items: 'Tote Sprache: „Latein“ → Latein' },
      want: 'Tote Sprache: „Latein“ → Latein — anhand des eingegebenen Textes erkannt und mit dem Katalog verknüpft.',
    },
    {
      lang: 'en',
      key: 'unresolved-catalogued-parameter-notice',
      args: { items: 'Living Language ("Gaelic")' },
      want: 'This character was saved with free text the rules catalogue does not recognize: Living Language ("Gaelic"). Kept exactly as typed. Check that anything relying on the typed text still works.',
    },
    {
      lang: 'de',
      key: 'unresolved-catalogued-parameter-notice',
      args: { items: 'Lebende Sprache („Gaelic“)' },
      want: 'Dieser Charakter wurde mit Freitext gespeichert, den der Regelkatalog nicht kennt: Lebende Sprache („Gaelic“). Genau wie eingegeben beibehalten. Prüfe, ob alles, was von diesem Text abhängt, noch funktioniert.',
    },
  ];

  it.each(cases)('$lang/$key reads correctly for one item', (c) => {
    expect(say(c.lang, c.key, c.args)).toBe(c.want);
  });
});
