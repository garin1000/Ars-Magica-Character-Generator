// End-to-end: the guided wizard's own machinery — rail order, gating, resuming a
// saved mid-flow document, what the flow tells the player, layout stability
// under the first interaction on a step, and the tab area at a short window
// height.
//
// A2 merged five previously separate spec files into this one. Each describe
// keeps its own local selectors and helpers scoped to its block, since several
// of them reused the same constant names (`WIZARD_RAIL`, `FINISH`,
// `NAME_INPUT`) with the same values, and two (`wizard-guidance-findings` and
// `tab-area`) each declare their own differently-behaved `setWindowHeight`
// helper.
//
// `guided guidance and unspent-budget findings (slice 11)` resizes to 1100x800
// with no restore of its own; its `after` hook below puts the window back to
// 1400x900, mirroring `tab area at a short window height`'s own `after` later in
// this file. `tab area at a short window height` already restores the window in
// its own `after` — kept describe-scoped, not hoisted.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import {
  advanceWizardTo,
  BOOT_TIMEOUT,
  clean,
  currentWizardPhase,
  resizeWindowTo,
  returnToStartScreen,
  runDocumentAction,
  satisfyMagusMinimums,
  setLanguage,
  setValidationMode,
  standOnWizardStep,
  startCharacter,
  startWizard,
  STEP_TIMEOUT,
  useFlatPoolFunding,
  wizardRailPhases,
} from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

// End-to-end: the guided creation wizard, driven on a magus — the longest flow
// and the only one that exercises the House step's placement before Virtues &
// Flaws. Covers entry from the startup screen, the rail's order, back/forward
// navigation, per-step gating (and the validation mode that lifts it), and
// finishing into the editor.
describe('guided creation wizard', () => {
  const WIZARD_RAIL = '[data-testid="wizard-rail"]';
  const TAB_BAR = '[role="tablist"]';
  const CHARACTER_TYPE = '[data-testid="character-type"]';
  const BACK = '[data-testid="wizard-back"]';
  const NEXT = '[data-testid="wizard-next"]';
  const FINISH = '[data-testid="wizard-finish"]';
  const NAME_INPUT = '[data-testid="identity-name"]';
  // The balance bar's Flaw half on the Virtues & Flaws step: the surviving home of the
  // Flaw budget after manual-testing-findings #3 removed the banner's restatement of
  // it. It is the surface that SPENDS the budget, which is why it is the one that keeps
  // stating it.
  const BALANCE_FLAWS = '[data-testid="balance-flaws"]';
  // A Minor Virtue with no prerequisites and no Flaws to fund it, so the V/F step
  // carries exactly one deterministic error: `unbalanced_virtues`.
  const ADD_VIRTUE = '[data-testid="add-virtue.keen_vision"]';
  const REMOVE_VIRTUE = '[data-testid^="remove-virtue.keen_vision"]';

  it('walks a magus through its declared phases and finishes in the editor', async () => {
    await startWizard('magus');

    // The wizard replaces the tab bar: the flow's order is the whole point, so
    // there is no way to jump around it.
    await $(WIZARD_RAIL).waitForExist({ timeout: BOOT_TIMEOUT });
    expect(await $(TAB_BAR).isExisting()).toBe(false);

    // It edits a real character, so the banner is above it and the document
    // controls are reachable — the unsaved-changes guard promises the work can be
    // saved rather than lost.
    //
    // "Saving is reachable here" used to be asserted as a rendered save-button.
    // C3c retired the toolbar, and the native menu states availability as an
    // ENABLED ITEM, which WebDriver cannot read — so that claim now lives in
    // `state.svelte.test.ts` ("offers the document-writing actions in the guided
    // wizard too"), against `documentActionEnabled` itself. What is proved HERE,
    // against the real binary, is the stronger thing: a save from the wizard
    // actually writes the file — see `saveAndRead()` in the describe below.
    const label = clean(await $(CHARACTER_TYPE).getText());
    expect(label).toContain('Magus');
    expect(label).not.toContain('type-magus');
    // The validation mode moved into the settings dialog in C4; U2 (P2) then
    // retired the header's own way in to it, so the native menu's Settings item
    // is the only route now, in the wizard as on every other screen.
    expect(await $('[data-testid="settings-button"]').isExisting()).toBe(false);

    // Slice 2 (#1) deleted the read-only `type` step; manual-testing-findings #3 then
    // deleted the explanatory lines its content had been relocated into. The banner
    // names the type and nothing more — no explainer, no budget sentence, no Gift
    // policy line — so the chrome above every step costs one line, not four.
    for (const testid of [
      'character-type-explainer',
      'character-type-budget',
      'character-type-gift-required',
      'character-type-gift-forbidden',
      'character-type-gift-optional',
    ]) {
      expect(await $(`[data-testid="${testid}"]`).isExisting()).toBe(false);
    }
    expect(await $('[data-testid="wizard-step-type"]').isExisting()).toBe(false);

    // The rail follows the ruleset, and the magus profile deliberately places the
    // House step BEFORE Virtues & Flaws — its free Virtue lands in that budget.
    const phases = await wizardRailPhases();
    expect(phases).toContain('house_specialisation');
    expect(phases).toContain('virtues_flaws');
    expect(phases.indexOf('house_specialisation')).toBeLessThan(phases.indexOf('virtues_flaws'));
    // The wizard appends its own closing step, which no profile declares.
    expect(phases.at(-1)).toBe('review');

    // A brand-new character has recorded nothing, so the opening step reads as
    // untouched — in the rail, which is now the only place it is said
    // (manual-testing-findings #2 removed the on-step notice) — while Next stays
    // live: the mark is information, never a gate.
    const conceptStep = await $('[data-testid="wizard-step-concept"]');
    expect(await conceptStep.getAttribute('data-incomplete')).toBe('true');
    await expect($('[data-testid="wizard-incomplete-concept"]')).toExist();
    expect(await $('[data-testid="wizard-incomplete-hint"]').isExisting()).toBe(false);
    expect(await $(NEXT).isEnabled()).toBe(true);

    // guided-creation-review-2026-08 #10: the rail's per-step marker is now
    // SCREEN-READER-ONLY. The flag is `completeness.incomplete_phases`, not "step
    // not opened", so on a fresh character every step carried the words at once.
    // It must still be announced — it is the marker's text, inside the button, that
    // puts "not started" in the step's accessible name — so it is hidden by clipping
    // it to a 1px box, never by leaving the accessibility tree.
    const railMarker = await browser.execute(() => {
      const marker = document.querySelector('[data-testid="wizard-incomplete-concept"]');
      if (!marker) return null;
      const box = marker.getBoundingClientRect();
      const style = getComputedStyle(marker);
      return {
        width: box.width,
        height: box.height,
        text: (marker.textContent ?? '').trim(),
        display: style.display,
        visibility: style.visibility,
        insideStepButton: marker.closest('button')?.dataset.testid ?? null,
      };
    });
    // Takes no visible space …
    expect(railMarker.width).toBeLessThanOrEqual(1);
    expect(railMarker.height).toBeLessThanOrEqual(1);
    // … but is still rendered, still worded, and still part of the step button's
    // accessible name. `display: none` / `visibility: hidden` would drop it from the
    // accessibility tree and leave `data-incomplete` a styling-only channel.
    expect(railMarker.display).not.toBe('none');
    expect(railMarker.visibility).toBe('visible');
    expect(railMarker.text.length).toBeGreaterThan(0);
    expect(railMarker.insideStepButton).toBe('wizard-step-concept');

    // manual-testing-findings #21: no step explains itself. The guidance paragraph
    // that used to head every step is gone from the shipped binary too.
    expect(await $('[data-testid="wizard-guidance"]').isExisting()).toBe(false);

    // Filling the step in clears the mark. The name is also what proves, at the
    // end, that Finish carries the wizard's character over untouched.
    await $(NAME_INPUT).setValue('Marcus of Bonisagus');
    await browser.waitUntil(async () => !(await conceptStep.getAttribute('data-incomplete')), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'naming the character did not clear the untouched mark on the Concept step',
    });

    // Back is dead on the first step; Next advances and the rail follows.
    expect(await $(BACK).isEnabled()).toBe(false);
    expect(await currentWizardPhase()).toBe('concept');
    await advanceWizardTo('house_specialisation');
    expect(await currentWizardPhase()).toBe('house_specialisation');
    expect(await $(BACK).isEnabled()).toBe(true);

    // The Characteristics step behind it was never opened, and Next carried the
    // flow straight over it all the same.
    expect(
      await $('[data-testid="wizard-step-characteristics"]').getAttribute('data-incomplete'),
    ).toBe('true');

    // Walk to Virtues & Flaws and break it: a Minor Virtue with no Flaws to fund
    // it is `unbalanced_virtues`, an error.
    await advanceWizardTo('virtues_flaws');

    // The Flaw budget survived manual-testing-findings #3 on the surface that spends
    // it: the balance bar states the profile's own number, read out of the ruleset
    // rather than written into a translation. This is the assertion that makes
    // deleting the banner's copy of it safe.
    const flawBudget = clean(await $(BALANCE_FLAWS).getText());
    expect(flawBudget).toMatch(/\d/);
    expect(flawBudget).not.toContain('balance-flaws');

    const addVirtue = await $(ADD_VIRTUE);
    await addVirtue.waitForExist({ timeout: STEP_TIMEOUT });
    await addVirtue.click();

    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'an unfunded Virtue did not block the Virtues & Flaws step',
    });
    // The rail marks the offending step, and says why Next is dead.
    const vfStep = await $('[data-testid="wizard-step-virtues_flaws"]');
    expect(await vfStep.getAttribute('data-blocked')).toBe('true');
    await expect($('[data-testid="wizard-blocked-hint"]')).toExist();

    // The step's own findings are shown; a finding from another phase is not.
    await expect($('[data-code="unbalanced_virtues"]')).toExist();
    expect(await $('[data-code="house_unset"]').isExisting()).toBe(false);

    // Advisory downgrades every error to a warning, so the gate lifts with the
    // illegal state standing — the validation mode is the intended escape hatch.
    await setValidationMode('advisory');
    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Advisory mode did not lift the step gate',
    });
    await setValidationMode('enforced');
    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'returning to Enforced did not restore the step gate',
    });

    // Drop the Virtue: the step unblocks.
    await $(REMOVE_VIRTUE).click();
    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'removing the unfunded Virtue did not unblock the step',
    });

    // Walk back two steps, then jump forward through the rail to a step already
    // visited — the two halves of the flow's navigation.
    await $(BACK).click();
    await $(BACK).click();
    const twoBack = await currentWizardPhase();
    await $(`[data-testid="wizard-step-virtues_flaws"]`).click();
    await browser.waitUntil(async () => (await currentWizardPhase()) === 'virtues_flaws', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `a rail jump forward from '${twoBack}' did not land on Virtues & Flaws`,
    });

    // Life-stage funding is not what this test is about (M6/D2 made it the wizard's
    // new default), so it opts back into the typed pool `satisfyMagusMinimums`
    // below expects to fill.
    await useFlatPoolFunding();

    // The Abilities step owes the Order its minimums — Parma Magica 1, Magic Theory 1
    // and Latin 1 (Core Rules.md:2437) — which are blocking errors for every magus, so
    // the walk to the closing step has to settle them (and the experience to pay for
    // them) rather than passing through.
    await advanceWizardTo('abilities');
    await satisfyMagusMinimums();

    // On to the closing step: Next gives way to Finish.
    await advanceWizardTo('review');
    expect(await $(NEXT).isExisting()).toBe(false);
    await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });

    // The closing step names what was walked past — this magus never opened its
    // Arts — and Finish is live regardless: legal, if unfinished.
    await expect($('[data-testid="wizard-review-incomplete-arts"]')).toExist();
    expect(await $(FINISH).isEnabled()).toBe(true);

    await $(FINISH).click();

    // Finishing lands in the ordinary editor, with the character the wizard built.
    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(WIZARD_RAIL).isExisting()).toBe(false);
    expect(await $(NAME_INPUT).getValue()).toBe('Marcus of Bonisagus');
  });
});

