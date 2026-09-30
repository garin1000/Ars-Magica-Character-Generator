import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset, PointItem, Selection } from '../types';

// D35 (Phase 2 C3): the number domain's bound is enforced in the component,
// not just declared as HTML `min`/`max` attributes (which a browser does not
// itself block typing past) — SSR can render the control but cannot fire a
// `change` event, so the clamp itself needs a mounted component to observe.
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
import ParameterPickerHarness from './ParameterPickerHarness.svelte';

const PROBE_ITEM = {
  id: 'virtue.simple_student_probe',
  kind: 'virtue',
  magnitude: 'minor',
  categories: ['general'],
  classification: 'narrative',
  entity_kinds: ['character'],
  parameters: [{ key: 'years', type: { number: { min: 1, max: 2 } }, domain: 'number' }],
} as unknown as PointItem;

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: { 'virtue.simple_student_probe': PROBE_ITEM },
      type_profiles: {
        magus: {
          id: 'magus',
          budget: { virtue_points: 10, flaw_points: 10 },
          hermetically_trained: true,
          order_member: true,
          gift_policy: 'required',
          creation_phases: [],
        },
      },
      abilities: {},
      arts: {},
      spells: {},
      houses: {},
      mythic_types: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: { 'virtue.simple_student_probe': { name: 'Simple Student Probe' } },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    selections: [{ ref: 'virtue.simple_student_probe', params: {} }],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
  store.result = { issues: [] };
}

let target: HTMLElement;
let app: { setSelection: (next: Selection) => void } | undefined;

function input(): HTMLInputElement {
  return target.querySelector(
    '[data-testid="param-virtue.simple_student_probe-years-0"]',
  ) as HTMLInputElement;
}

function selection(): Selection {
  return (store.entity.selections ?? [])[0];
}

/**
 * Types a value into the box and commits it (blur/Enter — a `change` event),
 * then hands the harness the fresh `Selection` `setParamAt` just wrote, the
 * same way the real app's `{#each store.entity.selections as selection}`
 * would on its own next render (see `ParameterPickerHarness.svelte`).
 */
function setAndCommit(value: string): void {
  const el = input();
  el.value = value;
  el.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
  app?.setSelection(selection());
  flushSync();
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();

  target = document.createElement('div');
  document.body.appendChild(target);
  app = mount(ParameterPickerHarness, {
    target,
    props: {
      initial: selection(),
      index: 0,
      params: PROBE_ITEM.parameters!,
    },
  }) as unknown as { setSelection: (next: Selection) => void };
  flushSync();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  store.result = null;
  vi.clearAllTimers();
  vi.useRealTimers();
});

describe('ParameterPicker number domain bounded input (D35)', () => {
  it('clamps a value above max down to max on commit', () => {
    setAndCommit('5');
    expect(input().value).toBe('2');
    expect(selection().params?.years).toBe('2');
  });

  it('clamps a value below min up to min on commit', () => {
    setAndCommit('0');
    expect(input().value).toBe('1');
    expect(selection().params?.years).toBe('1');
  });

  it('leaves an in-range value untouched', () => {
    setAndCommit('2');
    expect(input().value).toBe('2');
    expect(selection().params?.years).toBe('2');
  });

  it('does not clamp while the player is still typing — only on commit', () => {
    const el = input();
    el.value = '5';
    el.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    // `input` alone must not clamp: only `change` (commit) does, so a wider
    // future range (D3's truncated-apprenticeship-age reuse) is never
    // rewritten mid-keystroke.
    expect(el.value).toBe('5');
    expect(selection().params?.years).toBeUndefined();
  });

  it('records a committed blank box as an empty (not-yet-made) choice', () => {
    setAndCommit('1');
    setAndCommit('');
    expect(input().value).toBe('');
    expect(selection().params?.years).toBe('');
  });
});

