import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// manual-testing-findings-2026-11-15 #9 (F9): a Personality Trait's value is a signed
// field (|6| max, S29). A `type="number"` input's OWN value-sanitization algorithm
// (HTML, not this codebase) reads a syntactically incomplete entry — a bare "-" typed
// on the way to "-4" — back as the empty string: `event.currentTarget.value` is `''`
// at that instant even though the field still visibly shows the "-" the user just
// typed (confirmed directly against happy-dom, which implements the same algorithm).
// The current inline handler forwards `Number(raw) || 0` on every keystroke
// (`PersonalityTraits.svelte`'s `oninput`), so that `''` becomes a written 0, and the
// plain `value={trait.value}` binding then snaps the VISIBLE field back to "0",
// wiping the minus sign before the digit after it can ever be typed — the field can
// never take a negative number by typing, only via the spinner. This is the same
// "checks prevent proper typing" F9 reports, for a signed range rather than a
// reachable ceiling, and the same root cause as the required-field collapse in
// `LifeSections.svelte` (an empty read must never be forwarded as a real value). The
// birth-year pattern's gate — forward only a value that parses and falls inside the
// field's own range, otherwise leave the store untouched — fixes it.
//
// A `client` test: the `input`/`change` listeners on a live element and the text the
// live field shows between them, which SSR never dispatches.
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
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function mountField(): HTMLInputElement {
  store.addPersonalityTrait();
  store.setPersonalityTraitName(0, 'Brave');
  store.setPersonalityTraitValue(0, 3);
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(PersonalityTraits, { target });
  flushSync();
  const input = target.querySelector<HTMLInputElement>('[data-testid="personality-value-0"]');
  expect(input).toBeTruthy();
  return input!;
}

function typeInto(input: HTMLInputElement, text: string): void {
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

describe('PersonalityTraits value takes a typed minus sign (F9)', () => {
  // A number input reads the incomplete "-" back as '' (see the file banner); this
  // is that read, simulated directly.
  it('does not collapse to 0 on the empty read a mid-typed minus sign produces', () => {
    const input = mountField();
    expect(input.value).toBe('3');

    typeInto(input, '');
    expect(input.value, 'the field is left showing what the user typed').toBe('');
    expect(store.entity.personality_traits?.[0].value, 'no collapse to 0 mid-edit').toBe(3);

    typeInto(input, '-4');
    expect(input.value).toBe('-4');
    expect(store.entity.personality_traits?.[0].value).toBe(-4);
  });

  it('clamps a typed value past the |6| bound only on commit', () => {
    const input = mountField();

    typeInto(input, '-7');
    expect(input.value, 'typing is never fought').toBe('-7');
    expect(store.entity.personality_traits?.[0].value, 'no write past the bound yet').toBe(3);

    commit(input);
    expect(store.entity.personality_traits?.[0].value).toBe(-6);
    expect(input.value).toBe('-6');
  });
});
