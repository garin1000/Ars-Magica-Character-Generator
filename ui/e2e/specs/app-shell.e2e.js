// End-to-end: app-wide chrome that is not owned by any one editor tab — boot
// state and entry, the header's document-status readout, German localization of
// the chrome itself, and the window.close() bridge for a CLEAN document.
//
// A2 (spec consolidation) merged four previously separate spec files into this
// one, in this fixed order, for two hazards that only exist once they share a
// session:
//
//  1. `app entry`'s first `it` asserts the app's BOOT state (start screen, no
//     tablist, no doc-status, no mode-select). It must run before
//     any other `it` in this file creates a character, so it is placed as the
//     FIRST `it` of the FIRST describe below — not because of file-name sort
//     order (that used to matter when every spec got its own process; it no
//     longer does, see `e2e/README.md`), but because this is now the first thing
//     that runs in THIS file's own WebDriver session.
//  2. `German localization` switches the UI language to `de` and never restored
//     it on its own — harmless as a standalone process, but every describe below
//     it in this shared session would otherwise render in German. Its own
//     `after` hook restores `en`, the same pattern used by
//     `magus-editor.e2e.js` (ex-`arts.e2e.js`), `wizard-flow.e2e.js`
//     (ex-`tab-area.e2e.js`), and `magus-possessions.e2e.js`
//     (ex-`export-markdown.e2e.js`).
//
// `window.close() bridge — no unsaved changes` MUST stay last: a clean close
// actually destroys the window and ends this worker's WebDriver session, so
// nothing may run after it in this file.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { workerConfigHome } from '../driver.js';
import {
  activateMenuItem,
  clean,
  closeSettings,
  openSettings,
  returnToStartScreen,
  runDocumentAction,
  setLanguage,
  setTheme,
  startCharacter,
} from '../helpers.js';
// The app's save/load dialog seam (ARM_E2E_FILE) points at this fixed path, so
// Save and Open never raise a native dialog.
import { e2eFile } from '../wdio.conf.js';

const START_SCREEN = '[data-testid="start-screen"]';
const START_OPEN = '[data-testid="start-open"]';
const START_WIZARD_MAGUS = '[data-testid="start-wizard-magus"]';
const CHARACTER_TYPE = '[data-testid="character-type"]';
const TAB_BAR = '[role="tablist"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const NAME_INPUT = '[data-testid="identity-name"]';
const SETTINGS_DIALOG = '[data-testid="settings-dialog"]';

