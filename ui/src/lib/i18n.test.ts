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
    // is the trailing "any Dead Language" note, empty for a requirement that names
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
  it('calls an Ability a Fertigkeit wherever it names the game term', () => {
    const de = buildBundle('de');
    for (const key of ['ability-requires-virtue', 'issue-supernatural_ability_requires_virtue']) {
      const message = translate(de, key, { ability: 'Zweites Gesicht' });
      expect(message, `${key} still says Fähigkeit`).not.toContain('Fähigkeit');
      expect(message, `${key} does not say Fertigkeit`).toContain('Fertigkeit');
    }
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
});
