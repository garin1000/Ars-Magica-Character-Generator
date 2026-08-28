import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, Entity, LocalizedRuleset } from '../types';

// The panel reads the shared store singleton (entity + engine-derived totals)
// and the Fluent bundle. The store schedules a debounced revalidate over the
// Tauri IPC bridge; mock the bridge so nothing reaches a backend. Mirrors
// SpellBudgetBar.test.ts / XpBar.test.ts.
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
import DerivedTotalsPanel from './DerivedTotalsPanel.svelte';

function installRuleset(overrides: Partial<LocalizedRuleset['ruleset']> = {}): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      // Round 3, Task 3: the engine-surfaced aura bound the input reads instead
      // of a hardcoded -50/10.
      aura_modifier_min: -50,
      aura_modifier_max: 10,
      ...overrides,
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: 11,
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
  };
}

/** A minimal, complete DerivedTotals fixture; overrides layer on top. */
function derivedFixture(overrides: Partial<DerivedTotals> = {}): DerivedTotals {
  return {
    is_magus: true,
    lab_totals: [],
    casting_totals: [],
    penetration: [],
    magic_resistance: [],
    combat: [],
    soak: { addends: [], total: 0 },
    encumbrance: { load: 0, burden: 0, total: 0 },
    fatigue: [],
    wounds: [],
    size: 0,
    decrepitude_score: 0,
    warping_score: 0,
    warping_points: 0,
    surfaced_modifiers: [],
    ...overrides,
  };
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(DerivedTotalsPanel, { props: {} }).body;
}