// End-to-end: opening an EXISTING character into the guided wizard (Slice 5, #31).
// Two documents, two behaviours, both driven through the shipped binary:
//
//   1. a character saved part-way through the flow reopens on the step it was left
//      on, with the rail still stopping there — the wizard's progress survived the
//      save, which before this slice it never did;
//   2. a character built in the EDITOR opens with every rail step reachable and no
//      gating at all, because it never passed those gates in the first place.
//
// It also checks the unsaved-changes guard's new interaction on the real binary: a
// rail click still costs nothing, while a Next onto a step never reached before
// marks the document changed (the window title's ASCII `*`).
//
// A grog is the subject: the shortest declared flow, and `virtues_flaws` is a step
// a fresh grog can reach with nothing filled in.
describe('opening a saved character into the guided wizard', () => {
  const WIZARD_RAIL = '[data-testid="wizard-rail"]';
  const START_OPEN_WIZARD = '[data-testid="start-open-wizard"]';
  const FINISH = '[data-testid="wizard-finish"]';
  const NAME_INPUT = '[data-testid="identity-name"]';

  // Deliberately longer than the shared helpers.js STEP_TIMEOUT (10000): this spec
  // waits on save/load round trips through the real filesystem (ARM_E2E_FILE), not
  // just DOM updates, so it keeps its own local value rather than importing the
  // shared one (V36).
  const STEP_TIMEOUT = 15000;

  /** Save through the ARM_E2E_FILE seam and hand back the JSON that landed on disk. */
  async function saveAndRead() {
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });
    return JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
  }

  /** Come back to the startup screen and re-enter through the guided-open button. */
  async function reopenIntoWizard() {
    await returnToStartScreen();
    const open = await $(START_OPEN_WIZARD);
    await open.waitForClickable({ timeout: STEP_TIMEOUT });
    await open.click();
    await $(WIZARD_RAIL).waitForExist({ timeout: STEP_TIMEOUT });
  }

  /** Whether the window title shows the ASCII dirty marker (P3/U3 moved this off
   *  the retired `doc-status` chip and onto the title alone). */
  async function isDirty() {
    return (await browser.execute(() => document.title)).startsWith('*');
  }

  it('resumes a mid-flow save on the step it was left on', async () => {
    await startWizard('grog');
    await $(WIZARD_RAIL).waitForExist({ timeout: BOOT_TIMEOUT });

    // A fresh wizard character is clean; the first Next onto a step never reached
    // before is what records progress, and therefore what marks the document
    // changed. That is the DECIDED trade: rail browsing stays free.
    expect(await isDirty()).toBe(false);
    await advanceWizardTo('characteristics');
    await browser.waitUntil(async () => await isDirty(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'advancing a wizard step should mark the document changed',
    });

    await advanceWizardTo('virtues_flaws');
    const saved = await saveAndRead();

    // The progress is in the file, as the phase SLUG — not an index into a phase
    // list that is ruleset data.
    expect(saved.wizard_furthest_phase).toBe('virtues_flaws');
    expect(saved.type_id).toBe('grog');

    await reopenIntoWizard();

    // It lands where it was left, not at the start.
    expect(await currentWizardPhase()).toBe('virtues_flaws');
    // And a just-loaded document is the saved one: no edit, no marker.
    expect(await isDirty()).toBe(false);

    // The rail still stops there: nothing past the stored phase was ever reached
    // during the original run, so nothing past it may be jumped to now.
    const experience = await $(`${WIZARD_RAIL} [data-testid="wizard-step-experience"]`);
    await experience.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await experience.isEnabled()).toBe(false);

    // Rail navigation across the restored steps costs nothing — the whole reason
    // only `furthest` is persisted and never the current step.
    await standOnWizardStep('concept');
    await standOnWizardStep('virtues_flaws');
    expect(await isDirty()).toBe(false);
  });

  it('opens an editor-built character with every step reachable and ungated', async () => {
    await startCharacter('grog');
    await $(NAME_INPUT).setValue('Rolf the Turb');

    // A Minor Virtue with no Flaw to fund it is `unbalanced_virtues`, an error — so
    // this document carries a BLOCKING finding on a step in the middle of the rail.
    // A wizard run would have been held at it; a character assembled in the editor
    // never passed that gate, so the flow must not lock the steps behind it.
    await $('[data-testid="tab-virtues_flaws"]').click();
    const addVirtue = await $('[data-testid="add-virtue.keen_vision"]');
    await addVirtue.waitForExist({ timeout: STEP_TIMEOUT });
    await addVirtue.click();
    await $('[data-testid^="remove-virtue.keen_vision"]').waitForExist({ timeout: STEP_TIMEOUT });

    const saved = await saveAndRead();
    // Nothing walked this character through the flow, so the file records no
    // progress at all — which is exactly what makes it the ungated case.
    expect(saved.wizard_furthest_phase).toBeUndefined();
    expect(saved.selections.some((s) => s.ref === 'virtue.keen_vision')).toBe(true);

    await reopenIntoWizard();

    // Every step of the rail is reachable, not just the first. Collect the refusals
    // and assert on the list, so a failure names the steps that stayed shut instead
    // of only the first one.
    const steps = await $$(`${WIZARD_RAIL} button`);
    expect(steps.length).toBeGreaterThan(1);
    const unreachable = [];
    for (let i = 0; i < steps.length; i++) {
      if (await steps[i].isEnabled()) continue;
      unreachable.push(await steps[i].getAttribute('data-testid'));
    }
    expect(unreachable).toEqual([]);

    // The finding is still SHOWN — the exemption is from enforcement, not from
    // being told. (The closing step's panel is the unfiltered one.)
    await standOnWizardStep('review');
    expect(await currentWizardPhase()).toBe('review');
    await $('[data-code="unbalanced_virtues"]').waitForExist({ timeout: STEP_TIMEOUT });

    // ...and it gates nothing: the jump above crossed the offending step instead of
    // clamping at it, and Finish is live.
    await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(FINISH).isEnabled()).toBe(true);
  });
});

