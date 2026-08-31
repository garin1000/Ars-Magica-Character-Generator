import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

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
import PersonalityTraits from './PersonalityTraits.svelte';

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
  };
}

function html(): string {
  return render(PersonalityTraits, { props: {} }).body;
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// S29 (full-audit UX): a Personality Trait's value was editable ONLY through the
// +/- spinner buttons — every other scored control that carries a wide or signed
// range (aging points, Talisman bonus/level) offers direct numeric entry too. A
// value up to |6| (Personality Flaw, Core Rules) took up to twelve clicks to reach
// from 0 with no other way in.
describe('PersonalityTraits value entry (S29)', () => {
  it('renders a bound number input for the trait value, not just the spinner', () => {
    store.addPersonalityTrait();
    store.setPersonalityTraitName(0, 'Brave');
    const body = html();
    const input = /<input[^>]*data-testid="personality-value-0"[^>]*>/.exec(body);
    expect(input).not.toBeNull();
    expect(input![0]).toMatch(/type="number"/);
  });

  it('reads the stored value back into the input', () => {
    store.addPersonalityTrait();
    store.setPersonalityTraitValue(0, 3);
    const input = /<input[^>]*data-testid="personality-value-0"[^>]*>/.exec(html());
    expect(input![0]).toContain('value="3"');
  });

  it('renders a negative stored value with the ASCII hyphen, never U+2212', () => {
    store.addPersonalityTrait();
    store.setPersonalityTraitValue(0, -4);
    const input = /<input[^>]*data-testid="personality-value-0"[^>]*>/.exec(html());
    expect(input![0]).toContain('value="-4"');
    expect(input![0]).not.toContain('−');
  });

  it('bounds the input to the |6| range the store clamps to', () => {
    store.addPersonalityTrait();
    const input = /<input[^>]*data-testid="personality-value-0"[^>]*>/.exec(html());
    expect(input![0]).toContain('min="-6"');
    expect(input![0]).toContain('max="6"');
  });

  it('gives the value input an accessible name naming the trait', () => {
    store.addPersonalityTrait();
    store.setPersonalityTraitName(0, 'Brave');
    const input = /<input[^>]*data-testid="personality-value-0"[^>]*>/.exec(html());
    expect(input![0]).toMatch(/aria-label="[^"]*Brave[^"]*"/);
  });

  it('keeps the spinner buttons alongside the new input', () => {
    store.addPersonalityTrait();
    const body = html();
    expect(body).toContain('data-testid="personality-dec-0"');
    expect(body).toContain('data-testid="personality-inc-0"');
    expect(body).toContain('data-testid="personality-value-0"');
  });
});
