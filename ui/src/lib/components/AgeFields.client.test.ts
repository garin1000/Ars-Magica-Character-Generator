import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// N2 (try-out 2026-10-04, Norbert): "If I enter 500, or 600 and input is rewritten to
// 500, I can enter the field afterwards and add more digits without it being
// rechecked." `setAge` clamps to 1..max_age, and once the store already holds 500 a
// longer number clamps to the same 500, so Svelte has nothing to re-render and the
// field shows "5000". The field must show the stored age once the edit is committed.
//
// A `client` test: the behaviour is the `input`/`change` listeners firing on a live
// element and the text the live field shows afterwards. SSR dispatches no events.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  confirmDiscard: vi.fn().mockResolvedValue(true),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  readSettings: vi.fn(),
  writeSettings: vi.fn(),
  deriveAge: vi.fn().mockResolvedValue({ age: null, issues: [] }),
  // The birth year the stored age implies, so a no-op commit derives no change.
  deriveBirthYear: vi.fn().mockResolvedValue(720),
}));

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import AgeFields from './AgeFields.svelte';

const AGE = '[data-testid="age-input"]';

/** A ruleset whose aging block states the shipped app maximum age (slice A1). */
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
      aging: {
        start_age: 35,
        age_divisor: 10,
        apparent_age_increase_min: 3,
        living_conditions: [],
        outcomes: [],
        max_age: 500,
      },
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function character(age: number | null): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'grog',
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
    age,
    birth_year: age == null ? null : 1220 - age,
  };
}

/** Open `entity` as a saved document, so `dirty` starts false against it. */
async function openDocument(entity: Entity): Promise<void> {
  vi.mocked(ipc.loadEntity).mockResolvedValue({
    path: '/saga/grog.armc',
    entity,
    migrated_aging_characteristics: [],
  });
  const opening = store.open();
  if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
  await opening;
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function mountField(): HTMLInputElement {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(AgeFields, { target });
  flushSync();
  const input = target.querySelector<HTMLInputElement>(AGE);
  expect(input).toBeTruthy();
  return input!;
}

/** Type `text` one key at a time, appending to whatever the field shows. */
function typeKeys(input: HTMLInputElement, text: string): void {
  for (const key of text) {
    input.value = input.value + key;
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
  }
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(input: HTMLInputElement): void {
  input.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

describe('AgeFields shows the stored age once an edit is committed (N2)', () => {
  it('rewrites "5000" to the stored 500 when the store already held 500', async () => {
    await openDocument(character(500));
    const input = mountField();
    expect(input.value).toBe('500');

    typeKeys(input, '0');
    expect(input.value, 'typing is never fought').toBe('5000');
    expect(store.entity.age).toBe(500);

    commit(input);
    expect(input.value).toBe('500');
    expect(store.entity.age).toBe(500);
  });

  it('rewrites a typed 6000 to 500 — the clamp reached on the third key holds', async () => {
    await openDocument(character(null));
    const input = mountField();

    typeKeys(input, '6000');
    commit(input);
    expect(store.entity.age).toBe(500);
    expect(input.value).toBe('500');
  });

  it('does not dirty the document when the commit only rewrites the field', async () => {
    await openDocument(character(500));
    const input = mountField();

    typeKeys(input, '0');
    commit(input);
    await vi.waitFor(() => expect(vi.mocked(ipc.deriveBirthYear)).toHaveBeenCalled());
    expect(input.value).toBe('500');
    expect(store.dirty).toBe(false);
  });

  // `setAge` reads a non-positive number as "field cleared" (store test "clamps
  // non-positive/non-finite to null"); the field shows that rule, not a "0".
  it('shows an empty field for a typed 0, which setAge stores as no age', async () => {
    await openDocument(character(null));
    const input = mountField();

    typeKeys(input, '0');
    commit(input);
    expect(store.entity.age).toBeNull();
    expect(input.value).toBe('');
  });
});
