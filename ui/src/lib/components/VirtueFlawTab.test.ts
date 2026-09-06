import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LocalizedRuleset, PointItem, Selection } from '../types';

// The tab reads the shared store singleton (catalogue, entity selections, the
// engine's granted selections) and the Fluent bundle. The store's mutators go
// over the Tauri IPC bridge, so mock it away. Harness mirrors AbilityTab.test.ts.
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
import VirtueFlawTab from './VirtueFlawTab.svelte';

function item(overrides: Partial<PointItem> & Pick<PointItem, 'id'>): PointItem {
  return {
    kind: 'virtue',
    magnitude: 'minor',
    categories: ['general'],
    classification: 'narrative',
    entity_kinds: ['character'],
    ...overrides,
  } as PointItem;
}

// Three Virtues in two categories, plus one Supernatural one so a category can
// sort AFTER the granted item's own — the arrangement that made the old
// header-less granted group read as "Supernatural" (#9).
const ITEMS: PointItem[] = [
  item({ id: 'virtue.affinity', categories: ['general'] }),
  item({ id: 'virtue.heartbeast', categories: ['hermetic'] }),
  item({ id: 'virtue.hermetic_prestige', categories: ['hermetic'] }),
  item({ id: 'virtue.second_sight', categories: ['supernatural'] }),
  // The shipped dual-category case: Sufi's descriptor reads "Minor, Social
  // Status, Supernatural", and the book's index lists it under both headings
  // (Core Rules :3230 Social Status, Minor and :3179 Supernatural, Minor).
  item({ id: 'virtue.sufi', categories: ['social_status', 'supernatural'] }),
];

const NAMES: LocalizedRuleset['i18n'] = {
  'virtue.affinity': { name: 'Affinity with Art' },
  'virtue.heartbeast': { name: 'Heartbeast' },
  'virtue.hermetic_prestige': { name: 'Hermetic Prestige' },
  'virtue.second_sight': { name: 'Second Sight' },
  'virtue.sufi': { name: 'Sufi' },
};

function installRuleset(): void {
  const point_items: Record<string, PointItem> = {};
  for (const it of ITEMS) point_items[it.id] = it;
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items,
      type_profiles: {
        magus: {
          id: 'magus',
          budget: { virtue_points: 10, flaw_points: 10 },
          is_magus: true,
          gift_policy: 'required',
          creation_phases: [],
        },
      },
      abilities: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: NAMES,
  } as unknown as LocalizedRuleset;
}

function resetEntity(selections: Selection[] = []): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: 'magus',
    selections,
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
  };
  store.effective = null;
  store.result = { issues: [] };
}

function grant(...granted: Selection[]): void {
  store.effective = { granted_selections: granted } as unknown as EffectiveScores;
}

function html(): string {
  return render(VirtueFlawTab, { props: {} }).body;
}

/** `app.css` as text, for the layout rules no server-rendered markup can reveal. */
const appCss = readFileSync(fileURLToPath(new URL('../../app.css', import.meta.url)), 'utf-8');

/** Fluent isolates interpolated values with bidi marks; strip them for text matching. */
function clean(text: string): string {
  return text.replace(/[⁦-⁩]/g, '');
}

/** The Virtues column's markup (the side both fixtures' items live on). */
function virtueColumn(body: string): string {
  const start = body.indexOf('data-testid="selection-list-virtue"');
  if (start < 0) throw new Error('no Virtues column');
  const end = body.indexOf('data-testid="selection-list-flaw"');
  return body.slice(start, end < 0 ? undefined : end);
}

/**
 * The category headings and row names of the Virtues column, in document order:
 * `['# Hermetic', 'Heartbeast', …]` — enough to see which heading a row sits
 * under without a DOM (these component tests render to a string).
 */
function columnOutline(body: string): string[] {
  const column = virtueColumn(body);
  const out: string[] = [];
  for (const m of column.matchAll(
    /<h3 class="category">([\s\S]*?)<\/h3>|<span class="item-name">([\s\S]*?)<\/span>/g,
  )) {
    if (m[1] !== undefined) out.push(`# ${m[1].replace(/<[^>]*>/g, '').trim()}`);
    else
      out.push(
        m[2]
          .replace(/<[^>]*>/g, '')
          .replace(/[⁦-⁩]/g, '')
          .trim(),
      );
  }
  return out;
}

