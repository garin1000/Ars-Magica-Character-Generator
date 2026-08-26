import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { DerivedTotals, Entity, LocalizedRuleset } from './lib/types';

// The unsaved-changes guard is a MANDATORY product behavior (CLAUDE.md): closing
// or quitting with unsaved edits must prompt before discarding, on every
// platform and every quit path. The frontend half of that contract is a single
// `$effect` in App.svelte that mirrors `store.dirty` to Rust via
// `update_close_guard`.
//
// Nothing tested it. The audit recorded this as E2 (CRITICAL): the SSR suite
// renders through `svelte/server`, which never executes an `$effect`, so the
// mirror was invisible to all 800-odd tests — a regression breaking it would
// have passed the whole suite. This file is the reason the `client` vitest
// project exists (see CLAUDE.md, "Frontend test environments"); mounting the app
// client-side is what makes the effect actually run.
vi.mock('./lib/ipc', () => ({
  // Must resolve a usable ruleset: mounting runs `onMount`, which calls
  // `store.init()` -> `#reloadRuleset(true)`. That is also what seeds the clean
  // dirty-baseline, so a stub returning `undefined` both throws and leaves the
  // guard with nothing meaningful to mirror.
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

import * as ipc from './lib/ipc';
import { SCHEMA_VERSION, store } from './lib/state.svelte';
import App from './App.svelte';

function localizedRuleset(): LocalizedRuleset {
  return {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {
        companion: {
          id: 'companion',
          budget: { virtue_points: 10, flaw_points: 10 },
          permitted_categories: [],
          forbidden_categories: [],
          creation_phases: [],
        },
      },
      abilities: {},
      advancement: [],
      // Shipped life-stage rules are what makes the funding panel render at all
      // (`LifeStagePanel.svelte` gates on them, not on the character type), so the
      // Slice 2 bridge test below needs them here.
      life_stages: {
        childhood: {
          years: 5,
          native_language_ability: 'ability.living_language',
          native_language_xp: 75,
          spread_xp: 45,
          spread_abilities: [],
        },
        later_life: { xp_per_year: 15 },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function installRuleset(): void {
  store.ruleset = localizedRuleset();
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
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

/**
 * Mount the real app root into the DOM, run its initial effects, and wait for
 * the async `init()` that `onMount` kicks off to settle — `init()` is what seeds
 * the clean dirty-baseline, so asserting before it lands reads a transient state.
 */
async function mountApp(): Promise<void> {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(App, { target });
  flushSync();
  // Wait for the baseline to actually settle clean rather than just for the IPC
  // call: `store` is a module singleton, so it carries whatever the previous
  // test left behind until `init()` installs a fresh entity and re-seeds the
  // snapshot. Asserting earlier reads a transient dirty state — and because
  // `dirty` is a `$derived`, a stale `true` also suppresses the later re-run the
  // edit assertions depend on.
  await vi.waitFor(() => {
    flushSync();
    expect(store.dirty).toBe(false);
  });
}

/** The `dirty` argument of the most recent `update_close_guard` call. */
function lastMirroredDirty(): boolean {
  const calls = vi.mocked(ipc.updateCloseGuard).mock.calls;
  if (calls.length === 0) throw new Error('update_close_guard was never called');
  return calls[calls.length - 1][0];
}

beforeEach(() => {
  vi.mocked(ipc.updateCloseGuard).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.loadRuleset).mockReset().mockResolvedValue(localizedRuleset());
  store.lang = 'en';
  store.error = null;
  store.currentPath = null;
  installRuleset();
  // Deliberately dirty the singleton before each mount, so `mountApp`'s wait for
  // a clean baseline is proving that `init()` really re-seeded it rather than
  // passing on leftover state from the previous test.
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

describe('the unsaved-changes guard is mirrored to the backend', () => {
  it('mirrors the initial clean state on mount', async () => {
    await mountApp();

    // The effect must run at least once so Rust starts with a correct belief.
    expect(ipc.updateCloseGuard).toHaveBeenCalled();
    expect(lastMirroredDirty()).toBe(false);
  });

  it('sends the localized dialog strings, so no user-facing text lives in Rust', async () => {
    await mountApp();

    const [, labels] = vi.mocked(ipc.updateCloseGuard).mock.calls[0];
    // Assert the shape and that each slot is non-empty — not the exact prose,
    // which belongs to the .ftl files and must be free to change.
    expect(Object.keys(labels).sort()).toEqual(['cancel', 'discard', 'message', 'title']);
    for (const value of Object.values(labels)) {
      expect(typeof value).toBe('string');
      expect(value.length).toBeGreaterThan(0);
    }
  });

  it('re-mirrors as dirty once the character is edited', async () => {
    await mountApp();
    const before = vi.mocked(ipc.updateCloseGuard).mock.calls.length;

    store.entity.name = 'Iohannes filius Bonisagi';
    flushSync();

    expect(vi.mocked(ipc.updateCloseGuard).mock.calls.length).toBeGreaterThan(before);
    expect(lastMirroredDirty()).toBe(true);
  });

  it('mirrors clean again when the edit is undone back to the baseline', async () => {
    // This is the snapshot-compare property the guard depends on: a naive
    // one-way boolean would latch dirty forever and keep prompting after the
    // user had restored the original value.
    await mountApp();

    store.entity.name = 'edited';
    flushSync();
    expect(lastMirroredDirty()).toBe(true);

    delete store.entity.name;
    flushSync();
    expect(lastMirroredDirty()).toBe(false);
  });

  it('surfaces an IPC rejection instead of leaving the backend belief stale', async () => {
    // A swallowed rejection is the dangerous case: Rust would keep an outdated
    // dirty flag on a load-bearing guard with nothing shown to the user.
    const failure = { kind: 'ipc', message: 'bridge down' };
    vi.mocked(ipc.updateCloseGuard).mockReset().mockRejectedValue(failure);

    await mountApp();
    await vi.waitFor(() => expect(store.error).toEqual(failure));
  });
});

// S1 (round 2), S4 (round 3, tmp/review/review-round-3-sabine.md): the discard
// prompt's focus-restoration `$effect` (App.svelte:196-210) shipped with zero
// coverage anywhere in the suite. `App.test.ts` renders through `svelte/server`
// and never runs an `$effect`, so this file is the only place that can. A loaded
// entity here only needs to satisfy the shapes `open()`/`revalidate()` touch, not
// a real ruleset's full schema.
function loadableEntity(): Entity {
  return {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

describe('focus restoration around the discard-changes prompt (S1/S4)', () => {
  it('returns focus to the New button once a cancelled prompt closes', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    const newButton = document.querySelector('[data-testid="new-button"]') as HTMLElement;
    expect(newButton).toBeTruthy();
    newButton.focus();
    expect(document.activeElement).toBe(newButton);

    store.entity.name = 'a dirtying edit';
    flushSync();

    void store.newDocument();
    flushSync();
    expect(store.discardPromptOpen).toBe(true);

    store.resolveDiscardPrompt(false);
    flushSync();

    expect(document.activeElement).toBe(newButton);
  });

  it('returns focus to the Open button once a confirmed load lands back in the editor', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/example.armc.json',
      entity: loadableEntity(),
    });

    const openButton = document.querySelector('[data-testid="open-button"]') as HTMLElement;
    expect(openButton).toBeTruthy();
    openButton.focus();
    expect(document.activeElement).toBe(openButton);

    store.entity.name = 'a dirtying edit';
    flushSync();

    void store.open();
    flushSync();
    expect(store.discardPromptOpen).toBe(true);

    store.resolveDiscardPrompt(true);
    flushSync();

    // open() sets view = 'editor', which it already was, so the Open button is
    // never unmounted — this is the "trigger survives" half of the fix.
    expect(document.activeElement).toBe(openButton);
  });

  it('does not force focus onto a trigger that is no longer in the document', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    const newButton = document.querySelector('[data-testid="new-button"]') as HTMLElement;
    expect(newButton).toBeTruthy();
    newButton.focus();
    const focusSpy = vi.spyOn(newButton, 'focus');

    store.entity.name = 'a dirtying edit';
    flushSync();

    void store.newDocument();
    flushSync();
    expect(store.discardPromptOpen).toBe(true);

    // Simulate the trigger having left the document by the time the prompt
    // resolves — App.svelte's own comment describes exactly this case: "the
    // old element is disconnected and .focus() is skipped".
    newButton.remove();

    store.resolveDiscardPrompt(false);
    flushSync();

    expect(focusSpy).not.toHaveBeenCalled();
  });
});

// Slice 2 (#11) moved the funding panel and the life-stage plan off `AbilityTab`
// onto the wizard's new `experience` step. The editor does not get its own
// Experience tab until Slice 3, so `App.svelte` mounts `LifeStagePanel` on the
// editor's Abilities tab as an explicitly temporary bridge. Without it this slice
// would ship an editor with no surface at all for the funding mode, the age, the
// native language or the childhood package — and no e2e spec would catch it, since
// `life-stage-childhood.e2e.js` and `magus-apprenticeship.e2e.js` drive the wizard.
//
// A `client` test, not `ssr`: the active tab is component-local `$state` defaulting
// to `details` (`App.svelte:81`), so a `svelte/server` render can never reach the
// Abilities panel. Switching it needs a mounted instance.
describe('the editor still exposes the funding panel (Slice 2 bridge)', () => {
  it('mounts the life-stage panel on the editor Abilities tab', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    (document.getElementById('tab-abilities') as HTMLElement).click();
    flushSync();

    expect(document.querySelector('[data-testid="life-stage-panel"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="ability-funding-pool"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="ability-funding-life_stages"]')).not.toBeNull();
  });

  it('keeps the Available/Selected row a root-level sibling of the panel', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    (document.getElementById('tab-abilities') as HTMLElement).click();
    flushSync();

    // `.region-row` must stay the only `flex: 1` child of `.vf-tab` or both ability
    // lists collapse; the bridge mount is a sibling above it, never a wrapper.
    const panel = document.querySelector('[data-testid="life-stage-panel"]')!;
    const row = document.querySelector('.region-row')!;
    expect(row).not.toBeNull();
    expect(panel.parentElement).toBe(row.parentElement);
  });
});

describe('tablist keyboard navigation (S7/S4)', () => {
  function pressTabKey(key: string): void {
    const tablist = document.querySelector('[role="tablist"]') as HTMLElement;
    tablist.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
    flushSync();
  }

  it('moves the active tab and DOM focus with ArrowRight/ArrowLeft', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    const details = document.getElementById('tab-details') as HTMLElement;
    const characteristics = document.getElementById('tab-characteristics') as HTMLElement;
    expect(details.getAttribute('aria-selected')).toBe('true');
    expect(details.tabIndex).toBe(0);
    expect(characteristics.tabIndex).toBe(-1);

    pressTabKey('ArrowRight');
    expect(characteristics.getAttribute('aria-selected')).toBe('true');
    expect(characteristics.tabIndex).toBe(0);
    expect(details.getAttribute('aria-selected')).toBe('false');
    // Roving tabindex: only the active tab stays in the page's Tab order.
    expect(details.tabIndex).toBe(-1);
    await vi.waitFor(() => expect(document.activeElement?.id).toBe('tab-characteristics'));

    pressTabKey('ArrowLeft');
    expect(details.getAttribute('aria-selected')).toBe('true');
    await vi.waitFor(() => expect(document.activeElement?.id).toBe('tab-details'));
  });

  it('jumps to the first/last tab with Home/End', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();
    // The last tab is Totals (DerivedTotalsPanel), which reads store.derived
    // unconditionally — give it a minimal, complete fixture so navigating
    // there does not throw on an untested field.
    store.derived = {
      is_magus: false,
      lab_totals: [],
      casting_totals: [],
      penetration: [],
      magic_resistance: [],
      combat: [],
      soak: { addends: [], total: 0 },
      encumbrance: { load: 0, burden: 0, total: 0 },
      fatigue: [],
      wounds: [],
      size: 0,
      decrepitude_score: 0,
      warping_score: 0,
      warping_points: 0,
      surfaced_modifiers: [],
    } as DerivedTotals;
    flushSync();

    pressTabKey('ArrowRight');
    await vi.waitFor(() => expect(document.activeElement?.id).toBe('tab-characteristics'));

    pressTabKey('End');
    expect(document.getElementById('tab-totals')?.getAttribute('aria-selected')).toBe('true');
    await vi.waitFor(() => expect(document.activeElement?.id).toBe('tab-totals'));

    pressTabKey('Home');
    expect(document.getElementById('tab-details')?.getAttribute('aria-selected')).toBe('true');
    await vi.waitFor(() => expect(document.activeElement?.id).toBe('tab-details'));
  });
});
