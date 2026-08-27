// End-to-end: building a MAGUS through its life stages (slice 6b4). Slice 6b3
// shipped guided funding but refused it for a magus, because apprenticeship was not
// modelled; now it is, so this spec drives the whole magus narrative from the
// wizard's `experience` step onward.
//
// TWO STEPS, NOT ONE (Slice 2 of the guided-creation plan). The life-stage plan is
// entered on the `experience` step, which mounts `LifeStagePanel` alone, while the XP
// bar it prices — the general pool's total and the restricted blocks — belongs to the
// `abilities` step. The plan's findings are `experience`-phase findings, so the gate
// they trip is that step's Next; the bar figures are therefore read once, from the
// step that owns them, after the plan below is complete. The walk is forward-only:
// leaving `experience` needs the plan legal anyway (`goTo` clamps at the first
// blocking phase exactly as Next does).
//
// The arithmetic is the rulebook's: "The fifteen years of apprenticeship give the
// character 240 experience points … These experience points can be spent on Arts or
// Abilities" (Core Rules.md:2435), which makes apprenticeship the general pool; later
// life stops where it begins (`:2214`), so a magus of 25 lived five later-life years
// worth 75, spendable on Abilities alone; and "Magi must have the following minimum
// Abilities: Parma Magica 1, Magic Theory 1, Latin 1. Characters with lower scores
// would not be admitted to the Order" (`:2437`) is an error on every magus.
//
// NOTE: requires the production binary; the display comes from your desktop session
// or, when DISPLAY is unset, the Xvfb one WebdriverIO starts (see e2e/README.md). The
// wdio `onPrepare` hook builds `target/release/arm-app`, so this cannot run without
// that build step.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { advanceWizardTo, satisfyMagusMinimums, startWizard } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const PANEL = '[data-testid="life-stage-panel"]';
const FUNDING_LIFE_STAGES = '[data-testid="ability-funding-life_stages"]';
const AGE_INPUT = '[data-testid="life-stage-age-input"]';
const NATIVE_LANGUAGE = '[data-testid="native-language-input"]';
const GAUNTLET_NOTE = '[data-testid="life-stage-gauntlet-note"]';
const XP_POOL_INPUT = '[data-testid="xp-pool"]';
const XP_POOL_TOTAL = '[data-testid="xp-pool-total"]';
const APPRENTICESHIP = '[data-testid="life-stage-apprenticeship"]';
const LATER_LIFE = '[data-testid="life-stage-later-life"]';
const RESTRICTED = '[data-testid^="restricted-xp-"]';
const CHECKLIST = '[data-testid="magus-minimums"]';
const SUMMARY = '[data-testid="magus-minimums-summary"]';
const NEXT = '[data-testid="wizard-next"]';
const FINISH = '[data-testid="wizard-finish"]';
const TAB_BAR = '[role="tablist"]';
// The editor's Experience tab is where the funding panel and the plan live —
// mirroring the wizard's `experience` step (guided-creation review #28).
const EXPERIENCE_TAB = '[data-testid="tab-experience"]';
// The Hermetic-minimums checklist rides above the ability lists, so it is on the
// Abilities tab, not the Experience one the plan lives on.
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
// The docked step panel, scoped: other surfaces render `data-code` nodes too.
const DOCKED_ISSUES = '[data-testid="issue-list"]';

const STEP_TIMEOUT = 10000;
const BOOT_TIMEOUT = 30000;

/** Fluent wraps interpolated values in Unicode bidi isolation marks; strip them. */
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

/** The visible, bidi-stripped text of one element. */
async function textOf(selector) {
  return clean(await $(selector).getText());
}

/** How many findings of `code` the docked step panel is showing. */
async function issueCount(code) {
  const found = await $$(`${DOCKED_ISSUES} [data-code="${code}"]`);
  return found.length;
}

/** Every restricted experience pool row on screen, in DOM order. */
async function restrictedRows() {
  const found = await $$(RESTRICTED);
  const rows = [];
  for (let i = 0; i < found.length; i++) {
    rows.push({
      testid: await found[i].getAttribute('data-testid'),
      text: clean(await found[i].getText()),
    });
  }
  return rows;
}

/** One checklist row: its sentence and the `data-met` mirroring its status. */
async function checklistRow(testid) {
  const row = await $(`[data-testid="${testid}"]`);
  return { text: clean(await row.getText()), met: await row.getAttribute('data-met') };
}

