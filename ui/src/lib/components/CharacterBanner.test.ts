import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, EntityTypeProfile, LocalizedRuleset } from '../types';

// Reads the store singleton only (ruleset profile + Fluent bundle); the store's
// actions cross the Tauri bridge, so mock it away.
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
import CharacterBanner from './CharacterBanner.svelte';

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
    ability_funding: 'pool',
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
    creation_phases: ['experience'],
  };
}

function text(body: string, testid: string): string {
  const whole = new RegExp(`<(\\w+)[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</\\1>`, 'i').exec(
    body,
  );
  if (!whole) throw new Error(`no element with data-testid="${testid}"`);
  // Fluent wraps interpolated values in bidi isolation marks; strip them.
  return whole[2]
    .replace(/<[^>]*>/g, ' ')
    .replace(/[⁨⁩]/g, '')
    .replace(/\s+/g, ' ')
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

// Slice 2 (#1) deleted the read-only `type` wizard step. Its two facts that live
// nowhere else — the Virtue/Flaw budget numbers and the Gift policy line — moved
// here, to the banner both the editor and the wizard already show above every
// screen, so they stay reachable throughout the whole flow. These assertions are
// TypeStep.test.ts's, followed to the content's new home.
describe('CharacterBanner states what the character type commits the character to', () => {
  it('names the type through its Fluent key, never the raw slug', () => {
    const label = text(render(CharacterBanner).body, 'character-type');
    expect(label).toContain('Magus');
    expect(label).not.toContain('type-magus');
  });

  it("states the profile's Virtue and Flaw budget", () => {
    expect(text(render(CharacterBanner).body, 'character-type-budget')).toContain('10');
  });

  it('reads the budget from the profile rather than assuming the magus numbers', () => {
    installProfile({ ...magus(), id: 'grog', budget: { virtue_points: 3, flaw_points: 3 } });
    const budget = text(render(CharacterBanner).body, 'character-type-budget');
    expect(budget).toContain('3');
    expect(budget).not.toContain('10');
  });

  // Gated on the profile's Gift policy, never on the type id — a new Gifted type
  // gets the right line with no code change.
  it('states that this type requires The Gift', () => {
    const body = render(CharacterBanner).body;
    expect(body).toContain('data-testid="character-type-gift-required"');
    expect(body).not.toContain('data-testid="character-type-gift-forbidden"');
  });

  it('states that a companion may not have The Gift', () => {
    installProfile({ ...magus(), id: 'companion', is_magus: false, gift_policy: 'forbidden' });
    const body = render(CharacterBanner).body;
    expect(body).toContain('data-testid="character-type-gift-forbidden"');
    expect(body).not.toContain('data-testid="character-type-gift-required"');
  });

  it('says nothing about The Gift when the profile has no policy', () => {
    const profile = magus();
    delete profile.gift_policy;
    installProfile(profile);
    const body = render(CharacterBanner).body;
    expect(body).not.toContain('data-testid="character-type-gift-required"');
    expect(body).not.toContain('data-testid="character-type-gift-forbidden"');
    expect(body).not.toContain('data-testid="character-type-gift-optional"');
  });

  it('says the type is fixed, and says nothing at all without a profile', () => {
    expect(render(CharacterBanner).body).toContain('data-testid="character-type-explainer"');
    installProfile({ ...magus(), id: 'grog' });
    store.entity.type_id = 'sorcerer';
    const body = render(CharacterBanner).body;
    // No profile, no budget and no Gift claim: the numbers would be invented.
    expect(body).not.toContain('data-testid="character-type-budget"');
    expect(body).not.toContain('data-testid="character-type-gift-required"');
  });

  it('localizes to German, rendering prose rather than an echoed key', () => {
    store.lang = 'de';
    const body = render(CharacterBanner).body;
    expect(text(body, 'character-type')).toContain('Magus');
    expect(body).not.toContain('character-type-explainer =');
    expect(text(body, 'character-type-explainer')).not.toContain('banner-type-explainer');
  });
});
