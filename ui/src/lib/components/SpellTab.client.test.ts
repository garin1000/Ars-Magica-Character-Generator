import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type {
  DerivedTotals,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  ValidationResult,
} from '../types';

// S3 (tmp/review/review-round-2-sabine.md): adding a General Ritual spell
// through the ordinary "Add" click defaulted its level to the flat
// GENERAL_DEFAULT_LEVEL (5), which is immediately illegal — a Ritual's
// minimum learnable level is the engine's ritual_min_level (20). This needs a
// real click, so it belongs in the `client` project (SSR never executes the
// click handler's effect on `store`, only the markup). Mirrors
// SpellTab.test.ts's fixtures.
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

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import SpellTab from './SpellTab.svelte';

const RITUAL = 'spell.test_general_ritual';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      advancement: [],
      arts: {
        'art.creo': { id: 'art.creo', art_type: 'technique' },
        'art.animal': { id: 'art.animal', art_type: 'form' },
      },
      spells: {
        [RITUAL]: { id: RITUAL, technique: 'art.creo', form: 'art.animal', ritual: true },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      // Matches the shipped engine constant (spell::RITUAL_MIN_LEVEL); the
      // point of this test is that add() must read THIS, not a local literal.
      ritual_min_level: 20,
    },
    i18n: {
      'art.creo': { name: 'Creo', abbreviation: 'Cr' },
      'art.animal': { name: 'Animal', abbreviation: 'An' },
      [RITUAL]: { name: 'Test General Ritual' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  // revalidate() no-ops on the startup screen (a placeholder, not a real
  // character); this test edits and revalidates a real one.
  store.view = 'editor';
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
    spells: [],
  };
  store.effective = {
    ability_bonuses: [],
    art_bonuses: [],
    characteristic_caps: {},
    characteristic_floors: {},
    spell_levels_used: 0,
    spell_levels_budget: 120,
    spell_levels_profile_base: 120,
  } as unknown as EffectiveScores;
  store.result = null;
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
  vi.mocked(ipc.validateEntity).mockReset();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

describe('SpellTab defaults a new General Ritual to a legal level (S3)', () => {
  it('adds a General Ritual at the engine ritual minimum, not the flat General default, and produces no blocking legality error', async () => {
    // A minimal stand-in for the engine's CODE_SPELL_RITUAL_LEGALITY rule
    // (validation/magus.rs): a Ritual spell below ritual_min_level is a
    // blocking error. This proves the fix end-to-end from the frontend's own
    // wiring, not just that a number changed.
    vi.mocked(ipc.validateEntity).mockImplementation(
      async (entity: Entity): Promise<ValidationResult> => {
        const spell = entity.spells?.find((s) => s.spell === RITUAL);
        const illegal = spell != null && (spell.level ?? 0) < 20;
        return {
          issues: illegal
            ? [
                {
                  severity: 'error',
                  code: 'spell_ritual_legality',
                  phase: 'spells',
                  args: {},
                },
              ]
            : [],
        };
      },
    );

    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(SpellTab, { target });
    flushSync();

    const addButton = target.querySelector(`[data-testid="add-${RITUAL}"]`) as HTMLButtonElement;
    expect(addButton).toBeTruthy();
    addButton.click();
    flushSync();

    expect(store.entity.spells).toEqual([{ spell: RITUAL, level: 20 }]);

    await store.revalidate();
    expect(store.result?.issues.filter((i) => i.severity === 'error')).toEqual([]);
  });
});

// X10c: the within-focus toggle writes through the store like every other
// picker edit. A `client` test because it exercises the real `onchange`
// wiring, not just the rendered markup. Red-checkpoint protocol, phase 1:
// `SpellTab.svelte` carries no such toggle yet, so `toggle` throws looking
// for an element that does not exist.
describe('SpellTab within-focus toggle writes through the store (X10c)', () => {
  function mountTab(): void {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(SpellTab, { target });
    flushSync();
  }

  function toggle(): HTMLInputElement {
    const testid = `spell-within-focus-${RITUAL}-0`;
    const el = target.querySelector<HTMLInputElement>(`[data-testid="${testid}"]`);
    if (!el) throw new Error(`within-focus toggle not rendered: no [data-testid="${testid}"]`);
    return el;
  }

  beforeEach(() => {
    store.derived = {
      casting_totals: [
        {
          technique: 'art.creo',
          form: 'art.animal',
          within_focus: {
            focus_art: 0,
            formulaic: 41,
            ritual: 41,
            spontaneous_fatiguing: 20,
            spontaneous_non_fatiguing: 20,
          },
        },
      ],
    } as unknown as DerivedTotals;
  });

  it('toggling on writes within_focus: true onto the spell selection', () => {
    store.entity.spells = [{ spell: RITUAL, level: 20 }];
    mountTab();
    const input = toggle();
    input.checked = true;
    input.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.spells![0].within_focus).toBe(true);
  });

  it('toggling back off clears within_focus', () => {
    store.entity.spells = [{ spell: RITUAL, level: 20, within_focus: true }];
    mountTab();
    const input = toggle();
    input.checked = false;
    input.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.spells![0].within_focus ?? false).toBe(false);
  });
});

