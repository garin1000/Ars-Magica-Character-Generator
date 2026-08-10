// End-to-end: Great Characteristic raises a Characteristic's buy cap (it grants
// no free point — the score must be bought past +3), and Puissant Ability (+2)
// shows as an "effective" badge on its target ability. Drives the real binary.

import { $, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';

describe('characteristic cap + ability bonus', () => {
  it('Great Characteristic opens the cap, Puissant shows a badge', async () => {
    await startCharacter('companion');

    // Open the Characteristics tab; raise Strength to its base cap of +3.
    await $('[data-testid="tab-characteristics"]').waitForExist({ timeout: 30000 });
    await $('[data-testid="tab-characteristics"]').click();
    const strInc = await $('[data-testid="char-inc-str"]');
    await strInc.waitForExist({ timeout: 30000 });
    await strInc.click();
    await strInc.click();
    await strInc.click();
    expect(await $('[data-testid="char-value-str"]').getText()).toBe('+3');
    // At the base cap the increment button is disabled — no headroom yet.
    expect(await strInc.isEnabled()).toBe(false);

    // Add Great Characteristic and target Strength via the characteristic picker.
    await $('[data-testid="tab-virtues_flaws"]').click();
    const addGreat = await $('[data-testid="add-virtue.great_characteristic"]');
    await addGreat.waitForExist({ timeout: 10000 });
    await addGreat.click();
    // Match by testid prefix: the index suffix is the entity-array position,
    // which depends on what else is selected, so don't pin it.
    const charParam = await $('[data-testid^="param-virtue.great_characteristic-characteristic"]');
    await charParam.waitForExist({ timeout: 5000 });
    await charParam.selectByAttribute('value', 'characteristic.str');

    // Back on Characteristics: the cap is now +4, so increment is enabled again
    // and clicking it buys Strength up to +4 (no free point was granted).
    await $('[data-testid="tab-characteristics"]').click();
    const strInc2 = await $('[data-testid="char-inc-str"]');
    await strInc2.waitForClickable({ timeout: 10000 });
    await strInc2.click();
    expect(await $('[data-testid="char-value-str"]').getText()).toBe('+4');
    // +4 is the cap with one Great; the button disables again.
    expect(await strInc2.isEnabled()).toBe(false);

    // Buy Awareness up to 2, then add Puissant Ability targeting it.
    await $('[data-testid="tab-abilities"]').click();
    await $('[data-testid="xp-pool"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="xp-pool"]').setValue(30);
    await $('[data-testid="add-ability.awareness"]').click();
    const awarenessInc = await $('[data-testid="ability-inc-ability.awareness-0"]');
    await awarenessInc.waitForExist({ timeout: 5000 });
    await awarenessInc.click();
    await awarenessInc.click();
    expect(await $('[data-testid="ability-score-ability.awareness-0"]').getText()).toBe('2');

    await $('[data-testid="tab-virtues_flaws"]').click();
    await $('[data-testid="add-virtue.puissant_ability"]').click();
    const abilityParam = await $('[data-testid^="param-virtue.puissant_ability-ability"]');
    await abilityParam.waitForExist({ timeout: 5000 });
    await abilityParam.selectByAttribute('value', 'ability.awareness');

    // Back on Abilities: Awareness shows effective 4 (base 2, Puissant +2).
    await $('[data-testid="tab-abilities"]').click();
    const abilityBadge = await $('[data-testid="ability-eff-ability.awareness-0"]');
    await abilityBadge.waitForExist({ timeout: 10000 });
    expect(await abilityBadge.getText()).toContain('4');
  });
});