describe('app entry', () => {
  it('boots on the startup screen, not in the editor', async () => {
    await $(START_SCREEN).waitForExist({ timeout: 30000 });

    // The three ways in, all live: open an existing character, create one
    // directly, or walk one through the guided flow. The guided entry is per type
    // for the same reason creation is — the type is fixed at creation.
    await $(START_OPEN).waitForExist({ timeout: 10000 });
    await $(START_WIZARD_MAGUS).waitForExist({ timeout: 10000 });
    expect(await $(START_WIZARD_MAGUS).isEnabled()).toBe(true);
    expect((await $$('[data-testid^="start-wizard-"]')).length).toBeGreaterThan(0);

    // One create button per character type the ruleset declares. The set is data,
    // so the count is never pinned — only that the profiles arrived and that the
    // one the next test uses is among them.
    await $('[data-testid="start-create-magus"]').waitForExist({ timeout: 10000 });
    expect((await $$('[data-testid^="start-create-"]')).length).toBeGreaterThan(0);

    // No character exists yet, so nothing that acts on one is offered: no editor
    // tabs, no document status, no validation-mode toggle.
    //
    // Saving is unavailable here too, but since C3c that is no longer an absent
    // BUTTON — it is a disabled item on the native menu, which WebDriver cannot
    // see. The claim moved to `store.documentActionEnabled('save')` in
    // `state.svelte.test.ts`, where it is a predicate rather than a rendering.
    expect(await $(TAB_BAR).isExisting()).toBe(false);
    expect(await $('[data-testid="doc-status"]').isExisting()).toBe(false);
    expect(await $('[data-testid="mode-select"]').isExisting()).toBe(false);

    // The app's own preferences ARE offered here — they are not about a document.
    await expect($('[data-testid="settings-button"]')).toExist();

    // And the language is offered a second time, on this screen itself (C4). The
    // settings button says "Settings" in whatever language the app is running in,
    // which on a first launch is English; a reader who cannot read that word would
    // otherwise have no way to the one control that fixes it.
    await expect($('[data-testid="start-language-select"]')).toExist();
  });

  it('creates a magus with its free traits and a read-only type label', async () => {
    await startCharacter('magus');

    // The banner names the type through its Fluent key — a read-only label, and
    // never the raw `magus` slug.
    const label = await $(CHARACTER_TYPE);
    await label.waitForExist({ timeout: 10000 });
    const labelText = clean(await label.getText());
    expect(labelText).toContain('Magus');
    expect(labelText).not.toContain('type-magus');

    // The type is fixed at creation: the old header selector is gone for good.
    expect(await $('[data-testid="type-select"]').isExisting()).toBe(false);

    // The profile's mandatory free traits are seeded, so a direct-entry magus is
    // legal without hand-picking them. They are the profile's traits rather than
    // the player's, so they are listed as REQUIRED rows carrying no remove button
    // — hence reading the selected-Virtues column instead of a remove control.
    await $(VF_TAB).click();
    const selectedVirtues = await $('[data-testid="selection-list-virtue"]');
    await selectedVirtues.waitForExist({ timeout: 10000 });
    const listed = clean(await selectedVirtues.getText());
    expect(listed).toContain('The Gift');
    expect(listed).toContain('Hermetic Magus');
    expect(await $('[data-testid^="remove-virtue.the_gift"]').isExisting()).toBe(false);
  });

  it('returns to the startup screen with New, through the discard guard', async () => {
    await startCharacter('companion');

    // An edit makes the document dirty, so New must ask before discarding it.
    await $(NAME_INPUT).setValue('Abandoned draft');
    await runDocumentAction('new');
    await $('[data-testid="discard-prompt"]').waitForExist({ timeout: 10000 });

    // Cancelling keeps the character being edited.
    await $('[data-testid="discard-cancel"]').click();
    await browser.waitUntil(async () => !(await $('[data-testid="discard-prompt"]').isExisting()), {
      timeout: 10000,
      timeoutMsg: 'cancelling should close the discard prompt',
    });
    expect(await $(NAME_INPUT).getValue()).toBe('Abandoned draft');
    expect(await $(START_SCREEN).isExisting()).toBe(false);

    // Confirming discards it and lands back on the type choice.
    await runDocumentAction('new');
    await $('[data-testid="discard-confirm"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="discard-confirm"]').click();
    await $(START_SCREEN).waitForExist({ timeout: 10000 });
    expect(await $(TAB_BAR).isExisting()).toBe(false);
  });

  it('opens a saved character from the startup screen, with that file type', async () => {
    // Build and save a grog — a type the previous tests never used, so the type
    // shown after the Open can only have come off the file.
    await startCharacter('grog');
    await $(NAME_INPUT).setValue('Rolf the Turb');
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    expect(JSON.parse(fs.readFileSync(e2eFile, 'utf-8')).type_id).toBe('grog');

    // Back to the startup screen, then in again through its own Open button.
    await returnToStartScreen();
    const open = await $(START_OPEN);
    await open.waitForClickable({ timeout: 10000 });
    await open.click();

    // The editor comes up carrying the saved grog. This is also the one spec that
    // opens a character with NO Virtues or Flaws at all, which is how it caught the
    // engine's canonical JSON omitting an empty `selections` (see `open()` in
    // state.svelte.ts): the editor's mount threw mid-render and the startup screen
    // stayed frozen on screen.
    await $(TAB_BAR).waitForExist({ timeout: 15000 });
    await browser.waitUntil(async () => (await $(NAME_INPUT).getValue()) === 'Rolf the Turb', {
      timeout: 10000,
      timeoutMsg: 'Open should load the saved character',
    });
    expect(clean(await $(CHARACTER_TYPE).getText())).toContain('Grog');
  });
});

