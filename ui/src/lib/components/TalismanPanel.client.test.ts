import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset, Talisman } from '../types';

// S30 (tmp/review/BACKLOG-medium-ranked.md, Tier 1): "Remove talisman" used to
// discard the item's identity, attunements, and instilled effects on a single
// click, with no confirmation and no undo. Mirrors FamiliarPanel.client.test.ts
// (S18) — see that file's header for why this belongs in the `client` vitest
// project rather than the SSR one.
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
import TalismanPanel from './TalismanPanel.svelte';

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

function resetEntityWithTalisman(): void {
  const talisman: Talisman = {
    description: 'an ash staff',
    attunements: [],
    effects: [],
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
    talisman,
  };
  store.derived = null;
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntityWithTalisman();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

function mountPanel(): void {
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(TalismanPanel, { target });
  flushSync();
}

describe('TalismanPanel remove confirmation (S30)', () => {
  it('does not remove the talisman immediately on click — it opens a confirmation first', () => {
    mountPanel();
    const removeButton = target.querySelector(
      '[data-testid="talisman-remove-item"]',
    ) as HTMLButtonElement;
    expect(removeButton).toBeTruthy();

    removeButton.click();
    flushSync();

    expect(store.entity.talisman).not.toBeNull();
    const dialog = target.querySelector('[data-testid="talisman-remove-confirm-prompt"]');
    expect(dialog).toBeTruthy();
  });

  it('leaves the talisman untouched when the confirmation is cancelled', () => {
    mountPanel();
    (target.querySelector('[data-testid="talisman-remove-item"]') as HTMLButtonElement).click();
    flushSync();

    const cancel = target.querySelector(
      '[data-testid="talisman-remove-confirm-cancel"]',
    ) as HTMLButtonElement;
    expect(cancel).toBeTruthy();
    cancel.click();
    flushSync();

    expect(store.entity.talisman).not.toBeNull();
    expect(target.querySelector('[data-testid="talisman-remove-confirm-prompt"]')).toBeFalsy();
  });

  it('removes the talisman only once the destructive action is confirmed', () => {
    mountPanel();
    (target.querySelector('[data-testid="talisman-remove-item"]') as HTMLButtonElement).click();
    flushSync();

    const confirm = target.querySelector(
      '[data-testid="talisman-remove-confirm-confirm"]',
    ) as HTMLButtonElement;
    expect(confirm).toBeTruthy();
    confirm.click();
    flushSync();

    expect(store.entity.talisman).toBeNull();
    expect(target.querySelector('[data-testid="talisman-remove-confirm-prompt"]')).toBeFalsy();
  });

  it('moves initial focus to the safe Cancel control, never the destructive Remove one', () => {
    mountPanel();
    (target.querySelector('[data-testid="talisman-remove-item"]') as HTMLButtonElement).click();
    flushSync();

    const cancel = target.querySelector('[data-testid="talisman-remove-confirm-cancel"]');
    const confirm = target.querySelector('[data-testid="talisman-remove-confirm-confirm"]');
    expect(document.activeElement).toBe(cancel);
    expect(document.activeElement).not.toBe(confirm);
  });

  it('resolves Escape to cancel, never to remove', () => {
    mountPanel();
    (target.querySelector('[data-testid="talisman-remove-item"]') as HTMLButtonElement).click();
    flushSync();

    const dialog = target.querySelector('[role="alertdialog"]') as HTMLElement;
    expect(dialog).toBeTruthy();
    dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    flushSync();

    expect(store.entity.talisman).not.toBeNull();
    expect(target.querySelector('[data-testid="talisman-remove-confirm-prompt"]')).toBeFalsy();
  });
});
