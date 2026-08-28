// End-to-end: a MAGUS who lived on past its Gauntlet (slice 6b5). Slice 6b4 built a
// magus that stands AT its Gauntlet — childhood, later life and the fifteen years of
// apprenticeship — and stopped there. This one is 60 and was gauntleted at 25, so it
// has 35 years of life as a magus behind it.
//
// The arithmetic is the rulebook's: "For every year, the magus gets 30 points … Each
// point can be an experience point in an Art or Ability or one level of spell"
// (Ars Magica - Definitive Edition (Core Rules).md:2471), and "For each season that
// your magus spends working on a lab project, the character loses 10 points from the
// yearly 30 experience points, to a minimum of 0 if three or four seasons are spent on
// lab work" (`:2482`). For this magus:
//
//   later life    (25 - 5 - 15) x 15 = 75   — unchanged by the years that followed
//   post-Gauntlet 60 - 25 = 35 years  = 35 x 30 = 1050 points
//   lab work      10 charged seasons  = 1050 - 100 = 950 points
//   the split     300 levels of spells → 650 experience
//   general pool  240 apprenticeship + 650 = 890
//   spell levels  120 profile base + 300 = 420
//
// The five later-life years are the 6b4 regression lock: the Gauntlet age, not the
// character's age, is what apprenticeship ends at, so living on must not lengthen the
// span behind it.
//
// TWO STEPS, NOT ONE (Slice 2 of the guided-creation plan). Every field of the plan —
// age, Gauntlet age, lab seasons, the spell-level split, the native language — is
// typed on the wizard's `experience` step, which mounts `LifeStagePanel` alone, and
// every one of its findings is an `experience`-phase finding, so that step's Next is
// the gate they trip. The XP bar those numbers feed (the general pool's total, the
// post-Gauntlet row) is mounted on the `abilities` step, so the bar is read there,
// once, with the plan settled — deliberately after it rather than beside it, since
// leaving `experience` requires the plan to be legal anyway.
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
const GAUNTLET_AGE_INPUT = '[data-testid="life-stage-gauntlet-age-input"]';
const LAB_SEASONS_INPUT = '[data-testid="life-stage-lab-seasons-input"]';
const SPELL_LEVELS_INPUT = '[data-testid="life-stage-spell-levels-input"]';
const NATIVE_LANGUAGE = '[data-testid="native-language-input"]';
const GAUNTLET_NOTE = '[data-testid="life-stage-gauntlet-note"]';
const SUMMARY = '[data-testid="life-stage-post-gauntlet-summary"]';
const XP_POOL_INPUT = '[data-testid="xp-pool"]';
const XP_POOL_TOTAL = '[data-testid="xp-pool-total"]';
const APPRENTICESHIP = '[data-testid="life-stage-apprenticeship"]';
const LATER_LIFE = '[data-testid="life-stage-later-life"]';
const POST_GAUNTLET = '[data-testid="life-stage-post-gauntlet"]';
const ART_XP_POOL_TOTAL = '[data-testid="art-xp-pool-total"]';
const ART_POST_GAUNTLET = '[data-testid="art-life-stage-post-gauntlet"]';
const SPELL_LEVELS_AVAILABLE = '[data-testid="spell-levels-available"]';
const SPELL_LEVELS_POST_GAUNTLET = '[data-testid="spell-levels-post-gauntlet"]';
const NEXT = '[data-testid="wizard-next"]';
const FINISH = '[data-testid="wizard-finish"]';
const TAB_BAR = '[role="tablist"]';
// The editor's Experience tab is where the funding panel and the plan live —
// mirroring the wizard's `experience` step (guided-creation review #28).
const EXPERIENCE_TAB = '[data-testid="tab-experience"]';
const SPELLS_TAB = '[data-testid="tab-spells"]';
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

/** How many findings of `code` the docked panel is showing. */
async function issueCount(code) {
  const found = await $$(`${DOCKED_ISSUES} [data-code="${code}"]`);
  return found.length;
}

/** One docked finding's sentence and the `data-severity` mirroring its rank. */
async function issue(code) {
  const row = await $(`${DOCKED_ISSUES} [data-code="${code}"]`);
  return { text: clean(await row.getText()), severity: await row.getAttribute('data-severity') };
}