describe('header document status', () => {
  const STATUS = '[data-testid="doc-status"]';

  it('shows unsaved, then dirty, then the file name after save', async () => {
    // A brand-new character has never been saved, which is the state the first
    // assertion below is about — and the status only exists in the editor, never
    // on the startup screen.
    await startCharacter('companion');
    await $('[data-testid="tab-characteristics"]').waitForExist({ timeout: 30000 });
    const status = await $(STATUS);
    await status.waitForExist({ timeout: 10000 });

    // A fresh, never-saved document: an "unsaved" label, no file name, no marker.
    const initial = clean(await status.getText());
    expect(initial).not.toContain('.json');
    expect(initial.startsWith('*')).toBe(false);

    // Editing a value marks the document dirty -> the ASCII `*` marker appears.
    await $('[data-testid="tab-characteristics"]').click();
    const intInc = await $('[data-testid="char-inc-int"]');
    await intInc.waitForExist({ timeout: 10000 });
    await intInc.click();
    await browser.waitUntil(async () => clean(await status.getText()).startsWith('*'), {
      timeout: 5000,
      timeoutMsg: 'editing should show the dirty marker in the header',
    });

    // Saving shows the file name on-screen and clears the dirty marker.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    await browser.waitUntil(
      async () => {
        const t = clean(await status.getText());
        // Derived, never literal: the fixture carries a per-worker suffix so
        // parallel workers cannot clobber each other's save file, so the name
        // the header shows is only knowable from the path the config handed us.
        return t.includes(path.basename(e2eFile)) && !t.startsWith('*');
      },
      {
        timeout: 5000,
        timeoutMsg: 'header should show the saved file name without a dirty marker',
      },
    );
  });
});

// C5. The header is chrome, and every pixel of it is charged to the character
// surfaces below on a window whose `minHeight` is 900 (tauri.conf.json). It used
// to take a whole second row: `.brand` declared `flex: 1 1 100%`, so the controls
// wrapped at EVERY width — an unconditional break no available width could undo —
// and above and below that sat 1rem of padding around a 2.25rem logo and an `<h1>`
// repeating the OS window title.
//
// Measured in the real engine, because that is the only place a height exists.
// `render` from `svelte/server` attaches no stylesheet and happy-dom performs no
// layout, so app.css.test.ts can pin the numbers that were CHOSEN but never the
// box they produce.
describe('header real estate', () => {
  it('spends one row on the header, with the logo in its upper right', async () => {
    await startCharacter('companion');
    await $('[data-testid="doc-status"]').waitForExist({ timeout: 30000 });

    const box = await browser.execute(() => {
      const rect = (el) => {
        if (!el) return null;
        const r = el.getBoundingClientRect();
        return { top: r.top, bottom: r.bottom, left: r.left, right: r.right, height: r.height };
      };
      const header = document.querySelector('.app-header');
      return {
        header: rect(header),
        logo: rect(header.querySelector('.app-logo')),
        status: rect(header.querySelector('[data-testid="doc-status"]')),
        settings: rect(header.querySelector('[data-testid="settings-button"]')),
      };
    });

    // THE BUDGET. One row of controls, its padding and the bottom rule. The header
    // measured 113px before this slice, which is where the number to beat comes
    // from; 48 is comfortably above what a single control row needs and still less
    // than half of what was there.
    // Measured at 38.8px here against 96.8px before the slice — the budget below
    // is the guard, not the achievement.
    expect(box.header.height).toBeLessThanOrEqual(48);

    // ONE ROW, not merely a shorter stack: the three things on it all overlap each
    // other's vertical band. A wrapped header could satisfy the height budget alone
    // by shedding padding, and would still be the layout this slice is about.
    for (const child of [box.logo, box.status, box.settings]) {
      // `null` would mean the element left the header altogether — the logo, the
      // status and the Settings button all have to still BE here.
      expect(child).not.toBe(null);
      expect(child.top).toBeGreaterThanOrEqual(box.header.top - 1);
      expect(child.bottom).toBeLessThanOrEqual(box.header.bottom + 1);
    }
    expect(box.logo.top).toBeLessThan(box.settings.bottom);
    expect(box.settings.top).toBeLessThan(box.logo.bottom);
    expect(box.status.top).toBeLessThan(box.settings.bottom);

    // …and the logo is the rightmost thing on that row — the upper right corner.
    expect(box.logo.left).toBeGreaterThanOrEqual(box.settings.right);
    expect(box.logo.right).toBeLessThanOrEqual(box.header.right);
    // The document status keeps the left, where the eye starts.
    expect(box.status.left).toBeLessThan(box.settings.left);

    // The dirty marker's only on-screen home survives the shrink AND stays
    // readable: `getText()` returns '' for an element clipped by `overflow:
    // hidden`, so this fails loudly if the tightened row started clipping.
    expect(clean(await $('[data-testid="doc-status"]').getText())).not.toBe('');
  });
});