/** The Virtues *source* (Available) picker's markup: its filter bar through to the Flaws one. */
function virtueSourceColumn(body: string): string {
  const start = body.indexOf('data-testid="vf-search-virtue"');
  if (start < 0) throw new Error('no Virtues source picker');
  const end = body.indexOf('data-testid="vf-search-flaw"');
  return body.slice(start, end < 0 ? undefined : end);
}

/** `columnOutline`, but over the Available picker instead of the Selected column. */
function sourceOutline(body: string): string[] {
  const column = virtueSourceColumn(body);
  const out: string[] = [];
  for (const m of column.matchAll(
    /<h3 class="category"[^>]*>([\s\S]*?)<\/h3>|<span class="item-name">([\s\S]*?)<\/span>/g,
  )) {
    if (m[1] !== undefined) out.push(`# ${m[1].replace(/<[^>]*>/g, '').trim()}`);
    else out.push(clean(m[2].replace(/<[^>]*>/g, '')).trim());
  }
  return out;
}

/** Every `<ul>` start tag in the Virtues column, with the heading (if any) before it. */
function listsWithHeaders(body: string): { header: string | null }[] {
  const column = virtueColumn(body);
  const lists: { header: string | null }[] = [];
  let pendingHeader: string | null = null;
  for (const m of column.matchAll(/<h3 class="category">([\s\S]*?)<\/h3>|<ul\b/g)) {
    if (m[1] !== undefined) pendingHeader = m[1].replace(/<[^>]*>/g, '').trim();
    else {
      lists.push({ header: pendingHeader });
      pendingHeader = null;
    }
  }
  return lists;
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  store.view = 'editor';
  installRuleset();
  resetEntity();
});

// guided-creation-review-2026-08 #9 (DECIDED): granted V/F used to be pushed into
// a HEADER-LESS group below the chosen ones. Because a header-less group inherits
// the last heading rendered above it, a Hermetic-badged granted Virtue appeared
// under the SUPERNATURAL heading — actively misleading. They now join the ordinary
// category-grouped list, keyed off each grant's OWN category.
describe('VirtueFlawTab merges granted Virtues into the category list (#9)', () => {
  it('renders no separate granted group', () => {
    resetEntity([{ ref: 'virtue.second_sight' }]);
    grant({ ref: 'virtue.heartbeast' });
    // Every `<ul>` on the side carries a heading: there is no header-less group
    // left for a granted row to fall into.
    expect(listsWithHeaders(html()).every((l) => l.header !== null)).toBe(true);
  });

  it('renders a granted Hermetic virtue under the Hermetic heading', () => {
    // A bought Supernatural Virtue gives the list a category sorting AFTER
    // `hermetic`, which is exactly what the old trailing group inherited.
    resetEntity([{ ref: 'virtue.second_sight' }]);
    grant({ ref: 'virtue.heartbeast' });
    expect(columnOutline(html())).toEqual([
      '# Hermetic',
      'Heartbeast',
      '# Supernatural',
      'Second Sight',
    ]);
  });

  it('orders a granted row among the bought ones by localized name', () => {
    resetEntity([{ ref: 'virtue.hermetic_prestige' }, { ref: 'virtue.affinity' }]);
    grant({ ref: 'virtue.heartbeast' });
    expect(columnOutline(html())).toEqual([
      '# General',
      'Affinity with Art',
      '# Hermetic',
      // Granted Heartbeast sorts before bought Hermetic Prestige — by name, not
      // by provenance.
      'Heartbeast',
      'Hermetic Prestige',
    ]);
  });

  it('keeps the Granted marker as the granted row’s only distinction', () => {
    grant({ ref: 'virtue.heartbeast' });
    const column = virtueColumn(html());
    expect(column).toContain('>Granted<');
    // No remove button on a row the player did not buy.
    expect(column).not.toContain('data-testid="remove-virtue.heartbeast');
  });

  // The engine concatenates House, Mythic Companion, `grants_selection` and
  // warping grants WITHOUT dedup (`effective.rs` `entity_grants`), so one ref can
  // be granted twice. Both rows must render, and each must be individually
  // identifiable — the `{#each}` key that keeps them apart is asserted for real
  // (a duplicate throws `each_key_duplicate`) in VirtueFlawTab.client.test.ts,
  // because that error is raised only by the client reconciler.
  it('keys granted rows uniquely when the same ref is granted twice', () => {
    grant({ ref: 'virtue.heartbeast' }, { ref: 'virtue.heartbeast' });
    const column = virtueColumn(html());
    const ids = [...column.matchAll(/data-testid="(granted-selection-[^"]+)"/g)].map((m) => m[1]);
    expect(ids).toHaveLength(2);
    expect(new Set(ids).size).toBe(2);
  });
});

