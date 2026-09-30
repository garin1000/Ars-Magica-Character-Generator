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
  unlinkAbilityParameters: vi.fn().mockImplementation((entity: Entity) => Promise.resolve(entity)),
  // The native discard-confirmation answer, for the one CV7 test that drives
  // `store.open()` — short-circuits `FileOperations.confirmDiscard` before it
  // ever falls back to the in-app prompt, mirroring `state.svelte.test.ts`'s
  // own default.
  confirmDiscard: vi.fn().mockResolvedValue(true),
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
    saga_year: 1220,
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

// CV7 (design-cv-catalogued-values.md § 6.1/§ 6.3, § 5.5): the parameter
// picker combo box, and the removal flow's live conversion. Red-checkpoint
// protocol, phase 1: `AbilityTab.svelte`'s template is untouched, so every
// assertion below that queries for the combo box's markup fails looking for
// an element that does not exist yet.
describe('AbilityTab parameter picker combo box (CV7)', () => {
  const ORG_LORE = 'ability.organization_lore';
  const CGT = 'virtue.craft_guild_training';

  function installCv7Ruleset(): void {
    store.ruleset!.ruleset.abilities = {
      ...store.ruleset!.ruleset.abilities,
      [ORG_LORE]: { id: ORG_LORE, category: 'academic', parameter: 'organization' },
    };
    // The shared fixture only lists 'general' — 'academic' must join it or
    // `groupAbilitySelectionsByCategory` silently drops this row's whole
    // category group.
    store.ruleset!.ruleset.ability_category_order = [
      ...store.ruleset!.ruleset.ability_category_order,
      'academic',
    ];
    store.ruleset!.i18n = {
      ...store.ruleset!.i18n,
      'language.latin': { name: 'Latin' },
      [CGT]: { name: 'Craft Guild Training' },
    };
  }

  function setOptions(linked: { item: string; param: string; resolved: string | null }[]): void {
    store.effective = {
      ability_bonuses: [],
      ability_parameter_options: [
        {
          ability: ORG_LORE,
          catalogued: ['language.latin'],
          linked,
          hint: false,
        },
      ],
    } as unknown as EffectiveScores;
  }

  const COMBO_TESTID = `ability-param-select-${ORG_LORE}-0`;

  /** Throws a clear, diagnostic message rather than a bare TypeError when the
   *  combo box does not exist yet (red-checkpoint protocol, CV7 phase 1). */
  function comboSelect(): HTMLSelectElement {
    const el = target.querySelector<HTMLSelectElement>(`[data-testid="${COMBO_TESTID}"]`);
    if (!el) throw new Error(`combo box not rendered: no [data-testid="${COMBO_TESTID}"]`);
    return el;
  }

  beforeEach(() => {
    installCv7Ruleset();
    store.entity.ability_scores = [{ ability: ORG_LORE, score: 1 }];
    setOptions([]);
    flushSync();
  });

  it('offers a real, labeled, focusable native <select> — never a raw id in an option label', () => {
    setOptions([{ item: CGT, param: 'guild', resolved: "Smiths' Guild of Verdi" }]);
    flushSync();
    const select = comboSelect();
    expect(select.tagName).toBe('SELECT');
    expect(select.getAttribute('aria-label')).toBeTruthy();
    expect(select.disabled).toBe(false);
    // The raw id legitimately backs the option's OWN `value="cat:…"` wire
    // attribute; `textContent` is the visible label text alone, and that must
    // never show it.
    expect(select.textContent).not.toContain('language.latin');
  });

  it('selecting a catalogued option writes {id: ...}', () => {
    const select = comboSelect()!;
    select.value = 'cat:language.latin';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.ability_scores![0].parameter).toEqual({ id: 'language.latin' });
  });

  it('selecting a linked option writes {item, param}', () => {
    setOptions([{ item: CGT, param: 'guild', resolved: "Smiths' Guild of Verdi" }]);
    flushSync();
    const select = comboSelect()!;
    select.value = `link:${CGT}\u0000guild`;
    select.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.ability_scores![0].parameter).toEqual({ item: CGT, param: 'guild' });
  });

  it('choosing a different option after being linked unlinks it — never keeps both', () => {
    setOptions([{ item: CGT, param: 'guild', resolved: "Smiths' Guild of Verdi" }]);
    store.entity.ability_scores = [
      { ability: ORG_LORE, score: 1, parameter: { item: CGT, param: 'guild' } },
    ];
    flushSync();
    const select = comboSelect()!;
    select.value = 'cat:language.latin';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.ability_scores![0].parameter).toEqual({ id: 'language.latin' });
  });

  it('dirties the document when a combo option is chosen', async () => {
    // A clean baseline WITH the row: `store.open()` is the one flow that
    // resets `dirty` to false while also replacing the entity, unlike the
    // shared `beforeEach`'s direct `ability_scores` write (already a change
    // from whatever baseline preceded it). `effectiveScores` resolves once,
    // immediately, for this load's own revalidate — overriding the file's
    // deferred mock for exactly one call — then `setOptions` restores the
    // combo box's fixture for the actual assertion below.
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/saga.armc',
      entity: { ...store.entity, ability_scores: [{ ability: ORG_LORE, score: 1 }] },
      migrated_aging_characteristics: [],
    });
    vi.mocked(ipc.effectiveScores).mockResolvedValueOnce({
      ability_bonuses: [],
      ability_parameter_options: [],
    } as unknown as EffectiveScores);

    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
    setOptions([]);
    flushSync();

    expect(store.dirty).toBe(false);
    const select = comboSelect();
    select.value = 'cat:language.latin';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.dirty).toBe(true);
  });

  it('removing the linking Virtue converts the row to text live, without a reload', async () => {
    setOptions([{ item: CGT, param: 'guild', resolved: "Smiths' Guild of Verdi" }]);
    store.entity.selections = [{ ref: CGT }];
    store.entity.ability_scores = [
      { ability: ORG_LORE, score: 1, parameter: { item: CGT, param: 'guild' } },
    ];
    flushSync();

    vi.mocked(ipc.unlinkAbilityParameters).mockResolvedValueOnce({
      ...store.entity,
      ability_scores: [
        { ability: ORG_LORE, score: 1, parameter: { text: "Smiths' Guild of Verdi" } },
      ],
    });

    await store.removeSelectionAt(0);
    flushSync();

    expect(store.entity.selections).toEqual([]);
    expect(store.entity.ability_scores![0].parameter).toEqual({
      text: "Smiths' Guild of Verdi",
    });
  });
});

