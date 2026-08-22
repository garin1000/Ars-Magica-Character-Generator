import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from './lib/types';

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
