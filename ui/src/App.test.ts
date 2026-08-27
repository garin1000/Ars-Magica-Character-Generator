import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
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
  applyChildhoodPackage: vi.fn(),
}));

import * as ipc from './lib/ipc';
import { SCHEMA_VERSION, store } from './lib/state.svelte';
import App from './App.svelte';

/**
 * A minimal localized ruleset: the shell needs no catalogue, only the bundle —
 * plus whatever character-type profiles a test names (the shell reads them for
 * the type label and the startup screen's create buttons).
 */
function installRuleset(...typeIds: string[]): void {
  const type_profiles: Record<string, unknown> = {};
  for (const id of typeIds) {
    type_profiles[id] = {
      id,
      budget: { virtue_points: 10, flaw_points: 10 },
      permitted_categories: [],
      forbidden_categories: [],
      creation_phases: [],
    };
  }
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

/**
 * Give the installed ruleset life-stage rules.
 *
 * This is the flag the funding surface itself keys off — `LifeStagePanel.svelte`
 * gates on `ruleset.life_stages`, never on the character type — so the editor's
 * Experience tab must read the same one. Without it the panel renders nothing,
 * which is exactly the empty tab #28's fix must not produce.
 */
function installLifeStageRules(): void {
  (store.ruleset!.ruleset as { life_stages?: unknown }).life_stages = {
    childhood: {
      years: 5,
      native_language_ability: 'ability.living_language',
      native_language_xp: 75,
      spread_xp: 45,
      spread_abilities: [],
    },
    later_life: { xp_per_year: 15 },
  };
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
  // The Supernatural tab appears once the engine reports an effective Might, so
  // the tab list is only deterministic with the engine read-outs cleared.
  store.effective = null;
  store.derived = null;
}

/** Render the app root to an HTML string (node env, no DOM). */
function html(): string {
  return render(App).body;
}

/** The opening tag of the single element carrying `testid`, or null if absent. */
function openTag(body: string, testid: string): string | null {
  return new RegExp(`<[^>]*data-testid="${testid}"[^>]*>`, 'i').exec(body)?.[0] ?? null;
}

/**
 * The text content of the single element carrying `testid`, tags stripped.
 * Closes on the element's own tag name so nested markup (a label span beside a
 * value span) is included rather than cutting the match short.
 */
function textOf(body: string, testid: string): string {
  const match = new RegExp(`<(\\w+)[^>]*data-testid="${testid}"[^>]*>([\\s\\S]*?)</\\1>`, 'i').exec(
    body,
  );
  if (!match) throw new Error(`no element with data-testid="${testid}"`);
  return match[2]
    .replace(/<[^>]*>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(ipc.saveEntity).mockReset();
  store.lang = 'en';
  store.error = null;
  store.currentPath = null;
  // The screen is now part of what App renders, so every test has to say which
  // one it is about. The dialog-modality tests below were written against the
  // editor, so pin that here and let the start-screen tests opt out.
  store.view = 'editor';
  installRuleset();
  resetEntity();
});

afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

// --- the two screens (M6a) ---------------------------------------------------

describe('App screens', () => {
  it('opens on the startup screen, with no character surfaces', () => {
    store.view = 'start';
    const body = html();

    expect(openTag(body, 'start-screen')).not.toBeNull();
    // No character exists yet, so nothing that edits or reports on one is shown.
    expect(openTag(body, 'identity-name')).toBeNull();
    expect(openTag(body, 'character-type')).toBeNull();
    expect(openTag(body, 'tab-details')).toBeNull();
    expect(openTag(body, 'no-issues')).toBeNull();
  });

  it('keeps only the language control in the header on the startup screen', () => {
    store.view = 'start';
    const body = html();

    expect(openTag(body, 'language-select')).not.toBeNull();
    // Validation mode and the document toolbar are meaningless with no document,
    // and the status would read "unsaved" for a document that does not exist.
    expect(openTag(body, 'mode-select')).toBeNull();
    expect(openTag(body, 'save-button')).toBeNull();
    expect(openTag(body, 'doc-status')).toBeNull();
  });

  it('renders the editor, and no startup screen, once a character exists', () => {
    const body = html();

    expect(openTag(body, 'start-screen')).toBeNull();
    expect(openTag(body, 'identity-name')).not.toBeNull();
    expect(openTag(body, 'tab-details')).not.toBeNull();
    expect(openTag(body, 'no-issues')).not.toBeNull();
    expect(openTag(body, 'doc-status')).not.toBeNull();
  });

  it('shows the character type as a read-only label, not a selector', () => {
    installRuleset('magus');
    store.entity.type_id = 'magus';
    const body = html();

    expect(textOf(body, 'character-type')).toContain('Magus');
    // The type is fixed at creation: there is no control that changes it.
    expect(openTag(body, 'type-select')).toBeNull();
  });

  it('localizes the type label through Fluent, never rendering the slug', () => {
    installRuleset('mythic_companion');
    store.entity.type_id = 'mythic_companion';

    const en = textOf(html(), 'character-type');
    expect(en).toContain('Mythic Companion');
    expect(en).not.toContain('mythic_companion');

    store.lang = 'de';
    const de = textOf(html(), 'character-type');
    expect(de).toContain('Mythischer Gefährte');
    expect(de).not.toContain('mythic_companion');
  });

  // `translate()` falls back to the key itself, so a save naming a type the loaded
  // ruleset has no profile for would otherwise print `type-<slug>` on screen.
  it('names an unknown character type through a Fluent key, never its id', () => {
    installRuleset('magus');
    store.entity.type_id = 'sorcerer';
    const body = html();

    const label = textOf(body, 'character-type');
    expect(label).toContain(store.t('type-unknown'));
    expect(label).not.toContain('sorcerer');
    expect(label).not.toContain('type-sorcerer');
  });
});

// --- the wizard as the third screen (M6b1b) ----------------------------------

describe('App and the guided wizard', () => {
  beforeEach(() => {
    installRuleset('magus');
    store.entity.type_id = 'magus';
    store.view = 'wizard';
    store.wizardStep = 0;
    store.wizardFurthest = 0;
  });

  afterEach(() => {
    store.view = 'editor';
  });

  it('shows the wizard instead of the tab bar', () => {
    const body = html();
    expect(openTag(body, 'wizard-rail')).not.toBeNull();
    expect(openTag(body, 'tab-details')).toBeNull();
    expect(body).not.toContain('role="tablist"');
  });

  // The wizard edits a real character, so the banner belongs above it — and the
  // document toolbar has to be reachable, because the close guard promises the
  // user can save rather than lose the work.
  it('keeps the banner and the document controls', () => {
    const body = html();
    expect(openTag(body, 'character-type')).not.toBeNull();
    expect(openTag(body, 'identity-name')).not.toBeNull();
    expect(openTag(body, 'save-button')).not.toBeNull();
    expect(openTag(body, 'mode-select')).not.toBeNull();
    expect(openTag(body, 'doc-status')).not.toBeNull();
  });

  // The wizard docks its own step-scoped panel; a second, unfiltered one below it
  // would undo the filtering.
  it('leaves the whole-character issues footer to the editor', () => {
    const wizard = html();
    store.view = 'editor';
    const editor = html();
    const count = (body: string) => [...body.matchAll(/class="validation-docked"/g)].length;
    expect(count(wizard)).toBe(1);
    expect(count(editor)).toBe(1);
  });
});

// --- the editor's tab list mirrors the wizard's phase list (Slice 3, #28) -----

/** Every tab id in the rendered tab strip, in the order the strip shows them. */
function tabIdsInOrder(body: string): string[] {
  return [...body.matchAll(/data-testid="tab-([^"]+)"/g)].map((match) => match[1]);
}

describe('editor tabs for the phases split out in Slice 3', () => {
  it('exposes an Experience tab when the ruleset ships life-stage rules', () => {
    installLifeStageRules();
    const tag = openTag(html(), 'tab-experience');

    expect(tag).not.toBeNull();
    expect(tag).toContain('id="tab-experience"');
    expect(tag).toContain('aria-controls="tabpanel-experience"');
  });

  it('offers no Experience tab for a ruleset shipping no life-stage rules', () => {
    // The default fixture has none. The panel self-gates to nothing there, and a
    // tab whose panel is empty is the failure mode this slice must not create —
    // so tab and content read the very same flag.
    expect(openTag(html(), 'tab-experience')).toBeNull();
  });

  it('exposes separate Personality & Reputations and Aging tabs', () => {
    const body = html();

    const personality = openTag(body, 'tab-personality_reputations');
    expect(personality).not.toBeNull();
    expect(personality).toContain('aria-controls="tabpanel-personality_reputations"');

    const aging = openTag(body, 'tab-aging');
    expect(aging).not.toBeNull();
    expect(aging).toContain('aria-controls="tabpanel-aging"');
  });

  it('labels every new tab through Fluent, never rendering the slug', () => {
    installLifeStageRules();
    const body = html();

    // The Fluent keys HYPHENATE where the phase slug and the tab id use
    // underscores, so neither spelling can be generated from the other.
    for (const [testid, key] of [
      ['tab-experience', 'tab-experience'],
      ['tab-personality_reputations', 'tab-personality-reputations'],
      ['tab-aging', 'tab-aging'],
    ] as const) {
      // A missing key makes `t()` return the key itself, which would let the
      // comparison below pass vacuously.
      expect(store.t(key)).not.toBe(key);
      // Svelte escapes the ampersand in "Personality & Reputations" on the way
      // out; the label is the Fluent string, escaping aside.
      expect(textOf(body, testid).replace(/&amp;/g, '&')).toBe(store.t(key));
    }
  });

  it('moves Personality, Reputations and the aging cluster off the Details tab', () => {
    // `details` is the default tab, so a server render reaches its panel body.
    const body = html();

    expect(openTag(body, 'personality-add')).toBeNull();
    expect(openTag(body, 'reputation-empty')).toBeNull();
    expect(openTag(body, 'aging-panel')).toBeNull();
    expect(openTag(body, 'longevity-add')).toBeNull();
  });

  it('keeps identity, age and the Warping/Twilight cluster on Details', () => {
    // Those three have no wizard phase of their own (or belong to `concept`), so
    // Details is still their home — the split must not carry them off with the
    // sections that do have one.
    const body = html();

    expect(openTag(body, 'identity-concept')).not.toBeNull();
    expect(openTag(body, 'age-input')).not.toBeNull();
    expect(openTag(body, 'warping-points-input')).not.toBeNull();
    expect(openTag(body, 'twilight-scars-list')).not.toBeNull();
  });

  it('names each tab and its panel from the same id, so no reference dangles', () => {
    installLifeStageRules();
    const body = html();

    const tabs = [...body.matchAll(/<button[^>]*role="tab"[^>]*>/g)].map((match) => match[0]);
    expect(tabs.length).toBeGreaterThan(0);
    for (const tag of tabs) {
      const id = /\bid="tab-([^"]+)"/.exec(tag)?.[1];
      expect(id).toBeDefined();
      expect(tag).toContain(`aria-controls="tabpanel-${id}"`);
    }

    // Only the active tab's panel is rendered, so that is the one reference which
    // must resolve in this markup: it does, and it points back at the active tab.
    const panels = [...body.matchAll(/role="tabpanel"[^>]*/g)].map((match) => match[0]);
    expect(panels.length).toBe(1);
    const active = tabs.find((tag) => tag.includes('aria-selected="true"'))!;
    const activeId = /\bid="tab-([^"]+)"/.exec(active)![1];
    expect(panels[0]).toContain(`id="tabpanel-${activeId}"`);
    expect(panels[0]).toContain(`aria-labelledby="tab-${activeId}"`);
  });
});

