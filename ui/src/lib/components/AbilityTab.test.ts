import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type {
  EffectiveScores,
  Entity,
  LifeStageRules,
  LocalizedRuleset,
  MagusMinimumAbility,
} from '../types';

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
      abilities: {
        'ability.athletics': { id: 'ability.athletics', category: 'general' },
        'ability.parma_magica': { id: 'ability.parma_magica', category: 'arcane' },
      },
      advancement: [],
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'academic', 'arcane', 'martial', 'supernatural'],
      art_type_order: ['technique', 'form'],
      ...(rules ? { life_stages: rules } : {}),
    },
    i18n: { 'ability.parma_magica': { name: 'Parma Magica' } },
  } as unknown as LocalizedRuleset;
}

/** One unmet Hermetic minimum, as the engine sends it for a magus. */
function setChecklist(): void {
  const rows: MagusMinimumAbility[] = [
    {
      ability: 'ability.parma_magica',
      min_score: 1,
      score: 0,
      met: false,
      requirement: 'required',
    },
  ];
  store.effective = { magus_minimum_abilities: rows } as unknown as EffectiveScores;
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

// Slice 2 (#11): the funding model and the whole life-stage plan moved to the new
// `experience` step (`ExperienceStep.svelte`), so this tab is only the lists again.
// The mirror image of `ExperienceStep.test.ts`: what left here has to arrive there.
describe('AbilityTab leaves the funding choice to the experience step (Slice 2)', () => {
  it('no longer renders the life-stage panel', () => {
    expect(html()).not.toContain('data-testid="life-stage-panel"');
  });

  it('still renders the Available and Selected regions, with the row at root level', () => {
    const body = html();
    // `.region-row` must stay the only `flex: 1` child of `.vf-tab`, which is the
    // whole point of the split: with no preamble above it, it now gets the height.
    expect(body).toContain('class="region-row"');
    expect(depthOf(body, 'class="region-row"')).toBe(0);
    expect(body).toContain('data-testid="ability-search"');
    expect(body).toContain('class="region region-source"');
    expect(body).toContain('class="region region-selected"');
  });

  it('still renders the region row for a ruleset with no life-stage rules', () => {
    installRuleset(null);
    const body = html();
    expect(body).toContain('class="region-row"');
    expect(body).not.toContain('data-testid="life-stage-panel"');
    expect(depthOf(body, 'class="region-row"')).toBe(0);
  });
});

describe('AbilityTab mounts the magus minimums checklist (slice 6b4)', () => {
  it('renders it above the Available/Selected row', () => {
    setChecklist();
    const body = html();
    const checklist = body.indexOf('data-testid="magus-minimums"');
    const row = body.indexOf('class="region-row"');
    // What the Order demands of a magus's Abilities, then the lists it is about.
    expect(checklist).toBeGreaterThanOrEqual(0);
    expect(checklist).toBeLessThan(row);
  });

  it('keeps it a root-level sibling, so the region row still owns the height', () => {
    setChecklist();
    const body = html();
    // An auto-height sibling, never a wrapper: `.region-row` must stay the only
    // `flex: 1` child of `.vf-tab` or the Available/Selected lists collapse.
    expect(depthOf(body, 'data-testid="magus-minimums"')).toBe(0);
    expect(depthOf(body, 'class="region-row"')).toBe(0);
  });

  it('renders it in both funding modes', () => {
    setChecklist();
    // Flat pool: `:2437` is unconditional, so a magus owes the minimums either way …
    expect(html()).toContain('data-testid="magus-minimums"');
    // … and under life-stage funding just the same. The mode is stored since schema
    // 16, so it is set alongside the plan.
    store.entity.ability_funding = 'life_stages';
    store.entity.life_stages = {};
    expect(html()).toContain('data-testid="magus-minimums"');
  });

  it('renders it for a ruleset shipping no life-stage rules', () => {
    installRuleset(null);
    setChecklist();
    const body = html();
    // The funding panel is gone with the rules it needs, but a magus still owes the
    // Order its minimums — which is why the checklist is not mounted inside the panel.
    expect(body).not.toContain('data-testid="life-stage-panel"');
    expect(body).toContain('data-testid="magus-minimums"');
    expect(depthOf(body, 'class="region-row"')).toBe(0);
  });

  it('renders nothing of it when the engine sends no rows', () => {
    // Empty for every type but a magus — the tab needs no is_magus test of its own.
    expect(html()).not.toContain('data-testid="magus-minimums"');
  });
});
