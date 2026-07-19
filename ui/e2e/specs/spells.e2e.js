// End-to-end: Spells (magus-only). The Spells tab appears only for a magus;
// spells are picked from the catalogue (filtered by Technique/Form) against a
// 120-level budget with a per-spell level cap; Skilled Parens raises the budget;
// and the list round-trips through a save. All against the real binary.
//
// The wdio session is shared across the `it` blocks below, so they run as one
// ordered narrative (each builds on the prior state), like the Arts spec.
//
// NOTE: requires a display + the production binary (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, $$, expect, browser } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const TYPE_SELECT = '[data-testid="type-select"]';
const SPELLS_TAB = '[data-testid="tab-spells"]';
const ARTS_TAB = '[data-testid="tab-arts"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const BAR = '[data-testid="spell-levels-used"]';

// The app's save/load dialog seam (ARM_E2E_FILE) points at this fixed path.
const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function setType(value) {
  await $(TYPE_SELECT).selectByAttribute('value', value);
}

async function codeExists(code) {
  return (await $$(`[data-code="${code}"]`).length) > 0;
}

// Raise one Art's score by `times` clicks (score is set regardless of XP).
async function raiseArt(artId, times) {
  const inc = await $(`[data-testid="art-inc-${artId}"]`);
  await inc.waitForExist({ timeout: 10000 });
  for (let i = 0; i < times; i++) await inc.click();
}

describe('spells', () => {
  it('shows the Spells tab only for a magus', async () => {
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await setType('companion');
    await browser.waitUntil(async () => !(await $(SPELLS_TAB).isExisting()), {
      timeout: 5000,
      timeoutMsg: 'companion must not show the Spells tab',
    });
    await setType('magus');
    await $(SPELLS_TAB).waitForExist({ timeout: 5000 });
  });

  it('adds a catalogue spell (filtered by Technique/Form) onto the 120 budget', async () => {
    await $(SPELLS_TAB).click();
    // Filter to Creo Ignem, then add Pilum of Fire (CrIg 20).
    await $('[data-testid="spell-technique-filter"]').selectByAttribute('value', 'art.creo');
    await $('[data-testid="spell-form-filter"]').selectByAttribute('value', 'art.ignem');
    await $('[data-testid="add-spell.pilum_of_fire"]').click();

    await browser.waitUntil(async () => clean(await $(BAR).getText()).includes('20 / 120'), {
      timeout: 5000,
      timeoutMsg: 'spell-levels bar should read 20 / 120',
    });
  });

  it('flags a spell above the per-spell cap (fresh magus: cap 3)', async () => {
    // The magus has 0 Arts / Int / Magic Theory, so the cap is 0+0+0+0+3 = 3 and
    // Pilum (level 20) exceeds it.
    await browser.waitUntil(async () => codeExists('spell_level_exceeds_cap'), {
      timeout: 5000,
      timeoutMsg: 'expected spell_level_exceeds_cap while Arts are 0',
    });
  });

  it('clears the per-spell cap once the Arts are high enough', async () => {
    await $(ARTS_TAB).click();
    await $('[data-testid="art-xp-pool"]').setValue('1000');
    await raiseArt('art.creo', 12);
    await raiseArt('art.ignem', 12);
    // Cap is now 12 + 12 + 0 + 0 + 3 = 27 ≥ 20, so Pilum is legal.
    await $(SPELLS_TAB).click();
    await browser.waitUntil(async () => !(await codeExists('spell_level_exceeds_cap')), {
      timeout: 5000,
      timeoutMsg: 'raising Creo/Ignem should clear the per-spell cap',
    });
  });

  it('raises the spell-levels budget with Skilled Parens', async () => {
    await $(VF_TAB).click();
    const addParens = await $('[data-testid="add-virtue.skilled_parens"]');
    await addParens.waitForExist({ timeout: 10000 });
    await addParens.click();
    await $(SPELLS_TAB).click();
    await browser.waitUntil(async () => clean(await $(BAR).getText()).includes('/ 150'), {
      timeout: 5000,
      timeoutMsg: 'Skilled Parens should raise the budget to 150',
    });
  });

  it('flags going over the spell-levels budget', async () => {
    // Add a General spell (Aegis of the Hearth, ReVi). It lands at the default
    // level; its level input then appears inline on the selected row. Setting it
    // to a huge level pushes the total (20 + 200) past the 150 budget.
    await $('[data-testid="spell-technique-filter"]').selectByAttribute('value', '');
    await $('[data-testid="spell-form-filter"]').selectByAttribute('value', '');
    await $('[data-testid="add-spell.aegis_of_the_hearth"]').click();
    const levelInput = await $('[data-testid="spell-level-input"]');
    await levelInput.waitForExist({ timeout: 5000 });
    await levelInput.setValue('200');

    await browser.waitUntil(async () => codeExists('over_spell_levels'), {
      timeout: 5000,
      timeoutMsg: 'expected over_spell_levels once the total passes the budget',
    });
  });

  it('round-trips the spell list through a save', async () => {
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(11);
    expect(saved.spells.some((s) => s.spell === 'spell.pilum_of_fire')).toBe(true);
    expect(
      saved.spells.some((s) => s.spell === 'spell.aegis_of_the_hearth' && s.level === 200),
    ).toBe(true);
  });
});
