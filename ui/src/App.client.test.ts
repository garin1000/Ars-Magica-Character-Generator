import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { DerivedTotals, EffectiveScores, Entity, LocalizedRuleset } from './lib/types';

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
// name or unsaved state — only `document.title` did (the existing `$effect` in
// `App.svelte` that assigns it), which a Tauri window does NOT mirror
// into its own chrome automatically. `getCurrentWindow` is hoisted so the
// SAME mock function backs every `getCurrentWindow()` call, letting tests
// assert on it directly.
// U1 (P1): Window → Fullscreen had no handler outside macOS — muda's Windows
// and GTK backends never implement the predefined role at all (see
// `crates/arm-app/tests/menu.rs`'s
// `fullscreen_is_a_real_action_on_windows_and_linux_not_a_dead_predefined_item`),
// so the fix routes it through `getCurrentWindow().isFullscreen()`/`setFullscreen()`
// exactly like the title-bar effect already reaches `setTitle` — real Tauri
// window API, decoupled from muda's menu backend entirely. `isFullscreenMock`
// and `setFullscreenMock` join `setTitleMock` on the SAME hoisted mock so every
// `getCurrentWindow()` call in the component sees them.
const { setTitleMock, isFullscreenMock, setFullscreenMock } = vi.hoisted(() => ({
  setTitleMock: vi.fn().mockResolvedValue(undefined),
  isFullscreenMock: vi.fn().mockResolvedValue(false),
  setFullscreenMock: vi.fn().mockResolvedValue(undefined),
}));
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => ({
    setTitle: setTitleMock,
    isFullscreen: isFullscreenMock,
    setFullscreen: setFullscreenMock,
  }),
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
  menuShortcuts: vi.fn(),
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
    saga_year: 1220,
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
  isFullscreenMock.mockReset().mockResolvedValue(false);
  setFullscreenMock.mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.setAppMenu).mockReset().mockResolvedValue(undefined);
  vi.mocked(ipc.onMenuAction)
    .mockReset()
    .mockResolvedValue(() => {});
  // `[]` is what `menu_shortcuts` answers on Linux and macOS, where the menu's
  // accelerators fire natively; the U4 describe overrides it with Windows' list.
  vi.mocked(ipc.menuShortcuts).mockReset().mockResolvedValue([]);
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
// prompt's focus-restoration `$effect` (the one restoring
// `App.svelte::lastFocusOutsideDialog`, which `App.svelte::trackFocus` records)
// shipped with zero coverage anywhere in the suite. `App.test.ts` renders
// through `svelte/server`
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
    saga_year: 1220,
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
      migrated_aging_characteristics: [],
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
      hermetically_trained: false,
      lab_totals: [],
      casting_totals: [],
      spell_casting_totals: [],
      spell_casting_unusable: [],
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
      hermetically_trained: false,
      lab_totals: [],
      casting_totals: [],
      spell_casting_totals: [],
      spell_casting_unusable: [],
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
// string into `document.title` (the `app-title-document(-dirty)` Fluent keys),
// but a Tauri window's native chrome does not read `document.title` — only
// `getCurrentWindow().setTitle(...)` reaches it. So the title bar itself never
// showed the open file name or the unsaved marker. SSR never runs an `$effect`
// body, so this can only be proven mounted.
//
// P3/U3 (`docs/open-todos.md`) later retired the doc-status chip these keys'
// old comment contrasted against — the on-screen `app-document-name(-dirty)`/
// `app-document-unsaved(-dirty)` keys it named are gone with it, and the
// window title is the SOLE surviving carrier of this state.
describe('the native window title reflects the open document (S4, P3/U3)', () => {
  // P3 (`docs/open-todos.md`): the title used to be built from the FILE name
  // alone (`store.currentFileName`), which is not what P3 asks for — "put the
  // CHARACTER name in the window title". These four tests are the coordinator's
  // four required behaviours, in order. Every scenario below explicitly sets
  // `store.view`: it is the signal that distinguishes "no character exists yet"
  // (the startup screen, bullet 4) from "a real, merely unnamed/unsaved
  // character" (bullet 3) — the fixture's own `resetEntity()` gives even the
  // startup screen a real `type_id`, unlike the shipped app, so the title
  // effect cannot tell the two apart by looking at the entity; it has to read
  // the screen.

  // Bullet 4: no entity at all.
  it('shows the bare app title on the startup screen, with no character at all', async () => {
    store.view = 'start';
    await mountApp();
    expect(setTitleMock).toHaveBeenLastCalledWith(store.t('app-title'));
  });

  // Bullet 1: the character's own name, not the file name, once one is set —
  // proven against a document that ALSO has a file tracked, so a regression
  // back to file-name titling cannot hide behind "there was no file anyway".
  it('titles the window from the character name once one is set, not the file name', async () => {
    await mountApp();
    store.view = 'editor';
    store.currentPath = '/tmp/example.armc.json';
    flushSync();
    setTitleMock.mockClear();

    store.entity.name = 'Bonisagus of Bonisagus';
    flushSync();

    // The edit dirties the document too, so this is also the dirty variant —
    // the clean one is proven by "drops the dirty marker…" below with the
    // file-name fallback, since both variants share one Fluent key pair.
    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document-dirty', {
        name: 'Bonisagus of Bonisagus',
        app: store.t('app-title'),
      }),
    );
  });

  // Bullet 3, first fallback: file name, once the character has no name.
  it('falls back to the file name once a file is tracked and the character has no name', async () => {
    await mountApp();
    store.view = 'editor';
    store.currentPath = '/tmp/example.armc.json';
    flushSync();

    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document', { name: 'example.armc.json', app: store.t('app-title') }),
    );
  });

  // Bullet 3, second fallback: the new localized "Untitled" label, with
  // neither a character name nor a file.
  it('falls back to the localized "Untitled" label with no character name and no file', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document', {
        name: store.t('app-title-untitled'),
        app: store.t('app-title'),
      }),
    );
  });

  // Bullet 2: the asterisk appears whenever `store.dirty` is true, INCLUDING a
  // brand-new, never-saved, unnamed character — today's code shows the bare
  // app title here with no asterisk at all, because it only ever branches on
  // `currentFileName === null` and never reads `dirty` in that branch.
  it('marks the title dirty even for a brand-new, never-saved, unnamed character', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();
    setTitleMock.mockClear();

    store.entity.description = 'a dirtying edit';
    flushSync();

    const title = store.t('app-title-document-dirty', {
      name: store.t('app-title-untitled'),
      app: store.t('app-title'),
    });
    expect(setTitleMock).toHaveBeenLastCalledWith(title);
    expect(setTitleMock.mock.lastCall![0]).toBe(document.title);
    // The dirty source stays `AppStore.dirty` alone (CLAUDE.md): the marker
    // must still be the plain ASCII asterisk that guard already uses, never a
    // second, duplicated notion of "unsaved".
    expect(title.codePointAt(0)).toBe(0x2a);
  });

  it('marks the native title dirty with the same ASCII marker as document.title', async () => {
    await mountApp();
    store.view = 'editor';
    store.currentPath = '/tmp/example.armc.json';
    flushSync();
    setTitleMock.mockClear();

    store.entity.description = 'a dirtying edit';
    flushSync();

    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document-dirty', { name: 'example.armc.json', app: store.t('app-title') }),
    );
    expect(setTitleMock.mock.lastCall![0]).toBe(document.title);
  });

  it('drops the dirty marker once the edit is saved back to the baseline', async () => {
    await mountApp();
    store.view = 'editor';
    store.currentPath = '/tmp/example.armc.json';
    flushSync();

    store.entity.description = 'a dirtying edit';
    flushSync();
    expect(setTitleMock).toHaveBeenLastCalledWith(
      store.t('app-title-document-dirty', { name: 'example.armc.json', app: store.t('app-title') }),
    );

    delete store.entity.description;
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
      migrated_aging_characteristics: [],
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
      migrated_aging_characteristics: [],
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
      migrated_aging_characteristics: [],
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
      migrated_aging_characteristics: [],
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

  // U1/P1. This is the "no browser.keys" activation proof `crates/arm-app/tests/menu.rs`
  // cannot give on its own: that file proves the Rust MODEL now offers a real
  // `menu.fullscreen` action instead of a dead predefined item, but nothing
  // there can prove a click on it actually does anything — there is no live
  // Tauri window in a `cargo test` run. This is: `chooseMenuItem` delivers the
  // id exactly as the Rust menu-event bridge would, and a mounted component is
  // required because `getCurrentWindow()` is only ever called from a live
  // effect/handler, never under SSR.
  describe('the fullscreen menu action actually toggles the window (U1/P1)', () => {
    it('enters fullscreen when the window is not already fullscreen', async () => {
      await mountApp();
      isFullscreenMock.mockResolvedValue(false);

      chooseMenuItem('menu.fullscreen');

      await vi.waitFor(() => expect(setFullscreenMock).toHaveBeenCalled(), {
        timeout: 300,
        interval: 20,
      });
      expect(setFullscreenMock).toHaveBeenCalledWith(true);
    });

    it('exits fullscreen when the window is already fullscreen', async () => {
      await mountApp();
      isFullscreenMock.mockResolvedValue(true);

      chooseMenuItem('menu.fullscreen');

      await vi.waitFor(() => expect(setFullscreenMock).toHaveBeenCalled(), {
        timeout: 300,
        interval: 20,
      });
      expect(setFullscreenMock).toHaveBeenCalledWith(false);
    });
  });
});

