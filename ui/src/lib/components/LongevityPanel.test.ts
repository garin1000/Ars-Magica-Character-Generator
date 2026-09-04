import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, Entity, LocalizedRuleset, LongevityHint } from '../types';

// The longevity panel reads the shared store singleton (the entity's stored ritual
// plus the engine's derived read-out) and the Fluent bundle. The store schedules a
// debounced revalidate over the Tauri IPC bridge; mock the bridge so nothing
// reaches a backend. Harness mirrors SpellBudgetBar.test.ts.
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
import LongevityPanel from './LongevityPanel.svelte';

/** A minimal localized ruleset: the panel needs no catalogue, only the bundle. */
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
    },
    i18n: {},
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
  store.derived = null;
}

/** Install the engine's longevity read-out, the panel's only derived input. */
function setDerived(
  source: 'self_made' | 'external',
  bonus: number,
  entered: boolean,
  hint: LongevityHint | null,
): void {
  store.derived = {
    longevity: { source, bonus, entered, bronze_cord: 0, hint },
  } as unknown as DerivedTotals;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(LongevityPanel, { props: {} }).body;
}

/** The single element carrying a given data-testid, with its class attribute. */
function element(body: string, testid: string): { open: string; text: string } {
  const re = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i');
  const match = re.exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  const openMatch = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  return { open: openMatch![0], text: match[1].replace(/<[^>]*>/g, '').trim() };
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('LongevityPanel entered bonus (stored for both sources)', () => {
  it('offers the bonus input for a self-made ritual, not only an external one', () => {
    store.addLongevityRitual('self_made');
    setDerived('self_made', 0, false, { lab_total: 35, suggested_bonus: 7, halved: false });
    const { open } = element(html(), 'longevity-bonus');
    expect(open).toMatch(/<input/i);
  });

  it('leaves the input empty and flags "not entered" when no bonus is stored', () => {
    store.addLongevityRitual('self_made');
    setDerived('self_made', 0, false, { lab_total: 35, suggested_bonus: 7, halved: false });
    const body = html();
    // Empty, not a placeholder 0 that would read as a deliberate choice.
    expect(element(body, 'longevity-bonus').open).not.toMatch(/value="0"/);
    expect(element(body, 'longevity-not-entered').text).toContain('Not entered');
  });

  it('renders the stored bonus and drops the not-entered flag once entered', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(4);
    setDerived('self_made', 4, true, { lab_total: 35, suggested_bonus: 7, halved: false });
    const body = html();
    expect(element(body, 'longevity-bonus').open).toMatch(/value="4"/);
    expect(() => element(body, 'longevity-not-entered')).toThrow();
  });

  it('brings the not-entered marker back when the field is emptied', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityBonus(4);
    setDerived('self_made', 4, true, { lab_total: 35, suggested_bonus: 7, halved: false });
    expect(() => element(html(), 'longevity-not-entered')).toThrow();
    // Emptying the input is how a player takes back a value they mistyped — it must
    // restore "not entered", not write a deliberate 0 that can never be undone.
    store.setLongevityBonus(null);
    setDerived('self_made', 0, false, { lab_total: 35, suggested_bonus: 7, halved: false });
    const body = html();
    expect(element(body, 'longevity-not-entered').text).toContain('Not entered');
    expect(element(body, 'longevity-bonus').open).not.toMatch(/value="0"/);
  });

  it('keeps the bonus input for an external ritual too', () => {
    store.addLongevityRitual('external');
    store.setLongevityBonus(6);
    setDerived('external', 6, true, null);
    expect(element(html(), 'longevity-bonus').open).toMatch(/value="6"/);
  });
});

describe('LongevityPanel hint (engine-authoritative suggestion)', () => {
  it('shows the suggested bonus and the Lab Total it comes from', () => {
    store.addLongevityRitual('self_made');
    setDerived('self_made', 0, false, { lab_total: 35, suggested_bonus: 7, halved: false });
    const { text } = element(html(), 'longevity-hint');
    expect(text).toContain('35');
    // Signed via formatSigned, so a positive bonus reads "+7".
    expect(text).toContain('+7');
    // The editor shows the STORED magnitude (what to type into the field), which the
    // string names — the totals panel shows the same number as an aging-roll
    // modifier (-7), so neither surface can be mistaken for the other.
    expect(text.toLowerCase()).toContain('aging bonus');
  });

  it('marks the hint as halved when the engine flags a halving', () => {
    store.addLongevityRitual('self_made');
    setDerived('self_made', 0, false, { lab_total: 17, suggested_bonus: 4, halved: true });
    const body = html();
    expect(element(body, 'longevity-hint').text).toContain('17');
    expect(element(body, 'longevity-hint-halved').text).toContain('halved');
  });

  it('omits the halved marker when nothing halved the Lab Total', () => {
    store.addLongevityRitual('self_made');
    setDerived('self_made', 0, false, { lab_total: 35, suggested_bonus: 7, halved: false });
    expect(() => element(html(), 'longevity-hint-halved')).toThrow();
  });

  it('renders a negative Lab Total with the ASCII hyphen, never U+2212', () => {
    store.addLongevityRitual('self_made');
    setDerived('self_made', 0, false, { lab_total: -6, suggested_bonus: 0, halved: false });
    const { text } = element(html(), 'longevity-hint');
    expect(text).toContain('-6');
    expect(text).not.toContain('−');
  });

  // S26 (full-audit a11y): the hint text changes reactively as the Lab Total
  // shifts (an Art bought, a Puissant Art added), but was not announced —
  // unlike AgingSchedulePanel's and LivingConditionsPicker's equivalent
  // reactive figures, which both carry role="status".
  it('announces the hint so a screen reader hears it change, matching the other reactive hints', () => {
    store.addLongevityRitual('self_made');
    setDerived('self_made', 0, false, { lab_total: 35, suggested_bonus: 7, halved: false });
    const { open } = element(html(), 'longevity-hint');
    expect(open).toContain('role="status"');
  });

  it('offers no suggestion for an external ritual (another magus made it)', () => {
    store.addLongevityRitual('external');
    setDerived('external', 6, true, null);
    expect(() => element(html(), 'longevity-hint')).toThrow();
  });

  it('renders nothing derived before the engine answers', () => {
    store.addLongevityRitual('self_made');
    expect(() => element(html(), 'longevity-hint')).toThrow();
  });
});

describe('LongevityPanel focus and sterility', () => {
  it('renders the focus input bound to the stored focus', () => {
    store.addLongevityRitual('self_made');
    store.setLongevityFocus('A draught of quicksilver');
    const { open } = element(html(), 'longevity-focus');
    expect(open).toMatch(/<input/i);
    expect(open).toContain('A draught of quicksilver');
  });

  // manual-testing-findings #21: the sterility sentence is a rules consequence the
  // player reads in the book, not state this panel holds, so it is gone. The panel's
  // own inputs are untouched.
  it('notes nothing about sterility', () => {
    store.addLongevityRitual('self_made');
    const body = html();
    expect(() => element(body, 'longevity-sterility-note')).toThrow();
    expect(element(body, 'longevity-focus').open).toMatch(/<input/i);
  });

  it('no longer claims the bonus is computed for a self-made ritual', () => {
    store.addLongevityRitual('self_made');
    expect(() => element(html(), 'longevity-self-made-note')).toThrow();
  });

  it('renders only the add button when there is no ritual', () => {
    const body = html();
    expect(element(body, 'longevity-add').open).toMatch(/<button/i);
    expect(() => element(body, 'longevity-bonus')).toThrow();
    expect(() => element(body, 'longevity-focus')).toThrow();
  });
});
