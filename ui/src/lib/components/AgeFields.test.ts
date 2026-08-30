import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity } from '../types';

// The fields read the shared store singleton (the entity's age, the engine's age
// cap) and the Fluent bundle. The store schedules a debounced revalidate over the
// Tauri IPC bridge; mock the bridge so nothing reaches a backend. Harness mirrors
// LifeStagePanel.test.ts.
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
  agingPreview: vi.fn(),
  agingApply: vi.fn(),
  agingRevert: vi.fn(),
}));

import { SCHEMA_VERSION, store } from '../state.svelte';
import AgeFields from './AgeFields.svelte';

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
  };
  store.effective = null;
}

/** The engine's age→max-Ability-score cap, the fields' read-only echo. */
function setAgeCap(cap: number | null): void {
  store.effective = {
    ...(store.effective ?? {}),
    age_ability_cap: cap,
  } as unknown as EffectiveScores;
}

/** Render the fields to an HTML string (node env, no DOM). */
function html(props: { readonly?: boolean } = {}): string {
  return render(AgeFields, { props }).body;
}

/** The single element carrying a given data-testid, with its class attribute. */
function element(body: string, testid: string): { open: string; text: string } {
  const re = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i');
  const match = re.exec(body);
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  const openMatch = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  return { open: openMatch![0], text: match[1].replace(/<[^>]*>/g, '').trim() };
}

/** Whether any element carries the exact data-testid. */
function has(body: string, testid: string): boolean {
  return new RegExp(`data-testid="${testid}"`).test(body);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  resetEntity();
});

describe('AgeFields (slice 6b6b)', () => {
  it('offers the age', () => {
    store.entity.age = 40;
    setAgeCap(5);
    const body = html();
    // The age is what the aging schedule and the life-stage pricing both hang on,
    // and since Slice 12 this is the one editable field for it in either flow — the
    // Details tab and the wizard's Concept step keep the same testid verbatim.
    const age = element(body, 'age-input');
    expect(age.open).toMatch(/type="number"/);
    expect(age.open).toMatch(/value="40"/);
    expect(body).toContain('Age');
    // Nothing renders a slug: the label comes from Fluent.
    expect(body).not.toMatch(/>\s*age_ability_cap\s*</);
  });

  it('leaves the field empty for a character with no age', () => {
    setAgeCap(null);
    const body = html();
    expect(element(body, 'age-input').open).toMatch(/value=""/);
  });
});

describe('AgeFields read-only mode (slice 12, #24)', () => {
  it('renders the age as a read-out rather than an input when the prop is set', () => {
    store.entity.age = 40;
    const body = html({ readonly: true });
    // The Experience step needs to SHOW the age its Gauntlet age is measured
    // against without becoming a second place to change it.
    expect(has(body, 'age-input')).toBe(false);
    const readout = element(body, 'age-readout');
    expect(readout.open).not.toMatch(/<input/i);
    expect(readout.text).toContain('40');
    // Still labelled from Fluent, never a bare number.
    expect(body).toContain('Age');
  });

  it('offers the editable input by default', () => {
    store.entity.age = 40;
    const body = html();
    expect(has(body, 'age-input')).toBe(true);
    expect(has(body, 'age-readout')).toBe(false);
  });

  it('takes the mode from the prop alone, never from the flow in the store', () => {
    // Cross-cutting theme 3, and the seam #19 established: a per-mount difference is
    // declared by whoever mounts the component, not sniffed from `store.view` or the
    // funding mode. Same store, both modes, opposite markup.
    store.view = 'wizard';
    store.entity.ability_funding = 'life_stages';
    expect(has(html({ readonly: true }), 'age-input')).toBe(false);
    expect(has(html({ readonly: false }), 'age-input')).toBe(true);
  });

  it('renders the age cap note in neither mode', () => {
    // Slice 12 gives the cap note one home, beside the Ability lists it constrains.
    setAgeCap(5);
    expect(has(html(), 'age-cap-note')).toBe(false);
    expect(has(html({ readonly: true }), 'age-cap-note')).toBe(false);
  });
});
