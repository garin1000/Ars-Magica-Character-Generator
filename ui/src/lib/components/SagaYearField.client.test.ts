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
  confirmDiscard: vi.fn().mockResolvedValue(true),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
  readSettings: vi.fn().mockResolvedValue({
    default_saga_year: 1197,
    lang: null,
    theme: null,
    validation_mode: null,
  }),
  writeSettings: vi.fn().mockResolvedValue(undefined),
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
    saga_year: 1220,
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
  vi.mocked(ipc.writeSettings).mockClear();
  await store.loadSettings();

  // A CLEAN baseline, which the dirty assertion below needs and only a load can give:
  // the shared singleton's saved snapshot belongs to whatever ran before. Opening a
  // document re-seeds it, so `dirty` starts false and any change the field makes is
  // the only thing that could flip it.
  vi.mocked(ipc.loadEntity).mockResolvedValue({
    path: '/tmp/rhine.armc',
    entity: { ...store.entity },
    migrated_aging_characteristics: [],
  });
  const opening = store.open();
  if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
  await opening;
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

describe('SagaYearField (slice 12, #25; document state since C8)', () => {
  it("shows the DOCUMENT's saga year, not the installation's default", () => {
    // The store's default for new documents is 1197 here; this character was built
    // for a 1220 saga and the field must say so. Reading the setting instead is the
    // exact defect C8 exists to fix.
    typeSagaYear('1220');
    const input = target.querySelector<HTMLInputElement>('[data-testid="saga-year-input"]');
    expect(input!.value).toBe('1220');
    expect(store.defaultSagaYear).toBe(1197);
  });

  it('writes the typed year into the document, not the settings file', () => {
    typeSagaYear('1230');
    expect(store.entity.saga_year).toBe(1230);
    expect(vi.mocked(ipc.writeSettings)).not.toHaveBeenCalled();
  });

  it('dirties the document, and still leaves the stored pair alone', () => {
    typeSagaYear('1230');

    // D3.3 survives the move: the saga year is the reference the pair is measured
    // against, never a rewrite of it. A silent recompute would fabricate ages that
    // skipped their aging rolls.
    expect(store.entity.age).toBe(30);
    expect(store.entity.birth_year).toBe(1190);
    // But it IS stored now, so the unsaved-changes guard must see it move.
    expect(store.dirty).toBe(true);
    expect(store.closeGuardPayload().dirty).toBe(true);
  });
});