// C7: ONE OWNER PER CHORD, and the owner is the native menu item.
//
// Until this slice the webview held its own `keydown` handler for Ctrl+N/O/S,
// Ctrl+Shift+S and Ctrl+Shift+E, because C3a and C3c shipped the menu with no
// accelerators at all — deliberately, to avoid exactly the collision this
// describe now guards. The accelerators are declared on the menu items
// (`accelerator_for`, `crates/arm-app/src/menu.rs`) and GTK/macOS/Windows
// dispatch them, so the handler had to go: a surviving copy would see the same
// press the accel group sees, and one Ctrl+N on a dirty document would raise
// two discard prompts.
//
// These tests replace six that asserted the opposite (`saves with Ctrl+S`,
// `opens with Ctrl+O`, `exports with Ctrl+Shift+E`, their two startup-screen
// negatives and `leaves a bare Ctrl+E to the platform`). They were not deleted
// as redundant — the claim they pinned was retired, and its replacement is
// this. What the chords DO is now proved against the shipped binary, by the
// accelerator describe in `ui/e2e/specs/app-shell.e2e.js`, which is the only
// layer where a real accel group exists to press.
//
// U4 (finding 16) carves out Windows, where WebView2 keeps the keys from the
// host's accelerator table: there the webview mirrors the chords from the list
// Rust's `menu_shortcuts` returns (next describe). Everywhere else that list is
// empty — the default `beforeEach` stubs — and this describe still holds.
describe('the document chords belong to the native menu, not the webview', () => {
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

  /** Every chord the menu claims, spelled as the webview would see it. */
  const MENU_CHORDS: [string, KeyboardEventInit][] = [
    ['n', {}],
    ['o', {}],
    ['s', {}],
    ['s', { shiftKey: true }],
    ['e', { shiftKey: true }],
    [',', {}],
  ];

  // Asserted on the store's DISPATCHER rather than on the IPC each action ends
  // at. `runDocumentAction` is the one thing a webview handler could possibly
  // call — it is what the deleted `handleShortcut` called, and what the menu
  // bridge still calls — so watching it catches a surviving handler whatever
  // the action does afterwards. Watching IPC instead would have passed
  // vacuously for Export, whose route awaits `exportLabelKeys` first and so
  // reaches no spy within the tick.
  it.each(MENU_CHORDS)('dispatches nothing of its own on Ctrl+%s', async (key, options) => {
    await mountApp();
    store.view = 'editor';
    store.currentPath = '/tmp/example.armc.json';
    flushSync();
    const dispatch = vi.spyOn(store, 'runDocumentAction').mockResolvedValue();

    press(key, options);
    await Promise.resolve();

    expect(dispatch).not.toHaveBeenCalled();
    dispatch.mockRestore();
  });

  // The other half of "nowhere else": a bare Ctrl+E is not the menu's either,
  // so nothing may eat the GTK end-of-line binding a text entry answers to.
  it('leaves a bare Ctrl+E to the platform', async () => {
    await mountApp();
    store.view = 'editor';
    flushSync();

    press('e');

    expect(ipc.exportMarkdown).not.toHaveBeenCalled();
  });
});