// The rulebook's Virtue index lists Sufi twice — at
// `Ars Magica - Definitive Edition (Core Rules).md:3179` under
// "### Supernatural, Minor" and at :3230 under "### Social Status, Minor" — so
// the Available list offers it under both headings, exactly as the book does.
// The Selected list cannot: its rows are removed by `entity.selections` index.
describe('VirtueFlawTab offers a dual-category item under every heading', () => {
  it('lists Sufi under both of its categories in the Available picker', () => {
    const outline = sourceOutline(html());
    expect(outline).toEqual([
      '# General',
      'Affinity with Art',
      '# Hermetic',
      'Heartbeast',
      'Hermetic Prestige',
      '# Social Status',
      'Sufi',
      '# Supernatural',
      'Second Sight',
      'Sufi',
    ]);
  });

  it('still lists a bought Sufi as a single Selected row', () => {
    resetEntity([{ ref: 'virtue.sufi' }]);
    expect(columnOutline(html())).toEqual(['# Social Status', 'Sufi']);
  });

  // Listing an item under both headings makes the category FILTER's job explicit:
  // narrowing to Supernatural must show the Supernatural section only. Otherwise
  // Sufi — which passes a membership filter on either category — would come back
  // twice, once under a "Social Status" heading the player just filtered away.
  it('shows only the chosen category section when the filter narrows', () => {
    store.filters.vf.virtue.category = 'supernatural';
    try {
      expect(sourceOutline(html())).toEqual(['# Supernatural', 'Second Sight', 'Sufi']);
    } finally {
      store.filters.vf.virtue.category = '';
    }
  });

  it('offers every carried category in the filter dropdown', () => {
    const select = /data-testid="vf-category-filter-virtue"[\s\S]*?<\/select>/.exec(html());
    const options = [...(select?.[0] ?? '').matchAll(/<option value="([^"]*)"/g)].map((m) => m[1]);
    expect(options).toEqual(['', 'general', 'hermetic', 'social_status', 'supernatural']);
  });
});

