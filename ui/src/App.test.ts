import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';

import type { EffectiveScores, Entity, LocalizedRuleset } from './lib/types';

// While a native file dialog is open the app must be inert behind a blocking
// overlay: rfd dialogs are not input-modal on Linux, so without this the user can
// keep editing the character behind the dialog. Everything the app root reaches
// goes over the Tauri IPC bridge, so mock it away; harness mirrors
// lib/components/XpBar.test.ts.
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
    ability_funding: 'pool',
    saga_year: 1220,
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

  // UPDATED BY C4, from "keeps only the language control in the header". C4 moved
  // the language and the validation mode into the settings dialog, so what the
  // header keeps is the way IN to them — a button, on every screen, since the
  // preferences belong to the app rather than to a document.
  it('offers settings from the header on the startup screen', () => {
    store.view = 'start';
    const body = html();

    expect(openTag(body, 'settings-button')).not.toBeNull();
    // The two controls that used to sit here live in the dialog now, and the dialog
    // is closed, so neither is on screen.
    expect(openTag(body, 'language-select')).toBeNull();
    expect(openTag(body, 'mode-select')).toBeNull();
    // The status would read "unsaved" for a document that does not exist.
    //
    // Saving is unavailable here too, and used to be asserted as an absent
    // `save-button`. Since C3c that availability is a DISABLED MENU ITEM, which
    // no rendered string can show — the claim lives in `state.svelte.test.ts`'s
    // "withholds the document-writing actions on the startup screen", against
    // `documentActionEnabled` itself.
    expect(openTag(body, 'doc-status')).toBeNull();
  });

  it('offers settings from the header in the editor too', () => {
    expect(openTag(html(), 'settings-button')).not.toBeNull();
  });

  // --- the header's real estate (C5) -----------------------------------------
  //
  // The header used to spend a whole row on a logo and a visible `<h1>` repeating
  // the app's name, and `.brand`'s `flex: 1 1 100%` pushed the controls onto a
  // second row at EVERY width. The title is the OS window title already (set from
  // this same component) and, on macOS, the application menu's; rendering it a
  // third time inside the window is duplicated chrome on a 900px-minimum window.

  /** The `<header class="app-header">` element's inner markup. */
  function headerMarkup(body: string): string {
    const header = /<header[^>]*class="app-header"[^>]*>([\s\S]*?)<\/header>/i.exec(body);
    expect(header, 'the app should render a .app-header').not.toBeNull();
    return header![1];
  }

  // DROPPING THE HEADING OUTRIGHT WAS NOT AN OPTION. `<h1>` was the document's
  // only top-level heading and, since `app-logo-alt` names the *licence* logo
  // rather than the app, the only thing naming the application in-window to a
  // screen reader. It is kept and made `.sr-only` — announced, zero height — which
  // is the one utility in this stylesheet that makes that trade (see app.css).
  it('names the app in a top-level heading that costs the header no height', () => {
    const headings = [...html().matchAll(/<h1([^>]*)>([\s\S]*?)<\/h1>/gi)];

    // Exactly one: a second `<h1>` would break the document outline just as
    // surely as none at all.
    expect(headings).toHaveLength(1);
    expect(headings[0][1]).toMatch(/class="[^"]*\bsr-only\b[^"]*"/);
    // The name itself, through Fluent — never the raw key.
    const title = headings[0][2].replace(/<[^>]*>/g, '').trim();
    expect(title).toBe(store.t('app-title'));
    expect(title).not.toBe('app-title');
  });

  // The logo is an attribution mark for the Ars Magica Open License, and its alt
  // says so. That is the right name for what the image IS — which is exactly why
  // it could not be promoted to carry the app's name when the visible `<h1>` went
  // away, and why the heading above had to stay.
  it('keeps the licence logo named for what it is, not for the app', () => {
    const logo = /<img[^>]*class="app-logo"[^>]*>/i.exec(html());
    expect(logo).not.toBeNull();
    const alt = /alt="([^"]*)"/.exec(logo![0]);
    expect(alt, 'the logo should carry an alt').not.toBeNull();
    expect(alt![1].trim()).not.toBe('');
    expect(alt![1]).toBe(store.t('app-logo-alt'));
  });

  it('puts the logo last in the header, out of the way of what the user reads', () => {
    const header = headerMarkup(html());
    // Reading order as well as visual order: the licence mark is the least
    // important thing on the row, so it comes after the document status and after
    // every control rather than leading the header the way it used to.
    expect(header.indexOf('class="app-logo"')).toBeGreaterThan(
      header.indexOf('data-testid="settings-button"'),
    );
    expect(header.indexOf('class="app-logo"')).toBeGreaterThan(
      header.indexOf('data-testid="doc-status"'),
    );
  });

  // GREEN ON ARRIVAL, and deliberately so: this pins SURVIVAL through the shrink
  // rather than new behaviour. `.doc-status` is the only on-screen surface for
  // unsaved state since C3c retired the toolbar, and it sat inside the `.brand`
  // wrapper this slice deletes.
  it('keeps the document status and its ASCII dirty marker in the header', () => {
    store.currentPath = '/saves/bonisagus.armc.json';
    const header = headerMarkup(html());
    expect(header).toContain('data-testid="doc-status"');
    expect(textOf(html(), 'doc-status')).toContain('bonisagus.armc.json');

    // The marker is the dirty variant of that same key, and it must be a plain
    // ASCII asterisk — never a typographic glyph (CLAUDE.md).
    for (const key of ['app-document-name-dirty', 'app-document-unsaved-dirty']) {
      const label = store.t(key, { name: 'bonisagus.armc.json' });
      expect(label.codePointAt(0), key).toBe(0x2a);
    }
  });

  it('renders no settings dialog until it is asked for', () => {
    expect(openTag(html(), 'settings-dialog')).toBeNull();
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

// --- what the toolbar left behind (C3c) --------------------------------------
//
// C3c retires the in-app document toolbar: New/Open/Save/Save As/Export are the
// native menu's (C3a) and the keyboard's, and duplicating them as buttons is the
// second copy of a gate this project spent C3a collapsing into one.
//
// Two things inside `SaveLoadBar` were NOT document actions and had to survive
// it: the app's error banner and the "continue in the guided flow" entry. The
// first three tests here were written against the toolbar and passed unchanged
// once it was gone, which is the point of them — they pin the behaviour across
// the move rather than the markup it used to live in.
describe('the header after the document toolbar', () => {
  it('shows the error banner in the editor header', () => {
    store.error = { kind: 'io' } as never;
    const body = html();

    const banner = openTag(body, 'error');
    expect(banner).not.toBeNull();
    expect(banner).toMatch(/role="alert"/);
    expect(textOf(body, 'error')).toBe(store.t('error-io'));
    expect(textOf(body, 'error')).not.toBe('error-io');
  });

  // `AppError::Export` carries the specific missing Fluent/catalogue keys, and
  // the banner names them — a passive "some text is missing" leaves the user
  // nothing to act on or report.
  it('names the missing keys an export failure reports', () => {
    store.error = { kind: 'export', missing: ['spell-name', 'type-grog'] } as never;

    expect(textOf(html(), 'error')).toBe(
      store.t('error-export', { missing: 'spell-name, type-grog' }),
    );
  });

  it('renders no banner while nothing has failed', () => {
    store.error = null;
    expect(openTag(html(), 'error')).toBeNull();
  });

  // E4: this surface receives a ruleset failure too, and the start screen is not
  // the only one that can. `setLang` reloads the ruleset for the new language, so
  // a rules-editor who has just broken `rules/i18n/de/*.json` and switches the UI
  // to German lands HERE, in the editor, with the character they were building
  // still on screen. Before this the banner said only "The ruleset could not be
  // loaded" and the engine's own diagnostics went nowhere.
  it('offers the technical detail of a ruleset failure beside the sentence', () => {
    store.error = {
      kind: 'ruleset',
      ruleset_kind: 'integrity',
      errors: ["unknown prerequisite 'virtue.x' referenced by 'virtue.a'"],
    } as never;
    const body = html();

    expect(textOf(body, 'error')).toBe(store.t('error-ruleset'));
    expect(openTag(body, 'error-details')).not.toBeNull();
    expect(body).toContain("unknown prerequisite 'virtue.x' referenced by 'virtue.a'");
  });

  // `role="alert"` is `aria-live="assertive"` + `aria-atomic="true"`: whatever is
  // inside it is read in full, interrupting the user, and re-read whenever it
  // changes. An integrity failure carries one message per violation, so the
  // payload must stay OUTSIDE the region — the sentence is the alert, the
  // disclosure is a sibling the user opens deliberately.
  it('keeps the diagnostics out of the alert live region', () => {
    store.error = {
      kind: 'ruleset',
      ruleset_kind: 'integrity',
      errors: ["unknown prerequisite 'virtue.x' referenced by 'virtue.a'"],
    } as never;
    const body = html();

    expect(openTag(body, 'error')).toMatch(/role="alert"/);
    expect(textOf(body, 'error')).not.toContain('unknown prerequisite');

    const details = openTag(body, 'error-details');
    expect(details).not.toBeNull();
    expect(details).not.toMatch(/role="alert"/);
  });

  // No disclosure where there is no payload: every other failure kind says
  // everything it knows in its own localized sentence.
  it('offers no technical detail for a failure that carries none', () => {
    store.error = { kind: 'io' } as never;
    expect(openTag(html(), 'error-details')).toBeNull();
  });

  // The five document actions are the menu's and the keyboard's now. Asserting
  // their ABSENCE is what stops the toolbar growing back one button at a time.
  it('offers no in-app buttons for the document actions', () => {
    const body = html();
    for (const testid of [
      'new-button',
      'open-button',
      'save-button',
      'save-as-button',
      'export-button',
    ]) {
      expect(openTag(body, testid), testid).toBeNull();
    }
  });
});

// Slice 5 (#31): the character on screen can be walked through the guided flow,
// not only a brand-new one. Moved here from `SaveLoadBar.test.ts` with the
// toolbar's removal (C3c) — the action was never a document action, so it stayed
// in the header while the five that were went to the menu. Offered
// CONDITIONALLY: a save from another ruleset may name a type this build has no
// profile for, and a wizard with no rail is not a screen to enter.
describe('the guided-creation entry in the header', () => {
  it('offers continuing the loaded character in the guided flow', () => {
    installRuleset('companion');
    const body = html();

    expect(openTag(body, 'wizard-continue-button')).toMatch(/<button/i);
    expect(textOf(body, 'wizard-continue-button')).toBe(store.t('action-continue-in-wizard'));
    expect(textOf(body, 'wizard-continue-button')).not.toBe('action-continue-in-wizard');
  });

  it('localizes the label to German', () => {
    installRuleset('companion');
    store.lang = 'de';
    expect(textOf(html(), 'wizard-continue-button')).toBe(store.t('action-continue-in-wizard'));
    store.lang = 'en';
  });

  it('is not offered for a type_id the loaded ruleset has no profile for', () => {
    installRuleset();
    expect(store.ruleset!.ruleset.type_profiles[store.entity.type_id]).toBeUndefined();
    expect(openTag(html(), 'wizard-continue-button')).toBeNull();
  });

  it('is not offered while the wizard is already on screen', () => {
    installRuleset('companion');
    store.view = 'wizard';
    expect(openTag(html(), 'wizard-continue-button')).toBeNull();
    store.view = 'editor';
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
  // document actions have to be reachable, because the close guard promises the
  // user can save rather than lose the work.
  //
  // That last clause used to be asserted here as a rendered `save-button`. C3c
  // moved saving to the native menu, whose enabled state is not markup, so the
  // claim moved with it: `state.svelte.test.ts`'s "offers the document-writing
  // actions in the guided wizard too" pins `documentActionEnabled` for
  // `view === 'wizard'`. What is still a rendering — and so still belongs here —
  // is the rest of the header.
  it('keeps the banner and the document controls', () => {
    const body = html();
    expect(openTag(body, 'character-type')).not.toBeNull();
    expect(openTag(body, 'identity-name')).not.toBeNull();
    // `mode-select` stood here until C4 moved it into the settings dialog. What the
    // header keeps on every screen — the wizard included — is the way in to it.
    expect(openTag(body, 'settings-button')).not.toBeNull();
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

  // Focus Power is a *Supernatural* Virtue (ArMDE:3895-3896), open to every
  // character type including a magus — and it grants no Might. Gating the
  // Supernatural tab on Might alone would leave a companion or a magus holding
  // Focus Power with nowhere to enter one, so the tab opens on the pool too.
  it('opens the Supernatural tab for a Focus Power pool, with no Might at all', () => {
    store.entity.type_id = 'companion';
    store.effective = { focus_points_budget: 25, might: null } as unknown as EffectiveScores;
    expect(openTag(html(), 'tab-supernatural')).not.toBeNull();
  });

  it('opens it for a magus with a Focus Power pool, though magi never have Might', () => {
    store.ruleset!.ruleset.type_profiles.magus = {
      hermetically_trained: true,
    } as unknown as LocalizedRuleset['ruleset']['type_profiles'][string];
    store.entity.type_id = 'magus';
    store.effective = { focus_points_budget: 25, might: null } as unknown as EffectiveScores;
    expect(openTag(html(), 'tab-supernatural')).not.toBeNull();
  });

  it('leaves the Supernatural tab closed for a character with neither', () => {
    store.entity.type_id = 'companion';
    store.effective = { focus_points_budget: 0, might: null } as unknown as EffectiveScores;
    expect(openTag(html(), 'tab-supernatural')).toBeNull();
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
  hermetically_trained?: boolean;
  order_member?: boolean;
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
