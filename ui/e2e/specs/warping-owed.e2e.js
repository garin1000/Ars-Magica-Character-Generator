// End-to-end (Issue E): the off-budget Virtues/Flaws a character owes from its
// Warping Score ("Effects of Warping", Core:16547-16561). A warped non-magus
// surfaces one picker per owed slot on the Details tab; a magus (exempt — Twilight
// instead) never shows the section, even at the same Warping Score.
//
// Drives the real production binary. Each block creates its own character, so it
// neither inherits state nor needs to restore any.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md).

import { $, $$, browser, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

const DETAILS_TAB = '[data-testid="tab-details"]';
const WARPING_POINTS = '[data-testid="warping-points-input"]';
const WARPING_OWED = '[data-testid="warping-owed"]';
const MINOR_FLAW_FILL = '[data-testid="warping-fill-warping.minor_flaw.0"]';
// One group per owed kind, labelled with what its slots expect.
const MINOR_FLAW_GROUP = '[data-testid="warping-owed-group-warping-slot-minor-flaw"]';
const VIRTUE_GROUP = '[data-testid="warping-owed-group-warping-slot-supernatural-virtue"]';
const SLOT_SELECT = 'select[data-testid^="warping-fill-"]';
const FORM_PARAM =
  '[data-testid="param-virtue.master_of_form_creatures-form-warping.supernatural_virtue.0"]';
const MISSING_PARAM = '[data-code="missing_param"]';

describe('warping-owed Virtues/Flaws', () => {
  it('surfaces an owed Minor Flaw picker for a warped non-magus', async () => {
    await startCharacter('companion');
    await $(DETAILS_TAB).click();

    // 5 Warping Points → Warping Score 1 → owes one Minor Flaw.
    await $(WARPING_POINTS).waitForExist({ timeout: 10000 });
    await $(WARPING_POINTS).setValue('5');

    await $(WARPING_OWED).waitForExist({
      timeout: 10000,
    });
    await $(MINOR_FLAW_FILL).waitForExist({
      timeout: 10000,
    });

    // Filling the slot resolves cleanly (the fill is off-budget).
    await $(MINOR_FLAW_FILL).selectByAttribute('value', 'flaw.ability_block');
  });

  it('groups the owed slots per kind and resolves a parameterized fill', async () => {
    // Its own companion: the slot counts asserted below must be produced by this
    // test's Warping Score alone, with no fill left over from the previous one.
    await startCharacter('companion');
    await $(DETAILS_TAB).click();

    // 75 Warping Points → Warping Score 5 → owes 2 Minor Flaws + 1 supernatural
    // Minor Virtue (Core:16553-16559), in two labelled groups.
    await $(WARPING_POINTS).waitForExist({ timeout: 10000 });
    await $(WARPING_POINTS).setValue('75');
    await $(MINOR_FLAW_GROUP).waitForExist({ timeout: 10000 });
    await $(VIRTUE_GROUP).waitForExist({ timeout: 10000 });
    // Count only the slot selects: a parameterized pick adds its own control.
    await browser.waitUntil(
      async () => (await $$(`${MINOR_FLAW_GROUP} ${SLOT_SELECT}`)).length === 2,
      { timeout: 10000, timeoutMsg: 'the Minor Flaw group should hold both owed slots' },
    );
    expect((await $$(`${VIRTUE_GROUP} ${SLOT_SELECT}`)).length).toBe(1);

    // Master of (Form) Creatures is a supernatural Minor Virtue whose Form must be
    // named; unnamed, the engine reports the parameter missing.
    const virtueFill = await $(`${VIRTUE_GROUP} ${SLOT_SELECT}`);
    await virtueFill.selectByAttribute('value', 'virtue.master_of_form_creatures');
    await browser.waitUntil(async () => await $(MISSING_PARAM).isExisting(), {
      timeout: 10000,
      timeoutMsg: 'a parameterized owed fill with no parameter should report missing_param',
    });

    const param = await $(FORM_PARAM);
    await param.waitForExist({ timeout: 10000 });
    await param.selectByAttribute('value', 'art.ignem');
    await browser.waitUntil(async () => !(await $(MISSING_PARAM).isExisting()), {
      timeout: 10000,
      timeoutMsg: 'choosing the Form should clear missing_param',
    });
  });

  it('hides the section for a magus at the same Warping Score', async () => {
    // The comparison only means something at the SAME Warping Score, and a freshly
    // created magus starts at 0 — so give this one the previous test's 75 points
    // (Warping Score 5) before asserting the section is absent.
    await startCharacter('magus');
    await $(DETAILS_TAB).click();
    await $(WARPING_POINTS).waitForExist({ timeout: 10000 });
    await $(WARPING_POINTS).setValue('75');
    await browser.waitUntil(async () => (await $(WARPING_POINTS).getValue()) === '75', {
      timeout: 10000,
      timeoutMsg: 'the magus should carry the same 75 Warping Points',
    });

    await browser.waitUntil(async () => !(await $(WARPING_OWED).isExisting()), {
      timeout: 10000,
      timeoutMsg: 'a magus must not show the owed-warping section (Twilight instead)',
    });
  });
});
