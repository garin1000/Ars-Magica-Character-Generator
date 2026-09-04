import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

// Manual-testing finding #16 (2026-09-03), Characteristic surface. Here the stale pair does not
// just show a wrong number, it makes a badge that should not exist at all appear
// and then vanish: the gate is `engineEffective !== boughtScore`, and raising the
// bought score leaves the engine's (stale) effective behind it, so a character with
// no aging drop and no virtue delta flashed a "-> +0" badge for the length of the
// debounce. A `client` test — the SSR renderer never shows a mid-flight frame.
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
import CharacteristicPicker from './CharacteristicPicker.svelte';

/** Mirrors `VALIDATE_DEBOUNCE_MS` in `state.svelte.ts`. */
const DEBOUNCE_MS = 150;

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      characteristic_rules: {
        start_points: 7,
        base_max: 3,
        base_min: -3,
        costs: [-3, -2, -1, 0, 1, 2, 3].map((s) => ({ score: s, cost: Math.abs(s) })),
      },
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

/** Answer the `index`-th parked call with an engine effective Strength. */
async function answer(index: number, effectiveStrength: number): Promise<void> {
  inFlight[index].resolve({
    characteristic_effective: { str: effectiveStrength },
  } as unknown as EffectiveScores);
  await vi.advanceTimersByTimeAsync(0);
  flushSync();
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

function badge(): string | null {
  const el = target.querySelector('[data-testid="char-effective-str"]');
  return el ? (el.textContent ?? '').replace(/[⁦-⁩]/g, '').trim() : null;
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
  app = mount(CharacteristicPicker, { target });
  flushSync();

  // Settle a first pass: Strength bought 0, engine effective 0 — nothing to show.
  const first = store.revalidate();
  await answer(0, 0);
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

describe('CharacteristicPicker effective badge staleness (#16)', () => {
  it('does not flash a phantom badge while the recompute is in flight', async () => {
    expect(badge()).toBeNull();

    store.setCharacteristic('str', 1);
    flushSync();
    // Before the fix a "-> +0" badge appeared here: bought 1 vs. the engine's
    // effective 0, computed for the character as it was one keystroke ago.
    expect(badge()).toBeNull();

    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS);
    flushSync();
    expect(badge()).toBeNull();

    // The engine agrees there is no delta, so the badge never appears at all.
    await answer(1, 1);
    expect(badge()).toBeNull();
  });

  it('still shows the badge once the engine reports a real delta', async () => {
    store.setCharacteristic('str', 2);
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS);
    // An aging drop the engine applies on top of the bought 2.
    await answer(1, 1);
    expect(badge()).toBe('→ +1');
  });
});
