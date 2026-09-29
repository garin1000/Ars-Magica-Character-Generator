import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { Entity, LocalizedRuleset } from '../types';

// G17 (full-audit backlog, Tier 3-4): the SSR tests in EquipmentTab.test.ts only
// cover narrow accessible-name/separator regressions — nothing exercised adding an
// item from the SourcePicker, removing one, or changing its loadout state, i.e. the
// tab's actual reason for existing. Those are event-listener paths (`onclick`/
// `onchange`), which SSR never runs (CLAUDE.md: "event listeners" require a client
// test), so this file mounts the real component and clicks the real controls,
// mirroring SpellTab.client.test.ts's `addButton.click()` pattern.
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
/** Not yet carried — the row the "add from the source picker" test clicks. */
const AXE = 'weapon.axe';

function installRuleset(): void {
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: {},
      abilities: {},
      weapons: {
        [SWORD]: { id: SWORD, ability: 'ability.single_weapon' },
        [AXE]: { id: AXE, ability: 'ability.single_weapon' },
      },
      shields: {},
      armor: {},
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {
      [SWORD]: { name: 'Long Sword' },
      [AXE]: { name: 'Axe' },
    },
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.view = 'editor';
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
    saga_year: 1220,
    art_scores: [],
    personality_traits: [],
    reputations: [],
    spells: [],
    equipment: [{ item: SWORD, loadout: 'stowed' }],
  };
  store.effective = null;
  store.result = { issues: [] };
}

let target: HTMLElement;
let app: Record<string, unknown> | undefined;

beforeEach(() => {
  store.lang = 'en';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  if (app) unmount(app);
  app = undefined;
  target?.remove();
});

describe('EquipmentTab real interaction paths (G17)', () => {
  it('adds an item from the source picker', () => {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(EquipmentTab, { target });
    flushSync();

    const addButton = target.querySelector(`[data-testid="add-${AXE}"]`) as HTMLButtonElement;
    expect(addButton).toBeTruthy();
    addButton.click();
    flushSync();

    expect(store.entity.equipment).toEqual([
      { item: SWORD, loadout: 'stowed' },
      { item: AXE, loadout: 'wielded' },
    ]);
    // The addition is reflected in the rendered selected list, not just the store.
    const names = [...target.querySelectorAll('[data-testid^="equipment-name-"]')].map(
      (el) => el.textContent,
    );
    expect(names).toContain('Axe');
  });

  it('removes an item via the remove button', () => {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(EquipmentTab, { target });
    flushSync();

    const removeButton = target.querySelector(
      '[data-testid="equipment-remove-0"]',
    ) as HTMLButtonElement;
    expect(removeButton).toBeTruthy();
    removeButton.click();
    flushSync();

    expect(store.entity.equipment).toEqual([]);
    expect(target.querySelector('[data-testid="equipment-name-0"]')).toBeNull();
  });

  // K5 (design-f0-book-template-engine.md § 2a): the boolean Equipped checkbox
  // is replaced by a tri-state `<select>` — stowed / carried / wielded.
  it('changes the loadout via the tri-state control', () => {
    store.entity.equipment = [{ item: SWORD, loadout: 'stowed' }];
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(EquipmentTab, { target });
    flushSync();

    const select = target.querySelector('[data-testid="equipment-loadout-0"]') as HTMLSelectElement;
    expect(select).toBeTruthy();

    select.value = 'carried';
    select.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();

    expect(store.entity.equipment).toEqual([{ item: SWORD, loadout: 'carried' }]);
  });

  // K3 (design-f0-book-template-engine.md § 2b): toggling "mounted" calls
  // store.setMounted, which does not exist yet (phase 2) — RED CHECKPOINT.
  it('toggles mounted via store.setMounted', () => {
    target = document.createElement('div');
    document.body.appendChild(target);
    app = mount(EquipmentTab, { target });
    flushSync();

    const checkbox = target.querySelector('[data-testid="mounted-toggle"]') as HTMLInputElement;
    expect(checkbox).toBeTruthy();
    expect(checkbox.checked).toBe(false);

    checkbox.checked = true;
    checkbox.dispatchEvent(new Event('change', { bubbles: true }));
    flushSync();

    expect(store.entity.mounted).toBe(true);
  });
});
