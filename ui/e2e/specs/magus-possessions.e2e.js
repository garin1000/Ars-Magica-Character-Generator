// End-to-end: a magus's possessions — Spells, the familiar statblock, the
// talisman item, the Longevity Ritual, and the Markdown character-sheet export.
//
// A2 merged five previously separate spec files into this one. Each describe
// keeps its own local selectors and helpers scoped to its block, since several
// of them reused the same constant names (`VF_TAB`, `ARTS_TAB`,
// `POSSESSIONS_TAB`, `TOTALS_TAB`) with the same values but no reason to share
// a single module-level binding.
//
// `markdown export`'s German-locale test restores `en` in its own body before
// the end of the file, following the pattern already used at
// `magus-editor.e2e.js` (ex-`arts.e2e.js`) and `wizard-flow.e2e.js`
// (ex-`tab-area.e2e.js`).

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import {
  clean,
  dismissTooltip,
  hoverForTooltip,
  isRowBlocked,
  runDocumentAction,
  setLanguage,
  startCharacter,
  waitForIdle,
  waitUntilExplained,
} from '../helpers.js';
import { e2eExportFile, e2eFile } from '../wdio.conf.js';

// End-to-end: Spells (magus-only). The Spells tab appears only for a magus;
// spells are picked from the catalogue (filtered by Technique/Form) against a
// 120-level budget with a per-spell level cap; Skilled Parens raises the budget;
// and the list round-trips through a save.
//
// The wdio session is shared across the `it` blocks below, so they run as one
// ordered narrative (each builds on the prior state), like the Arts spec. The
// first block creates the magus all the others go on building, so it is the only
// one that calls `startCharacter`.
describe('spells', () => {
  const SPELLS_TAB = '[data-testid="tab-spells"]';
  const ARTS_TAB = '[data-testid="tab-arts"]';
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  // The spell-levels status bar above both lists reads like the XP bar: the used
  // figure over the total it is charged against, then Available, the base and any V/F
  // bonus. The effective budget is base + bonus, so Available is what proves the total.
  const BAR_USED = '[data-testid="spell-levels-used"]';
  // The base is its OWN entry beside the pair since #18 — the pair closes against
  // base + post-Gauntlet levels (`spell-levels-total`), which is a different number
  // whenever a magus has lived past its Gauntlet. In the EDITOR it is still the
  // editable field (#19 makes only the wizard's mount read-only), which is what this
  // spec drives.
  const BAR_BASE = '[data-testid="spell-levels-base"]';
  const BAR_TOTAL = '[data-testid="spell-levels-total"]';
  const BAR_AVAILABLE = '[data-testid="spell-levels-available"]';
  const BAR_BONUS = '[data-testid="spell-levels-bonus"]';

  async function codeExists(code) {
    return (await $$(`[data-code="${code}"]`).length) > 0;
  }

  // Raise one Art's score by `times` clicks (score is set regardless of XP).
  async function raiseArt(artId, times) {
    const inc = await $(`[data-testid="art-inc-${artId}"]`);
    await inc.waitForExist({ timeout: 10000 });
    for (let i = 0; i < times; i++) await inc.click();
  }

  it('shows the Spells tab only for a magus', async () => {
    await startCharacter('companion');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    expect(await $(SPELLS_TAB).isExisting()).toBe(false);

    // The magus every block below goes on building.
    await startCharacter('magus');
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
    await browser.waitUntil(async () => await isRowBlocked(pilum), {
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
    expect(await isRowBlocked(row)).toBe(true);
    await hoverForTooltip(row);
    const reason = await $('[data-testid="tooltip-reason"]');
    await reason.waitForExist({ timeout: 5000 });
    expect((await reason.getText()).trim().length).toBeGreaterThan(0);
    const desc = await $('[data-testid="tooltip-text"]');
    await desc.waitForExist({ timeout: 5000 });
    expect((await desc.getText()).trim().length).toBeGreaterThan(0);
    // Dismiss the popup so it does not linger into the next step, which raises
    // the Arts and re-enables Pilum.
    await dismissTooltip(row);
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
    await browser.waitUntil(async () => !(await isRowBlocked(pilum)), {
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
    await hoverForTooltip(row);
    const pop = await $('[data-testid="tooltip-text"]');
    await pop.waitForExist({ timeout: 5000 });
    expect((await pop.getText()).trim().length).toBeGreaterThan(0);
    // Dismiss it: the popup now takes the pointer, so left open it could
    // swallow a click the next test aims at the tabs.
    await dismissTooltip(row);
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

  // Slice 6b8c: the OTHER half of the same Virtue. "You gain an additional 60
  // experience points and 30 spell levels during apprenticeship" (`ArMDE:4966`) — the
  // engine has always granted the 60, but the XP bar charged the spend against the
  // typed pool alone, so this legal magus read a negative Available with no error
  // anywhere. Real-binary arithmetic, because only the live payload carries the
  // engine's own pool.
  it('raises the experience pool with the same Skilled Parens, and says so', async () => {
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 5000 });
    // Creo 12 + Ignem 12 + Rego 9 + Vim 9 = 78 + 78 + 45 + 45 = 246 XP against a
    // typed 1000 raised to 1060. The +60 is spent FIRST, like a restricted pool, so
    // 186 is charged to the base and 1000 - 186 = 814 stays available.
    await browser.waitUntil(
      async () =>
        clean(await $('[data-testid="art-xp-bonus"]').getText()).includes('60') &&
        clean(await $('[data-testid="art-xp-spent"]').getText()).trim() === '186' &&
        clean(await $('[data-testid="art-xp-available"]').getText()).includes('814'),
      {
        timeout: 5000,
        timeoutMsg: 'Skilled Parens should fund the first 60 XP and leave 814 of the base',
      },
    );
    // The editable total stays the number the player typed — the bonus is listed
    // beside it, never folded into the field.
    expect(await pool.getValue()).toBe('1000');
    // Back to the Spells tab: the blocks below carry on there.
    await $(SPELLS_TAB).click();
    await $(BAR_BASE).waitForExist({ timeout: 5000 });
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
    // #18: the field edits the BASE, and the denominator follows it. With no
    // post-Gauntlet levels the two coincide, so the pair reads 0 / 80 — the field is
    // still what moves the total, it simply no longer occupies its slot.
    expect(clean(await $(BAR_TOTAL).getText())).toBe('80');
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
    await browser.waitUntil(async () => !(await isRowBlocked(add)), {
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
    await browser.waitUntil(async () => !(await isRowBlocked(aegis)), {
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
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(23);
    expect(saved.spells.some((s) => s.spell === 'spell.pilum_of_fire')).toBe(true);
    expect(
      saved.spells.some((s) => s.spell === 'spell.aegis_of_the_hearth' && s.level === 200),
    ).toBe(true);
  });
});

// End-to-end: the familiar is a CREATURE STATBLOCK, not a name plus three cords.
// Driven through the real binary: animal, a negative Size, Magic Might, a
// Characteristic, a Personality Trait, the three cords and one bond-invested power
// are all enterable; the totals panel's bonding level folds the negative Size in;
// the cord points follow the 5/15/30/50/75 curve; NO validation issue appears (the
// guidance-only contract, end to end); and NO power-levels budget bar is rendered
// (ArMDE:10866 — there is no limit). Then the whole statblock round-trips through
// save + Open.
describe('familiar', () => {
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const POSSESSIONS_TAB = '[data-testid="tab-possessions"]';
  const TOTALS_TAB = '[data-testid="tab-totals"]';

  const BINDING = '[data-testid="derived-familiar-binding"]';
  const CORDS = '[data-testid="derived-familiar-cords"]';
  const INVESTED = '[data-testid="derived-familiar-invested"]';

  /** Wait for a (debounced) engine read-out to contain every fragment. */
  async function waitForText(selector, ...fragments) {
    await waitUntilExplained(
      async () => {
        const text = clean(await $(selector).getText());
        return fragments.every((f) => text.includes(f));
      },
      10000,
      async () =>
        `${selector} never showed ${fragments.join(' + ')}; read ` +
        `"${clean(await $(selector).getText())}"`,
    );
  }

  async function set(testid, value) {
    const field = await $(`[data-testid="${testid}"]`);
    await field.waitForExist({ timeout: 10000 });
    await field.setValue(value);
  }

  // An issue only a MAGUS raises (no House chosen yet), used to prove the validation
  // panel is showing this magus's own pass. No House is ever chosen in this spec, so
  // the warning is present for every snapshot taken here.
  const MAGUS_GATE = '[data-code="house_unset"]';

  /**
   * The validation panel's issue counts, once a validation pass for the CURRENT
   * character type has rendered. A bare magus already carries unrelated advisories
   * (unspent points and the like), so the guidance-only contract is "these counts do
   * not move", not "there are none".
   *
   * `gate` is a selector for an issue only the current type raises — waiting for it is
   * what proves the debounced pass for THIS magus has landed. Waiting merely for the
   * panel to *exist* proves nothing: it may still be showing the pass for whatever the
   * previous spec left on screen, so the counts could be read from a pre-magus pass and
   * the stale baseline would surface later as a spurious red.
   */
  async function issueCounts(gate) {
    await browser.waitUntil(async () => await $(gate).isExisting(), {
      timeout: 10000,
      timeoutMsg: `the validation panel never rendered a pass containing ${gate}`,
    });
    return {
      errors: await $$('[data-severity="error"]').length,
      warnings: await $$('[data-severity="warning"]').length,
    };
  }

  it('enters the whole statblock and derives the bonding numbers as guidance', async () => {
    await startCharacter('magus');
    await $(VF_TAB).waitForExist({ timeout: 30000 });

    await $(POSSESSIONS_TAB).click();
    // Baseline for the guidance-only check: whatever this magus already complains
    // about before a familiar exists — read from the MAGUS's own validation pass,
    // never one left over from the previous spec.
    const issuesBefore = await issueCounts(MAGUS_GATE);

    await $('[data-testid="familiar-add"]').click();

    await set('familiar-name', 'Corvus');
    await set('familiar-animal', 'raven');
    // A raven is Size -4 (ArMDE:17829-17856). The rendered value must carry the
    // ASCII hyphen-minus, never the mathematical minus U+2212.
    await set('familiar-size', '-4');
    const size = await $('[data-testid="familiar-size"]');
    expect(await size.getValue()).toBe('-4');
    expect(await size.getValue()).not.toContain('−');

    // The familiar's own Magic Might: 10, aligned to the Magic Realm.
    await $('[data-testid="familiar-might-add"]').click();
    await $('[data-testid="familiar-might-realm"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="familiar-might-realm"]').selectByAttribute('value', 'magic');
    await set('familiar-might-score', '10');

    // Human intelligence at Int -3, gained from the bond (ArMDE:10854).
    await set('familiar-char-int', '-3');
    expect(await $('[data-testid="familiar-char-int"]').getValue()).toBe('-3');

    // The bond's Loyal (partner) +3, entered by hand — never auto-applied.
    await $('[data-testid="familiar-personality-add"]').click();
    await set('familiar-personality-name-0', 'Loyal (Marcus)');
    const inc = await $('[data-testid="familiar-personality-inc-0"]');
    for (let i = 0; i < 3; i++) await inc.click();
    expect(clean(await $('[data-testid="familiar-personality-value-0"]').getText())).toBe('+3');

    // Cords 3 / 2 / 1 → 30 + 15 + 5 = 50 points off the curve (ArMDE:10836).
    await set('familiar-cord-gold', '3');
    await set('familiar-cord-silver', '2');
    await set('familiar-cord-bronze', '1');

    // One power invested in the bond, at level 20.
    await $('[data-testid="familiar-power-add"]').click();
    await set('familiar-power-name-0', 'Mental communication');
    await set('familiar-power-level-0', '20');

    // NO budget bar for the invested powers: the character's own powers get a
    // `power-levels-used` read-out, but ArMDE:10866 sets no limit on what may be
    // invested in a familiar, so a bar here would invent one. The absent bar is the
    // whole statement — manual-testing-findings #21 removed the sentence too.
    expect(await $('[data-testid="power-levels-used"]').isExisting()).toBe(false);
    expect(await $('[data-testid="familiar-powers-note"]').isExisting()).toBe(false);

    // The totals panel: binding level = Magic Might 10 + 25 + 5 x Size(-4) = 15.
    // The negative Size takes 20 points off, which is the whole point of the rule.
    await $(TOTALS_TAB).click();
    await waitForText(BINDING, '15');
    await waitForText(CORDS, '50');
    await waitForText(INVESTED, '20');

    // Guidance only, end to end: a familiar whose cords cost 50 points — far more
    // than any Lab Total this magus has — and which carries 20 levels of invested
    // power adds not one issue, of any severity.
    expect(await issueCounts(MAGUS_GATE)).toEqual(issuesBefore);
  });

  it('round-trips the whole statblock through save and Open', async () => {
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });

    // Every statblock field reaches disk, at the engine's current schema — the
    // statblock fields are additive, so 5.5c bumped nothing of its own.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(23);
    expect(saved.familiar.name).toBe('Corvus');
    expect(saved.familiar.animal).toBe('raven');
    expect(saved.familiar.size).toBe(-4);
    expect(saved.familiar.might).toEqual({ realm: 'magic', score: 10 });
    expect(saved.familiar.characteristics).toEqual({ int: -3 });
    expect(saved.familiar.personality_traits).toEqual([{ name: 'Loyal (Marcus)', value: 3 }]);
    expect(saved.familiar.cord_gold).toBe(3);
    expect(saved.familiar.cord_silver).toBe(2);
    expect(saved.familiar.cord_bronze).toBe(1);
    expect(saved.familiar.powers).toEqual([{ name: 'Mental communication', level: 20 }]);

    // And back in through the shipped binary's own Open path.
    await runDocumentAction('open');
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    await $(POSSESSIONS_TAB).click();
    const animal = await $('[data-testid="familiar-animal"]');
    await animal.waitForExist({ timeout: 15000 });
    expect(await animal.getValue()).toBe('raven');
    expect(await $('[data-testid="familiar-size"]').getValue()).toBe('-4');
    expect(await $('[data-testid="familiar-might-score"]').getValue()).toBe('10');
    expect(await $('[data-testid="familiar-char-int"]').getValue()).toBe('-3');
    expect(await $('[data-testid="familiar-power-level-0"]').getValue()).toBe('20');
    expect(clean(await $('[data-testid="familiar-personality-value-0"]').getText())).toBe('+3');
  });

  // The remove controls are only ever CLICKED here: the panel's unit tests render to
  // a string (SSR), so no handler runs there. A row-remove wired to the wrong list or
  // off by one would delete the player's other row — silent data loss — and every
  // other gate would stay green. Each removal is proved by which row SURVIVES.
  it('removes the clicked row, and then the whole familiar', async () => {
    await $(POSSESSIONS_TAB).click();

    // Two Personality Traits, then remove the FIRST: the second must survive and
    // slide up to index 0.
    await $('[data-testid="familiar-personality-add"]').click();
    await set('familiar-personality-name-1', 'Curious');
    await $('[data-testid="familiar-personality-remove-0"]').click();
    await browser.waitUntil(
      async () => (await $('[data-testid="familiar-personality-name-0"]').getValue()) === 'Curious',
      { timeout: 10000, timeoutMsg: 'removing trait row 0 should leave the second trait behind' },
    );
    expect(await $('[data-testid="familiar-personality-name-1"]').isExisting()).toBe(false);

    // Same for a bond-invested power.
    await $('[data-testid="familiar-power-add"]').click();
    await set('familiar-power-name-1', 'Shapeshift');
    await $('[data-testid="familiar-power-remove-0"]').click();
    await browser.waitUntil(
      async () => (await $('[data-testid="familiar-power-name-0"]').getValue()) === 'Shapeshift',
      { timeout: 10000, timeoutMsg: 'removing power row 0 should leave the second power behind' },
    );
    expect(await $('[data-testid="familiar-power-name-1"]').isExisting()).toBe(false);

    // And the familiar itself: removing it now asks for confirmation first
    // (S18 — a whole statblock is too much to discard on one click), so
    // confirm before the empty state (the Add button) comes back.
    await $('[data-testid="familiar-remove"]').click();
    await $('[data-testid="familiar-remove-confirm-confirm"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="familiar-remove-confirm-confirm"]').click();
    await $('[data-testid="familiar-add"]').waitForExist({ timeout: 10000 });
    expect(await $('[data-testid="familiar-name"]').isExisting()).toBe(false);
  });
});

// End-to-end: the talisman is an ITEM, not a bare attunement list. Driven through
// the real binary: identity + attunement + instilled effect are all enterable, the
// capacity read-out equals highest Technique + highest Form, and the item-level
// budget is UNMOVED by an instilled effect (the budget non-goal, visible in the UI).
// Then the schema-14 shape on disk, and the legacy-save migration through the
// shipped binary's own Open path.
describe('talisman', () => {
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const ARTS_TAB = '[data-testid="tab-arts"]';
  const POSSESSIONS_TAB = '[data-testid="tab-possessions"]';

  const CAPACITY = '[data-testid="talisman-capacity"]';
  const CAPACITY_NOTE = '[data-testid="talisman-capacity-note"]';
  const ITEM_LEVELS = '[data-testid="item-level-used"]';

  /** Wait for a (debounced) engine read-out to contain every fragment. */
  async function waitForText(selector, ...fragments) {
    await waitUntilExplained(
      async () => {
        const text = clean(await $(selector).getText());
        return fragments.every((f) => text.includes(f));
      },
      10000,
      async () =>
        `${selector} never showed ${fragments.join(' + ')}; read ` +
        `"${clean(await $(selector).getText())}"`,
    );
  }

  it('enters an item, derives its capacity, and leaves the item-level budget alone', async () => {
    await startCharacter('magus');
    await $(VF_TAB).waitForExist({ timeout: 30000 });

    // Creo 5 / Perdo 2 (Techniques) and Corpus 4 / Ignem 3 (Forms) from the shared
    // Art XP pool. Highest Technique 5 + highest Form 4 = 9 pawns; the lower Arts
    // exist so "highest" is actually exercised rather than "the only one bought".
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('200');
    const inc = async (art, times) => {
      const button = await $(`[data-testid="art-inc-${art}"]`);
      await button.waitForExist({ timeout: 5000 });
      for (let i = 0; i < times; i++) await button.click();
    };
    await inc('art.creo', 5);
    await inc('art.perdo', 2);
    await inc('art.corpus', 4);
    await inc('art.ignem', 3);

    // Magic Items tab: no talisman yet.
    await $(POSSESSIONS_TAB).click();
    await $('[data-testid="talisman-empty-item"]').waitForExist({ timeout: 10000 });
    // The item-level budget starts at 0 used — the baseline for the non-goal check.
    await waitForText(ITEM_LEVELS, '0');
    const itemLevelsBefore = clean(await $(ITEM_LEVELS).getText());

    await $('[data-testid="talisman-add"]').click();

    // Identity (shape and material).
    const description = await $('[data-testid="talisman-description"]');
    await description.waitForExist({ timeout: 10000 });
    await description.setValue('An ash staff shod with silver');

    // The capacity read-out: highest Technique 5 + highest Form 4 = 9 pawns, with
    // both contributing scores spelled out in the note.
    await waitForText(CAPACITY, '9');
    await waitForText(CAPACITY_NOTE, '5', '4');

    // An attunement (descriptor + shape bonus).
    await $('[data-testid="talisman-attunement-add"]').click();
    const attunement = await $('[data-testid="talisman-desc-0"]');
    await attunement.waitForExist({ timeout: 5000 });
    await attunement.setValue('Controlling things at a distance');
    await $('[data-testid="talisman-bonus-0"]').setValue('4');

    // An instilled effect at level 15.
    await $('[data-testid="talisman-effect-add"]').click();
    const effectName = await $('[data-testid="talisman-effect-name-0"]');
    await effectName.waitForExist({ timeout: 5000 });
    await effectName.setValue('Wielding the Invisible Sling');
    await $('[data-testid="talisman-effect-level-0"]').setValue('15');

    // THE NON-GOAL, VISIBLE IN THE UI: 15 levels instilled in the talisman must not
    // touch the item-level budget, which belongs to the Redcap-only Virtues and can
    // never fund a magus's talisman. The read-out is unchanged.
    await waitForText(CAPACITY, '9');
    expect(clean(await $(ITEM_LEVELS).getText())).toBe(itemLevelsBefore);

    // Raising Corpus raises the capacity: highest Form becomes 5 → 10 pawns.
    await $(ARTS_TAB).click();
    await $('[data-testid="art-inc-art.corpus"]').click();
    await $(POSSESSIONS_TAB).click();
    await waitForText(CAPACITY, '10');

    // The schema-14 shape reaches disk: a nested talisman, no legacy key.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(23);
    expect(saved.talisman_attunements).toBeUndefined();
    expect(saved.talisman.description).toBe('An ash staff shod with silver');
    expect(saved.talisman.attunements).toEqual([
      { description: 'Controlling things at a distance', bonus: 4 },
    ]);
    expect(saved.talisman.effects).toEqual([{ name: 'Wielding the Invisible Sling', level: 15 }]);
  });

  it('migrates a legacy schema-13 save through the shipped Open path', async () => {
    // The engine-boundary half of this proof lives in `arm-app`'s command tests;
    // this is the UI half — a real Open of a real legacy file in the real binary.
    fs.writeFileSync(
      e2eFile,
      JSON.stringify({
        schema_version: 13,
        ruleset: { id: 'arm5-core', version: '2024.1' },
        entity_kind: 'character',
        type_id: 'magus',
        selections: [{ ref: 'virtue.the_gift' }, { ref: 'virtue.hermetic_magus' }],
        talisman_attunements: [{ description: 'Projecting bolts and missiles', bonus: 3 }],
      }),
    );

    await runDocumentAction('open');
    // The document is dirty from the previous test, so confirm discarding first.
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    // The legacy attunement now lives under the talisman, so the item's own controls
    // are present and the attunement row carries the migrated values.
    await $(POSSESSIONS_TAB).click();
    const attunement = await $('[data-testid="talisman-desc-0"]');
    await attunement.waitForExist({ timeout: 15000 });
    expect(await attunement.getValue()).toBe('Projecting bolts and missiles');
    expect(await $('[data-testid="talisman-bonus-0"]').getValue()).toBe('3');
    // The item exists, so its identity field renders — nothing was invented into it.
    expect(await $('[data-testid="talisman-description"]').getValue()).toBe('');
  });

  // The remove controls are only ever CLICKED here: the panel's unit tests render to
  // a string (SSR), so no handler runs there. A row-remove wired to the wrong list or
  // off by one would delete the player's other row — silent data loss — and every
  // other gate would stay green. Each removal is proved by which row SURVIVES.
  it('removes the clicked row, and then the whole talisman', async () => {
    await $(POSSESSIONS_TAB).click();

    // The migrated attunement plus a second one, then remove the FIRST: the second
    // must survive and slide up to index 0.
    await $('[data-testid="talisman-attunement-add"]').click();
    const second = await $('[data-testid="talisman-desc-1"]');
    await second.waitForExist({ timeout: 10000 });
    await second.setValue('Warding against wood');
    await $('[data-testid="talisman-remove-0"]').click();
    await browser.waitUntil(
      async () =>
        (await $('[data-testid="talisman-desc-0"]').getValue()) === 'Warding against wood',
      {
        timeout: 10000,
        timeoutMsg: 'removing attunement row 0 should leave the second attunement behind',
      },
    );
    expect(await $('[data-testid="talisman-desc-1"]').isExisting()).toBe(false);

    // Same for the instilled effects, which are a separate list on the same item.
    await $('[data-testid="talisman-effect-add"]').click();
    const firstEffect = await $('[data-testid="talisman-effect-name-0"]');
    await firstEffect.waitForExist({ timeout: 10000 });
    await firstEffect.setValue('Lamp Without Flame');
    await $('[data-testid="talisman-effect-add"]').click();
    const secondEffect = await $('[data-testid="talisman-effect-name-1"]');
    await secondEffect.waitForExist({ timeout: 10000 });
    await secondEffect.setValue('Endurance of the Berserkers');
    await $('[data-testid="talisman-effect-remove-0"]').click();
    await browser.waitUntil(
      async () =>
        (await $('[data-testid="talisman-effect-name-0"]').getValue()) ===
        'Endurance of the Berserkers',
      {
        timeout: 10000,
        timeoutMsg: 'removing effect row 0 should leave the second effect behind',
      },
    );
    expect(await $('[data-testid="talisman-effect-name-1"]').isExisting()).toBe(false);

    // And the item itself: removing it now asks for confirmation first (S30
    // — identity, attunements and instilled effects are too much to discard
    // on one click), so confirm before the empty state comes back.
    await $('[data-testid="talisman-remove-item"]').click();
    await $('[data-testid="talisman-remove-confirm-confirm"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="talisman-remove-confirm-confirm"]').click();
    await $('[data-testid="talisman-empty-item"]').waitForExist({ timeout: 10000 });
    expect(await $('[data-testid="talisman-description"]').isExisting()).toBe(false);
  });
});

// End-to-end: the Longevity Ritual is a STORED player-entered value with a live
// suggestion beside it. The point of the slice, driven through the real binary:
// the hint must move when Creo / the aura / a halving Flaw move, while the entered
// bonus must not budge. Also covers the deleted `aura != 0` gate, the halved
// marker, the focus field + sterility note, and that the source radio no longer
// discards the entered bonus.
describe('longevity ritual', () => {
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const ARTS_TAB = '[data-testid="tab-arts"]';
  // The aura is a possession; the ritual is a term of the aging total, so since
  // Slice 3 (#28) it lives on the Aging tab — for every character type, not just a
  // magus. The two tabs are therefore both in play here, and the hint's inputs
  // (Creo/Corpus and the aura) sit on neither of them.
  const POSSESSIONS_TAB = '[data-testid="tab-possessions"]';
  const AGING_TAB = '[data-testid="tab-aging"]';
  const TOTALS_TAB = '[data-testid="tab-totals"]';

  const HINT = '[data-testid="longevity-hint"]';
  const HALVED = '[data-testid="longevity-hint-halved"]';
  const BONUS = '[data-testid="longevity-bonus"]';
  const FOCUS = '[data-testid="longevity-focus"]';

  // The WebDriver keycode for Backspace, sent through Element Send Keys so the webview
  // raises a real `input` event (see the emptying step below for why `clearValue()`
  // will not do).
  const BACKSPACE = String.fromCharCode(0xe003);

  async function hintText() {
    return clean(await $(HINT).getText());
  }

  /**
   * Bring one element into the tab's scrollport, then hand it back.
   *
   * The ritual sits at the far end of the aging surface, and that tab scrolls, so
   * landing on it can leave the panel below the fold — where WebKitWebDriver refuses
   * to drive it ("element not interactable"). `click()` scrolls the element into view
   * itself; `setValue`/`addValue` do not, so anything driven through those is scrolled
   * to first. Mirrors `reach()` in `life-stage-childhood.e2e.js`.
   */
  async function reach(selector) {
    const element = await $(selector);
    await element.waitForExist({ timeout: 10000 });
    // Scroll INSIDE the wait, re-scrolling each poll, rather than scrolling once and
    // then waiting.
    //
    // `block: 'center'` because a bare `scrollIntoView()` aligns to the NEAREST edge,
    // which in a short scrollport leaves the element's centre outside the visible box —
    // and the centre is the point WebKitWebDriver hit-tests, so the interaction is
    // refused even though the element is on screen. The aging surfaces nest three
    // scrollports and the innermost measures 218px against 1340px of content (#34), so
    // there is very little room to be wrong by.
    //
    // And re-scrolling matters because the step's own height is not stable at first
    // paint: the guidance paragraph, the budget bar and the validation footer all fill
    // in from debounced engine round trips, so the box this element sits in can grow
    // AFTER a one-shot scroll has already centred it — which silently un-centres it.
    // Scrolling once and waiting was still marginal for exactly that reason; this
    // element has now failed to be interactable in three specs across four slices
    // (`aging-crisis`, `aging`, and this one twice), so the wait belongs in the shared
    // helper and it has to survive the layout moving under it.
    await browser.waitUntil(
      async () => {
        await element.scrollIntoView({ block: 'center' });
        return element.isClickable();
      },
      { timeout: 10000, timeoutMsg: `${selector} never became clickable, even re-centred` },
    );
    return element;
  }

  /** Wait for the engine's (debounced) read-out to contain every fragment. */
  async function waitForHint(...fragments) {
    await waitUntilExplained(
      async () => {
        const text = await hintText();
        return fragments.every((f) => text.includes(f));
      },
      10000,
      async () => `hint never showed ${fragments.join(' + ')}; read "${await hintText()}"`,
    );
  }

  it('stores the entered bonus while the hint tracks the live Lab Total', async () => {
    await startCharacter('magus');
    await $(VF_TAB).waitForExist({ timeout: 30000 });

    // Creo 5 + Corpus 5 against the shared Art XP pool (15 + 15 XP).
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('200');
    const creoInc = await $('[data-testid="art-inc-art.creo"]');
    await creoInc.waitForExist({ timeout: 5000 });
    for (let i = 0; i < 5; i++) await creoInc.click();
    const corpusInc = await $('[data-testid="art-inc-art.corpus"]');
    for (let i = 0; i < 5; i++) await corpusInc.click();

    // Magic Items tab: aura 5. Then the Aging tab, which owns the ritual.
    await $(POSSESSIONS_TAB).click();
    const auraInput = await $('[data-testid="aura-input"]');
    await auraInput.waitForExist({ timeout: 10000 });
    await auraInput.setValue('5');
    await $(AGING_TAB).click();
    await $('[data-testid="longevity-add"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="longevity-add"]').click();

    // The entered-bonus input exists for a SELF-MADE ritual (it used to be
    // external-only, because a self-made bonus was derived).
    await $(BONUS).waitForExist({ timeout: 10000 });
    expect(await $('[data-testid="longevity-source-self_made"]').isSelected()).toBe(true);
    // Nothing entered yet: the field is empty, not a 0 that reads as a choice.
    expect(await $(BONUS).getValue()).toBe('');
    await $('[data-testid="longevity-not-entered"]').waitForExist({ timeout: 5000 });

    // The hint: Creo 5 + Corpus 5 + Aura 5 = 15 → ceil(15/5) = +3.
    await waitForHint('15', '+3');

    // The totals panel reports the ritual as not entered.
    await $(TOTALS_TAB).click();
    const derived = await $('[data-testid="derived-longevity"]');
    await derived.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => clean(await derived.getText()).includes('not entered'), {
      timeout: 10000,
      timeoutMsg: 'totals panel should report the ritual as not entered',
    });

    // Enter a bonus of 9 — a value no hint in this spec ever suggests, so the two
    // numbers can never be confused.
    await $(AGING_TAB).click();
    await (await reach(BONUS)).setValue('9');
    await browser.waitUntil(
      async () => !(await $('[data-testid="longevity-not-entered"]').isExisting()),
      {
        timeout: 5000,
        timeoutMsg: 'the not-entered flag should clear once a bonus is typed',
      },
    );

    // EMPTYING the box must come back to not-entered, never store a 0. This is the
    // only test at any level that exercises that decision (`raw === '' ? null :
    // Number(raw)` in the panel's `onBonus`): the panel's unit tests render to an
    // SSR string so no handler ever runs there, and both the store test and the
    // "brings the marker back" test call `setLongevityBonus(null)` directly, past the
    // panel. Reverting `onBonus` to `num(event)` — which turns an emptied field into
    // a permanent, irreversible 0 — otherwise leaves every gate green.
    //
    // The field is emptied with a real BACKSPACE keystroke, not `clearValue()`:
    // verified here against the shipped binary, WebDriver's Element Clear blanks the
    // DOM value but dispatches no `input` event, so Svelte never hears about it and
    // the test would pass whatever `onBonus` does. Element Send Keys (what
    // `addValue`/`setValue` use) delivers real key events, which the webview turns
    // into a genuine `input`. The stored bonus is one digit, so one Backspace empties
    // it.
    await (await reach(BONUS)).addValue(BACKSPACE);
    expect(await $(BONUS).getValue()).toBe('');
    await $('[data-testid="longevity-not-entered"]').waitForExist({ timeout: 5000 });

    // A typed 0, by contrast, IS a claim ("this ritual grants nothing"), so the
    // marker must stay gone — the 0-vs-empty distinction, pinned at both ends.
    await (await reach(BONUS)).setValue('0');
    await browser.waitUntil(
      async () => !(await $('[data-testid="longevity-not-entered"]').isExisting()),
      {
        timeout: 5000,
        timeoutMsg: 'a typed 0 is an entered value; the not-entered flag must stay gone',
      },
    );
    expect(await $(BONUS).getValue()).toBe('0');

    // Back to the 9 the rest of the spec asserts on.
    await (await reach(BONUS)).setValue('9');
    await browser.waitUntil(async () => (await $(BONUS).getValue()) === '9', {
      timeout: 5000,
      timeoutMsg: 'the bonus should read 9 again',
    });

    // The totals panel shows the same stored 9 as what it DOES — the modifier
    // subtracted from aging rolls, so "-9", signed exactly once (never "--9") and
    // with the ASCII hyphen, while the editor keeps showing the magnitude 9.
    await $(TOTALS_TAB).click();
    await browser.waitUntil(async () => clean(await derived.getText()).includes('-9'), {
      timeout: 10000,
      timeoutMsg: 'the totals panel should show the stored bonus as the aging-roll modifier',
    });
    const derivedText = clean(await derived.getText());
    expect(derivedText).not.toContain('--');
    expect(derivedText).not.toContain('−');

    // Raise Creo by 1: THE HINT MOVES, THE ENTERED BONUS DOES NOT. This is the
    // whole reason the slice exists.
    await $(ARTS_TAB).click();
    await $('[data-testid="art-inc-art.creo"]').click();
    await $(AGING_TAB).click();
    // Creo 6 + Corpus 5 + Aura 5 = 16 → ceil(16/5) = +4.
    await waitForHint('16', '+4');
    expect(await $(BONUS).getValue()).toBe('9');

    // Aura 0 still suggests a bonus — the removed `aura != 0` gate. The Aura
    // Modifier is a plain addend: 6 + 5 = 11 → ceil(11/5) = +3.
    await $(POSSESSIONS_TAB).click();
    await (await reach('[data-testid="aura-input"]')).setValue('0');
    await $(AGING_TAB).click();
    await waitForHint('11', '+3');
    expect(await $(BONUS).getValue()).toBe('9');

    // Difficult Longevity Ritual halves the Lab Total: 11 → 5 → ceil(5/5) = +1,
    // and the hint is marked as halved.
    await $(VF_TAB).click();
    const addFlaw = await $('[data-testid="add-flaw.difficult_longevity_ritual"]');
    await addFlaw.waitForExist({ timeout: 10000 });
    await addFlaw.click();
    await $(AGING_TAB).click();
    await waitForHint('5', '+1');
    await $(HALVED).waitForExist({ timeout: 5000 });
    expect(await $(BONUS).getValue()).toBe('9');

    // The focus is stored; the sterility consequence the rules attach to it is in the
    // rulebook, not on the panel (manual-testing-findings #21).
    await (await reach(FOCUS)).setValue('A draught of quicksilver at midwinter');
    expect(await $('[data-testid="longevity-sterility-note"]').isExisting()).toBe(false);

    // Switching source keeps the entered bonus and the focus: who made the ritual
    // does not change its value (this used to null the bonus).
    await $('[data-testid="longevity-source-external"]').click();
    expect(await $(BONUS).getValue()).toBe('9');
    // An external ritual gets no suggestion — its bonus came from another magus.
    await browser.waitUntil(async () => !(await $(HINT).isExisting()), {
      timeout: 10000,
      timeoutMsg: 'an external ritual must not show a hint',
    });
    await $('[data-testid="longevity-source-self_made"]').click();
    expect(await $(BONUS).getValue()).toBe('9');
    expect(await $(FOCUS).getValue()).toBe('A draught of quicksilver at midwinter');

    // The stored bonus and focus reach the save file.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.longevity_ritual.source).toBe('self_made');
    expect(saved.longevity_ritual.bonus).toBe(9);
    expect(saved.longevity_ritual.focus).toBe('A draught of quicksilver at midwinter');

    // Removal is only ever CLICKED here — the panel's unit tests render to a string
    // (SSR), so its handler never runs there. The empty state (the Add button) must
    // come back and take the whole editor with it.
    //
    // The file existing above does NOT mean the save is finished: Rust writes it
    // before the frontend's promise resolves, and the shell stays `inert` until
    // that resolves, so this click used to be swallowed (Erika E4, round 3).
    await waitForIdle();
    await $('[data-testid="longevity-remove"]').click();
    await $('[data-testid="longevity-add"]').waitForExist({ timeout: 10000 });
    expect(await $(BONUS).isExisting()).toBe(false);
    expect(await $(FOCUS).isExisting()).toBe(false);
  });
});

// End-to-end: the Markdown character-sheet export (M5.6). Builds a magus in the
// real binary, clicks Export, and reads the `.md` the app wrote off disk. This is
// the only layer that proves the whole chain — the frontend's Fluent label map
// travelling over real IPC, the engine's formatter, and the file write — so it
// asserts entered values, engine-computed read-outs, the deliberately excluded
// sections, both languages, and that an export is not a save.
//
// The native save dialog can't be driven by WebDriver, so the app writes to the
// ARM_E2E_EXPORT_FILE seam (see crates/arm-app/src/commands.rs), a fixed `.md`
// path separate from the JSON save file the other specs round-trip.
describe('markdown export', () => {
  const NAME = 'Marcus of Bonisagus';

  // The two file bindings this describe round-trips, under names matching the
  // rest of the block (the JSON save file vs the exported `.md`).
  const exportFile = e2eExportFile;
  const saveFile = e2eFile;

  // The last section a character with no aging annotations emits, so its presence
  // means the write has fully landed rather than merely started.
  const EN_LAST_SECTION = '## Magic Items';
  const DE_LAST_SECTION = '## Magische Gegenstände';

  // The language control moved into the settings dialog in C4, so the spec-local
  // wrapper is now the shared helper — which opens the dialog, chooses, waits for
  // the value to land, and closes it again.
  const setLang = setLanguage;

  async function clickTab(id) {
    const tab = await $(`[data-testid="tab-${id}"]`);
    await tab.waitForExist({ timeout: 10000 });
    await tab.click();
  }

  /**
   * Click Export and return what landed on disk. The file is deleted first, so
   * "was written" can never pass on a stale export from an earlier assertion.
   */
  async function exportSheet(marker) {
    if (fs.existsSync(exportFile)) fs.unlinkSync(exportFile);
    await runDocumentAction('export');
    await browser.waitUntil(
      () => fs.existsSync(exportFile) && fs.readFileSync(exportFile, 'utf-8').includes(marker),
      { timeout: 15000, timeoutMsg: `export did not write a document containing "${marker}"` },
    );
    return fs.readFileSync(exportFile, 'utf-8');
  }

  /** The body of one `## ` section, so a shared label can be asserted in context. */
  function section(md, heading) {
    const start = md.indexOf(`## ${heading}\n`);
    if (start < 0) throw new Error(`missing section: ${heading}`);
    const rest = md.slice(start + heading.length + 4);
    const end = rest.indexOf('\n## ');
    return end < 0 ? rest : rest.slice(0, end);
  }

  it('builds a magus worth exporting', async () => {
    // A magus of this spec's own, from the startup screen: specs share one app
    // instance, so nothing here may be inherited from the previous file. The
    // language is app-wide state and does carry over, so pin it to English.
    await startCharacter('magus');
    await setLang('en');

    await $('[data-testid="identity-name"]').setValue(NAME);

    // Intelligence +2 and Stamina +2 — the latter is what gives Soak a non-zero
    // total, so the computed Soak section is emitted at all.
    await clickTab('characteristics');
    const intInc = await $('[data-testid="char-inc-int"]');
    await intInc.waitForExist({ timeout: 10000 });
    await intInc.click();
    await intInc.click();
    const staInc = await $('[data-testid="char-inc-sta"]');
    await staInc.click();
    await staInc.click();
    expect(await $('[data-testid="char-value-sta"]').getText()).toBe('+2');

    // A balanced minor Virtue / minor Flaw pair.
    await clickTab('virtues_flaws');
    const addVirtue = await $('[data-testid="add-virtue.keen_vision"]');
    await addVirtue.waitForExist({ timeout: 10000 });
    await addVirtue.click();
    await $('[data-testid="add-flaw.poor_student"]').click();

    // Single Weapon 2 with a specialty — also the Ability the equipped Long Sword
    // rolls, so it shows up again in the computed Combat row.
    await clickTab('abilities');
    const pool = await $('[data-testid="xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('200');
    await $('[data-testid="add-ability.single_weapon"]').click();
    const abilityInc = await $('[data-testid="ability-inc-ability.single_weapon-0"]');
    await abilityInc.waitForExist({ timeout: 5000 });
    await abilityInc.click();
    await abilityInc.click();
    await $('[data-testid="ability-specialty-ability.single_weapon-0"]').setValue('longsword');

    // Creo 5, and Ignem 12 so the Creo Ignem per-spell cap (Cr + Ig + Int +
    // Magic Theory + 3) clears Pilum of Fire's level 20 and its add control is
    // not greyed.
    await clickTab('arts');
    const creoInc = await $('[data-testid="art-inc-art.creo"]');
    await creoInc.waitForExist({ timeout: 10000 });
    for (let i = 0; i < 5; i++) await creoInc.click();
    const ignemInc = await $('[data-testid="art-inc-art.ignem"]');
    for (let i = 0; i < 12; i++) await ignemInc.click();

    await clickTab('spells');
    const addSpell = await $('[data-testid="add-spell.pilum_of_fire"]');
    await addSpell.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => !(await isRowBlocked(addSpell)), {
      timeout: 5000,
      timeoutMsg: 'Creo 5 / Ignem 12 should clear the cap on Pilum of Fire',
    });
    await addSpell.click();

    // A carried weapon. Picking one wields it by default (K5), which is what
    // produces a Combat line, so this only confirms the state rather than
    // changing it.
    await clickTab('equipment');
    const addWeapon = await $('[data-testid="add-weapon.sword_long"]');
    await addWeapon.waitForExist({ timeout: 10000 });
    await addWeapon.click();
    const loadout = await $('[data-testid="equipment-loadout-0"]');
    await loadout.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => (await loadout.getValue()) === 'wielded', {
      timeout: 5000,
      timeoutMsg: 'a newly carried weapon should default to wielded',
    });

    // One magic possession: an assumed aura and an enchanted device.
    await clickTab('possessions');
    const aura = await $('[data-testid="aura-input"]');
    await aura.waitForExist({ timeout: 10000 });
    await aura.setValue('3');
    await $('[data-testid="device-add"]').click();
    const deviceName = await $('[data-testid="device-name-0"]');
    await deviceName.waitForExist({ timeout: 5000 });
    await deviceName.setValue('Ring of the Warding Flame');
    await $('[data-testid="device-level-0"]').setValue('20');
  });

  it('writes the entered values and the engine-computed read-outs', async () => {
    const md = await exportSheet(EN_LAST_SECTION);

    // The H1 is the character's name; the subtitle carries the localized type.
    expect(md.startsWith(`# ${NAME}\n`)).toBe(true);
    expect(md).toContain('*Magus');

    // English section headings, all resolved from the Fluent keys.
    for (const heading of [
      '## Characteristics',
      '## Virtues & Flaws',
      '## Abilities',
      '## Arts',
      '## Spells',
      '## Equipment',
      '## Combat',
      '## Soak',
      EN_LAST_SECTION,
    ]) {
      expect(md).toContain(heading);
    }

    // Entered values: the Ability score with its specialty, the Art, and the spell
    // with its short Art-and-level code (Creo + Ignem + level 20).
    expect(md).toMatch(/\| Single Weapon \| longsword \| 2 \|/);
    expect(md).toMatch(/\| Creo \| 5 \|/);
    expect(md).toMatch(/\| Pilum of Fire \| CrIg 20 \|/);
    // Every Art is on the sheet, not only the two that were scored.
    expect(md).toMatch(/\| Terram \| 0 \|/);
    expect(section(md, 'Magic Items')).toContain('Ring of the Warding Flame');
    // A Virtue row carries its type (the item's category) beside its magnitude.
    expect(md).toMatch(/\| Keen Vision \| General \| Minor \|/);

    // Computed read-outs: the Combat line for the equipped weapon (Attack 10 =
    // Dex 0 + Single Weapon 2 + specialty 1 + weapon 4 ... engine-owned, so only
    // the row's identity is pinned here) and the Soak total from Stamina +2.
    expect(section(md, 'Combat')).toMatch(/\| Long Sword \| Single Weapon \|/);
    const soak = section(md, 'Soak');
    expect(soak).toContain('- **Stamina**: +2');
    expect(soak).toContain('- **Total**: +2');

    // Excluded by design: the sheet carries no Lab, Casting or Penetration totals.
    expect(md).not.toContain('Lab Totals');
    expect(md).not.toContain('Casting Totals');
    expect(md).not.toContain('Penetration');

    // No raw slug may reach the document: every label the engine printed came from
    // the frontend's Fluent bundle, so a key it failed to send would show here.
    for (const slug of [
      'type-magus',
      'param-label-',
      'derived-section-',
      'export-col-',
      'magnitude-',
      'category-',
      'characteristic-int',
      'identity-name',
    ]) {
      expect(md).not.toContain(slug);
    }
  });

  it('writes German headings when the UI is German, and overwrites cleanly', async () => {
    await setLang('de');
    const de = await exportSheet(DE_LAST_SECTION);

    for (const heading of [
      '## Eigenschaften',
      '## Tugenden & Fehler',
      '## Fertigkeiten',
      '## Künste',
      '## Zauber',
      '## Ausrüstung',
      '## Kampf',
      '## Absorption',
      DE_LAST_SECTION,
    ]) {
      expect(de).toContain(heading);
    }
    expect(de).not.toContain('## Characteristics');

    // Restore English before the spec ends: specs share one app instance and run
    // in filename order, so a German session would leak into every later spec.
    await setLang('en');

    // Exporting again over the existing file replaces it rather than appending —
    // one H1 and one Soak section, not two.
    await runDocumentAction('export');
    await browser.waitUntil(() => fs.readFileSync(exportFile, 'utf-8').includes(EN_LAST_SECTION), {
      timeout: 15000,
      timeoutMsg: 'the second export did not replace the German document',
    });
    const md = fs.readFileSync(exportFile, 'utf-8');
    expect(md.match(/^# /gm).length).toBe(1);
    expect(md.match(/^## Soak$/gm).length).toBe(1);
  });

  it('does not mark the document saved, and leaves the current file alone', async () => {
    // Save first, so the document has a known current file and a clean baseline.
    if (fs.existsSync(saveFile)) fs.unlinkSync(saveFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(saveFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    // P3/U3 (`docs/open-todos.md`) moved this state off the retired `doc-status`
    // chip and onto the window title alone — and the character HAS a name
    // (`NAME`, typed in this describe's `before`), so the title shows THAT
    // rather than the file's own name: P3's priority is the character name
    // first, the file name only once the character has none.
    await browser.waitUntil(
      async () => {
        const title = await browser.execute(() => document.title);
        return title.includes(NAME) && !title.startsWith('*');
      },
      {
        timeout: 5000,
        timeoutMsg: 'the title should show the character name with no dirty marker',
      },
    );

    // Edit again so the document is unmistakably dirty, then export.
    await clickTab('characteristics');
    await $('[data-testid="char-inc-int"]').click();
    await browser.waitUntil(
      async () => (await browser.execute(() => document.title)).startsWith('*'),
      { timeout: 5000, timeoutMsg: 'editing should show the dirty marker' },
    );
    const before = await browser.execute(() => document.title);

    await exportSheet(EN_LAST_SECTION);

    // The export changed nothing about the document's identity or dirtiness.
    expect(await browser.execute(() => document.title)).toBe(before);

    // And Save still writes the JSON to the file the export never touched.
    fs.unlinkSync(saveFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(saveFile), {
      timeout: 10000,
      timeoutMsg: 'save after an export did not write the JSON file',
    });
    expect(JSON.parse(fs.readFileSync(saveFile, 'utf-8')).name).toBe(NAME);
  });
});
