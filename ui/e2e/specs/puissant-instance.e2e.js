// End-to-end: Puissant (Ability) targets one specific instance of a
// parameterized ability ((Area) Lore). The +2 must attach to exactly the chosen
// area's row and not bleed onto the character's other areas. Drives the real
// binary; assertions read the rendered effective-score badges.

import { $, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

// Add an (Area) Lore instance at row `i`, name its area, and raise it to `score`.
async function addLore(i, area, score) {
  await $('[data-testid="add-ability.area_lore"]').click();
  const param = await $(`[data-testid="ability-param-ability.area_lore-${i}"]`);
  await param.waitForExist({ timeout: 5000 });
  await param.setValue(area);
  const inc = await $(`[data-testid="ability-inc-ability.area_lore-${i}"]`);
  for (let n = 0; n < score; n++) await inc.click();
}

describe('Puissant Ability targets one ability instance', () => {
  it('boosts only the chosen (Area) Lore, not the others', async () => {
    // The row indices below are the ability-array positions, so this needs a
    // character with no abilities yet.
    await startCharacter('companion');

    // Abilities tab: fund XP, then add three distinct Area Lore instances.
    const abilitiesTab = await $('[data-testid="tab-abilities"]');
    await abilitiesTab.waitForExist({ timeout: 30000 });
    await abilitiesTab.click();
    await $('[data-testid="xp-pool"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="xp-pool"]').setValue(100);

    await addLore(0, 'Brandenburg', 2);
    await addLore(1, 'Berlin', 2);
    await addLore(2, 'Bavaria', 1);
    expect(await $('[data-testid="ability-score-ability.area_lore-0"]').getText()).toBe('2');

    // Virtues tab: add Puissant twice, targeting Brandenburg and Bavaria.
    await $('[data-testid="tab-virtues_flaws"]').click();
    const add = await $('[data-testid="add-virtue.puissant_ability"]');
    await add.waitForExist({ timeout: 10000 });
    await add.click();
    await add.click();

    const first = await $('[data-testid="param-virtue.puissant_ability-ability-0"]');
    await first.waitForExist({ timeout: 5000 });
    await first.selectByVisibleText('Brandenburg Lore');
    const second = await $('[data-testid="param-virtue.puissant_ability-ability-1"]');
    await second.selectByVisibleText('Bavaria Lore');

    // Back on Abilities: the two targeted instances show their score + 2, and the
    // untargeted Berlin row has no effective badge at all.
    await $('[data-testid="tab-abilities"]').click();
    const brandenburg = await $('[data-testid="ability-eff-ability.area_lore-0"]');
    await brandenburg.waitForExist({ timeout: 10000 });
    expect(await brandenburg.getText()).toContain('4'); // 2 + 2

    const bavaria = await $('[data-testid="ability-eff-ability.area_lore-2"]');
    await bavaria.waitForExist({ timeout: 10000 });
    expect(await bavaria.getText()).toContain('3'); // 1 + 2

    // Berlin was never targeted: no Puissant bonus bled onto it.
    expect(await $('[data-testid="ability-eff-ability.area_lore-1"]').isExisting()).toBe(false);
  });
});