// U4 (try-out finding 16): on Windows, Ctrl+O and Ctrl+N did nothing. The File
// menu draws the chords, but WebView2's child window keeps the keys, so the
// host's accelerator table never sees them. The decided fix (Norbert, option a)
// is a Windows-only webview mirror: Rust's `menu_shortcuts` returns the menu's
// own chords on Windows and `[]` elsewhere, and the app runs a press through the
// same `runMenuAction` a menu click takes, gated by the same flags
// `set_app_menu` receives. The list here is what Windows would return.
describe('on Windows the webview mirrors the menu chords (U4)', () => {
  const WINDOWS_LIST = [
    { action: 'menu.new', accelerator: 'CmdOrCtrl+N' },
    { action: 'menu.open', accelerator: 'CmdOrCtrl+O' },
    { action: 'menu.save', accelerator: 'CmdOrCtrl+S' },
    { action: 'menu.save-as', accelerator: 'CmdOrCtrl+Shift+S' },
    { action: 'menu.export', accelerator: 'CmdOrCtrl+Shift+E' },
    { action: 'menu.settings', accelerator: 'CmdOrCtrl+,' },
  ];

  function press(key: string, options: KeyboardEventInit = {}): KeyboardEvent {
    const event = new KeyboardEvent('keydown', {
      key,
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
      ...options,
    });
    window.dispatchEvent(event);
    flushSync();
    return event;
  }

  /** Mount with Windows' list and wait until the app has asked for it. */
  async function mountOnWindows(): Promise<void> {
    vi.mocked(ipc.menuShortcuts).mockResolvedValue(WINDOWS_LIST);
    await mountApp();
    await vi.waitFor(() => expect(ipc.menuShortcuts).toHaveBeenCalled(), {
      timeout: 300,
      interval: 20,
    });
    await Promise.resolve();
  }

  it('asks Rust for the list exactly once', async () => {
    await mountOnWindows();

    expect(ipc.menuShortcuts).toHaveBeenCalledTimes(1);
  });

  it('runs the open action once on Ctrl+O, as a menu click would', async () => {
    await mountOnWindows();
    const dispatch = vi.spyOn(store, 'runDocumentAction').mockResolvedValue();

    const event = press('o');

    expect(dispatch).toHaveBeenCalledTimes(1);
    expect(dispatch).toHaveBeenCalledWith('open');
    expect(event.defaultPrevented).toBe(true);
    dispatch.mockRestore();
  });

  // The startup screen withholds Save (`store.menuFlags().save === false`), and
  // the native item is greyed out there; the mirror must not fire either — and
  // must leave the key alone, since nothing claimed it.
  it('runs nothing and cancels nothing for an action the menu withholds', async () => {
    await mountOnWindows();
    store.view = 'start';
    flushSync();
    expect(store.menuFlags().save).toBe(false);
    const dispatch = vi.spyOn(store, 'runDocumentAction').mockResolvedValue();

    const event = press('s');

    expect(dispatch).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
    dispatch.mockRestore();
  });

  it('stops listening when the app goes away', async () => {
    await mountOnWindows();
    const dispatch = vi.spyOn(store, 'runDocumentAction').mockResolvedValue();
    // The positive control first, so the negative below is not vacuous.
    press('o');
    expect(dispatch).toHaveBeenCalledTimes(1);

    unmount(app!);
    app = undefined;
    flushSync();
    press('o');

    expect(dispatch).toHaveBeenCalledTimes(1);
    dispatch.mockRestore();
  });
});

