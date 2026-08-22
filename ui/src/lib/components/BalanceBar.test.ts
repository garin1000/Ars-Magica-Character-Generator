import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, LocalizedRuleset } from '../types';

// BalanceBar reads only the shared store singleton's `effective` slice; no
// ruleset/entity setup is needed the way other panels require. Mock the IPC
// bridge anyway, mirroring every other component test, so nothing reaches a
// backend if some other suite left a stray subscription active.
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

import { store } from '../state.svelte';
import BalanceBar from './BalanceBar.svelte';

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

function installEffective(overrides: Partial<EffectiveScores> = {}): void {
  store.effective = {
    virtue_points: 0,
    flaw_points: 0,
    virtue_budget: 10,
    flaw_budget: 10,
    ...overrides,
  } as unknown as EffectiveScores;
}

function html(): string {
  return render(BalanceBar, { props: {} }).body;
}

beforeEach(() => {
  store.lang = 'en';
  store.ruleset = null;
  store.effective = null;
});

// Round 4, G1: the bar used to re-derive its "spent" half in TypeScript
// (`balance()`) while its "budget" half already read `EffectiveScores`, so the
// two halves could disagree with nothing to catch it. Both halves now read
// `EffectiveScores` alone.
describe('BalanceBar (round 4, G1: engine-authoritative spend)', () => {
  it('shows nothing before the ruleset loads', () => {
    const body = html();
    expect(body).not.toContain('data-testid="balance"');
  });

  it('mounts as soon as the ruleset loads, showing 0/0 before the engine totals arrive', () => {
    // The bar mounts on the ruleset alone (matching the wizard's Virtues & Flaws
    // phase, which renders it immediately) rather than waiting on the first
    // `effective_scores` round trip, so it shows the honest "nothing spent yet"
    // 0/0 instead of staying hidden.
    installRuleset();
    const body = html();
    expect(body).toContain('data-testid="balance"');
    expect(body).toContain('data-testid="balance-virtues"');
    expect(body).toContain('data-testid="balance-flaws"');
  });

  it('reads the spent points from EffectiveScores rather than recomputing them', () => {
    installRuleset();
    installEffective({ virtue_points: 4, flaw_points: 2, virtue_budget: 10, flaw_budget: 10 });
    const body = html();
    expect(body).toContain('data-testid="balance-virtues"');
    expect(body).toContain('data-testid="balance-flaws"');
  });

  it('marks the virtue side over when the engine-surfaced spend exceeds the engine budget', () => {
    installRuleset();
    installEffective({ virtue_points: 12, flaw_points: 2, virtue_budget: 10, flaw_budget: 10 });
    const body = html();
    const match = /<span[^>]*data-testid="balance-virtues"[^>]*>/i.exec(body);
    if (!match) throw new Error('no element with data-testid="balance-virtues"');
    expect(match[0]).toContain('over');
  });

  it('does not mark either side over when within the engine budget', () => {
    installRuleset();
    installEffective({ virtue_points: 4, flaw_points: 2, virtue_budget: 10, flaw_budget: 10 });
    const body = html();
    const virtues = /<span[^>]*data-testid="balance-virtues"[^>]*>/i.exec(body)![0];
    const flaws = /<span[^>]*data-testid="balance-flaws"[^>]*>/i.exec(body)![0];
    expect(virtues).not.toContain('over');
    expect(flaws).not.toContain('over');
  });
});
