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

  it('groups the source spells by Technique/Form', async () => {
    await $(SPELLS_TAB).click();
    // The source list is grouped like the Ability picker; the Creo Ignem group
    // header composes the two localized Art names (never a raw id).
    const header = await $('[data-testid="spell-group-art.creo-art.ignem"]');
    await header.waitForExist({ timeout: 5000 });
    // The header composes the two localized Art names; the `.category` class
    // uppercases the text for display (same convention as the Ability picker),
    // so compare case-insensitively rather than pinning the display casing.
    expect(
      clean(await header.getText())
        .trim()
        .toLowerCase(),
    ).toBe('creo ignem');
  });

  it('narrows the source list with the min/max level filter', async () => {
    // With no Technique/Form filter the full catalogue is grouped. Capping the
    // max level drops every fixed-level spell above it (General spells always
    // remain), so the list shrinks and Pilum (level 20) disappears.
    const before = (await $$('[data-testid^="add-spell."]')).length;
    expect(await $('[data-testid="add-spell.pilum_of_fire"]').isExisting()).toBe(true);

    await $('[data-testid="spell-level-max-filter"]').setValue('5');
    await browser.waitUntil(async () => (await $$('[data-testid^="add-spell."]')).length < before, {
      timeout: 5000,
      timeoutMsg: 'a max-level filter should narrow the source list',
    });
    expect(await $('[data-testid="add-spell.pilum_of_fire"]').isExisting()).toBe(false);

    // Restore the open range so later steps see the full list again. Clear via a
    // dispatched input event: WebDriver clearValue() does not reliably fire the
    // event Svelte's bind:value listens to on a number input, so the bound state
    // would otherwise stay at 5 and leak into later steps. This still exercises
    // the real bind (empty field -> open range), just deterministically.
    await browser.execute(
      (el) => {
        el.value = '';
        el.dispatchEvent(new Event('input', { bubbles: true }));
      },
      await $('[data-testid="spell-level-max-filter"]'),
    );
    await browser.waitUntil(
      async () => (await $$('[data-testid^="add-spell."]')).length === before,
      {
        timeout: 5000,
        timeoutMsg: 'clearing the max-level filter should restore the list',
      },
    );
  });

  it('greys a spell above the per-spell cap for a fresh magus (cap 3)', async () => {
    // The magus has 0 Arts / Int / Magic Theory, so the CrIg cap is 0+0+0+0+3 = 3;
    // Pilum (level 20) exceeds it, so its add control is greyed (disabled).
    await $('[data-testid="spell-technique-filter"]').selectByAttribute('value', 'art.creo');
    await $('[data-testid="spell-form-filter"]').selectByAttribute('value', 'art.ignem');
    const pilum = await $('[data-testid="add-spell.pilum_of_fire"]');
    await pilum.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => !(await pilum.isEnabled()), {
      timeout: 5000,
      timeoutMsg: 'Pilum should be greyed while the cap is 3',
    });
  });

  it('adds Pilum onto the 120 budget once the Arts are high enough', async () => {
    // Raise Creo/Ignem (for Pilum) and Rego/Vim (for the General ritual added
    // later) so both clear their per-spell caps.
    await $(ARTS_TAB).click();
    await $('[data-testid="art-xp-pool"]').setValue('1000');
    await raiseArt('art.creo', 12);
    await raiseArt('art.ignem', 12);
    await raiseArt('art.rego', 9);
    await raiseArt('art.vim', 9);
    // CrIg cap is now 12 + 12 + 3 = 27 ≥ 20, so Pilum is takeable again.
    await $(SPELLS_TAB).click();
    const pilum = await $('[data-testid="add-spell.pilum_of_fire"]');
    await browser.waitUntil(async () => await pilum.isEnabled(), {
      timeout: 5000,
      timeoutMsg: 'raising Creo/Ignem should re-enable Pilum',
    });
    await pilum.click();
    await browser.waitUntil(async () => clean(await $(BAR).getText()).includes('20 / 120'), {
      timeout: 5000,
      timeoutMsg: 'spell-levels bar should read 20 / 120',
    });
    // The cap no longer flags Pilum.
    await browser.waitUntil(async () => !(await codeExists('spell_level_exceeds_cap')), {
      timeout: 5000,
      timeoutMsg: 'a legal Pilum should not flag the per-spell cap',
    });
  });

  it('shows a description tooltip when hovering a spell row', async () => {
    // The Creo Ignem filter is still applied, so Pilum of Fire is in the source
    // list. Hovering its row appends the description popup to <body>.
    const row = await $('[data-testid="add-spell.pilum_of_fire"]');
    await row.waitForExist({ timeout: 5000 });
    // Dispatch mouseenter directly: the `tooltip` action binds to it, so this
    // exercises the real wiring (action attached + description present + popup
    // built) deterministically. Pointer moveTo / el.focus() are unreliable under
    // parallel webdriver runs because the webview window is blurred, which
    // suppresses OS hover/focus events; a synthetic event is focus-independent.
    await browser.execute((el) => {
      el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
    }, row);
    const pop = await $('.tooltip-pop .tooltip-text');
    await pop.waitForExist({ timeout: 5000 });
    expect((await pop.getText()).trim().length).toBeGreaterThan(0);
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

  it('applies an editable spell-levels budget override', async () => {
    // The override field's placeholder is the type profile's base (120), proving
    // the default is data-driven (surfaced by the engine, not a UI literal).
    const override = await $('[data-testid="spell-levels-override"]');
    await override.waitForExist({ timeout: 5000 });
    expect(await override.getAttribute('placeholder')).toBe('120');
    // Override the base to 80; Skilled Parens's +30 still adds on top → 80 + 30 = 110.
    await override.setValue('80');
    await browser.waitUntil(async () => clean(await $(BAR).getText()).includes('/ 110'), {
      timeout: 5000,
      timeoutMsg: 'overriding the base to 80 should make the budget 80 + 30 = 110',
    });
    // Reset the override (empty field -> profile base) so later specs see 150
    // again. Dispatch the input event directly: clearValue() does not reliably
    // fire the event Svelte listens to on a number input (same reason as the
    // level-max filter reset above).
    await browser.execute((el) => {
      el.value = '';
      el.dispatchEvent(new Event('input', { bubbles: true }));
    }, override);
    await browser.waitUntil(async () => clean(await $(BAR).getText()).includes('/ 150'), {
      timeout: 5000,
      timeoutMsg: 'clearing the override should restore the profile-based budget (150)',
    });
  });

  it('flags going over the spell-levels budget', async () => {
    // Add a General spell (Aegis of the Hearth, ReVi). It lands at the default
    // level; its level input then appears inline on the selected row. Setting it
    // to a huge level pushes the total (20 + 200) past the 150 budget.
    await $('[data-testid="spell-technique-filter"]').selectByAttribute('value', '');
    await $('[data-testid="spell-form-filter"]').selectByAttribute('value', '');
    // Aegis is a ReVi General ritual (min learnable level 20); Rego 9 + Vim 9 + 3
    // = 21 clears its cap, so its add control is enabled.
    const aegis = await $('[data-testid="add-spell.aegis_of_the_hearth"]');
    await aegis.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => await aegis.isEnabled(), {
      timeout: 5000,
      timeoutMsg: 'Aegis should be takeable once Rego/Vim clear its ritual cap',
    });
    await aegis.click();
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
    expect(saved.schema_version).toBe(13);
    expect(saved.spells.some((s) => s.spell === 'spell.pilum_of_fire')).toBe(true);
    expect(
      saved.spells.some((s) => s.spell === 'spell.aegis_of_the_hearth' && s.level === 200),
    ).toBe(true);
  });
});
