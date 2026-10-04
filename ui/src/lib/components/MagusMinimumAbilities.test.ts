import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  Ability,
  EffectiveScores,
  Entity,
  LifeStageRules,
  LocalizedRuleset,
  MagusMinimumAbility,
} from '../types';

// The checklist reads the shared store singleton (the engine's checklist rows, the
// entity's bought Ability instances, the ruleset's Ability names) and the Fluent
// bundle. The store schedules a debounced revalidate over the Tauri IPC bridge; mock
// the bridge so nothing reaches a backend. Harness mirrors XpBar.test.ts.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import MagusMinimumAbilities from './MagusMinimumAbilities.svelte';
import ValidationPanel from './ValidationPanel.svelte';

/** The apprenticeship block, so the recommended package can be priced from data. */
function lifeStageRules(): LifeStageRules {
  return {
    apprenticeship: {
      minimum_abilities: [],
      recommended_abilities: [],
      recommended_xp: 90,
      xp: 240,
      years: 15,
      truncated_xp_per_year: 16,
      truncated_spell_levels_per_year: 8,
    },
    childhood: {
      years: 5,
      native_language_ability: 'ability.living_language',
      native_language_xp: 75,
      spread_xp: 45,
      spread_abilities: ['ability.athletics'],
    },
    later_life: { xp_per_year: 15 },
  };
}

/**
 * Install a minimal localized ruleset carrying the three Abilities the checklist
 * names, one of them parameterized — the case whose name is a template.
 */
