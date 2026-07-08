// End-to-end: Mythic Companion types (mythic-companion-only). Choosing the
// character type surfaces a "Type" tab (gated on the profile's
// `has_mythic_type` flag); picking a type auto-grants its free status + Minor
// Virtue (read-only), seeds its required V/F package as bought selections, and
// raises the balance ceilings by the type's bonus points (Devil Child 37/17).
// A required Flaw is swappable for a substitute. All against the real binary.
//
// NOTE: requires a display + the production binary (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, expect, browser } from '@wdio/globals';

const TYPE_SELECT = '[data-testid="type-select"]';
const MYTHIC_TAB = '[data-testid="tab-mythic_type"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const MYTHIC_SELECT = '[data-testid="mythic-type-select"]';
const VIRTUE_BUDGET = '[data-testid="balance-virtues"]';
const FLAW_BUDGET = '[data-testid="balance-flaws"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function setType(value) {
  await $(TYPE_SELECT).selectByAttribute('value', value);
}

async function selectMythicType(value) {
  await $(MYTHIC_TAB).click();
  const select = await $(MYTHIC_SELECT);
  await select.waitForExist({ timeout: 10000 });
  await select.selectByAttribute('value', value);
}

describe('mythic companion types', () => {
  it('shows the Type tab only for a mythic companion', async () => {
    await setType('companion');
    await expect($(MYTHIC_TAB)).not.toExist();
    await setType('mythic_companion');
    await $(MYTHIC_TAB).waitForExist({ timeout: 10000 });
  });

  it('grants the free status + Minor Virtue and raises the budget to the type bonus', async () => {
    await setType('mythic_companion');
    await selectMythicType('mythic_type.devil_child');

    // On the Type tab: the free status Virtue is a read-only grant row, and the
    // free Minor is a defaulted choice (Demonic Might/Powers).
    await $('[data-testid="mythic-granted-virtue.devil_child"]').waitForExist({ timeout: 10000 });
    await expect($('[data-testid="mythic-choice-devil_child_free_minor"]')).toExist();

    // The balance bar lives on the Virtues & Flaws tab: Devil Child raises the
    // ceilings to 37 V / 17 F (base 20/10 + 7·2 + 3 free / +7 F).
    await $(VF_TAB).click();
    await browser.waitUntil(async () => clean(await $(VIRTUE_BUDGET).getText()).includes('/ 37'), {
      timeout: 5000,
      timeoutMsg: 'virtue budget not raised to 37',
    });
    expect(clean(await $(FLAW_BUDGET).getText())).toContain('/ 17');
  });

  it('seeds the required package and offers a required-Flaw substitute', async () => {
    await setType('mythic_companion');
    await selectMythicType('mythic_type.devil_child');

    // The required Flaw swap dropdown is on the Type tab, with >1 substitute.
    const swap = await $('[data-testid="mythic-required-flaw-flaw.tragic_life"]');
    await swap.waitForExist({ timeout: 10000 });
    expect((await swap.$$('option')).length).toBeGreaterThan(1);

    // The required package is seeded as bought selections, shown on the V&F tab
    // (Demonic Blood in the Virtues list; Tragic Life in the Flaws list).
    await $(VF_TAB).click();
    await $('[data-testid="selection-list-virtue"]').waitForExist({ timeout: 10000 });
    expect(await $('[data-testid="selection-list-virtue"]').getText()).toContain('Demonic Blood');
    expect(await $('[data-testid="selection-list-flaw"]').getText()).toContain('Tragic Life');
  });
});
