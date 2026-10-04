import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { AbilityParamValue, LinkTarget, LocalizedRuleset } from '../types';

// N4b: what the combo reports to its owner when the player uses it. A `client`
// test because the events need a live component; the markup is pinned by the
// SSR sibling `AbilityParameterCombo.test.ts`.
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
import AbilityParameterCombo from './AbilityParameterCombo.svelte';

const LINK: LinkTarget = {
  item: 'virtue.craft_guild_training',
  param: 'guild',
  resolved: 'Smiths of Verdi',
};

let target: HTMLElement;
let component: ReturnType<typeof mount> | null = null;
const onchoose = vi.fn();
const ontext = vi.fn();

function mountWith(value: AbilityParamValue | null | undefined): void {
  component = mount(AbilityParameterCombo, {
    target,
    props: {
      catalogued: ['language.arabic', 'language.greek'],
      linked: [LINK],
      linkLabel: (link: LinkTarget) => link.resolved ?? '',
      value,
      label: 'Native language',
      placeholder: 'e.g. German',
      selectTestId: 'combo-select',
      inputTestId: 'combo-input',
      onchoose,
      ontext,
    },
  });
  flushSync();
}

function select(): HTMLSelectElement {
  const el = target.querySelector<HTMLSelectElement>('[data-testid="combo-select"]');
  if (!el) throw new Error('combo select not rendered');
  return el;
}

function choose(value: string): void {
  const el = select();
  el.value = value;
  el.dispatchEvent(new Event('change', { bubbles: true }));
  flushSync();
}

beforeEach(() => {
  store.lang = 'en';
  store.ruleset = {
    ruleset: { id: 'test', version: '1', point_items: {}, type_profiles: {}, abilities: {} },
    i18n: { 'language.arabic': { name: 'Arabic' }, 'language.greek': { name: 'Greek' } },
  } as unknown as LocalizedRuleset;
  onchoose.mockClear();
  ontext.mockClear();
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (component) unmount(component);
  component = null;
  target.remove();
});

describe("AbilityParameterCombo reports the player's choice (N4b)", () => {
  it('reports a catalogue value as {id}', () => {
    mountWith({ text: 'Gaelic' });
    choose('cat:language.greek');
    expect(onchoose.mock.calls).toEqual([[{ id: 'language.greek' }]]);
  });

  it('reports a link target as {item, param}', () => {
    mountWith(undefined);
    choose(`link:${LINK.item}\u0000${LINK.param}`);
    expect(onchoose.mock.calls).toEqual([[{ item: LINK.item, param: LINK.param }]]);
  });

  it('reports "Other…" as clearing the value', () => {
    mountWith({ id: 'language.arabic' });
    choose('other');
    expect(onchoose.mock.calls).toEqual([[undefined]]);
  });

  it('reports typed text from the "Other…" field', () => {
    mountWith({ text: '' });
    const input = target.querySelector<HTMLInputElement>('[data-testid="combo-input"]');
    if (!input) throw new Error('"Other…" field not rendered');
    input.value = 'Gaelic';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(ontext.mock.calls).toEqual([['Gaelic']]);
  });

  it('writes nothing on mount, even for text spelling a catalogue name', () => {
    // A normalising write on mount would move the dirty baseline of a document
    // the player merely opened; the load fold is where text becomes a value.
    mountWith({ text: 'Arabic' });
    expect(onchoose).not.toHaveBeenCalled();
    expect(ontext).not.toHaveBeenCalled();
  });
});