describe('German localization', () => {
  const SPELLS_TAB = '[data-testid="tab-spells"]';

  // This spec switches the whole app to German and does not restore it on its
  // own line, unlike the other language-switching specs in this suite (compare
  // `magus-editor.e2e.js`'s ex-`arts.e2e.js` block). A dedicated `after` hook
  // restores `en` here instead, so every describe below this one in the shared
  // session still starts in English.
  //
  // C4 moved the language control into the settings dialog, so the switch goes
  // through `setLanguage` — which opens the dialog, chooses, waits and closes it
  // again. The language now also PERSISTS, which makes restoring `en` here matter
  // beyond this session: the worker's own `XDG_CONFIG_HOME` keeps the written file
  // out of the developer's config either way, but a worker left in German would
  // still start its next run there.
  after(async () => {
    await setLanguage('en');
  });

  it('renders German chrome and a non-empty German spell tooltip', async () => {
    // A magus of this spec's own — the Spells tab is magus-only.
    await startCharacter('magus');

    await setLanguage('de');

    // The Spells tab's "Available" region title must render in German
    // ("Verfügbar"), proving UI chrome re-localizes.
    await $(SPELLS_TAB).waitForExist({ timeout: 10000 });
    await $(SPELLS_TAB).click();
    const available = await $('[data-testid="available-title"]');
    await available.waitForExist({ timeout: 5000 });
    // `.region-title` uppercases via CSS for display, so compare case-insensitively.
    expect(
      clean(await available.getText())
        .trim()
        .toLowerCase(),
    ).toBe('verfügbar');

    // Hover a spell row and assert the description tooltip has text (German where
    // translated, English fallback otherwise) — never the empty tooltip bug.
    const row = await $('[data-testid="add-spell.pilum_of_fire"]');
    await row.waitForExist({ timeout: 10000 });
    await browser.execute((el) => {
      el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
    }, row);
    const pop = await $('[data-testid="tooltip-text"]');
    await pop.waitForExist({ timeout: 5000 });
    expect((await pop.getText()).trim().length).toBeGreaterThan(0);
  });
});

