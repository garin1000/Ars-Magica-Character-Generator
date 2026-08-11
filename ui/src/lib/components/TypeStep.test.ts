import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, EntityTypeProfile, LocalizedRuleset } from '../types';

// Reads the store singleton only (ruleset profile + Fluent bundle); the store's
// actions cross the Tauri bridge, so mock it away. Harness mirrors
// StartScreen.test.ts.
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
import TypeStep from './TypeStep.svelte';

function installProfile(profile: EntityTypeProfile): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: { [profile.id]: profile },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: profile.id,
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

function magus(): EntityTypeProfile {
  return {
    id: 'magus',
    budget: { virtue_points: 10, flaw_points: 10 },
    is_magus: true,
    gift_policy: 'required',
    gift_id: 'virtue.the_gift',
    creation_phases: ['type'],
  };
}

function text(body: string, testid: string): string {
  const whole = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i').exec(body);
  if (!whole) throw new Error(`no element with data-testid="${testid}"`);
  // Fluent wraps interpolated values in bidi isolation marks; strip them.
  return whole[1]
    .replace(/<[^>]*>/g, '')
    .replace(/[⁨⁩]/g, '')
    .trim();
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installProfile(magus());
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

describe('TypeStep', () => {
  it('names the type through its Fluent key, never the raw slug', () => {
    const label = text(render(TypeStep).body, 'type-step-name');
    expect(label).toBe('Magus');
    expect(label).not.toBe('magus');
    expect(label).not.toBe('type-magus');
  });

  it("states the profile's Virtue and Flaw budget", () => {
    const budget = text(render(TypeStep).body, 'type-step-budget');
    expect(budget).toContain('10');
  });

  it('reads the budget from the profile rather than assuming the magus numbers', () => {
    installProfile({ ...magus(), id: 'grog', budget: { virtue_points: 3, flaw_points: 3 } });
    expect(text(render(TypeStep).body, 'type-step-budget')).toContain('3');
  });

  // Gated on the profile's Gift policy, never on the type id — a new Gifted type
  // gets the right line with no code change.
  it('states that this type requires The Gift', () => {
    const body = render(TypeStep).body;
    expect(body).toContain('data-testid="type-step-gift-required"');
    expect(body).not.toContain('data-testid="type-step-gift-forbidden"');
  });

  it('states that a companion may not have The Gift', () => {
    installProfile({ ...magus(), id: 'companion', is_magus: false, gift_policy: 'forbidden' });
    const body = render(TypeStep).body;
    expect(body).toContain('data-testid="type-step-gift-forbidden"');
    expect(body).not.toContain('data-testid="type-step-gift-required"');
  });

  it('says nothing about The Gift when the profile has no policy', () => {
    const profile = magus();
    delete profile.gift_policy;
    installProfile(profile);
    const body = render(TypeStep).body;
    expect(body).not.toContain('data-testid="type-step-gift-required"');
    expect(body).not.toContain('data-testid="type-step-gift-forbidden"');
    expect(body).not.toContain('data-testid="type-step-gift-optional"');
  });

  it('localizes to German', () => {
    store.lang = 'de';
    const body = render(TypeStep).body;
    expect(text(body, 'type-step-name')).toBe('Magus');
    // The explainer is real German prose, not an echoed key.
    expect(body).not.toContain('phase-type-explainer');
  });
});
