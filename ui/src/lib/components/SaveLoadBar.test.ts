import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, EntityTypeProfile, LocalizedRuleset } from '../types';

// The toolbar reads the shared store singleton (busy flag, error banner) and the
// Fluent bundle, and every button delegates to a store action that goes over the
// Tauri IPC bridge; mock the bridge so nothing reaches a backend. Harness mirrors
// XpBar.test.ts.
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
import SaveLoadBar from './SaveLoadBar.svelte';

/** A minimal localized ruleset: the toolbar needs no catalogue, only the bundle. */
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
  };
}

/** Give the installed ruleset a profile for the current character's type. */
function installProfileForCurrentType(): void {
  const profile: EntityTypeProfile = {
    id: store.entity.type_id,
    budget: { virtue_points: 10, flaw_points: 10 },
    permitted_categories: [],
    forbidden_categories: [],
    creation_phases: ['concept'],
  };
  store.ruleset!.ruleset.type_profiles[store.entity.type_id] = profile;
}

/** Render the toolbar to an HTML string (node env, no DOM). */
function html(): string {
  return render(SaveLoadBar).body;
}

/** The opening tag and the text of the single element carrying `testid`. */
function element(body: string, testid: string): { open: string; text: string } {
  const open = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body);
  if (!open) throw new Error(`no element with data-testid="${testid}"`);
  const whole = new RegExp(`<[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</`, 'i').exec(body)!;
  return { open: open[0], text: whole[1].replace(/<[^>]*>/g, '').trim() };
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(ipc.saveEntity).mockReset();
  vi.mocked(ipc.exportMarkdown).mockReset();
  vi.mocked(ipc.exportLabelKeys).mockReset();
  vi.mocked(ipc.exportMarkdown).mockResolvedValue('/tmp/marcus.md');
  vi.mocked(ipc.exportLabelKeys).mockResolvedValue([]);
  store.lang = 'en';
  store.error = null;
  store.currentPath = null;
  installRuleset();
  resetEntity();
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

describe('SaveLoadBar Export action', () => {
  it('renders an Export button with its localized label', () => {
    const { open, text } = element(html(), 'export-button');
    expect(open).toMatch(/<button/i);
    expect(open).toMatch(/type="button"/);
    expect(text).toBe('Export');
  });

  it('localizes the Export label to German', () => {
    store.lang = 'de';
    expect(element(html(), 'export-button').text).toBe('Exportieren');
    store.lang = 'en';
  });

  it('is enabled while no file operation is running', () => {
    expect(element(html(), 'export-button').open).not.toMatch(/disabled/);
  });

  it('is disabled while a file operation is in flight, like the other actions', async () => {
    // A save that never settles models an open native dialog.
    let finishSave: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        finishSave = resolve;
      }),
    );

    const saving = store.save();
    expect(store.busy).toBe(true);
    expect(element(html(), 'export-button').open).toMatch(/disabled/);

    finishSave('/tmp/marcus.armc');
    await saving;
    expect(element(html(), 'export-button').open).not.toMatch(/disabled/);
  });

  // A server-rendered string carries no event handlers, so the click itself is
  // covered by e2e/specs/export-markdown.e2e.js against the shipped binary. What
  // this pins is the other half: the action the button names reaches the export
  // command and writes nothing through the save path.
  it('delegates to a store action that exports without saving', async () => {
    await store.exportMarkdown();

    expect(ipc.exportMarkdown).toHaveBeenCalledTimes(1);
    expect(ipc.saveEntity).not.toHaveBeenCalled();
  });
});

// Slice 5 (#31): the character on screen can be walked through the guided flow,
// not only a brand-new one. The action is OFFERED conditionally — a save from
// another ruleset may name a type this build has no profile for, and a wizard with
// no rail is not a screen to enter.
describe('SaveLoadBar guided-creation action', () => {
  beforeEach(() => {
    store.view = 'editor';
  });

  it('offers continuing the loaded character in the guided flow', () => {
    installProfileForCurrentType();
    const { open, text } = element(html(), 'wizard-continue-button');
    expect(open).toMatch(/<button/i);
    expect(text).toBe(store.t('action-continue-in-wizard'));
    expect(text).not.toBe('action-continue-in-wizard');
  });

  it('localizes the label to German', () => {
    installProfileForCurrentType();
    store.lang = 'de';
    expect(element(html(), 'wizard-continue-button').text).toBe(
      store.t('action-continue-in-wizard'),
    );
    store.lang = 'en';
  });

  it('is not offered for a type_id the loaded ruleset has no profile for', () => {
    expect(store.ruleset!.ruleset.type_profiles[store.entity.type_id]).toBeUndefined();
    expect(html()).not.toContain('data-testid="wizard-continue-button"');
  });

  it('is not offered while the wizard is already on screen', () => {
    installProfileForCurrentType();
    store.view = 'wizard';
    expect(html()).not.toContain('data-testid="wizard-continue-button"');
  });
});
