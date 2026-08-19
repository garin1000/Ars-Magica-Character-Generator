import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { AgingOutcome, AgingTotal, CrisisPreview } from '../ipc';
import type {
  AgingReadout,
  AgingScheduleYear,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
} from '../types';

// The calculator is a read-out over UI-only draft state plus the engine's own
// preview: the schedule off `store.effective.aging`, the typed die off
// `store.agingDraft`, and the total/outcome off `store.agingPreview`. Mock the
// Tauri bridge so nothing reaches a backend; harness mirrors
// AgingSchedulePanel.test.ts.
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
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
}));

import { SCHEMA_VERSION, defaultAgingDraft, store, type AgingDraft } from '../state.svelte';
import AgingRollCalculator from './AgingRollCalculator.svelte';

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
}

/**
 * A minimal localized ruleset, so the crisis panel can resolve the two ids it
 * shows — the Crisis Table row and the attendant's Ability — through the rules
 * i18n rather than printing a slug. The Crisis die's own bounds ride along,
 * because "a zero counts as ten" (`:474`) is data, not a literal in the component.
 */
function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      aging: {
        start_age: 35,
        age_divisor: 10,
        apparent_age_increase_min: 3,
        living_conditions: [],
        outcomes: [],
        crisis: { rows: [], die: { min: 1, max: 10 } },
      },
    },
    i18n: {
      'crisis.minor_illness': { name: 'Minor illness' },
      'crisis.bedridden_week': { name: 'Bedridden for a week' },
      'crisis.terminal_illness': { name: 'Terminal illness' },
      'ability.medicine': { name: 'Medicine' },
      'virtue.mild_aging': { name: 'Mild Aging' },
    },
  } as unknown as LocalizedRuleset;
}

/** One Crisis as the engine reads it, every field the panel shows present. */
function crisis(over: Partial<CrisisPreview> = {}): CrisisPreview {
  return {
    total: { age: 40, die: 10, age_modifier: 4, decrepitude_score: 1, total: 15 },
    row: 'crisis.minor_illness',
    outcome: { type: 'illness', severity: 'minor', ease_factor: 3, ritual_level: 20 },
    survival: {
      ease_factor: 3,
      ritual_level: 20,
      modifiers: [],
      modifier_total: 0,
      allowances: [],
    },
    ...over,
  };
}

/** A crisis outcome that sends the year to the Crisis Table. */
function crisisOutcome(): AgingOutcome {
  return outcome({
    total: 13,
    awards: [{ target: { kind: 'next_decrepitude_level' }, points: 5 }],
    crisis: true,
  });
}

/** A schedule of owed years, `recorded` for the ages listed. */
function schedule(ages: number[], recorded: number[] = []): AgingScheduleYear[] {
  return ages.map((age) => ({ age, year: null, recorded: recorded.includes(age) }));
}

function setSchedule(years: AgingScheduleYear[]): void {
  const aging: AgingReadout = {
    first_roll_age: 36,
    begins_after_age: 35,
    schedule: years,
    rolls_owed: years.length,
    rolls_recorded: years.filter((y) => y.recorded).length,
    age_modifier: 4,
    living_conditions_modifier: 0,
    longevity_modifier: 0,
    longevity_clamp_active: false,
    fixed_total: 4,
  };
  store.effective = { ...(store.effective ?? {}), aging } as unknown as EffectiveScores;
}

/** An AGING TOTAL as the engine reports it, every term present. */
function total(overrides: Partial<AgingTotal> = {}): AgingTotal {
  return {
    age: 40,
    die: 10,
    age_modifier: 4,
    living_conditions: { rows: [], from_table: 0, from_traits: 0, total: 0 },
    longevity_bonus: 0,
    trait_modifier: 0,
    uncapped_total: 14,
    total: 14,
    capped_by_longevity: false,
    ...overrides,
  };
}

function outcome(overrides: Partial<AgingOutcome> = {}): AgingOutcome {
  return {
    total: 14,
    apparent_age_increases: true,
    awards: [],
    crisis: false,
    ...overrides,
  };
}

