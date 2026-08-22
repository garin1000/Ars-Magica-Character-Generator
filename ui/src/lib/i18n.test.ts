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
    // (Basisregeln.md:2433, :2449 — Lehrlingszeit, Lehrlingsprüfung).
    expect(translate(de, 'life-stage-apprenticeship', { years: '15', xp: '240' })).toContain(
      'Lehrlingszeit',
    );
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