// A descriptor may name two categories, and both are mechanically real (either
// one can make the item permitted or forbidden). The badge row therefore shows
// one `category-<id>` badge per category, in the descriptor's own order — so the
// FIRST badge names the category the Selected row's heading uses, which is what
// `houses.e2e.js` compares that heading against.
describe('VirtueFlawTab badges every category an item carries', () => {
  /** The `.badge.type` texts of the Virtues selection column, in document order. */
  function typeBadges(body: string): string[] {
    return [...virtueColumn(body).matchAll(/<span class="badge type">([\s\S]*?)<\/span>/g)].map(
      (m) => clean(m[1].replace(/<[^>]*>/g, '').trim()),
    );
  }

  it('renders one localized badge per category, primary first', () => {
    resetEntity([{ ref: 'virtue.sufi' }]);
    expect(typeBadges(html())).toEqual(['Social Status', 'Supernatural']);
  });

  it('renders no raw category slug', () => {
    resetEntity([{ ref: 'virtue.sufi' }]);
    expect(virtueColumn(html())).not.toContain('social_status');
  });

  it('still renders exactly one badge for a single-category item', () => {
    resetEntity([{ ref: 'virtue.heartbeast' }]);
    expect(typeBadges(html())).toEqual(['Hermetic']);
  });

  // A Selected row sits under ONE heading — the descriptor's first-listed
  // category, the same one its first badge names, which is the invariant
  // houses.e2e.js asserts.
  it('puts a dual-category row under the heading its first badge names', () => {
    resetEntity([{ ref: 'virtue.sufi' }]);
    expect(columnOutline(html())).toEqual(['# Social Status', 'Sufi']);
  });

  // The badge stack is absolutely positioned and vertically centered, so nothing
  // in the row's own flow makes space for it — `app.css` does, via a min-height
  // sized to the stack. A third badge needs the taller of the two rules, and the
  // only thing linking the markup to it is this class name. Assert both ends:
  // the component emits it exactly on the rows that need it, and the rule that
  // gives it meaning still exists. Otherwise a stack taller than its row bleeds
  // over the border into the neighbouring rows, and no other test would notice.
  it('marks a three-badge row so the CSS can grow it, and no other row', () => {
    resetEntity([{ ref: 'virtue.sufi' }, { ref: 'virtue.heartbeast' }]);
    const nameWraps = [
      ...virtueColumn(html()).matchAll(/<span class="([^"]*name-wrap[^"]*)"/g),
    ].map((m) => m[1]);
    expect(nameWraps).toHaveLength(2);
    // Sufi (two categories -> three badges) is marked; Heartbeast (one) is not.
    expect(nameWraps.filter((c) => c.includes('tall-badges'))).toHaveLength(1);
  });

  it('sizes the marked row to contain three badges, not two', () => {
    const twoTall = /\.selection-list \.name-wrap \{\s*min-height:\s*([\d.]+)rem/.exec(appCss);
    const threeTall =
      /\.selection-list \.name-wrap\.tall-badges \{\s*min-height:\s*([\d.]+)rem/.exec(appCss);
    expect(twoTall, 'the two-badge min-height rule is gone').not.toBeNull();
    expect(threeTall, 'the three-badge min-height rule is gone').not.toBeNull();
    // One more badge plus its gap, so strictly taller than the two-badge box.
    expect(Number(threeTall![1])).toBeGreaterThan(Number(twoTall![1]));
  });
});

// Full-audit fix round: the category/magnitude filter `<select>`s carried no
// accessible name at all — the same defect as S6 (AbilityTab's ability-category
// filter) and S9 (EquipmentTab's equipment-group filter), just not caught in the
// original a11y sweep because this tab has two sides. Each select now gets a
// Fluent-sourced `aria-label` naming both its side (Virtues/Flaws) and what it
// filters by, so a screen-reader user tabbing between the four selects (two
// sides × two filters) can tell them apart.
describe('VirtueFlawTab filter selects', () => {
  it('gives the Virtues category filter an accessible name naming its side', () => {
    const select = /<select[^>]*data-testid="vf-category-filter-virtue"[^>]*>/.exec(html());
    expect(select).not.toBeNull();
    expect(clean(select![0])).toContain('aria-label="Filter Virtues by category"');
  });

  it('gives the Flaws category filter an accessible name naming its side', () => {
    const select = /<select[^>]*data-testid="vf-category-filter-flaw"[^>]*>/.exec(html());
    expect(select).not.toBeNull();
    expect(clean(select![0])).toContain('aria-label="Filter Flaws by category"');
  });

  it('gives the Virtues magnitude filter an accessible name naming its side', () => {
    const select = /<select[^>]*data-testid="vf-magnitude-filter-virtue"[^>]*>/.exec(html());
    expect(select).not.toBeNull();
    expect(clean(select![0])).toContain('aria-label="Filter Virtues by magnitude"');
  });

  it('gives the Flaws magnitude filter an accessible name naming its side', () => {
    const select = /<select[^>]*data-testid="vf-magnitude-filter-flaw"[^>]*>/.exec(html());
    expect(select).not.toBeNull();
    expect(clean(select![0])).toContain('aria-label="Filter Flaws by magnitude"');
  });

  it('gives all four filter selects distinct accessible names', () => {
    const body = html();
    const labels = ['virtue', 'flaw'].flatMap((side) =>
      ['category', 'magnitude'].map((kind) => {
        const select = new RegExp(`<select[^>]*data-testid="vf-${kind}-filter-${side}"[^>]*>`).exec(
          body,
        );
        return /aria-label="([^"]+)"/.exec(select![0])![1];
      }),
    );
    expect(new Set(labels).size).toBe(labels.length);
  });

  // S23 (full-audit a11y): the free-text search box on each side carries only a
  // placeholder, which is not an accessible name — same defect as S5/S8.
  it('gives the Virtues search box an accessible name via Fluent', () => {
    const input = /<input[^>]*data-testid="vf-search-virtue"[^>]*>/.exec(html());
    expect(input).not.toBeNull();
    expect(clean(input![0])).toContain('aria-label="Search…"');
  });

  it('gives the Flaws search box an accessible name via Fluent', () => {
    const input = /<input[^>]*data-testid="vf-search-flaw"[^>]*>/.exec(html());
    expect(input).not.toBeNull();
    expect(clean(input![0])).toContain('aria-label="Search…"');
  });
});

