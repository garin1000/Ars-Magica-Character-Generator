// End-to-end: Hermetic Arts (magus-only). The Arts tab appears only for a magus,
// an Art is bought against the shared XP pool, and Puissant Art adds +3 to the
// targeted Art's effective score — all against the real binary.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, expect, browser } from '@wdio/globals';

import { clean, startCharacter } from '../helpers.js';

const ARTS_TAB = '[data-testid="tab-arts"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const LANG_SELECT = '[data-testid="language-select"]';

describe('hermetic arts', () => {
  it('shows the Arts tab only for a magus', async () => {
    // A companion has no Arts tab...
    await startCharacter('companion');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    expect(await $(ARTS_TAB).isExisting()).toBe(false);

    // ...while a magus does (capability flag, not the type id).
    await startCharacter('magus');
    await $(ARTS_TAB).waitForExist({ timeout: 5000 });
  });

  it('buys an Art against the shared XP pool', async () => {
    await startCharacter('magus');
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

  // Regression: the bar must show an OVERSPENT pool as a negative Available. This
  // needs the real binary — `xp_general_used` alone can never express it (it is a
  // max-flow value capped by the pool), so a unit test with a hand-written
  // `general_used > pool` payload asserts a state the engine cannot produce and
  // passes even when the app shows 0. Only the live IPC payload proves it.
  it('shows an overspent XP pool as a negative Available', async () => {
    // Deliberately NO startCharacter here: this test carries on from the magus the
    // previous one funded, because reaching an overspent pool needs that state.
    await $(ARTS_TAB).click();

    // Carrying on from the previous test: pool 20, Creo 5 (15 XP). Raising Creo to
    // 7 costs 28 XP (triangular), so the 20-point pool is overspent by 8.
    const inc = await $('[data-testid="art-inc-art.creo"]');
    await inc.waitForExist({ timeout: 5000 });
    for (let i = 0; i < 2; i++) await inc.click();

    const spent = await $('[data-testid="art-xp-spent"]');
    const available = await $('[data-testid="art-xp-available"]');
    await browser.waitUntil(
      async () =>
        clean(await spent.getText()).includes('28') &&
        // ASCII hyphen-minus, never U+2212.
        clean(await available.getText()).includes('-8'),
      {
        timeout: 5000,
        timeoutMsg: 'Creo 7 (28 XP) against a 20-point pool should read Available: -8',
      },
    );

    // Both figures carry the stable semantic signal (styling classes can change
    // freely without breaking this assertion).
    expect(await spent.getAttribute('data-overspent')).toBe('true');
    expect(await available.getAttribute('data-overspent')).toBe('true');

    // Restore a legal pool so later specs start from a funded state.
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.setValue('40');
    await browser.waitUntil(async () => clean(await available.getText()).includes('12'), {
      timeout: 5000,
      timeoutMsg: 'a 40-point pool should cover Creo 7 (28 XP) with 12 left',
    });
  });

  it('applies Puissant Art as a +3 effective bonus on the targeted Art', async () => {
    await startCharacter('magus');

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
    await startCharacter('magus');
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
