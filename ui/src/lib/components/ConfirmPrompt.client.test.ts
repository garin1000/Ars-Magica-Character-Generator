import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, Familiar, LocalizedRuleset } from '../types';

// A `client` test, not `ssr`, and it could not be anything else. Everything under
// test here is made of the three things SSR has none of: a live
// `document.activeElement`, keydown listeners that actually fire, and `$effect`
// bodies that actually run. An SSR assertion about "Tab wraps to Cancel" would
// compare strings, find them unchanged, and report green having exercised nothing.
//
// Sabine 2 (full-audit round 1): `ConfirmPrompt` announces itself
// `role="alertdialog" aria-modal="true"` and moves initial focus to Cancel, but had
// no Tab containment and no focus restore. `SettingsDialog` already says so in
// prose — "DiscardPrompt is contained only by accident … and ConfirmPrompt is not
// contained at all" — and unlike the other two modals this one renders INSIDE the
// app shell, so nothing else contains it either: one Tab past Confirm put focus on
// a live control that `aria-modal` had just removed from the accessibility tree.
//
// The host is `FamiliarPanel`, not a synthetic harness. `ConfirmPrompt` takes
// `open` as a prop, so a harness would need a fixture component purely to make that
// prop reactive — and `FamiliarPanel` is one of the two real render sites
// (`FamiliarPanel.svelte`, `TalismanPanel.svelte`), supplies a genuine invoker to
// restore focus to, and supplies live controls behind the dialog for the trap to
// contain focus against.
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
import FamiliarPanel from './FamiliarPanel.svelte';

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

function resetEntityWithFamiliar(): void {
  const familiar: Familiar = {
    name: 'Corax',
    animal: 'raven',
    might: null,
    characteristics: {},
    size: 0,
    personality_traits: [],
    cord_gold: 0,
    cord_silver: 0,
    cord_bronze: 0,
    powers: [],
  };
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
    familiar,
  };
  store.derived = null;
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntityWithFamiliar();
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(FamiliarPanel, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

/** The Remove button — the invoker focus must come back to. */
function removeButton(): HTMLButtonElement {
  const button = target.querySelector<HTMLButtonElement>('[data-testid="familiar-remove"]');
  expect(button).toBeTruthy();
  return button!;
}

/** Open the confirmation the way a keyboard user does: from a focused control. */
function openFromInvoker(): void {
  removeButton().focus();
  removeButton().click();
  flushSync();
}

function cancel(): HTMLButtonElement {
  return target.querySelector<HTMLButtonElement>('[data-testid="familiar-remove-confirm-cancel"]')!;
}

function confirm(): HTMLButtonElement {
  return target.querySelector<HTMLButtonElement>(
    '[data-testid="familiar-remove-confirm-confirm"]',
  )!;
}

function press(key: string, shiftKey = false): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { key, shiftKey, bubbles: true, cancelable: true });
  document.activeElement?.dispatchEvent(event);
  flushSync();
  return event;
}

describe('the confirmation prompt focus trap (Sabine 2)', () => {
  it('wraps Tab from the last control back to the first', () => {
    openFromInvoker();

    confirm().focus();
    press('Tab');

    expect(document.activeElement).toBe(cancel());
  });

  it('wraps Shift+Tab from the first control back to the last', () => {
    openFromInvoker();

    cancel().focus();
    press('Tab', true);

    expect(document.activeElement).toBe(confirm());
  });

  it('leaves a Tab that is not at either end of the ring to the browser', () => {
    // The trap closes only the two ENDS of the ring. Calling `preventDefault` on
    // every Tab would mean reimplementing the browser's focus order by hand, which
    // is how a trap starts skipping controls it does not know about.
    openFromInvoker();

    cancel().focus();
    const event = press('Tab');

    expect(event.defaultPrevented).toBe(false);
  });

  it('pulls a Tab pressed outside the dialog back inside it', () => {
    // Defence for the case the trap exists for: the shell behind this dialog stays
    // live (unlike DiscardPrompt/SettingsDialog, nothing marks it `inert`), so a
    // stray focus on a control behind the modal must not be allowed to keep Tabbing
    // through an app that `aria-modal="true"` has hidden from assistive tech.
    openFromInvoker();

    removeButton().focus();
    press('Tab');

    expect(document.activeElement).toBe(cancel());
  });
});

describe('the confirmation prompt focus restoration (Sabine 2)', () => {
  it('returns focus to the control that opened it when cancelled', () => {
    const invoker = removeButton();
    openFromInvoker();
    expect(document.activeElement).not.toBe(invoker);

    cancel().click();
    flushSync();

    expect(document.activeElement).toBe(invoker);
  });

  it('returns focus to the invoker when Escape dismisses it', () => {
    const invoker = removeButton();
    openFromInvoker();

    press('Escape');

    expect(document.activeElement).toBe(invoker);
  });

  it('leaves focus alone when confirming removes the invoker itself', () => {
    // Confirming deletes the familiar, and the Remove button goes with it. Focusing
    // a detached node silently does nothing, so the restore must skip it rather than
    // force focus onto an arbitrary substitute — the same `isConnected` guard
    // `SettingsDialog` carries.
    openFromInvoker();
    const invoker = removeButton();

    confirm().click();
    flushSync();

    expect(invoker.isConnected).toBe(false);
    expect(document.activeElement).not.toBe(invoker);
  });
});
