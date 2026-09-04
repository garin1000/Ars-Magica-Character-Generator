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
      reputation_type_order: ['local', 'ecclesiastical', 'hermetic', 'academic'],
    },
    i18n: {
      'flaw.infamous': { name: 'Infamous' },
      'virtue.famous': { name: 'Famous' },
      'flaw.apostate': { name: 'Apostate' },
      'virtue.senior_clergy': { name: 'Senior Clergy' },
    },
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

function grants(...list: { source: string; kind: string | null; score: number }[]): void {
  store.effective = { reputation_grants: list } as unknown as EffectiveScores;
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
  it('shows the empty message and no rows when nothing grants a Reputation', () => {
    const body = html();
    expect(body).toContain('data-testid="reputation-empty"');
    expect(body).not.toContain('data-testid="reputation-content-0"');
  });

  it('never offers an add control — a grant IS the row', () => {
    grants({ source: 'flaw.infamous', kind: 'local', score: 4 });
    const body = html();
    expect(body).not.toMatch(/data-testid="reputation-add/);
    expect(body).toContain('data-testid="reputation-content-0"');
  });

  it('renders a granted slot with its kind, level and the Flaw that granted it', () => {
    grants({ source: 'flaw.infamous', kind: 'local', score: 4 });
    const body = html();
    expect(body).not.toContain('data-testid="reputation-empty"');
    const source = /data-testid="reputation-source-0"[^>]*>([\s\S]*?)<\/span>/.exec(body);
    expect(source).not.toBeNull();
    expect(source![1]).toContain('Infamous');
    expect(source![1]).toContain('4');
    expect(body).toContain('Local');
  });

  it('names the granting item, never its raw id', () => {
    grants({ source: 'flaw.infamous', kind: 'local', score: 4 });
    expect(html()).not.toContain('flaw.infamous');
  });

  it('gives a granted row no remove control — the grant is not the player&#39;s to drop', () => {
    grants({ source: 'flaw.infamous', kind: 'local', score: 4 });
    expect(html()).not.toContain('data-testid="reputation-remove-0"');
  });

  it('offers a type picker on a wildcard grant, from the engine taxonomy', () => {
    grants({ source: 'virtue.famous', kind: null, score: 4 });
    const body = html();
    const select = /<select[^>]*data-testid="reputation-kind-0"[\s\S]*?<\/select>/.exec(body);
    expect(select).not.toBeNull();
    // The empty prompt plus one option per engine-declared Reputation type.
    expect(select![0]).toContain('Choose a type');
    for (const label of ['Local', 'Ecclesiastical', 'Hermetic', 'Academic']) {
      expect(select![0]).toContain(label);
    }
  });

  it('renders one row per grant, in grant order', () => {
    grants(
      { source: 'flaw.apostate', kind: 'ecclesiastical', score: 4 },
      { source: 'virtue.famous', kind: null, score: 4 },
    );
    const body = html();
    expect(body.indexOf('data-testid="reputation-content-0"')).toBeGreaterThan(-1);
    expect(body.indexOf('data-testid="reputation-content-1"')).toBeGreaterThan(
      body.indexOf('data-testid="reputation-content-0"'),
    );
    // The wildcard is the second row, so only IT carries a type picker.
    expect(body).not.toContain('data-testid="reputation-kind-0"');
    expect(body).toContain('data-testid="reputation-kind-1"');
  });
});

describe('Reputations stored rows', () => {
  it('reads a stored description back into its granted slot', () => {
    grants({ source: 'flaw.infamous', kind: 'local', score: 4 });
    store.addReputation('local', 4, 'dragon slayer');
    const input = /<input[^>]*data-testid="reputation-content-0"[^>]*>/.exec(html());
    expect(input).not.toBeNull();
    expect(input![0]).toContain('value="dragon slayer"');
  });

  it('gives the description input an accessible name', () => {
    grants({ source: 'flaw.infamous', kind: 'local', score: 4 });
    const input = /<input[^>]*data-testid="reputation-content-0"[^>]*>/.exec(html());
    expect(input![0]).toContain('aria-label="What it is for"');
  });

  it('fills both rows when two grants share a kind and score', () => {
    // Apostate and Senior Clergy each grant Ecclesiastical 4. Which row is
    // attributed to which Virtue/Flaw is arbitrary; both must be described.
    grants(
      { source: 'flaw.apostate', kind: 'ecclesiastical', score: 4 },
      { source: 'virtue.senior_clergy', kind: 'ecclesiastical', score: 4 },
    );
    store.addReputation('ecclesiastical', 4, 'archdeacon of Reims');
    store.addReputation('ecclesiastical', 4, 'renounced his vows');
    const body = html();
    expect(body).toContain('value="archdeacon of Reims"');
    expect(body).toContain('value="renounced his vows"');
  });

  it('still renders a Reputation no grant covers, with a remove control', () => {
    // A legacy or hand-edited save. The engine keeps reporting
    // `reputation_not_granted` for it; the panel lets the player clear it.
    store.addReputation('academic', 2, 'a legacy row');
    const body = html();
    expect(body).toContain('value="a legacy row"');
    expect(body).toContain('data-testid="reputation-remove-0"');
  });

  it('disables the description of a wildcard slot until a type is chosen', () => {
    grants({ source: 'virtue.famous', kind: null, score: 4 });
    const input = /<input[^>]*data-testid="reputation-content-0"[^>]*>/.exec(html());
    expect(input![0]).toContain('disabled');
  });
});
