import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// E1 (tmp/review/review-round-2-erika.md): StartScreen's initial-focus $effect
// (moves focus to the Open button so keyboard/screen-reader users land inside
// the screen instead of on <body>) shipped in round 1 with zero test coverage
// at any layer — SSR-only StartScreen.test.ts cannot see it, since SSR never
// executes an `$effect`. Mirrors DiscardPrompt.client.test.ts and the pattern
// documented in CLAUDE.md's "Frontend test environments".
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
import StartScreen from './StartScreen.svelte';

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

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: '',
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

beforeEach(() => {
  store.lang = 'en';
  store.error = null;
  store.loading = false;
  store.view = 'start';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

describe('StartScreen focus management (E1)', () => {
  it('moves initial focus to the Open button on mount', () => {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(StartScreen, { target });
    flushSync();

    const openButton = target.querySelector('[data-testid="start-open"]');
    expect(openButton).toBeTruthy();
    expect(document.activeElement).toBe(openButton);
  });
});
