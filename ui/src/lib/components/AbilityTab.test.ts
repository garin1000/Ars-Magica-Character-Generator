import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LifeStageRules, LocalizedRuleset } from '../types';

// The tab reads the shared store singleton (ruleset catalogue, entity rows, filter
// state) and the Fluent bundle. The store schedules a debounced revalidate over the
// Tauri IPC bridge; mock the bridge so nothing reaches a backend.
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
import AbilityTab from './AbilityTab.svelte';

/** The life-stage rules as `rules/core/life_stages.json` ships them. */
function lifeStageRules(): LifeStageRules {
  return {
    childhood: {
      years: 5,
      native_language_ability: 'ability.living_language',
      native_language_xp: 75,
      spread_xp: 45,
      spread_abilities: ['ability.athletics'],
    },
    later_life: { xp_per_year: 15 },
  };
}

/** Install a minimal localized ruleset; `rules` is null for one shipping no plan. */
function installRuleset(rules: LifeStageRules | null = lifeStageRules()): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {
        companion: {
          id: 'companion',
          budget: { virtue_points: 10, flaw_points: 10 },
          is_magus: false,
          gift_policy: 'forbidden',
          creation_phases: [],
        },
      },
      abilities: { 'ability.athletics': { id: 'ability.athletics', category: 'general' } },
      advancement: [],
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
      ...(rules ? { life_stages: rules } : {}),
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
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
  store.result = { issues: [] };
}

/** Render the tab to an HTML string (node env, no DOM). */
function html(): string {
  return render(AbilityTab, { props: {} }).body;
}

const VOID_TAGS = new Set([
  'area',
  'base',
  'br',
  'col',
  'embed',
  'hr',
  'img',
  'input',
  'link',
  'meta',
  'source',
  'track',
  'wbr',
]);

/**
 * Nesting depth of the first element whose start tag contains `marker` — 0 for a
 * root-level node. Walks the start/end tags, so it can tell a sibling from a child
 * without a DOM (these component tests render to a string).
 */
function depthOf(body: string, marker: string): number {
  const tags = body.matchAll(/<(\/?)([a-zA-Z][a-zA-Z0-9-]*)([^>]*)>/g);
  let depth = 0;
  for (const tag of tags) {
    const [, closing, name, attrs] = tag;
    if (closing) {
      depth -= 1;
      continue;
    }
    if (attrs.includes(marker)) return depth;
    if (!VOID_TAGS.has(name.toLowerCase()) && !attrs.trimEnd().endsWith('/')) depth += 1;
  }
  throw new Error(`no element whose start tag contains ${marker}`);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('AbilityTab mounts the life-stage panel (slice 6b3b)', () => {
  it('renders the panel above the Available/Selected row', () => {
    const body = html();
    const panel = body.indexOf('data-testid="life-stage-panel"');
    const row = body.indexOf('class="region-row"');
    expect(panel).toBeGreaterThanOrEqual(0);
    expect(row).toBeGreaterThanOrEqual(0);
    // The funding choice is read before the lists it funds, in DOM order — which is
    // also the reading order for a screen reader and the keyboard tab order.
    expect(panel).toBeLessThan(row);
  });

  it('keeps the region row a root-level sibling of the panel', () => {
    const body = html();
    // `.region-row` must stay the only `flex: 1` child of `.vf-tab`: nesting it
    // inside the panel would collapse the Available and Selected lists.
    expect(depthOf(body, 'class="region-row"')).toBe(0);
    expect(depthOf(body, 'data-testid="life-stage-panel"')).toBe(0);
  });

  it('still renders the region row for a ruleset with no life-stage rules', () => {
    installRuleset(null);
    const body = html();
    expect(body).toContain('class="region-row"');
    expect(body).not.toContain('data-testid="life-stage-panel"');
    expect(depthOf(body, 'class="region-row"')).toBe(0);
  });
});
