import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// A `client` test, not `ssr`, for two reasons that SSR cannot cover:
//
//  1. The behaviour under test is an `oninput` LISTENER firing on a live element.
//     SSR renders markup and never dispatches an event, so an `ssr` assertion after
//     "type into the field" would report green with nothing having happened.
//  2. `store.dirty` is a `$derived` read after that listener ran, i.e. live reactive
//     state rather than rendered markup — and "editing the saga year does not dirty
//     the document" (D3.3) is the one guarantee of this slice that touches the
//     mandatory unsaved-changes guard.
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
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  sagaYear: vi.fn().mockResolvedValue(1220),
  setSagaYear: vi.fn().mockResolvedValue(undefined),
  deriveAge: vi.fn().mockResolvedValue({ age: null, issues: [] }),
  deriveBirthYear: vi.fn().mockResolvedValue(null),
}));

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import SagaYearField from './SagaYearField.svelte';

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
    age: 30,
    birth_year: 1190,
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(async () => {
  store.lang = 'en';
  store.view = 'editor';
  installRuleset();
  resetEntity();
  vi.mocked(ipc.setSagaYear).mockClear();
  await store.loadSagaYear();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

/** Mount the field and type `value` into it, as a user would. */
function typeSagaYear(value: string): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(SagaYearField, { target });
  flushSync();

  const input = target.querySelector<HTMLInputElement>('[data-testid="saga-year-input"]');
  expect(input).toBeTruthy();
  input!.value = value;
  input!.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

describe('SagaYearField (slice 12, #25)', () => {
  it('shows the loaded saga year', () => {
    typeSagaYear('1220');
    const input = target.querySelector<HTMLInputElement>('[data-testid="saga-year-input"]');
    expect(input!.value).toBe('1220');
  });

  it('persists the typed saga year as saga state', () => {
    typeSagaYear('1230');
    expect(store.sagaYear).toBe(1230);
    expect(vi.mocked(ipc.setSagaYear)).toHaveBeenCalledWith(1230);
  });

  it('leaves the stored pair alone and does not dirty the document', () => {
    // Compared against the flag as it stood, the way the picker-filter guard does:
    // the shared singleton's saved baseline belongs to whatever ran before, so the
    // claim is "this edit moved nothing", not "the document happens to be clean".
    const dirtyBefore = store.dirty;
    typeSagaYear('1230');

    // D3.3: the saga year is a reference for derivation, never a rewrite. A silent
    // recompute would fabricate ages that skipped their aging rolls.
    expect(store.entity.age).toBe(30);
    expect(store.entity.birth_year).toBe(1190);
    expect(store.dirty).toBe(dirtyBefore);
    expect(store.closeGuardPayload().dirty).toBe(dirtyBefore);
  });
});
