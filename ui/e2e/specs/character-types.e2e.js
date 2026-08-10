// End-to-end: a character's type is fixed at creation, and the type it was
// created as is what drives its profile — budget and category rules alike.
// Verifies the shipped types (grog, companion, mythic companion, magus) produce
// different validation against the real binary, and that the editor offers no way
// to change the type afterwards.
//
// This file used to test the header's character-type SELECTOR, switching type in
// place on one character. That control no longer exists (M6a): the type is a
// creation-time choice, so each type is now built as its own character.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, $$, expect, browser } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const VIRTUE_BUDGET = '[data-testid="balance-virtues"]';
const FLAW_BUDGET = '[data-testid="balance-flaws"]';
const CHARACTER_TYPE = '[data-testid="character-type"]';
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

/** Create a character of `type` and open its Virtues & Flaws tab (the balance bar). */
async function startOnVfTab(type) {
  await startCharacter(type);
  const vfTab = await $(VF_TAB);
  await vfTab.waitForExist({ timeout: 30000 });
  await vfTab.click();
  await $(VIRTUE_BUDGET).waitForExist({ timeout: 10000 });
}

describe('character types', () => {
  it('gives each created type its own profile budget in the balance bar', async () => {
    // Companion: 10 / 10.
    await startOnVfTab('companion');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 10'), {
      timeout: 5000,
      timeoutMsg: 'companion virtue budget not 10',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 10');

    // Grog: 3 / 3.
    await startOnVfTab('grog');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 3'), {
      timeout: 5000,
      timeoutMsg: 'grog virtue budget not 3',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 3');

    // Mythic Companion: 20 virtue points (2:1 funding) / 10 flaw points.
    await startOnVfTab('mythic_companion');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 20'), {
      timeout: 5000,
      timeoutMsg: 'mythic companion virtue budget not 20',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 10');
  });

  it('applies the created type category rules to the same selection', async () => {
    // As a companion, the Hermetic flaw is a forbidden-category error.
    await startOnVfTab('companion');
    const addHermetic = await $(HERMETIC_FLAW);
    await addHermetic.waitForExist({ timeout: 10000 });
    await addHermetic.click();
    await browser.waitUntil(async () => (await codes()).includes('forbidden_category'), {
      timeout: 5000,
      timeoutMsg: 'companion should forbid the Hermetic flaw',
    });

    // As a magus, the Hermetic category is permitted: the same pick raises no
    // forbidden-category error, and a magus-specific issue (no House chosen yet →
    // `house_unset`) appears — a positive signal that this really is the magus
    // profile, not just an absence. (The required Hermetic Magus status is seeded
    // at creation, so `missing_required_trait` does not fire either.)
    await startOnVfTab('magus');
    const addAsMagus = await $(HERMETIC_FLAW);
    await addAsMagus.waitForExist({ timeout: 10000 });
    await addAsMagus.click();
    await browser.waitUntil(
      async () => {
        const seen = await codes();
        return !seen.includes('forbidden_category') && seen.includes('house_unset');
      },
      { timeout: 5000, timeoutMsg: 'the magus profile rules did not apply to the new character' },
    );
  });

  it('shows the type as a read-only label, with no selector to change it', async () => {
    await startCharacter('grog');

    // The banner names the type through its Fluent key, never the raw slug...
    const label = await $(CHARACTER_TYPE);
    await label.waitForExist({ timeout: 10000 });
    const text = clean(await label.getText());
    expect(text).toContain('Grog');
    expect(text).not.toContain('type-grog');

    // ...and it is a label, not a control: the header selector this file used to
    // drive is gone, and nothing editable replaced it.
    expect(await $('[data-testid="type-select"]').isExisting()).toBe(false);
    expect(
      (await $$(`${CHARACTER_TYPE} select, ${CHARACTER_TYPE} input, ${CHARACTER_TYPE} button`))
        .length,
    ).toBe(0);
  });
});