// Sabine 14 (full-audit round 1): every tab carried `title={store.t(t.key)}` —
// character-for-character the same Fluent string as its visible text. With no
// `aria-label`, the button's content IS its accessible name, so `title` becomes
// its accessible DESCRIPTION, and AT announces the label twice on every tab stop
// ("Abilities, Abilities") at every window width.
//
// The `title` is not deleted, because the need behind it is real: the strip is
// one line and ellipsizes (app.css), and German's thirteen-tab magus set clips
// at the 900px the window is resizable down to — a clipped label with no `title`
// is unreadable. So it is made CONDITIONAL on the label actually being clipped,
// which is the only state that ever justified it.
//
// A `client` test necessarily: the action measures `scrollWidth`/`clientWidth`
// on a live node, and SSR emits no node to measure. happy-dom performs no
// layout, so both widths are 0 there — the clipping is stubbed on the prototype
// rather than pretended into existence.
describe('a tab carries its full label in `title` only while the strip clips it (Sabine 14)', () => {
  /**
   * Make every element report overflow (or not), and hand back the undo.
   * Stubbed on the prototype because the action measures during its own setup,
   * before a test could reach the instance.
   */
  function stubClipping(clipped: boolean): () => void {
    const widths: Record<string, number> = { scrollWidth: clipped ? 200 : 100, clientWidth: 100 };
    const saved = new Map<string, PropertyDescriptor | undefined>();
    for (const [name, value] of Object.entries(widths)) {
      saved.set(name, Object.getOwnPropertyDescriptor(HTMLElement.prototype, name));
      Object.defineProperty(HTMLElement.prototype, name, { configurable: true, get: () => value });
    }
    return () => {
      for (const [name, descriptor] of saved) {
        if (descriptor) Object.defineProperty(HTMLElement.prototype, name, descriptor);
        else delete (HTMLElement.prototype as unknown as Record<string, unknown>)[name];
      }
    };
  }

  function tabButtons(): HTMLElement[] {
    return [...document.querySelectorAll<HTMLElement>('[role="tab"]')];
  }

  it('sets no title while every label fits', async () => {
    const restore = stubClipping(false);
    try {
      await mountApp();
      store.view = 'editor';
      flushSync();

      const tabs = tabButtons();
      expect(tabs.length).toBeGreaterThan(0);
      // The defect: a description that merely repeats the name, on every tab
      // stop, at a width where nothing is truncated at all.
      for (const tab of tabs) {
        expect(tab.hasAttribute('title'), `${tab.id} should carry no title`).toBe(false);
      }
    } finally {
      restore();
    }
  });

  it('restores the full label in `title` once the label is clipped', async () => {
    const restore = stubClipping(true);
    try {
      await mountApp();
      store.view = 'editor';
      flushSync();

      const tabs = tabButtons();
      expect(tabs.length).toBeGreaterThan(0);
      for (const tab of tabs) {
        // The same Fluent string as the visible text, never a second wording.
        expect(tab.getAttribute('title')).toBe(tab.textContent?.trim());
      }
    } finally {
      restore();
    }
  });
});

