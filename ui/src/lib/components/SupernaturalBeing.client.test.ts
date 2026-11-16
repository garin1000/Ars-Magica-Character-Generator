import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// manual-testing-findings-2026-11-15 #9 (F9), phase 2 fold-in: the Might score is
// unsigned but reaches a hard ceiling (0..255, `clamp.ts::U8_MAX`). The current
// inline handler forwards `Number(raw)` unconditionally on every keystroke
// (`SupernaturalBeing.svelte`'s `num`), so a value past 255 writes immediately and
// the plain `value={might.score}` binding snaps the field back before typing
// finishes — same mechanism as `AgingRecordPanel.client.test.ts`'s Aging Points case.
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
import SupernaturalBeing from './SupernaturalBeing.svelte';

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
    type_id: 'mythic_companion',
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
  store.setMightRealm('magic');
  store.setMightScore(10);
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(SupernaturalBeing, { target });
  flushSync();
  const input = target.querySelector<HTMLInputElement>('[data-testid="might-score"]');
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

describe('SupernaturalBeing Might score stays within its ceiling while typing (F9)', () => {
  it('keeps an out-of-range value out of the store, and clamps only on commit', () => {
    const input = mountPanel();
    expect(input.value).toBe('10');

    typeInto(input, '300');
    expect(input.value, 'typing is never fought').toBe('300');
    expect(store.entity.might?.score, 'no write past the bound until the edit is committed').toBe(
      10,
    );

    commit(input);
    expect(store.entity.might?.score).toBe(255);
    expect(input.value).toBe('255');
  });
});