/** A drafted roll: the year, the stress die, the placed points and the Simple Die. */
function draft(over: Partial<AgingDraft> = {}): AgingDraft {
  return { age: 40, die: 10, distribution: {}, crisisDie: null, ...over };
}

/** Put the engine's answer on the store, as the debounced preview would. */
function setPreview(t: AgingTotal, o: AgingOutcome, c: CrisisPreview | null = null): void {
  store.agingPreview = { total: t, outcome: o, crisis: c };
}

/** Render the calculator to an HTML string (node env, no DOM). */
function html(): string {
  return render(AgingRollCalculator, { props: {} }).body;
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

/** The opening tag of the single element carrying a data-testid. */
function open(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[0];
}

/**
 * The visible text of the element carrying a data-testid, tags stripped. Closes on
 * the element's own tag name, so a block holding several paragraphs (the outcome)
 * is read whole rather than truncated at its first child.
 */
function text(body: string, testid: string): string {
  const opening = new RegExp(`<([a-z]+)[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!opening) throw new Error(`no element with data-testid="${testid}"`);
  const inner = body.slice(opening.index + opening[0].length);
  const closing = new RegExp(`</${opening[1]}>`, 'i').exec(inner);
  return stripMarks((closing ? inner.slice(0, closing.index) : inner).replace(/<[^>]*>/g, ' '))
    .replace(/\s+/g, ' ')
    .trim();
}

/** Everything the user can actually read: all markup stripped, isolation marks too. */
function visibleText(body: string): string {
  return stripMarks(body.replace(/<[^>]*>/g, ' '));
}

function stripMarks(s: string): string {
  return s.replace(/[⁨⁩]/g, '');
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  resetEntity();
  installRuleset();
  store.agingDraft = defaultAgingDraft();
  store.agingPreview = null;
  store.agingRejections = [];
  store.agingNotes = [];
  setSchedule(schedule([36, 37, 38, 39, 40]));
});

describe('AgingRollCalculator (slice 6b6c)', () => {
  it('shows nothing until a die is typed', () => {
    const body = html();
    // The two inputs are always there — the year to roll for and the die itself.
    expect(has(body, 'aging-year-select')).toBe(true);
    expect(has(body, 'aging-die-input')).toBe(true);
    // A stress die explodes, so the enterable value has a floor and no ceiling.
    expect(open(body, 'aging-die-input')).toContain('min="0"');
    expect(open(body, 'aging-die-input')).not.toContain('max=');
    // Nothing is claimed about a roll nobody has made.
    expect(has(body, 'aging-total')).toBe(false);
    expect(has(body, 'aging-outcome')).toBe(false);
    expect(has(body, 'aging-apply')).toBe(false);
    // The app never rolls; the hint says so.
    expect(text(body, 'aging-die-hint').length).toBeGreaterThan(0);
  });

  it("renders the engine's total and every part of it", () => {
    store.agingDraft = draft({ die: 10 });
    setPreview(
      total({
        die: 10,
        age_modifier: 4,
        living_conditions: { rows: [], from_table: 2, from_traits: 0, total: 2 },
        longevity_bonus: 1,
        trait_modifier: -1,
        uncapped_total: 10,
        total: 10,
      }),
      outcome({ total: 10 }),
    );
    const body = html();
    // The engine's number, announced — it changes on every keystroke in the die.
    expect(text(body, 'aging-total')).toContain('10');
    expect(open(body, 'aging-total')).toMatch(/role="status"/);
    // Every term that made it, so the arithmetic is visible rather than asserted.
    const parts = text(body, 'aging-total-parts');
    expect(parts).toContain('+10'); // the die
    expect(parts).toContain('+4'); // age/10, rounded up
    expect(parts).toContain('-2'); // living conditions are SUBTRACTED
    expect(parts).toContain('-1'); // and so is the Longevity Ritual
  });

  it('names the Characteristics an outcome fixes, in words', () => {
    store.agingDraft = draft({ die: 10 });
    setPreview(
      total(),
      outcome({ awards: [{ target: { kind: 'named', characteristic: 'qik' }, points: 1 }] }),
    );
    const body = html();
    const rendered = text(body, 'aging-outcome');
    expect(rendered).toContain('Quickness');
    // A raw slug is never a label.
    expect(rendered).not.toContain('qik');
    expect(visibleText(body)).not.toContain('qik');
    // The outcome changes with the die, so its arrival must be announced.
    expect(open(body, 'aging-outcome')).toMatch(/role="status"/);
  });

  it('will not apply until the distribution matches what the award owes', () => {
    // "Gain sufficient Aging Points (in any Characteristics)" — plural, so the
    // player may spread them; forcing them into one target would force drops the
    // player could legally avoid.
    // Source: Ars Magica - Definitive Edition (Core Rules).md:16602
    store.agingDraft = draft({ die: 9 });
    setPreview(
      total({ die: 9, total: 13 }),
      outcome({
        total: 13,
        awards: [{ target: { kind: 'next_decrepitude_level' }, points: 5 }],
        crisis: true,
      }),
    );
    let body = html();
    // One number input per Characteristic, and a running count against the award.
    expect(has(body, 'aging-distribute-qik')).toBe(true);
    expect(has(body, 'aging-distribute-sta')).toBe(true);
    expect(text(body, 'aging-distribute-remaining')).toContain('5');
    expect(open(body, 'aging-apply')).toContain('disabled');

    // Four of five placed is still short.
    store.agingDraft = draft({ die: 9, distribution: { qik: 2, sta: 2 } });
    expect(open(html(), 'aging-apply')).toContain('disabled');

    // Spread across three Characteristics, summing exactly: now it may be applied.
    store.agingDraft = draft({ die: 9, distribution: { qik: 2, sta: 2, per: 1 } });
    body = html();
    expect(open(body, 'aging-apply')).not.toContain('disabled');
    expect(text(body, 'aging-distribute-remaining')).toContain('5');
  });

  it('says a Crisis follows, and offers the die that resolves it', () => {
    store.agingDraft = draft({ die: 9 });
    setPreview(total({ die: 9, total: 13 }), crisisOutcome());
    const body = html();
    expect(has(body, 'aging-outcome-crisis')).toBe(true);
    expect(text(body, 'aging-outcome-crisis').length).toBeGreaterThan(0);
    // "Roll a ten-sided die. Each number counts for its value, except that a zero
    // counts as ten." (`:474`) — the bounds are the ruleset's, not a literal here.
    expect(has(body, 'crisis-die-input')).toBe(true);
    expect(open(body, 'crisis-die-input')).toContain('min="1"');
    expect(open(body, 'crisis-die-input')).toContain('max="10"');

    // A roll that is no Crisis says nothing about one, and asks for no die.
    setPreview(total(), outcome());
    const plain = html();
    expect(has(plain, 'aging-outcome-crisis')).toBe(false);
    expect(has(plain, 'crisis-die-input')).toBe(false);
  });

  it('reads the CRISIS TOTAL and names the row in words, never as a slug', () => {
    // "CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score"
    // Source: Ars Magica - Definitive Edition (Core Rules).md:16621
    store.agingDraft = draft({ die: 9, crisisDie: 10 });
    setPreview(total({ die: 9, total: 13 }), crisisOutcome(), crisis());
    const body = html();

    expect(text(body, 'crisis-total')).toContain('15');
    expect(open(body, 'crisis-total')).toMatch(/role="status"/);
    const parts = text(body, 'crisis-total-parts');
    expect(parts).toContain('+10'); // the Simple Die
    expect(parts).toContain('+4'); // age/10, rounded up
    expect(parts).toContain('+1'); // the Decrepitude this year raised

    // The row's text lives in `rules/i18n/<lang>/aging.json`, keyed by its id.
    const row = text(body, 'crisis-row');
    expect(row).toContain('Minor illness');
    expect(row).not.toContain('crisis.minor_illness');
    // The severity is an engine enum, so it reaches the screen through Fluent.
    expect(row.toLowerCase()).toContain('minor');
    expect(visibleText(body)).not.toContain('crisis.');
  });

  it('names every survival modifier separately, and invents none', () => {
    // "The character's aging rolls benefit from a +1 bonus … Furthermore, he
    // receives a +3 bonus to rolls to survive an aging crisis" (`:4530`), and
    // "you can apply your bronze cord score as a bonus to … rolls to resist
    // aging" (`:10844`). The panel has to NAME each, so they are never summed away.
    store.agingDraft = draft({ die: 9, crisisDie: 10 });
    setPreview(
      total({ die: 9, total: 13 }),
      crisisOutcome(),
      crisis({
        survival: {
          ease_factor: 3,
          ritual_level: 20,
          modifiers: [
            { source: { kind: 'trait', item: 'virtue.mild_aging' }, amount: 3 },
            { source: { kind: 'bronze_cord' }, amount: 2 },
          ],
          modifier_total: 5,
          allowances: [],
        },
      }),
    );
    let body = html();
    const survival = text(body, 'crisis-survival');
    // Ease Factor 3 or CrCo20 (`:16628`), both offered.
    expect(survival).toContain('3');
    expect(survival).toContain('20');
    expect(text(body, 'crisis-modifier-0')).toContain('Mild Aging');
    expect(text(body, 'crisis-modifier-0')).toContain('+3');
    expect(text(body, 'crisis-modifier-1')).toContain('+2');
    expect(text(body, 'crisis-modifier-total')).toContain('+5');
    expect(visibleText(body)).not.toContain('virtue.mild_aging');
    expect(visibleText(body)).not.toContain('bronze_cord');

    // A character with no familiar shows no cord line at all, never a "+0".
    setPreview(total({ die: 9, total: 13 }), crisisOutcome(), crisis());
    body = html();
    expect(has(body, 'crisis-modifier-0')).toBe(false);
    expect(has(body, 'crisis-modifier-total')).toBe(false);
  });

  it('states what an attending doctor may bring, without scoring it', () => {
    // "An Int + Medicine roll against an Ease Factor of 6 allows the character to
    // add the attendant's Medicine score … if the doctor botches the character
    // must subtract 3." (`:16634`) The Medicine belongs to another character, so
    // the app states the allowance and no more.
    store.agingDraft = draft({ die: 9, crisisDie: 10 });
    setPreview(
      total({ die: 9, total: 13 }),
      crisisOutcome(),
      crisis({
        survival: {
          ease_factor: 3,
          ritual_level: 20,
          modifiers: [],
          modifier_total: 0,
          allowances: [
            {
              kind: 'attendant',
              ability: 'ability.medicine',
              characteristic: 'int',
              ease_factor: 6,
              botch_penalty: -3,
            },
          ],
        },
      }),
    );
    const body = html();
    const allowance = text(body, 'crisis-allowance-0');
    expect(allowance).toContain('Medicine');
    expect(allowance).toContain('Intelligence');
    expect(allowance).toContain('6');
    expect(allowance).toContain('-3');
    expect(visibleText(body)).not.toContain('ability.medicine');
    expect(visibleText(body)).not.toContain('int ');
  });

  it('says a bedridden Crisis is time rather than a roll', () => {
    // "Bedridden for a week" (`:16626`) — no Stamina roll, no Ritual level, and
    // no empty read-out that would say "survivable on a 0".
    store.agingDraft = draft({ die: 9, crisisDie: 4 });
    setPreview(
      total({ die: 9, total: 13 }),
      crisisOutcome(),
      crisis({
        total: { age: 40, die: 4, age_modifier: 4, decrepitude_score: 1, total: 9 },
        row: 'crisis.bedridden_week',
        outcome: { type: 'bedridden' },
        survival: null,
      }),
    );
    const body = html();
    expect(text(body, 'crisis-row')).toContain('Bedridden for a week');
    expect(has(body, 'crisis-survival')).toBe(false);
    expect(text(body, 'crisis-bedridden').length).toBeGreaterThan(0);
  });

  it('says the Terminal row offers no roll at all', () => {
    // "**Terminal illness**. CrCo40 required to survive." (`:16632`) — an absent
    // Ease Factor is no roll, never an unbeatable one.
    store.agingDraft = draft({ die: 9, crisisDie: 10 });
    setPreview(
      total({ die: 9, total: 13 }),
      crisisOutcome(),
      crisis({
        total: { age: 40, die: 10, age_modifier: 4, decrepitude_score: 5, total: 19 },
        row: 'crisis.terminal_illness',
        outcome: { type: 'illness', severity: 'terminal', ritual_level: 40 },
        survival: { ritual_level: 40, modifiers: [], modifier_total: 0, allowances: [] },
      }),
    );
    const body = html();
    expect(text(body, 'crisis-survival')).toContain('40');
    expect(has(body, 'crisis-survival-ease-factor')).toBe(false);
    expect(text(body, 'crisis-survival-no-roll').length).toBeGreaterThan(0);
  });

  it('reports the Longevity Ritual the applied year spent', () => {
    // "the ritual assures that the character survives, but its power is spent"
    // (`:16573`) — reported, because the entity keeps the stored choice, so this
    // is the only place the player can be told.
    store.agingNotes = [{ kind: 'longevity_ritual_spent' }];
    const body = html();
    expect(text(body, 'aging-note-0').length).toBeGreaterThan(0);
    expect(visibleText(body)).not.toContain('longevity_ritual_spent');

    store.agingNotes = [];
    expect(has(html(), 'aging-note-0')).toBe(false);
  });

  it('writes every crisis figure with an ASCII hyphen', () => {
    store.agingDraft = draft({ die: 9, crisisDie: 10 });
    setPreview(
      total({ die: 9, total: 13 }),
      crisisOutcome(),
      crisis({
        total: { age: 40, die: 10, age_modifier: 4, decrepitude_score: 1, total: 15 },
        survival: {
          ease_factor: 3,
          ritual_level: 20,
          modifiers: [{ source: { kind: 'bronze_cord' }, amount: -2 }],
          modifier_total: -2,
          allowances: [
            {
              kind: 'attendant',
              ability: 'ability.medicine',
              characteristic: 'int',
              ease_factor: 6,
              botch_penalty: -3,
            },
          ],
        },
      }),
    );
    const body = html();
    expect(text(body, 'crisis-modifier-total')).toContain('-2');
    // U+2212 never reaches the screen; formatSigned is the source of truth.
    expect(body).not.toContain('−');
  });

  it('says in words that nothing it shows is recorded until applied', () => {
    store.agingDraft = draft({ die: 10 });
    setPreview(total(), outcome());
    const note = text(html(), 'aging-calculator-note');
    expect(note.length).toBeGreaterThan(0);
    // Not a bare glyph or an icon: the guarantee is stated in words.
    expect(note.split(' ').length).toBeGreaterThan(3);
  });

  it('writes every displayed modifier with an ASCII hyphen', () => {
    store.agingDraft = draft({ die: 10 });
    setPreview(
      total({
        living_conditions: { rows: [], from_table: 2, from_traits: 0, total: 2 },
        longevity_bonus: 7,
        trait_modifier: -3,
        uncapped_total: 2,
        total: 2,
        capped_by_longevity: true,
      }),
      outcome({ total: 2, apparent_age_increases: false }),
    );
    const body = html();
    expect(text(body, 'aging-total-parts')).toContain('-2');
    expect(text(body, 'aging-total-parts')).toContain('-7');
    expect(text(body, 'aging-total-parts')).toContain('-3');
    // The clamp bit this roll, so it is named — and only then.
    expect(has(body, 'aging-die-capped')).toBe(true);
    // U+2212 never reaches the screen; formatSigned is the source of truth.
    expect(body).not.toContain('−');
  });

  it('offers a revert for each year already recorded, and only those', () => {
    setSchedule(schedule([36, 37, 38, 39, 40], [36, 37]));
    const body = html();
    expect(has(body, 'aging-revert-36')).toBe(true);
    expect(has(body, 'aging-revert-37')).toBe(true);
    expect(has(body, 'aging-revert-38')).toBe(false);
  });
});