// X10b: the banked-XP input writes through the store like every other picker
// edit, and dirties the document exactly as any other entity edit does. A
// `client` test because it exercises the real `oninput` wiring, not just the
// rendered markup. Red-checkpoint protocol, phase 1: `AbilityTab.svelte`
// carries no such input yet, so `bankedXpInput` throws looking for an element
// that does not exist.
describe('AbilityTab banked XP input writes through the store (X10b)', () => {
  function bankedXpInput(): HTMLInputElement {
    const testid = `ability-banked-xp-${ATHLETICS}-0`;
    const el = target.querySelector<HTMLInputElement>(`[data-testid="${testid}"]`);
    if (!el) throw new Error(`banked XP input not rendered: no [data-testid="${testid}"]`);
    return el;
  }

  it("writes a typed value onto the Ability row's banked_xp", () => {
    const input = bankedXpInput();
    input.value = '4';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(store.entity.ability_scores![0].banked_xp).toBe(4);
  });

  it('typing 0 clears/omits the banked XP, leaving the bought score alone', () => {
    store.entity.ability_scores = [{ ability: ATHLETICS, score: 3, banked_xp: 4 }];
    flushSync();
    const input = bankedXpInput();
    input.value = '0';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(store.entity.ability_scores![0].score).toBe(3);
    expect(store.entity.ability_scores![0].banked_xp ?? 0).toBe(0);
  });

  it('dirties the document when banked XP is edited', async () => {
    // A CLEAN baseline, which the dirty assertion needs and only a load can
    // give (mirrors SagaYearField.client.test.ts). `mockResolvedValueOnce`
    // short-circuits the deferred `effectiveScores` mock for exactly the one
    // call `open()`'s own revalidate makes.
    vi.mocked(ipc.loadEntity).mockResolvedValue({
      path: '/tmp/saga.armc',
      entity: { ...store.entity },
      migrated_aging_characteristics: [],
    });
    vi.mocked(ipc.effectiveScores).mockResolvedValueOnce({
      ability_bonuses: [],
    } as unknown as EffectiveScores);
    const opening = store.open();
    if (store.discardPromptOpen) store.resolveDiscardPrompt(true);
    await opening;
    flushSync();
    expect(store.dirty).toBe(false);

    const input = bankedXpInput();
    input.value = '2';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(store.dirty).toBe(true);
  });
});
