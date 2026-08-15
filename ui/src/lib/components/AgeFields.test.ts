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
function html(): string {
  return render(AgeFields, { props: {} }).body;
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
  it("offers the age and echoes the engine's Ability cap", () => {
    store.entity.age = 40;
    setAgeCap(5);
    const body = html();
    // The age is the aging schedule's only input, so the guided flow needs this
    // field wherever it mounts — and the Details tab keeps its testid verbatim.
    const age = element(body, 'age-input');
    expect(age.open).toMatch(/type="number"/);
    expect(age.open).toMatch(/value="40"/);
    expect(body).toContain('Age');
    // The cap is the engine's, echoed read-only through the shared Fluent key.
    const cap = element(body, 'age-cap-note');
    expect(cap.text).toContain('Max Ability score');
    expect(cap.text).toContain('5');
    // Nothing renders a slug: the label comes from Fluent.
    expect(body).not.toMatch(/>\s*age_ability_cap\s*</);
  });

  it('leaves the field empty for a character with no age', () => {
    setAgeCap(null);
    const body = html();
    expect(element(body, 'age-input').open).toMatch(/value=""/);
    // No cap, no read-out — an empty echo would be a dead control.
    expect(has(body, 'age-cap-note')).toBe(false);
  });
});