// D79: the within-potent-field toggle writes through the store exactly like
// the within-focus one above. A `client` test for the same reason (real
// `onchange` wiring, not just markup).
describe('SpellTab within-potent-field toggle writes through the store (D79)', () => {
  function mountTab(): void {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(SpellTab, { target });
    flushSync();
  }

  function toggle(): HTMLInputElement {
    const testid = `spell-within-potent-field-${RITUAL}-0`;
    const el = target.querySelector<HTMLInputElement>(`[data-testid="${testid}"]`);
    if (!el)
      throw new Error(`within-potent-field toggle not rendered: no [data-testid="${testid}"]`);
    return el;
  }

  beforeEach(() => {
    store.derived = {
      casting_totals: [
        {
          technique: 'art.creo',
          form: 'art.animal',
          within_potent_field: {
            formulaic: 35,
            ritual: 35,
            spontaneous_fatiguing: 17,
            spontaneous_non_fatiguing: 17,
          },
        },
      ],
    } as unknown as DerivedTotals;
  });

  it('toggling on writes within_potent_field: true onto the spell selection', () => {
    store.entity.spells = [{ spell: RITUAL, level: 20 }];
    mountTab();
    const input = toggle();
    input.checked = true;
    input.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.spells![0].within_potent_field).toBe(true);
  });

  it('toggling back off clears within_potent_field', () => {
    store.entity.spells = [{ spell: RITUAL, level: 20, within_potent_field: true }];
    mountTab();
    const input = toggle();
    input.checked = false;
    input.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.spells![0].within_potent_field ?? false).toBe(false);
  });

  // D79: toggling the Potent Magic marker must never touch the independent
  // Magical Focus marker on the same row.
  it('does not disturb an independently-set within_focus on the same row', () => {
    store.derived = {
      casting_totals: [
        {
          technique: 'art.creo',
          form: 'art.animal',
          within_focus: {
            focus_art: 0,
            formulaic: 41,
            ritual: 41,
            spontaneous_fatiguing: 20,
            spontaneous_non_fatiguing: 20,
          },
          within_potent_field: {
            formulaic: 35,
            ritual: 35,
            spontaneous_fatiguing: 17,
            spontaneous_non_fatiguing: 17,
          },
        },
      ],
    } as unknown as DerivedTotals;
    store.entity.spells = [{ spell: RITUAL, level: 20, within_focus: true }];
    mountTab();
    const input = toggle();
    input.checked = true;
    input.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.spells![0].within_focus).toBe(true);
    expect(store.entity.spells![0].within_potent_field).toBe(true);
  });
});

// D81.5: the picker's "add within focus" action writes through the store
// with `within_focus: true` already set — a real click, so it belongs in the
// `client` project exactly like the General-Ritual add above (SSR renders
// the markup but never runs the click handler against `store`).
describe('SpellTab "add within focus" action adds the spell already marked (D81.5)', () => {
  const CANDIDATE = 'spell.test_focus_candidate';

  function installCandidate(): void {
    store.ruleset!.ruleset.spells = {
      ...store.ruleset!.ruleset.spells,
      [CANDIDATE]: { id: CANDIDATE, technique: 'art.creo', form: 'art.animal', level: 15 },
    };
    store.ruleset!.i18n[CANDIDATE] = { name: 'Test Focus Candidate' };
    // Exceeds the plain cap (10) but fits the Magical-Focus-doubled one (20).
    store.effective!.spell_caps = [{ spell: CANDIDATE, cap: 10, within_focus_cap: 20 }];
  }

  function mountTab(): void {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(SpellTab, { target });
    flushSync();
  }

  it('adds the spell with within_focus: true set from the click, not a later toggle', () => {
    installCandidate();
    mountTab();
    const button = target.querySelector(
      `[data-testid="add-within-focus-${CANDIDATE}"]`,
    ) as HTMLButtonElement;
    expect(button).toBeTruthy();
    button.click();
    flushSync();
    expect(store.entity.spells).toEqual([{ spell: CANDIDATE, within_focus: true }]);
  });
});
