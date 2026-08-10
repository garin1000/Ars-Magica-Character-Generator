// End-to-end: Spells (magus-only). The Spells tab appears only for a magus;
// spells are picked from the catalogue (filtered by Technique/Form) against a
// 120-level budget with a per-spell level cap; Skilled Parens raises the budget;
// and the list round-trips through a save. All against the real binary.
//
// The wdio session is shared across the `it` blocks below, so they run as one
// ordered narrative (each builds on the prior state), like the Arts spec.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, $$, expect, browser } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const TYPE_SELECT = '[data-testid="type-select"]';
const SPELLS_TAB = '[data-testid="tab-spells"]';
const ARTS_TAB = '[data-testid="tab-arts"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
// The spell-levels status bar above both lists reads like the XP bar: the used
// figure, the editable base (bracketed), then Available and any V/F bonus. The
// effective budget is base + bonus, so Available is what proves the total.
const BAR_USED = '[data-testid="spell-levels-used"]';
const BAR_BASE = '[data-testid="spell-levels-base"]';
const BAR_AVAILABLE = '[data-testid="spell-levels-available"]';
const BAR_BONUS = '[data-testid="spell-levels-bonus"]';

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

  it('shows the cap reason ABOVE the description on a greyed spell', async () => {
    // Pilum is still greyed (per-spell cap 3), the Creo Ignem filter still
    // applied. Its tooltip must show the non-takeable REASON and, below it, the
    // spell's normal description — reason first, not instead of the description.
    const row = await $('[data-testid="add-spell.pilum_of_fire"]');
    await row.waitForExist({ timeout: 5000 });
    expect(await row.isEnabled()).toBe(false);
    // Dispatch mouseenter directly (same rationale as the description-tooltip
    // test below: synthetic events are focus-independent under parallel wdio).
    await browser.execute((el) => {
      el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
    }, row);
    const reason = await $('.tooltip-pop .tooltip-reason');
    await reason.waitForExist({ timeout: 5000 });
    expect((await reason.getText()).trim().length).toBeGreaterThan(0);
    const desc = await $('.tooltip-pop .tooltip-text');
    await desc.waitForExist({ timeout: 5000 });
    expect((await desc.getText()).trim().length).toBeGreaterThan(0);
    // Dismiss the popup so it does not linger into the next step, which raises
    // the Arts and re-enables Pilum.
    await browser.execute((el) => {
      el.dispatchEvent(new MouseEvent('mouseleave', { bubbles: true }));
    }, row);
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
    await browser.waitUntil(
      async () =>
        clean(await $(BAR_USED).getText()).includes('20') &&
        // No spell-levels V/F yet, so the budget is the 120 base: 120 - 20 = 100.
        clean(await $(BAR_AVAILABLE).getText()).includes('100'),
      {
        timeout: 5000,
        timeoutMsg: 'spell-levels bar should read 20 used with 100 available of 120',
      },
    );
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
    // The +30 is listed as its own pool and spent BEFORE the base (like restricted
    // XP): Pilum's 20 levels shift off the base onto the bonus, so the bonus entry
    // reads 20 / 30 and the untouched 120 base is fully available again.
    await browser.waitUntil(
      async () => {
        const bonus = clean(await $(BAR_BONUS).getText());
        return (
          bonus.includes('20') &&
          bonus.includes('30') &&
          clean(await $(BAR_AVAILABLE).getText()).includes('120')
        );
      },
      {
        timeout: 5000,
        timeoutMsg: 'Skilled Parens should report 20 / 30 spent and free the 120 base',
      },
    );
  });

  it('applies an editable spell-levels budget override', async () => {
    // The bracketed base field's placeholder is the type profile's base (120),
    // proving the default is data-driven (surfaced by the engine, not a UI literal).
    const override = await $(BAR_BASE);
    await override.waitForExist({ timeout: 5000 });
    expect(await override.getAttribute('placeholder')).toBe('120');
    // Override the base to 80; Skilled Parens's +30 still adds on top (a 110
    // budget). Pilum's 20 levels are charged to the bonus, so the whole 80 base is
    // available.
    await override.setValue('80');
    await browser.waitUntil(async () => clean(await $(BAR_AVAILABLE).getText()).includes('80'), {
      timeout: 5000,
      timeoutMsg: 'overriding the base to 80 should leave the full 80 base available',
    });
    // Reset the override (empty field -> profile base) so later specs see 150
    // again. Dispatch the input event directly: clearValue() does not reliably
    // fire the event Svelte listens to on a number input (same reason as the
    // level-max filter reset above).
    await browser.execute((el) => {
      el.value = '';
      el.dispatchEvent(new Event('input', { bubbles: true }));
    }, override);
    await browser.waitUntil(async () => clean(await $(BAR_AVAILABLE).getText()).includes('120'), {
      timeout: 5000,
      timeoutMsg: 'clearing the override should restore the profile base (120 available)',
    });
  });

  it("takes a parametrized spell once per distinct Form (Wizard's Boost)", async () => {
    // Clear the Technique/Form filter so the MuVi Wizard's Boost is listed. It is
    // a General meta-magic Vim spell whose target (Form) is a per-instance
    // selection: Muto 0 + Vim 9 + 3 = 12 clears the default level (5), and the
    // 150 budget has room (only Pilum's 20 is spent so far).
    await $('[data-testid="spell-technique-filter"]').selectByAttribute('value', '');
    await $('[data-testid="spell-form-filter"]').selectByAttribute('value', '');
    const add = await $('[data-testid="add-spell.wizards_boost_form"]');
    await add.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => await add.isEnabled(), {
      timeout: 5000,
      timeoutMsg: "Wizard's Boost should be takeable (MuVi cap 12, budget has room)",
    });
    // The source candidate reads its param hint, not a raw token: "Wizard's Boost
    // (Form) (General)".
    expect(clean(await add.getText())).toContain('(Form)');

    const paramSelects = '[data-testid^="spell-param-spell.wizards_boost_form-"]';
    const nameSpans = '[data-testid^="spell-name-spell.wizards_boost_form-"]';

    // (a) Add one instance and choose Form = Ignem; its selected label shows it.
    await add.click();
    await browser.waitUntil(async () => (await $$(paramSelects)).length === 1, {
      timeout: 5000,
      timeoutMsg: "adding Wizard's Boost should show one target-Form select",
    });
    await (await $$(paramSelects))[0].selectByAttribute('value', 'art.ignem');
    await browser.waitUntil(
      async () => clean(await (await $$(nameSpans))[0].getText()).includes('Ignem'),
      { timeout: 5000, timeoutMsg: 'the chosen Form (Ignem) should show in the row label' },
    );

    // (b) Add the SAME base spell again choosing Form = Aquam; BOTH coexist.
    await add.click();
    await browser.waitUntil(async () => (await $$(paramSelects)).length === 2, {
      timeout: 5000,
      timeoutMsg: 'the same base spell may be taken once per distinct Form',
    });
    await (await $$(paramSelects))[1].selectByAttribute('value', 'art.aquam');
    await browser.waitUntil(
      async () => {
        // Exactly two instances now; index them directly (a `$$` ElementArray is
        // not safely spread/mapped across an await here).
        const spans = await $$(nameSpans);
        if (spans.length !== 2) return false;
        const both = clean(await spans[0].getText()) + '|' + clean(await spans[1].getText());
        return both.includes('Ignem') && both.includes('Aquam');
      },
      { timeout: 5000, timeoutMsg: 'both the Ignem and Aquam instances should coexist' },
    );

    // (c) A third instance's Form picker greys the Forms already used at this
    // level (Ignem, Aquam), so the same (spell, level, Form) cannot be taken
    // twice — while every unused Form (e.g. Terram) stays selectable. Choosing an
    // unused Form adds a distinct instance without any duplicate_spell flag.
    await add.click();
    await browser.waitUntil(async () => (await $$(paramSelects)).length === 3, {
      timeout: 5000,
      timeoutMsg: 'a third instance should be added (its Form is not yet chosen)',
    });
    const thirdSelect = (await $$(paramSelects))[2];
    await browser.waitUntil(
      async () => !(await (await thirdSelect.$('option[value="art.ignem"]')).isEnabled()),
      { timeout: 5000, timeoutMsg: 'the already-used Ignem Form should be greyed in a new row' },
    );
    expect(await (await thirdSelect.$('option[value="art.aquam"]')).isEnabled()).toBe(false);
    expect(await (await thirdSelect.$('option[value="art.terram"]')).isEnabled()).toBe(true);
    await thirdSelect.selectByAttribute('value', 'art.terram');
    await browser.waitUntil(async () => !(await codeExists('duplicate_spell')), {
      timeout: 5000,
      timeoutMsg: 'a distinct Form (Terram) must not flag duplicate_spell',
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
    // Target Aegis's own level input (per-spell testid), so other General spells
    // still in the list (e.g. leftover Wizard's Boost instances) don't shadow it.
    const levelInput = await $('[data-testid^="spell-level-input-spell.aegis_of_the_hearth-"]');
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
    expect(saved.schema_version).toBe(14);
    expect(saved.spells.some((s) => s.spell === 'spell.pilum_of_fire')).toBe(true);
    expect(
      saved.spells.some((s) => s.spell === 'spell.aegis_of_the_hearth' && s.level === 200),
    ).toBe(true);
  });
});
