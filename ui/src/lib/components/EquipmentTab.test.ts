import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from '../types';

// The tab reads the shared store singleton (equipment catalogues, entity slots,
// filter state) and the Fluent bundle. The store's mutators go over the Tauri IPC
// bridge; mock it so nothing reaches a backend. Harness mirrors SpellTab.test.ts.
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
import EquipmentTab from './EquipmentTab.svelte';

const SWORD = 'weapon.long_sword';
/** An id in none of the three catalogues — the header-less trailing bucket. */
const UNKNOWN = 'weapon.nonesuch';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      weapons: { [SWORD]: { id: SWORD, ability: 'ability.single_weapon' } },
      shields: {},
      armor: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: { [SWORD]: { name: 'Long Sword' } },
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
    ability_scores: [],
    xp_pool: 0,
    ability_funding: 'pool',
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
    equipment: [{ item: SWORD }, { item: UNKNOWN }],
  };
  store.effective = null;
  store.result = { issues: [] };
}

function html(): string {
  return render(EquipmentTab, { props: {} }).body;
}

/** The selected side's markup, where the grouped `<ul>`s live. */
function selectedRegion(body: string): string {
  const start = body.indexOf('class="region region-selected"');
  if (start < 0) throw new Error('no selected region');
  return body.slice(start);
}

/**
 * `app.css` as text. CSS is not observable through `render` from `svelte/server`
 * (no stylesheet is attached), and NOT importable as `./app.css?raw` either:
 * vitest stubs CSS modules by extension regardless of the query, so `?raw` would
 * yield `''` and every assertion would pass vacuously.
 */
const appCss = readFileSync(fileURLToPath(new URL('../../app.css', import.meta.url)), 'utf-8');

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

// guided-creation-review-2026-08 #8: `SelectionList.svelte` opens a NEW `<ul>` per
// group, and the row separator was a `border-bottom` suppressed on `:last-child` —
// a selector scoped per PARENT, so the suppression fired once per group. A headed
// group hides that (the next `<h3 class="category">` draws its own boundary), but
// an item in none of the three catalogues lands in a HEADER-LESS trailing group,
// which butted straight against the group above it with no divider at all.
// S9 (full-audit a11y): the equipment-kind filter `<select>` had no accessible
// name, same shape as AbilityTab's category filter (S6) — no visible label to
// associate with, so a Fluent-sourced `aria-label` is the fix.
describe('EquipmentTab kind filter (S9)', () => {
  it('gives the kind filter select an accessible name via Fluent', () => {
    const body = html();
    const select = /<select[^>]*data-testid="equipment-group-filter"[^>]*>/.exec(body);
    expect(select).not.toBeNull();
    expect(select![0]).toContain('aria-label="Filter by equipment type"');
  });
});

describe('EquipmentTab separates a header-less group from the one above it (#8)', () => {
  it('emits the header-less group as a bare sibling list, with no heading to divide it', () => {
    const region = selectedRegion(html());
    const lists = [...region.matchAll(/<ul\b[^>]*>/g)];
    expect(lists).toHaveLength(2);
    const between = region.slice(
      region.indexOf('</ul>'),
      region.indexOf('<ul', region.indexOf('</ul>')),
    );
    expect(between).not.toContain('<h3');
    expect(
      between
        .replace(/<!--[\s\S]*?-->/g, '')
        .replace(/<\/ul>/, '')
        .trim(),
    ).toBe('');
  });

  it('draws the separator at the list boundary rather than per-parent last-child', () => {
    expect(appCss).toMatch(/^\.equipment-list \+ \.equipment-list.*\{[^}]*border-top:/ms);
    expect(appCss).not.toMatch(/^\.equipment-list li:last-child/m);
  });
});
