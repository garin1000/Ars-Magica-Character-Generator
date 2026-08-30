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

import { advanceWizardTo, satisfyMagusMinimums, setWizardAge, startWizard } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const PANEL = '[data-testid="life-stage-panel"]';
const FUNDING_LIFE_STAGES = '[data-testid="ability-funding-life_stages"]';
// Slice 12 (#24): this panel shows the age read-only now — the Gauntlet age below is
// measured against it — and the one editable field lives on the `concept` step.
const AGE_READOUT = '[data-testid="age-readout"]';
const NATIVE_LANGUAGE = '[data-testid="native-language-input"]';
const GAUNTLET_NOTE = '[data-testid="life-stage-gauntlet-note"]';
const XP_POOL_INPUT = '[data-testid="xp-pool"]';
const XP_POOL_TOTAL = '[data-testid="xp-pool-total"]';
const APPRENTICESHIP = '[data-testid="life-stage-apprenticeship"]';
const LATER_LIFE = '[data-testid="life-stage-later-life"]';
// Every life-stage chip, in DOM order — the chronology #14 asserts. Scoped to the
// bar, because the life-stage PANEL carries `life-stage-`-prefixed testids of its own.
const LIFE_STAGE_CHIPS = '.xp-summary [data-testid^="life-stage-"]';
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
  return await rowsOf(RESTRICTED);
}

/** Every element matching `selector`, in DOM order, as `{ testid, text }`. */
async function rowsOf(selector) {
  const found = await $$(selector);
  const rows = [];
  for (let i = 0; i < found.length; i++) {
    rows.push({
      testid: await found[i].getAttribute('data-testid'),
      text: clean(await found[i].getText()),
    });
  }
  return rows;
}

/**
 * Where the XP bar sits relative to the box that actually scrolls it, plus whether it
 * owns its own pixels. Read in one round trip, because a geometry read taken piecemeal
 * can straddle two different scroll states.
 *
 * The scrollport is DISCOVERED rather than named: the bar's nearest scrolling
 * ancestor is the box its stickiness is measured against, and asserting against a
 * hardcoded selector would silently start measuring the wrong box the moment the
 * height chain moves.
 */
function stickyBarMetrics() {
  return browser.execute(() => {
    const bar = document.querySelector('.xp-summary');
    let port = bar.parentElement;
    while (port && port.scrollHeight <= port.clientHeight) port = port.parentElement;
    const barRect = bar.getBoundingClientRect();
    const portRect = port.getBoundingClientRect();
    const hit = document.elementFromPoint(barRect.left + 4, barRect.top + barRect.height / 2);
    return {
      barTop: barRect.top,
      barBottom: barRect.bottom,
      portTop: portRect.top,
      portBottom: portRect.bottom,
      portClass: port.className,
      scrollRange: port.scrollHeight - port.clientHeight,
      barPosition: getComputedStyle(bar).position,
      ownsItsPixels: !!hit && (bar === hit || bar.contains(hit)),
    };
  });
}

