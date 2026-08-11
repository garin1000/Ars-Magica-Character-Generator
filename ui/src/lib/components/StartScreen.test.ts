import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, EntityTypeProfile, LocalizedRuleset } from '../types';

// The start screen reads the shared store singleton (ruleset, loading/busy flags,
// error banner) and the Fluent bundle; both of its actions delegate to store
// actions that go over the Tauri IPC bridge, so mock the bridge away. Harness
// mirrors SaveLoadBar.test.ts.
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

import * as ipc from '../ipc';
import { SCHEMA_VERSION, store } from '../state.svelte';
import StartScreen from './StartScreen.svelte';

/** A profile skeleton: only its id matters here — it names the `type-<id>` key. */
function profile(id: string): EntityTypeProfile {
  return {
    id,
    budget: { virtue_points: 10, flaw_points: 10 },
    permitted_categories: [],
    forbidden_categories: [],
    creation_phases: [],
  };
}

/**
 * Install a localized ruleset carrying exactly these type profiles. The screen
 * needs no catalogue beyond them — the create buttons ARE the profile set.
 */
function installProfiles(...ids: string[]): void {
  const type_profiles: Record<string, EntityTypeProfile> = {};
  for (const id of ids) type_profiles[id] = profile(id);
  store.ruleset = {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles,
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: ['general'],
      art_type_order: ['technique', 'form'],
    },
    i18n: {},
  } as unknown as LocalizedRuleset;
}

function resetEntity(): void {
  store.entity = {
    schema_version: SCHEMA_VERSION,
    ruleset: { id: 'test', version: '1' },
    entity_kind: 'character',
    type_id: '',
    selections: [],
    characteristics: {} as Entity['characteristics'],
    characteristic_descriptions: {},
    ability_scores: [],
    xp_pool: 0,
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

/** Render the start screen to an HTML string (node env, no DOM). */
function html(): string {
  return render(StartScreen).body;
}

/** The opening tag and the text of the single element carrying `testid`. */
function element(body: string, testid: string): { open: string; text: string } {
  const open = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!open) throw new Error(`no element with data-testid="${testid}"`);
  const whole = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i').exec(body)!;
  return { open: open[0], text: whole[1].replace(/<[^>]*>/g, '').trim() };
}

/** Every `start-create-*` test id present in the markup, in document order. */
function createTestIds(body: string): string[] {
  return [...body.matchAll(/data-testid="(start-create-[^"]+)"/g)].map((m) => m[1]);
}

/** Every `start-wizard-*` test id present in the markup, in document order. */
function wizardTestIds(body: string): string[] {
  return [...body.matchAll(/data-testid="(start-wizard-[^"]+)"/g)].map((m) => m[1]);
}

beforeEach(() => {
  vi.useFakeTimers();
  store.lang = 'en';
  store.error = null;
  store.loading = false;
  store.currentPath = null;
  store.view = 'start';
  installProfiles('grog', 'companion', 'mythic_companion', 'magus');
  resetEntity();
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

describe('StartScreen', () => {
  it('renders the start screen with a heading', () => {
    const body = html();
    expect(element(body, 'start-screen').open).not.toBeNull();
    expect(body).toMatch(/<h1|<h2/i);
  });

  it('offers one create button per type profile in the ruleset', () => {
    installProfiles('grog', 'magus');
    expect(createTestIds(html())).toEqual(['start-create-grog', 'start-create-magus']);
  });

  it('follows the ruleset rather than a hardcoded set of types', () => {
    installProfiles('custos');
    expect(createTestIds(html())).toEqual(['start-create-custos']);
  });

  it('labels each create button through its Fluent key, never the raw slug', () => {
    const body = html();
    const label = element(body, 'start-create-mythic_companion').text;
    expect(label).toBe('Mythic Companion');
    expect(label).not.toBe('mythic_companion');
    expect(label).not.toBe('type-mythic_companion');
  });

  it('localizes the create labels to German', () => {
    store.lang = 'de';
    expect(element(html(), 'start-create-companion').text).toBe('Gefährte');
  });

  it('offers opening an existing character with the shared Open label', () => {
    const { open, text } = element(html(), 'start-open');
    expect(open).toMatch(/<button/i);
    expect(text).toBe('Open');
  });

  // The wizard is entered per character type, exactly like direct creation: the
  // type is fixed at creation, so it must be chosen before the flow starts.
  it('offers one guided-wizard button per type profile in the ruleset', () => {
    installProfiles('grog', 'magus');
    expect(wizardTestIds(html())).toEqual(['start-wizard-grog', 'start-wizard-magus']);
  });

  it('no longer shows a single, permanently disabled wizard entry', () => {
    const body = html();
    expect(body).not.toContain('data-testid="start-wizard"');
    expect(element(body, 'start-wizard-magus').open).not.toMatch(/disabled/);
  });

  it('labels each wizard button through its type Fluent key, never the raw slug', () => {
    const label = element(html(), 'start-wizard-mythic_companion').text;
    expect(label).toBe('Mythic Companion');
    expect(label).not.toBe('mythic_companion');
    expect(label).not.toBe('type-mythic_companion');
  });

  it('explains the guided entry in real prose, not an echoed key', () => {
    const body = html();
    expect(body).toContain(store.t('start-wizard-hint'));
    expect(store.t('start-wizard-hint')).not.toBe('start-wizard-hint');
  });

  it('offers nothing to start while the ruleset has no profiles', () => {
    installProfiles();
    const body = html();
    expect(createTestIds(body)).toEqual([]);
    expect(wizardTestIds(body)).toEqual([]);
  });

  it('disables every action while the ruleset is loading', () => {
    store.loading = true;
    const body = html();
    expect(element(body, 'start-open').open).toMatch(/disabled/);
    expect(element(body, 'start-create-magus').open).toMatch(/disabled/);
  });

  it('disables every action while a native file dialog is open', async () => {
    // A write that never settles models an open native dialog (the same in-flight
    // flag every file operation raises).
    let finishSave: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        finishSave = resolve;
      }),
    );

    const saving = store.saveAs();
    expect(store.busy).toBe(true);
    const body = html();
    expect(element(body, 'start-open').open).toMatch(/disabled/);
    expect(element(body, 'start-create-magus').open).toMatch(/disabled/);

    finishSave(null);
    await saving;
    expect(element(html(), 'start-open').open).not.toMatch(/disabled/);
  });

  // Without this the only error surface is SaveLoadBar's, which the start screen
  // does not render: a failed ruleset load would leave the user with no create
  // buttons and no explanation.
  it('surfaces a failed ruleset load through the shared error keys', () => {
    store.ruleset = null;
    store.error = { kind: 'ruleset', ruleset_kind: 'parse', errors: ['bad json'] };

    const { open, text } = element(html(), 'start-error');
    expect(open).toMatch(/role="alert"/);
    expect(text).toBe(store.t('error-ruleset'));
    expect(text).not.toBe('error-ruleset');
  });

  it('renders no error banner when nothing failed', () => {
    expect(html()).not.toContain('data-testid="start-error"');
  });
});
