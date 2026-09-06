// End-to-end: a repeatable parameterized virtue (Great Characteristic) can be
// added several times, each instance keeps its own chosen target, and once all
// targets are set the "missing parameter" error clears. Reproduces the report
// that the error stuck around even after every instance had a Characteristic.

import { $, $$, browser, expect } from '@wdio/globals';

import { isRowBlocked, startCharacter, STEP_TIMEOUT } from '../helpers.js';

const MISSING_PARAM = 'missing the parameter';

async function errorTexts() {
  const errors = await $$('[data-severity="error"]');
  const texts = [];
  for (let i = 0; i < errors.length; i++) {
    texts.push(await errors[i].getText());
  }
  return texts;
}

describe('repeated parameterized virtues', () => {
  it('keeps a distinct target per Great Characteristic instance', async () => {
    // The instance indices below are the selections-array positions, so this needs
    // a character carrying no Great Characteristic yet.
    await startCharacter('companion');

    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();

    const add = await $('[data-testid="add-virtue.great_characteristic"]');
    await add.waitForExist({ timeout: 10000 });

    // Add six instances; the Add button must stay enabled across repeats.
    const targets = ['int', 'per', 'str', 'sta', 'pre', 'com'];
    for (let i = 0; i < targets.length; i++) {
      await add.click();
    }

    // While unfilled, a missing-parameter error must be reported (and the panel
    // must keep rendering — six identical issues must not crash the keyed list).
    await browser.waitUntil(
      async () => (await errorTexts()).some((t) => t.includes(MISSING_PARAM)),
      {
        timeout: 5000,
        timeoutMsg: 'expected a missing-parameter error while instances are unfilled',
      },
    );

    // Each instance gets its own characteristic (all distinct, so under the cap).
    for (let i = 0; i < targets.length; i++) {
      const param = await $(
        `[data-testid="param-virtue.great_characteristic-characteristic-${i}"]`,
      );
      await param.waitForExist({ timeout: 5000 });
      await param.selectByAttribute('value', `characteristic.${targets[i]}`);
    }

    // The selected value must persist in the DOM for every instance.
    for (let i = 0; i < targets.length; i++) {
      const param = await $(
        `[data-testid="param-virtue.great_characteristic-characteristic-${i}"]`,
      );
      expect(await param.getValue()).toBe(`characteristic.${targets[i]}`);
    }

    // Once every instance has a target, the missing-parameter error must clear.
    await browser.waitUntil(
      async () => !(await errorTexts()).some((t) => t.includes(MISSING_PARAM)),
      {
        timeout: 5000,
        timeoutMsg: 'missing-parameter error did not clear after all targets were set',
      },
    );
  });
});

// Mirror of finding 34's spec above, for the OTHER cap: max_total bounds copies
// of an item TOTAL across every distinct parameter target (Puissant Art, capped
// at two — rules/core/virtues_flaws.json), distinct from max_per_target (one
// copy per identical target, which virtue.great_characteristic above tests and
// which carries no max_total of its own — its Add button must stay enabled
// indefinitely, unaffected by this cap).
//
// A magus, not a companion: Puissant Art is Hermetic-categoried, and the
// companion profile forbids that category (rules/core/character_types.json) —
// the Available row would still render (categories are not filtered there),
// but taking it would raise an unrelated category_not_permitted finding. Magus
// permits Hermetic, so this stays a clean test of the max_total cap alone.
describe('max_total copy cap on the Available list', () => {
  it('greys out Puissant Art once both of its two total copies are taken', async () => {
    await startCharacter('magus');

    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();

    const add = await $('[data-testid="add-virtue.puissant_art"]');
    await add.waitForExist({ timeout: 10000 });

    // The param selects are named `param-virtue.puissant_art-art-{index}`, where
    // {index} is the position in entity.selections — NOT a per-virtue counter. A
    // magus's mandatory free traits (Hermetic Magus + The Gift, seeded on
    // creation from the type profile — see arts.e2e.js) already occupy the first
    // slots, so the two Puissant Art instances land at whatever index follows
    // them, never 0 and 1. Match by prefix and pick by how many exist so far,
    // rather than hardcoding an offset.
    const puissantArtParams = () => $$('[data-testid^="param-virtue.puissant_art-art-"]');

    // First copy, targeting Ignem — one of two, so the row must stay takeable.
    await add.click();
    await browser.waitUntil(async () => (await puissantArtParams()).length === 1, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'expected one Puissant Art param select after the first Add click',
    });
    const firstArt = (await puissantArtParams())[0];
    await firstArt.selectByAttribute('value', 'art.ignem');
    expect(await isRowBlocked(add)).toBe(false);

    // Second copy, targeting Perdo — a DIFFERENT target, so max_per_target (one
    // copy per identical target) never fires; only max_total (two total) does.
    await add.click();
    await browser.waitUntil(async () => (await puissantArtParams()).length === 2, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'expected two Puissant Art param selects after the second Add click',
    });
    const secondArt = (await puissantArtParams())[1];
    await secondArt.selectByAttribute('value', 'art.perdo');

    await browser.waitUntil(async () => isRowBlocked(add), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Puissant Art did not grey out once both of its two total copies were taken',
    });
  });
});
