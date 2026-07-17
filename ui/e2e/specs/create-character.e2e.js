// End-to-end: drive the tabbed editor — set a Characteristic (spinner), pick a
// balanced V/F pair, buy an Ability against an XP pool — then save and reload.
// Drives the real binary; assertions read the DOM and the saved file.

import { browser, $, expect } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

describe('character editor', () => {
  it('edits across tabs, validates, and round-trips a save', async () => {
    // Open the Characteristics tab; its spinner appearing means the ruleset
    // has loaded.
    await $('[data-testid="tab-characteristics"]').waitForExist({ timeout: 30000 });
    await $('[data-testid="tab-characteristics"]').click();
    const intInc = await $('[data-testid="char-inc-int"]');
    await intInc.waitForExist({ timeout: 30000 });

    // Raise Intelligence to +2 with the spinner (0 -> +1 -> +2).
    await intInc.click();
    await intInc.click();
    expect(await $('[data-testid="char-value-int"]').getText()).toBe('+2');

    // Virtues & Flaws tab: a minor virtue funded by a minor flaw is balanced.
    await $('[data-testid="tab-virtues_flaws"]').click();
    await $('[data-testid="add-virtue.keen_vision"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="add-virtue.keen_vision"]').click();
    await $('[data-testid="add-flaw.poor_student"]').click();
    // Match by testid prefix: the suffix is the entity-array index, which shifts
    // after canonical save/load reordering of selections.
    const removeKeenVision = await $('[data-testid^="remove-virtue.keen_vision"]');
    await removeKeenVision.waitForExist({ timeout: 5000 });

    // Abilities tab: give an XP pool, then buy Awareness up to 2 (15 xp).
    await $('[data-testid="tab-abilities"]').click();
    await $('[data-testid="xp-pool"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="xp-pool"]').setValue(30);
    await $('[data-testid="add-ability.awareness"]').click();
    const awarenessInc = await $('[data-testid="ability-inc-ability.awareness-0"]');
    await awarenessInc.waitForExist({ timeout: 5000 });
    await awarenessInc.click();
    await awarenessInc.click();
    expect(await $('[data-testid="ability-score-ability.awareness-0"]').getText()).toBe('2');

    // Save through the ARM_E2E_FILE seam, then confirm the JSON on disk.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(7);
    expect(saved.selections.some((s) => s.ref === 'virtue.keen_vision')).toBe(true);
    expect(saved.characteristics.int).toBe(2);
    expect(
      saved.ability_scores.some((a) => a.ability === 'ability.awareness' && a.score === 2),
    ).toBe(true);
    expect(saved.xp_pool).toBe(30);

    // Back on the V/F tab, drop the virtue, reload the file, and confirm it
    // returns — proving load repopulates the entity.
    await $('[data-testid="tab-virtues_flaws"]').click();
    await removeKeenVision.click();
    await $('[data-testid="load-button"]').click();
    await $('[data-testid^="remove-virtue.keen_vision"]').waitForExist({ timeout: 10000 });
  });
});