// Erika F8 (full-audit round 1): the LAST `$effect` in App.svelte with no
// coverage. Seven of its eight have named tests in this file (the close-guard
// mirror, `<html lang>`, the palette, the live OS switch, the window title, the
// menu rebuild, focus restoration); the tab-fallback one had none.
//
// `App.test.ts` asserts only that a tab is ABSENT from a freshly rendered strip,
// which says nothing about the TRANSITION — and the transition is where the
// damage is. Removing the Focus Power Virtue while sitting on the Supernatural
// tab leaves `tab` naming a tab that no longer exists: the content area renders
// nothing, and `onTabsKeydown` computes `current === -1` and returns early, so
// Arrow/Home/End navigation is dead and no tab carries `aria-selected="true"`.
// A stuck editor, one click away, with every gate green.
//
// A `client` test necessarily: SSR never executes an `$effect` body, so the
// assertions below would compare a strip that had never been corrected and pass
// vacuously. (Verified by neutering the effect — see the fix report's RED.)
describe('the active tab falls back when its own tab disappears (F8)', () => {
  /** Mount, enter the editor, and open the Supernatural tab on a Focus Power pool. */
  async function onSupernaturalTab(): Promise<HTMLElement> {
    await mountApp();
    store.view = 'editor';
    store.effective = { focus_points_budget: 25, might: null } as unknown as EffectiveScores;
    flushSync();

    const supernatural = document.getElementById('tab-supernatural');
    expect(supernatural, 'a Focus Power pool should open the Supernatural tab').not.toBeNull();
    supernatural!.click();
    flushSync();
    expect(supernatural!.getAttribute('aria-selected')).toBe('true');
    return supernatural!;
  }

  /** Withdraw the Focus Power pool — the tab's only reason to exist here. */
  function removeTheGrant(): void {
    store.effective = { focus_points_budget: 0, might: null } as unknown as EffectiveScores;
    flushSync();
  }

  it('selects Details when the tab it was on is removed from the strip', async () => {
    await onSupernaturalTab();

    removeTheGrant();

    expect(document.getElementById('tab-supernatural')).toBeNull();
    expect(document.getElementById('tab-details')?.getAttribute('aria-selected')).toBe('true');
  });

  it('leaves exactly one tab selected, so the strip is never orphaned', async () => {
    // The precise failure: with `tab` naming a tab that is gone, EVERY remaining
    // tab renders `aria-selected="false"` and the panel area renders nothing.
    await onSupernaturalTab();

    removeTheGrant();

    const selected = document.querySelectorAll('[role="tab"][aria-selected="true"]');
    expect(selected.length).toBe(1);
    expect(document.querySelector('[role="tabpanel"]')?.id).toBe('tabpanel-details');
  });

  it('keeps arrow navigation alive afterwards', async () => {
    // The consequence that makes this more than cosmetic: `onTabsKeydown` looks
    // the active tab up with `findIndex` and returns early on -1, so an orphaned
    // `tab` value silently kills Arrow/Home/End for the whole strip.
    await onSupernaturalTab();
    removeTheGrant();

    const tablist = document.querySelector('[role="tablist"]') as HTMLElement;
    tablist.dispatchEvent(
      new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, cancelable: true }),
    );
    flushSync();

    expect(document.getElementById('tab-characteristics')?.getAttribute('aria-selected')).toBe(
      'true',
    );
  });
});

