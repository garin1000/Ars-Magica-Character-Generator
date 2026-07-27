import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// The familiar panel reads the shared store singleton (the entity's stored familiar
// statblock) and the Fluent bundle. The store schedules a debounced revalidate over
// the Tauri IPC bridge; mock the bridge so nothing reaches a backend. Harness
// mirrors TalismanPanel.test.ts.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import FamiliarPanel from './FamiliarPanel.svelte';

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
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.derived = null;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(FamiliarPanel, { props: {} }).body;
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

describe('FamiliarPanel identity', () => {
  it('renders only the add button when there is no familiar', () => {
    const body = html();
    expect(element(body, 'familiar-add').open).toMatch(/<button/i);
    expect(() => element(body, 'familiar-name')).toThrow();
    expect(() => element(body, 'familiar-animal')).toThrow();
    expect(() => element(body, 'familiar-size')).toThrow();
    expect(() => element(body, 'familiar-power-list')).toThrow();
  });

  it('renders the name and animal inputs bound to the statblock', () => {
    store.addFamiliar();
    store.setFamiliarName('Corax');
    store.setFamiliarAnimal('raven');
    const body = html();
    expect(element(body, 'familiar-name').open).toContain('Corax');
    expect(element(body, 'familiar-animal').open).toContain('raven');
    expect(element(body, 'familiar-remove').open).toMatch(/<button/i);
  });

  it('renders a negative Size with the ASCII hyphen, never U+2212', () => {
    store.addFamiliar();
    store.setFamiliarSize(-4);
    const { open } = element(html(), 'familiar-size');
    expect(open).toMatch(/value="-4"/);
    expect(open).not.toContain('−');
  });
});

describe('FamiliarPanel Magic Might', () => {
  it('offers to add a Might and says none is entered', () => {
    store.addFamiliar();
    const body = html();
    expect(element(body, 'familiar-might-add').open).toMatch(/<button/i);
    expect(element(body, 'familiar-might-empty').text).toContain('No Magic Might');
    expect(() => element(body, 'familiar-might-score')).toThrow();
  });

  it("renders the familiar's own Realm and Might Score once entered", () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('magic');
    store.setFamiliarMightScore(10);
    const body = html();
    expect(element(body, 'familiar-might-realm').open).toMatch(/<select/i);
    expect(element(body, 'familiar-might-score').open).toMatch(/value="10"/);
    expect(element(body, 'familiar-might-clear').open).toMatch(/<button/i);
  });

  it('labels the score with its own key, not the character\'s "Base Might Score"', () => {
    store.addFamiliar();
    store.setFamiliarMightRealm('magic');
    const body = html();
    // The familiar's Might takes no Virtue grants on top, so "Base" would be a lie.
    expect(body).not.toContain('Base Might Score');
  });
});

describe('FamiliarPanel Characteristics', () => {
  it('renders one signed input per Characteristic, plus the point-buy note', () => {
    store.addFamiliar();
    store.setFamiliarCharacteristic('int', -3);
    const body = html();
    expect(element(body, 'familiar-char-int').open).toMatch(/value="-3"/);
    expect(element(body, 'familiar-char-qik').open).toMatch(/value="0"/);
    // All eight are offered, not only the entered ones.
    for (const c of ['int', 'per', 'str', 'sta', 'pre', 'com', 'dex', 'qik']) {
      expect(() => element(body, `familiar-char-${c}`)).not.toThrow();
    }
    expect(element(body, 'familiar-characteristics-note').text).toContain('Characteristic points');
  });
});

describe('FamiliarPanel Personality Traits', () => {
  it('renders an empty row when the familiar has none', () => {
    store.addFamiliar();
    expect(element(html(), 'familiar-personality-list').text).toContain('No Personality Traits');
  });

  it('renders a stored trait name and its signed value', () => {
    store.addFamiliar();
    store.addFamiliarPersonalityTrait();
    store.setFamiliarPersonalityTraitName(0, 'Loyal (Marcus)');
    store.setFamiliarPersonalityTraitValue(0, 3);
    const body = html();
    expect(element(body, 'familiar-personality-name-0').open).toContain('Loyal (Marcus)');
    expect(element(body, 'familiar-personality-value-0').text).toBe('+3');
    expect(element(body, 'familiar-personality-remove-0').open).toMatch(/<button/i);
  });

  it('renders a negative trait value with the ASCII hyphen', () => {
    store.addFamiliar();
    store.addFamiliarPersonalityTrait();
    store.setFamiliarPersonalityTraitValue(0, -2);
    const { text } = element(html(), 'familiar-personality-value-0');
    expect(text).toBe('-2');
    expect(text).not.toContain('−');
  });
});

describe('FamiliarPanel invested powers', () => {
  it('renders an empty row when nothing is invested', () => {
    store.addFamiliar();
    expect(element(html(), 'familiar-power-list').text).toContain('No supernatural powers yet');
  });

  it('renders a stored power name and level', () => {
    store.addFamiliar();
    store.addFamiliarPower();
    store.setFamiliarPowerName(0, 'Mental communication');
    store.setFamiliarPowerLevel(0, 15);
    const body = html();
    expect(element(body, 'familiar-power-name-0').open).toContain('Mental communication');
    expect(element(body, 'familiar-power-level-0').open).toMatch(/value="15"/);
    expect(element(body, 'familiar-power-remove-0').open).toMatch(/<button/i);
  });

  it('renders NO power-levels budget bar — there is no limit (Core:10866)', () => {
    store.addFamiliar();
    store.addFamiliarPower();
    store.setFamiliarPowerLevel(0, 400);
    const body = html();
    // The character's own powers get a `power-levels-used` budget read-out; the
    // familiar bond must not, or the UI would invent a limit the rules deny.
    expect(() => element(body, 'power-levels-used')).toThrow();
    expect(body).not.toContain('Power levels:');
    expect(element(body, 'familiar-powers-note').text).toContain('no limit');
  });
});

describe('FamiliarPanel bond note', () => {
  it('surfaces True Friend / Loyal (partner) +3 / Int -3 without applying them', () => {
    store.addFamiliar();
    const note = element(html(), 'familiar-bond-note').text;
    expect(note).toContain('True Friend');
    expect(note).toContain('Loyal (partner) +3');
    expect(note).toContain('Intelligence -3');
    expect(note).not.toContain('−');
    // Surfaced only: nothing is written into the entity's selections or traits.
    expect(store.entity.selections).toEqual([]);
    expect(store.entity.personality_traits ?? []).toEqual([]);
    expect(store.entity.familiar?.personality_traits ?? []).toEqual([]);
  });
});
