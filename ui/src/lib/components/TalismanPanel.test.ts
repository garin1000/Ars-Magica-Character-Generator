import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, Entity, LocalizedRuleset, TalismanCapacity } from '../types';

// The talisman panel reads the shared store singleton (the entity's stored talisman
// plus the engine's capacity read-out) and the Fluent bundle. The store schedules a
// debounced revalidate over the Tauri IPC bridge; mock the bridge so nothing reaches
// a backend. Harness mirrors LongevityPanel.test.ts.
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
import TalismanPanel from './TalismanPanel.svelte';

/** A minimal localized ruleset: the panel needs no catalogue, only the bundle and
 * the rules-i18n names of the two Arts the capacity note spells out. */
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
    i18n: { 'art.creo': { name: 'Creo' }, 'art.corpus': { name: 'Corpus' } },
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

/** Install the engine's capacity read-out, the panel's only derived input. */
function setCapacity(capacity: TalismanCapacity | null): void {
  store.derived = { talisman_capacity: capacity } as unknown as DerivedTotals;
}

/** Render the panel to an HTML string (node env, no DOM). */
function html(): string {
  return render(TalismanPanel, { props: {} }).body;
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

describe('TalismanPanel item identity', () => {
  it('renders only the add button when there is no talisman', () => {
    const body = html();
    expect(element(body, 'talisman-add').open).toMatch(/<button/i);
    expect(() => element(body, 'talisman-description')).toThrow();
    expect(() => element(body, 'talisman-list')).toThrow();
    expect(() => element(body, 'talisman-effect-list')).toThrow();
  });

  it('renders the identity input bound to the stored description', () => {
    store.addTalisman();
    store.setTalismanDescription('An ash staff shod with silver');
    const { open } = element(html(), 'talisman-description');
    expect(open).toMatch(/<input/i);
    expect(open).toContain('An ash staff shod with silver');
  });

  it('offers removal once a talisman exists', () => {
    store.addTalisman();
    expect(element(html(), 'talisman-remove-item').open).toMatch(/<button/i);
  });
});

describe('TalismanPanel attunements', () => {
  it('renders an empty row when the talisman has no attunements', () => {
    store.addTalisman();
    expect(element(html(), 'talisman-list').text).toContain('No talisman attunements yet');
  });

  it('renders a stored attunement descriptor and bonus', () => {
    store.addTalisman();
    store.addTalismanAttunement();
    store.setTalismanAttunementDescription(0, 'Controlling things at a distance');
    store.setTalismanAttunementBonus(0, 4);
    const body = html();
    expect(element(body, 'talisman-desc-0').open).toContain('Controlling things at a distance');
    expect(element(body, 'talisman-bonus-0').open).toMatch(/value="4"/);
    expect(element(body, 'talisman-remove-0').open).toMatch(/<button/i);
  });

  it('renders a negative attunement bonus with the ASCII hyphen, never U+2212', () => {
    store.addTalisman();
    store.addTalismanAttunement();
    store.setTalismanAttunementBonus(0, -2);
    const { open } = element(html(), 'talisman-bonus-0');
    expect(open).toMatch(/value="-2"/);
    expect(open).not.toContain('−');
  });
});

describe('TalismanPanel instilled effects', () => {
  it('renders an empty row when nothing is instilled', () => {
    store.addTalisman();
    expect(element(html(), 'talisman-effect-list').text).toContain('No instilled effects yet');
  });

  it('renders a stored effect name and level', () => {
    store.addTalisman();
    store.addTalismanEffect();
    store.setTalismanEffectName(0, 'Wielding the Invisible Sling');
    store.setTalismanEffectLevel(0, 15);
    const body = html();
    expect(element(body, 'talisman-effect-name-0').open).toContain('Wielding the Invisible Sling');
    expect(element(body, 'talisman-effect-level-0').open).toMatch(/value="15"/);
    expect(element(body, 'talisman-effect-remove-0').open).toMatch(/<button/i);
  });
});

describe('TalismanPanel capacity read-out (engine-authoritative)', () => {
  it('shows the capacity in pawns and the two Arts it comes from', () => {
    store.addTalisman();
    setCapacity({
      technique: 'art.creo',
      form: 'art.corpus',
      technique_score: 10,
      form_score: 12,
      pawns: 22,
    });
    const body = html();
    expect(element(body, 'talisman-capacity').text).toContain('22');
    // The note spells out the derivation, so both contributing scores must render —
    // otherwise technique_score/form_score would be dead fields on the payload.
    const note = element(body, 'talisman-capacity-note').text;
    expect(note).toContain('10');
    expect(note).toContain('12');
    // …and both Arts must be NAMED, through the rules-i18n lookup: a bare
    // "Technique 10 + Form 12" cannot be checked against a character sheet, and a
    // raw `art.creo` slug as a label is never acceptable.
    expect(note).toContain('Creo');
    expect(note).toContain('Corpus');
    expect(note).not.toContain('art.');
  });

  it('pluralizes the pawns through a Fluent selector, not a hardcoded "(s)"', () => {
    store.addTalisman();
    setCapacity({
      technique: 'art.creo',
      form: 'art.corpus',
      technique_score: 1,
      form_score: 0,
      pawns: 1,
    });
    // A stringified argument cannot drive a plural selector, so the call site has to
    // pass the number — the singular is the proof it does. (Fluent wraps the
    // interpolated number in bidi isolation marks, so the noun is matched alone.)
    expect(element(html(), 'talisman-capacity').text).toContain('pawn of');
    expect(element(html(), 'talisman-capacity').text).not.toContain('pawns of');
    setCapacity({
      technique: 'art.creo',
      form: 'art.corpus',
      technique_score: 10,
      form_score: 12,
      pawns: 22,
    });
    expect(element(html(), 'talisman-capacity').text).toContain('pawns of');
  });

  it('renders no capacity before the engine answers', () => {
    store.addTalisman();
    expect(() => element(html(), 'talisman-capacity')).toThrow();
  });

  it('renders no capacity without a talisman even if one is in the payload', () => {
    setCapacity({
      technique: 'art.creo',
      form: 'art.corpus',
      technique_score: 10,
      form_score: 12,
      pawns: 22,
    });
    expect(() => element(html(), 'talisman-capacity')).toThrow();
  });
});
