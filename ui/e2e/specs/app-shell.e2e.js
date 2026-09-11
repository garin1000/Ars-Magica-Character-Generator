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

import { clean, returnToStartScreen, runDocumentAction, startCharacter } from '../helpers.js';
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

    // The language applies to the whole app, so it IS offered here.
    await expect($('[data-testid="language-select"]')).toExist();
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

describe('German localization', () => {
  const LANG_SELECT = '[data-testid="language-select"]';
  const SPELLS_TAB = '[data-testid="tab-spells"]';

  // This spec switches the whole app to German and does not restore it on its
  // own line, unlike the other language-switching specs in this suite (compare
  // `magus-editor.e2e.js`'s ex-`arts.e2e.js` block). A dedicated `after` hook
  // restores `en` here instead, so every describe below this one in the shared
  // session still starts in English.
  after(async () => {
    await $(LANG_SELECT).selectByAttribute('value', 'en');
  });

  it('renders German chrome and a non-empty German spell tooltip', async () => {
    // A magus of this spec's own — the Spells tab is magus-only.
    await startCharacter('magus');
    await $(LANG_SELECT).waitForExist({ timeout: 30000 });

    // Switch to German and confirm the language actually changed.
    await $(LANG_SELECT).selectByAttribute('value', 'de');
    await browser.waitUntil(async () => (await $(LANG_SELECT).getValue()) === 'de', {
      timeout: 5000,
      timeoutMsg: 'language should switch to German',
    });

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

// End-to-end: the window.close() bypass fix (N1 / round-2 GA2, E2), no-unsaved-
// changes half — the companion case to `window-close-bridge-dirty` in
// `companion-editor.e2e.js` (read that describe's header for the full context on
// why this exists and what it does and does not prove).
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
