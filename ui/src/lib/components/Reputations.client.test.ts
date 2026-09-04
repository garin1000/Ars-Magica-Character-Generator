import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { EffectiveScores, LocalizedRuleset } from '../types';

// A `client` test, because the claim under test is about what does NOT happen at
// runtime. The panel derives its rows from the grants rather than materializing
// them onto the entity, and the unsaved-changes guard depends on that: writing a
// granted Reputation row while merely rendering would make `store.dirty` true the
// instant a character with a granting Virtue/Flaw is opened, so closing it would
// prompt to discard edits the player never made.
//
// SSR cannot prove this. `$effect` bodies never run under the server renderer, so
// an effect that wrote to the entity would sit there green and unobserved — the
// exact silent gap the `client` project exists for. Only a mounted component with
// `flushSync()` actually runs the effects that could write.
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
    },
  } as unknown as LocalizedRuleset;
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(async () => {
  store.lang = 'en';
  installRuleset();
  // A freshly opened character: a clean saved baseline, so `dirty` is false and
  // any write the panel performs would flip it.
  await store.createCharacter('companion');
  store.effective = {
    reputation_grants: [
      { source: 'flaw.infamous', kind: 'local', score: 4 },
      { source: 'virtue.famous', kind: null, score: 4 },
    ],
  } as unknown as EffectiveScores;
  target = document.createElement('div');
  document.body.appendChild(target);
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
  store.effective = null;
});

describe('Reputations writes nothing on load', () => {
  it('renders every granted slot without touching the entity or the dirty flag', () => {
    expect(store.dirty).toBe(false);

    app = mount(Reputations, { target });
    flushSync();

    // Both grants are on screen...
    expect(target.querySelector('[data-testid="reputation-content-0"]')).not.toBeNull();
    expect(target.querySelector('[data-testid="reputation-content-1"]')).not.toBeNull();
    // ...and nothing has been written to the character for them.
    expect(store.entity.reputations ?? []).toEqual([]);
    expect(store.dirty).toBe(false);
  });

  it('writes only once the player types a description, and unwrites when emptied', () => {
    app = mount(Reputations, { target });
    flushSync();

    const input = target.querySelector<HTMLInputElement>('[data-testid="reputation-content-0"]')!;
    input.value = 'dragon slayer';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(store.entity.reputations).toEqual([
      { kind: 'local', score: 4, content: 'dragon slayer' },
    ]);
    expect(store.dirty).toBe(true);

    input.value = '';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(store.entity.reputations).toEqual([]);
  });

  it('persists the type picked on a wildcard slot before any description exists', () => {
    app = mount(Reputations, { target });
    flushSync();

    const select = target.querySelector<HTMLSelectElement>('[data-testid="reputation-kind-1"]')!;
    select.value = 'hermetic';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();
    expect(store.entity.reputations).toEqual([{ kind: 'hermetic', score: 4, content: '' }]);
  });
});