// max_total slice: an item's bought+granted copies must not exceed its
// `max_total` ceiling (Puissant Art, capped at two total across every Art
// target) — the UI half of the engine's `too_many_selections` validator
// (`crates/arm-rules/src/validation/selections.rs`). This describe block
// installs its own small ruleset so the shared ITEMS fixture (and the exact
// category-list assertions above) stay untouched.
describe('VirtueFlawTab enforces max_total on the Available list', () => {
  const CAPPED: PointItem = item({
    id: 'virtue.puissant_art',
    categories: ['hermetic'],
    parameters: [{ key: 'art', type: 'ref', domain: 'art' }],
    max_total: 2,
  });

  function installCappedRuleset(): void {
    store.ruleset = {
      ruleset: {
        id: 'test',
        version: '1',
        point_items: { 'virtue.puissant_art': CAPPED },
        type_profiles: {
          magus: {
            id: 'magus',
            budget: { virtue_points: 10, flaw_points: 10 },
            is_magus: true,
            gift_policy: 'required',
            creation_phases: [],
          },
        },
        abilities: {},
        magnitude_points: { free: 0, minor: 1, major: 3 },
        ability_category_order: ['general'],
        art_type_order: ['technique', 'form'],
      },
      i18n: { 'virtue.puissant_art': { name: 'Puissant {art}' } },
    } as unknown as LocalizedRuleset;
  }

  beforeEach(() => {
    installCappedRuleset();
  });

  /** The Available list's `add-virtue.puissant_art` button's opening tag. */
  function addRow(body: string): string {
    return /<button[^>]*data-testid="add-virtue\.puissant_art"[^>]*>/.exec(body)![0];
  }

  it('leaves the row enabled below max_total', () => {
    resetEntity([{ ref: 'virtue.puissant_art', params: { art: 'art.ignem' } }]);
    expect(addRow(html())).toContain('aria-disabled="false"');
  });

  it('greys out the row once bought copies alone reach max_total', () => {
    resetEntity([
      { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } },
      { ref: 'virtue.puissant_art', params: { art: 'art.perdo' } },
    ]);
    expect(addRow(html())).toContain('aria-disabled="true"');
  });

  it('counts a granted copy toward the same cap as a bought one', () => {
    resetEntity([{ ref: 'virtue.puissant_art', params: { art: 'art.ignem' } }]);
    grant({ ref: 'virtue.puissant_art', params: { art: 'art.perdo' } });
    expect(addRow(html())).toContain('aria-disabled="true"');
  });

  // The once-only leg beside it already ignores ValidationMode, and a
  // copy-count cap is the same class of rule — only the incompatibility leg
  // stays mode-aware.
  it('stays disabled in advisory mode, unlike the mode-aware incompatibility leg', () => {
    resetEntity([
      { ref: 'virtue.puissant_art', params: { art: 'art.ignem' } },
      { ref: 'virtue.puissant_art', params: { art: 'art.perdo' } },
    ]);
    store.mode = 'advisory';
    try {
      expect(addRow(html())).toContain('aria-disabled="true"');
    } finally {
      store.mode = 'enforced';
    }
  });

  it('leaves an item with zero copies enabled (never over-filters)', () => {
    resetEntity([]);
    expect(addRow(html())).toContain('aria-disabled="false"');
  });
});