// C6. The native menu, driven and read through the two `e2e-testing` seams in
// `crates/arm-app/src/commands.rs`.
//
// WHY SEAMS AT ALL, when C3c argued so firmly for the keyboard. C3c's argument
// was about the five DOCUMENT ACTIONS, which the keyboard genuinely offers and
// which the portable suite (built without `e2e-testing`) can only reach that
// way. It was never an argument that the menu itself needs no coverage, and two
// claims sit outside the keyboard's reach entirely:
//
//   * that ACTIVATING a menu item does anything at all. The keyboard chord
//     proves `shortcutAction` -> `runDocumentAction`; it says nothing about the
//     Rust `on_menu_event` -> `menu://action` -> `runMenuAction` path, which is
//     the only route a mouse user has since the toolbar went.
//   * that the model the frontend pushes ever reached the OS. `menu_model` is
//     unit tested as a pure value, and a pure value cannot show that a runtime
//     language switch REBUILT the installed menu — the exact failure C3a called
//     out, where the bar sits in English behind a German UI all session.
//
// Both seams are inert in the shipped build (`menu_test_seams_enabled`), proved
// in `crates/arm-app/tests/commands.rs` under the plain `cargo test` gate.
describe('the native menu', () => {
  const MENU_SAVE = 'menu.save';

  /**
   * The menu Tauri has actually installed, read off `AppHandle::menu()`.
   *
   * `browser.execute` does not await a promise, so the answer is parked on a
   * global and polled for — steadier here than the deprecated `executeAsync`
   * under WebKitWebDriver's script timeout.
   *
   * The answer is WRAPPED rather than parked bare, because WebDriver
   * serializes `undefined` to `null`: a bare `window.x = undefined` sentinel
   * is indistinguishable over the wire from the command having answered
   * `None`, so the poll would fall through on its first tick and read a menu
   * that had not arrived. An object is `null` until it exists, and nothing
   * else ever is.
   */
  async function installedMenu() {
    await browser.execute(() => {
      window.__armInstalledMenu = null;
      window.__TAURI_INTERNALS__.invoke('installed_menu').then(
        (menu) => {
          window.__armInstalledMenu = { ok: true, menu };
        },
        (error) => {
          window.__armInstalledMenu = { ok: false, error: String(error) };
        },
      );
    });
    await browser.waitUntil(
      async () => (await browser.execute(() => window.__armInstalledMenu)) !== null,
      { timeout: 10000, timeoutMsg: 'the installed_menu seam never answered' },
    );

    const answer = await browser.execute(() => window.__armInstalledMenu);
    // A rejection here means the command is not registered or the seam is not
    // compiled in; say which rather than failing on a null menu three lines on.
    if (!answer.ok) throw new Error(`installed_menu rejected: ${answer.error}`);
    // `null` now means the command answered but NO menu is installed, which is
    // itself the failure this whole describe exists to catch.
    expect(answer.menu).not.toBe(null);
    return answer.menu;
  }

  /**
   * The File submenu. On GTK it is the only section there is — muda renders no
   * other predefined item, so `menu_model` builds no Edit or Window menu here
   * (`crates/arm-app/src/menu.rs`).
   */
  function fileSection(menu) {
    expect(menu.length).toBeGreaterThan(0);
    return menu[0];
  }

  function itemById(section, id) {
    const item = section.items.find((entry) => entry.kind === 'action' && entry.id === id);
    if (item === undefined) throw new Error(`the installed File menu has no ${id}`);
    return item;
  }

  after(async () => {
    await setLanguage('en');
  });

  it('runs the action an activated item names, gate and store included', async () => {
    await startCharacter('grog');
    await $(NAME_INPUT).setValue('Menu-driven Marius');
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);

    // Not a keyboard chord and not a button: the id goes to Rust, which
    // forwards it on `menu://action` through the very function
    // `on_menu_event` calls.
    await activateMenuItem(MENU_SAVE);

    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'activating the Save menu item did not save the document',
    });
    expect(JSON.parse(fs.readFileSync(e2eFile, 'utf-8')).name).toBe('Menu-driven Marius');
  });

  it('leaves a disabled item disabled, rather than working around the gate', async () => {
    // The startup screen has no document, so Save is withheld — the one place
    // `documentActionEnabled` says no to something the menu still shows.
    await returnToStartScreen();
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);

    // Read off the real object, not assumed: the item really is greyed out.
    expect(itemById(fileSection(await installedMenu()), MENU_SAVE).enabled).toBe(false);

    await activateMenuItem(MENU_SAVE);

    // A refusal produces nothing to wait for, so establish ORDER instead of
    // sleeping: menu ids arrive on one event channel in the order they were
    // emitted, so once Settings — which IS enabled here — has opened, the Save
    // fired before it has already been delivered and turned away. That also
    // makes this the positive half: an enabled item activates on this very
    // screen, so the withholding is the gate's doing and not a dead seam.
    await activateMenuItem('menu.settings');
    await $(SETTINGS_DIALOG).waitForExist({ timeout: 10000 });
    await closeSettings();

    expect(fs.existsSync(e2eFile)).toBe(false);
  });

  it('carries the same actions, in menu order, that the model declares', async () => {
    await startCharacter('grog');
    const file = fileSection(await installedMenu());

    // The order C3a's `the_file_menu_offers_every_document_action_on_every_platform`
    // asserts of the MODEL, asserted here of the object the OS was handed.
    expect(file.items.filter((item) => item.kind === 'action').map((item) => item.id)).toEqual([
      'menu.new',
      'menu.open',
      MENU_SAVE,
      'menu.save-as',
      'menu.export',
      'menu.settings',
    ]);
    // …and the separators between them survived the crossing. A predefined item
    // reports no enabled state and no role (see `InstalledMenuItem`), so an
    // empty title is all a separator can be recognised by.
    expect(file.items.some((item) => item.kind === 'other' && item.title === '')).toBe(true);
  });

  // C7, TRAP 2 — WHERE "EXACTLY ONCE" IS PROVED, AND WHY NOT HERE.
  //
  // The chord and the menu item are the same owner since C7: the chord is the
  // item's accelerator, so the OS turns a press into the activation this
  // describe already exercises. What must never happen is one press producing
  // the action TWICE, which is what a surviving webview keydown handler beside
  // the accelerator would do — and on a dirty document that is two discard
  // prompts for one Ctrl+N, the hazard that made C3a and C3c ship no
  // accelerators at all.
  //
  // Neither half of that can be counted from a spec, and both were tried:
  //
  //   * The press cannot be made. WebKitWebDriver feeds a synthesized key into
  //     WebKit's own input pipeline rather than delivering it as an event on the
  //     window, so it never reaches the accel group GTK matches in
  //     `gtk_window_key_press_event` — measured, not assumed: every chord was
  //     inert under `browser.keys` while `gtk_accel_groups_activate` on the live
  //     window answered true for Ctrl+N.
  //   * The dispatches cannot be counted. Wrapping
  //     `window.__TAURI_INTERNALS__.invoke` to tally commands looks like the
  //     obvious tally, and does not work: Tauri locks the property down, so
  //     neither assignment nor `Object.defineProperty` replaces it — a version
  //     of this test carrying that patch reported zero saves for a save that had
  //     demonstrably happened. Do not reach for it again.
  //
  // So exactly-once is proved structurally instead, in two halves that together
  // leave no second owner: the page dispatches nothing for any of these chords
  // (`the document chords belong to the native menu, not the webview`,
  // `App.client.test.ts`), and Rust announces a chosen item through one function
  // and only one (`the_os_menu_handler_and_the_activation_seam_share_one_dispatch_path`,
  // `crates/arm-app/tests/commands.rs`).

  // THE HEADLINE. `set_app_menu` is re-invoked on every language switch, and the
  // client unit test proves the frontend makes that call — but only this can
  // show the call LANDED. A menu that Tauri never replaced would keep every
  // English label and pass every other gate in the repo.
  it('is rebuilt in German when the language changes, in the OS and not just in the model', async () => {
    await startCharacter('grog');

    const english = fileSection(await installedMenu());
    expect(english.title).toBe('File');
    expect(itemById(english, MENU_SAVE).title).toBe('Save');

    await setLanguage('de');

    // The rebuild is a round trip (the ruleset reloads first), so wait for it
    // rather than reading once and hoping.
    await browser.waitUntil(async () => fileSection(await installedMenu()).title === 'Datei', {
      timeout: 10000,
      timeoutMsg: 'the installed menu kept its English File title after switching to German',
    });

    const german = fileSection(await installedMenu());
    expect(itemById(german, 'menu.new').title).toBe('Neu');
    expect(itemById(german, 'menu.open').title).toBe('Öffnen…');
    expect(itemById(german, MENU_SAVE).title).toBe('Speichern');
    expect(itemById(german, 'menu.save-as').title).toBe('Speichern unter…');
    expect(itemById(german, 'menu.export').title).toBe('Als Markdown exportieren…');
    expect(itemById(german, 'menu.settings').title).toBe('Einstellungen…');
  });
});

