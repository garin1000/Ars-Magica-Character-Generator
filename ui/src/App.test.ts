import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { Entity, LocalizedRuleset } from './lib/types';

// While a native file dialog is open the app must be inert behind a blocking
// overlay: rfd dialogs are not input-modal on Linux, so without this the user can
// keep editing the character behind the dialog. Everything the app root reaches
// goes over the Tauri IPC bridge, so mock it away; harness mirrors
// lib/components/SaveLoadBar.test.ts.
vi.mock('./lib/ipc', () => ({
  loadRuleset: vi.fn(),
  validateEntity: vi.fn().mockResolvedValue({ issues: [] }),
  effectiveScores: vi.fn().mockResolvedValue({}),
  derivedTotals: vi.fn().mockResolvedValue({}),
  saveEntity: vi.fn(),
  loadEntity: vi.fn(),
  updateCloseGuard: vi.fn(),
  exportMarkdown: vi.fn(),
  exportLabelKeys: vi.fn(),
}));

import * as ipc from './lib/ipc';
import { SCHEMA_VERSION, store } from './lib/state.svelte';
import App from './App.svelte';

/** A minimal localized ruleset: the shell needs no catalogue, only the bundle. */
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
    art_scores: [],
    personality_traits: [],
    reputations: [],
  };
}

/** Render the app root to an HTML string (node env, no DOM). */
function html(): string {
  return render(App).body;
}

/** The opening tag of the single element carrying `testid`, or null if absent. */
function openTag(body: string, testid: string): string | null {
  return new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body)?.[0] ?? null;
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(ipc.saveEntity).mockReset();
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

describe('App dialog modality', () => {
  it('leaves the app interactive while no file operation is running', () => {
    const body = html();

    expect(openTag(body, 'busy-overlay')).toBeNull();
    const shell = openTag(body, 'app-shell')!;
    expect(shell).not.toMatch(/\binert\b/);
    expect(shell).toMatch(/aria-busy="false"/);
  });

  it('blocks the whole app behind an overlay while a dialog-backed save is open', async () => {
    // A save that never settles models an open native dialog.
    let finishSave: (path: string | null) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((resolve) => {
        finishSave = resolve;
      }),
    );

    const saving = store.save();
    const busyBody = html();
    expect(openTag(busyBody, 'busy-overlay')).not.toBeNull();
    const busyShell = openTag(busyBody, 'app-shell')!;
    expect(busyShell).toMatch(/\binert\b/);
    expect(busyShell).toMatch(/aria-busy="true"/);

    finishSave('/tmp/marcus.armc');
    await saving;

    const idleBody = html();
    expect(openTag(idleBody, 'busy-overlay')).toBeNull();
    expect(openTag(idleBody, 'app-shell')!).not.toMatch(/\binert\b/);
  });

  it('lifts the overlay when the dialog-backed call fails', async () => {
    let failSave: (reason: unknown) => void = () => {};
    vi.mocked(ipc.saveEntity).mockReturnValue(
      new Promise<string | null>((_resolve, reject) => {
        failSave = reject;
      }),
    );

    const saving = store.saveAs();
    expect(openTag(html(), 'busy-overlay')).not.toBeNull();

    failSave({ kind: 'io' });
    await saving;

    expect(openTag(html(), 'busy-overlay')).toBeNull();
    expect(openTag(html(), 'app-shell')!).not.toMatch(/\binert\b/);
  });
});
