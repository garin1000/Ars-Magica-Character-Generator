import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LifeSection, LifeStagePlan } from '../types';

// G3: the life-section editor — one row per named section (name, years, move up,
// move down, remove), an add button, and for a magus each section's own lab years
// plus the Remaining row's (the plan-level counts). Rendered with `svelte/server`:
// everything asserted here is markup; the listeners are wired in
// `LifeSections.client.test.ts`. Importing the store means mocking the IPC bridge.
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
import LifeSections from './LifeSections.svelte';

function resetEntity(plan: LifeStagePlan): void {
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
    ability_funding: 'life_stages',
    life_stages: plan,
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
    age: 40,
  };
}

/** Two sections, the second unnamed so its placeholder label shows. */
function sections(): LifeSection[] {
  return [
    { name: 'Squire', years: 5, lab_years: { four_seasons: 2 } },
    { name: '', years: 10 },
  ];
}

function html(
  props: Partial<{
    labYears: boolean;
    labYearsReadonly: boolean;
    labYearsDescribedBy: string;
  }> = {},
): string {
  return render(LifeSections, { props: { labYears: false, ...props } }).body;
}

/** The opening tag of the element carrying `testid`. */
function openTag(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  expect(match, `no element with data-testid="${testid}"`).not.toBeNull();
  return match![0];
}

function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

/** The markup of section row `i`: from its own testid to the next row's (or the add button). */
function row(body: string, i: number): string {
  const start = body.indexOf(`data-testid="life-section-${i}"`);
  expect(start, `no row ${i}`).not.toBe(-1);
  const next = body.indexOf(`data-testid="life-section-${i + 1}"`, start);
  const end = next === -1 ? body.indexOf('data-testid="life-section-add"', start) : next;
  return body.slice(start, end === -1 ? undefined : end);
}

/** The text of the first `<legend>` inside `markup`. */
function legend(markup: string): string {
  const match = /<legend[^>]*>([\s\S]*?)<\/legend>/i.exec(markup);
  expect(match, 'no legend').not.toBeNull();
  return match![1].replace(/<[^>]*>/g, '').trim();
}

beforeEach(() => {
  store.lang = 'en';
  resetEntity({ life_sections_enabled: true, life_sections: sections() });
});

describe('LifeSections editor rows (G3)', () => {
  it('lists the sections in order, as an ordered list named through Fluent', () => {
    const body = html();
    const list = openTag(body, 'life-sections-list');
    expect(list).toMatch(/^<ol\b/);
    expect(list).toMatch(/aria-label="Life sections"/);
    expect(body.indexOf('data-testid="life-section-0"')).toBeLessThan(
      body.indexOf('data-testid="life-section-1"'),
    );
    expect(has(body, 'life-section-2')).toBe(false);
  });

  it('groups each row under a legend naming the section, or "Section n" when unnamed', () => {
    const body = html();
    expect(legend(row(body, 0))).toBe('Squire');
    expect(legend(row(body, 1))).toBe('Section 2');
  });

  it('offers the name as a labelled text field, with the fallback as its placeholder', () => {
    const body = html();
    const name0 = openTag(body, 'life-section-0-name-input');
    expect(name0).toMatch(/type="text"/);
    expect(name0).toMatch(/value="Squire"/);
    expect(name0).toMatch(/placeholder="Section 1"/);
    expect(openTag(body, 'life-section-1-name-input')).toMatch(/placeholder="Section 2"/);
    expect(row(body, 0)).toContain('Name');
  });

  it('offers the years as a labelled number field', () => {
    const body = html();
    const years = openTag(body, 'life-section-1-years-input');
    expect(years).toMatch(/type="number"/);
    expect(years).toMatch(/min="0"/);
    expect(years).toMatch(/value="10"/);
    expect(row(body, 1)).toContain('Years');
  });

  it('gives every button an accessible name through Fluent', () => {
    const body = html();
    expect(openTag(body, 'life-section-0-up')).toMatch(/aria-label="Move Squire up"/);
    expect(openTag(body, 'life-section-0-down')).toMatch(/aria-label="Move Squire down"/);
    expect(openTag(body, 'life-section-0-delete')).toMatch(/aria-label="Remove Squire"/);
    expect(openTag(body, 'life-section-1-up')).toMatch(/aria-label="Move Section 2 up"/);
    expect(openTag(body, 'life-section-1-delete')).toMatch(/aria-label="Remove Section 2"/);
    for (const testid of ['life-section-0-up', 'life-section-1-down', 'life-section-add']) {
      expect(openTag(body, testid)).toMatch(/^<button\b[^>]*type="button"/);
    }
  });

  it('disables moving the first section up and the last one down, and nothing else', () => {
    const body = html();
    expect(openTag(body, 'life-section-0-up')).toMatch(/\bdisabled\b/);
    expect(openTag(body, 'life-section-0-down')).not.toMatch(/\bdisabled\b/);
    expect(openTag(body, 'life-section-1-up')).not.toMatch(/\bdisabled\b/);
    expect(openTag(body, 'life-section-1-down')).toMatch(/\bdisabled\b/);
  });

  it('offers an add button labelled through Fluent, even with no section yet', () => {
    resetEntity({ life_sections_enabled: true });
    const body = html();
    expect(has(body, 'life-section-0')).toBe(false);
    expect(body).toMatch(/data-testid="life-section-add"[^>]*>\s*Add section\s*</);
  });
});

