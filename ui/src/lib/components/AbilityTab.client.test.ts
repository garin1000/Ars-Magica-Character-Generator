import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

// Manual-testing finding #16 (2026-09-03), Ability surface. The bought score is mutated
// synchronously by `adjustAbilityAt`, while the bonus/floor behind the badge is a
// debounce plus an IPC round trip old, so the badge rendered `newScore + oldBonus`
// in between. A `client` test because the defect is a mid-flight frame: the SSR
// renderer only ever sees settled state, so an SSR assertion reports green
// whatever the badge does while a recompute is open.
vi.mock('../ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn(),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
  applyChildhoodPackage: vi.fn(),
}));

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import AbilityTab from './AbilityTab.svelte';

const ATHLETICS = 'ability.athletics';
/** Mirrors `VALIDATE_DEBOUNCE_MS` in `state.svelte.ts`. */
const DEBOUNCE_MS = 150;
const MAX = 5;

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: { [ATHLETICS]: { id: ATHLETICS, category: 'general' } },
      advancement: [
        { score: 1, total_xp: 5 },
        { score: MAX, total_xp: 75 },
      ],
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: { [ATHLETICS]: { name: 'Athletics' } },
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
    ability_scores: [{ ability: ATHLETICS, score: 3 }],
    xp_pool: 0,
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

interface Deferred {
  promise: Promise<EffectiveScores>;
  resolve: (value: EffectiveScores) => void;
}

let inFlight: Deferred[] = [];

function newDeferred(): Deferred {
  let resolve!: (value: EffectiveScores) => void;
  const promise = new Promise<EffectiveScores>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

async function answer(index: number, bonus: number): Promise<void> {
  inFlight[index].resolve({
    ability_bonuses: [{ ability: ATHLETICS, bonus }],
  } as unknown as EffectiveScores);
  await vi.advanceTimersByTimeAsync(0);
  flushSync();
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

function badge(): string | null {
  const el = target.querySelector(`[data-testid="ability-eff-${ATHLETICS}-0"]`);
  return el ? (el.textContent ?? '').replace(/[⁦-⁩]/g, '').trim() : null;
}

function score(): string {
  return (
    target.querySelector(`[data-testid="ability-score-${ATHLETICS}-0"]`)?.textContent ?? ''
  ).trim();
}

beforeEach(async () => {
  vi.useFakeTimers();
  inFlight = [];
  vi.mocked(ipc.effectiveScores).mockImplementation(() => {
    const deferred = newDeferred();
    inFlight.push(deferred);
    return deferred.promise;
  });
  store.lang = 'en';
  store.view = 'editor';
  installRuleset();
  resetEntity();
  store.effective = null;

  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(AbilityTab, { target });
  flushSync();

  const first = store.revalidate();
  await answer(0, 2);
  await first;
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  store.effective = null;
  store.view = 'start';
  vi.useRealTimers();
});

describe('AbilityTab effective-score badge staleness (#16)', () => {
  it('holds the settled badge value while the recompute is in flight', async () => {
    expect(score()).toBe('3');
    expect(badge()).toBe('5');

    store.adjustAbilityAt(0, 1, MAX);
    flushSync();
    expect(score()).toBe('4');
    // Before the fix this read '6' (4 + the bonus computed for 3).
    expect(badge()).toBe('5');

    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS);
    flushSync();
    expect(badge()).toBe('5');

    await answer(1, 3);
    expect(badge()).toBe('7');
  });
});
