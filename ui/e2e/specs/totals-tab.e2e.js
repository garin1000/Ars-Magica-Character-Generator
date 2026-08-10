// End-to-end: the Totals tab must render for a character who carries the SAME
// weapon twice. `combat_totals` emits one line per equipped slot
// (crates/arm-rules/src/derived.rs:899) and `addEquipment` never dedups, so two
// slots holding one catalogue id produce two combat lines with the same
// `line.weapon`. Keying that `{#each}` by `line.weapon` made Svelte throw
// `each_key_duplicate` — in production as well as dev — which aborted the whole
// panel's render: the tab button lit up, the Totals panel never mounted, and the
// previously selected tab's DOM was never torn down. That is the reported symptom
// ("clicking Totals does nothing"), and only the real binary can catch it: vitest
// renders through `svelte/server`, which does not validate `{#each}` keys.
//
// The spec then equips a shield, which splits every one-handed weapon into a
// with-shield and a bare line — a second, now-ordinary source of duplicate
// `line.weapon` values, and the reason that `{#each}` must stay unkeyed forever.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, $$, browser, expect } from '@wdio/globals';

const LANG_SELECT = '[data-testid="language-select"]';
const TYPE_SELECT = '[data-testid="type-select"]';
const STATUS = '[data-testid="doc-status"]';

const EQUIPMENT_TAB = '[data-testid="tab-equipment"]';
const TOTALS_TAB = '[data-testid="tab-totals"]';

const COMBAT = '[data-testid="derived-combat"]';
const COMBAT_ROWS = '[data-testid="derived-combat"] tbody tr';
const EQUIPMENT_LIST = '[data-testid="equipment-list"]';

// The catalogue weapon carried twice, and its English display name.
const WEAPON = 'weapon.sword_long';
const WEAPON_NAME = 'Long Sword';

// The shield equipped afterwards, splitting each weapon's line in two.
const SHIELD = 'shield.round';
const SHIELD_NAME = 'Round Shield';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

describe('totals tab', () => {
  it('renders the panel when the same weapon is carried twice', async () => {
    await $(TYPE_SELECT).waitForExist({ timeout: 30000 });

    // Specs share one app instance, so this one inherits the previous spec's
    // edits and language. Save first — that clears the dirty flag, so New cannot
    // raise the discard prompt and the reset needs no conditional branch.
    await $(LANG_SELECT).selectByAttribute('value', 'en');
    await browser.waitUntil(async () => (await $(LANG_SELECT).getValue()) === 'en', {
      timeout: 5000,
      timeoutMsg: 'language should switch to en',
    });
    const status = await $(STATUS);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(async () => !clean(await status.getText()).startsWith('*'), {
      timeout: 10000,
      timeoutMsg: 'saving should clear the dirty marker',
    });
    await $('[data-testid="new-button"]').click();
    await browser.waitUntil(
      async () => (await $('[data-testid="identity-name"]').getValue()) === '',
      { timeout: 10000, timeoutMsg: 'New should reset the document' },
    );

    // A magus, as reported — the type that renders the panel's widest surface
    // (Lab/Casting Totals, Penetration, Magic Resistance) above the combat table.
    await $(TYPE_SELECT).selectByAttribute('value', 'magus');

    // Carry the same Long Sword twice. Nothing dedups this, and it is what a real
    // character does: several instances of one catalogue weapon.
    await $(EQUIPMENT_TAB).click();
    const addWeapon = await $(`[data-testid="add-${WEAPON}"]`);
    await addWeapon.waitForExist({ timeout: 10000 });
    await addWeapon.click();
    await addWeapon.click();

    // Two selected slots, both equipped (adding equips by default), so the engine
    // emits two combat lines carrying the same `weapon` id.
    const secondSlot = await $('[data-testid="equipment-name-1"]');
    await secondSlot.waitForExist({ timeout: 10000 });
    expect(clean(await $('[data-testid="equipment-name-0"]').getText())).toBe(WEAPON_NAME);
    expect(clean(await secondSlot.getText())).toBe(WEAPON_NAME);
    expect(await $('[data-testid="equipment-equipped-0"]').isSelected()).toBe(true);
    expect(await $('[data-testid="equipment-equipped-1"]').isSelected()).toBe(true);

    // THE REGRESSION: the Totals panel must actually mount. A duplicate-key throw
    // leaves this waiting forever.
    await $(TOTALS_TAB).click();
    const combat = await $(COMBAT);
    await combat.waitForExist({ timeout: 15000 });

    // Both slots get their own row — the duplicate ids are two real lines, not one.
    await browser.waitUntil(async () => (await $$(COMBAT_ROWS)).length === 2, {
      timeout: 10000,
      timeoutMsg: 'the combat table should hold one row per equipped slot',
    });
    const rowNames = [];
    for (const cell of await $$(`${COMBAT_ROWS} th`)) {
      rowNames.push(clean(await cell.getText()));
    }
    expect(rowNames).toEqual([WEAPON_NAME, WEAPON_NAME]);

    // And the switch really happened: the Equipment tab's content is gone. The
    // failure mode was not a blank panel but the PREVIOUS tab left on screen,
    // because the aborted render never tore its DOM down.
    expect(await $(EQUIPMENT_LIST).isExisting()).toBe(false);
    await expect($('[data-testid="derived-panel"]')).toExist();

    // Equipping a shield doubles every one-handed weapon's lines: with the shield,
    // then bare. That makes `line.weapon` duplicate for a THIRD reason, so this is
    // also the only guard left against the `each_key_duplicate` regression above.
    await $(EQUIPMENT_TAB).click();
    const addShield = await $(`[data-testid="add-${SHIELD}"]`);
    await addShield.waitForExist({ timeout: 10000 });
    await addShield.click();

    await $(TOTALS_TAB).click();
    await $(COMBAT).waitForExist({ timeout: 15000 });
    await browser.waitUntil(async () => (await $$(COMBAT_ROWS)).length === 4, {
      timeout: 10000,
      timeoutMsg: 'each weapon should gain a with-shield row beside its bare one',
    });
    const shieldedRowNames = [];
    for (const cell of await $$(`${COMBAT_ROWS} th`)) {
      shieldedRowNames.push(clean(await cell.getText()));
    }
    const WITH_SHIELD = `${WEAPON_NAME} & ${SHIELD_NAME}`;
    expect(shieldedRowNames).toEqual([WITH_SHIELD, WEAPON_NAME, WITH_SHIELD, WEAPON_NAME]);

    // The panel survived the new duplication source.
    await expect($('[data-testid="derived-panel"]')).toExist();
  });
});
