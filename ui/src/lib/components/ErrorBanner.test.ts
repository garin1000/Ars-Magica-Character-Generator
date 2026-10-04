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
  store.unresolvedCatalogueParameters = [];
  store.migratedCatalogueParameters = [];
  store.movedAbilityParameters = [];
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

// CV4b (design § 5.5, plan-review gap): an unresolved catalogued parameter — a
// value that did not spell out any catalogue entry's name in either locale and
// so stayed free text — must reach the player exactly like a migrated aging
// Characteristic does. § 4 rule 1's Literal-only matching means a
// previously-working-by-luck authorization or restricted-pool funding can
// silently stop applying, so staying silent here would hide exactly that.
describe('ErrorBanner — the unresolved-catalogued-parameter notice', () => {
  it('renders nothing when nothing is unresolved', () => {
    const body = html();
    expect(openTag(body, 'unresolved-catalogue-notice')).toBeNull();
  });

  it('renders the notice with the Ability’s localized name, never the raw id', () => {
    store.ruleset!.i18n['ability.dead_language'] = { name: 'Dead Language' };
    store.unresolvedCatalogueParameters = [{ ability: 'ability.dead_language', text: 'Klingon' }];

    const body = html();

    expect(openTag(body, 'unresolved-catalogue-notice')).not.toBeNull();
    expect(body).toContain('Dead Language');
    expect(body).toContain('Klingon');
    expect(body).not.toContain('ability.dead_language');
  });

  it('announces the notice politely, never as an alert', () => {
    store.ruleset!.i18n['ability.dead_language'] = { name: 'Dead Language' };
    store.unresolvedCatalogueParameters = [{ ability: 'ability.dead_language', text: 'Klingon' }];

    const tag = openTag(html(), 'unresolved-catalogue-notice')!;

    expect(tag).toContain('role="status"');
    expect(tag).not.toContain('role="alert"');
  });
});

// CV4b's positive counterpart (design § 5.5): a value the fold DID recognize as
// a catalogue entry's name is reported too — "what you typed is now linked to
// its catalogue entry" — not only the failure case above.
describe('ErrorBanner — the migrated-catalogued-parameter notice', () => {
  it('renders nothing when nothing was recognized', () => {
    const body = html();
    expect(openTag(body, 'migrated-catalogue-notice')).toBeNull();
  });

  it('renders the notice with the localized Ability name and the resolved catalogue value, never a raw id', () => {
    store.ruleset!.i18n['ability.dead_language'] = { name: 'Dead Language' };
    store.migratedCatalogueParameters = [
      { ability: 'ability.dead_language', text: 'latein', resolved: 'language.latin' },
    ];

    const body = html();

    expect(openTag(body, 'migrated-catalogue-notice')).not.toBeNull();
    expect(body).toContain('Dead Language');
    expect(body).toContain('latein');
    // The resolved catalogue id must render as a name, not the slug itself.
    expect(body).toContain('Latin');
    expect(body).not.toContain('language.latin');
    expect(body).not.toContain('ability.dead_language');
  });

  it('announces the notice politely, never as an alert', () => {
    store.ruleset!.i18n['ability.dead_language'] = { name: 'Dead Language' };
    store.migratedCatalogueParameters = [
      { ability: 'ability.dead_language', text: 'latein', resolved: 'language.latin' },
    ];

    const tag = openTag(html(), 'migrated-catalogue-notice')!;

    expect(tag).toContain('role="status"');
    expect(tag).not.toContain('role="alert"');
  });
});

// L1b (try-out finding 6, decisions C5): loading a pre-22 save moved a language
// instance between Dead Language and Living Language (L1a split their lists).
// The player is told once, in the same polite sibling block as the CV4b notices.
describe('ErrorBanner — the moved-ability-parameter notice', () => {
  function installLanguages(): void {
    store.ruleset!.i18n['ability.dead_language'] = {
      name: '{language} (Dead Language)',
      name_unfilled: 'Dead Language',
    };
    store.ruleset!.i18n['ability.living_language'] = {
      name: '{language} (Living Language)',
      name_unfilled: 'Living Language',
    };
    store.ruleset!.i18n['language.arabic'] = { name: 'Arabic' };
    store.ruleset!.ruleset.abilities = {
      'ability.dead_language': {
        id: 'ability.dead_language',
        category: 'academic',
        parameter: 'language',
      },
      'ability.living_language': {
        id: 'ability.living_language',
        category: 'general',
        parameter: 'language',
      },
    };
  }

  it('renders nothing when nothing moved', () => {
    expect(openTag(html(), 'moved-ability-notice')).toBeNull();
  });

  it('renders the notice with localized names, never a raw id', () => {
    installLanguages();
    store.movedAbilityParameters = [
      {
        from: 'ability.dead_language',
        to: 'ability.living_language',
        value: 'language.arabic',
        score: 3,
      },
    ];

    const body = html();

    expect(openTag(body, 'moved-ability-notice')).not.toBeNull();
    expect(body).toContain('Arabic');
    expect(body).toContain('Dead Language');
    expect(body).toContain('Living Language');
    expect(body).not.toContain('language.arabic');
    expect(body).not.toContain('ability.dead_language');
  });

  it('announces the notice politely, never as an alert', () => {
    installLanguages();
    store.movedAbilityParameters = [
      {
        from: 'ability.dead_language',
        to: 'ability.living_language',
        value: 'language.arabic',
        score: 3,
      },
    ];

    const tag = openTag(html(), 'moved-ability-notice')!;

    expect(tag).toContain('role="status"');
    expect(tag).not.toContain('role="alert"');
  });
});
