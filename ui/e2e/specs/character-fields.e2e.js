// End-to-end: per-character fields (Phase 7). A "Details" tab (always present)
// holds age, a read-only Confidence readout, Personality Traits, and Reputations;
// the age → Ability cap, the Personality ±3/±6 range, the Reputation grant gate,
// and the Supernatural-Ability greying are all exercised against the real binary.
//
// The wdio session is shared, so the `it` blocks run as one ordered narrative:
// the first creates the companion the rest go on editing, right through to the
// closing save round-trip. Only that first one calls `startCharacter`.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md).

import { $, $$, expect, browser } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { startCharacter } from '../helpers.js';

const DETAILS_TAB = '[data-testid="tab-details"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
const CONFIDENCE = '[data-testid="confidence-readout"]';
const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}
async function codeExists(code) {
  return (await $$(`[data-code="${code}"]`).length) > 0;
}

describe('character details', () => {
  it('shows a Details tab with Confidence for a companion, hidden for a grog', async () => {
    // Grogs have no Confidence — the readout is absent for one entirely.
    await startCharacter('grog');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(DETAILS_TAB).click();
    await browser.waitUntil(async () => !(await $(CONFIDENCE).isExisting()), {
      timeout: 5000,
      timeoutMsg: 'grog must not show a Confidence readout',
    });

    // A companion has it, at the default Score 1 / 3 points. This is also the
    // character the rest of the file goes on editing.
    await startCharacter('companion');
    await $(DETAILS_TAB).click();
    await $(CONFIDENCE).waitForExist({ timeout: 10000 });
    const text = clean(await $(CONFIDENCE).getText());
    expect(text).toContain('1');
    expect(text).toContain('3');
  });

  it('flags an Ability above the age cap', async () => {
    await $(DETAILS_TAB).click();
    await $('[data-testid="age-input"]').setValue('25'); // cap 5
    await $(ABILITIES_TAB).click();
    await $('[data-testid="add-ability.awareness"]').click();
    const inc = await $('[data-testid="ability-inc-ability.awareness-0"]');
    await inc.waitForExist({ timeout: 10000 });
    for (let i = 0; i < 6; i++) await inc.click(); // raise to 6, over the age-25 cap
    await browser.waitUntil(async () => codeExists('ability_above_age_cap'), {
      timeout: 5000,
      timeoutMsg: 'expected ability_above_age_cap for score 6 at age 25',
    });
  });

  it('enforces the Personality Trait range, widened by a Major Personality Flaw', async () => {
    await $(DETAILS_TAB).click();
    await $('[data-testid="personality-add"]').click();
    const inc = await $('[data-testid="personality-inc-0"]');
    for (let i = 0; i < 4; i++) await inc.click(); // +4, beyond the ±3 default
    await browser.waitUntil(async () => codeExists('personality_trait_out_of_range'), {
      timeout: 5000,
      timeoutMsg: 'a +4 trait should be out of range without a Major Personality Flaw',
    });

    // Pagan is a Major Personality Flaw; it lifts one trait to ±6.
    await $(VF_TAB).click();
    await $('[data-testid="add-flaw.pagan"]').click();
    await browser.waitUntil(async () => !(await codeExists('personality_trait_out_of_range')), {
      timeout: 5000,
      timeoutMsg: 'a Major Personality Flaw should allow the ±6 trait',
    });
  });

  it('greys a Supernatural Ability until the Gift opens a free slot', async () => {
    await $(ABILITIES_TAB).click();
    const secondSight = await $('[data-testid="add-ability.second_sight"]');
    await secondSight.waitForExist({ timeout: 10000 });
    // No Gift, no granting Virtue → locked.
    expect(await secondSight.isEnabled()).toBe(false);

    // Taking The Gift opens the one free Supernatural slot for a companion.
    await $(VF_TAB).click();
    await $('[data-testid="add-virtue.the_gift"]').click();
    await $(ABILITIES_TAB).click();
    await browser.waitUntil(
      async () => await $('[data-testid="add-ability.second_sight"]').isEnabled(),
      { timeout: 5000, timeoutMsg: 'The Gift should unlock one free Supernatural Ability' },
    );
  });

  it('offers Reputation input only once a granting Flaw is taken', async () => {
    await $(DETAILS_TAB).click();
    await expect($('[data-testid="reputation-empty"]')).toExist();

    // Infamous grants a Local Reputation.
    await $(VF_TAB).click();
    await $('[data-testid="add-flaw.infamous"]').click();
    await $(DETAILS_TAB).click();
    const add = await $('[data-testid="reputation-add-local"]');
    await add.waitForExist({ timeout: 5000 });
    await add.click();
    await $('[data-testid="reputation-content-0"]').waitForExist({ timeout: 5000 });
    await $('[data-testid="reputation-content-0"]').setValue('dragon slayer');
  });

  it('edits name and description in the header banner and concept on Details', async () => {
    // Name + short description live in the always-visible header banner.
    await $('[data-testid="identity-name"]').setValue('Marcus of Bonisagus');
    await $('[data-testid="identity-description"]').setValue('Knight of the Teutonic Order');
    // Concept is a multi-line textarea on the Details tab.
    await $(DETAILS_TAB).click();
    await $('[data-testid="identity-concept"]').setValue('A grim knight turned magus.');
  });

  it('round-trips the new fields through a save', async () => {
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.age).toBe(25);
    expect(saved.name).toBe('Marcus of Bonisagus');
    expect(saved.description).toBe('Knight of the Teutonic Order');
    expect(saved.concept).toBe('A grim knight turned magus.');
    expect(saved.personality_traits.length).toBeGreaterThan(0);
    expect(saved.reputations.some((r) => r.kind === 'local' && r.content === 'dragon slayer')).toBe(
      true,
    );
  });
});