// End-to-end: the window.close() bypass fix (N1 / round-2 GA2, E2), no-unsaved-
// changes half — the companion case to `window-close-bridge-dirty` in
// `companion-editor.e2e.js` (read that describe's header for the full context on
// why this exists and what it does and does not prove).
// End-to-end: the settings dialog (C4), against the real binary and the real file.
//
// Two things only this layer can prove. The first is that a chosen setting reaches
// DISK at all — the store, the IPC wrapper and the Rust writer are each unit-tested,
// but nothing below this exercises the three of them joined to a real
// `app_config_dir()`. The second is the data-loss bug this slice fixed: writing one
// setting must leave every other key in the file standing. That was a
// read-modify-write correction in `crates/arm-app/src/settings.rs`, and here it is
// checked on the file the shipped app actually wrote.
//
// SETTINGS ISOLATION — the same contract `grog-wizard-aging.e2e.js`'s saga-year
// block relies on, now carrying three more keys. Each worker runs the app under its
// own `XDG_CONFIG_HOME` (`driver.js`'s `workerConfigHome`, wired in by both wdio
// configs' `beforeSession`), so this spec can never poison a concurrent worker or
// the developer's own settings file. It recomputes that path from the same function
// rather than hardcoding one, and restores every value it changed, because the
// directory persists across runs.
describe('the settings dialog', () => {
  // `app_config_dir()` on Linux is `$XDG_CONFIG_HOME/<identifier>`, and the
  // identifier is `crates/arm-app/tauri.conf.json`'s.
  const settingsFile = path.join(
    workerConfigHome(
      path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..'),
      process.env,
    ),
    'io.github.garin1000.armchargen',
    'settings.json',
  );

  /** The settings file's parsed contents, or `{}` before anything was written. */
  function storedSettings() {
    if (!fs.existsSync(settingsFile)) return {};
    return JSON.parse(fs.readFileSync(settingsFile, 'utf8'));
  }

  after(async () => {
    await setLanguage('en');
    await setTheme('auto');
  });

  it('persists a chosen theme, and repaints with it', async () => {
    await startCharacter('grog');

    await setTheme('light');

    // The palette is named on <html>, which is the whole switch between the two
    // `:root` blocks in app.css — so this proves the choice reached the DOM, not
    // merely the store.
    await browser.waitUntil(async () => (await $('html').getAttribute('data-theme')) === 'light', {
      timeout: 5000,
      timeoutMsg: 'the light palette should be applied to <html>',
    });
    expect(storedSettings().theme).toBe('light');
  });

  it('writing one setting leaves every other one standing', async () => {
    // The regression C4 fixed. Before it, `write_saga_year` rebuilt the whole
    // document from the one field it was given, so the SECOND key written silently
    // deleted the first — no error, no banner, no undo.
    await setLanguage('de');

    const stored = storedSettings();
    expect(stored.lang).toBe('de');
    expect(stored.theme).toBe('light');
    // And the default saga year, written by an entirely different surface, is
    // untouched. (C8 renamed the key from `saga_year`; the old name must never
    // reappear, because a file carrying both would have two answers.)
    expect(stored.saga_year).toBe(undefined);
    expect(
      stored.default_saga_year === undefined || typeof stored.default_saga_year === 'number',
    ).toBe(true);
  });

  // C8. The saga year itself is document state now — a storyguide runs more than one
  // saga, and the machine-global number was wrong for all but one of them. What is
  // left in this dialog is the year a NEW document starts at, and the claim with
  // teeth is that it seeds the next character WITHOUT reaching into the open one.
  it('persists a default saga year that seeds the next character and not the open one', async () => {
    const SAGA_YEAR_INPUT = '[data-testid="saga-year-input"]';
    const DEFAULT_INPUT = '[data-testid="default-saga-year-input"]';
    const DETAILS_TAB = '[data-testid="tab-details"]';

    /** The open character's own saga year, read off the editor's Details tab. */
    async function openDetails() {
      const tab = await $(DETAILS_TAB);
      await tab.waitForExist({ timeout: 10000 });
      await tab.click();
      await $(SAGA_YEAR_INPUT).waitForExist({ timeout: 10000 });
    }

    await startCharacter('grog');
    await openDetails();

    // The fresh character carries the current default, which a worker with no
    // stored value gets from the engine's own constant.
    expect(await $(SAGA_YEAR_INPUT).getValue()).toBe('1220');

    await openSettings();
    await $(DEFAULT_INPUT).setValue('1197');
    await browser.waitUntil(() => storedSettings().default_saga_year === 1197, {
      timeout: 5000,
      timeoutMsg: 'the chosen default saga year should reach the settings file',
    });
    await closeSettings();

    // The open document was built for 1220 and keeps its own year.
    expect(await $(SAGA_YEAR_INPUT).getValue()).toBe('1220');

    // The NEXT one is stamped with the new default.
    await startCharacter('grog');
    await openDetails();
    await browser.waitUntil(async () => (await $(SAGA_YEAR_INPUT).getValue()) === '1197', {
      timeout: 5000,
      timeoutMsg: 'a newly created character should start at the configured default',
    });

    // Restore, since the worker's settings directory persists across runs.
    await openSettings();
    await $(DEFAULT_INPUT).setValue('1220');
    await browser.waitUntil(() => storedSettings().default_saga_year === 1220, {
      timeout: 5000,
      timeoutMsg: 'the default saga year should be restored for the next run',
    });
    await closeSettings();
  });
});

