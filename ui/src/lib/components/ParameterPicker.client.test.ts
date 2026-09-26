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
