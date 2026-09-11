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
// S4 (full-audit UX): the native OS window title never reflected the open file
// name or unsaved state — only `document.title` did (the existing `$effect`
// just above this file's App.svelte:163), which a Tauri window does NOT mirror
// into its own chrome automatically. `getCurrentWindow` is hoisted so the
// SAME mock function backs every `getCurrentWindow()` call, letting tests
// assert on it directly.
const { setTitleMock } = vi.hoisted(() => ({ setTitleMock: vi.fn().mockResolvedValue(undefined) }));
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({ setTitle: setTitleMock }),
}));

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
  confirmDiscard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
  setAppMenu: vi.fn(),
  onMenuAction: vi.fn(),
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
    ability_funding: 'pool',
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
  setTitleMock.mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.setAppMenu).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.onMenuAction)
    .mockReset()
    .mockResolvedValue(() => {});
  vi.mocked(ipc.updateCloseGuard).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.saveEntity).mockReset().mockResolvedValue(null);
  vi.mocked(ipc.loadEntity).mockReset().mockResolvedValue(null);
  // Reset alongside save/load (C3c): the export shortcut tests assert on call
  // COUNTS, and a spy carrying the previous test's call makes "did not export"
  // unprovable.
  vi.mocked(ipc.exportMarkdown).mockReset().mockResolvedValue(null);
  vi.mocked(ipc.exportLabelKeys).mockReset().mockResolvedValue([]);
  // `null` = "this build has no native discard dialog", which is what the
  // `e2e-testing` binary answers and what sends the frontend to the in-app
  // fallback modal — the one the focus tests below are about. A test wanting the
  // native path overrides this.
  vi.mocked(ipc.confirmDiscard).mockReset().mockResolvedValue(null);
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
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

/**
 * Wait for the in-app fallback modal to actually be on screen. Since C3b the
 * confirmation starts with an IPC round-trip to the native dialog, so the modal
 * no longer appears in the same synchronous turn as the New/Open click.
 */
async function awaitInAppPrompt(): Promise<void> {
  await vi.waitFor(() => {
    flushSync();
    expect(store.discardPromptOpen).toBe(true);
  });
}

/**
 * Wait for the confirmation to be fully over. Answering the modal resolves the
 * user's choice, but `discardConfirmPending` — what the focus-restoring
 * `$effect` and the shell's `inert` read — only drops on the `finally` a
 * microtask later, so asserting on focus before this would read the state
 * mid-flight and pass whether or not the effect ever ran.
 */
async function awaitConfirmationSettled(): Promise<void> {
  await vi.waitFor(() => {
    flushSync();
    expect(store.discardConfirmPending).toBe(false);
  });
}

