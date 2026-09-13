import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// An `ssr` test: everything here is static rendered markup — which element a
// sentence lands in, and what live-region role it carries. No `$effect`, no
// focus, no live instance.
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
import ErrorBanner from './ErrorBanner.svelte';

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
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: 11,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'companion',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

function html(): string {
  return render(ErrorBanner).body;
}

/** The opening tag of the element carrying `data-testid`, or null. */
function openTag(body: string, testid: string): string | null {
  const match = new RegExp(`<[a-z]+[^>]*data-testid="${testid}"[^>]*>`).exec(body);
  return match?.[0] ?? null;
}

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
  store.error = null;
  store.migratedAgingCharacteristics = [];
});

// Slice 3 handoff, finished here: the notice composer, both locales' strings and
// the `OpenedDocument` wire field all shipped, but nothing rendered the result.
//
// The migration is LOSSY — the engine reconstructs the smallest Aging Point
// total that still reproduces the recorded scores, so the original is gone — and
// the next Save writes the reconstruction back as the document's own figures.
// Until this rendered, the user was told none of that.
describe('ErrorBanner — the schema-migration notice', () => {
  it('renders nothing at all with neither an error nor a migration', () => {
    // Svelte's SSR renderer always emits its hydration markers, so "empty" is
    // "carries none of this component's own elements", not a zero-length string.
    const body = html();
    expect(openTag(body, 'error')).toBeNull();
    expect(openTag(body, 'migration-notice')).toBeNull();
    expect(body).not.toContain('error-banner-block');
  });

  it('renders the notice for a migrated document, with no error present', () => {
    // It is not an error and must not depend on one: a clean load that merely
    // needed migrating is the ordinary case.
    store.migratedAgingCharacteristics = ['com', 'sta'];

    const body = html();

    expect(openTag(body, 'migration-notice')).not.toBeNull();
    expect(body).toContain(store.t('characteristic-com'));
    expect(openTag(body, 'error')).toBeNull();
  });

  it('announces the notice politely, never as an alert', () => {
    // `role="status"` (polite): the user has just opened a file and is reading
    // it, so this waits its turn. `role="alert"` is assertive and reserved for
    // the failed-operation sentence — the same split BalanceBar/XpBar use.
    store.migratedAgingCharacteristics = ['com'];

    const tag = openTag(html(), 'migration-notice')!;

    expect(tag).toContain('role="status"');
    expect(tag).not.toContain('role="alert"');
  });

  it('keeps the notice out of the error sentence when both are present', () => {
    // The alert span is `aria-atomic` by default and assertive; folding a second
    // unrelated sentence into it would have the whole thing re-announced, and
    // would tie a notice's lifetime to an error's. Siblings, exactly as
    // `ErrorDetails` is a sibling.
    store.error = { kind: 'io', message: 'disk on fire' };
    store.migratedAgingCharacteristics = ['com'];

    const body = html();

    expect(openTag(body, 'error')).not.toBeNull();
    expect(openTag(body, 'migration-notice')).not.toBeNull();
    const alert = body.slice(body.indexOf('data-testid="error"'));
    const sentence = alert.slice(0, alert.indexOf('</span>'));
    expect(sentence).not.toContain(store.t('characteristic-com'));
  });

  it('says the reconstruction is approximate, not merely that something changed', () => {
    // The reason the notice exists: a copy-edit reducing it to "upgraded" would
    // lose the only warning that data was lost.
    store.migratedAgingCharacteristics = ['com'];

    expect(html().toLowerCase()).toMatch(/smallest|minimal|approximat/);
  });
});
