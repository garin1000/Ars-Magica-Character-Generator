import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// E1 (tmp/review/review-round-2-erika.md): DiscardPrompt's focus-management
// $effect (moves initial focus to Cancel, the SAFE control) was added in
// round 1 as an accessibility fix for a dialog whose own comment says "the
// destructive choice must never be reachable as a side effect" — and shipped
// with zero test coverage at any layer. SSR (the whole rest of the suite)
// never executes an `$effect`, so a regression here — landing on Discard, or
// nowhere at all — would pass every existing test. This is exactly what the
// `client` vitest project exists to catch (see CLAUDE.md, "Frontend test
// environments").
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
import DiscardPrompt from './DiscardPrompt.svelte';

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

/** A dirtied entity: `store.dirty` compares this against the singleton's
 * construction-time baseline, which this always differs from. */
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
    name: 'a dirtying edit',
  };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

/** Opens the prompt exactly the way the app does: a New/Open action while
 * dirty (`AppStore.newDocument`). Not awaited — `confirmDiscard()` sets
 * `discardPromptOpen` synchronously inside its Promise executor before
 * suspending on the `await`, so it is already true by the next flush. */
function openPrompt(): void {
  void store.newDocument();
  flushSync();
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  // A test that opened the prompt but never resolved it would otherwise leak
  // discardPromptOpen=true into the next test.
  if (store.discardPromptOpen) store.resolveDiscardPrompt(false);
});

describe('DiscardPrompt focus management (E1)', () => {
  it('moves initial focus to the safe Cancel control, never the destructive Discard one', () => {
    expect(store.dirty).toBe(true); // sanity: newDocument() only prompts when dirty

    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(DiscardPrompt, { target });
    flushSync();

    openPrompt();
    expect(store.discardPromptOpen).toBe(true);

    const cancel = target.querySelector('[data-testid="discard-cancel"]');
    const discard = target.querySelector('[data-testid="discard-confirm"]');
    expect(cancel).toBeTruthy();
    expect(document.activeElement).toBe(cancel);
    expect(document.activeElement).not.toBe(discard);
  });

  it('resolves Escape to cancel, never to discard, leaving the document untouched', async () => {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(DiscardPrompt, { target });
    flushSync();

    openPrompt();
    expect(store.discardPromptOpen).toBe(true);

    const dialog = target.querySelector('[role="alertdialog"]') as HTMLElement;
    expect(dialog).toBeTruthy();
    dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    flushSync();

    expect(store.discardPromptOpen).toBe(false);
    // Resolving the prompt's promise only continues `newDocument()`'s `await`
    // on a LATER microtask — checking `store.entity.name` immediately would
    // still read the pre-continuation value and pass whether or not the
    // continuation goes on to discard, which is exactly the vacuous-check
    // trap (asserting before the effect being tested has actually run). A
    // macrotask tick guarantees every already-queued microtask (the
    // continuation included) has drained first.
    await new Promise((resolve) => setTimeout(resolve, 0));
    flushSync();

    // A discard would have replaced the entity (newDocument() proceeding past
    // the confirmDiscard() guard, which discards on `true`); Escape-as-cancel
    // must leave it exactly as it was, with the dirtying edit still present.
    expect(store.entity.name).toBe('a dirtying edit');
  });
});