describe('focus restoration around the discard-changes prompt (S1/S4)', () => {
  /**
   * Whatever had focus when the confirmation was raised.
   *
   * These tests used to focus the toolbar's own New/Open buttons, because a
   * click on one was how a discard was reached. C3c removed that toolbar, so
   * New and Open now arrive from the native menu or from Ctrl+N/Ctrl+O — and
   * in BOTH cases focus is wherever the user left it, never on a control that
   * belongs to the action. Which makes the character-name field the honest
   * trigger to test with: someone typing a name, pressing Ctrl+N, then thinking
   * better of it should be returned to the field they were typing in. The
   * mechanism under test is unchanged; only the element standing in for "the
   * thing that had focus" is.
   */
  function focusedTrigger(): HTMLElement {
    const field = document.querySelector('[data-testid="identity-name"]') as HTMLElement;
    expect(field).toBeTruthy();
    field.focus();
    expect(document.activeElement).toBe(field);
    return field;
  }

  it('returns focus where it was once a cancelled prompt closes', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    const trigger = focusedTrigger();

    store.entity.name = 'a dirtying edit';
    flushSync();

    void store.newDocument();
    await awaitInAppPrompt();

    store.resolveDiscardPrompt(false);
    await awaitConfirmationSettled();

    expect(document.activeElement).toBe(trigger);
  });

  it('returns focus where it was once a confirmed load lands back in the editor', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/example.armc.json',
      entity: loadableEntity(),
    });

    const trigger = focusedTrigger();

    store.entity.name = 'a dirtying edit';
    flushSync();

    void store.open();
    await awaitInAppPrompt();

    store.resolveDiscardPrompt(true);
    await awaitConfirmationSettled();

    // open() sets view = 'editor', which it already was, so the field is never
    // unmounted — this is the "trigger survives" half of the fix.
    expect(document.activeElement).toBe(trigger);
  });

  it('does not force focus onto a trigger that is no longer in the document', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    const trigger = focusedTrigger();
    const focusSpy = vi.spyOn(trigger, 'focus');

    store.entity.name = 'a dirtying edit';
    flushSync();

    void store.newDocument();
    await awaitInAppPrompt();

    // Simulate the trigger having left the document by the time the prompt
    // resolves — App.svelte's own comment describes exactly this case: "the
    // old element is disconnected and .focus() is skipped". This is the common
    // case for New, which navigates to the startup screen and unmounts the
    // whole editor the trigger belonged to.
    trigger.remove();

    store.resolveDiscardPrompt(false);
    await awaitConfirmationSettled();

    expect(focusSpy).not.toHaveBeenCalled();
  });

  // C3b: the native dialog is parented to the window but NOT input-modal on
  // Linux (the same limitation `busy` covers for the native file dialogs), so
  // the shell has to be switched off for it too — otherwise the user can keep
  // editing the character behind the very dialog asking whether to throw those
  // edits away. `discardPromptOpen` cannot carry this: there is no in-app modal
  // on the native path.
  it('switches the shell off while the NATIVE discard confirmation is pending', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    // A confirmation that has not answered yet models a native dialog still on
    // screen. Resolvable, so the pending flag cannot leak into the next test —
    // the store is a module singleton, and a permanently-pending confirmation
    // would leave every later test's shell inert.
    let answer: (discard: boolean | null) => void = () => {};
    vi.mocked(ipc.confirmDiscard).mockReturnValue(
      new Promise<boolean | null>((resolve) => {
        answer = resolve;
      }),
    );
    store.entity.name = 'a dirtying edit';
    flushSync();

    const shell = document.querySelector('[data-testid="app-shell"]') as HTMLElement;
    expect(shell.inert).toBe(false);

    const pending = store.newDocument();
    flushSync();

    expect(store.discardConfirmPending).toBe(true);
    // Nothing in-app is showing: this is the native dialog's own pending state.
    expect(store.discardPromptOpen).toBe(false);
    expect(shell.inert).toBe(true);

    answer(false);
    await pending;
    await awaitConfirmationSettled();
    expect(shell.inert).toBe(false);
  });
});

// Slice 3 (#28) gives the editor the tabs its wizard phases already had, and
// retires the Slice 2 bridge that kept the funding panel on the Abilities tab in
// the meantime. Everything below asserts on a PANEL BODY rather than the tab
// strip, which is why it is a `client` test: the active tab is component-local
// `$state` defaulting to `details` (`App.svelte`), with no store mirror, so a
// `svelte/server` render can never reach any other panel. Switching tabs needs a
// mounted instance.
function clickTab(id: string): void {
  (document.getElementById(`tab-${id}`) as HTMLElement).click();
  flushSync();
}