// C5b (D9 part 3, `docs/vf-audit/design-c0-parameter-model.md` § 8, § 10): the
// multi-select control for a `multi_ref` parameter — `flaw.corrupted_spells_probe`
// stands in for Corrupted Spells (ArMDE:5859-5864), since no shipped entry uses
// `multi_ref` yet (C5c). A `client` test, because selecting/deselecting a
// checkbox and observing the resulting write and the live `store.dirty` flag are
// both actions/reactive reads SSR cannot exercise (see this file's own header
// comment on why the D35 number-domain tests above are `client` too).
//
// RED-CHECKPOINT PHASE 1: `ParameterPicker.svelte`'s `param.type === 'multi_ref'`
// branch is a deliberately empty stub, so no checkbox exists yet — every test
// below is expected to fail at its OWN `expect(cb).not.toBeNull()` (or
// `expect(fieldset()).not.toBeNull()`) assertion, which is the right place to
// fail: it names exactly the missing control, rather than crashing later on a
// null dereference.
describe('ParameterPicker multi-select (multi_ref, C5b, D9 part 3)', () => {
  const MULTI_ITEM = {
    id: 'flaw.corrupted_spells_probe',
    kind: 'flaw',
    magnitude: 'minor',
    categories: ['general'],
    classification: 'uncomputed_rule',
    entity_kinds: ['character'],
    parameters: [{ key: 'targets', type: 'multi_ref', domain: 'spell' }],
  } as unknown as PointItem;

  const LEARNED_A = 'spell.pilum_of_fire';
  const LEARNED_B = 'spell.aegis_of_the_hearth';
  const UNLEARNED = 'spell.unlearned_probe';
  const TESTID = 'param-flaw.corrupted_spells_probe-targets-0';

  let multiTarget: HTMLElement;
  let multiApp: { setSelection: (next: Selection) => void } | undefined;

  function fieldset(): HTMLElement | null {
    return multiTarget.querySelector(`[data-testid="${TESTID}"]`);
  }
  function checkbox(spellId: string): HTMLInputElement | null {
    return multiTarget.querySelector(`[data-testid="${TESTID}-${spellId}"]`);
  }
  function multiSelection(): Selection {
    return (store.entity.selections ?? [])[0];
  }

  /**
   * A CLEAN saved baseline (`store.dirty === false`) that already carries the
   * probe selection and the character's two learned spells. `store.save()`
   * (not `open()`) is the mechanism: `save()`/`#writeTo()` has no
   * dirty-check/discard-prompt gate of its own (only `open()`/`newDocument()`
   * do), so assigning the fixture entity and immediately saving it — through
   * the mocked `ipc.saveEntity` — marks exactly that state as the baseline
   * with no discard-prompt dance to race against.
   */
  async function establishCleanMultiRefBaseline(): Promise<void> {
    store.ruleset!.ruleset.point_items['flaw.corrupted_spells_probe'] = MULTI_ITEM;
    store.ruleset!.ruleset.spells = {
      [LEARNED_A]: { id: LEARNED_A, technique: 'art.creo', form: 'art.ignem' },
      [LEARNED_B]: { id: LEARNED_B, technique: 'art.rego', form: 'art.aquam' },
      [UNLEARNED]: { id: UNLEARNED, technique: 'art.creo', form: 'art.rego' },
    };
    store.ruleset!.i18n['flaw.corrupted_spells_probe'] = { name: 'Corrupted Spells Probe' };
    store.ruleset!.i18n[LEARNED_A] = { name: 'Pilum of Fire' };
    store.ruleset!.i18n[LEARNED_B] = { name: 'Aegis of the Hearth' };
    store.ruleset!.i18n[UNLEARNED] = { name: 'Unlearned Probe' };
    store.entity = {
      schema_version: SCHEMA_VERSION,
      ruleset: { id: 'test', version: '1' },
      entity_kind: 'character',
      type_id: 'magus',
      selections: [{ ref: 'flaw.corrupted_spells_probe', params: {} }],
      characteristics: {} as Entity['characteristics'],
      characteristic_descriptions: {},
      ability_scores: [],
      xp_pool: 0,
      ability_funding: 'pool',
      saga_year: 1220,
      art_scores: [],
      personality_traits: [],
      reputations: [],
      spells: [{ spell: LEARNED_A }, { spell: LEARNED_B }],
    };
    store.currentPath = null;
    await store.save();
  }

  beforeEach(async () => {
    await establishCleanMultiRefBaseline();
    multiTarget = document.createElement('div');
    document.body.appendChild(multiTarget);
    multiApp = mount(ParameterPickerHarness, {
      target: multiTarget,
      props: { initial: multiSelection(), index: 0, params: MULTI_ITEM.parameters! },
    }) as unknown as { setSelection: (next: Selection) => void };
    flushSync();
  });

  afterEach(() => {
    if (multiApp) unmount(multiApp);
    multiApp = undefined;
    multiTarget?.remove();
  });

  /** Toggles a checkbox and re-syncs the harness with the fresh `Selection`
   *  `store.setMultiParamAt` (once it exists, phase 2) writes — mirrors
   *  `setAndCommit` above. Asserts the checkbox exists FIRST, so a phase-1
   *  RED fails there rather than crashing on a null dereference. */
  function toggle(spellId: string, checked: boolean): void {
    const cb = checkbox(spellId);
    expect(cb).not.toBeNull();
    cb!.checked = checked;
    cb!.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    multiApp?.setSelection(multiSelection());
    flushSync();
  }

  it('renders one checkbox per learned spell and none for an unlearned one', () => {
    expect(fieldset()).not.toBeNull();
    expect(checkbox(LEARNED_A)).not.toBeNull();
    expect(checkbox(LEARNED_B)).not.toBeNull();
    expect(checkbox(UNLEARNED)).toBeNull();
  });

  it('selecting two values writes them as a sorted, deduplicated array', () => {
    // Click order deliberately reversed from sorted order ('spell.aegis_of_the_hearth'
    // < 'spell.pilum_of_fire'), so a passing test proves canonicalization runs
    // rather than merely preserving insertion order.
    toggle(LEARNED_A, true);
    toggle(LEARNED_B, true);
    expect(multiSelection().params?.targets).toEqual([LEARNED_B, LEARNED_A]);
  });

  it('deselecting one value removes only that member', () => {
    toggle(LEARNED_A, true);
    toggle(LEARNED_B, true);
    toggle(LEARNED_A, false);
    expect(multiSelection().params?.targets).toEqual([LEARNED_B]);
  });

  it('is keyboard-operable — a native checkbox needs no key handler of its own', () => {
    const cb = checkbox(LEARNED_A);
    expect(cb).not.toBeNull();
    expect(cb!.type).toBe('checkbox');
    // A real, reachable Tab stop — the DOM's own answer to "can the keyboard
    // land here" (mirrors `ArtGrid.client.test.ts`'s Sabine-3 precedent).
    expect(cb!.tabIndex).toBeGreaterThanOrEqual(0);
    // `.click()` is the same DOM outcome (checked flips, `change` fires) a
    // Space/Enter press on a focused checkbox produces natively — a native
    // <input type="checkbox"> needs no bespoke keydown handler for this.
    cb!.click();
    flushSync();
    multiApp?.setSelection(multiSelection());
    flushSync();
    expect(multiSelection().params?.targets).toEqual([LEARNED_A]);
  });

  it('flips the dirty flag when a value is toggled', () => {
    expect(store.dirty).toBe(false);
    toggle(LEARNED_A, true);
    expect(store.dirty).toBe(true);
  });

  it('writes an empty (not absent) array once every value is unchecked — the blank/missing-param state (C5a semantics)', () => {
    toggle(LEARNED_A, true);
    toggle(LEARNED_A, false);
    expect(multiSelection().params?.targets).toEqual([]);
  });
});

