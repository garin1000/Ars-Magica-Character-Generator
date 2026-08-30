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

/** The shipped checklist: three minimums of `:2437`, four recommendations of `:2451`. */
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
    // needs no is_magus test of its own.
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

  it('prices the recommended package from the rules data, never a literal', () => {
    setChecklist(shippedChecklist());
    const hint = element(html(), 'magus-recommended-hint');
    expect(clean(hint.text)).toContain('90');
    expect(clean(hint.text)).toContain('weak relative to other magi');
  });

  it('names each Ability through the rules i18n, with the bought instance filled in', () => {
    store.entity.ability_scores = [
      { ability: 'ability.dead_language', parameter: 'Latin', score: 4 },
    ] as Entity['ability_scores'];
    setChecklist(shippedChecklist({ 'ability.dead_language': 4 }));
    const body = html();
    const latin = element(body, 'magus-minimum-ability.dead_language');
    // The instance comes from the character's own row, so the checklist reads as the
    // Ability list does — never the id, never an unresolved template token.
    expect(clean(latin.text)).toContain('Latin (Dead Language)');
    // The id belongs in the testid and nowhere a player can read it.
    expect(clean(latin.text)).not.toContain('ability.dead_language');
    expect(body).not.toContain('{language}');
  });

  it('names the Ability without a doubled placeholder when nothing is bought yet', () => {
    // #13: with no instance held, the generic "(Language)" hint used to be stacked on
    // the template's own "(Dead Language)" literal, reading
    // "(Language) (Dead Language) 1 is not met". The entry's `name_unfilled` is the
    // opt-out; the token must still never appear.
    setChecklist(shippedChecklist());
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
    // Both groups, their headings and the price of the recommended package: nothing
    // is withheld from the expanded view, and nothing escapes the collapsed one.
    for (const testid of [
      'magus-minimum-ability.parma_magica',
      'magus-recommended-ability.parma_magica',
      'magus-recommended-hint',
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

describe('MagusMinimumAbilities and its validation message (slice 7, #13 + #32)', () => {
  /** The one `<li>` of a ValidationPanel showing the magus-minimum error. */
  function issueText(): string {
    store.result = {
      issues: [
        {
          severity: 'error',
          code: 'magus_minimum_ability',
          phase: 'abilities',
          // Exactly what the engine emits: the Ability id, the rules' exemplar slug,
          // and the two scores.
          args: { ability: 'ability.dead_language', exemplar: 'latin', min: '1', score: '0' },
        },
      ],
    } as unknown as NonNullable<typeof store.result>;
    const body = render(ValidationPanel, { props: { phase: 'abilities' } }).body;
    const match = /<li[^>]*data-code="magus_minimum_ability"[^>]*>([\s\S]*?)<\/li>/.exec(body);
    if (!match) throw new Error('no <li> for magus_minimum_ability');
    return clean(match[1].replace(/<[^>]*>/g, '')).trim();
  }

  it('names the rules exemplar beside the widened requirement, in both surfaces', () => {
    // `:2437` says "Latin 1"; the engine can only enforce "any Dead Language 1", so
    // the exemplar is shown as a label. Shared label path -> both surfaces agree.
    setChecklist(shippedChecklist());
    const rowText = clean(element(html(), 'magus-minimum-ability.dead_language').text);
    expect(rowText).toContain('Dead Language (e.g. Latin)');
    expect(issueText()).toContain('Dead Language (e.g. Latin)');
    // Never the slug, in either surface.
    expect(rowText).not.toContain('latin"');
    expect(issueText()).not.toContain('exemplar.latin');
  });

  it('reads the requirement identically in the row and the message', () => {
    setChecklist(shippedChecklist());
    const rowText = clean(element(html(), 'magus-minimum-ability.dead_language').text);
    // One shared label path, so the phrase naming the requirement is byte-identical.
    const phrase = 'Dead Language (e.g. Latin) 1';
    expect(rowText.startsWith(phrase)).toBe(true);
    expect(issueText()).toContain(phrase);
  });

  it('names the exemplar in German too', () => {
    store.lang = 'de';
    store.ruleset!.i18n['exemplar.latin'] = { name: 'Latein' };
    store.ruleset!.i18n['ability.dead_language'] = {
      name: '{language} (Tote Sprache)',
      name_unfilled: 'Tote Sprache',
    };
    setChecklist(shippedChecklist());
    const rowText = clean(element(html(), 'magus-minimum-ability.dead_language').text);
    expect(rowText).toContain('Tote Sprache (z. B. Latein)');
    expect(issueText()).toContain('Tote Sprache (z. B. Latein)');
  });
});