/** Scroll the bar's own scrollport to the very bottom; returns how far it went. */
function scrollBarPortToBottom() {
  return browser.execute(() => {
    const bar = document.querySelector('.xp-summary');
    let port = bar.parentElement;
    while (port && port.scrollHeight <= port.clientHeight) port = port.parentElement;
    port.scrollTop = port.scrollHeight;
    return port.scrollTop;
  });
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

    // The plan's own fields arrive with it — the age the years are priced from is
    // read out first of all — and the note explains what that age means for a magus.
    // (That the plan also retires the typed pool is read off the XP bar, on the step
    // that mounts it; see the abilities step below.)
    await $(AGE_READOUT).waitForExist({ timeout: STEP_TIMEOUT });
    expect(clean(await $(GAUNTLET_NOTE).getText()).length).toBeGreaterThan(0);
  });

  it('refuses an age younger than the Gauntlet, and takes 25', async () => {
    // Typed on `concept` and priced here (Slice 12, #24): this panel is the surface
    // the age has to REACH, and no longer the one it is entered on.
    await setWizardAge(15);

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

    await setWizardAge(25);
    // The read-out on this step follows the value typed on the other one.
    expect(clean(await $(AGE_READOUT).getText())).toContain('25');
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

    // Under life-stage funding the pools are derived from the stages, so the editable
    // field is gone and the total is read-only: apprenticeship's 240,
    // the pool the spend is charged against (`:2435`).
    expect(await $(XP_POOL_INPUT).isExisting()).toBe(false);
    expect(await textOf(XP_POOL_TOTAL)).toBe('240');
    expect(await textOf(APPRENTICESHIP)).toContain('240');
    // Later life stops at the Gauntlet: (25 - 5 - 15) x 15 = 75, not a companion's 300.
    expect(await textOf(LATER_LIFE)).toContain('75');

    // THE BLOCKS READ AS A CHRONOLOGY (#14). The engine forms three life-stage pools
    // for this magus — childhood's native-language block (which only forms once the
    // language is named), childhood's spread, and, for a magus alone, later life —
    // and each is folded into the chip for its own block rather than listed a second
    // time as a generic restricted row. So the bar shows four chips in the order the
    // rules state the periods (`:2213-2216`, `:2364`) and NO restricted rows at all,
    // this magus having no V/F that grants an experience pool.
    await browser.waitUntil(async () => (await rowsOf(LIFE_STAGE_CHIPS)).length === 3, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the plan should have formed its life-stage chips by the Abilities step',
    });
    const chips = await rowsOf(LIFE_STAGE_CHIPS);
    // Three, not four: this magus stands at its Gauntlet, so there is no block for
    // the years after it.
    expect(chips.map((chip) => chip.testid)).toEqual([
      'life-stage-early-childhood',
      'life-stage-later-life',
      'life-stage-apprenticeship',
    ]);
    // Childhood's two figures under the one heading (`:2378`), in one chip.
    expect(chips[0].text).toContain('Early childhood');
    expect(chips[0].text).toContain('Native language');
    expect(chips[0].text).toContain('0 / 75');
    expect(chips[0].text).toContain('0 / 45');
    // Later life, with the ages it spans and the pool it forms merged into one line:
    // a magus gauntleted at 25 lived ages 5-10 (`:2402`), 5 × 15 = 75.
    expect(chips[1].text).toContain('Later life');
    expect(chips[1].text).toContain('ages 5-10');
    expect(chips[1].text).toContain('0 / 75');
    // No label twice anywhere in the bar — the defect #14 names.
    expect(await restrictedRows()).toEqual([]);
    // Every chip is labelled, never printed as its block slug.
    for (const chip of chips) {
      expect(chip.text).not.toContain('childhood_native_language');
      expect(chip.text).not.toContain('childhood_spread');
      expect(chip.text).not.toContain('later_life');
    }
  });

  // #16 (HIGH — the user had to "change them blindfold"): the bar you are spending
  // against must not scroll off the top. The ability lists below it carry min-height
  // FLOORS (`.region-row` 12rem, `.list-scroll` 6rem) that no shrinking removes, so
  // the step body overflows at the shipped window size and a bar in ordinary flow
  // simply leaves the screen while the player spends against it.
  //
  // Only a real browser with real layout can show this: `render` from `svelte/server`
  // attaches no stylesheet and happy-dom does no layout, so the unit-level guard in
  // `app.css.test.ts` can only assert the stylesheet's TEXT. It also cannot see the
  // failure this test found — `position: sticky` was measured doing NOTHING, because
  // the bar's containing block had been flex-shrunk to a height of 0 while holding
  // 538px of content.
  it('keeps the XP bar on screen while the abilities step scrolls', async () => {
    try {
      const before = await stickyBarMetrics();
      // Sticky at all, and pinned to a box that really scrolls.
      expect(before.barPosition).toBe('sticky');
      const scrolled = await scrollBarPortToBottom();
      // The premise, asserted rather than assumed: if the step stopped scrolling,
      // THIS fails loudly instead of the test passing on a condition never reached.
      expect(scrolled).toBeGreaterThan(0);
      const after = await stickyBarMetrics();
      // Still wholly inside the scrollport after scrolling to the very bottom.
      expect(after.barTop).toBeGreaterThanOrEqual(after.portTop - 1);
      expect(after.barBottom).toBeLessThanOrEqual(after.portBottom + 1);
      // Pinned, not merely still painted somewhere: it stopped travelling with the
      // content, so it moved up by strictly less than the distance scrolled.
      expect(before.barTop - after.barTop).toBeLessThan(scrolled);
      // The rows scroll BEHIND it rather than through it: the bar's own left-hand
      // midpoint hit-tests to the bar, which a transparent bar would not give.
      expect(after.ownsItsPixels).toBe(true);
    } finally {
      await browser.execute(() => {
        const bar = document.querySelector('.xp-summary');
        let port = bar && bar.parentElement;
        while (port && port.scrollTop === 0) port = port.parentElement;
        if (port) port.scrollTop = 0;
      });
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
    // The funding mode is a stored field since schema 16, written always; the plan's
    // presence no longer carries it. Nothing was typed into the pool here.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.ability_funding).toBe('life_stages');
    expect(saved.life_stages).toEqual({ native_language: 'German' });
    expect(saved.xp_pool ?? 0).toBe(0);
    expect(saved.age).toBe(25);

    await $('[data-testid="open-button"]').click();
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(EXPERIENCE_TAB).click();
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });

    // Guided funding is read off the loaded entity's stored mode, apprenticeship and all.
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