// End-to-end: what the guided flow TELLS the player (Slice 11 —
// guided-creation-review-2026-08 #12, #30), plus the geometry lock that came out of
// the same slice's re-measure.
//
// Four things only the running app can show:
//
//  - The Virtues & Flaws step explains NOTHING. It used to state the character type's
//        own Story and Personality Flaw caps (#7); manual-testing-findings #21 removed
//        the whole `wizard-guidance-*` family, so the caps now reach the player only
//        as validator findings. Checked on two character types, because the paragraph
//        was table-driven and a leftover mount would show up on one type alone.
//  - #12 The Hermetic minimums are one collapsed line that expands on demand, and its
//        count matches the rows it heads (it used to read "N of 7" above a list of
//        three). Only a real disclosure can be opened.
//  - #30 Leaving experience or spell levels unspent raises a WARNING on the owning
//        step and again on Review, and Finish stays available. This is the whole
//        finding pipeline — engine, IPC, phase filter, Fluent — so an `issue-<code>`
//        missing from a locale shows up here as a slug on screen.
//  - The floors `.region-row { min-height }` / `.list-scroll { min-height }` are still
//        load-bearing on the magus Abilities step at 800px, which is what the Slice 11
//        re-measure established (see `app.css`'s comments for the numbers). Locked so
//        the next reader does not have to re-derive it in a ten-minute run.
describe('guided guidance and unspent-budget findings (slice 11)', () => {
  const GUIDANCE = '[data-testid="wizard-guidance"]';
  const MINIMUMS = '[data-testid="magus-minimums"]';
  const SUMMARY = '[data-testid="magus-minimums-summary"]';
  const FINISH = '[data-testid="wizard-finish"]';

  /** The `data-code` of every finding the panel is currently showing. */
  async function shownCodes() {
    const rows = await $$('[data-testid="issue-list"] li');
    const codes = [];
    for (const row of Array.from(rows)) codes.push(await row.getAttribute('data-code'));
    return codes;
  }

  /** Wait until `code` is on screen, and hand back its rendered text. */
  async function waitForFinding(code) {
    let codes = [];
    await browser.waitUntil(
      async () => {
        codes = await shownCodes();
        return codes.includes(code);
      },
      { timeout: STEP_TIMEOUT, timeoutMsg: () => `'${code}' never appeared; showing ${codes}` },
    );
    return clean(await $(`[data-testid="issue-list"] li[data-code="${code}"]`).getText());
  }

  // Shared with the other resizing describes; see `helpers.js` for why the
  // viewport is no longer the same number as the window height.
  const setWindowHeight = (height) => resizeWindowTo(1100, height);

  // A2: this describe used to leave the window at 1100x800 for whatever ran next.
  // Restore the shipped default (crates/arm-app/tauri.conf.json), mirroring
  // `tab area at a short window height`'s own `after` later in this file — "always
  // hand the following specs the window they expect, even on failure".
  after(async () => {
    await browser.setWindowSize(1400, 900);
  });

  // manual-testing-findings #21, in the shipped binary. The paragraph was generated
  // per character type from `flaw_category_caps`, so a grog and a magus went down the
  // same code path with different data — hence both are checked here, and the magus is
  // checked last so the rail is standing on `virtues_flaws` for the suite below.
  //
  // The caps themselves are not lost: they are the validator's
  // `too_many_<category>_flaws` findings, and the finding pipeline is what the #30
  // test at the end of this file exercises end to end.
  it('explains nothing on the Virtues & Flaws step, for either character type', async () => {
    await startWizard('grog');
    await advanceWizardTo('virtues_flaws');
    expect(await $(GUIDANCE).isExisting()).toBe(false);

    await startWizard('magus');
    await advanceWizardTo('virtues_flaws');
    expect(await $(GUIDANCE).isExisting()).toBe(false);
    // The step's own input surface is untouched by the prose removal.
    await expect($('[data-testid="balance-flaws"]')).toExist();
  });

  it('collapses the Hermetic minimums to one line that expands on demand (#12)', async () => {
    await standOnWizardStep('virtues_flaws');
    // Not a life-stage-funding test (M6/D2 made it the wizard's new default).
    await useFlatPoolFunding();
    await advanceWizardTo('abilities');
    await $(MINIMUMS).waitForExist({ timeout: STEP_TIMEOUT });

    // Collapsed: the summary is on screen, the rows are not DISPLAYED. `<details>`
    // keeps them in the DOM, so displayed-ness is the property that matters.
    const parma = await $('[data-testid="magus-minimum-ability.parma_magica"]');
    expect(await $(SUMMARY).isDisplayed()).toBe(true);
    expect(await parma.isDisplayed()).toBe(false);

    // The count heads the whole checklist, so it equals the number of rows inside.
    const summaryText = clean(await $(SUMMARY).getText());
    const numbers = summaryText.match(/\d+/g);
    expect(numbers).toHaveLength(2);
    const rowCount = await browser.execute(
      () =>
        document.querySelectorAll(
          '[data-testid^="magus-minimum-ability."], [data-testid^="magus-recommended-ability."]',
        ).length,
    );
    expect(Number(numbers[1])).toBe(rowCount);
    // A fresh magus meets none of them, so every row is outstanding — which is also
    // what makes the "7 of 7 above a list of 3" bug impossible to reproduce silently.
    expect(Number(numbers[0])).toBe(rowCount);

    // Expanding reveals both groups; collapsing hides them again.
    await $(SUMMARY).click();
    await parma.waitForDisplayed({ timeout: STEP_TIMEOUT });
    expect(await $('[data-testid="magus-recommended-ability.parma_magica"]').isDisplayed()).toBe(
      true,
    );
    await $(SUMMARY).click();
    await parma.waitForDisplayed({ timeout: STEP_TIMEOUT, reverse: true });
  });

  it('keeps the Abilities lists usable at 800px, which is why the floors stay', async () => {
    // The Slice 11 re-measure: `.region-row`'s floor is STILL load-bearing on this
    // very step — removing it took the row from 180px to 54px at 800px and to 0 at
    // 600px, collapsing the Selected list with it. Structural, never a pixel count:
    // both lists must have room for rows.
    await setWindowHeight(800);
    const metrics = await browser.execute(() => {
      const height = (selector) => document.querySelector(selector)?.clientHeight ?? null;
      return {
        regionRow: height('.region-row'),
        listScroll: height('.region-source .list-scroll'),
        selectedScroll: height('.selected-scroll'),
      };
    });
    expect(metrics.regionRow).toBeGreaterThanOrEqual(120);
    expect(metrics.listScroll).toBeGreaterThanOrEqual(60);
    expect(metrics.selectedScroll).toBeGreaterThanOrEqual(60);
  });

  it('warns about unspent experience and spell levels without blocking Finish (#30)', async () => {
    // 30 experience points buys the three minimums at 5 each, so 15 are left over.
    await satisfyMagusMinimums();
    const xpWarning = await waitForFinding('general_xp_unspent');
    expect(xpWarning).not.toContain('issue-general_xp_unspent');
    expect(xpWarning).toContain('15');
    // Factual: a count, and no claim that the points are lost.
    expect(xpWarning.toLowerCase()).not.toContain('wast');
    expect(
      await $('[data-testid="issue-list"] li[data-code="general_xp_unspent"]').getAttribute(
        'data-severity',
      ),
    ).toBe('warning');
    // The Spells step's own budget is not reported here — each warning belongs to the
    // step that holds its budget.
    expect(await shownCodes()).not.toContain('spell_levels_unspent');

    // The whole 120-level grant is untouched on the Spells step.
    await advanceWizardTo('spells');
    const spellWarning = await waitForFinding('spell_levels_unspent');
    expect(spellWarning).not.toContain('issue-spell_levels_unspent');
    expect(spellWarning).toContain('120');
    expect(spellWarning.toLowerCase()).not.toContain('wast');

    // Review renders unfiltered, so both appear there — and neither gates Finish,
    // because a warning is advice and `canFinish` counts errors alone.
    await advanceWizardTo('review');
    await waitForFinding('general_xp_unspent');
    await waitForFinding('spell_levels_unspent');
    // The DISTINCT severities of every row for either code, wherever it is rendered.
    // Distinct, not a positional list: the Review step carries a validation panel of
    // its own AND the docked one below it, both unfiltered on the terminal step, so
    // each finding legitimately appears twice. What must hold is that no rendering of
    // either is an error — count them and the assertion breaks on a layout change
    // rather than on a severity change.
    const severities = await browser.execute(() => [
      ...new Set(
        [
          ...document.querySelectorAll(
            '[data-code="general_xp_unspent"], [data-code="spell_levels_unspent"]',
          ),
        ].map((row) => row.dataset.severity),
      ),
    ]);
    expect(severities).toEqual(['warning']);
    expect(await $(FINISH).isEnabled()).toBe(true);
  });
});

