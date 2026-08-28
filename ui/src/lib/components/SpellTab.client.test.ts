import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EffectiveScores, Entity, LocalizedRuleset, ValidationResult } from '../types';

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
