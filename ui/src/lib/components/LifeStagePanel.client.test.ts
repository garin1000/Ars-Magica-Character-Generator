import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EntityTypeProfile, Entity, LocalizedRuleset } from '../types';

// Row 56: typing into one of the four lab-year counts must reach the store as that
// count and no other. A `client` test: the behaviour is an `input`/`change` listener
// firing on a live element, which SSR never dispatches.
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
  deriveBirthYear: vi.fn().mockResolvedValue(null),
}));

import { U32_MAX } from '../clamp';
import { SCHEMA_VERSION, store } from '../state.svelte';
import LifeStagePanel from './LifeStagePanel.svelte';

/** A ruleset carrying the post-apprenticeship block and a magus profile. */
function installRuleset(): void {
  const magus = {
    id: 'magus',
    budget: { virtue_points: 10, flaw_points: 10 },
    hermetically_trained: true,
    gift_policy: 'required',
    creation_phases: [],
  } as unknown as EntityTypeProfile;
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: { magus },
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      life_stages: {
        childhood: {
          years: 5,
          native_language_ability: 'ability.living_language',
          native_language_xp: 75,
          spread_xp: 45,
          spread_abilities: [],
        },
        later_life: { xp_per_year: 15 },
        post_apprenticeship: {
          points_per_year: 30,
          lab_season_cost: 10,
          max_charged_lab_seasons_per_year: 3,
        },
      },
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

/** A guided magus with an empty life-stage plan. */
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
    ability_funding: 'life_stages',
    life_stages: {},
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
    age: 40,
  };
  store.effective = null;
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

function mountPanel(): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(LifeStagePanel, { target });
  flushSync();
}

function field(n: number): HTMLInputElement {
  const input = target.querySelector<HTMLInputElement>(
    `[data-testid="life-stage-lab-years-${n}-input"]`,
  );
  expect(input, `no lab-years field for ${n} seasons`).toBeTruthy();
  return input!;
}

/** Replace the field's text, as typing over a selection does. */
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

describe('LifeStagePanel lab-year counts (row 56)', () => {
  const buckets = ['one_season', 'two_seasons', 'three_seasons', 'four_seasons'] as const;

  it('writes each field to its own count', () => {
    mountPanel();
    buckets.forEach((bucket, i) => {
      typeInto(field(i + 1), String(i + 5));
      expect(store.entity.life_stages?.post_gauntlet_lab_years?.[bucket]).toBe(i + 5);
    });
    expect(store.entity.life_stages?.post_gauntlet_lab_years).toEqual({
      one_season: 5,
      two_seasons: 6,
      three_seasons: 7,
      four_seasons: 8,
    });
  });

  it('drops the count, and then the whole key, when its field is emptied', () => {
    mountPanel();
    typeInto(field(3), '2');
    typeInto(field(3), '');
    expect(store.entity.life_stages).toEqual({});
  });

  it('shows the stored count once an over-long edit is committed', () => {
    mountPanel();
    typeInto(field(4), '99999999999');
    commit(field(4));
    expect(store.entity.life_stages?.post_gauntlet_lab_years?.four_seasons).toBe(U32_MAX);
    expect(field(4).value).toBe(String(U32_MAX));
  });
});