describe('the editor tabs Slice 3 splits out', () => {
  it('mounts the life-stage panel on the Experience tab', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    clickTab('experience');

    expect(document.querySelector('[data-testid="life-stage-panel"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="ability-funding-pool"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="ability-funding-life_stages"]')).not.toBeNull();
  });

  it('leaves the Abilities tab to the Available/Selected lists alone', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    clickTab('abilities');

    // The bridge is gone: `.region-row` is the tab's only content below the XP bar,
    // which is the arrangement #11 asked for and Slice 2 could only half-deliver.
    expect(document.querySelector('[data-testid="life-stage-panel"]')).toBeNull();
    expect(document.querySelector('.region-row')).not.toBeNull();
  });

  it('mounts Personality Traits and Reputations on their own tab', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    clickTab('personality_reputations');

    expect(document.querySelector('[data-testid="personality-add"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="reputation-empty"]')).not.toBeNull();
  });

  it('mounts the aging surface and the Longevity Ritual on the Aging tab', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    clickTab('aging');

    expect(document.querySelector('[data-testid="aging-panel"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="aging-record"]')).not.toBeNull();
    expect(document.querySelector('[data-testid="longevity-add"]')).not.toBeNull();
  });

  // The acceptance criterion #28 asks to be asserted rather than eyeballed: every
  // tab's `aria-controls` must name a panel that actually exists once that tab is
  // active, and the panel must point back at it. Only the active panel is
  // rendered, so this can only be checked by activating each tab in turn.
  it('resolves every tab aria-controls to the panel it labels', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();
    // The Totals tab reads `store.derived` unconditionally, so walking onto it
    // needs a complete fixture.
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

    const ids = [...document.querySelectorAll('[role="tab"]')].map((tab) => tab.id);
    expect(ids.length).toBeGreaterThan(0);

    for (const id of ids) {
      const tab = document.getElementById(id)!;
      const controls = tab.getAttribute('aria-controls')!;
      tab.click();
      flushSync();

      const panel = document.getElementById(controls);
      expect(panel, `tab ${id} controls a missing panel ${controls}`).not.toBeNull();
      expect(panel!.getAttribute('role')).toBe('tabpanel');
      expect(panel!.getAttribute('aria-labelledby')).toBe(id);
      // A tab whose panel gates itself to nothing is the failure #28's fix must
      // not introduce, so every panel has to carry something.
      expect(panel!.querySelector('*'), `panel ${controls} is empty`).not.toBeNull();
    }
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

// Slice 5 (#31) gave the guard a NEW source of dirtiness: advancing a wizard step
// records the furthest phase on the entity, so a Next marks the document changed
// even on a step left empty. The guard is a mandatory product behavior (CLAUDE.md),
// so the `$effect` mirror has to fire for that source too — and only a mounted
// component runs an `$effect` body, which is why this case cannot live in the ssr
// suite: an assertion placed after an effect that never runs still reports green.
// S3 (full-audit i18n): `<html lang="en">` in index.html is static markup that
// never updates once the user switches the UI language at runtime, so
// assistive tech keeps announcing German text with an English voice/
// pronunciation. The fix is an `$effect` in App.svelte keyed on `store.lang`
// that writes `document.documentElement.lang`. SSR never runs an `$effect`
// body, so this has to be a mounted-component (`client`) test or the effect
// would never actually execute and the assertion would pass vacuously.
describe('document language attribute tracks the active UI language (S3)', () => {
  it('sets document.documentElement.lang to the initial language on mount', async () => {
    await mountApp();
    expect(document.documentElement.lang).toBe('en');
  });

  it('updates document.documentElement.lang when the user switches language', async () => {
    await mountApp();
    expect(document.documentElement.lang).toBe('en');

    store.lang = 'de';
    flushSync();

    expect(document.documentElement.lang).toBe('de');
  });
});

// `app.css` ships two palettes and picks between them on `<html data-theme>`, so
// the whole light/dark/auto mechanism is one `$effect` writing that attribute and
// one subscription to the OS preference. Neither is observable under SSR: an
// effect body never runs there, and an assertion placed after one that never
// fires still reports green — the exact silent gap the `client` project was
// created for.
describe('the palette follows the OS unless told otherwise', () => {
  /** A `MediaQueryList` stand-in whose listeners the test can inspect and fire. */
  interface FakeQuery {
    matches: boolean;
    readonly listeners: Set<(event: MediaQueryListEvent) => void>;
  }

  let realMatchMedia: typeof globalThis.matchMedia | undefined;
  let query: FakeQuery;

  /** Install a fake `matchMedia` reporting `prefersLight`, and return the query. */
  function stubMatchMedia(prefersLight: boolean): FakeQuery {
    const listeners = new Set<(event: MediaQueryListEvent) => void>();
    const fake: FakeQuery = { matches: prefersLight, listeners };
    globalThis.matchMedia = ((media: string) => ({
      media,
      matches: fake.matches,
      addEventListener: (_type: string, listener: (event: MediaQueryListEvent) => void) => {
        listeners.add(listener);
      },
      removeEventListener: (_type: string, listener: (event: MediaQueryListEvent) => void) => {
        listeners.delete(listener);
      },
    })) as unknown as typeof globalThis.matchMedia;
    return fake;
  }

  /** What the OS would deliver when the user switches their desktop theme. */
  function emitOsChange(matches: boolean): void {
    query.matches = matches;
    for (const listener of query.listeners) listener({ matches } as MediaQueryListEvent);
    flushSync();
  }

  beforeEach(() => {
    realMatchMedia = globalThis.matchMedia;
    query = stubMatchMedia(false);
    store.theme = 'auto';
    store.osPrefersLight = false;
  });

  afterEach(() => {
    if (realMatchMedia) globalThis.matchMedia = realMatchMedia;
    store.theme = 'auto';
    store.osPrefersLight = false;
    document.documentElement.removeAttribute('data-theme');
  });

  it('names the resolved palette on <html>, never the unresolved choice', async () => {
    await mountApp();

    // 'auto' is a preference, not a palette: the stylesheet has a `:root` and a
    // `:root[data-theme='light']` and no rule whatsoever for 'auto'.
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('starts on the light palette when that is what the OS already prefers', async () => {
    query = stubMatchMedia(true);
    await mountApp();

    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });

  // The point of `auto`, and the half a startup-only read silently drops: a user
  // whose desktop flips to light at dusk would otherwise sit in a dark app until
  // they restarted it.
  it('repaints on a LIVE OS switch rather than reading the preference once', async () => {
    await mountApp();
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');

    emitOsChange(true);
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');

    emitOsChange(false);
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('stops following the OS the moment the choice becomes explicit', async () => {
    await mountApp();

    store.theme = 'light';
    flushSync();
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');

    // The subscription stays live — 'auto' must still work if the user goes back
    // to it — but it may no longer decide the palette.
    emitOsChange(false);
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');
  });

  it('unsubscribes from the OS preference when the app goes away', async () => {
    await mountApp();
    expect(query.listeners.size).toBe(1);

    unmount(app!);
    app = undefined;
    flushSync();

    // A `MediaQueryList` outlives the component that subscribed to it, so a
    // listener left behind keeps writing into the store forever — and every
    // remount adds another.
    expect(query.listeners.size).toBe(0);
  });
});

// S4 (full-audit UX): App.svelte already computed the right localized title
// string into `document.title` (the `app-title-document(-dirty)` Fluent
// keys — NOT `app-document-name(-dirty)`, which are shaped for the on-screen
// `doc-status` chip beside the app logo and carry no " — app" suffix at all),
// but a Tauri window's native chrome does not read `document.title` — only
// `getCurrentWindow().setTitle(...)` reaches it. So the title bar itself never
// showed the open file name or the unsaved marker. SSR never runs an `$effect`
// body, so this can only be proven mounted.
describe('the native window title reflects the open document (S4)', () => {
  it('sets no document-specific title before a file has ever been saved', async () => {
    await mountApp();
    expect(setTitleMock).toHaveBeenCalledWith(store.t('app-title'));
  });

  it('sets the native title to the file name once a file is tracked', async () => {
    await mountApp();
    store.currentPath = '/tmp/example.armc.json';
    flushSync();

    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document', { name: 'example.armc.json', app: store.t('app-title') }),
    );
  });

  it('marks the native title dirty with the same ASCII marker as document.title', async () => {
    await mountApp();
    store.currentPath = '/tmp/example.armc.json';
    flushSync();
    setTitleMock.mockClear();

    store.entity.name = 'a dirtying edit';
    flushSync();

    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document-dirty', { name: 'example.armc.json', app: store.t('app-title') }),
    );
    expect(setTitleMock.mock.lastCall![0]).toBe(document.title);
  });

  it('drops the dirty marker once the edit is saved back to the baseline', async () => {
    await mountApp();
    store.currentPath = '/tmp/example.armc.json';
    flushSync();

    store.entity.name = 'a dirtying edit';
    flushSync();
    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document-dirty', { name: 'example.armc.json', app: store.t('app-title') }),
    );

    delete store.entity.name;
    flushSync();
    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document', { name: 'example.armc.json', app: store.t('app-title') }),
    );
  });
});

describe('the unsaved-changes guard mirrors wizard progress (S5/#31)', () => {
  it('re-mirrors as dirty when the wizard advances a step', async () => {
    await mountApp();
    store.view = 'editor';
    // A rail to move along — the fixture profile declares no phases — and a loaded
    // character, whose clean baseline leaves the Next below as the only thing that
    // can dirty the document. The ruleset is rebuilt per test, so this leaks nowhere.
    store.ruleset!.ruleset.type_profiles.companion.creation_phases = ['concept', 'characteristics'];
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/example.armc.json',
      entity: loadableEntity(),
    });
    await store.open();
    flushSync();
    expect(store.dirty).toBe(false);

    const before = vi.mocked(ipc.updateCloseGuard).mock.calls.length;
    store.wizardNext();
    flushSync();

    expect(vi.mocked(ipc.updateCloseGuard).mock.calls.length).toBeGreaterThan(before);
    expect(lastMirroredDirty()).toBe(true);
  });

  it('opens a migrated legacy save clean, so no spurious discard prompt appears (Slice 0)', async () => {
    // Slice 0 folds a v0.2.x save's typed `being` labels onto `being.*` ids. That
    // fold REWRITES the entity, and the unsaved-changes guard is a mandatory
    // product behavior (CLAUDE.md) — so if the rewrite landed anywhere after the
    // baseline snapshot, every migrated save would open dirty and prompt to
    // discard changes the user never made. It happens inside Rust's
    // `load_entity_migrating`, i.e. before the entity crosses the IPC boundary, so
    // the frontend only ever sees the post-migration shape; `open()` must snapshot
    // THAT and nothing earlier. Asserted rather than assumed, and asserted here
    // because the mirror is an `$effect` that only a mounted component runs.
    await mountApp();
    store.view = 'editor';
    const migrated: Entity = {
      ...loadableEntity(),
      selections: [
        { ref: 'flaw.offensive_to_beings', params: { being: 'being.mundane_humans' } },
        { ref: 'flaw.unbearable_to_beings', params: { being: 'being.demons' } },
        // The two choices the migration cannot know stay unfilled — a paramless
        // row must not make the document dirty either.
        { ref: 'virtue.folk_magic' },
        { ref: 'flaw.slow_power' },
      ],
    };
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/v0.2.0-character.armc.json',
      entity: migrated,
    });

    await store.open();
    flushSync();

    expect(store.dirty).toBe(false);
    expect(lastMirroredDirty()).toBe(false);
    // And the frontend passed the migrated values through untouched, so the
    // baseline it snapshotted is the file's own content.
    expect(store.entity.selections).toEqual(migrated.selections);
  });

  it('opens a save whose params the frontend does not renormalize, so no prompt appears (Slice 2)', async () => {
    // Slice 2 trims every parameter value, and — like slice 0's fold — it does so
    // in Rust's `load_entity_migrating`, BEFORE the entity crosses IPC. That
    // placement is the whole point: `Entity::normalize()` runs on every *save*, so
    // trimming there would reorder rows in an existing file at save time, and
    // trimming in the frontend's `open()` would rewrite the entity AFTER `#snapshot()`
    // takes the baseline — making every such save open dirty and prompt to discard
    // changes the user never made. The unsaved-changes guard is a mandatory product
    // behavior (CLAUDE.md), so this is asserted, not assumed.
    //
    // The fixture deliberately carries a value that is STILL padded: whatever the
    // engine hands over is by definition the baseline, and the frontend must pass it
    // through untouched. If a well-meaning trim were ever added to the load path
    // after the snapshot, `dirty` would flip to true here.
    await mountApp();
    store.view = 'editor';
    const loaded: Entity = {
      ...loadableEntity(),
      selections: [{ ref: 'flaw.lesser_power', params: { power: ' Wolf Shape ' } }],
    };
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/padded-character.armc.json',
      entity: loaded,
    });

    await store.open();
    flushSync();

    expect(store.dirty).toBe(false);
    expect(lastMirroredDirty()).toBe(false);
    expect(store.entity.selections).toEqual(loaded.selections);
  });

  it('does not re-mirror as dirty for rail navigation', async () => {
    await mountApp();
    store.view = 'editor';
    store.ruleset!.ruleset.type_profiles.companion.creation_phases = ['concept', 'characteristics'];
    // A file that already recorded progress: its rail opens as far as the stored
    // phase with no edit, so every move below is pure browsing.
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/example.armc.json',
      entity: { ...loadableEntity(), wizard_furthest_phase: 'characteristics' },
    });
    await store.openIntoWizard();
    flushSync();
    expect(store.view).toBe('wizard');
    expect(store.wizardFurthest).toBeGreaterThan(0);
    expect(store.dirty).toBe(false);

    store.wizardGoTo(0);
    store.wizardGoTo(store.wizardFurthest);
    flushSync();

    expect(lastMirroredDirty()).toBe(false);
  });
});