/** The text content of the single element carrying `testid`, tags stripped. */
function textOf(body: string, testid: string): string {
  const match = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i').exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[1]
    .replace(/<[^>]*>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('DerivedTotalsPanel longevity read-out', () => {
  // E4 (round-1 audit): the panel negates the stored bonus into an aging-roll
  // modifier and documents that a zero bonus must read "0", never "-0" — this
  // is the load-bearing call site formatSigned(-0) exists to protect.
  it('shows a zero longevity bonus as "0", never "-0"', () => {
    store.derived = derivedFixture({
      longevity: { source: 'self_made', bonus: 0, entered: true, bronze_cord: 0 },
    });
    const text = textOf(html(), 'derived-longevity');
    expect(text).toContain('0');
    expect(text).not.toContain('-0');
  });

  it('negates a positive stored bonus into a negative aging-roll modifier with an ASCII hyphen', () => {
    store.derived = derivedFixture({
      longevity: { source: 'self_made', bonus: 10, entered: true, bronze_cord: 0 },
    });
    const text = textOf(html(), 'derived-longevity');
    expect(text).toContain('-10');
    // ASCII hyphen-minus (U+002D), never the mathematical minus (U+2212).
    expect(text).not.toContain('−');
  });

  it('shows the not-entered hint when no ritual has been recorded yet', () => {
    store.derived = derivedFixture({
      longevity: { source: 'external', bonus: 0, entered: false, bronze_cord: 0 },
    });
    const text = textOf(html(), 'derived-longevity');
    expect(text).toContain(store.t('derived-longevity-not-entered'));
  });
});

describe('DerivedTotalsPanel loading state', () => {
  it('shows a loading placeholder before the engine totals arrive', () => {
    store.derived = null;
    const body = html();
    expect(body).toContain(store.t('loading'));
  });
});

describe('DerivedTotalsPanel aura bound (round 3, Task 3)', () => {
  function auraInputTag(): string {
    const body = html();
    const match = /<[^>]*data-testid="derived-aura-input"[^>]*>/i.exec(body);
    if (!match) throw new Error('no element with data-testid="derived-aura-input"');
    return match[0];
  }

  // The input used to carry min="-2147483648" max="2147483647" — the full i32
  // range — instead of the engine's actual rules bound, so the browser never
  // hinted at the real range and an out-of-range entry was silently rewritten
  // only later, at save.
  it('bounds the aura input to the engine-surfaced rules range, not the raw i32 range', () => {
    store.derived = derivedFixture({ is_magus: true });
    const input = auraInputTag();
    expect(input).toContain('min="-50"');
    expect(input).toContain('max="10"');
    expect(input).not.toContain('-2147483648');
    expect(input).not.toContain('2147483647');
  });

  it('reads the bound from whatever the ruleset payload carries, not a hardcoded pair', () => {
    installRuleset({ aura_modifier_min: -7, aura_modifier_max: 4 });
    store.derived = derivedFixture({ is_magus: true });
    const input = auraInputTag();
    expect(input).toContain('min="-7"');
    expect(input).toContain('max="4"');
  });

  it('shows no out-of-range hint for a legal aura value', () => {
    store.entity.aura = 3;
    store.derived = derivedFixture({ is_magus: true });
    const body = html();
    expect(body).not.toContain('data-testid="derived-aura-out-of-range"');
  });

  it('shows an out-of-range hint when the stored aura is beyond the engine bound', () => {
    // A hand-edited or stale save can carry an out-of-range aura before the next
    // save re-normalizes it (Entity::normalize is not called on load).
    store.entity.aura = 999;
    store.derived = derivedFixture({ is_magus: true });
    const body = html();
    expect(body).toContain('data-testid="derived-aura-out-of-range"');
  });

  // Round 4, S2: the hint used to be visually adjacent to the aura input with no
  // programmatic association, so a screen-reader user tabbing to the input never
  // learned their value was out of range.
  it('associates the out-of-range hint with the aura input via aria-describedby', () => {
    store.entity.aura = 999;
    store.derived = derivedFixture({ is_magus: true });
    const input = auraInputTag();
    expect(input).toContain('aria-describedby="derived-aura-out-of-range"');
  });

  it('does not describe the input when the aura is in range', () => {
    store.entity.aura = 3;
    store.derived = derivedFixture({ is_magus: true });
    const input = auraInputTag();
    expect(input).not.toContain('aria-describedby');
  });

  it('announces the hint as a polite live region, not an interrupting one', () => {
    // role="status" is implicitly aria-live="polite" — the hint fires while the
    // player is still typing, so an assertive region would talk over them.
    store.entity.aura = 999;
    store.derived = derivedFixture({ is_magus: true });
    const body = html();
    const match = /<[^>]*data-testid="derived-aura-out-of-range"[^>]*>/i.exec(body);
    if (!match) throw new Error('no element with data-testid="derived-aura-out-of-range"');
    expect(match[0]).toContain('role="status"');
  });
});

describe('DerivedTotalsPanel Weak Enchanter lab-total read-out (round 3, G2)', () => {
  /**
   * `textOf` stops at the FIRST nested closing tag, so it only captures the
   * `<dl data-testid="derived-lab-total">`'s first child (the `<dt>` label) —
   * fine for the single-child longevity paragraph it was written for, wrong
   * here where the `dl` has several `<dt>`/`<dd>` children. Pull the whole
   * `dl`'s inner markup instead (safe: this `dl` never nests another `dl`).
   */
  function labTotalDl(body: string): string {
    const match = /<dl[^>]*data-testid="derived-lab-total"[^>]*>([\s\S]*?)<\/dl>/i.exec(body);
    if (!match) throw new Error('no derived-lab-total dl found');
    return match[1];
  }

  // G2: LabTotal.enchanting was computed engine-side but never reached the
  // frontend at all — the TS type omitted it and the panel never read it, so a
  // Weak Enchanter magus saw no mechanical effect of the Flaw anywhere.
  it('shows the enchanting figure only when it differs from the plain total', () => {
    store.derived = derivedFixture({
      is_magus: true,
      lab_totals: [
        {
          technique: 'art.creo',
          form: 'art.corpus',
          addends: [],
          total: 16,
          deficient: false,
          enchanting: 8,
        },
      ],
    });
    const dl = labTotalDl(html());
    expect(dl).toContain('>16<');
    expect(dl).toContain('>8<');
  });

  it('hides the enchanting row for a magus without Weak Enchanter (enchanting === total)', () => {
    store.derived = derivedFixture({
      is_magus: true,
      lab_totals: [
        {
          technique: 'art.creo',
          form: 'art.corpus',
          addends: [],
          total: 16,
          deficient: false,
          enchanting: 16,
        },
      ],
    });
    const body = html();
    expect(body).not.toContain('derived-lab-enchanting');
  });
});
