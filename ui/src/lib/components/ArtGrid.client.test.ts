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
  confirmDiscard: vi.fn().mockResolvedValue(true),
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
      [CREO]: { name: 'Creo', abbreviation: 'Cr', description: 'I create.' },
      [IGNEM]: { name: 'Ignem', abbreviation: 'Ig', description: 'Fire and heat.' },
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
    saga_year: 1220,
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

// Sabine 3 (full-audit round 1): `use:tooltip` opens on `focusin` as well as
// `mouseenter`, so it is keyboard-ready BY CONSTRUCTION — but only if the element it
// is attached to can take focus, and this one was a bare `<span>`. For an Art that
// tooltip is the ONLY place the app renders Creo's or Ignem's rules text, so a
// keyboard or screen-reader user had no route to it at all. The asymmetry is the
// tell: the *available* side of every picker is built from buttons and is reachable;
// the *chosen* side is spans and was not.
//
// A `client` test, and it has to be: actions do not run under the SSR renderer, so
// only a mounted instance can show that focus actually opens the popup. Asserting
// the `tabindex` attribute alone would pin the mechanism without proving the
// outcome.
describe('ArtGrid rules text is reachable by keyboard (Sabine 3)', () => {
  /** The chosen Art's name cell — the element the tooltip hangs on. */
  function nameCell(): HTMLElement {
    const cell = target.querySelector<HTMLElement>('.art-list .item-name');
    expect(cell).toBeTruthy();
    return cell!;
  }

  afterEach(() => {
    document.querySelectorAll('.tooltip-pop').forEach((pop) => pop.remove());
  });

  it('puts the Art name in the page tab order', () => {
    // `tabIndex` is the DOM's OWN answer to "can Tab land here", which is what this
    // finding is about — and it is the assertion to make rather than one about
    // `document.activeElement`, because happy-dom lets `.focus()` succeed on an
    // element no real browser would ever move focus to. A bare `<span>` reports -1.
    expect(nameCell().tabIndex).toBeGreaterThanOrEqual(0);
  });

  it('opens the Art rules text once the keyboard reaches the name', () => {
    expect(nameCell().tabIndex).toBeGreaterThanOrEqual(0);

    nameCell().focus();
    // The tooltip opens only after its 500 ms rest delay (try-out finding 3,
    // `actions.ts::tooltip`); this file already runs on fake timers.
    vi.advanceTimersByTime(500);
    flushSync();

    const describedBy = nameCell().getAttribute('aria-describedby');
    expect(describedBy).toBeTruthy();
    expect(document.getElementById(describedBy!)?.textContent).toContain('I create.');
  });
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

// I3 (try-out finding 15): the effective badge says WHY it differs from the bought
// score, in the Characteristics badge's own shape — "Bought X, effective Y." and
// one line per source Virtue, by its localized name, with a signed amount — from
// the engine's per-source breakdown (`art_bonus_sources`). A `client` test: the
// tooltip is an action, which the SSR renderer never runs, and the badge must be
// reachable by keyboard (the `tooltip-host-parity.test.ts` rule), which only a
// mounted node can show.
describe('ArtGrid effective badge explains its sources (I3)', () => {
  afterEach(() => {
    document.querySelectorAll('.tooltip-pop').forEach((pop) => pop.remove());
  });

  /** Settle a pass in which unbought Ignem carries Puissant Art +3 and Elemental
   *  Magic +3 (the engine's summed bonus is 6). */
  async function settleIgnemBreakdown(): Promise<void> {
    store.ruleset!.i18n = {
      ...store.ruleset!.i18n,
      'virtue.puissant_art': { name: 'Puissant {art}', name_unfilled: 'Puissant Art' },
      'virtue.elemental_magic': { name: 'Elemental Magic' },
    } as LocalizedRuleset['i18n'];
    const pass = store.revalidate();
    inFlight[1].resolve({
      art_bonuses: [{ art: IGNEM, bonus: 6 }],
      art_bonus_sources: [
        {
          art: IGNEM,
          sources: [
            { source: 'virtue.puissant_art', amount: 3 },
            { source: 'virtue.elemental_magic', amount: 3 },
          ],
        },
      ],
    } as unknown as EffectiveScores);
    await vi.advanceTimersByTimeAsync(0);
    await pass;
    flushSync();
  }

  function ignemBadge(): HTMLElement {
    const el = target.querySelector<HTMLElement>(`[data-testid="art-eff-${IGNEM}"]`);
    expect(el).toBeTruthy();
    return el!;
  }

  /** The text of the tooltip the badge opens on keyboard focus, isolates stripped. */
  function tooltipOnFocus(el: HTMLElement): string {
    el.focus();
    vi.advanceTimersByTime(500);
    flushSync();
    const describedBy = el.getAttribute('aria-describedby');
    expect(describedBy, 'the badge opened no tooltip').toBeTruthy();
    return (document.getElementById(describedBy!)?.textContent ?? '').replace(/[⁦-⁩]/g, '');
  }

  it('puts the effective badge in the tab order', async () => {
    await settleIgnemBreakdown();
    expect(ignemBadge().tabIndex).toBeGreaterThanOrEqual(0);
  });

  it('lists bought, each source Virtue with its signed amount, and effective', async () => {
    await settleIgnemBreakdown();
    const text = tooltipOnFocus(ignemBadge());
    expect(text).toContain('Bought 0, effective 6.');
    expect(text).toContain('Puissant Art +3');
    expect(text).toContain('Elemental Magic +3');
    expect(text).not.toContain('virtue.');
  });
});

// X10b: the banked-XP input writes through the store like every other picker
// edit. A `client` test because it exercises the real `oninput` wiring, not
// just the rendered markup. Red-checkpoint protocol, phase 1: `ArtGrid.svelte`
// carries no such input yet, so `bankedXpInput` throws looking for an element
// that does not exist.
describe('ArtGrid banked XP input writes through the store (X10b)', () => {
  function bankedXpInput(art: string): HTMLInputElement {
    const testid = `art-banked-xp-${art}`;
    const el = target.querySelector<HTMLInputElement>(`[data-testid="${testid}"]`);
    if (!el) throw new Error(`banked XP input not rendered: no [data-testid="${testid}"]`);
    return el;
  }

  it("writes a typed value onto the Art's banked_xp", () => {
    const input = bankedXpInput(CREO);
    input.value = '5';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(store.entity.art_scores?.find((a) => a.art === CREO)?.banked_xp).toBe(5);
  });

  it('typing 0 clears/omits the banked XP, leaving the bought score alone', () => {
    store.entity.art_scores = [{ art: CREO, score: 3, banked_xp: 5 }];
    flushSync();
    const input = bankedXpInput(CREO);
    input.value = '0';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    const entry = store.entity.art_scores?.find((a) => a.art === CREO);
    expect(entry?.score).toBe(3);
    expect(entry?.banked_xp ?? 0).toBe(0);
  });
});

// X10b, coordinator ruling (phase 2): `adjust()` used to prune the art_scores
// row the moment score hit 0, which is correct when there is nothing else to
// remember but DATA LOSS once the row can also carry banked_xp — lowering the
// score to 0 would silently discard recorded XP. A row is pruned only once
// BOTH fields are back to their defaults.
describe('ArtGrid banked XP is never pruned while it is still positive (X10b)', () => {
  it('creates a bare row (score 0) to hold banked XP alone', () => {
    // IGNEM (not CREO): the shared `resetEntity()` already seeds a score-3 row
    // for CREO, which is not what this test is about.
    store.setArtBankedXp(IGNEM, 5);
    flushSync();
    expect(store.entity.art_scores?.find((a) => a.art === IGNEM)).toEqual({
      art: IGNEM,
      score: 0,
      banked_xp: 5,
    });
  });

  it('does not prune the row when the spinner lowers the score to 0 while banked XP remains', () => {
    store.entity.art_scores = [{ art: CREO, score: 1, banked_xp: 5 }];
    flushSync();
    store.adjustArt(CREO, -1, MAX);
    flushSync();
    expect(store.entity.art_scores?.find((a) => a.art === CREO)).toEqual({
      art: CREO,
      score: 0,
      banked_xp: 5,
    });
  });

  it('prunes the row once BOTH score and banked XP return to 0', () => {
    store.entity.art_scores = [{ art: CREO, score: 0, banked_xp: 5 }];
    flushSync();
    store.setArtBankedXp(CREO, 0);
    flushSync();
    expect(store.entity.art_scores?.find((a) => a.art === CREO)).toBeUndefined();
  });
});