// C3a: the native application menu. Its shape is Rust's, but its text and its
// enabled state are pushed from here — and both change at runtime, so the push
// is an `$effect` and the menu is REBUILT rather than installed once. Neither
// half is observable under SSR: an effect body never runs there, so an
// assertion placed after one that never fires still reports green.
describe('the native application menu', () => {
  /** The arguments of the most recent `set_app_menu` call. */
  function lastMenu(): { labels: Record<string, string>; flags: Record<string, boolean> } {
    const calls = vi.mocked(ipc.setAppMenu).mock.calls;
    if (calls.length === 0) throw new Error('set_app_menu was never called');
    const [labels, flags] = calls[calls.length - 1];
    return { labels: labels as unknown as Record<string, string>, flags };
  }

  /** Deliver a menu click exactly as the Rust menu-event bridge would. */
  function chooseMenuItem(id: string): void {
    const calls = vi.mocked(ipc.onMenuAction).mock.calls;
    expect(calls.length, 'the app never subscribed to menu events').toBeGreaterThan(0);
    calls[calls.length - 1][0](id);
    flushSync();
  }

  it('installs the menu on mount, in the active language', async () => {
    await mountApp();

    expect(ipc.setAppMenu).toHaveBeenCalled();
    expect(lastMenu().labels.file).toBe(store.t('menu-file'));
  });

  // A menu built once at startup keeps the language it was built in forever,
  // and the app lets the user switch language at runtime — so the whole menu
  // bar would stay English behind a German UI.
  it('rebuilds the whole menu when the UI language changes', async () => {
    await mountApp();
    expect(lastMenu().labels.file).toBe('File');
    const before = vi.mocked(ipc.setAppMenu).mock.calls.length;

    store.lang = 'de';
    flushSync();

    expect(vi.mocked(ipc.setAppMenu).mock.calls.length).toBeGreaterThan(before);
    expect(lastMenu().labels.file).toBe('Datei');
    expect(lastMenu().labels.saveAs).toBe('Speichern unter…');
  });

  // The menu has no `disabled` attribute and no window listener to gate it, so
  // the store's availability predicate has to be pushed across as data — and
  // pushed again whenever the answer changes.
  it('re-pushes the enabled state when an action becomes available', async () => {
    await mountApp();
    store.view = 'start';
    flushSync();
    expect(lastMenu().flags).toEqual(store.menuFlags());
    expect(lastMenu().flags.save).toBe(false);

    store.view = 'editor';
    flushSync();

    expect(lastMenu().flags.save).toBe(true);
    // C4 gave Settings a screen, so it is live everywhere — including the startup
    // screen, since the preferences are not about a document.
    expect(lastMenu().flags.settings).toBe(true);
  });

  it('runs the very store action the chosen item names', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();
    vi.mocked(ipc.loadEntity).mockResolvedValue(null);

    chooseMenuItem('menu.open');

    await vi.waitFor(() => expect(ipc.loadEntity).toHaveBeenCalled());
  });

  it('obeys the same gate the keyboard shortcuts obey', async () => {
    await mountApp();
    store.view = 'start';
    flushSync();

    // `menu.settings` stood beside `menu.save` here as the other withheld item.
    // C4 gave it a screen, so it is no longer withheld and the claim moved to
    // "opens the settings dialog from the menu" below.
    chooseMenuItem('menu.save');

    expect(ipc.saveEntity).not.toHaveBeenCalled();
  });

  // The settings dialog (C4), reachable from the native menu's Settings item —
  // which C3a shipped present-but-disabled precisely so it would promise nothing
  // until this screen existed.
  it('opens the settings dialog from the menu, on the startup screen too', async () => {
    await mountApp();
    store.view = 'start';
    flushSync();
    expect(document.querySelector('[data-testid="settings-dialog"]')).toBeNull();

    chooseMenuItem('menu.settings');
    flushSync();

    expect(document.querySelector('[data-testid="settings-dialog"]')).not.toBeNull();
    store.closeSettings();
    flushSync();
  });

  it('makes the shell inert while the settings dialog is open', async () => {
    await mountApp();
    store.openSettings();
    flushSync();

    // The same containment the discard confirmation gets: without it a keyboard
    // user tabbing forward would walk the whole live, visually-obscured app before
    // ever reaching the dialog, and assistive tech would announce it all.
    expect(document.querySelector('[data-testid="app-shell"]')?.hasAttribute('inert')).toBe(true);

    store.closeSettings();
    flushSync();
    expect(document.querySelector('[data-testid="app-shell"]')?.hasAttribute('inert')).toBe(false);
  });

  it('ignores an id it does not know, rather than throwing', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    expect(() => chooseMenuItem('menu.nonsense')).not.toThrow();
    expect(ipc.saveEntity).not.toHaveBeenCalled();
  });

  // A menu that never appeared, with nothing said about it, reads as "this app
  // has no menu" rather than "something went wrong". `AppError::Menu` exists so
  // the banner can say which of the two it is.
  it('surfaces a failed menu build instead of leaving it silent', async () => {
    const failure = { kind: 'menu', message: 'the window system refused' };
    vi.mocked(ipc.setAppMenu).mockReset().mockRejectedValue(failure);

    await mountApp();

    await vi.waitFor(() => expect(store.error).toEqual(failure));
  });

  it('unsubscribes from menu events when the app goes away', async () => {
    const unlisten = vi.fn();
    vi.mocked(ipc.onMenuAction).mockResolvedValue(unlisten);
    await mountApp();
    await vi.waitFor(() => expect(ipc.onMenuAction).toHaveBeenCalled());

    unmount(app!);
    app = undefined;
    flushSync();

    expect(unlisten).toHaveBeenCalled();
  });
});

