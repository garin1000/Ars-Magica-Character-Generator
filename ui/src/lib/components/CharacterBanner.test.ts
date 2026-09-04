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

// manual-testing-findings #3: the banner names the character's type and nothing
// else. The Virtue/Flaw budget sentence and the Gift-policy line were relocated here
// by Slice 2 (#1) from the deleted `type` step; they explained the rules rather than
// showing the character, and this banner sits above every tab and every wizard step,
// so what it does not say is height the surfaces below get back. The budget numbers
// remain on the Virtues & Flaws balance bar, where they are acted on.
describe('CharacterBanner names the character type and nothing else', () => {
  it('names the type through its Fluent key, never the raw slug', () => {
    const label = text(render(CharacterBanner).body, 'character-type');
    expect(label).toContain('Magus');
    expect(label).not.toContain('type-magus');
  });

  it('carries no budget, explainer or Gift-policy line, for any Gift policy', () => {
    const removed = [
      'character-type-explainer',
      'character-type-budget',
      'character-type-gift-required',
      'character-type-gift-forbidden',
      'character-type-gift-optional',
    ];
    for (const policy of ['required', 'forbidden', 'allowed'] as const) {
      installProfile({ ...magus(), gift_policy: policy });
      const body = render(CharacterBanner).body;
      for (const testid of removed) {
        expect(body, `${testid} survived for gift_policy=${policy}`).not.toContain(
          `data-testid="${testid}"`,
        );
      }
    }
  });

  // The `type-unknown` fallback is the banner's remaining conditional and the one
  // thing it still has to get right: a save naming a type the ruleset has no profile
  // for must not render the raw slug.
  it('falls back to a localized label for a type the ruleset has no profile for', () => {
    installProfile({ ...magus(), id: 'grog' });
    store.entity.type_id = 'sorcerer';
    const label = text(render(CharacterBanner).body, 'character-type');
    expect(label).not.toContain('sorcerer');
    expect(label).not.toContain('type-unknown');
  });

  it('localizes to German, rendering prose rather than an echoed key', () => {
    store.lang = 'de';
    const body = render(CharacterBanner).body;
    expect(text(body, 'character-type')).toContain('Magus');
    expect(body).not.toContain('type-label =');
  });
});
