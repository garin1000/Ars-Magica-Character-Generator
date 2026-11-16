import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, EntityTypeProfile, LifeStagePlan, LocalizedRuleset } from '../types';

// G3: every control of the life-section editor must reach the store as the edit it
// names, on the section it belongs to. A `client` test: the behaviour is a
// `click`/`input`/`change` listener firing on a live element, which SSR never
// dispatches. The markup itself is pinned in `LifeSections.test.ts`.
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

import { SCHEMA_VERSION, store } from '../state.svelte';
import LifeSections from './LifeSections.svelte';
import LifeStagePanel from './LifeStagePanel.svelte';

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

function resetEntity(plan: LifeStagePlan): void {
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
    life_stages: plan,
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
  resetEntity({
    life_sections_enabled: true,
    life_sections: [
      { name: 'A', years: 1 },
      { name: 'B', years: 2 },
    ],
  });
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function mountEditor(): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(LifeSections, { target, props: { labYears: true } });
  flushSync();
}

function mountPanel(): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(LifeStagePanel, { target });
  flushSync();
}

function byTestId<T extends HTMLElement>(testid: string): T {
  const element = target.querySelector<T>(`[data-testid="${testid}"]`);
  expect(element, `no element with data-testid="${testid}"`).toBeTruthy();
  return element!;
}

function typeInto(testid: string, text: string): void {
  const input = byTestId<HTMLInputElement>(testid);
  input.value = text;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

function click(testid: string): void {
  byTestId<HTMLElement>(testid).click();
  flushSync();
}

/** Commit the field, as leaving it (or Enter) does. */
function commit(testid: string): void {
  byTestId<HTMLInputElement>(testid).dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

const names = () => store.entity.life_stages?.life_sections?.map((s) => s.name);

describe('LifeSections editor wiring (G3)', () => {
  it('adds a section from the add button', () => {
    mountEditor();
    click('life-section-add');
    expect(names()).toEqual(['A', 'B', '']);
  });

  it('writes a typed name and years to their own section', () => {
    mountEditor();
    typeInto('life-section-1-name-input', 'Abbey');
    typeInto('life-section-1-years-input', '7');
    expect(store.entity.life_stages?.life_sections?.[1]).toEqual({ name: 'Abbey', years: 7 });
    expect(store.entity.life_stages?.life_sections?.[0]).toEqual({ name: 'A', years: 1 });
  });

  it('moves and removes the section whose button was pressed', () => {
    mountEditor();
    click('life-section-1-up');
    expect(names()).toEqual(['B', 'A']);
    click('life-section-0-down');
    expect(names()).toEqual(['A', 'B']);
    click('life-section-0-delete');
    expect(names()).toEqual(['B']);
  });

  it('writes a section’s lab years to that section, and Remaining’s to the plan', () => {
    mountEditor();
    typeInto('life-section-1-lab-years-4-input', '2');
    typeInto('life-stage-lab-years-1-input', '3');
    expect(store.entity.life_stages?.life_sections?.[1].lab_years).toEqual({ four_seasons: 2 });
    expect(store.entity.life_stages?.post_gauntlet_lab_years).toEqual({ one_season: 3 });
  });

  // manual-testing-findings-2026-11-15 #9 (F9): `setLifeSectionYears` reads a blank
  // field as 0 (the field is "required" — there is no stored `null`), so clearing it
  // to retype used to write 0 on the very next keystroke, and the plain `value={...}`
  // binding snapped the field straight back to "0" before the player could type a
  // replacement — exactly the "checks prevent proper typing" F9 reports. The
  // birth-year pattern's "or nothing" clause (`actions.ts::commitStored`'s own doc,
  // and `IdentityFields.svelte::onBirthYearInput`) is for precisely this case: a
  // field with no legitimate blank value must leave the store untouched while typing
  // rather than force a default through it, and clamp only on commit.
  it('keeps a cleared years field blank instead of snapping to the required 0 (F9)', () => {
    mountEditor();
    typeInto('life-section-1-years-input', '');
    expect(
      byTestId<HTMLInputElement>('life-section-1-years-input').value,
      'typing is never fought',
    ).toBe('');
    expect(
      store.entity.life_stages?.life_sections?.[1].years,
      'no write until the edit is committed',
    ).toBe(2);

    commit('life-section-1-years-input');
    expect(store.entity.life_stages?.life_sections?.[1].years).toBe(0);
    expect(byTestId<HTMLInputElement>('life-section-1-years-input').value).toBe('0');
  });
});

describe('LifeStagePanel life-section switch wiring (G3)', () => {
  it('switches the sections off and on from the checkbox', () => {
    mountPanel();
    const toggle = byTestId<HTMLInputElement>('life-sections-toggle');
    toggle.click();
    flushSync();
    expect(store.entity.life_stages?.life_sections_enabled).toBeUndefined();
    toggle.click();
    flushSync();
    expect(store.entity.life_stages?.life_sections_enabled).toBe(true);
  });
});
