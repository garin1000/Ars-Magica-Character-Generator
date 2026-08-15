import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { AgingOutcome, AgingTotal } from '../ipc';
import type { AgingReadout, AgingScheduleYear, EffectiveScores, Entity } from '../types';

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

import { SCHEMA_VERSION, defaultAgingDraft, store } from '../state.svelte';
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

/** Put the engine's answer on the store, as the debounced preview would. */
function setPreview(t: AgingTotal, o: AgingOutcome): void {
  store.agingPreview = { total: t, outcome: o };
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
  store.agingDraft = defaultAgingDraft();
  store.agingPreview = null;
  store.agingRejections = [];
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
    store.agingDraft = { age: 40, die: 10, distribution: {} };
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
    store.agingDraft = { age: 40, die: 10, distribution: {} };
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
    store.agingDraft = { age: 40, die: 9, distribution: {} };
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
    store.agingDraft = { age: 40, die: 9, distribution: { qik: 2, sta: 2 } };
    expect(open(html(), 'aging-apply')).toContain('disabled');

    // Spread across three Characteristics, summing exactly: now it may be applied.
    store.agingDraft = { age: 40, die: 9, distribution: { qik: 2, sta: 2, per: 1 } };
    body = html();
    expect(open(body, 'aging-apply')).not.toContain('disabled');
    expect(text(body, 'aging-distribute-remaining')).toContain('5');
  });

  it('warns that a Crisis is not resolved by the app yet', () => {
    store.agingDraft = { age: 40, die: 9, distribution: {} };
    setPreview(
      total({ die: 9, total: 13 }),
      outcome({
        total: 13,
        awards: [{ target: { kind: 'next_decrepitude_level' }, points: 5 }],
        crisis: true,
      }),
    );
    const body = html();
    expect(has(body, 'aging-outcome-crisis')).toBe(true);
    expect(text(body, 'aging-outcome-crisis').length).toBeGreaterThan(0);

    // A roll that is no Crisis says nothing about one.
    setPreview(total(), outcome());
    expect(has(html(), 'aging-outcome-crisis')).toBe(false);
  });

  it('says in words that nothing it shows is recorded until applied', () => {
    store.agingDraft = { age: 40, die: 10, distribution: {} };
    setPreview(total(), outcome());
    const note = text(html(), 'aging-calculator-note');
    expect(note.length).toBeGreaterThan(0);
    // Not a bare glyph or an icon: the guarantee is stated in words.
    expect(note.split(' ').length).toBeGreaterThan(3);
  });

  it('writes every displayed modifier with an ASCII hyphen', () => {
    store.agingDraft = { age: 40, die: 10, distribution: {} };
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