describe('LifeSections lab years (G3)', () => {
  it('carries no lab-year counts unless told the sections are a magus’s', () => {
    resetEntity({
      life_sections_enabled: true,
      life_sections: sections(),
      post_gauntlet_lab_years: { one_season: 1 },
    });
    const body = html({ labYears: false });
    expect(has(body, 'life-section-0-lab-years-1-input')).toBe(false);
    expect(has(body, 'life-stage-lab-years-1-input')).toBe(false);
  });

  it('gives each section its own four counts, under a legend naming the section', () => {
    const body = html({ labYears: true });
    const first = row(body, 0);
    for (const n of [1, 2, 3, 4]) {
      expect(first).toContain(`data-testid="life-section-0-lab-years-${n}-input"`);
    }
    expect(openTag(body, 'life-section-0-lab-years-4-input')).toMatch(/value="2"/);
    expect(openTag(body, 'life-section-1-lab-years-4-input')).toMatch(/value=""/);
    expect(first).toMatch(/<legend[^>]*>\s*Squire: years of lab work\s*<\/legend>/);
    expect(row(body, 1)).toMatch(/<legend[^>]*>\s*Section 2: years of lab work\s*<\/legend>/);
  });

  it('puts the plan-level counts in a Remaining row, under their usual testids', () => {
    resetEntity({
      life_sections_enabled: true,
      life_sections: sections(),
      post_gauntlet_lab_years: { one_season: 3 },
    });
    const body = html({ labYears: true });
    const remaining = body.slice(body.indexOf('data-testid="life-section-remaining"'));
    expect(has(body, 'life-section-remaining')).toBe(true);
    expect(remaining).toMatch(/<legend[^>]*>\s*Remaining: years of lab work\s*<\/legend>/);
    expect(openTag(body, 'life-stage-lab-years-1-input')).toMatch(/value="3"/);
    expect(remaining).toContain('data-testid="life-stage-lab-years-1-input"');
  });

  it('closes every count read-only, pointing at the note, when told to', () => {
    const body = html({ labYears: true, labYearsReadonly: true, labYearsDescribedBy: 'why' });
    for (const testid of ['life-section-0-lab-years-1-input', 'life-stage-lab-years-4-input']) {
      const open = openTag(body, testid);
      expect(open).toMatch(/readonly/);
      expect(open).toMatch(/aria-describedby="why"/);
    }
  });
});

describe('LifeSections in German (G3)', () => {
  it('labels the rows and buttons in German', () => {
    store.lang = 'de';
    const body = html({ labYears: true });
    expect(legend(row(body, 1))).toBe('Abschnitt 2');
    expect(openTag(body, 'life-section-1-delete')).toMatch(/aria-label="Abschnitt 2 entfernen"/);
    expect(openTag(body, 'life-section-0-up')).toMatch(/aria-label="Squire nach oben verschieben"/);
    expect(body).toMatch(/data-testid="life-section-add"[^>]*>\s*Abschnitt hinzufügen\s*</);
  });
});