//
// `guard_blocks_quit` (`crates/arm-app/src/main.rs`) returns `false` — allow —
// when the entity is not dirty, so the shadowed `window.close()` must reach the
// real close uninterrupted. Without this half, a "fix" that made `request_close`
// an unconditional no-op (rather than actually routing to `WebviewWindow::close()`)
// would pass the dirty-side spec vacuously — the window would never disappear
// either way — and never be caught.
//
// MUST BE THE LAST DESCRIBE IN THIS FILE: a clean close ends this app instance
// and its WebDriver session, so nothing may run after it in the same worker.
describe('window.close() bridge — no unsaved changes', () => {
  it('a clean entity actually closes on window.close() called from page script', async () => {
    // Creating a character is not itself a dirty edit ("a freshly created
    // character is not dirty" — `ui/src/lib/state.svelte.ts`), so no confirmation
    // dialog stands in the way.
    await startCharacter('grog');

    // No dialog blocks it and this single-window app has nothing to fall back to,
    // so the app — and the WebDriver session riding on it — should actually go
    // away. The teardown can land *during* this call rather than after it: the
    // session dies while `execute` is still in flight, so the call itself rejects
    // ("invalid session id" / "session deleted because of page crash or hang").
    // That rejection is not a test failure, it IS the evidence the close
    // happened, so it counts as success rather than propagating.
    let sessionGone = false;
    try {
      await browser.execute(() => window.close());
    } catch {
      sessionGone = true;
    }

    if (!sessionGone) {
      // The close can also complete just after the call returns, so fall back to
      // polling until the session stops answering.
      await browser.waitUntil(
        async () => {
          try {
            await browser.getWindowHandles();
            return false;
          } catch {
            return true;
          }
        },
        {
          timeout: 10000,
          timeoutMsg: 'window.close() did not close the window when the document was clean',
        },
      );
    }
  });
});
