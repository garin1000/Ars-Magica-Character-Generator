// End-to-end: the character-type selector switches the active profile, and the
// profile's budget + category rules change accordingly. Verifies the four
// shipped types (grog, companion, mythic companion, magus) drive different
// validation against the real binary.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, $$, expect, browser } from '@wdio/globals';

const TYPE_SELECT = '[data-testid="type-select"]';
const VIRTUE_BUDGET = '[data-testid="balance-virtues"]';
const FLAW_BUDGET = '[data-testid="balance-flaws"]';
// `flaw.blatant_gift` is in the Hermetic category: forbidden for a companion,
// permitted for a magus. A deterministic way to observe category rules.
const HERMETIC_FLAW = '[data-testid="add-flaw.blatant_gift"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks (FSI/PDI);
// strip them so plain substring matching on numbers works.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function budget(selector) {
  return clean(await $(selector).getText());
}

async function codes() {
  const items = await $$('[data-testid="issue-list"] li');
  const result = [];
  for (let i = 0; i < items.length; i++) {
    result.push(await items[i].getAttribute('data-code'));
  }
  return result;
}

async function setType(value) {
  const select = await $(TYPE_SELECT);
  await select.selectByAttribute('value', value);
}

describe('character type selector', () => {
  it('switches type and reflects each profile budget in the balance bar', async () => {
    // The balance bar lives in the Virtues & Flaws tab; the type select is in
    // the always-visible header.
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();
    await $(VIRTUE_BUDGET).waitForExist({ timeout: 10000 });

    // Companion (default): 10 / 10.
    await setType('companion');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 10'), {
      timeout: 5000,
      timeoutMsg: 'companion virtue budget not 10',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 10');

    // Grog: 3 / 3.
    await setType('grog');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 3'), {
      timeout: 5000,
      timeoutMsg: 'grog virtue budget not 3',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 3');

    // Mythic Companion: 20 virtue points (2:1 funding) / 10 flaw points.
    await setType('mythic_companion');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 20'), {
      timeout: 5000,
      timeoutMsg: 'mythic companion virtue budget not 20',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 10');
  });

  it('applies the active profile category rules to the same selection', async () => {
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.click();

    // As a companion, the Hermetic flaw is a forbidden-category error.
    await setType('companion');
    const addHermetic = await $(HERMETIC_FLAW);
    await addHermetic.waitForExist({ timeout: 10000 });
    await addHermetic.click();
    await browser.waitUntil(async () => (await codes()).includes('forbidden_category'), {
      timeout: 5000,
      timeoutMsg: 'companion should forbid the Hermetic flaw',
    });

    // As a magus, the Hermetic category is permitted: that error clears, and a
    // magus-specific issue (no House chosen yet → `house_unset`) appears — a
    // positive signal that the switch applied, not just an absence. (The
    // required Hermetic Magus status is auto-added on the switch, so
    // `missing_required_trait` no longer fires — that is the point of the
    // mandatory-trait auto-selection.)
    await setType('magus');
    await browser.waitUntil(
      async () => {
        const seen = await codes();
        return !seen.includes('forbidden_category') && seen.includes('house_unset');
      },
      { timeout: 5000, timeoutMsg: 'magus profile rules did not apply after switching type' },
    );
  });
});