describe('a magus past its Gauntlet', () => {
  it('explains the years after the Gauntlet instead of sending them elsewhere', async () => {
    await startWizard('magus');
    await advanceWizardTo('experience');
    await $(PANEL).waitForExist({ timeout: BOOT_TIMEOUT });

    await $(FUNDING_LIFE_STAGES).click();
    // The plan's fields arrive with it; that it also retires the typed pool is read off
    // the XP bar, on the step that mounts it (see the Abilities step below).
    await $(AGE_INPUT).waitForExist({ timeout: STEP_TIMEOUT });

    // Until 6b5 the note ended "an older magus should use the experience pool
    // instead" — the guided flow disowning the very years this slice models. It must
    // now cost them instead: 30 points a year (`:2471`), said in words.
    const note = await textOf(GAUNTLET_NOTE);
    expect(note).toContain('30');
    expect(note).not.toContain('experience pool');
    // The three fields those points come from are on screen, and the split is one of
    // them — this step owns it (see the Spells step below).
    for (const field of [GAUNTLET_AGE_INPUT, LAB_SEASONS_INPUT, SPELL_LEVELS_INPUT]) {
      expect(await $(field).isExisting()).toBe(true);
    }
  });

  it('ends apprenticeship at the Gauntlet age, not at the age the magus reached', async () => {
    await $(AGE_INPUT).setValue('60');
    await $(GAUNTLET_AGE_INPUT).setValue('25');
    await $(NATIVE_LANGUAGE).setValue('German');

    // 35 years as a magus, worth 30 points each and nothing charged against them yet.
    await browser.waitUntil(async () => (await textOf(SUMMARY)).includes('35 years'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a magus of 60 gauntleted at 25 has lived 35 years as a magus',
    });
    expect(await textOf(SUMMARY)).toContain('1050 points');
    // (Later life and apprenticeship are BLOCKS OF THE POOL, so the XP bar shows them
    // and the Abilities step is where they are read — see below.)
  });

  it('charges lab seasons against the yearly points, and refuses more than the years hold', async () => {
    await $(LAB_SEASONS_INPUT).setValue('10');

    // 10 charged seasons cost 10 points each: 1050 - 100 = 950.
    await browser.waitUntil(async () => (await textOf(SUMMARY)).includes('950 points'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'ten charged lab seasons should take 100 off the 1050',
    });
    // (The XP bar shows the deduction itself, derived from the engine's points rather
    // than recomputed from the season count — read on the Abilities step, which is
    // where the bar is mounted.)

    // Only three seasons a year are ever charged (`:2482`), so 35 years hold 105 and
    // 200 is not a plan — past the cap the extra seasons are simply free, which reads
    // as a bargain unless it is said out loud.
    await $(LAB_SEASONS_INPUT).setValue('200');
    await browser.waitUntil(
      async () => (await issueCount('life_stage_lab_seasons_out_of_range')) === 1,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: '200 lab seasons across 35 years should be refused',
      },
    );
    const refusal = await issue('life_stage_lab_seasons_out_of_range');
    // Reads as a sentence, never as its code, and names the ceiling it is about.
    expect(refusal.text).not.toContain('life_stage_lab_seasons_out_of_range');
    expect(refusal.text).toContain('105');
    expect(refusal.severity).toBe('error');
    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'lab seasons beyond the cap must block the Experience step',
    });

    await $(LAB_SEASONS_INPUT).setValue('10');
    await browser.waitUntil(
      async () => (await issueCount('life_stage_lab_seasons_out_of_range')) === 0,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'restoring ten seasons should clear the refusal',
      },
    );
    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'clearing the lab-season refusal should unblock the step',
    });
  });

  it('splits the points between experience and levels of spells', async () => {
    await $(SPELL_LEVELS_INPUT).setValue('300');

    // 950 points, 300 of them taken as levels of spells, so 650 are experience.
    await browser.waitUntil(async () => (await textOf(SUMMARY)).includes('650 XP'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'taking 300 levels of spells should leave 650 experience',
    });
    expect(await textOf(SUMMARY)).toContain('300 levels');
    // (Those 650 join the general pool — the block that may buy Arts as well as
    // Abilities, `:2435`, `:2471` — which the XP bar reads as 240 + 650 on the
    // Abilities step below.)

    // A split larger than the points there are to divide. Reported on THIS step, which
    // is the deliberate part: the number is typed here, and the magus phase order is
    // experience, abilities, arts, spells — filing it under `spells` would send the
    // wizard forward past the only surface that can correct it. The docked panel is
    // scoped to the current phase, so seeing it here IS the phase attribution.
    await $(SPELL_LEVELS_INPUT).setValue('5000');
    await browser.waitUntil(
      async () => (await issueCount('life_stage_spell_level_split_exceeds_points')) === 1,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'a split beyond the points should be refused on the Experience step',
      },
    );
    const refusal = await issue('life_stage_spell_level_split_exceeds_points');
    expect(refusal.text).not.toContain('life_stage_spell_level_split_exceeds_points');
    expect(refusal.text).toContain('950');
    expect(refusal.severity).toBe('error');
    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'an over-large split must block the Experience step',
    });

    await $(SPELL_LEVELS_INPUT).setValue('300');
    await browser.waitUntil(
      async () => (await issueCount('life_stage_spell_level_split_exceeds_points')) === 0,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'restoring the 300-level split should clear the refusal',
      },
    );
  });

  it('refuses a Gauntlet the magus has not reached yet', async () => {
    // The budget clamps a Gauntlet in the future to the character's age — Advisory and
    // Silent do not block an error, so the arithmetic has to stay sane — which would
    // otherwise cost this magus its 35 years without a word.
    await $(GAUNTLET_AGE_INPUT).setValue('99');
    await browser.waitUntil(
      async () => (await issueCount('life_stage_gauntlet_age_after_age')) === 1,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'a Gauntlet at 99 for a magus of 60 should be refused',
      },
    );
    const refusal = await issue('life_stage_gauntlet_age_after_age');
    expect(refusal.text).not.toContain('life_stage_gauntlet_age_after_age');
    expect(refusal.text).toContain('99');
    expect(refusal.severity).toBe('error');

    await $(GAUNTLET_AGE_INPUT).setValue('25');
    await browser.waitUntil(
      async () =>
        (await issueCount('life_stage_gauntlet_age_after_age')) === 0 &&
        (await textOf(SUMMARY)).includes('35 years'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'restoring the Gauntlet age should clear the refusal and the years with it',
      },
    );
  });

  it('hands the settled plan to the Abilities step as one pool', async () => {
    // The XP bar is the `abilities` step's (Slice 2), so everything the plan above
    // priced is read here, in one place, with the plan final. Reaching this step at all
    // is itself the proof that no life-stage error is left standing: Next and a rail
    // jump alike clamp at a blocking phase.
    await advanceWizardTo('abilities');
    await $(XP_POOL_TOTAL).waitForExist({ timeout: STEP_TIMEOUT });

    // Under life-stage funding the pools are derived, so the editable field is gone.
    expect(await $(XP_POOL_INPUT).isExisting()).toBe(false);
    // 240 of apprenticeship plus the 650 the split left as experience (`:2435`, `:2471`).
    await browser.waitUntil(async () => (await textOf(XP_POOL_TOTAL)) === '890', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the post-Gauntlet experience should join apprenticeship in the general pool',
    });
    // And the bar carries the lab-season deduction itself, derived from the engine's
    // points rather than recomputed from the season count: 1050 - 100 = 950.
    const bar = await textOf(POST_GAUNTLET);
    expect(bar).toContain('100');
    expect(bar).toContain('950');

    // THE 6b4 REGRESSION LOCK, read off the bar's own blocks. Later life stops where
    // apprenticeship begins (`:2214`), and apprenticeship is the fifteen years ending
    // at the GAUNTLET — so this magus lived (25 - 5 - 15) = 5 later-life years worth
    // 75, exactly as it did standing at its Gauntlet. Reading its own age instead
    // would grant 40 years and 600.
    expect(await textOf(LATER_LIFE)).toContain('75');
    // Apprenticeship is a fixed block, untouched by the years that followed it.
    expect(await textOf(APPRENTICESHIP)).toContain('240');

    // Parma Magica, Magic Theory and Latin at 1 (`:2437`) are errors on THIS step for
    // every magus, so they are bought here — both because the flow cannot leave the
    // step until they are and because the steps beyond it are what the tests below are
    // about. (Before Slice 2 they had to be bought early, so that a "Next is disabled"
    // over the plan's numbers could not be theirs; the gate is per phase, and the plan
    // now has a step of its own, so that is no longer a concern.)
    await satisfyMagusMinimums('Latin');
    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the three minimum Abilities should leave the Abilities step unblocked',
    });
  });

  it('lets the Arts step spend the very same pool', async () => {
    await advanceWizardTo('arts');

    // One pool for Abilities and Arts, post-Gauntlet experience included: the Arts bar
    // shows the same 890, and names the block it came from.
    await $(ART_XP_POOL_TOTAL).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(ART_XP_POOL_TOTAL)).toBe('890');
    expect(await $(ART_POST_GAUNTLET).isExisting()).toBe(true);
    expect(await textOf(ART_POST_GAUNTLET)).toContain('650');
  });

  it('shows the Spells step the levels the Experience step gave it', async () => {
    await advanceWizardTo('spells');
    await $(SPELL_LEVELS_AVAILABLE).waitForExist({ timeout: STEP_TIMEOUT });

    // 120 from the type profile plus the 300 the split bought.
    expect(await textOf(SPELL_LEVELS_POST_GAUNTLET)).toContain('300');
    expect(await textOf(SPELL_LEVELS_AVAILABLE)).toContain('420');
    // Read-only here on purpose: the split defines the experience pool three steps
    // back, so editing it from the Spells step would retroactively shrink a pool
    // already spent. The Experience step owns the choice; this step shows what it did.
    expect(await $(SPELL_LEVELS_INPUT).isExisting()).toBe(false);
  });

  it('owes the aging rolls a character over 35 must make, on the step that owns them', async () => {
    await advanceWizardTo('aging');

    // "a character over the age of 35 must make aging rolls … before the game begins"
    // (`:2232`) — which a 60-year-old magus plainly has not. The finding belongs to
    // the AGING phase, whose step is where the age is typed and the rolls are made,
    // so it is counted EXACTLY here: outside Review only the docked panel is mounted
    // and it is filtered to the current phase, which is what makes one the proof.
    await browser.waitUntil(async () => (await issueCount('aging_rolls_pending')) === 1, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a magus of 60 with an empty aging log should be reminded of its aging rolls',
    });
    const pending = await issue('aging_rolls_pending');
    expect(pending.severity).toBe('warning');
    expect(pending.text).not.toContain('aging_rolls_pending');
    expect(pending.text).toContain('60');
    // Advice, not admission: unrolled years must not trap the magus on the step.
    expect(await $(NEXT).isEnabled()).toBe(true);
  });

  it('carries the reminder onto the closing step without gating on it', async () => {
    await advanceWizardTo('review');
    await $(FINISH).waitForExist({ timeout: STEP_TIMEOUT });

    // Counted loosely, not pinned to one: the Review step renders the whole-character
    // panel in its body AND the docked one in the footer, so every finding is on
    // screen twice here. (Every other step shows the docked panel alone, which is why
    // the counts above — the aging step's included — are exact.)
    await browser.waitUntil(async () => (await issueCount('aging_rolls_pending')) > 0, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the closing step should still carry the pending-rolls reminder',
    });
    expect((await issue('aging_rolls_pending')).severity).toBe('warning');
    expect(await $(FINISH).isEnabled()).toBe(true);
  });

  it('finishes into the editor and survives a save and reload', async () => {
    await $(FINISH).click();

    // Finishing lands in the ordinary editor with the character the wizard built.
    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(EXPERIENCE_TAB).click();
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(XP_POOL_TOTAL)).toBe('890');

    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });

    // CHOICES, never resolved values: one Gauntlet age, one season total, one split —
    // every figure above is derived from these four and the age. Asserted exactly, so
    // a stray derived field cannot creep into the save.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    // Plus the stored funding mode, which since schema 16 is a choice of its own.
    expect(saved.ability_funding).toBe('life_stages');
    expect(saved.life_stages).toEqual({
      native_language: 'German',
      gauntlet_age: 25,
      post_gauntlet_lab_seasons: 10,
      post_gauntlet_spell_levels: 300,
    });
    expect(saved.xp_pool ?? 0).toBe(0);
    expect(saved.age).toBe(60);

    // The reload comes AFTER Finish deliberately: loading a file lands in the editor,
    // so there would be no wizard left to advance.
    await $('[data-testid="open-button"]').click();
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(EXPERIENCE_TAB).click();
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });

    // Everything is re-derived from the loaded plan and its stored mode, with no
    // reconciliation step.
    await browser.waitUntil(async () => await $(XP_POOL_TOTAL).isExisting(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a loaded magus plan should restore guided funding',
    });
    expect(await $(XP_POOL_INPUT).isExisting()).toBe(false);
    expect(await textOf(XP_POOL_TOTAL)).toBe('890');
    expect(await $(GAUNTLET_AGE_INPUT).getValue()).toBe('25');
    expect(await $(LAB_SEASONS_INPUT).getValue()).toBe('10');
    expect(await $(SPELL_LEVELS_INPUT).getValue()).toBe('300');
    expect(await textOf(SUMMARY)).toContain('650 XP');
    expect(await textOf(LATER_LIFE)).toContain('75');

    // And the levels half of the split rides along on the Spells tab.
    await $(SPELLS_TAB).click();
    await $(SPELL_LEVELS_AVAILABLE).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(SPELL_LEVELS_POST_GAUNTLET)).toContain('300');
    expect(await textOf(SPELL_LEVELS_AVAILABLE)).toContain('420');
  });
});
