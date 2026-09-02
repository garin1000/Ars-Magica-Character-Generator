import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

// E2 (full-audit backlog, Tier 3-4): Reputations.svelte had no test file at all.
// Mirrors PersonalityTraits.test.ts's harness — the sibling component it was
// extracted from CharacterDetails alongside — since both are plain store-bound
// lists with no SourcePicker involved, so SSR + direct store-mutator calls covers
// the real behavior without needing a mounted client test.
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
import Reputations from './Reputations.svelte';

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
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
}

function html(): string {
  return render(Reputations, { props: {} }).body;
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

describe('Reputations grant gating', () => {
  it('shows the empty message and no add button when nothing grants a Reputation', () => {
    const body = html();
    expect(body).toContain('data-testid="reputation-empty"');
    expect(body).not.toMatch(/data-testid="reputation-add-/);
  });

  it('offers an add button per granting kind, naming the kind and level', () => {
    store.effective = {
      reputation_grants: [{ kind: 'hermetic', score: 2 }],
    } as unknown as EffectiveScores;
    const body = html();
    expect(body).not.toContain('data-testid="reputation-empty"');
    const button =
      /<button[^>]*data-testid="reputation-add-hermetic"[^>]*>([\s\S]*?)<\/button>/.exec(body);
    expect(button).not.toBeNull();
    expect(button![1]).toContain('Hermetic');
    expect(button![1]).toContain('2');
  });

  it('offers one add button per distinct granting kind, in grant order', () => {
    store.effective = {
      reputation_grants: [
        { kind: 'hermetic', score: 2 },
        { kind: 'local', score: 1 },
      ],
    } as unknown as EffectiveScores;
    const body = html();
    const hermeticIndex = body.indexOf('data-testid="reputation-add-hermetic"');
    const localIndex = body.indexOf('data-testid="reputation-add-local"');
    expect(hermeticIndex).toBeGreaterThan(-1);
    expect(localIndex).toBeGreaterThan(hermeticIndex);
  });
});

describe('Reputations already-taken list', () => {
  it('lists a Reputation added via store.addReputation, with its type and score', () => {
    store.addReputation('academic', 3);
    const body = html();
    expect(body).toContain('data-testid="reputation-list"');
    expect(body).toMatch(/<li>[\s\S]*Academic[\s\S]*3[\s\S]*<\/li>/);
  });

  it('reads the stored content back into the content input', () => {
    store.addReputation('local', 1);
    store.setReputationContent(0, 'known among the fishermen');
    const input = /<input[^>]*data-testid="reputation-content-0"[^>]*>/.exec(html());
    expect(input).not.toBeNull();
    expect(input![0]).toContain('value="known among the fishermen"');
  });

  it('gives the content input an accessible name', () => {
    store.addReputation('local', 1);
    const input = /<input[^>]*data-testid="reputation-content-0"[^>]*>/.exec(html());
    expect(input![0]).toContain('aria-label="What it is for"');
  });

  it('renders a remove button for each Reputation, one per row', () => {
    store.addReputation('local', 1);
    store.addReputation('hermetic', 2);
    const body = html();
    expect(body).toContain('data-testid="reputation-remove-0"');
    expect(body).toContain('data-testid="reputation-remove-1"');
  });

  it('renders both rows independently when two Reputations of the same kind are taken', () => {
    store.addReputation('local', 1);
    store.addReputation('local', 1);
    const body = html();
    expect(body).toContain('data-testid="reputation-content-0"');
    expect(body).toContain('data-testid="reputation-content-1"');
  });
});
