// End-to-end: mutually exclusive Virtues/Flaws cannot both be picked in Enforced
// mode — the counterpart's Add button greys out — while Advisory leaves the pick
// open and reports the clash as an issue instead. Covers both flavours of
// exclusion: a hand-authored clique (Gentle vs Blatant Gift) and a Major/Minor
// magnitude pair of the same Flaw (Ambitious). Drives the real binary.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md).

import { $, browser, expect } from '@wdio/globals';

const MODE_SELECT = '[data-testid="mode-select"]';
const INCOMPATIBLE_ISSUE = '[data-testid="issue-list"] li[data-code="incompatible"]';

async function addButton(ref) {
  const button = await $(`[data-testid="add-${ref}"]`);
  await button.waitForExist({ timeout: 10000 });
  return button;
}

async function waitForEnabled(ref, enabled) {
  const button = await addButton(ref);
  await browser.waitUntil(async () => (await button.isEnabled()) === enabled, {
    timeout: 5000,
    timeoutMsg: `expected add-${ref} to be ${enabled ? 'enabled' : 'greyed out'}`,
  });
}

describe('mutually exclusive Virtues/Flaws', () => {
  before(async () => {
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();
  });

  it('greys out an excluded counterpart in Enforced mode, but not in Advisory', async () => {
    // Gentle Gift and Blatant Gift exclude each other via `incompatible_with`.
    const blatant = await addButton('flaw.blatant_gift');
    expect(await blatant.isEnabled()).toBe(true);
    await blatant.click();

    // Enforced (default): the counterpart is no longer takeable, so the illegal
    // combination cannot be reached by clicking at all.
    await waitForEnabled('virtue.gentle_gift', false);

    // Advisory: the pick is allowed again and the clash is reported instead.
    const modeSelect = await $(MODE_SELECT);
    await modeSelect.selectByAttribute('value', 'advisory');
    await waitForEnabled('virtue.gentle_gift', true);
    await (await addButton('virtue.gentle_gift')).click();
    await $(INCOMPATIBLE_ISSUE).waitForExist({ timeout: 5000 });

    // Back to Enforced: the same state now reports at error severity.
    await modeSelect.selectByAttribute('value', 'enforced');
    await browser.waitUntil(
      async () => (await $(INCOMPATIBLE_ISSUE).getAttribute('data-severity')) === 'error',
      { timeout: 5000, timeoutMsg: 'expected an error-severity incompatibility issue' },
    );

    // Clear both picks so the magnitude-pair case starts from a clean sheet.
    for (const ref of ['virtue.gentle_gift', 'flaw.blatant_gift']) {
      const remove = await $(`[data-testid^="remove-${ref}-"]`);
      await remove.waitForExist({ timeout: 5000 });
      await remove.click();
    }
    await $('[data-testid="no-issues"]').waitForExist({ timeout: 5000 });
  });

  it('greys out the Major variant of an already selected Minor Flaw', async () => {
    // A character may take only one magnitude of the same Virtue/Flaw.
    await (await addButton('flaw.ambitious_minor')).click();
    await waitForEnabled('flaw.ambitious_major', false);

    // Removing the Minor variant frees the Major one again.
    const remove = await $('[data-testid^="remove-flaw.ambitious_minor-"]');
    await remove.waitForExist({ timeout: 5000 });
    await remove.click();
    await waitForEnabled('flaw.ambitious_major', true);
  });
});