// The mapping #28 decided, as data. The phase list comes from the SHIPPED
// ruleset file — the same `creation_phases` the wizard's rail and the editor's
// gating read — rather than a list retyped here, so a phase added there without
// an editor counterpart fails this test instead of drifting silently. Read as
// text for the reason app.css.test.ts states: it is data, not a module.
const shippedTypeProfiles = JSON.parse(
  readFileSync(
    fileURLToPath(new URL('../../rules/core/character_types.json', import.meta.url)),
    'utf-8',
  ),
) as {
  id: string;
  is_magus?: boolean;
  has_mythic_type?: boolean;
  creation_phases: string[];
}[];

// Phase slug → editor tab id, or null for a phase with no tab. `review` has
// none: finishing the wizard lands in the editor, which IS the review surface.
const TAB_FOR_PHASE: Record<string, string | null> = {
  concept: 'details',
  characteristics: 'characteristics',
  virtues_flaws: 'virtues_flaws',
  experience: 'experience',
  abilities: 'abilities',
  arts: 'arts',
  spells: 'spells',
  house_specialisation: 'house_specialisation',
  mythic_type: 'mythic_type',
  personality_reputations: 'personality_reputations',
  aging: 'aging',
  review: null,
};

// The two phases whose tab position is NOT asserted. Both are type markers whose
// editor slot predates this slice (House sits with the other magus-only tabs,
// Type with the mythic-companion one) and no issue asks to move them; and the
// profiles disagree about where they belong anyway — the magus puts
// `house_specialisation` third, the mythic companion puts `mythic_type` second,
// so no single static strip can match both orders. Membership IS asserted for
// them; only the position is exempt.
const POSITION_EXEMPT = new Set(['house_specialisation', 'mythic_type']);

