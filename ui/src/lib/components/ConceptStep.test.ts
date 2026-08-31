import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// ConceptStep is a pure composition (IdentityFields + AgeFields + SagaYearField,
// each already covered by its own test file); mock the IPC bridge the same way
// its children's own tests do, so mounting all three here reaches no backend.
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
import ConceptStep from './ConceptStep.svelte';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
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
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
  store.effective = null;
}

function html(): string {
  return render(ConceptStep, { props: {} }).body;
}

beforeEach(() => {
  store.lang = 'en';
  store.sagaYear = 1220;
  installRuleset();
  resetEntity();
});

// G12 (full-audit test-adequacy): ConceptStep had no test file. It is a 3-line
// composition with no logic of its own (IdentityFields, AgeFields and
// SagaYearField each carry their own real coverage), so a smoke test proving it
// actually mounts all three, in order, inside the wizard's step panel is
// proportionate — a regression that dropped one of the three at composition time
// (e.g. an accidental removal in a merge) would otherwise pass every other test
// in the suite, since each child is also tested standalone.
describe('ConceptStep composes the concept phase surfaces', () => {
  it('mounts the step panel with Identity, Age and Saga Year fields, in order', () => {
    const body = html();
    expect(body).toContain('data-testid="concept-step"');
    const identity = body.indexOf('data-testid="identity-concept"');
    const age = body.indexOf('data-testid="age-input"');
    const sagaYear = body.indexOf('data-testid="saga-year-input"');
    expect(identity).toBeGreaterThanOrEqual(0);
    expect(age).toBeGreaterThan(identity);
    expect(sagaYear).toBeGreaterThan(age);
  });
});
