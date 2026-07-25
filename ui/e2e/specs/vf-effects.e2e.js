// End-to-end: Phase-3 virtue/flaw mechanical effects, against the real binary —
// Improved Characteristics (+3 buy budget), restricted XP pools (Warrior),
// Affinity (reduced Art XP), and an ability_score_grant floor (Second Sight 1).
//
// NOTE: requires a display + the production binary (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, expect, browser } from '@wdio/globals';

const TYPE_SELECT = '[data-testid="type-select"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const CHARS_TAB = '[data-testid="tab-characteristics"]';
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
const ARTS_TAB = '[data-testid="tab-arts"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function setType(value) {
  await $(TYPE_SELECT).selectByAttribute('value', value);
}

async function addVirtue(id) {
  await $(VF_TAB).click();
  const add = await $(`[data-testid="add-${id}"]`);
  await add.waitForExist({ timeout: 10000 });
  await add.click();
}

describe('phase-3 virtue/flaw effects', () => {
  it('Improved Characteristics raises the Characteristic-buy budget by 3', async () => {
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await setType('companion');

    // Base budget is 7.
    await $(CHARS_TAB).click();
    const points = await $('[data-testid="characteristic-points"]');
    await points.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => clean(await points.getText()).includes('/ 7'), {
      timeout: 5000,
      timeoutMsg: 'base Characteristic budget should be 7',
    });

    await addVirtue('virtue.improved_characteristics');

    await $(CHARS_TAB).click();
    await browser.waitUntil(async () => clean(await points.getText()).includes('/ 10'), {
      timeout: 5000,
      timeoutMsg: 'Improved Characteristics should raise the budget to 10',
    });
  });

  it('Warrior adds a restricted XP pool eligible for Martial Abilities', async () => {
    await setType('companion');
    await addVirtue('virtue.warrior');

    await $(ABILITIES_TAB).click();
    const pool = await $('[data-testid="restricted-xp-0"]');
    await pool.waitForExist({ timeout: 10000 });
    // 50 restricted points, none yet spent.
    await browser.waitUntil(async () => clean(await pool.getText()).includes('0 / 50'), {
      timeout: 5000,
      timeoutMsg: 'Warrior should show a 0 / 50 restricted pool',
    });

    // Buy a Martial Ability at 1 (5 XP): the Warrior pool funds it, so none of it
    // draws on the general pool. In the single-row layout the restricted sub-budget
    // rises to "5 / 50" while the read-only general-used figure stays 0 — the general
    // and restricted portions are shown separately, not merged into one "Spent".
    const add = await $('[data-testid="add-ability.single_weapon"]');
    await add.waitForExist({ timeout: 5000 });
    await add.click();
    const inc = await $('[data-testid="ability-inc-ability.single_weapon-0"]');
    await inc.waitForExist({ timeout: 5000 });
    await inc.click();

    await browser.waitUntil(async () => clean(await pool.getText()).includes('5 / 50'), {
      timeout: 5000,
      timeoutMsg: 'buying a Martial Ability should draw on the Warrior pool',
    });
    await browser.waitUntil(
      async () => clean(await $('[data-testid="xp-spent"]').getText()).trim() === '0',
      {
        timeout: 5000,
        timeoutMsg: 'the general-used figure should stay 0 when the Warrior pool funds the spend',
      },
    );

    // The suite shares one app instance and setType does not reset the entity, so
    // clean up the bought Ability to leave the next test's state pristine.
    await $('[data-testid="remove-ability.single_weapon-0"]').click();
    await browser.waitUntil(async () => clean(await pool.getText()).includes('0 / 50'), {
      timeout: 5000,
      timeoutMsg: 'removing the Martial Ability should free the Warrior pool again',
    });
  });

  it('Second Sight confers the Ability at a free effective floor of 1', async () => {
    await setType('companion');
    await addVirtue('virtue.second_sight');

    // Add the Second Sight Ability at 0; the grant floors its effective score at 1.
    await $(ABILITIES_TAB).click();
    const add = await $('[data-testid="add-ability.second_sight"]');
    await add.waitForExist({ timeout: 10000 });
    await add.click();

    const eff = await $('[data-testid="ability-eff-ability.second_sight-0"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('1'), {
      timeout: 5000,
      timeoutMsg: 'Second Sight should read as effective 1 at score 0',
    });
    // The bought score is still 0 — the grant cost no experience.
    expect(await $('[data-testid="ability-score-ability.second_sight-0"]').getText()).toBe('0');
  });

  it('Affinity with Art reduces the XP charged for that Art', async () => {
    await setType('magus');

    // Affinity with Creo, then raise Creo to 5 (table cost 15 → charged 10).
    await addVirtue('virtue.affinity_art');
    const target = await $('[data-testid^="param-virtue.affinity_art-art-"]');
    await target.waitForExist({ timeout: 5000 });
    await target.selectByAttribute('value', 'art.creo');

    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('30');
    const inc = await $('[data-testid="art-inc-art.creo"]');
    await inc.waitForExist({ timeout: 5000 });
    for (let i = 0; i < 5; i++) await inc.click();

    await browser.waitUntil(
      async () => clean(await $('[data-testid="art-xp-spent"]').getText()).includes('10'),
      { timeout: 5000, timeoutMsg: 'Affinity should reduce Creo 5 from 15 to 10 XP' },
    );
  });
});
