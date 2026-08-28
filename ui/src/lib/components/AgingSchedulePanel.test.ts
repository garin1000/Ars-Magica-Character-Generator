import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { AgingReadout, EffectiveScores, Entity } from '../types';

// The panel is a pure read-out over `store.effective.aging` — the engine's
// die-independent half of the aging total — plus the Fluent bundle. The store
// schedules a debounced revalidate over the Tauri IPC bridge; mock the bridge so
// nothing reaches a backend. Harness mirrors LifeStagePanel.test.ts.
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

import { SCHEMA_VERSION, store } from '../state.svelte';
import AgingSchedulePanel from './AgingSchedulePanel.svelte';

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
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
}

/**
 * A readout as the engine builds it: nothing owed under the shipped threshold,
 * every modifier zero. Overrides are merged on top.
 */
function readout(overrides: Partial<AgingReadout> = {}): AgingReadout {
  return {
    first_roll_age: 36,
    begins_after_age: 35,
    schedule: [],
    rolls_owed: 0,
    rolls_recorded: 0,
    age_modifier: 0,
    living_conditions_modifier: 0,
    longevity_modifier: 0,
    trait_modifier: 0,
    longevity_clamp_active: false,
    fixed_total: 0,
    ...overrides,
  };
}

/** A character of `age` owing every year from `first` up, none recorded yet. */
function owing(age: number, first: number, birthYear: number | null = null): AgingReadout {
  const schedule = [];
  for (let a = first; a <= age; a += 1) {
    schedule.push({ age: a, year: birthYear == null ? null : birthYear + a, recorded: false });
  }
  return readout({
    first_roll_age: first,
    begins_after_age: first - 1,
    schedule,
    rolls_owed: schedule.length,
    age_modifier: Math.ceil(age / 10),
    fixed_total: Math.ceil(age / 10),
  });
}

function setAging(aging: AgingReadout | null): void {
  store.effective = {
    ...(store.effective ?? {}),
    aging,
  } as unknown as EffectiveScores;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(AgingSchedulePanel, { props: {} }).body;
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

/** All rendered text, tags stripped — for the "no slug anywhere" sweeps. */
function text(body: string): string {
  return body.replace(/<[^>]*>/g, ' ');
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  resetEntity();
});

describe('AgingSchedulePanel (slice 6b6b)', () => {
  it('renders nothing for a ruleset shipping no aging rules', () => {
    setAging(null);
    // A subsystem the data stands down must not appear as an empty read-out.
    expect(
      html()
        .replace(/<!--[\s\S]*?-->/g, '')
        .trim(),
    ).toBe('');
  });

  it('says nothing is owed for a character who has not begun aging', () => {
    setAging(readout());
    const body = html();
    expect(has(body, 'aging-schedule')).toBe(true);
    expect(has(body, 'aging-rolls-none')).toBe(true);
    expect(has(body, 'aging-rolls-owed')).toBe(false);
    // It changes the moment an age is typed, so its arrival must be announced.
    expect(element(body, 'aging-schedule').open).toMatch(/role="status"/);
  });

  it('names the years a character of forty owes rolls for', () => {
    setAging(owing(40, 36, 1180));
    const body = html();
    const owed = element(body, 'aging-rolls-owed');
    // 36, 37, 38, 39, 40 — five rolls, the engine's own count.
    expect(owed.text).toContain('5');
    expect(owed.text).toContain('36');
    expect(owed.text).toContain('40');
    expect(has(body, 'aging-rolls-none')).toBe(false);
    // A birth year turns each owed age into a calendar year.
    expect(element(body, 'aging-rolls-years').text).toContain('1216');
    expect(element(body, 'aging-rolls-years').text).toContain('1220');
    // None of them recorded yet.
    expect(element(body, 'aging-rolls-recorded').text).toContain('0');
    expect(element(body, 'aging-rolls-recorded').text).toContain('5');
  });

  it('reads the first-roll age off the engine, never a literal', () => {
    // A ruleset whose aging starts later: the panel must print 40 and 39, not the
    // shipped 36 and 35.
    setAging(readout({ first_roll_age: 40, begins_after_age: 39 }));
    const first = element(html(), 'aging-first-roll-age');
    expect(first.text).toContain('40');
    expect(first.text).toContain('39');
    expect(first.text).not.toContain('36');
    expect(first.text).not.toContain('35');
  });

  it('writes every modifier with an ASCII hyphen', () => {
    setAging(
      readout({
        age_modifier: 4,
        living_conditions_modifier: -2,
        longevity_modifier: 7,
        fixed_total: -1,
      }),
    );
    const formula = element(html(), 'aging-total-formula');
    // Living Conditions and the ritual are SUBTRACTED, so a stored -2 raises the
    // total by 2 and a stored +7 lowers it by 7.
    expect(formula.text).toContain('+4');
    expect(formula.text).toContain('+2');
    expect(formula.text).toContain('-7');
    expect(formula.text).toContain('-1');
    // U+2212 never reaches the screen; formatSigned is the source of truth.
    expect(formula.text).not.toContain('−');
    expect(html()).not.toContain('−');
  });

  // guided-creation-review-2026-08 #22: the sentence named exactly the book's three
  // terms while `fixed_total` is the engine's sum of ALL of them, so a character with
  // an aging-roll Virtue or Flaw read "+4 (age) 0 (living conditions) 0 (Longevity
  // Ritual) = stress die +3" — a rules read-out that does not add up. The check is
  // the arithmetic itself, not the mere presence of a fourth placeholder.
  it('names the Virtue/Flaw term, and its terms sum to the stated total', () => {
    // A character of 40 with Faerie Blood: ceil(40/10) = +4 for the age, and the
    // Flaw's -1 aging-roll modifier, which the book's three lines do not name.
    setAging(readout({ age_modifier: 4, trait_modifier: -1, fixed_total: 3 }));
    const formula = element(html(), 'aging-total-formula').text;
    expect(formula).toContain('Virtues and Flaws');
    // Every signed figure in the sentence, in order: the four named terms, then the
    // total they are claimed to make.
    const figures = [...formula.matchAll(/[+-]?\d+/g)].map((match) => Number(match[0]));
    expect(figures).toHaveLength(5);
    const total = figures.pop();
    expect(figures.reduce((sum, term) => sum + term, 0)).toBe(total);
    expect(total).toBe(3);
    // ASCII hyphen-minus, never U+2212.
    expect(formula).not.toContain('−');
  });

  it('names the standing Longevity Ritual clamp only while it stands', () => {
    setAging(readout({ longevity_modifier: 7 }));
    expect(has(html(), 'aging-longevity-clamp')).toBe(false);
    setAging(readout({ longevity_modifier: 7, longevity_clamp_active: true }));
    expect(has(html(), 'aging-longevity-clamp')).toBe(true);
  });

  it('renders no raw slug or code as a label', () => {
    setAging(owing(40, 36, 1180));
    const rendered = text(html());
    for (const slug of [
      'first_roll_age',
      'begins_after_age',
      'rolls_owed',
      'rolls_recorded',
      'age_modifier',
      'living_conditions_modifier',
      'longevity_modifier',
      'trait_modifier',
      'longevity_clamp_active',
      'fixed_total',
      'aging-rolls-owed',
    ]) {
      expect(rendered).not.toContain(slug);
    }
    // And the labels are the English bundle's, so nothing fell back to a key.
    expect(rendered).toContain('Aging');
  });
});