// A2/D56 § 6, row 20(c): the tab list used to bundle Arts/Spells/Possessions
// AND House/Order under one client-derived `isMagus` boolean, which is wrong
// the moment a character is Hermetically trained without being an Order member
// (or vice versa) — unreachable on `main` before this slice (nothing built a
// trained-non-Order character), but real once `phases_in_force` resolves each
// fact separately. Proved with a test-only `phases_in_force` DTO fixture, per
// the design note's sub-slice ordering: the shipped `flaw.abandoned_apprentice`
// entry is not touched until slice D3, so this is not yet reachable through the
// real Flaw — only through the resolved DTO field this slice wires the tab list
// to.
describe('the tab list reads phases_in_force, not a client-derived isMagus (A2/D56)', () => {
  it('shows Arts, Spells and Possessions but not House for a trained-by-selection fixture', async () => {
    await mountApp();
    store.view = 'editor';
    store.effective = {
      phases_in_force: [
        'concept',
        'characteristics',
        'virtues_flaws',
        'experience',
        'abilities',
        'arts',
        'spells',
        'personality_reputations',
        'aging',
      ],
    } as unknown as EffectiveScores;
    flushSync();

    expect(document.getElementById('tab-arts')).not.toBeNull();
    expect(document.getElementById('tab-spells')).not.toBeNull();
    expect(document.getElementById('tab-possessions')).not.toBeNull();
    expect(document.getElementById('tab-house_specialisation')).toBeNull();
  });

  it('shows House but neither Arts/Spells nor Possessions for an Order-member-without-training fixture', async () => {
    await mountApp();
    store.view = 'editor';
    store.effective = {
      phases_in_force: [
        'concept',
        'characteristics',
        'virtues_flaws',
        'experience',
        'abilities',
        'house_specialisation',
        'personality_reputations',
        'aging',
      ],
    } as unknown as EffectiveScores;
    flushSync();

    expect(document.getElementById('tab-house_specialisation')).not.toBeNull();
    expect(document.getElementById('tab-arts')).toBeNull();
    expect(document.getElementById('tab-spells')).toBeNull();
    expect(document.getElementById('tab-possessions')).toBeNull();
  });
});
