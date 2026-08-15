import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { AgingReadout, EffectiveScores, Entity, LocalizedRuleset } from '../types';

// The picker reads the shared store singleton: the loaded ruleset's Living
// Conditions catalogue (`rules/core/aging.json` + `rules/i18n/<lang>/aging.json`),
// the entity's chosen rows, and the engine's resolved modifier off
// `effective.aging`. Mock the Tauri bridge so nothing reaches a backend; harness
// mirrors AgingSchedulePanel.test.ts.
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
import LivingConditionsPicker from './LivingConditionsPicker.svelte';

/**
 * Four rows of the shipped table (Core Rules.md:16583-16592): two plain
 * alternatives and two of the five asterisked, cumulative ones. Catalogue size is
 * data, so nothing here asserts the shipped ten — only that every row the ruleset
 * carries gets a checkbox.
 */
const ROWS = [
  { id: 'living_condition.wealthy_or_healthy_location', modifier: 2 },
  { id: 'living_condition.average_peasant', modifier: 0 },
  { id: 'living_condition.work_in_a_mine', modifier: -1, cumulative: true },
  { id: 'living_condition.leper', modifier: -2, cumulative: true },
];

const NAMES: Record<string, string> = {
  'living_condition.wealthy_or_healthy_location': 'Wealthy, or healthy location',
  'living_condition.average_peasant': 'Average peasant',
  'living_condition.work_in_a_mine': 'Work in a mine',
  'living_condition.leper': 'Leper',
};

/** Install a minimal localized ruleset; `null` ships no aging block at all. */
function installRuleset(rows: typeof ROWS | null = ROWS): void {
  const i18n: Record<string, { name: string }> = {};
  for (const [id, name] of Object.entries(NAMES)) i18n[id] = { name };
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
      ...(rows
        ? {
            aging: {
              start_age: 35,
              age_divisor: 10,
              apparent_age_increase_min: 3,
              living_conditions: rows,
              outcomes: [],
            },
          }
        : {}),
    },
    i18n,
  } as unknown as LocalizedRuleset;
}

function resetEntity(chosen: string[] = []): void {
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
    ...(chosen.length > 0 ? { living_conditions: chosen } : {}),
  };
}

/** The engine's readout, with only the field this picker reads set. */
function setModifier(modifier: number | null): void {
  const aging: AgingReadout | null =
    modifier == null
      ? null
      : ({
          first_roll_age: 36,
          begins_after_age: 35,
          schedule: [],
          rolls_owed: 0,
          rolls_recorded: 0,
          age_modifier: 0,
          living_conditions_modifier: modifier,
          longevity_modifier: 0,
          longevity_clamp_active: false,
          fixed_total: -modifier,
        } as AgingReadout);
  store.effective = { ...(store.effective ?? {}), aging } as unknown as EffectiveScores;
}

/** Render the picker to an HTML string (node env, no DOM). */
function html(): string {
  return render(LivingConditionsPicker, { props: {} }).body;
}

/** A data-testid holding a dotted id needs its dots escaped for the regex. */
function escaped(testid: string): string {
  return testid.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${escaped(testid)}"`).test(body);
}

/** The opening tag of the single element carrying a data-testid. */
function open(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${escaped(testid)}"[^>]*>`, 'i').exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[0];
}

/** The visible text of the element carrying a data-testid, tags stripped. */
function text(body: string, testid: string): string {
  const match = new RegExp(
    `<[^>]*data-testid="${escaped(testid)}"[^>]*>([\\s\\S]*?)</[a-z]+>`,
    'i',
  ).exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return stripMarks(match[1].replace(/<[^>]*>/g, ' '))
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
  installRuleset();
  resetEntity();
  setModifier(0);
});

