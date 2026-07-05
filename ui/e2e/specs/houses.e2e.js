// End-to-end: Hermetic Houses (magus-only). Selecting a House auto-grants its
// free Virtue — free of the point budget — Mystery Houses seed a Supernatural
// Ability at an effective floor of 1, a `choice` grant offers its options and an
// unresolved pick raises a localized issue, and an `open` grant is additive (it
// never charges the bought budget). All against the real binary.
//
// NOTE: requires a display + the production binary (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`.
//
// Coverage boundary (mirrors validation-errors.e2e.js): two Phase-4 facts are
// NOT click-reachable in the shipped `rules/core/` seed, so they are asserted by
// the Rust unit/integration tests (plan step 10), not here:
//   * The `too_many_major_hermetic_virtues` cap TRIP — the seed has exactly one
//     Major Hermetic Virtue (`virtue.gentle_gift`, non-repeatable), so a second
//     bought one cannot be reached through clicks. What IS reachable and checked
//     below: a House's granted (Minor) Virtue never contributes to the cap.
//   * Ex Miscellanea's `ex_misc_major_virtue` open pick — the seed has no Major
//     non-Hermetic Virtue, so that picker is empty. The reachable Ex-Misc fact
//     checked below is that an `open` grant is additive to the bought budget.

import { $, expect, browser } from '@wdio/globals';

const TYPE_SELECT = '[data-testid="type-select"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
const ARTS_TAB = '[data-testid="tab-arts"]';
const HOUSE_TAB = '[data-testid="tab-house_specialisation"]';
const HOUSE_SELECT = '[data-testid="house-select"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function setType(value) {
  await $(TYPE_SELECT).selectByAttribute('value', value);
}

async function selectHouse(value) {
  await $(HOUSE_TAB).click();
  const select = await $(HOUSE_SELECT);
  await select.waitForExist({ timeout: 10000 });
  await select.selectByAttribute('value', value);
}

// Add an Ability from the picker (grant-seeded abilities must be added before
// the abilities view can show their granted effective floor — same as the
// Second Sight grant in vf-effects.e2e.js).
async function addAbility(id) {
  await $(ABILITIES_TAB).click();
  const add = await $(`[data-testid="add-${id}"]`);
  await add.waitForExist({ timeout: 10000 });
  await add.click();
}

