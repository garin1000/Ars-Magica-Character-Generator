import { describe, expect, it } from 'vitest';

import { buildBundle, translate } from './i18n';

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
    // (Basisregeln.md:2433, :2449 — Lehrlingszeit, Lehrlingsprüfung). The chip that
    // carried this term is now `xp-pool-block-apprenticeship` (#14), asserted below.
    expect(translate(de, 'life-stage-gauntlet-note')).toContain('Lehrlingsprüfung');
    // The two checklist headings are the rulebook's own (Basisregeln.md:2437, :2451).
    expect(translate(de, 'magus-minimums-label')).toBe('Mindestfertigkeiten');
    expect(translate(de, 'magus-recommended-label')).toBe('Empfohlene Mindestfertigkeiten');
    // Fertigkeiten, never "Fähigkeiten": the glossary's word for an Ability.
    expect(translate(de, 'magus-minimums-summary', { unmet: '3', total: '7' })).not.toContain(
      'Fähigkeiten',
    );
    expect(translate(de, 'magus-recommended-hint', { xp: '90' })).toContain('Erfahrungspunkte');
    // Whole sentences per status, so neither row leans on colour alone.
    const row = { ability: 'Parma Magica', min: '1', score: '0' };
    expect(translate(de, 'magus-minimum-met', row)).toContain('erfüllt');
    expect(translate(de, 'magus-minimum-unmet', row)).toContain('nicht erfüllt');
  });

  // Slice 2 (#1, #11): the read-only `type` step is gone and the `experience` step
  // took the funding choice off the Abilities step. The Rust side already asserts
  // `phase-<slug>`/`wizard-guidance-<slug>` exist for every `CreationPhase`; this
  // pins the other half — that the removed phase's keys left with it, so no key
  // names a phase the engine no longer has.
  it('keys the experience phase and no longer keys the removed type phase', () => {
    for (const lang of ['en', 'de']) {
      const keys = messageKeys(sourceForLang(lang));
      expect(keys).toContain('phase-experience');
      expect(keys).toContain('wizard-guidance-experience');
      expect(keys).not.toContain('phase-type');
      expect(keys).not.toContain('wizard-guidance-type');
      // #1's two surviving facts were re-keyed to their new home, never deleted.
      expect([...keys].filter((key) => key.startsWith('phase-type-'))).toEqual([]);
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
    // The prose keeps "as a magus"/"als Magus": it is a description, not the label.
    expect(translate(en, 'life-stage-lab-seasons-hint')).toContain('as a magus');
    expect(translate(de, 'life-stage-lab-seasons-hint')).toContain('als Magus');
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
