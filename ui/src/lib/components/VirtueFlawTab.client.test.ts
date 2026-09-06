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

// A dual-category item, so the Available picker renders the SAME `{#each}` key
// (`getId` = the item id) in two different group blocks. Keys are scoped per
// block instance, so this is legal — but `each_key_duplicate` aborts the whole
// tab in production and only the client reconciler raises it, so the claim is
// asserted rather than assumed. Sufi is "*Minor, Social Status, Supernatural*"
// and the book indexes it under both (Core Rules :3230 and :3179).
const SUFI: PointItem = {
  id: 'virtue.sufi',
  kind: 'virtue',
  magnitude: 'minor',
  categories: ['social_status', 'supernatural'],
  classification: 'narrative',
  entity_kinds: ['character'],
} as PointItem;

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: { [HEARTBEAST.id]: HEARTBEAST, [SUFI.id]: SUFI },
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
    i18n: { 'virtue.heartbeast': { name: 'Heartbeast' }, 'virtue.sufi': { name: 'Sufi' } },
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

// The Available picker lists a dual-category item under BOTH of its headings, so
// its `{#each group.items as item (getId(item))}` key repeats across two group
// blocks. That is legal (keys are scoped to one block instance) but the mirror
// image of the #9 hazard above, and only a mounted render can prove it: SSR never
// runs the reconciler that raises `each_key_duplicate`.
describe('VirtueFlawTab offers a dual-category item under both headings', () => {
  it('mounts with the same source row keyed in two groups', () => {
    target = document.createElement('div');
    document.body.appendChild(target);

    expect(() => {
      app = mount(VirtueFlawTab, { target });
      flushSync();
    }).not.toThrow();

    expect(target.querySelectorAll('[data-testid="add-virtue.sufi"]')).toHaveLength(2);
  });
});

// max_total slice: the at-cap block gets its own tooltip reason, the way the
// incompatibility leg already does (`vf-blocked-max-total`, `sourceTip` in
// VirtueFlawTab.svelte). Only a mounted render can show it: the reason text is
// appended to the tooltip popup on `focusin`, by the `tooltip` action, which is
// a no-op under SSR.
describe('VirtueFlawTab gives the at-cap block a reason', () => {
  const CAPPED: PointItem = {
    id: 'virtue.puissant_art',
    kind: 'virtue',
    magnitude: 'minor',
    categories: ['hermetic'],
    classification: 'narrative',
    entity_kinds: ['character'],
    parameters: [{ key: 'art', type: 'ref', domain: 'art' }],
    max_total: 2,
  } as PointItem;

  function installCappedRuleset(): void {
    store.ruleset = {
      ruleset: {
        id: 'test',
        version: '1',
        point_items: { 'virtue.puissant_art': CAPPED },
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
      i18n: { 'virtue.puissant_art': { name: 'Puissant {art}' } },
    } as unknown as LocalizedRuleset;
  }

  it('shows "Maximum of 2 already reached" once both copies are taken', () => {
    installCappedRuleset();
    resetEntity([
      { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } },
      { ref: 'virtue.puissant_art', params: { art: 'art.perdo' } },
    ]);

    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(VirtueFlawTab, { target });
    flushSync();

    const row = target.querySelector('[data-testid="add-virtue.puissant_art"]');
    expect(row).not.toBeNull();
    row!.dispatchEvent(new Event('focusin', { bubbles: true }));
    flushSync();

    const reason = document.querySelector('[data-testid="tooltip-reason"]');
    // Fluent wraps the interpolated { $max } in bidi isolation marks (FSI/PDI).
    expect(reason?.textContent?.replace(/[⁦-⁩]/g, '')).toBe('Maximum of 2 already reached');
  });
});
