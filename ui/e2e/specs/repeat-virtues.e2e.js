// End-to-end: a repeatable parameterized virtue (Great Characteristic) can be
// added several times, each instance keeps its own chosen target, and once all
// targets are set the "missing parameter" error clears. Reproduces the report
// that the error stuck around even after every instance had a Characteristic.

import { $, $$, browser, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

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
