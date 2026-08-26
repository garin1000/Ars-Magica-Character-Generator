import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LifeStageRules, LocalizedRuleset } from '../types';

// The step reads the shared store singleton (life-stage rules, the stored plan) and
// the Fluent bundle. The store schedules a debounced revalidate over the Tauri IPC
// bridge; mock the bridge so nothing reaches a backend.
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
import ExperienceStep from './ExperienceStep.svelte';

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
      },
      childhoods: {
        'childhood.peasant': {
          id: 'childhood.peasant',
          entries: [],
        },
      },
      advancement: [],
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
      ...(rules ? { life_stages: rules } : {}),
    },
    i18n: { 'childhood.peasant': { name: 'Peasant' } },
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

/** Render the step to an HTML string (node env, no DOM). */
function html(): string {
  return render(ExperienceStep, { props: {} }).body;
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('ExperienceStep (Slice 2, #11)', () => {
  it('renders the life-stage funding panel', () => {
    const body = html();
    expect(body).toContain('data-testid="life-stage-panel"');
    // The funding chooser is the one control every character type needs here, so it
    // must be present without a life-stage plan already stored.
    expect(body).toContain('data-testid="ability-funding-pool"');
    expect(body).toContain('data-testid="ability-funding-life_stages"');
  });

  it('renders the childhood picker once the character is funded by life stages', () => {
    // `abilityFunding` is derived from plan presence, so a stored plan is what
    // selects the guided mode the detailed fields sit behind.
    store.entity.life_stages = {};
    const body = html();
    expect(body).toContain('data-testid="childhood-package-select"');
    expect(body).toContain('data-testid="native-language-input"');
    expect(body).toContain('data-testid="life-stage-age-input"');
  });

  it('renders nothing for a ruleset shipping no life-stage rules', () => {
    installRuleset(null);
    expect(html()).not.toContain('data-testid="life-stage-panel"');
  });

  // The step owns the funding preamble and nothing else: the Available/Selected
  // lists stayed on the `abilities` step, which is the whole point of the split.
  it('does not carry the ability lists', () => {
    expect(html()).not.toContain('class="region-row"');
  });
});
