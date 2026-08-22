// End-to-end: how the app is entered (M6a). The app boots on a startup choice
// screen rather than a blank character; a character's type is picked once, by
// creating one; New comes back to that screen; and Open enters the editor with
// the saved file's own type. Drives the real binary.
//
// KEEP THIS FILE NAMED `app-entry.e2e.js`. wdio globs `specs/**/*.e2e.js` and
// runs the files in sorted order, so `app-entry` sorts ahead of `arts` and every
// other spec — which makes it the ONLY spec that can legitimately observe the
// app's boot state. Rename it to anything sorting later and the first test below
// silently starts asserting the previous spec's leftovers instead. (Everything
// after the first test creates its own character through `startCharacter`, like
// every other spec.)
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { returnToStartScreen, startCharacter } from '../helpers.js';
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

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

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
    // tabs, no document status, no validation-mode toggle, no save/export bar.
    expect(await $(TAB_BAR).isExisting()).toBe(false);
    expect(await $('[data-testid="doc-status"]').isExisting()).toBe(false);
    expect(await $('[data-testid="mode-select"]').isExisting()).toBe(false);
    expect(await $('[data-testid="save-button"]').isExisting()).toBe(false);

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
    await $('[data-testid="new-button"]').click();
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
    await $('[data-testid="new-button"]').click();
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
    await $('[data-testid="save-button"]').click();
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