describe('hermetic houses', () => {
  it('shows the House tab only for a magus', async () => {
    await $(VF_TAB).waitForExist({ timeout: 30000 });

    // Companion (default) has no House tab (gated on the capability flag).
    await setType('companion');
    await browser.waitUntil(async () => !(await $(HOUSE_TAB).isExisting()), {
      timeout: 5000,
      timeoutMsg: 'companion must not show the House tab',
    });

    // Magus reveals it, and the HouseSelector renders behind it.
    await setType('magus');
    await $(HOUSE_TAB).waitForExist({ timeout: 5000 });
    await $(HOUSE_TAB).click();
    await $('[data-testid="house-selector"]').waitForExist({ timeout: 5000 });
    await $(HOUSE_SELECT).waitForExist({ timeout: 5000 });
  });

  it('grants a fixed Virtue for free — Bjornaer confers Heartbeast', async () => {
    await setType('magus');

    // Record the bought-virtue balance BEFORE choosing a House. A granted Virtue
    // must not change it (compute_balance counts bought selections only).
    await $(VF_TAB).click();
    const balance = await $('[data-testid="balance-virtues"]');
    await balance.waitForExist({ timeout: 10000 });
    const before = clean(await balance.getText());

    await selectHouse('house.bjornaer');

    // The free Virtue shows read-only in the House panel…
    const granted = await $('[data-testid="house-granted-virtue.heartbeast"]');
    await granted.waitForExist({ timeout: 5000 });

    // …and its ability_score_grant floors Heartbeast at effective 1 for 0 XP.
    // Match by testid prefix: the index suffix is the ability-array position,
    // which depends on what else was added earlier in the session.
    await addAbility('ability.heartbeast');
    const eff = await $('[data-testid^="ability-eff-ability.heartbeast-"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('1'), {
      timeout: 5000,
      timeoutMsg: 'granted Heartbeast should read as effective 1',
    });
    expect(await $('[data-testid^="ability-score-ability.heartbeast-"]').getText()).toBe('0');

    // The bought-virtue balance is unchanged — the grant is free and, being a
    // Minor Virtue, never touches the Major-Hermetic-Virtue cap either.
    await $(VF_TAB).click();
    expect(clean(await balance.getText())).toBe(before);
  });

  it('seeds a Mystery Ability at an effective floor of 1 — Merinita → Faerie Magic', async () => {
    await setType('magus');
    await selectHouse('house.merinita');

    await addAbility('ability.faerie_magic');
    // Match by testid prefix — a prior test already occupies index 0.
    const eff = await $('[data-testid^="ability-eff-ability.faerie_magic-"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('1'), {
      timeout: 5000,
      timeoutMsg: 'granted Faerie Magic should read as effective 1',
    });
    // No experience was charged — the bought score stays 0.
    expect(await $('[data-testid^="ability-score-ability.faerie_magic-"]').getText()).toBe('0');
  });

  it('offers a choice grant, flags an unresolved pick, and applies the chosen bonus', async () => {
    await setType('magus');
    await selectHouse('house.flambeau');

    // The choice select offers both Puissant Perdo and Puissant Ignem.
    const choice = await $('[data-testid="house-choice-flambeau_puissant"]');
    await choice.waitForExist({ timeout: 5000 });
    const options = await choice.$$('option');
    const labels = [];
    for (const opt of options) labels.push(clean(await opt.getText()));
    expect(labels.some((l) => l.includes('Perdo'))).toBe(true);
    expect(labels.some((l) => l.includes('Ignem'))).toBe(true);

    // With no pick made, the House flags the choice as unresolved.
    await browser.waitUntil(
      async () => await $('[data-code="house_choice_unresolved"]').isExisting(),
      { timeout: 5000, timeoutMsg: 'an unresolved choice should raise house_choice_unresolved' },
    );

    // Pick Puissant Ignem (match by label, not a positional value).
    const ignemIndex = labels.findIndex((l) => l.includes('Ignem'));
    await choice.selectByIndex(ignemIndex);

    // The issue clears once the choice resolves.
    await browser.waitUntil(
      async () => !(await $('[data-code="house_choice_unresolved"]').isExisting()),
      { timeout: 5000, timeoutMsg: 'resolving the choice should clear the issue' },
    );

    // The granted Puissant adds +3 to Ignem. The effective badge only renders for
    // Arts present in the score list (art_bonuses is keyed by bought Arts), so
    // raise Ignem to 2 first, then it reads effective 2 + 3 = 5.
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('20');
    const inc = await $('[data-testid="art-inc-art.ignem"]');
    await inc.waitForExist({ timeout: 5000 });
    await inc.click();
    await inc.click();

    const artEff = await $('[data-testid="art-eff-art.ignem"]');
    await artEff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await artEff.getText()).includes('5'), {
      timeout: 5000,
      timeoutMsg: 'granted Puissant Ignem should make Ignem 2 read effective 5',
    });
  });

  it('adds an open grant on top of the bought budget — Ex Miscellanea', async () => {
    await setType('magus');

    await $(VF_TAB).click();
    const balance = await $('[data-testid="balance-virtues"]');
    await balance.waitForExist({ timeout: 10000 });
    const before = clean(await balance.getText());

    await selectHouse('house.ex_miscellanea');

    // The Minor-Hermetic-Virtue open picker offers eligible items; pick the first
    // real option (the prompt option carries an empty value).
    const open = await $('[data-testid="house-open-ex_misc_minor_virtue"]');
    await open.waitForExist({ timeout: 5000 });
    const options = await open.$$('option');
    let picked = null;
    for (const opt of options) {
      const value = await opt.getValue();
      if (value) {
        picked = value;
        break;
      }
    }
    expect(picked).not.toBe(null);
    await open.selectByAttribute('value', picked);

    // The granted Virtue is "in addition to the normal allowance" — the bought
    // virtue balance is untouched.
    await $(VF_TAB).click();
    expect(clean(await balance.getText())).toBe(before);
  });
});
