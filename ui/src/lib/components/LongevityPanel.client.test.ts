import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// manual-testing-findings-2026-11-15 #9 (F9), phase 2 fold-in: the Longevity Ritual
// bonus is signed (-128..127, `clamp.ts::I8_MIN`/`I8_MAX`). `onBonus` already treats
// a genuinely empty field as `null` rather than 0 (so this field never had the
// collapse-to-0 bug), but a `type="number"` input's own value-sanitization algorithm
// reads an INCOMPLETE entry — a bare "-" on the way to "-4" — back as the empty
// string too (confirmed directly against happy-dom, which implements the same
// algorithm), so the handler cannot tell the two apart: typing "-" writes `null`
// immediately, and the plain `value={longevity.bonus ?? ''}` binding blanks the
// field before the digit after the sign can ever be typed. Same mechanism as
// `PersonalityTraits.client.test.ts`/`FamiliarPanel.client.test.ts`, with a
// different wrong value (null/blank instead of 0).
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
import LongevityPanel from './LongevityPanel.svelte';

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
    ability_funding: 'pool',
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
  store.derived = null;
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

function mountPanel(): HTMLInputElement {
  store.addLongevityRitual('self_made');
  store.setLongevityBonus(5);
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(LongevityPanel, { target });
  flushSync();
  const input = target.querySelector<HTMLInputElement>('[data-testid="longevity-bonus"]');
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

/**
 * Force `input.validity.badInput` to read as `value` for the next read, standing in
 * for the one moment happy-dom cannot reproduce: a real browser's number-input value
 * sanitization algorithm runs on every `.value` SET (ours included), so a half-typed
 * "-" is already sanitized down to "" before `onBonus` ever sees it, and happy-dom's
 * own `ValidityState.badInput` getter (`value.length > 0 && ...`) therefore always
 * reads `false` once that sanitized, zero-length value is in place — there is no way
 * to make it read `true` by setting `.value` alone, in happy-dom or in a real browser.
 * A real browser instead tracks the user's actual keystrokes internally, separately
 * from the sanitized `.value`, which is exactly what `badInput` exposes. Stubbing the
 * getter is the honest way to exercise that half-typed branch here.
 */
function stubBadInput(input: HTMLInputElement, value: boolean): void {
  Object.defineProperty(input, 'validity', {
    configurable: true,
    get: () => ({ badInput: value }),
  });
}

describe('manual-testing-findings-2026-11-15 #9 follow-up: LongevityPanel bonus tells a genuine clear from a half-typed minus sign', () => {
  it('forwards a genuine empty read (validity.badInput false) to the store at once, as "not entered"', () => {
    const input = mountPanel();
    expect(input.value).toBe('5');

    typeInto(input, '');
    expect(input.value, 'the field is left showing what the user typed').toBe('');
    expect(
      store.entity.longevity_ritual?.bonus,
      'a genuine clear reaches the store immediately, matching the other nullable fields',
    ).toBe(null);
  });

  it('ignores an empty read reported as badInput (a half-typed minus sign), keeping the stored bonus', () => {
    const input = mountPanel();
    expect(input.value).toBe('5');

    stubBadInput(input, true);
    typeInto(input, '');
    expect(input.value, 'the field is left showing what the user typed').toBe('');
    expect(
      store.entity.longevity_ritual?.bonus,
      'no write to null while the field only reads as an incomplete entry',
    ).toBe(5);

    stubBadInput(input, false);
    typeInto(input, '-4');
    expect(input.value).toBe('-4');
    expect(store.entity.longevity_ritual?.bonus).toBe(-4);
  });

  it('clamps a typed value past the |128| bound only on commit', () => {
    const input = mountPanel();

    typeInto(input, '200');
    expect(input.value, 'typing is never fought').toBe('200');
    expect(store.entity.longevity_ritual?.bonus, 'no write past the bound yet').toBe(5);

    commit(input);
    expect(store.entity.longevity_ritual?.bonus).toBe(127);
    expect(input.value).toBe('127');
  });
});
