import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { AbilityParamValue, LinkTarget, LocalizedRuleset } from '../types';

// N4b: the Ability row's select + "Other…" combo, lifted out of AbilityTab so the
// native language and the childhood slots of a catalogued Ability use the same
// control. Catalogue values first, then the row's link targets, then "Other…",
// which opens a free-text field beside the select. The component writes nothing
// itself: it reports a choice or typed text to its owner.
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

import { store } from '../state.svelte';
import AbilityParameterCombo from './AbilityParameterCombo.svelte';

const CATALOGUED = ['language.arabic', 'language.greek'];

function installNames(): void {
  store.ruleset = {
    ruleset: { id: 'test', version: '1', point_items: {}, type_profiles: {}, abilities: {} },
    i18n: {
      'language.arabic': { name: 'Arabic' },
      'language.greek': { name: 'Greek' },
      'virtue.craft_guild_training': { name: 'Craft Guild Training' },
    },
  } as unknown as LocalizedRuleset;
}

interface Overrides {
  value?: AbilityParamValue | null;
  linked?: LinkTarget[];
  invalid?: boolean;
}

function html(overrides: Overrides = {}): string {
  return render(AbilityParameterCombo, {
    props: {
      catalogued: CATALOGUED,
      linked: overrides.linked,
      linkLabel: (link: LinkTarget) => `follows ${link.resolved ?? ''}`,
      value: overrides.value,
      label: 'Native language',
      placeholder: 'e.g. German',
      invalid: overrides.invalid,
      selectTestId: 'combo-select',
      inputTestId: 'combo-input',
      onchoose: () => {},
      ontext: () => {},
    },
  }).body;
}

/** The whole `<select>` block carrying a data-testid. */
function selectBlock(body: string, testid: string): string {
  const match = new RegExp(`<select[^>]*data-testid="${testid}"[\\s\\S]*?</select>`, 'i').exec(
    body,
  );
  if (!match) throw new Error(`no <select> with data-testid="${testid}"`);
  return match[0];
}

/** The opening tag of the element carrying a data-testid. */
function open(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[0];
}

/** The option values in document order, and the one marked selected. */
function options(select: string): { values: string[]; selected: string | null } {
  const tags = [...select.matchAll(/<option([^>]*)>/g)].map((m) => m[1]);
  const values = tags.map((attrs) => /value="([^"]*)"/.exec(attrs)?.[1] ?? '');
  const selected = tags.find((attrs) => /\bselected\b/.test(attrs));
  return {
    values,
    selected: selected ? (/value="([^"]*)"/.exec(selected)?.[1] ?? null) : null,
  };
}

beforeEach(() => {
  store.lang = 'en';
  installNames();
});

describe('AbilityParameterCombo (N4b)', () => {
  it('offers the catalogue by name in a labelled native select, then "Other…"', () => {
    const select = selectBlock(html(), 'combo-select');
    expect(open(select, 'combo-select')).toMatch(/aria-label="Native language"/);
    expect(options(select).values).toEqual(['cat:language.arabic', 'cat:language.greek', 'other']);
    const visible = select.replace(/<[^>]*>/g, ' ');
    expect(visible).toContain('Arabic');
    expect(visible).toContain('Greek');
    expect(visible).toContain('Other…');
    // An id backs the option's own value attribute and is never shown.
    expect(visible).not.toContain('language.');
  });

  it('selects a catalogue value and opens no text field', () => {
    const body = html({ value: { id: 'language.arabic' } });
    expect(options(selectBlock(body, 'combo-select')).selected).toBe('cat:language.arabic');
    expect(body).not.toContain('data-testid="combo-input"');
  });

  it('selects "Other…" for typed text and shows the text beside the select', () => {
    const body = html({ value: { text: 'Gaelic' } });
    expect(options(selectBlock(body, 'combo-select')).selected).toBe('other');
    const input = open(body, 'combo-input');
    expect(input).toMatch(/type="text"/);
    expect(input).toMatch(/value="Gaelic"/);
    expect(input).toMatch(/placeholder="e\.g\. German"/);
  });

  it('selects "Other…" with an empty field while nothing is chosen', () => {
    for (const value of [undefined, null]) {
      const body = html({ value });
      expect(options(selectBlock(body, 'combo-select')).selected).toBe('other');
      expect(open(body, 'combo-input')).not.toMatch(/value="[^"]+"/);
    }
  });

  it('keeps the select and its field in one wrapper, select first, with no inner div', () => {
    const body = html({ value: { text: 'Gaelic' } });
    const start = body.indexOf('class="ability-param-combo"');
    expect(start).toBeGreaterThan(-1);
    const combo = body.slice(start, body.indexOf('</div>', start));
    expect(combo.indexOf('data-testid="combo-select"')).toBeGreaterThan(-1);
    expect(combo.indexOf('data-testid="combo-select"')).toBeLessThan(
      combo.indexOf('data-testid="combo-input"'),
    );
  });

  it('lists link targets between the catalogue and "Other…" only when given', () => {
    const linked = [
      { item: 'virtue.craft_guild_training', param: 'guild', resolved: 'Smiths of Verdi' },
    ];
    const select = selectBlock(html({ linked }), 'combo-select');
    expect(options(select).values).toEqual([
      'cat:language.arabic',
      'cat:language.greek',
      'link:virtue.craft_guild_training\u0000guild',
      'other',
    ]);
    expect(select).toContain('follows Smiths of Verdi');
    expect(options(selectBlock(html(), 'combo-select')).values).not.toContain(
      'link:virtue.craft_guild_training\u0000guild',
    );
  });

  it('marks both parts invalid when its owner says so', () => {
    const body = html({ value: { text: 'Gaelic' }, invalid: true });
    expect(open(body, 'combo-select')).toMatch(/aria-invalid="true"/);
    expect(open(body, 'combo-input')).toMatch(/aria-invalid="true"/);
  });
});