// End-to-end: controls must not move under the pointer (Slice 9 of
// docs/guided-creation-implementation-plan.md — cross-cutting theme 1 of
// docs/guided-creation-review-2026-08, plus #2, #17 and #27).
//
// WHY THIS SPEC EXISTS AT ALL. Every claim here is about MOVEMENT, and no unit test
// can see it: `render` from `svelte/server` attaches no stylesheet and happy-dom does
// no layout, so the component tests can only pin the class hooks and the stylesheet
// contract. Whether a control actually stays where the user aimed is measurable only
// in a real layout engine, which is this suite.
//
// HOW POSITIONS ARE MEASURED, and why not with viewport rects. Every interaction
// below can also SCROLL its surface, and a scroll moves every viewport rect by the
// same amount — which reads as "everything shifted" when nothing was relaid out at
// all. So each measurement is taken relative to a stable ancestor AND has the scroll
// offsets between element and ancestor added back in (`stablePosition`), or is taken
// as an inset inside the element's own parent (`panelInsets`), where both rects move
// together and scroll cancels out by construction. A rect read straight off the
// viewport would make these assertions fail for the wrong reason — or, worse, pass
// for one.
describe('layout stability under first interaction', () => {
  // The wizard screen's root. It is bounded to the window and never scrolls itself, so
  // it is a fixed frame the step's content moves within — exactly what is needed to
  // see a step body shift upward when something above it collapses.
  const WIZARD = '[data-testid="wizard"]';
  // The rail entry for the characteristics step. Its `data-incomplete` attribute is the
  // observable form of the engine's `completeness.incomplete_phases` flip — the state
  // change that used to relay out the step body, now that no on-step notice reacts to it.
  const CHARACTERISTICS_STEP = '[data-testid="wizard-step-characteristics"]';
  const CHAR_PANEL = '[data-testid="char-panel"]';
  const INT_INC = '[data-testid="char-inc-int"]';

  const ARTS_TAB = '[data-testid="tab-arts"]';
  const SPELLS_TAB = '[data-testid="tab-spells"]';
  const CHARACTERISTICS_TAB = '[data-testid="tab-characteristics"]';
  // Two Creo Ignem spells a magus with Creo 1 / Ignem 1 can learn (per-spell cap
  // 1 + 1 + 0 + 0 + 3 = 5): Moonbeam at level 3 and Palm of Flame at level 5. The
  // first is mastered, the second is the control row that must not move with it.
  const MASTERED = 'spell.moonbeam';
  const UNMASTERED = 'spell.palm_of_flame';
  // The selected list's `<ul>`. Both spells share one Technique/Form group, so both
  // rows are items of this one box — a stable horizontal frame for the row geometry.
  const SPELL_LIST = '[data-testid="spell-list"]';

  /**
   * Where `selector` sits in the layout, in pixels relative to `anchor`, with every
   * scroll offset between the two added back in — so the number changes only when
   * something was actually relaid out, never because the surface was scrolled.
   *
   * @param {string} selector element to locate
   * @param {string} anchor a non-scrolling ancestor to measure from
   * @returns {Promise<{top: number, left: number, width: number, height: number}|null>}
   */
  function stablePosition(selector, anchor) {
    return browser.execute(
      (sel, anchorSel) => {
        const element = document.querySelector(sel);
        const frame = document.querySelector(anchorSel);
        if (!element || !frame) return null;
        const origin = frame.getBoundingClientRect();
        const box = element.getBoundingClientRect();
        // Scrolling a container down by N lifts its content's viewport top by N, so
        // adding the offsets back yields the position in the container's own content
        // frame. Without this the assertions below would report a shift for every
        // interaction that happened to scroll the step.
        let scrollTop = 0;
        let scrollLeft = 0;
        for (let node = element.parentElement; node && node !== frame; node = node.parentElement) {
          scrollTop += node.scrollTop;
          scrollLeft += node.scrollLeft;
        }
        return {
          top: Math.round(box.top - origin.top + scrollTop),
          left: Math.round(box.left - origin.left + scrollLeft),
          width: Math.round(box.width),
          height: Math.round(box.height),
        };
      },
      selector,
      anchor,
    );
  }

  /**
   * How `selector` sits inside its own parent's content box: the gap on each side, the
   * parent's inner width, and its own. Insets are differences between two rects that
   * scroll together, so this measurement is scroll-proof by construction.
   *
   * Centred means the two insets are equal; stretched means both are 0 and the widths
   * match.
   *
   * @param {string} selector element to measure
   */
  function panelInsets(selector) {
    return browser.execute((sel) => {
      const element = document.querySelector(sel);
      if (!element || !element.parentElement) return null;
      const parent = element.parentElement;
      const style = getComputedStyle(parent);
      const parentBox = parent.getBoundingClientRect();
      const contentLeft =
        parentBox.left + parseFloat(style.paddingLeft) + parseFloat(style.borderLeftWidth);
      const contentRight =
        parentBox.right - parseFloat(style.paddingRight) - parseFloat(style.borderRightWidth);
      const box = element.getBoundingClientRect();
      return {
        parentClass: parent.className,
        leftInset: Math.round(box.left - contentLeft),
        rightInset: Math.round(contentRight - box.right),
        containerWidth: Math.round(contentRight - contentLeft),
        width: Math.round(box.width),
      };
    }, selector);
  }

  /**
   * Assert a `panelInsets` reading describes a box centred in its container.
   *
   * Centring is only a claim if the box is narrower than the container: a stretched box
   * satisfies "equal insets" with both at zero, which is exactly the state #27 reports.
   *
   * @param {{leftInset: number, rightInset: number, containerWidth: number, width: number}} insets
   */
  function expectCentredInItsContainer(insets) {
    expect(insets.width).toBeLessThan(insets.containerWidth);
    expect(insets.leftInset).toBeGreaterThan(0);
    // 1px of tolerance: an odd amount of free space splits unevenly.
    expect(Math.abs(insets.leftInset - insets.rightInset)).toBeLessThanOrEqual(1);
  }

  /** Raise one Art by `times` clicks (the score is set regardless of XP). */
  async function raiseArt(artId, times) {
    const inc = await $(`[data-testid="art-inc-${artId}"]`);
    await inc.waitForExist({ timeout: STEP_TIMEOUT });
    await inc.waitForClickable({ timeout: STEP_TIMEOUT });
    for (let i = 0; i < times; i++) await inc.click();
  }

  /** Add a catalogue spell to the character on screen. */
  async function addSpell(spellId) {
    const add = await $(`[data-testid="add-${spellId}"]`);
    await add.waitForExist({ timeout: STEP_TIMEOUT });
    await add.waitForClickable({ timeout: STEP_TIMEOUT });
    await add.click();
  }

  // guided-creation-review-2026-08 #2. The click measured here is the one that flips
  // the engine's `completeness.incomplete_phases` for this step — the very first
  // recorded value — and it used to unmount a paragraph above the input surface and
  // drag the stepper out from under the pointer. Characteristics is the worst case of
  // the general pattern because its controls are clicked repeatedly.
  //
  // THE GUARANTEE OUTLIVED ITS ORIGINAL FIX, which is why this test stays. #2 was
  // first answered by keeping the paragraph mounted and reserving its box
  // (`.hidden-reserved`); manual-testing-findings #2 then deleted the paragraph
  // outright, which is a strictly stronger answer — an element that does not exist
  // cannot enter or leave the flow. What must still hold is unchanged and is what is
  // asserted below: on the completeness flip, nothing above the stepper relays out.
  // Any future chrome keyed on the same flag would break this test, which is the
  // point.
  it('does not move the characteristics stepper on the click that completes the step', async () => {
    await startWizard('grog');
    await advanceWizardTo('characteristics');

    const railStep = await $(CHARACTERISTICS_STEP);
    await railStep.waitForExist({ timeout: BOOT_TIMEOUT });
    // Nothing is recorded yet, so the step reads as untouched — and no on-step notice
    // says so, because there is none any more.
    expect(await railStep.getAttribute('data-incomplete')).toBe('true');
    expect(await $('[data-testid="wizard-incomplete-hint"]').isExisting()).toBe(false);

    const inc = await $(INT_INC);
    await inc.waitForExist({ timeout: STEP_TIMEOUT });
    await inc.waitForClickable({ timeout: STEP_TIMEOUT });

    const stepperBefore = await stablePosition(INT_INC, WIZARD);
    expect(stepperBefore).not.toBe(null);

    await inc.click();

    // Wait for the state change that used to move the surface: the phase is complete,
    // so the rail drops its untouched mark.
    await browser.waitUntil(async () => !(await railStep.getAttribute('data-incomplete')), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'recording a Characteristic did not clear the untouched mark on the step',
    });
    expect(await $('[data-testid="char-value-int"]').getText()).toContain('1');

    // The claim of the slice: the control the user clicked has not moved.
    const stepperAfter = await stablePosition(INT_INC, WIZARD);
    expect(stepperAfter).toEqual(stepperBefore);

    // And a second click on the same control still lands on the same control.
    await inc.click();
    await browser.waitUntil(
      async () => (await $('[data-testid="char-value-int"]').getText()).includes('2'),
      { timeout: STEP_TIMEOUT, timeoutMsg: 'a second stepper click did not raise the score' },
    );
    expect(await stablePosition(INT_INC, WIZARD)).toEqual(stepperBefore);
  });

  // guided-creation-review-2026-08 #27. One component, two mounts: the editor puts it
  // straight into `.tab-panel`, which centres its children, while the wizard wraps
  // every step body in `.vf-tab`, which is `width: 100%` and stretches them. The panel
  // now centres itself, so the two agree without either mount arranging it.
  //
  // The two containers are not the same width — the wizard's step body reserves a
  // scrollbar gutter — so "positioned identically" is asserted as the property that
  // holds in both frames: centred inside its own container, and not stretched across
  // it. An absolute left edge would differ by that gutter alone.
  it('centres the characteristics panel in the wizard step and in the editor tab', async () => {
    await startWizard('grog');
    await advanceWizardTo('characteristics');
    await $(CHAR_PANEL).waitForExist({ timeout: BOOT_TIMEOUT });
    const inWizard = await panelInsets(CHAR_PANEL);

    await startCharacter('grog');
    const tab = await $(CHARACTERISTICS_TAB);
    await tab.waitForExist({ timeout: BOOT_TIMEOUT });
    await tab.waitForClickable({ timeout: STEP_TIMEOUT });
    await tab.click();
    await $(CHAR_PANEL).waitForExist({ timeout: STEP_TIMEOUT });
    const inEditor = await panelInsets(CHAR_PANEL);

    // Asserted as two explicit calls rather than a loop, so a failure's line number
    // names the mount that broke.
    expectCentredInItsContainer(inWizard);
    expectCentredInItsContainer(inEditor);
    // The parents really are the two different boxes this is about, so neither mount
    // has quietly grown a wrapper that does the centring for it.
    expect(inWizard.parentClass).toContain('vf-tab');
    expect(inEditor.parentClass).toContain('tab-panel');
  });

  // guided-creation-review-2026-08 #17. The mastery special-abilities picker is much
  // wider than the score spinner. While the two shared a content-width column beside
  // the spell name, reaching mastery 1 widened that column, the elastic name gave
  // ground, and the spinner — another repeated-click control — slid sideways mid-click.
  // The picker now takes a wrap line of its own below the row's controls.
  it('does not move the mastery spinner when a spell is first mastered', async () => {
    await startCharacter('magus');

    // Per-spell level cap = Te + Fo + Int + Magic Theory + 3, so Creo 1 / Ignem 1
    // clears both level-5-and-under spells.
    await $(ARTS_TAB).waitForExist({ timeout: BOOT_TIMEOUT });
    await $(ARTS_TAB).click();
    const artPool = await $('[data-testid="art-xp-pool"]');
    await artPool.waitForExist({ timeout: STEP_TIMEOUT });
    await artPool.waitForClickable({ timeout: STEP_TIMEOUT });
    await artPool.setValue('100');
    await raiseArt('art.creo', 1);
    await raiseArt('art.ignem', 1);

    await $(SPELLS_TAB).click();
    await addSpell(MASTERED);
    await addSpell(UNMASTERED);

    const masteredInc = `[data-testid="spell-mastery-inc-${MASTERED}-0"]`;
    const unmasteredInc = `[data-testid="spell-mastery-inc-${UNMASTERED}-1"]`;
    const abilities = `[data-testid="spell-mastery-abilities-${MASTERED}-0"]`;
    await $(masteredInc).waitForExist({ timeout: STEP_TIMEOUT });
    await $(unmasteredInc).waitForExist({ timeout: STEP_TIMEOUT });
    // Nothing is mastered yet, so no abilities line exists on either row.
    expect(await $(abilities).isExisting()).toBe(false);

    const masteredBefore = await stablePosition(masteredInc, SPELL_LIST);
    const unmasteredBefore = await stablePosition(unmasteredInc, SPELL_LIST);
    expect(masteredBefore).not.toBe(null);
    // Both rows start their controls at the same offset, which is what the rejected
    // `min-width` fix would have changed for every unmastered row at once.
    expect(unmasteredBefore.left).toBe(masteredBefore.left);

    const inc = await $(masteredInc);
    await inc.waitForClickable({ timeout: STEP_TIMEOUT });
    await inc.click();

    // The abilities line appears at mastery 1 — the state change under test.
    await $(abilities).waitForExist({
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'mastery 1 did not bring up the special-abilities picker',
    });
    expect(await $(`[data-testid="spell-mastery-score-${MASTERED}-0"]`).getText()).toContain('1');

    const masteredAfter = await stablePosition(masteredInc, SPELL_LIST);
    const unmasteredAfter = await stablePosition(unmasteredInc, SPELL_LIST);

    // The claim: no HORIZONTAL movement of the clicked control. Its row grows taller
    // by the new line, which is why only the horizontal axis is pinned.
    expect(masteredAfter.left).toBe(masteredBefore.left);
    // And the row below is not indented — nor moved sideways at all.
    expect(unmasteredAfter.left).toBe(unmasteredBefore.left);
    expect(unmasteredAfter.left).toBe(masteredAfter.left);

    // The picker really is a line of its own: it starts left of the controls (at the
    // row's own left edge) and sits below them, rather than beside them where it used
    // to squeeze the name.
    const abilitiesBox = await stablePosition(abilities, SPELL_LIST);
    expect(abilitiesBox.left).toBeLessThan(masteredAfter.left);
    expect(abilitiesBox.top).toBeGreaterThanOrEqual(masteredAfter.top + masteredAfter.height);

    // And it claims the WHOLE line, which is what makes the guarantee independent of
    // the window width. A content-sized picker merely happens to wrap while the row
    // is narrow; widen the window and it would rejoin the controls line and squeeze
    // the name all over again. The 100% basis is what rules that out, so the width is
    // asserted rather than only the wrap. (Verified by removing `flex-basis: 100%`:
    // the assertions above still passed at this window size, this one did not.)
    const listBox = await stablePosition(SPELL_LIST, SPELL_LIST);
    expect(abilitiesBox.width).toBeGreaterThanOrEqual(listBox.width - 2);
  });
});

