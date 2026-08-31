import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

import { SCHEMA_VERSION, store } from '../state.svelte';
import IdentityFields from './IdentityFields.svelte';

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
  store.effective = null;
}

function html(): string {
  return render(IdentityFields, { props: {} }).body;
}

/** The value the SSR renderer wrote for the (non-textarea) field matching `testid`. */
function valueOf(body: string, testid: string): string | null {
  const tag = new RegExp(`<input[^>]*data-testid="${testid}"[^>]*>`).exec(body);
  if (!tag) throw new Error(`no input ${testid}`);
  return /\bvalue="([^"]*)"/.exec(tag[0])?.[1] ?? null;
}

/** The text content SSR wrote inside the `<textarea>` matching `testid`. */
function textareaValueOf(body: string, testid: string): string {
  const match = new RegExp(
    `<textarea[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</textarea>`,
  ).exec(body);
  if (!match) throw new Error(`no textarea ${testid}`);
  return match[1];
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// G12 (full-audit test-adequacy): IdentityFields had no test file at all — its
// six flavor fields (concept, gender, birth year, sigil, covenant, parens) were
// exercised only through CharacterDetails's e2e-driven presence check
// (`data-testid="identity-concept"` exists) and the slow e2e layer directly.
// These assert that every field round-trips the entity's stored value into the
// rendered markup, which is what a screen-reader/keyboard user's next edit
// would build on.
describe('IdentityFields reflects stored entity values', () => {
  it('renders empty fields for a fresh entity', () => {
    const body = html();
    expect(textareaValueOf(body, 'identity-concept')).toBe('');
    expect(valueOf(body, 'identity-gender')).toBe('');
    expect(valueOf(body, 'identity-sigil')).toBe('');
    expect(valueOf(body, 'identity-covenant')).toBe('');
    expect(valueOf(body, 'identity-parens')).toBe('');
  });

  it('reads the concept text back from the entity', () => {
    store.entity.concept = 'A wandering scholar of the Bonisagus line.';
    expect(textareaValueOf(html(), 'identity-concept')).toBe(
      'A wandering scholar of the Bonisagus line.',
    );
  });

  it('reads gender, sigil, covenant and parens back from the entity', () => {
    store.entity.gender = 'She/her';
    store.entity.sigil = 'A cold wind that smells of ash';
    store.entity.covenant_name = 'Fengheld';
    store.entity.parens = 'Magister Bonisagus';
    const body = html();
    expect(valueOf(body, 'identity-gender')).toBe('She/her');
    expect(valueOf(body, 'identity-sigil')).toBe('A cold wind that smells of ash');
    expect(valueOf(body, 'identity-covenant')).toBe('Fengheld');
    expect(valueOf(body, 'identity-parens')).toBe('Magister Bonisagus');
  });

  it('renders an empty birth-year field when none is set, never the literal "null"', () => {
    const body = html();
    expect(valueOf(body, 'identity-birth-year')).toBe('');
    expect(body).not.toContain('>null<');
  });

  it('reads a set birth year back from the entity', () => {
    store.entity.birth_year = 1220;
    expect(valueOf(html(), 'identity-birth-year')).toBe('1220');
  });
});