describe('magus apprenticeship through the life stages', () => {
  it('offers guided funding to a fresh wizard magus', async () => {
    await startWizard('magus');
    await advanceWizardTo('experience');
    await $(PANEL).waitForExist({ timeout: BOOT_TIMEOUT });

    // 6b3 disabled this option for a magus; 6b4 models the period it was missing.
    expect(await $(FUNDING_LIFE_STAGES).isEnabled()).toBe(true);
    await $(FUNDING_LIFE_STAGES).click();

    // The plan's own fields arrive with it — the age the years are priced from is the
    // first of them — and the note explains what that age means for a magus. (That the
    // plan also retires the typed pool is read off the XP bar, on the step that mounts
    // it; see the abilities step below.)
    await $(AGE_INPUT).waitForExist({ timeout: STEP_TIMEOUT });
    expect(clean(await $(GAUNTLET_NOTE).getText()).length).toBeGreaterThan(0);
  });

  it('refuses an age younger than the Gauntlet, and takes 25', async () => {
    await $(AGE_INPUT).setValue('15');

    // Childhood plus the fifteen years of apprenticeship put the Gauntlet at 20, so 15
    // is not an age a magus can be generated at.
    await browser.waitUntil(
      async () => (await issueCount('life_stage_age_before_gauntlet')) === 1,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'age 15 should be refused as younger than the Gauntlet',
      },
    );
    const refusal = await textOf(`${DOCKED_ISSUES} [data-code="life_stage_age_before_gauntlet"]`);
    // Reads as a sentence, never as its code, and names the floor it is about.
    expect(refusal).not.toContain('life_stage_age_before_gauntlet');
    expect(refusal).toContain('20');
    expect(await $(NEXT).isEnabled()).toBe(false);

    await $(AGE_INPUT).setValue('25');
    await browser.waitUntil(
      async () => (await issueCount('life_stage_age_before_gauntlet')) === 0,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'age 25 should clear the Gauntlet-age refusal',
      },
    );
  });

  it('funds the magus from its apprenticeship, with later life restricted', async () => {
    await $(NATIVE_LANGUAGE).setValue('German');

    // Wait on the engine's answer to the language rather than on a figure the previous
    // test already settled: an unnamed native language is an error on this step, so its
    // going is what says the round trip landed — and what lets the step be left at all.
    await browser.waitUntil(
      async () => (await issueCount('life_stage_native_language_unset')) === 0,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'naming the native language should clear its refusal',
      },
    );

    // Every figure the plan earns is a BLOCK OF THE POOL, and the pool's bar belongs to
    // the `abilities` step (Slice 2), so they are all read there — one step on, with the
    // plan complete, which is also what allows the step to be left in the first place.
    await advanceWizardTo('abilities');
    await $(XP_POOL_TOTAL).waitForExist({ timeout: STEP_TIMEOUT });

    // A plan and a typed pool are mutually exclusive (the engine forbids both at once),
    // so the editable field is gone and the total is read-only: apprenticeship's 240,
    // the pool the spend is charged against (`:2435`).
    expect(await $(XP_POOL_INPUT).isExisting()).toBe(false);
    expect(await textOf(XP_POOL_TOTAL)).toBe('240');
    expect(await textOf(APPRENTICESHIP)).toContain('240');
    // Later life stops at the Gauntlet: (25 - 5 - 15) x 15 = 75, not a companion's 300.
    expect(await textOf(LATER_LIFE)).toContain('75');

    // Three restricted pools now, in the order the engine pushes them: childhood's
    // native-language block (which only forms once the language is named), childhood's
    // spread, and — for a magus alone — later life, Abilities only. The later-life row
    // is therefore `restricted-xp-2` for this character; the indices are read off the
    // DOM rather than assumed, since a V/F granting extra Ability experience would
    // shift them.
    await browser.waitUntil(async () => (await restrictedRows()).length === 3, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the plan should have formed three restricted pools by the Abilities step',
    });
    const rows = await restrictedRows();
    expect(rows.map((row) => row.testid)).toEqual([
      'restricted-xp-0',
      'restricted-xp-1',
      'restricted-xp-2',
    ]);
    expect(rows[0].text).toContain('Native language');
    expect(rows[1].text).toContain('Early childhood');
    expect(rows[2].text).toContain('Later life');
    // Abilities only: apprenticeship is the block that may also buy Arts.
    expect(rows[2].text).toContain('Abilities only');
    expect(rows[2].text).toContain('75');
    // Every row is labelled, never printed as its block slug.
    for (const row of rows) {
      expect(row.text).not.toContain('childhood_native_language');
      expect(row.text).not.toContain('childhood_spread');
      expect(row.text).not.toContain('later_life');
    }
  });

  it('holds the magus to the Hermetic minimums the Order demands', async () => {
    await $(CHECKLIST).waitForExist({ timeout: STEP_TIMEOUT });

    // Three demanded Abilities, none of them bought yet.
    for (const ability of [
      'ability.parma_magica',
      'ability.magic_theory',
      'ability.dead_language',
    ]) {
      const row = await checklistRow(`magus-minimum-${ability}`);
      expect(row.met).toBe('false');
      // Named in words, never as the id — and the status is in the sentence itself,
      // not carried by `data-met` or by colour.
      expect(row.text).not.toContain(ability);
      expect(row.text).toContain('is not met');
    }
    // Seven rows in all: the three of `:2437` plus the four recommended ones of
    // `:2451-2461`, none of them met yet.
    expect(await textOf(SUMMARY)).toContain('7 of 7');

    // And they block the step: "Characters with lower scores would not be admitted to
    // the Order" is an error, on a guided magus and a flat one alike.
    await browser.waitUntil(async () => (await issueCount('magus_minimum_ability')) === 3, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a magus owing all three minimum Abilities should report three errors',
    });
    expect(await $(NEXT).isEnabled()).toBe(false);
  });

  it('lifts the gate once the three are bought', async () => {
    // Parma Magica 1, Magic Theory 1 and a dead language at 1 — Latin, named on the
    // row itself. No experience pool to fill: apprenticeship funds all three (they are
    // Arcane and Academic, which later life may not buy).
    await satisfyMagusMinimums('Latin');

    await browser.waitUntil(async () => (await issueCount('magus_minimum_ability')) === 0, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'buying the three minimum Abilities should clear their errors',
    });
    for (const ability of [
      'ability.parma_magica',
      'ability.magic_theory',
      'ability.dead_language',
    ]) {
      const row = await checklistRow(`magus-minimum-${ability}`);
      expect(row.met).toBe('true');
      expect(row.text).toContain('is met');
    }
    // The dead-language row names the instance the character actually bought, which
    // proves it came from the entity and not from a hardcoded slug.
    expect((await checklistRow('magus-minimum-ability.dead_language')).text).toContain('Latin');

    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'clearing the minimum-Ability errors should unblock the step',
    });
  });

  it('leaves the recommended Abilities as warnings that do not gate', async () => {
    // Latin 4, Magic Theory 3 and Artes Liberales 1 are advice, not admission
    // (`:2451-2461`), so they stay unmet …
    const latin = await checklistRow('magus-recommended-ability.dead_language');
    expect(latin.met).toBe('false');
    expect(latin.text).toContain('is not met');
    expect(await textOf('[data-testid="magus-recommended-hint"]')).toContain('90');

    // … as warnings, with Next still enabled: a warning does not gate a step.
    expect(await issueCount('magus_recommended_ability')).toBeGreaterThan(0);
    expect(await issueCount('magus_minimum_ability')).toBe(0);
    expect(await $(NEXT).isEnabled()).toBe(true);
  });

  it('lets the Arts step spend the same apprenticeship experience', async () => {
    await advanceWizardTo('arts');

    // "These experience points can be spent on Arts or Abilities" (`:2435`): the Arts
    // bar shows the same 240, already charged 15 for the three Abilities.
    await $('[data-testid="art-xp-pool-total"]').waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf('[data-testid="art-xp-pool-total"]')).toBe('240');
    expect(await textOf('[data-testid="art-xp-spent"]')).toBe('15');

    // Creo 5 costs 15 more (triangular), so the pool is charged 30 of its 240.
    const inc = await $('[data-testid="art-inc-art.creo"]');
    await inc.waitForExist({ timeout: STEP_TIMEOUT });
    for (let i = 0; i < 5; i++) await inc.click();
    await browser.waitUntil(async () => (await textOf('[data-testid="art-xp-spent"]')) === '30', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Creo 5 should charge 15 more of the apprenticeship pool',
    });
    expect(await textOf('[data-testid="art-xp-available"]')).toContain('210');
  });

  it('finishes into the editor and survives a save and reload', async () => {
    await advanceWizardTo('review');
    await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });
    await $(FINISH).click();

    // Finishing lands in the ordinary editor with the character the wizard built.
    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(EXPERIENCE_TAB).click();
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(XP_POOL_TOTAL)).toBe('240');

    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });
    // The plan carries the guided mode; there is no separate flag, and no typed pool.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.life_stages).toEqual({ native_language: 'German' });
    expect(saved.xp_pool ?? 0).toBe(0);
    expect(saved.age).toBe(25);

    await $('[data-testid="open-button"]').click();
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(EXPERIENCE_TAB).click();
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });

    // Guided funding is derived from the loaded plan, apprenticeship and all.
    await browser.waitUntil(async () => await $(XP_POOL_TOTAL).isExisting(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a loaded magus plan should restore guided funding',
    });
    expect(await $(XP_POOL_INPUT).isExisting()).toBe(false);
    expect(await textOf(XP_POOL_TOTAL)).toBe('240');
    expect(await textOf(APPRENTICESHIP)).toContain('240');
    expect(await textOf(LATER_LIFE)).toContain('75');
    // The checklist rides along, still met — on the Abilities tab, where it sits
    // above the lists it is about.
    await $(ABILITIES_TAB).click();
    await $('[data-testid="magus-minimum-ability.parma_magica"]').waitForExist({
      timeout: STEP_TIMEOUT,
    });
    expect((await checklistRow('magus-minimum-ability.parma_magica')).met).toBe('true');
  });
});
