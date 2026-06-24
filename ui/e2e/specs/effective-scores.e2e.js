// End-to-end: a virtue's score bonus shows as an "effective" badge in direct
// entry. Great Characteristic (+1) on Strength and Puissant Ability (+2) on
// Awareness each add to the displayed base score. Drives the real binary.

import { $, expect } from '@wdio/globals';

describe('effective-score badges', () => {
  it('shows Great Characteristic and Puissant Ability bonuses', async () => {
    // Characteristics is the default tab; raise Strength to +3 (Great's minimum).
    const strInc = await $('[data-testid="char-inc-str"]');
    await strInc.waitForExist({ timeout: 30000 });
    await strInc.click();
    await strInc.click();
    await strInc.click();
    expect(await $('[data-testid="char-value-str"]').getText()).toBe('+3');

    // Add Great Characteristic and target Strength via the characteristic picker.
    await $('[data-testid="tab-virtues_flaws"]').click();
    const addGreat = await $('[data-testid="add-virtue.great_characteristic"]');
    await addGreat.waitForExist({ timeout: 10000 });
    await addGreat.click();
    const charParam = await $('[data-testid="param-virtue.great_characteristic-characteristic"]');
    await charParam.waitForExist({ timeout: 5000 });
    await charParam.selectByAttribute('value', 'characteristic.str');

    // Back on Characteristics: Strength shows effective +4 (base +3, Great +1).
    // Fluent wraps the placeholder in bidi isolation marks, so match the value
    // as a substring rather than the exact string.
    await $('[data-testid="tab-characteristics"]').click();
    const strBadge = await $('[data-testid="char-eff-str"]');
    await strBadge.waitForExist({ timeout: 10000 });
    expect(await strBadge.getText()).toContain('+4');

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
    const abilityParam = await $('[data-testid="param-virtue.puissant_ability-ability"]');
    await abilityParam.waitForExist({ timeout: 5000 });
    await abilityParam.setValue('ability.awareness');

    // Back on Abilities: Awareness shows effective 4 (base 2, Puissant +2).
    await $('[data-testid="tab-abilities"]').click();
    const abilityBadge = await $('[data-testid="ability-eff-ability.awareness-0"]');
    await abilityBadge.waitForExist({ timeout: 10000 });
    expect(await abilityBadge.getText()).toContain('4');
  });
});