describe('the editor tab list mirrors the shipped creation phases (#28)', () => {
  /** Install one shipped profile, with life-stage rules, and render the editor. */
  function renderFor(profile: (typeof shippedTypeProfiles)[number]): string[] {
    installRuleset(profile.id);
    (store.ruleset!.ruleset.type_profiles as Record<string, unknown>)[profile.id] = profile;
    installLifeStageRules();
    resetEntity();
    store.entity.type_id = profile.id;
    return tabIdsInOrder(html());
  }

  it('knows a tab (or a deliberate absence) for every shipped phase', () => {
    for (const profile of shippedTypeProfiles) {
      for (const phase of profile.creation_phases) {
        expect(Object.keys(TAB_FOR_PHASE)).toContain(phase);
      }
    }
  });

  it.each(shippedTypeProfiles.map((p) => [p.id, p] as const))(
    'gives %s exactly one tab per mapped phase',
    (_id, profile) => {
      const tabs = renderFor(profile);
      for (const phase of profile.creation_phases) {
        const tab = TAB_FOR_PHASE[phase];
        if (tab === null) {
          expect(tabs).not.toContain(phase);
          continue;
        }
        expect(tabs.filter((candidate) => candidate === tab)).toEqual([tab]);
      }
    },
  );

  it.each(shippedTypeProfiles.map((p) => [p.id, p] as const))(
    'shows %s its mapped tabs in phase order',
    (_id, profile) => {
      const tabs = renderFor(profile);
      const expected = profile.creation_phases
        .filter((phase) => !POSITION_EXEMPT.has(phase))
        .map((phase) => TAB_FOR_PHASE[phase])
        .filter((tab): tab is string => tab !== null);
      expect(tabs.filter((tab) => expected.includes(tab))).toEqual(expected);
    },
  );
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