// End-to-end: a short window must not make the tab area unusable. Found while
// measuring the list tabs' layout — at a window height of 600 the source (Available)
// picker's `.list-scroll` was flex-shrunk to a height of ZERO, so the whole
// selectable list vanished, and the panel's non-shrinkable head (title + filter bar)
// overflowed the panel by ~20px, which `.tab-content`'s `overflow: hidden` then
// clipped with no way to reach it.
//
// Two invariants, both about the same failure: the Available list keeps a usable
// height, and whatever still cannot fit stays REACHABLE — the tab area scrolls
// rather than clipping. Geometry is the subject, so only a real layout engine can
// check it.
describe('tab area at a short window height', () => {
  // The default window is 1400x900 (crates/arm-app/tauri.conf.json — widened from
  // 1100x800 by manual-testing-findings-2026-09-03 #33, so the aging surface's three
  // column tracks fit at the default size). 600 is a plausible short window — a laptop
  // screen with OS panels — and is where the source list measured zero.
  const SHORT_HEIGHT = 600;
  const DEFAULT_SIZE = { width: 1400, height: 900 };

  // Under two source rows: this rejects the collapse, not a particular row height.
  const MIN_USABLE_LIST = 60;

  /** Geometry of the tab area and the source picker's scrolling list. */
  function tabAreaMetrics() {
    return browser.execute(() => {
      const main = document.querySelector('main.tab-content');
      const list = document.querySelector('.region-source .list-scroll');
      const panel = document.querySelector('.region-source .panel');
      const tabbar = document.querySelector('.tabbar');
      if (!main || !list || !panel || !tabbar) return null;
      return {
        mainScrollHeight: main.scrollHeight,
        mainClientHeight: main.clientHeight,
        // `hidden` means anything that does not fit is simply lost: no scrollbar, no
        // keyboard scroll, nothing to drag.
        mainOverflowY: getComputedStyle(main).overflowY,
        listClientHeight: list.clientHeight,
        panelClientHeight: panel.clientHeight,
        panelScrollHeight: panel.scrollHeight,
        // The strip sits above every panel, so its height is subtracted from theirs.
        tabbarScrollHeight: tabbar.scrollHeight,
        tabbarClientHeight: tabbar.clientHeight,
        tabbarRight: tabbar.getBoundingClientRect().right,
        // Whether each tab's own centre hit-tests to that tab: the property a click
        // depends on, and the one an overflow scrollbar's hit area destroys.
        tabsHittableAtCentre: [...tabbar.querySelectorAll('[role="tab"]')].map((tab) => {
          const rect = tab.getBoundingClientRect();
          const hit = document.elementFromPoint(
            rect.left + rect.width / 2,
            rect.top + rect.height / 2,
          );
          return [tab.id, rect.right, hit ? tab.contains(hit) || hit === tab : false];
        }),
        // `.tab` is `overflow: hidden` + `text-overflow: ellipsis` on one nowrap line,
        // so a label wider than its box is exactly a label wearing an ellipsis. Each
        // entry carries the rendered text too, so a failure names the offender rather
        // than only its id.
        tabsTruncated: [...tabbar.querySelectorAll('[role="tab"]')]
          .filter((tab) => tab.scrollWidth > tab.clientWidth + 1)
          .map((tab) => [tab.id, tab.textContent.trim(), tab.scrollWidth, tab.clientWidth]),
        // Both dimensions of the smallest tab, against WCAG 2.5.8's 24x24 CSS px.
        smallestTabBox: [...tabbar.querySelectorAll('[role="tab"]')].reduce(
          (smallest, tab) => {
            const rect = tab.getBoundingClientRect();
            return [Math.min(smallest[0], rect.width), Math.min(smallest[1], rect.height)];
          },
          [Infinity, Infinity],
        ),
      };
    });
  }

  /**
   * The label one tab currently renders, read out of the DOM.
   *
   * NOT `$(...).getText()`, and that is a measured decision rather than a
   * preference. Against the shipped WebKitGTK driver, WebDriver's Get Element Text
   * returns the EMPTY STRING for every `.tab` — and for `.tabbar` as a whole — in
   * English exactly as much as in German, while the same call on the header's New
   * button returns "New" normally. The cause is `.tab`'s `overflow: hidden`
   * (app.css, the ellipsis safety net): this driver treats an `overflow: hidden`
   * box as hiding its own text whether or not anything is actually clipped.
   * Isolated by mutating one property at a time on a live tab: flipping only
   * `overflow` to `visible` made the same `getText()` return "Abilities", whereas
   * removing `role="tab"` or the flex sizing (`min-width`/`flex`) changed nothing.
   * So a `getText()` assertion on a tab can only ever fail, no matter what the app
   * renders.
   *
   * `textContent` is unaffected — it is what `tabsTruncated` above already reads —
   * so the German re-render is confirmed through the same channel that measures it.
   *
   * @param {string} id the tab button's element id (`tab-abilities`)
   * @returns {Promise<string|null>} the trimmed label, or null if the tab is gone
   */
  function tabLabel(id) {
    return browser.execute((tabId) => {
      const tab = document.getElementById(tabId);
      return tab ? tab.textContent.trim() : null;
    }, id);
  }

  const setWindowHeight = (height) => resizeWindowTo(DEFAULT_SIZE.width, height);

  before(async () => {
    // A magus, whose Virtues & Flaws source list is the longest one the picker
    // renders — and freshly created, so the geometry measured below is that of an
    // untouched list rather than one the previous spec had filtered or filled.
    await startCharacter('magus');
    await $('[data-testid="tab-virtues_flaws"]').click();
    await $('.region-source .list-scroll').waitForExist({ timeout: 10000 });
  });

  // Always hand the following specs the window they expect, even on failure.
  after(async () => {
    await setWindowHeight(DEFAULT_SIZE.height);
  });

  it('keeps the Available list usable and anything clipped reachable', async () => {
    await setWindowHeight(SHORT_HEIGHT);
    const m = await tabAreaMetrics();
    expect(m).not.toBe(null);

    // 1. The selectable list did not collapse: a picker with no visible rows is a
    //    picker you cannot pick from.
    expect(m.listClientHeight).toBeGreaterThanOrEqual(MIN_USABLE_LIST);

    // 2. Whatever still does not fit is reachable. The panel's head (title + filter
    //    bar) has a min-content floor no amount of shrinking removes, so the tab area
    //    must scroll to it instead of clipping it away.
    if (m.mainScrollHeight > m.mainClientHeight) {
      expect(m.mainOverflowY).not.toBe('hidden');
    }
  });

  // Slice 3 (#28) took the magus tab count to thirteen, whose labels are ~1350px of
  // text against ~1050px of room. Two failure modes were measured here at the
  // default window, and this test rejects both:
  //
  //  1. Left to the flex defaults the buttons shrink below their text width and the
  //     text wraps INSIDE them, so the strip gains a second line and every panel
  //     below it loses that height — which is what broke the full-height assertion
  //     below by 25px.
  //  2. Making the strip scroll sideways instead fixes the height and breaks the
  //     controls: WebKitGTK's overlay horizontal scrollbar claims the hit area
  //     across the bottom of the scroll container while taking no layout height, so
  //     on a 40px strip everything below ~19px stopped hit-testing to the button and
  //     the lower half of every tab became unclickable.
  //
  // Both invariants are structural — no vertical overflow, every tab hit-testable at
  // its own centre and inside the strip — never a pixel count.
  it('keeps the tab strip one line tall with every tab clickable', async () => {
    await setWindowHeight(DEFAULT_SIZE.height);
    const m = await tabAreaMetrics();

    expect(m.tabbarScrollHeight).toBeLessThanOrEqual(m.tabbarClientHeight + 1);
    // The offenders are collected rather than asserted one by one, so a failure
    // names the tabs instead of stopping at the first.
    expect(m.tabsHittableAtCentre.filter(([, , hit]) => !hit).map(([id]) => id)).toEqual([]);
    expect(
      m.tabsHittableAtCentre.filter(([, right]) => right > m.tabbarRight + 1).map(([id]) => id),
    ).toEqual([]);
  });

  // manual-testing-findings-2026-09 #1: the strip fitting is not the same claim as
  // the strip being ONE LINE — the test above was green throughout, because the
  // labels were ellipsizing rather than wrapping. GERMAN is the binding locale (the
  // magus set is ~146 characters of label against English's ~130), and it is the one
  // no earlier assertion here ever rendered, so English fitting proved nothing about
  // the case that actually failed. `app.css.test.ts` budgets this arithmetically
  // from the same `.ftl` strings; only a real engine can confirm the arithmetic.
  it('shows every tab label in full, in German as well as English', async () => {
    await setWindowHeight(DEFAULT_SIZE.height);

    const english = await tabAreaMetrics();
    expect(english.tabsTruncated).toEqual([]);
    // The type shrank to fit; the button must not have shrunk with it.
    expect(english.smallestTabBox[0]).toBeGreaterThanOrEqual(24);
    expect(english.smallestTabBox[1]).toBeGreaterThanOrEqual(24);

    await setLanguage('de');
    // Wait for a label the two locales spell differently, so the assertions below
    // cannot race the re-render and measure English boxes.
    await browser.waitUntil(async () => (await tabLabel('tab-abilities')) === 'Fertigkeiten', {
      timeout: 5000,
      timeoutMsg: 'the tab strip should re-render in German',
    });

    const german = await tabAreaMetrics();
    expect(german.tabsTruncated).toEqual([]);
    expect(german.tabbarScrollHeight).toBeLessThanOrEqual(german.tabbarClientHeight + 1);
    expect(
      german.tabsHittableAtCentre
        .filter(([, right]) => right > german.tabbarRight + 1)
        .map(([id]) => id),
    ).toEqual([]);
    expect(german.smallestTabBox[0]).toBeGreaterThanOrEqual(24);
    expect(german.smallestTabBox[1]).toBeGreaterThanOrEqual(24);

    // Hand the next spec in this file the language it expects.
    await setLanguage('en');
  });

  it('restores the full-height layout when the window grows back', async () => {
    await setWindowHeight(DEFAULT_SIZE.height);
    const m = await tabAreaMetrics();

    // At the default size everything fits, so no scrollbar appears on the tab area
    // and the list has room for many rows.
    expect(m.mainScrollHeight).toBeLessThanOrEqual(m.mainClientHeight + 1);
    expect(m.listClientHeight).toBeGreaterThan(MIN_USABLE_LIST);
    expect(m.panelScrollHeight).toBeLessThanOrEqual(m.panelClientHeight + 1);
  });

  // U7 (`docs/open-todos.md`, "`.icon-btn` lost its e2e coverage"). `app.css`'s
  // `.icon-btn` rule claims a WCAG 2.5.8 pointer-target floor of 24x24 CSS px —
  // every `×` remove button and every `+`/`-` stepper — but the spec that used
  // to measure the RENDERED button went dark in the 43->10 spec consolidation
  // and nothing noticed: `grep -rn "icon-btn" ui/e2e/` came back empty. The only
  // surviving guard (`app.css.test.ts`) can prove the DECLARED width/height and
  // nothing else — it cannot see a button squeezed by its container, which is
  // exactly the failure a real layout catches and the reason this spec existed.
  // The Characteristics tab's Spinner steppers are used rather than the V/F
  // remove button this describe happens to be sitting on: they render
  // unconditionally on a fresh character, with no dependency on which Virtues
  // happen to be pre-selected.
  it('keeps every icon button at the WCAG 2.5.8 pointer-target floor', async () => {
    await setWindowHeight(DEFAULT_SIZE.height);
    await $('[data-testid="tab-characteristics"]').click();
    await $('.icon-btn').waitForExist({ timeout: 10000 });

    const boxes = await browser.execute(() =>
      [...document.querySelectorAll('.icon-btn')].map((btn) => {
        const rect = btn.getBoundingClientRect();
        return [rect.width, rect.height];
      }),
    );
    expect(boxes.length).toBeGreaterThan(0);
    for (const [width, height] of boxes) {
      expect(width).toBeGreaterThanOrEqual(24);
      expect(height).toBeGreaterThanOrEqual(24);
    }

    // Hand the next spec in this file the tab it expects.
    await $('[data-testid="tab-virtues_flaws"]').click();
  });
});
