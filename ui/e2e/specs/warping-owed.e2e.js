// End-to-end (Issue E): the off-budget Virtues/Flaws a character owes from its
// Warping Score ("Effects of Warping", Core:16547-16561). A warped non-magus
// surfaces one picker per owed slot on the Details tab; a magus (exempt — Twilight
// instead) never shows the section, even at the same Warping Score.
//
// Drives the real production binary. Runs last alphabetically and restores the
// type to companion at the end so it leaves no state behind.
//
// NOTE: requires a display + the production binary (see e2e/README.md).

import { $, browser } from '@wdio/globals';

const TYPE_SELECT = '[data-testid="type-select"]';
const DETAILS_TAB = '[data-testid="tab-details"]';
const WARPING_POINTS = '[data-testid="warping-points-input"]';
const WARPING_OWED = '[data-testid="warping-owed"]';
const MINOR_FLAW_FILL = '[data-testid="warping-fill-warping.minor_flaw.0"]';

async function setType(value) {
  await $(TYPE_SELECT).selectByAttribute('value', value);
}

describe('warping-owed Virtues/Flaws', () => {
  it('surfaces an owed Minor Flaw picker for a warped non-magus', async () => {
    await $(TYPE_SELECT).waitForExist({ timeout: 30000 });
    await setType('companion');
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

  it('hides the section for a magus at the same Warping Score', async () => {
    await setType('magus');
    await $(DETAILS_TAB).click();
    await browser.waitUntil(async () => !(await $(WARPING_OWED).isExisting()), {
      timeout: 10000,
      timeoutMsg: 'a magus must not show the owed-warping section (Twilight instead)',
    });

    // Restore a clean companion with no Warping so later runs start fresh.
    await setType('companion');
    await $(DETAILS_TAB).click();
    await $(WARPING_POINTS).setValue('0');
  });
});
