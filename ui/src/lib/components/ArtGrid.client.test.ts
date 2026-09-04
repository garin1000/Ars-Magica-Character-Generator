import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EffectiveScores, Entity, LocalizedRuleset } from '../types';

// Manual-testing finding #16 (2026-09-03): editing one Art made the effective-score badges
// cycle through values that were never true of any character — the bought score
// is mutated synchronously by `adjustArt`, while the bonus behind it is up to one
// debounce plus one IPC round trip old, so the badge rendered `newScore +
// oldBonus` in between. Reported against Elemental Magic (the one nonlinear
// cross-Art recompute), but the mechanism is general.
//
// This MUST be a `client` test. The whole defect lives in *when* a value reaches
// the DOM relative to an async round trip; the SSR renderer takes one snapshot of
// already-settled state and can never show an intermediate frame. An assertion
// written against `render()` here would report green no matter what the badge did
// mid-flight.
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
import ArtGrid from './ArtGrid.svelte';

const CREO = 'art.creo';
const IGNEM = 'art.ignem';
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
      abilities: {},
      art_advancement: [
        { score: 1, total_xp: 5 },
        { score: MAX, total_xp: 75 },
      ],
      arts: {
        [CREO]: { id: CREO, art_type: 'technique' },
        [IGNEM]: { id: IGNEM, art_type: 'form' },
      },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      [CREO]: { name: 'Creo', abbreviation: 'Cr' },
      [IGNEM]: { name: 'Ignem', abbreviation: 'Ig' },
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
    art_scores: [{ art: CREO, score: 3 }],
    personality_traits: [],
    reputations: [],
  };
}

interface Deferred {
  promise: Promise<EffectiveScores>;
  resolve: (value: EffectiveScores) => void;
}

/** Every `effectiveScores` call parks here until the test hands it a payload,
 *  so a pass can be left in flight or resolved out of order on purpose. */
let inFlight: Deferred[] = [];

function newDeferred(): Deferred {
  let resolve!: (value: EffectiveScores) => void;
  const promise = new Promise<EffectiveScores>((r) => {
    resolve = r;
  });
  return { promise, resolve };
}

/** Answer the `index`-th still-parked call with `bonus` on Creo. */
async function answer(index: number, bonus: number): Promise<void> {
  inFlight[index].resolve({ art_bonuses: [{ art: CREO, bonus }] } as unknown as EffectiveScores);
  await vi.advanceTimersByTimeAsync(0);
  flushSync();
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

/** The badge's text, bidi isolates stripped, or null when no badge is rendered. */
function badge(): string | null {
  const el = target.querySelector(`[data-testid="art-eff-${CREO}"]`);
  return el ? (el.textContent ?? '').replace(/[⁦-⁩]/g, '').trim() : null;
}

function score(): string {
  return (target.querySelector(`[data-testid="art-score-${CREO}"]`)?.textContent ?? '').trim();
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
  app = mount(ArtGrid, { target });
  flushSync();

  // Settle a first pass so the badge has a value to hold: Creo 3 with a +2 bonus.
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

describe('ArtGrid effective-score badge staleness (#16)', () => {
  it('holds the settled badge value while the recompute is in flight', async () => {
    expect(score()).toBe('3');
    expect(badge()).toBe('5');

    // The player raises Creo. The spinner is direct feedback and updates at once…
    store.adjustArt(CREO, 1, MAX);
    flushSync();
    expect(score()).toBe('4');
    // …but the badge must NOT mix the fresh score with the bonus that was computed
    // for the old one. Before the fix this read '6' (4 + stale 2).
    expect(badge()).toBe('5');

    // Still held once the debounce fires and the round trip is actually open.
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS);
    flushSync();
    expect(badge()).toBe('5');

    // One single transition, straight to the value that is true of the character.
    await answer(1, 3);
    expect(badge()).toBe('7');
  });

  it('never adopts a response that arrives out of order', async () => {
    store.adjustArt(CREO, 1, MAX);
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS); // pass A opens over Creo 4
    store.adjustArt(CREO, 1, MAX);
    await vi.advanceTimersByTimeAsync(DEBOUNCE_MS); // pass B opens over Creo 5
    flushSync();
    expect(score()).toBe('5');
    expect(badge()).toBe('5');

    // A (superseded) answers first, with a bonus that belongs to Creo 4.
    await answer(1, 9);
    expect(badge()).toBe('5');

    // Only B may move the badge, and it moves it exactly once.
    await answer(2, 3);
    expect(badge()).toBe('8');
  });

  it('shows no badge until the very first pass has settled', async () => {
    // A character with nothing settled behind it: no badge is better than a badge
    // built from a bonus that has not been computed yet.
    store.effective = null;
    resetEntity();
    flushSync();
    expect(badge()).toBeNull();

    const first = store.revalidate();
    await answer(1, 2);
    await first;
    flushSync();
    expect(badge()).toBe('5');
  });
});
