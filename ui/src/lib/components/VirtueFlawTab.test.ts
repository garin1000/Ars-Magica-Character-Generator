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
  // Status, Supernatural", so `social_status` is its primary and `supernatural`
  // its secondary.
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

// A descriptor may name two categories, and both are mechanically real (either
// one can make the item permitted or forbidden). The badge row therefore shows
// one `category-<id>` badge per category, in the descriptor's own order so the
// FIRST badge is the primary — which is also what the group heading above the row
// says, and what `houses.e2e.js` compares that heading against.
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

  // The heading a dual-category row sits under is its PRIMARY category — the same
  // one its first badge names, which is the invariant houses.e2e.js asserts.
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