// Functional review 2026-09-30 #1 (HIGH): `multiRefOptions` only handles
// `domain === 'spell'`, so today NO checkbox exists for an `ability`-domain
// multi_ref parameter — the exact shape `flaw.restricted_learning` and
// `virtue.magian_lineage_major` ship with. A client test, because checking a
// box and observing the resulting `params` write needs a live component
// exactly as the spell-domain block above does.
describe('ParameterPicker multi-select ability domain (functional review 2026-09-30 #1)', () => {
  const ABILITY_A = 'ability.parma_magica';
  const ABILITY_B = 'ability.premonitions';
  const TESTID = 'param-flaw.corrupted_abilities_probe-targets-0';

  const MULTI_ABILITY_ITEM = {
    id: 'flaw.corrupted_abilities_probe',
    kind: 'flaw',
    magnitude: 'minor',
    categories: ['general'],
    classification: 'uncomputed_rule',
    entity_kinds: ['character'],
    parameters: [{ key: 'targets', type: 'multi_ref', domain: 'ability' }],
  } as unknown as PointItem;

  let abilityTarget: HTMLElement;
  let abilityApp: { setSelection: (next: Selection) => void } | undefined;

  function fieldset(): HTMLElement | null {
    return abilityTarget.querySelector(`[data-testid="${TESTID}"]`);
  }
  function checkbox(abilityId: string): HTMLInputElement | null {
    return abilityTarget.querySelector(`[data-testid="${TESTID}-${abilityId}"]`);
  }
  function abilitySelection(): Selection {
    return (store.entity.selections ?? [])[0];
  }

  beforeEach(() => {
    store.ruleset!.ruleset.point_items['flaw.corrupted_abilities_probe'] = MULTI_ABILITY_ITEM;
    store.ruleset!.ruleset.abilities = {
      [ABILITY_A]: { id: ABILITY_A, category: 'arcane' },
      [ABILITY_B]: { id: ABILITY_B, category: 'supernatural' },
    };
    store.ruleset!.i18n['flaw.corrupted_abilities_probe'] = { name: 'Corrupted Abilities Probe' };
    store.ruleset!.i18n[ABILITY_A] = { name: 'Parma Magica' };
    store.ruleset!.i18n[ABILITY_B] = { name: 'Premonitions' };
    store.entity.selections = [{ ref: 'flaw.corrupted_abilities_probe', params: {} }];

    abilityTarget = document.createElement('div');
    document.body.appendChild(abilityTarget);
    abilityApp = mount(ParameterPickerHarness, {
      target: abilityTarget,
      props: { initial: abilitySelection(), index: 0, params: MULTI_ABILITY_ITEM.parameters! },
    }) as unknown as { setSelection: (next: Selection) => void };
    flushSync();
  });

  afterEach(() => {
    if (abilityApp) unmount(abilityApp);
    abilityApp = undefined;
    abilityTarget?.remove();
  });

  /** Mirrors `toggle` above: asserts the checkbox exists FIRST, so today's RED
   *  fails there (no options ever rendered for this domain) rather than on a
   *  null dereference. */
  function toggle(abilityId: string, checked: boolean): void {
    const cb = checkbox(abilityId);
    expect(cb).not.toBeNull();
    cb!.checked = checked;
    cb!.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    abilityApp?.setSelection(abilitySelection());
    flushSync();
  }

  it('renders one checkbox per catalogue ability', () => {
    expect(fieldset()).not.toBeNull();
    expect(checkbox(ABILITY_A)).not.toBeNull();
    expect(checkbox(ABILITY_B)).not.toBeNull();
  });

  it('checking a box writes the ability id into the multi_ref value', () => {
    toggle(ABILITY_A, true);
    expect(abilitySelection().params?.targets).toEqual([ABILITY_A]);
  });

  it('unchecking removes only that member, leaving the other checked one', () => {
    toggle(ABILITY_A, true);
    toggle(ABILITY_B, true);
    toggle(ABILITY_A, false);
    expect(abilitySelection().params?.targets).toEqual([ABILITY_B]);
  });
});
