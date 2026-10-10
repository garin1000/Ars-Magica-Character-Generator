import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from './lib/types';

// `ssr`: where the update check's two surfaces sit in the app (M6.6). The prompt
// is a modal OUTSIDE the shell, which goes inert behind it; the notice is INSIDE
// the shell, on every screen, so that same `inert` gates it.
vi.mock('./lib/ipc', () => ({
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

import { SCHEMA_VERSION, store } from './lib/state.svelte';
import App from './App.svelte';

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
  store.effective = null;
  store.derived = null;
}

function html(): string {
  return render(App).body;
}

function openTag(body: string, testid: string): string | null {
  return new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body)?.[0] ?? null;
}

/** The `.app-shell` element's whole markup, found by balancing its `<div>`s. */
function shellMarkup(body: string): string {
  const start = body.indexOf(openTag(body, 'app-shell')!);
  const tags = /<div\b|<\/div>/g;
  tags.lastIndex = start;
  let depth = 0;
  for (let match = tags.exec(body); match; match = tags.exec(body)) {
    depth += match[0] === '</div>' ? -1 : 1;
    if (depth === 0) return body.slice(start, tags.lastIndex);
  }
  throw new Error('the app shell never closes');
}

beforeEach(() => {
  store.lang = 'en';
  store.error = null;
  store.view = 'start';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  store.updateCheck.promptOpen = false;
  store.updateCheck.available = null;
  store.updateCheck.dismissed = false;
});

describe('the update prompt in the app', () => {
  it('renders outside the shell and turns the shell inert', () => {
    store.updateCheck.promptOpen = true;
    const body = html();

    expect(openTag(body, 'update-prompt')).not.toBeNull();
    expect(shellMarkup(body)).not.toContain('data-testid="update-prompt"');
    expect(openTag(body, 'app-shell')).toMatch(/\binert\b/);
  });

  it('leaves the shell live while closed', () => {
    expect(openTag(html(), 'app-shell')).not.toMatch(/\binert\b/);
  });
});

describe('the update notice in the app', () => {
  it('shows inside the shell on the startup screen', () => {
    store.updateCheck.available = '99.0.0';

    expect(shellMarkup(html())).toContain('data-testid="update-notice"');
  });

  it('shows inside the shell in the editor too', () => {
    store.view = 'editor';
    store.updateCheck.available = '99.0.0';

    expect(shellMarkup(html())).toContain('data-testid="update-notice"');
  });
});
