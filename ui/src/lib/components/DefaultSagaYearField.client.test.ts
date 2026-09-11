import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// A `client` test for the same two reasons `SagaYearField.client.test.ts` gives: the
// behaviour under test is an `oninput` LISTENER firing on a live element, and the
// claim afterwards is about live reactive state (`store.dirty`, `store.entity`)
// rather than rendered markup. An SSR assertion after "type into the field" reports
// green with nothing having happened.
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
import DefaultSagaYearField from './DefaultSagaYearField.svelte';

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
    // Built for a Rhine saga, while the installation's default is an Iberian 1197.
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

  // A clean baseline, so "this edit dirtied nothing" is a claim about the edit and
  // not about whatever ran before in the shared singleton.
  vi.mocked(ipc.loadEntity).mockResolvedValue({
    path: '/tmp/rhine.armc',
    entity: { ...store.entity },
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
function typeDefaultSagaYear(value: string): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(DefaultSagaYearField, { target });
  flushSync();

  const input = target.querySelector<HTMLInputElement>('[data-testid="default-saga-year-input"]');
  expect(input).toBeTruthy();
  input!.value = value;
  input!.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

describe('DefaultSagaYearField (C8)', () => {
  it("shows the installation's default, not the open document's year", () => {
    typeDefaultSagaYear('1197');
    const input = target.querySelector<HTMLInputElement>('[data-testid="default-saga-year-input"]');
    expect(input!.value).toBe('1197');
    expect(store.entity.saga_year).toBe(1220);
  });

  it('persists the typed default, naming only its own key', () => {
    typeDefaultSagaYear('1230');
    expect(store.defaultSagaYear).toBe(1230);
    // Read-modify-write on the Rust side, so a patch naming one key cannot disturb
    // the theme or the language sharing the file.
    expect(vi.mocked(ipc.writeSettings)).toHaveBeenCalledWith({ default_saga_year: 1230 });
  });

  it('leaves the open document — and the dirty flag — completely alone', () => {
    typeDefaultSagaYear('1230');

    // A character already built for another saga keeps its own year: this control
    // seeds the NEXT document, never the current one.
    expect(store.entity.saga_year).toBe(1220);
    expect(store.dirty).toBe(false);
    expect(store.closeGuardPayload().dirty).toBe(false);
  });
});