function installRuleset(): void {
  const abilities: Record<string, Ability> = {
    'ability.dead_language': {
      id: 'ability.dead_language',
      category: 'academic',
      parameter: 'language',
    } as unknown as Ability,
    'ability.magic_theory': { id: 'ability.magic_theory', category: 'arcane' },
    'ability.parma_magica': { id: 'ability.parma_magica', category: 'arcane' },
  };
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities,
      life_stages: lifeStageRules(),
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      // The one shape whose hint substitution doubles: a `{token}` plus a
      // parenthetical literal. `name_unfilled` is its opt-out (#13, option d).
      'ability.dead_language': {
        name: '{language} (Dead Language)',
        name_unfilled: 'Dead Language',
      },
      'ability.magic_theory': { name: 'Magic Theory' },
      'ability.parma_magica': { name: 'Parma Magica' },
      // The rules' own exemplar for the widened dead-language check (#32).
      'exemplar.latin': { name: 'Latin' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
}

function row(
  ability: string,
  min_score: number,
  score: number,
  requirement: MagusMinimumAbility['requirement'],
  exemplar?: string,
): MagusMinimumAbility {
  return { ability, min_score, score, met: score >= min_score, requirement, exemplar };
}

/** The shipped checklist: three minimums of `ArMDE:2437`, four recommendations of `ArMDE:2451`. */
function shippedChecklist(scores: Record<string, number> = {}): MagusMinimumAbility[] {
  const at = (ability: string) => scores[ability] ?? 0;
  return [
    row('ability.dead_language', 1, at('ability.dead_language'), 'required', 'latin'),
    row('ability.magic_theory', 1, at('ability.magic_theory'), 'required'),
    row('ability.parma_magica', 1, at('ability.parma_magica'), 'required'),
    row('ability.dead_language', 4, at('ability.dead_language'), 'recommended', 'latin'),
    row('ability.magic_theory', 3, at('ability.magic_theory'), 'recommended'),
    row('ability.parma_magica', 1, at('ability.parma_magica'), 'recommended'),
  ];
}

function setChecklist(rows: MagusMinimumAbility[]): void {
  store.effective = { magus_minimum_abilities: rows } as unknown as EffectiveScores;
}

/** Render the checklist to an HTML string (node env, no DOM). */
function html(): string {
  return render(MagusMinimumAbilities, { props: {} }).body;
}

/** The single element carrying a given data-testid, with its class attribute. */
function element(body: string, testid: string): { open: string; text: string } {
  const re = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i');
  const match = re.exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  const openMatch = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  return { open: openMatch![0], text: match[1].replace(/<[^>]*>/g, '').trim() };
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

/** Fluent isolates interpolated values with bidi marks; strip them for text matching. */
function clean(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('MagusMinimumAbilities without a checklist (slice 6b4)', () => {
  it('renders nothing when the engine sends an empty list', () => {
    // Empty for every type but a magus, so absence is the whole gate — the component
    // needs no hermetically_trained test of its own.
    setChecklist([]);
    const body = html();
    expect(has(body, 'magus-minimums')).toBe(false);
    expect(body.replace(/<!--[\s\S]*?-->/g, '').trim()).toBe('');
  });

  it('renders nothing before the effective scores arrive', () => {
    store.effective = null;
    expect(has(html(), 'magus-minimums')).toBe(false);
  });
});

describe('MagusMinimumAbilities checklist (slice 6b4)', () => {
  it('renders the demanded group and the recommended group under their own headings', () => {
    setChecklist(shippedChecklist());
    const body = html();
    expect(has(body, 'magus-minimums')).toBe(true);
    // h3, not h2: the region titles ("Available", "Selected") own the h2 level.
    const headings = [...body.matchAll(/<h3[^>]*>([\s\S]*?)<\/h3>/g)].map((m) =>
      m[1].replace(/<[^>]*>/g, '').trim(),
    );
    expect(headings).toContain('Minimum Abilities');
    expect(headings).toContain('Recommended minimum Abilities');
    expect(body).not.toMatch(/<h2/i);
    // Both groups render their rows; Parma is demanded by BOTH lists, so it
    // legitimately appears once per group.
    expect(has(body, 'magus-minimum-ability.parma_magica')).toBe(true);
    expect(has(body, 'magus-recommended-ability.parma_magica')).toBe(true);
  });

  it('states each row met or unmet in words, mirrored by data-met', () => {
    setChecklist(shippedChecklist({ 'ability.magic_theory': 1 }));
    const body = html();
    const unmet = element(body, 'magus-minimum-ability.parma_magica');
    // Never colour or an icon alone: the status is a sentence a screen reader reads.
    expect(clean(unmet.text)).toContain('is not met');
    expect(unmet.open).toMatch(/data-met="false"/);
    const met = element(body, 'magus-minimum-ability.magic_theory');
    expect(clean(met.text)).toContain('is met');
    expect(met.open).toMatch(/data-met="true"/);
    // The same score falls short of the recommended threshold of 3, and says so.
    expect(clean(element(body, 'magus-recommended-ability.magic_theory').text)).toContain(
      'is not met',
    );
  });

  it('summarizes the outstanding rows once, in a single live region', () => {
    setChecklist(shippedChecklist({ 'ability.magic_theory': 1 }));
    const body = html();
    const summary = element(body, 'magus-minimums-summary');
    expect(summary.open).toMatch(/role="status"/);
    // One row of the six is met, so five are outstanding.
    expect(clean(summary.text)).toContain('5');
    expect(clean(summary.text)).toContain('6');
    // ONE live region: a row-level one would announce six sentences on every keystroke.
    expect([...body.matchAll(/role="status"/g)]).toHaveLength(1);
  });

  // manual-testing-findings #21: the "the recommended Abilities cost 90 xp, below
  // them the magus is weak" sentence is gone. The rows themselves are the content;
  // the price of the package was rules teaching, and the player has the book.
  it('states no price for the recommended package', () => {
    setChecklist(shippedChecklist());
    expect(has(html(), 'magus-recommended-hint')).toBe(false);
  });

  it('names each Ability through the rules i18n, with the bought instance filled in', () => {
    store.entity.ability_scores = [
      { ability: 'ability.dead_language', parameter: { text: 'Latin' }, score: 4 },
    ] as Entity['ability_scores'];
    // A requirement naming NO exemplar, which is the case the instance labels: where
    // the rules do name one it heads the row instead (see the exemplar suite below).
    setChecklist([row('ability.dead_language', 1, 4, 'required')]);
    const body = html();
    const latin = element(body, 'magus-minimum-ability.dead_language');
    // The instance comes from the character's own row, so the checklist reads as the
    // Ability list does — never the id, never an unresolved template token.
    expect(clean(latin.text)).toContain('Latin (Dead Language)');
    // The id belongs in the testid and nowhere a player can read it.
    expect(clean(latin.text)).not.toContain('ability.dead_language');
    expect(body).not.toContain('{language}');
  });

  // CV8 (design-cv-catalogued-values.md § 6.4): `instanceOf` must resolve a
  // CATALOGUED bought instance the same way it already resolves `Text`
  // (the test above) — through `abilityParamDisplay`, never the raw id.
  it('names each Ability through the rules i18n for a CATALOGUED bought instance too', () => {
    // Deliberately the GERMAN name for the catalogue id, regardless of
    // `store.lang` — proves the row reads the merged `ruleset.i18n`, not a
    // structural "humanize the id" guess (which would read the
    // English-shaped "Latin" no matter what `store.lang` is). Matches
    // `AbilityTab.test.ts`'s CV7 precedent.
    store.ruleset!.i18n['language.latin'] = { name: 'Latein' };
    store.entity.ability_scores = [
      { ability: 'ability.dead_language', parameter: { id: 'language.latin' }, score: 4 },
    ] as Entity['ability_scores'];
    setChecklist([row('ability.dead_language', 1, 4, 'required')]);
    const body = html();
    const latin = element(body, 'magus-minimum-ability.dead_language');
    expect(clean(latin.text)).toContain('Latein (Dead Language)');
    expect(clean(latin.text)).not.toContain('language.latin');
  });

  // L2: a requirement may name its instance as a catalogue value id. With no exemplar
  // to head the row, the instance is named — by its localized name, never the id.
  it('names a requirement instance given as a catalogue id by its localized name', () => {
    store.ruleset!.i18n['language.latin'] = { name: 'Latin' };
    setChecklist([
      { ...row('ability.dead_language', 1, 0, 'required'), parameter: 'language.latin' },
    ]);
    const text = clean(element(html(), 'magus-minimum-ability.dead_language').text);
    expect(text).toContain('Latin (Dead Language) 1');
    expect(text).not.toContain('language.latin');
  });

  it('names the Ability without a doubled placeholder when nothing is bought yet', () => {
    // #13: with no instance held, the generic "(Language)" hint used to be stacked on
    // the template's own "(Dead Language)" literal, reading
    // "(Language) (Dead Language) 1 is not met". The entry's `name_unfilled` is the
    // opt-out; the token must still never appear.
    setChecklist([row('ability.dead_language', 1, 0, 'required')]);
    const latin = element(html(), 'magus-minimum-ability.dead_language');
    expect(clean(latin.text)).toContain('Dead Language');
    expect(clean(latin.text)).not.toContain('(Language)');
    expect(clean(latin.text)).not.toContain('{language}');
  });
});

// guided-creation-review-2026-08 #12 (DECIDED): the checklist duplicates the
// Validation panel — its own comment says the rows come from the same engine
// findings — so it collapses to the summary line, expandable on demand. Validation
// stays the authoritative surface.
describe('MagusMinimumAbilities collapsed to a summary (slice 11, #12)', () => {
  /** The `<details>…</details>` slice of the rendered body. */
  function disclosure(body: string): string {
    const match = /<details[^>]*data-testid="magus-minimums"[\s\S]*?<\/details>/.exec(body);
    if (!match) throw new Error('the checklist is not inside a <details> disclosure');
    return match[0];
  }

  it('renders only the summary line by default', () => {
    setChecklist(shippedChecklist({ 'ability.magic_theory': 1 }));
    const body = html();
    // A native disclosure: the summary is its own control, so keyboard and screen
    // reader support come from the platform rather than from an aria-expanded of
    // our own.
    const open = /<details[^>]*data-testid="magus-minimums"[^>]*>/.exec(body);
    expect(open).not.toBeNull();
    // Closed by default — that IS the collapse. `open` would ship the old surface
    // under a new element.
    expect(open![0]).not.toMatch(/\sopen[\s>=]/);
    // The count still announces once, from the always-visible summary.
    expect(element(body, 'magus-minimums-summary').open).toMatch(/role="status"/);
    expect([...body.matchAll(/role="status"/g)]).toHaveLength(1);
  });

  it('keeps the whole checklist inside the disclosure, so opening it reveals everything', () => {
    setChecklist(shippedChecklist());
    const inside = disclosure(html());
    // Both groups and their headings: nothing is withheld from the expanded view,
    // and nothing escapes the collapsed one.
    for (const testid of [
      'magus-minimum-ability.parma_magica',
      'magus-recommended-ability.parma_magica',
    ]) {
      expect(has(inside, testid), `${testid} is outside the disclosure`).toBe(true);
    }
    expect(inside).toContain('Minimum Abilities');
    expect(inside).toContain('Recommended minimum Abilities');
  });

  // THE COUNTING BUG. The summary said "N of 7" while sitting under the *Minimum
  // Abilities* heading, above a list of three: it counted every row but headed only
  // the required ones. The fix is positional — the summary becomes the disclosure's
  // own label, heading the whole checklist that its total actually counts.
  it('the summary counts only the rows it heads', () => {
    setChecklist(shippedChecklist({ 'ability.magic_theory': 1 }));
    const body = html();
    const summaryAt = body.indexOf('data-testid="magus-minimums-summary"');
    const minimumsHeadingAt = body.indexOf('Minimum Abilities');
    expect(summaryAt).toBeGreaterThanOrEqual(0);
    // It must not sit inside the scope of the "Minimum Abilities" heading, or its
    // total reads as that heading's list.
    expect(summaryAt).toBeLessThan(minimumsHeadingAt);

    // And the total equals the number of rows the disclosure holds.
    const inside = disclosure(body);
    const rows = [...inside.matchAll(/data-testid="magus-(?:minimum|recommended)-ability\./g)];
    const total = clean(element(body, 'magus-minimums-summary').text).match(/\d+/g) ?? [];
    expect(total).toHaveLength(2);
    expect(Number(total[1])).toBe(rows.length);
    // One of the six is met, so five are outstanding.
    expect(Number(total[0])).toBe(5);
  });
});

// N5 (try-out 2026-10-04): the demanded and the recommended blocks side by side —
// "|Minimum Abilities|Recommended Minimum Abilities|". The grid is app.css's
// (`.magus-minimums-columns`, pinned in app.css.test.ts); this pins the markup it
// lays out: one wrapper inside the disclosure, one column per block.
describe('MagusMinimumAbilities side-by-side columns (N5)', () => {
  const WRAPPER = '<div class="magus-minimums-columns">';

  /** Each column's markup, in document order: its heading and its list. */
  function columns(body: string): string[] {
    return [...body.matchAll(/<div class="magus-minimums-column">([\s\S]*?)<\/ul>/g)].map(
      (m) => m[1],
    );
  }

  /** The visible text of a column's one heading. */
  function headingOf(column: string): string {
    const match = /<h3[^>]*>([\s\S]*?)<\/h3>/.exec(column);
    return match ? match[1].replace(/<[^>]*>/g, '').trim() : '';
  }

  /** The row testids a column holds, in order. */
  function rowsOf(column: string): string[] {
    return [...column.matchAll(/data-testid="(magus-(?:minimum|recommended)-[^"]+)"/g)].map(
      (m) => m[1],
    );
  }

  it('wraps both blocks in one wrapper right after the summary, which stays a direct child', () => {
    setChecklist(shippedChecklist());
    const body = html();
    const details = /<details[^>]*data-testid="magus-minimums"[^>]*>([\s\S]*)<\/details>/.exec(
      body,
    );
    expect(details).not.toBeNull();
    const inside = details![1];
    // The `<summary>` opens the disclosure — nothing but whitespace or a hydration
    // marker before it — or it is no longer the disclosure's control.
    expect(inside.replace(/<!--[\s\S]*?-->/g, '').trimStart()).toMatch(/^<summary/);
    const summaryEnd = inside.indexOf('</summary>');
    const wrapperAt = inside.indexOf(WRAPPER);
    expect(wrapperAt, 'the columns wrapper is missing').toBeGreaterThan(summaryEnd);
    // Directly after the summary: the wrapper is the disclosure's second child.
    const between = inside.slice(summaryEnd + '</summary>'.length, wrapperAt);
    expect(between.replace(/<!--[\s\S]*?-->/g, '').trim()).toBe('');
    // Every heading and every row sits inside the wrapper, none beside it.
    const outside = inside.slice(0, wrapperAt);
    expect(outside).not.toMatch(/<h3|<ul|data-testid="magus-(?:minimum|recommended)-/);
  });

  it('gives each block its own column, demanded first, rows in their own order', () => {
    setChecklist(shippedChecklist());
    const [required, recommended, ...rest] = columns(html());
    expect(rest).toHaveLength(0);
    expect(headingOf(required ?? '')).toBe('Minimum Abilities');
    expect(headingOf(recommended ?? '')).toBe('Recommended minimum Abilities');
    expect(rowsOf(required ?? '')).toEqual([
      'magus-minimum-ability.dead_language',
      'magus-minimum-ability.magic_theory',
      'magus-minimum-ability.parma_magica',
    ]);
    expect(rowsOf(recommended ?? '')).toEqual([
      'magus-recommended-ability.dead_language',
      'magus-recommended-ability.magic_theory',
      'magus-recommended-ability.parma_magica',
    ]);
  });

  it('renders a lone demanded block as the only column, with no empty recommended one', () => {
    setChecklist(shippedChecklist().filter((r) => r.requirement === 'required'));
    const body = html();
    const all = columns(body);
    expect(all).toHaveLength(1);
    expect(headingOf(all[0])).toBe('Minimum Abilities');
    expect(body).not.toContain('Recommended minimum Abilities');
  });
});

describe('MagusMinimumAbilities and its validation message (slice 7, #13 + #32)', () => {
  /** The one `<li>` of a ValidationPanel showing a finding about a dead language. */
  function findingText(
    code: string,
    // Exactly what the engine emits: the Ability id, the rules' exemplar slug, and
    // whichever scores the finding carries.
    args: Record<string, string> = {
      ability: 'ability.dead_language',
      exemplar: 'latin',
      min: '1',
      score: '0',
    },
  ): string {
    store.result = {
      issues: [{ severity: 'error', code, phase: 'abilities', args }],
    } as unknown as NonNullable<typeof store.result>;
    const body = render(ValidationPanel, { props: { phase: 'abilities' } }).body;
    const match = new RegExp(`<li[^>]*data-code="${code}"[^>]*>([\\s\\S]*?)</li>`).exec(body);
    if (!match) throw new Error(`no <li> for ${code}`);
    return clean(match[1].replace(/<[^>]*>/g, '')).trim();
  }

  /** The magus-minimum error, the finding this suite is mostly about. */
  function issueText(): string {
    return findingText('magus_minimum_ability');
  }

  /** Install the German rules text the exemplar rows need, and switch the bundle. */
  function speakGerman(): void {
    store.lang = 'de';
    store.ruleset!.i18n['exemplar.latin'] = { name: 'Latein' };
    store.ruleset!.i18n['ability.dead_language'] = {
      name: '{language} (Tote Sprache)',
      name_unfilled: 'Tote Sprache',
    };
  }

  // guided-creation-review-2026-08 #12: the exemplar used to sit BETWEEN the Ability
  // and its score — "below Dead Language (e.g. Latin) 1" — so the sentence read as
  // though "e.g. Latin" were the thing being scored, and the demand the rules
  // actually make ("Latin 1", `ArMDE:2437`) was buried. The example now heads the
  // requirement with the score right after it, and the Ability it is bought as
  // trails as one short note. L2 (try-out finding 8): the engine now checks Latin
  // itself, so the note names the Ability and no longer claims "any Dead Language".
  it('states the demand as "Latin 1", with its Ability trailing it', () => {
    setChecklist(shippedChecklist());
    const rowText = clean(element(html(), 'magus-minimum-ability.dead_language').text);
    expect(rowText).toBe('Latin 1 (Dead Language) is not met: this character has 0.');
    // `toContain`, not `toBe`: the panel prefixes each finding with its severity.
    expect(issueText()).toContain(
      'No magus is admitted to the Order below Latin 1 (Dead Language); this character has 0.',
    );
    // The example never separates the requirement from its score again.
    for (const text of [rowText, issueText()]) {
      expect(text).toContain('Latin 1');
      expect(text).not.toContain('e.g.');
      // Never a slug, in either surface.
      expect(text).not.toContain('ability.dead_language');
      expect(text).not.toContain('exemplar.latin');
    }
  });

  it('states the recommended demand the same way', () => {
    setChecklist(shippedChecklist());
    const rowText = clean(element(html(), 'magus-recommended-ability.dead_language').text);
    expect(rowText).toBe('Latin 4 (Dead Language) is not met: this character has 0.');
  });

  // The warning twin folds the same `exemplar` arg through the same path. The
  // Academic-Ability warning (L2, ruling F5) names every language that satisfies
  // ArMDE:7151 instead — Latin or Hebrew as a Dead Language, Greek or Arabic as a
  // Living Language — from the engine's `languages` arg.
  it('states the sibling findings the same way', () => {
    expect(
      findingText('magus_recommended_ability', {
        ability: 'ability.dead_language',
        exemplar: 'latin',
        min: '4',
        score: '0',
      }),
    ).toContain(
      'Latin 4 (Dead Language) is recommended for a magus just out of apprenticeship; this character has 0.',
    );
    store.ruleset!.i18n['language.latin'] = { name: 'Latin' };
    store.ruleset!.i18n['language.hebrew'] = { name: 'Hebrew' };
    store.ruleset!.i18n['language.greek'] = { name: 'Greek' };
    store.ruleset!.i18n['language.arabic'] = { name: 'Arabic' };
    expect(
      findingText('academic_ability_without_scholarly_language', {
        languages: 'language.latin, language.hebrew, language.greek, language.arabic',
        min: '3',
      }),
    ).toContain(
      'An Academic Ability normally requires Latin, Hebrew, Greek or Arabic at 3 or better.',
    );
  });

  it('reads the requirement identically in the row and the message', () => {
    setChecklist(shippedChecklist());
    const rowText = clean(element(html(), 'magus-minimum-ability.dead_language').text);
    // One shared label path, so the phrase naming the requirement is byte-identical.
    const phrase = 'Latin 1 (Dead Language)';
    expect(rowText.startsWith(phrase)).toBe(true);
    expect(issueText()).toContain(phrase);
  });

  it('states the demand as "Latein 1" in German too', () => {
    speakGerman();
    setChecklist(shippedChecklist());
    const rowText = clean(element(html(), 'magus-minimum-ability.dead_language').text);
    expect(rowText).toBe('Latein 1 (Tote Sprache) ist nicht erfüllt: dieser Charakter hat 0.');
    expect(issueText()).toContain(
      'Kein Magus wird unter Latein 1 (Tote Sprache) in den Orden aufgenommen; dieser Charakter hat 0.',
    );
    for (const text of [rowText, issueText()]) {
      expect(text).toContain('Latein 1');
      expect(text).not.toContain('z. B.');
      expect(text).not.toContain('ability.dead_language');
      expect(text).not.toContain('exemplar.latin');
    }
  });
});
