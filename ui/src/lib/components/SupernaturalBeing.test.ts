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
    saga_year: 1220,
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

  // The budget bar charges for Penetration as well as level (ArMDE:4019,
  // and the engine's `powers_used`), so the row must offer a control for it —
  // otherwise the player is billed for a value nothing on screen can set.
  it('reads a power row Penetration back from the entity, defaulting to 0', () => {
    store.entity.powers = [
      { name: 'Wolf Shape', level: 20, penetration: 20 },
      { name: 'Stormcall', level: 60 },
    ];
    const body = html();
    expect(element(body, 'power-penetration-0').open).toMatch(/value="20"/);
    expect(element(body, 'power-penetration-1').open).toMatch(/value="0"/);
  });

  it("exposes the engine's [0, 65535] range on the Penetration input", () => {
    store.entity.powers = [{ name: 'Wolf Shape', level: 20, penetration: 20 }];
    const open = element(html(), 'power-penetration-0').open;
    expect(open).toContain('min="0"');
    expect(open).toContain('max="65535"');
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

  // Focus Power's 25 points are a SECOND currency (ArMDE:3899) — 2 per level of
  // effect, 1 per Penetration — so the panel draws its own list and its own bar,
  // and neither reads the other's numbers.
  it('renders a focus-power row per entry with its own max-level and Penetration inputs', () => {
    store.entity.focus_powers = [
      { name: 'Wolves of the wood', max_level: 10, penetration: 5 },
      { name: 'Storms', max_level: 5 },
    ];
    const body = html();
    expect(element(body, 'focus-power-name-0').open).toMatch(/value="Wolves of the wood"/);
    expect(element(body, 'focus-power-max-level-0').open).toMatch(/value="10"/);
    expect(element(body, 'focus-power-penetration-0').open).toMatch(/value="5"/);
    // An unspent Penetration reads back as 0, never blank.
    expect(element(body, 'focus-power-penetration-1').open).toMatch(/value="0"/);
    expect(has(body, 'focus-power-remove-0')).toBe(true);
    expect(has(body, 'focus-power-remove-1')).toBe(true);
    expect(has(body, 'focus-power-add')).toBe(true);
  });

  it('reads the focus-points budget/used readout from the engine, never recomputed locally', () => {
    store.entity.focus_powers = [{ name: 'Wolves', max_level: 10, penetration: 5 }];
    store.effective = {
      focus_points_used: 25,
      focus_points_budget: 50,
    } as unknown as EffectiveScores;
    const text = clean(element(html(), 'focus-points-used').text);
    expect(text).toContain('25');
    expect(text).toContain('50');
  });

  // A magus reaches this tab only through Focus Power, and "Magi never have
  // Might" — so the Might block and the level-budget power list, which are a
  // Might-being's, must not be offered to him. The Focus Power section is.
  it('hides the Might and level-budget blocks for a magus, keeping the focus section', () => {
    store.entity.type_id = 'magus';
    store.ruleset!.ruleset.type_profiles.magus = {
      hermetically_trained: true,
    } as unknown as LocalizedRuleset['ruleset']['type_profiles'][string];
    const body = html();
    expect(has(body, 'might-add')).toBe(false);
    expect(has(body, 'power-add')).toBe(false);
    expect(has(body, 'focus-power-add')).toBe(true);
  });

  // Magnitude, Initiative and the Fatigue cost are all engine-derived
  // (ArMDE:3899, ArMDE:3901, ArMDE:9097) and only displayed here.
  it('shows the engine-derived magnitude, Initiative and Fatigue cost per focus power', () => {
    store.entity.focus_powers = [{ name: 'Wolves', max_level: 10, penetration: 5 }];
    store.derived = {
      focus_powers: [
        {
          name: 'Wolves',
          max_level: 10,
          penetration: 5,
          magnitude: 2,
          initiative: -1,
          fatigue_levels: 1,
        },
      ],
    } as unknown as DerivedTotals;
    const text = clean(element(html(), 'focus-power-derived-0').text);
    expect(text).toContain('2');
    // ASCII hyphen, never U+2212.
    expect(text).toContain('-1');
    expect(text).not.toContain('−');
  });
});
