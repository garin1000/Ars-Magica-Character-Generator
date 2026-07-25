// End-to-end: Hermetic Arts (magus-only). The Arts tab appears only for a magus,
// an Art is bought against the shared XP pool, and Puissant Art adds +3 to the
// targeted Art's effective score — all against the real binary.
//
// NOTE: requires a display + the production binary (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, expect, browser } from '@wdio/globals';

const TYPE_SELECT = '[data-testid="type-select"]';
const ARTS_TAB = '[data-testid="tab-arts"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const LANG_SELECT = '[data-testid="language-select"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function setType(value) {
  await $(TYPE_SELECT).selectByAttribute('value', value);
}

describe('hermetic arts', () => {
  it('shows the Arts tab only for a magus', async () => {
    await $(VF_TAB).waitForExist({ timeout: 30000 });

    // Companion (default) has no Arts tab.
    await setType('companion');
    await browser.waitUntil(async () => !(await $(ARTS_TAB).isExisting()), {
      timeout: 5000,
      timeoutMsg: 'companion must not show the Arts tab',
    });

    // Magus reveals it (capability flag, not the type id).
    await setType('magus');
    await $(ARTS_TAB).waitForExist({ timeout: 5000 });
  });

  it('buys an Art against the shared XP pool', async () => {
    await setType('magus');
    await $(ARTS_TAB).click();

    // Fund the shared pool. All 15 Arts are always shown — raise Creo to 5
    // directly (no pick step); triangular cost = 15 XP.
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('20');

    const inc = await $('[data-testid="art-inc-art.creo"]');
    await inc.waitForExist({ timeout: 5000 });
    for (let i = 0; i < 5; i++) await inc.click();

    await browser.waitUntil(
      async () => clean(await $('[data-testid="art-xp-spent"]').getText()).includes('15'),
      { timeout: 5000, timeoutMsg: 'Creo 5 should cost 15 Art XP' },
    );
    expect(clean(await $('[data-testid="art-xp-available"]').getText())).toContain('5');
  });

  it('applies Puissant Art as a +3 effective bonus on the targeted Art', async () => {
    await setType('magus');

    // Add Puissant Art on the Virtues & Flaws tab and target Ignem.
    await $(VF_TAB).click();
    const addPuissant = await $('[data-testid="add-virtue.puissant_art"]');
    await addPuissant.waitForExist({ timeout: 10000 });
    await addPuissant.click();
    const target = await $('[data-testid^="param-virtue.puissant_art-art-"]');
    await target.waitForExist({ timeout: 5000 });
    await target.selectByAttribute('value', 'art.ignem');

    // Raise Ignem to 2 on the Arts tab → effective 2 + 3 = 5.
    await $(ARTS_TAB).click();
    const inc = await $('[data-testid="art-inc-art.ignem"]');
    await inc.waitForExist({ timeout: 5000 });
    await inc.click();
    await inc.click();

    const eff = await $('[data-testid="art-eff-art.ignem"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('5'), {
      timeout: 5000,
      timeoutMsg: 'Puissant Art should make Ignem 2 read as effective 5',
    });
  });

  it('shows the single-row XP summary and localizes it to German (Issue G)', async () => {
    await setType('magus');
    await $(ARTS_TAB).click();

    // The general-pool total is the only editable field and retains its value.
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('30');
    await browser.waitUntil(async () => (await pool.getValue()) === '30', {
      timeout: 5000,
      timeoutMsg: 'the editable general-pool total should retain the entered value',
    });

    // English: the Available readout renders in the same row with its label.
    const available = await $('[data-testid="art-xp-available"]');
    await available.waitForExist({ timeout: 5000 });
    expect(clean(await available.getText())).toContain('Available');

    // German: switching the language re-localizes the same XP row.
    await $(LANG_SELECT).selectByAttribute('value', 'de');
    await browser.waitUntil(async () => clean(await available.getText()).includes('Verfügbar'), {
      timeout: 5000,
      timeoutMsg: 'the XP row Available label should localize to German',
    });

    // Restore English so later specs run against the default locale.
    await $(LANG_SELECT).selectByAttribute('value', 'en');
  });
});
