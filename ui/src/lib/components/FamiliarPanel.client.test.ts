import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, Familiar, LocalizedRuleset } from '../types';

// S18 (tmp/review/BACKLOG-medium-ranked.md, Tier 1): "Remove familiar" used to
// discard the entire statblock — name, Might, Characteristics, personality
// traits, cords, powers — on a single click, with no confirmation and no
// undo. This suite pins the fix: the click must open a confirmation (reusing
// the DiscardPrompt pattern via the new shared ConfirmPrompt component) and
// only actually clear `store.entity.familiar` once the user confirms.
// SSR never executes a click handler's *consequence* the way a mounted
// component does (no live DOM to dispatch a real `click` at), so — like
// DiscardPrompt's own focus-management tests — this belongs in the `client`
// vitest project, not the SSR one.
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
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function mountPanel(): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(FamiliarPanel, { target });
  flushSync();
}

describe('FamiliarPanel remove confirmation (S18)', () => {
  it('does not remove the familiar immediately on click — it opens a confirmation first', () => {
    mountPanel();
    const removeButton = target.querySelector(
      '[data-testid="familiar-remove"]',
    ) as HTMLButtonElement;
    expect(removeButton).toBeTruthy();

    removeButton.click();
    flushSync();

    expect(store.entity.familiar).not.toBeNull();
    const dialog = target.querySelector('[data-testid="familiar-remove-confirm-prompt"]');
    expect(dialog).toBeTruthy();
  });

  it('leaves the familiar untouched when the confirmation is cancelled', () => {
    mountPanel();
    (target.querySelector('[data-testid="familiar-remove"]') as HTMLButtonElement).click();
    flushSync();

    const cancel = target.querySelector(
      '[data-testid="familiar-remove-confirm-cancel"]',
    ) as HTMLButtonElement;
    expect(cancel).toBeTruthy();
    cancel.click();
    flushSync();

    expect(store.entity.familiar).not.toBeNull();
    expect(target.querySelector('[data-testid="familiar-remove-confirm-prompt"]')).toBeFalsy();
  });

  it('removes the familiar only once the destructive action is confirmed', () => {
    mountPanel();
    (target.querySelector('[data-testid="familiar-remove"]') as HTMLButtonElement).click();
    flushSync();

    const confirm = target.querySelector(
      '[data-testid="familiar-remove-confirm-confirm"]',
    ) as HTMLButtonElement;
    expect(confirm).toBeTruthy();
    confirm.click();
    flushSync();

    expect(store.entity.familiar).toBeNull();
    expect(target.querySelector('[data-testid="familiar-remove-confirm-prompt"]')).toBeFalsy();
  });

  it('moves initial focus to the safe Cancel control, never the destructive Remove one', () => {
    mountPanel();
    (target.querySelector('[data-testid="familiar-remove"]') as HTMLButtonElement).click();
    flushSync();

    const cancel = target.querySelector('[data-testid="familiar-remove-confirm-cancel"]');
    const confirm = target.querySelector('[data-testid="familiar-remove-confirm-confirm"]');
    expect(document.activeElement).toBe(cancel);
    expect(document.activeElement).not.toBe(confirm);
  });

  it('resolves Escape to cancel, never to remove', () => {
    mountPanel();
    (target.querySelector('[data-testid="familiar-remove"]') as HTMLButtonElement).click();
    flushSync();

    const dialog = target.querySelector('[role="alertdialog"]') as HTMLElement;
    expect(dialog).toBeTruthy();
    dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    flushSync();

    expect(store.entity.familiar).not.toBeNull();
    expect(target.querySelector('[data-testid="familiar-remove-confirm-prompt"]')).toBeFalsy();
  });
});
