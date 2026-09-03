import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EffectiveScores, Entity, LocalizedRuleset, PointItem, Selection } from '../types';

// guided-creation-review-2026-08 #9, the production hazard: the engine
// concatenates House, Mythic Companion, `grants_selection` and warping grants
// WITHOUT dedup (`effective.rs` `entity_grants`), so one ref can be granted
// twice. A duplicate `{#each}` key then throws `each_key_duplicate` and aborts
// the whole tab's render — for the user, not just in a test.
//
// This must be a `client` test: `each_key_duplicate` is raised by the client
// reconciler alone (`svelte/internal/client/dom/blocks/each.js`), so an SSR
// render of the same fixture passes no matter how the rows are keyed. The
// companion SSR test in VirtueFlawTab.test.ts can only assert that two rows come
// out; this one asserts the render survives at all.
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
import VirtueFlawTab from './VirtueFlawTab.svelte';

const HEARTBEAST: PointItem = {
  id: 'virtue.heartbeast',
  kind: 'virtue',
  magnitude: 'minor',
  categories: ['hermetic'],
  classification: 'narrative',
  entity_kinds: ['character'],
} as PointItem;

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: { [HEARTBEAST.id]: HEARTBEAST },
      type_profiles: {
        magus: {
          id: 'magus',
          budget: { virtue_points: 10, flaw_points: 10 },
          is_magus: true,
          gift_policy: 'required',
          creation_phases: [],
        },
      },
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: { 'virtue.heartbeast': { name: 'Heartbeast' } },
  } as unknown as LocalizedRuleset;
}

function resetEntity(selections: Selection[] = []): void {
  store.view = 'editor';
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    selections,
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
  store.result = { issues: [] };
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
  store.effective = null;
});

describe('VirtueFlawTab survives a doubly-granted Virtue (#9)', () => {
  it('renders both rows of the same ref granted twice without throwing', () => {
    store.effective = {
      granted_selections: [{ ref: 'virtue.heartbeast' }, { ref: 'virtue.heartbeast' }],
    } as unknown as EffectiveScores;

    target = document.createElement('div');
    document.body.appendChild(target);

    // The whole point: a duplicate key throws HERE, out of `mount`.
    expect(() => {
      app = mount(VirtueFlawTab, { target });
      flushSync();
    }).not.toThrow();

    expect(target.querySelectorAll('[data-testid^="granted-selection-"]')).toHaveLength(2);
  });

  it('still renders both when one bought row shares the granted ref’s category', () => {
    // A bought row in the SAME group as the granted ones: bought keys and granted
    // keys now live in one `{#each}`, so a bought entity index must not be able to
    // collide with a granted position either.
    resetEntity([{ ref: 'virtue.heartbeast' }]);
    store.effective = {
      granted_selections: [{ ref: 'virtue.heartbeast' }, { ref: 'virtue.heartbeast' }],
    } as unknown as EffectiveScores;

    target = document.createElement('div');
    document.body.appendChild(target);

    expect(() => {
      app = mount(VirtueFlawTab, { target });
      flushSync();
    }).not.toThrow();

    expect(target.querySelectorAll('li')).not.toHaveLength(0);
  });
});
