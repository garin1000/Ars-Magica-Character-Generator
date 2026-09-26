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
      'flaw.outsider_major': { name: 'Outsider' },
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

  it('writes the pre-filled minimum level for a ranged grant (Outsider, D11/Q5)', () => {
    // Outsider grants Local 1, max_score 3 ("a bad Reputation of level 1 to
    // 3", ArMDE:6554). Before the level is ever raised, the first description
    // typed must record the grant's own score — its minimum — exactly like
    // any exact grant, not some other figure.
    store.effective = {
      reputation_grants: [{ source: 'flaw.outsider_major', kind: 'local', score: 1, max_score: 3 }],
    } as unknown as EffectiveScores;
    app = mount(Reputations, { target });
    flushSync();

    const input = target.querySelector<HTMLInputElement>('[data-testid="reputation-content-0"]')!;
    input.value = 'outcast';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(store.entity.reputations).toEqual([{ kind: 'local', score: 1, content: 'outcast' }]);
  });
});

describe('Reputations level control (D11/Q5, D58)', () => {
  // Outsider is the one grant in the catalogue stating a range: "a bad
  // Reputation of level 1 to 3" (ArMDE:6554). A legal character with level 2
  // or 3 must be enterable, so the panel needs a level control bounded to
  // `[grant.score, grant.max_score]` — an exact grant (Infamous, Famous) gets
  // none, since there is nothing to choose.
  function installOutsiderGrant(): void {
    store.effective = {
      reputation_grants: [{ source: 'flaw.outsider_major', kind: 'local', score: 1, max_score: 3 }],
    } as unknown as EffectiveScores;
  }

  it('raises the level within the grant’s range and cannot exceed max_score', () => {
    installOutsiderGrant();
    app = mount(Reputations, { target });
    flushSync();

    const inc = target.querySelector<HTMLButtonElement>('[data-testid="reputation-level-inc-0"]')!;
    expect(inc).not.toBeNull();

    inc.click();
    flushSync();
    expect(store.entity.reputations).toEqual([{ kind: 'local', score: 2, content: '' }]);

    inc.click();
    flushSync();
    expect(store.entity.reputations).toEqual([{ kind: 'local', score: 3, content: '' }]);

    // At the grant's max_score, the increment control is disabled — the
    // Spinner's own bound, the same mechanism ArtGrid/AbilityTab use at their
    // caps — so a further click is a no-op rather than an over-range score.
    expect(inc.disabled).toBe(true);
    inc.click();
    flushSync();
    expect(store.entity.reputations).toEqual([{ kind: 'local', score: 3, content: '' }]);
  });

  it('cannot lower the level below the grant’s minimum', () => {
    installOutsiderGrant();
    app = mount(Reputations, { target });
    flushSync();

    const dec = target.querySelector<HTMLButtonElement>('[data-testid="reputation-level-dec-0"]')!;
    expect(dec.disabled).toBe(true);
    dec.click();
    flushSync();
    // Still unfilled — a disabled decrement never fired addReputation.
    expect(store.entity.reputations ?? []).toEqual([]);
  });

  it('preserves an already-typed description when the level is raised', () => {
    installOutsiderGrant();
    app = mount(Reputations, { target });
    flushSync();

    const input = target.querySelector<HTMLInputElement>('[data-testid="reputation-content-0"]')!;
    input.value = 'outcast';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();

    const inc = target.querySelector<HTMLButtonElement>('[data-testid="reputation-level-inc-0"]')!;
    inc.click();
    flushSync();
    expect(store.entity.reputations).toEqual([{ kind: 'local', score: 2, content: 'outcast' }]);
  });

  it('offers no level control for an exact grant', () => {
    // Infamous grants exactly 4 (ArMDE:6310-6312) — nothing to choose.
    app = mount(Reputations, { target });
    flushSync();

    expect(target.querySelector('[data-testid="reputation-level-inc-0"]')).toBeNull();
    expect(target.querySelector('[data-testid="reputation-level-dec-0"]')).toBeNull();
  });
});
