import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import type { DerivedTotals, Entity, EntityTypeProfile, LocalizedRuleset } from '../types';

import { SCHEMA_VERSION, store } from '../state.svelte';
import IdentityFields from './IdentityFields.svelte';

function installRuleset(profiles: Record<string, EntityTypeProfile> = {}): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: profiles,
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

/** A magus-shaped profile, `hermetically_trained` set at the PROFILE level —
 * the shape manual-testing-findings #18 gates the Wizard's Sigil on. */
function magusProfile(): EntityTypeProfile {
  return {
    id: 'magus',
    budget: { virtue_points: 10, flaw_points: 10 },
    hermetically_trained: true,
    creation_phases: [],
  };
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
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
  store.effective = null;
  // #18's gating tests set this directly; reset so no test leaks into the next.
  store.derived = null;
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

/** Whether an element carrying `data-testid` exists in the rendered markup at all. */
function has(body: string, testid: string): boolean {
  return body.includes(`data-testid="${testid}"`);
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
    expect(valueOf(body, 'identity-covenant')).toBe('');
    // #18: a companion with no Hermetic training is neither magus-shaped nor
    // a Redcap, so neither field is offered at all — see the gating describe
    // block below.
    expect(has(body, 'identity-sigil')).toBe(false);
    expect(has(body, 'identity-parens')).toBe(false);
  });

  it('reads the concept text back from the entity', () => {
    store.entity.concept = 'A wandering scholar of the Bonisagus line.';
    expect(textareaValueOf(html(), 'identity-concept')).toBe(
      'A wandering scholar of the Bonisagus line.',
    );
  });

  it('reads gender and covenant back from the entity', () => {
    store.entity.gender = 'She/her';
    store.entity.covenant_name = 'Fengheld';
    const body = html();
    expect(valueOf(body, 'identity-gender')).toBe('She/her');
    expect(valueOf(body, 'identity-covenant')).toBe('Fengheld');
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

// Slice A1: the birth year is the age's other view, so its input's lower bound is
// saga year - the ruleset's maximum age; the two cannot drift apart. 321, not the
// shipped 500, proves the bound comes from the ruleset.
describe('IdentityFields birth-year lower bound (slice A1)', () => {
  function minOf(body: string): string | null {
    const tag = /<input[^>]*data-testid="identity-birth-year"[^>]*>/.exec(body);
    if (!tag) throw new Error('no birth-year input');
    return /\bmin="([^"]*)"/.exec(tag[0])?.[1] ?? null;
  }

  it('is the saga year less the ruleset maximum age', () => {
    (store.ruleset!.ruleset as unknown as Record<string, unknown>).aging = {
      start_age: 35,
      age_divisor: 10,
      apparent_age_increase_min: 3,
      living_conditions: [],
      outcomes: [],
      max_age: 321,
    };
    expect(minOf(html())).toBe('899');
    store.entity.saga_year = 1300;
    expect(minOf(html())).toBe('979');
  });

  it('falls back to the stored width when the ruleset states no maximum age', () => {
    expect(minOf(html())).toBe('-2147483648');
  });
});

// D42: the concept's optional default realm, a free identity field exactly
// like the six above — set in the concept step, read back on reopen.
describe('IdentityFields concept realm', () => {
  it('renders the "none" option selected for a fresh entity', () => {
    const body = html();
    const select = /<select[^>]*data-testid="identity-concept-realm"[\s\S]*?<\/select>/.exec(
      body,
    )![0];
    expect(select).toMatch(/<option value="" selected/);
  });

  it('reads a stored concept realm back as the selected option', () => {
    store.entity.concept_realm = 'faerie';
    const body = html();
    const select = /<select[^>]*data-testid="identity-concept-realm"[\s\S]*?<\/select>/.exec(
      body,
    )![0];
    expect(select).toMatch(/<option value="faerie" selected/);
  });
});

// manual-testing-findings #18: Sigil and Parens used to show for every
// character, companions and grogs included. Sigil is a magus's own thing
// ("recurrent mark or quirk in a magus's spell", ArMDE:12453) — gated on the
// TYPE PROFILE's own `hermetically_trained` flag, never the entity-level
// union, so an Abandoned Apprentice (hermetically trained by selection, not
// by profile) still does not see it. Parens is broader — any Hermetically
// trained character, which is the entity-level union, OR a Redcap: Redcap's
// own effects (`replaces_life_stage_xp`) never set `ConfersHermeticTraining`,
// so the entity-level union reads false for a Redcap and the virtue must be
// checked directly ("trained in a similar manner to magi", ArMDE:4919).
describe('IdentityFields sigil/parens gating (#18)', () => {
  it('hides both fields for an untrained companion', () => {
    installRuleset({});
    const body = html();
    expect(has(body, 'identity-sigil')).toBe(false);
    expect(has(body, 'identity-parens')).toBe(false);
  });

  it('shows both fields for a magus-shaped profile', () => {
    installRuleset({ magus: magusProfile() });
    store.entity.type_id = 'magus';
    // A real magus's engine-derived `hermetically_trained` union is true
    // because the PROFILE itself declares it; the test fixture sets both,
    // exactly as the real app's `derivedTotals()` round trip would.
    store.derived = { hermetically_trained: true } as unknown as DerivedTotals;
    const body = html();
    expect(has(body, 'identity-sigil')).toBe(true);
    expect(has(body, 'identity-parens')).toBe(true);
  });

  it('hides Sigil but shows Parens for an entity hermetically trained by selection, not by profile (Abandoned Apprentice)', () => {
    installRuleset({});
    store.derived = { hermetically_trained: true } as unknown as DerivedTotals;
    const body = html();
    expect(has(body, 'identity-sigil')).toBe(false);
    expect(has(body, 'identity-parens')).toBe(true);
  });

  it('shows Parens but not Sigil for a companion holding virtue.redcap', () => {
    installRuleset({});
    store.entity.selections = [{ ref: 'virtue.redcap' }];
    const body = html();
    expect(has(body, 'identity-sigil')).toBe(false);
    expect(has(body, 'identity-parens')).toBe(true);
  });

  it('shows Parens for a companion holding virtue.lone_redcap', () => {
    installRuleset({});
    store.entity.selections = [{ ref: 'virtue.lone_redcap' }];
    expect(has(html(), 'identity-parens')).toBe(true);
  });

  it('keeps a stored value on an entity that no longer qualifies, hidden but not dropped', () => {
    installRuleset({ magus: magusProfile() });
    store.entity.type_id = 'magus';
    store.entity.sigil = 'A cold wind that smells of ash';
    store.entity.parens = 'Magister Bonisagus';
    html();
    expect(has(html(), 'identity-sigil')).toBe(true);

    // The type changes away from magus — no selections to re-derive training
    // from, so both fields are gated off — but the stored strings survive on
    // the entity untouched (CLAUDE.md: saves store choices, never drops data).
    store.entity.type_id = 'companion';
    const body = html();
    expect(has(body, 'identity-sigil')).toBe(false);
    expect(has(body, 'identity-parens')).toBe(false);
    expect(store.entity.sigil).toBe('A cold wind that smells of ash');
    expect(store.entity.parens).toBe('Magister Bonisagus');
  });
});