// The window-level Ctrl/Cmd shortcuts had their own hand-written copy of the
// busy check, because `inert` on the shell does not reach a window listener.
// They now read the store's single predicate instead — and nothing covered
// them before, so a refactor could have silently retired them.
describe('the document keyboard shortcuts', () => {
  function press(key: string, options: KeyboardEventInit = {}): void {
    window.dispatchEvent(
      new KeyboardEvent('keydown', {
        key,
        ctrlKey: true,
        bubbles: true,
        cancelable: true,
        ...options,
      }),
    );
    flushSync();
  }

  it('saves with Ctrl+S while a character is being edited', async () => {
    await mountApp();
    store.view = 'editor';
    store.currentPath = '/tmp/example.armc.json';
    flushSync();
    vi.mocked(ipc.saveEntity).mockResolvedValue('/tmp/example.armc.json');

    press('s');

    await vi.waitFor(() => expect(ipc.saveEntity).toHaveBeenCalled());
  });

  it('does not save from the startup screen, which has no document', async () => {
    await mountApp();
    store.view = 'start';
    flushSync();

    press('s');

    expect(ipc.saveEntity).not.toHaveBeenCalled();
  });

  it('opens with Ctrl+O from either screen', async () => {
    await mountApp();
    store.view = 'start';
    flushSync();
    vi.mocked(ipc.loadEntity).mockResolvedValue(null);

    press('o');

    await vi.waitFor(() => expect(ipc.loadEntity).toHaveBeenCalled());
  });

  // C3c: Export lost its button with the toolbar, and a native menu item is not
  // in the webview's tab order — so without a chord of its own, Export became
  // the one document action a keyboard user inside the window could not reach.
  // Shift+E rather than a bare Ctrl+E: on GTK, Ctrl+E is the readline
  // end-of-line binding text entries answer to, and the character name field is
  // exactly where a user would press it meaning "end of line".
  it('exports with Ctrl+Shift+E while a character is being edited', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();
    vi.mocked(ipc.exportMarkdown).mockResolvedValue('/tmp/marcus.md');
    vi.mocked(ipc.exportLabelKeys).mockResolvedValue([]);

    press('e', { shiftKey: true });

    await vi.waitFor(() => expect(ipc.exportMarkdown).toHaveBeenCalled());
  });

  it('does not export from the startup screen, which has no document', async () => {
    await mountApp();
    store.view = 'start';
    flushSync();

    press('e', { shiftKey: true });

    expect(ipc.exportMarkdown).not.toHaveBeenCalled();
  });

  // A bare Ctrl+E must stay unclaimed, or the GTK end-of-line binding above is
  // exactly what this handler eats.
  it('leaves a bare Ctrl+E to the platform', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    press('e');

    expect(ipc.exportMarkdown).not.toHaveBeenCalled();
  });
});