describe('LivingConditionsPicker (slice 6b6c)', () => {
  it('renders nothing for a ruleset shipping no aging rules', () => {
    installRuleset(null);
    // A subsystem the data stands down must not appear as an empty checklist.
    expect(
      html()
        .replace(/<!--[\s\S]*?-->/g, '')
        .trim(),
    ).toBe('');
  });

  it('offers one checkbox per catalogue row, labelled from the rules i18n', () => {
    const body = html();
    expect(has(body, 'living-conditions')).toBe(true);
    for (const row of ROWS) {
      expect(has(body, `living-condition-${row.id}`)).toBe(true);
    }
    // The full id is the testid — the repo's `prefix-fullid` law, so there is one
    // id vocabulary and not two.
    expect(has(body, 'living-condition-leper')).toBe(false);
    // Exactly the catalogue's rows, no more: one checkbox each.
    expect([...body.matchAll(/type="checkbox"/g)]).toHaveLength(ROWS.length);
    // Every label is the localized rules name; no slug ever reaches the screen.
    const visible = visibleText(body);
    for (const name of Object.values(NAMES)) expect(visible).toContain(name);
    expect(visible).not.toContain('living_condition.');
  });

  it('checks the rows the character already lives under', () => {
    resetEntity(['living_condition.leper', 'living_condition.work_in_a_mine']);
    const body = html();
    expect(open(body, 'living-condition-living_condition.leper')).toContain('checked');
    expect(open(body, 'living-condition-living_condition.work_in_a_mine')).toContain('checked');
    expect(open(body, 'living-condition-living_condition.average_peasant')).not.toContain(
      'checked',
    );
    // A chosen row means the "nothing chosen" line is gone.
    expect(has(body, 'living-conditions-none')).toBe(false);
  });

  it('says so when the character lives under none of them', () => {
    // An empty set IS the table's own "Average peasant 0", so this is a complete
    // answer and not a missing one.
    expect(has(html(), 'living-conditions-none')).toBe(true);
  });

  it("shows the engine's resolved modifier, not a sum of its own", () => {
    // Wealthy alone would sum to +2 in JS. The engine reports -3, because it also
    // counts the Virtue/Flaw `living_conditions` modifiers the picker cannot see —
    // so the read-out must be the engine's number, never a local sum.
    resetEntity(['living_condition.wealthy_or_healthy_location']);
    setModifier(-3);
    const body = html();
    expect(text(body, 'living-conditions-total')).toContain('-3');
    expect(text(body, 'living-conditions-total')).not.toContain('+2');
    // It changes the moment a row is ticked, so its arrival must be announced.
    expect(open(body, 'living-conditions-total')).toMatch(/role="status"/);
  });

  it('marks the cumulative rows and explains what that means', () => {
    const body = html();
    // "Modifiers marked with an asterisk are cumulative with each other."
    // Source: Ars Magica - Definitive Edition (Core Rules).md:16594
    for (const row of ROWS.filter((r) => r.cumulative)) {
      expect(open(body, `living-condition-${row.id}`)).toContain('data-cumulative="true"');
    }
    for (const row of ROWS.filter((r) => !r.cumulative)) {
      expect(open(body, `living-condition-${row.id}`)).not.toContain('data-cumulative');
    }
    // Not colour-only: each cumulative row carries a readable label, and the rule
    // itself is spelled out once.
    expect(has(body, 'living-conditions-cumulative-note')).toBe(true);
    expect(text(body, 'living-conditions-cumulative-note').length).toBeGreaterThan(0);
    const marks = [...visibleText(body).matchAll(/cumulative/gi)];
    expect(marks.length).toBeGreaterThanOrEqual(ROWS.filter((r) => r.cumulative).length);
  });

  it('writes displayed negative modifiers with an ASCII hyphen', () => {
    setModifier(-3);
    const body = html();
    expect(text(body, 'living-condition-modifier-living_condition.work_in_a_mine')).toBe('-1');
    expect(text(body, 'living-condition-modifier-living_condition.leper')).toBe('-2');
    expect(text(body, 'living-condition-modifier-living_condition.average_peasant')).toBe('0');
    expect(
      text(body, 'living-condition-modifier-living_condition.wealthy_or_healthy_location'),
    ).toBe('+2');
    // U+2212 never reaches the screen; formatSigned is the source of truth.
    expect(body).not.toContain('−');
  });
});
