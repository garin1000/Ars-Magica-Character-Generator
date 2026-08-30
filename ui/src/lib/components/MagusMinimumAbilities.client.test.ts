import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type {
  Ability,
  EffectiveScores,
  Entity,
  LocalizedRuleset,
  MagusMinimumAbility,
} from '../types';

// guided-creation-review-2026-08 #12: the checklist collapses to a summary line that
// expands on demand.
//
// This must be a `client` test. Opening a disclosure is a change to LIVE DOM state:
// the SSR render can only show that the `<details>` carries no `open` attribute and
// that the rows sit inside it, which is markup, not behaviour. What it cannot show is
// that the summary is actually wired as the disclosure's control — a `<summary>`
// nested one element too deep still renders identically and still contains all the
// rows, but no longer toggles anything. That failure is invisible until something
// clicks it, so something here does.
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
import MagusMinimumAbilities from './MagusMinimumAbilities.svelte';

function installRuleset(): void {
  const abilities: Record<string, Ability> = {
    'ability.magic_theory': { id: 'ability.magic_theory', category: 'arcane' } as Ability,
    'ability.parma_magica': { id: 'ability.parma_magica', category: 'arcane' } as Ability,
  };
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities,
      life_stages: {
        apprenticeship: {
          minimum_abilities: [],
          recommended_abilities: [],
          recommended_xp: 90,
          xp: 240,
          years: 15,
        },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general', 'arcane'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      'ability.magic_theory': { name: 'Magic Theory' },
      'ability.parma_magica': { name: 'Parma Magica' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
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
  };
}

function row(
  ability: string,
  requirement: MagusMinimumAbility['requirement'],
): MagusMinimumAbility {
  return { ability, min_score: 1, score: 0, met: false, requirement };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
  store.effective = {
    magus_minimum_abilities: [
      row('ability.magic_theory', 'required'),
      row('ability.parma_magica', 'recommended'),
    ],
  } as unknown as EffectiveScores;
  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(MagusMinimumAbilities, { target });
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  store.effective = null;
});

describe('MagusMinimumAbilities disclosure (slice 11, #12)', () => {
  it('expands to the full checklist on demand', () => {
    const details = target.querySelector<HTMLDetailsElement>(
      'details[data-testid="magus-minimums"]',
    );
    expect(details).not.toBeNull();
    expect(details!.open).toBe(false);

    // The summary must be the disclosure's OWN control: a direct `<summary>` child.
    // This is the wiring the SSR test cannot see.
    const summary = details!.querySelector('summary');
    expect(summary).not.toBeNull();
    expect(summary!.parentElement).toBe(details);

    summary!.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    flushSync();
    expect(details!.open).toBe(true);

    // Both groups are revealed, not just the demanded one.
    expect(details!.querySelectorAll('[data-testid^="magus-minimum-ability."]')).toHaveLength(1);
    expect(details!.querySelectorAll('[data-testid^="magus-recommended-ability."]')).toHaveLength(
      1,
    );

    // And it closes again, so the collapse is a toggle rather than a one-way reveal.
    summary!.dispatchEvent(new MouseEvent('click', { bubbles: true }));
    flushSync();
    expect(details!.open).toBe(false);
  });
});
