import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, Entity, EffectiveScores, LocalizedRuleset } from '../types';

// The tab reads the shared store singleton (entity Might/powers, engine-derived
// effective Might/power budget, and the derived Magic Resistance read-out) and
// the Fluent bundle. The store schedules a debounced revalidate over the Tauri
// IPC bridge; mock the bridge so nothing reaches a backend. Harness mirrors
// AbilityTab.test.ts / FamiliarPanel.test.ts.
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
import SupernaturalBeing from './SupernaturalBeing.svelte';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
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
    type_id: 'mythic_companion',
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
  store.effective = null;
  store.derived = null;
}

function html(): string {
  return render(SupernaturalBeing, { props: {} }).body;
}

/** The single element carrying a given data-testid, its open tag and text. */
function element(body: string, testid: string): { open: string; text: string } {
  const re = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i');
  const match = re.exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  const openMatch = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  return { open: openMatch![0], text: match[1].replace(/<[^>]*>/g, '').trim() };
}

function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

/** Fluent isolates interpolated values with bidi marks; strip them for text matching. */
function clean(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// E1 (full-audit test-adequacy, HIGH): SupernaturalBeing had zero coverage at
// any layer — no vitest file and no e2e spec ever visited the `supernatural`
// tab. This covers the Might empty/set states, the power-level/score numeric
// inputs Erika flagged as the risk area (their clamp range and read-back), and
// the engine-authoritative effective-Might/MR readouts.
describe('SupernaturalBeing Might state', () => {
  it('renders the empty-state message with an Add control when the entity has no Might', () => {
    const body = html();
    expect(body).toContain(store.t('might-empty'));
    expect(has(body, 'might-add')).toBe(true);
    expect(has(body, 'might-realm')).toBe(false);
    expect(has(body, 'might-score')).toBe(false);
    expect(has(body, 'might-clear')).toBe(false);
  });

  it('renders realm/score/clear once the entity has a Might Score', () => {
    store.entity.might = { realm: 'faerie', score: 12 };
    const body = html();
    expect(body).not.toContain(store.t('might-empty'));
    expect(has(body, 'might-add')).toBe(false);
    expect(element(body, 'might-score').open).toMatch(/value="12"/);
    // The Realm select's chosen option carries `selected` (Svelte SSR marks the
    // chosen <option>, not a `value` attribute on the <select> itself).
    const realmSelect = /<select[^>]*data-testid="might-realm"[\s\S]*?<\/select>/.exec(body)![0];
    expect(realmSelect).toMatch(/<option value="faerie" selected/);
  });

  it("exposes the engine's [0, 255] Might Score range on the number input", () => {
    store.entity.might = { realm: 'magic', score: 0 };
    const open = element(html(), 'might-score').open;
    expect(open).toContain('min="0"');
    expect(open).toContain('max="255"');
  });

  it('renders no effective-Might or MR readout while unset', () => {
    store.entity.might = { realm: 'magic', score: 5 };
    // No `store.effective`/`store.derived` installed — the engine has not
    // reported anything to echo yet.
    const body = html();
    expect(has(body, 'might-effective')).toBe(false);
    expect(has(body, 'might-mr')).toBe(false);
  });

  it('renders the effective Might read-out only once the engine reports it', () => {
    store.entity.might = { realm: 'infernal', score: 5 };
    store.effective = {
      might: { realm: 'infernal', score: 8 },
    } as unknown as EffectiveScores;
    const body = html();
    expect(has(body, 'might-effective')).toBe(true);
    expect(clean(element(body, 'might-effective').text)).toContain('8');
  });

  it('renders the Magic Resistance read-out only alongside an effective Might', () => {
    store.entity.might = { realm: 'divine', score: 5 };
    store.effective = { might: { realm: 'divine', score: 5 } } as unknown as EffectiveScores;
    store.derived = {
      magic_resistance: [{ form: 'ignem', total: 15 }],
    } as unknown as DerivedTotals;
    const body = html();
    expect(has(body, 'might-mr')).toBe(true);
    expect(clean(element(body, 'might-mr').text)).toContain('15');
  });

  it('never shows the MR read-out when the engine has not derived one, even with an effective Might', () => {
    store.entity.might = { realm: 'magic', score: 5 };
    store.effective = { might: { realm: 'magic', score: 5 } } as unknown as EffectiveScores;
    store.derived = { magic_resistance: [] } as unknown as DerivedTotals;
    expect(has(html(), 'might-mr')).toBe(false);
  });
});

describe('SupernaturalBeing power list', () => {
  it('renders the empty-powers fallback row when the entity has none', () => {
    const body = html();
    expect(has(body, 'power-list')).toBe(true);
    expect(body).toContain(store.t('powers-empty'));
    expect(has(body, 'power-name-0')).toBe(false);
  });

  it('reads a power row name and level back from the entity', () => {
    store.entity.powers = [{ name: 'Fangs of the Beast', level: 20 }];
    const body = html();
    expect(element(body, 'power-name-0').open).toMatch(/value="Fangs of the Beast"/);
    expect(element(body, 'power-level-0').open).toMatch(/value="20"/);
  });

  it("exposes the engine's [0, 65535] power-level range on the number input", () => {
    store.entity.powers = [{ name: 'Fangs of the Beast', level: 20 }];
    const open = element(html(), 'power-level-0').open;
    expect(open).toContain('min="0"');
    expect(open).toContain('max="65535"');
  });

  it('renders one row per power, keyed by index, with its own remove button', () => {
    store.entity.powers = [
      { name: 'Fangs of the Beast', level: 20 },
      { name: 'Fear', level: 5 },
    ];
    const body = html();
    expect(element(body, 'power-name-0').open).toMatch(/value="Fangs of the Beast"/);
    expect(element(body, 'power-name-1').open).toMatch(/value="Fear"/);
    expect(has(body, 'power-remove-0')).toBe(true);
    expect(has(body, 'power-remove-1')).toBe(true);
  });

  it('reads the power-levels budget/used readout from the engine, never recomputed locally', () => {
    store.entity.powers = [{ name: 'Fangs of the Beast', level: 20 }];
    store.effective = {
      power_levels_used: 20,
      power_levels_budget: 50,
    } as unknown as EffectiveScores;
    const text = clean(element(html(), 'power-levels-used').text);
    expect(text).toContain('20');
    expect(text).toContain('50');
  });
});
