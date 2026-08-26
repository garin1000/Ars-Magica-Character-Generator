// End-to-end: funding Abilities from the character's life stages, and taking a
// Sample Childhood package (slice 6b3). Driven on a COMPANION — the type the
// rulebook's 15-experience-per-year later life applies to unchanged, and the one
// whose short phase list puts the Abilities step two clicks away.
//
// The narrative: switch the funding source, type an age and a native language, take
// the Traveling Childhood, and watch the two childhood pools fill. It closes on a
// save/load round-trip (guided funding must survive a reload) and on a magus, which
// slice 6b4 admits to the guided mode as well — built at its Gauntlet, funded by its
// apprenticeship.
//
// TWO STEPS, NOT ONE (Slice 2 of the guided-creation plan). This flow needs three
// surfaces, and the wizard now splits them across two steps:
//
//   `experience` — `LifeStagePanel`: the funding chooser, the plan's fields (age,
//                  native language) and the childhood picker. Every `life_stage_*` and
//                  `childhood_*` finding is an `experience`-phase finding, so this is
//                  the step they gate.
//   `abilities`  — `XpBar` (the pool total and every block and restricted pool it is
//                  made of) plus the Available/Selected lists a package writes rows to.
//
// Until Slice 2 the `abilities` step mounted all three, which is why this spec drove
// it alone; each test now stands on the step that owns what it asserts, through
// `standOnWizardStep`. Moving forward off `experience` needs the plan legal — a rail
// jump clamps at a blocking phase exactly as Next does — which is why the pool figures
// are read after the native language is named rather than before it.
//
// (The editor's Abilities tab mounts all three together even now, through Slice 2's
// temporary `App.svelte` bridge, but it cannot drive this: the panel is an auto-height
// sibling of the lists there, so at the default window the childhood Apply button is
// clipped by `.tab-content`'s `overflow: hidden` and WebDriver reports it as not
// interactable. The reload half below only reads that surface, which works.)
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`, so this cannot run without that build step.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { advanceWizardTo, standOnWizardStep, startWizard } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const PANEL = '[data-testid="life-stage-panel"]';
const FUNDING_POOL = '[data-testid="ability-funding-pool"]';
const FUNDING_LIFE_STAGES = '[data-testid="ability-funding-life_stages"]';
const AGE_INPUT = '[data-testid="life-stage-age-input"]';
const NATIVE_LANGUAGE = '[data-testid="native-language-input"]';
const XP_POOL_INPUT = '[data-testid="xp-pool"]';
const XP_POOL_TOTAL = '[data-testid="xp-pool-total"]';
const LATER_LIFE = '[data-testid="life-stage-later-life"]';
const APPRENTICESHIP = '[data-testid="life-stage-apprenticeship"]';
const GAUNTLET_NOTE = '[data-testid="life-stage-gauntlet-note"]';
const RESTRICTED_0 = '[data-testid="restricted-xp-0"]';
const RESTRICTED_1 = '[data-testid="restricted-xp-1"]';
const PACKAGE_SELECT = '[data-testid="childhood-package-select"]';
const PACKAGE_PREVIEW = '[data-testid="childhood-package-preview"]';
const APPLY = '[data-testid="childhood-apply"]';
const TAKEN = '[data-testid="childhood-taken"]';
const NEXT = '[data-testid="wizard-next"]';
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
const TAB_BAR = '[role="tablist"]';
// The docked findings panel — scoped, because the childhood picker renders the
// engine's package rejections with `data-code` too.
const DOCKED_ISSUES = '[data-testid="issue-list"]';

// Traveling Childhood is the only shipped package with three slots
// (`rules/core/childhoods.json`): two Area Lore regions and one language.
const TRAVELING = 'childhood.traveling';
const AREA_LORE_SCORES = '[data-testid^="ability-score-ability.area_lore-"]';

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

async function issueExists(code) {
  const found = await $$(`${DOCKED_ISSUES} [data-code="${code}"]`);
  return found.length > 0;
}

async function countOf(selector) {
  const found = await $$(selector);
  return found.length;
}

/**
 * The row index of the bought `ability` whose parameter reads `value` — the only
 * way to address one of two rows of the same parameterized Ability (Living
 * Language German vs. Italian).
 */
async function rowIndexFor(ability, value) {
  const prefix = `ability-param-${ability}-`;
  const params = await $$(`[data-testid^="${prefix}"]`);
  for (let i = 0; i < params.length; i++) {
    if ((await params[i].getValue()) === value) {
      const testid = await params[i].getAttribute('data-testid');
      return testid.replace(prefix, '');
    }
  }
  throw new Error(`no bought ${ability} row carries the parameter '${value}'`);
}

/** The bought score shown for the `ability` row whose parameter reads `value`. */
async function scoreOf(ability, value) {
  const index = await rowIndexFor(ability, value);
  return clean(await $(`[data-testid="ability-score-${ability}-${index}"]`).getText());
}

/**
 * Bring one element into the step's scrollport, then hand it back.
 *
 * The `experience` step scrolls (`WizardStep`'s `scroll: true`) and the childhood
 * picker sits at the far end of the panel, so landing on the step leaves the picker
 * below the fold — where WebKitWebDriver refuses to drive it, reporting "element not
 * interactable". `click()` scrolls the element into view itself; `selectByAttribute`
 * (it clicks the `<option>`, not the `<select>`) and `setValue` do not, so anything
 * driven through those is scrolled to first.
 */
async function reach(selector) {
  const element = await $(selector);
  await element.waitForExist({ timeout: STEP_TIMEOUT });
  await element.scrollIntoView();
  return element;
}

/** Every id in an element's `aria-describedby`, in order. */
async function describedByIds(selector) {
  const attribute = await $(selector).getAttribute('aria-describedby');
  return (attribute ?? '').split(/\s+/).filter((id) => id.length > 0);
}

describe('life-stage funding and Sample Childhoods', () => {
  it('offers a typed experience pool by default, with no life-stage plan', async () => {
    await startWizard('companion');
    await advanceWizardTo('experience');
    await $(PANEL).waitForExist({ timeout: BOOT_TIMEOUT });

    // Flat funding is the default and must stay byte-identical: the chooser stands on
    // the typed pool, so the plan's own fields are not on the step at all.
    expect(await $(FUNDING_POOL).isSelected()).toBe(true);
    expect(await $(AGE_INPUT).isExisting()).toBe(false);
    // Every character type may be built either way, so the guided option is live.
    expect(await $(FUNDING_LIFE_STAGES).isEnabled()).toBe(true);

    // And the pool that default is named for is on the next step, editable, with no
    // life-stage row and no restricted block anywhere near it.
    await advanceWizardTo('abilities');
    expect(await $(XP_POOL_INPUT).getTagName()).toBe('input');
    expect(await $(XP_POOL_TOTAL).isExisting()).toBe(false);
    expect(await $(LATER_LIFE).isExisting()).toBe(false);
    expect(await $(RESTRICTED_0).isExisting()).toBe(false);
  });

  it('takes an age and refuses a plan with no native language', async () => {
    await standOnWizardStep('experience');
    await $(FUNDING_LIFE_STAGES).click();
    // The plan's fields arrive with it.
    await $(AGE_INPUT).waitForExist({ timeout: STEP_TIMEOUT });
    await $(AGE_INPUT).setValue('25');
    // Five years of childhood, said on the step that prices them.
    const ageCap = await $('[data-testid="life-stage-age-cap"]');
    await ageCap.waitForExist({ timeout: STEP_TIMEOUT });
    expect(clean(await ageCap.getText())).toContain('5');

    // An unnamed native language is an error, not a nicety: the engine needs the name
    // to instantiate the right parameterized Living Language and to enforce the
    // childhood spread's "other than the character's native language" clause. The
    // docked panel is scoped to the current phase, so seeing it here IS the phase
    // attribution — since Slice 2 these findings belong to `experience`.
    await browser.waitUntil(async () => await issueExists('life_stage_native_language_unset'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a plan without a native language must report life_stage_native_language_unset',
    });
    // The finding reads as a sentence, never as its code.
    const unset = await textOf(`${DOCKED_ISSUES} [data-code="life_stage_native_language_unset"]`);
    expect(unset.length).toBeGreaterThan(0);
    expect(unset).not.toContain('life_stage_native_language_unset');
    // And it gates the step, exactly like any other phase finding.
    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the unset native language did not block the Experience step',
    });

    await $(NATIVE_LANGUAGE).setValue('German');

    // The error goes and the gate lifts; what remains is the warning that those 75
    // points are still unspent — which a Childhood package is about to spend.
    await browser.waitUntil(async () => !(await issueExists('life_stage_native_language_unset')), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'naming the native language did not clear the error',
    });
    await browser.waitUntil(
      async () => await issueExists('life_stage_native_language_missing_score'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'a named but unbought native language should warn that its block is unspent',
      },
    );
    await browser.waitUntil(async () => await $(NEXT).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'naming the native language did not re-enable Next',
    });
  });

  it('counts later life from the age and shows the two childhood blocks', async () => {
    // The pool and every block it is made of belong to the XP bar, one step on.
    await standOnWizardStep('abilities');
    await $(XP_POOL_TOTAL).waitForExist({ timeout: STEP_TIMEOUT });
    // Under a plan the engine forbids a typed pool, so the input gives way to a
    // read-only total. Later life is (age - childhood years) x 15 = (25 - 5) x 15 = 300.
    await browser.waitUntil(async () => (await textOf(XP_POOL_TOTAL)) === '300', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'age 25 should earn a companion 300 later-life experience',
    });
    expect(await $(XP_POOL_INPUT).isExisting()).toBe(false);
    const laterLife = await textOf(LATER_LIFE);
    expect(laterLife).toContain('20');
    expect(laterLife).toContain('15');
    expect(laterLife).toContain('300');

    // A fresh companion has no restricted-pool Virtue (no Educated, Warrior or
    // Privileged Upbringing), so the only restricted pools on screen are the two
    // childhood blocks and their indices are deterministic: the 75-point native
    // language block, restricted to that one Ability instance, then the 45-point
    // spread. Both localized, never the raw block slug.
    const native = await textOf(RESTRICTED_0);
    expect(native).toContain('0 / 75');
    expect(native).toContain('Native language');
    expect(native).not.toContain('childhood_native_language');
    const spread = await textOf(RESTRICTED_1);
    expect(spread).toContain('0 / 45');
    expect(spread).toContain('Early childhood');
    expect(spread).not.toContain('childhood_spread');
    // REGRESSION LOCK for the 6b4 pool restructuring: a magus's later life became a
    // restricted, Abilities-only pool of its own (its general pool is apprenticeship
    // instead), and that must NOT have happened to anyone else. For a companion later
    // life IS the general pool, so childhood's two blocks are the whole of it and there
    // is no third restricted row.
    expect(await $('[data-testid="restricted-xp-2"]').isExisting()).toBe(false);
  });

  it('previews a drafted childhood through the plan and refuses an unanswered slot', async () => {
    await standOnWizardStep('experience');
    await (await reach(PACKAGE_SELECT)).selectByAttribute('value', TRAVELING);
    await $(PACKAGE_PREVIEW).waitForExist({ timeout: STEP_TIMEOUT });

    const preview = await textOf(PACKAGE_PREVIEW);
    // The native entry is resolved through the plan's language: never the
    // `{language}` token, never the ability id.
    expect(preview).toContain('German (Living Language) 5');
    expect(preview).not.toContain('{language}');
    expect(preview).not.toContain('ability.living_language');
    // An unanswered slot falls back to the localized parameter hint.
    expect(preview).toContain('(Area) Lore 1');
    expect(preview).not.toContain('{area}');

    // Nothing is written while a slot is unanswered, and the button says why through
    // `aria-describedby` rather than being a silently grey control.
    expect(await $(APPLY).isEnabled()).toBe(false);
    const ids = await describedByIds(APPLY);
    expect(ids).toContain('childhood-apply-reason');
    // The id resolves to a real, spoken reason — not a dangling reference.
    const reason = await textOf('#childhood-apply-reason');
    expect(reason.length).toBeGreaterThan(0);
    expect(reason).not.toContain('childhood-slot-empty-reason');
  });

  it('refuses a childhood language that repeats the native one', async () => {
    await (await reach('[data-testid="childhood-slot-area_a"]')).setValue('Rhine');
    await (await reach('[data-testid="childhood-slot-area_b"]')).setValue('Provence');
    // The spread buys a Living Language "other than the character's native
    // language", so German is not an answer here.
    await (await reach('[data-testid="childhood-slot-language"]')).setValue('German');

    const languageSlot = '[data-testid="childhood-slot-language"]';
    await browser.waitUntil(
      async () => (await $(languageSlot).getAttribute('aria-invalid')) === 'true',
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'the native language repeated in the childhood language slot must be flagged',
      },
    );
    // Flagged in words as well as by state, never by colour alone.
    const slotReason = await textOf('[data-testid="childhood-slot-language-reason"]');
    expect(slotReason.length).toBeGreaterThan(0);
    expect(await $(APPLY).isEnabled()).toBe(false);

    await (await reach(languageSlot)).setValue('Italian');
    await browser.waitUntil(async () => await $(APPLY).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a distinct childhood language should let the package be taken',
    });
    expect(await $(languageSlot).getAttribute('aria-invalid')).not.toBe('true');
  });

  it('takes the package, filling both childhood blocks exactly', async () => {
    await $(APPLY).click();

    // The native language is bought now, so the unspent warning is gone.
    await browser.waitUntil(
      async () => !(await issueExists('life_stage_native_language_missing_score')),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'buying the native language should retire the unspent-block warning',
      },
    );
    // And the package is recorded on the character, named in words.
    const taken = await textOf(TAKEN);
    expect(taken).toContain('Traveling Childhood');
    expect(taken).not.toContain(TRAVELING);

    // What it bought is on the Abilities step: German 5 costs the whole 75-point
    // native block, and the spread's 45 buy the two Area Lores, Folk Ken, Survival
    // and Italian.
    await standOnWizardStep('abilities');
    await $(RESTRICTED_0).waitForExist({ timeout: STEP_TIMEOUT });
    await browser.waitUntil(async () => (await textOf(RESTRICTED_0)).includes('75 / 75'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the native-language block should be fully spent by the package',
    });
    expect(await textOf(RESTRICTED_1)).toContain('45 / 45');

    expect(await scoreOf('ability.living_language', 'German')).toBe('5');
    expect(await scoreOf('ability.living_language', 'Italian')).toBe('1');
    expect(await countOf(AREA_LORE_SCORES)).toBe(2);
    expect(await scoreOf('ability.area_lore', 'Rhine')).toBe('1');
    expect(await scoreOf('ability.area_lore', 'Provence')).toBe('1');
  });

  it('is idempotent: taking the same package twice adds no second set of rows', async () => {
    const before = await countOf(AREA_LORE_SCORES);

    await standOnWizardStep('experience');
    await $(APPLY).click();

    // A package raises scores monotonically, so re-taking it must not duplicate a
    // row for the same Ability and answer.
    await standOnWizardStep('abilities');
    await browser.waitUntil(async () => (await countOf(AREA_LORE_SCORES)) === before, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 're-taking the package changed the Area Lore row count',
    });
    expect(await scoreOf('ability.area_lore', 'Rhine')).toBe('1');
    expect(await textOf(RESTRICTED_0)).toContain('75 / 75');
  });

  it('keeps the bought rows when the funding source is switched back and forth', async () => {
    await standOnWizardStep('experience');
    await $(FUNDING_POOL).click();
    // The plan is gone, so its fields go with it.
    await browser.waitUntil(async () => !(await $(AGE_INPUT).isExisting()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'leaving guided funding should retire the plan fields',
    });

    // The editable total is back — at 0, which the input renders as its empty
    // placeholder — and every life-stage block with it.
    await standOnWizardStep('abilities');
    await $(XP_POOL_INPUT).waitForExist({ timeout: STEP_TIMEOUT });
    expect(['', '0']).toContain(await $(XP_POOL_INPUT).getValue());
    expect(await $(XP_POOL_TOTAL).isExisting()).toBe(false);
    expect(await $(LATER_LIFE).isExisting()).toBe(false);
    expect(await $(RESTRICTED_0).isExisting()).toBe(false);

    // The work survives: the rows the package wrote are still there, and the
    // now-unfunded spend is reported rather than silently wiped.
    expect(await countOf(AREA_LORE_SCORES)).toBe(2);
    expect(await scoreOf('ability.living_language', 'German')).toBe('5');
    await browser.waitUntil(async () => await issueExists('not_enough_xp'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a pool of 0 against 120 experience of rows must report not_enough_xp',
    });

    await standOnWizardStep('experience');
    await $(FUNDING_LIFE_STAGES).click();
    await $(AGE_INPUT).waitForExist({ timeout: STEP_TIMEOUT });
    // A new plan starts empty rather than resurrecting the dropped one, so the
    // native language has to be named again — and no drafted slot shows a fault.
    expect(await $(NATIVE_LANGUAGE).getValue()).toBe('');
    expect(await countOf('[data-testid^="childhood-slot-"][data-testid$="-reason"]')).toBe(0);
    // The draft went with the plan it belonged to: the select is back on its prompt
    // option, so nothing prefills a package for a plan that starts from nothing.
    expect(await $(PACKAGE_SELECT).getValue()).toBe('');
    expect(await $(PACKAGE_PREVIEW).isExisting()).toBe(false);

    // Put the plan back the way the save below should record it.
    await $(AGE_INPUT).setValue('25');
    await $(NATIVE_LANGUAGE).setValue('German');

    // The rows still stand through the second switch, and the restored plan funds them
    // again — the childhood block is spent by the scores that are already there.
    await standOnWizardStep('abilities');
    await $(RESTRICTED_0).waitForExist({ timeout: STEP_TIMEOUT });
    await browser.waitUntil(async () => (await textOf(RESTRICTED_0)).includes('75 / 75'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'restoring the native language did not re-fund the childhood block',
    });
    expect(await countOf(AREA_LORE_SCORES)).toBe(2);
  });

  it('restores guided funding and the recorded package from a save', async () => {
    // Leaving the guided mode dropped the plan, and both the recorded package (the
    // plan IS the record) and the draft went with it, so re-take it from the select
    // and the slots.
    await standOnWizardStep('experience');
    await (await reach(PACKAGE_SELECT)).selectByAttribute('value', TRAVELING);
    await (await reach('[data-testid="childhood-slot-area_a"]')).setValue('Rhine');
    await (await reach('[data-testid="childhood-slot-area_b"]')).setValue('Provence');
    await (await reach('[data-testid="childhood-slot-language"]')).setValue('Italian');
    await browser.waitUntil(async () => await $(APPLY).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the re-filled package should be takeable again',
    });
    await $(APPLY).click();
    await $(TAKEN).waitForExist({ timeout: STEP_TIMEOUT });

    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });
    // The plan is what carries the guided mode — there is no separate flag.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.life_stages).toEqual({
      childhood_package: TRAVELING,
      native_language: 'German',
    });
    expect(saved.xp_pool ?? 0).toBe(0);

    await $('[data-testid="open-button"]').click();
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    // A load lands in the editor, so the panel is reached through the Abilities tab.
    await $(TAB_BAR).waitForExist({ timeout: STEP_TIMEOUT });
    await $(ABILITIES_TAB).click();
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });

    // Guided funding is derived from the loaded plan, with no reconciliation step.
    await browser.waitUntil(async () => await $(XP_POOL_TOTAL).isExisting(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a loaded plan should restore guided funding',
    });
    expect(await $(XP_POOL_INPUT).isExisting()).toBe(false);
    expect(await textOf(XP_POOL_TOTAL)).toBe('300');
    expect(await $(NATIVE_LANGUAGE).getValue()).toBe('German');
    expect(await textOf(TAKEN)).toContain('Traveling Childhood');

    // The recorded package is history, not a draft: the select starts unselected, so
    // no applied package can display a spurious empty-slot fault.
    expect(await $(PACKAGE_SELECT).getValue()).toBe('');
    expect(await $(PACKAGE_PREVIEW).isExisting()).toBe(false);
    expect(await countOf('[data-testid^="childhood-slot-"][data-testid$="-reason"]')).toBe(0);
  });

  it('offers guided funding to a magus, funded by its apprenticeship', async () => {
    await startWizard('magus');
    await advanceWizardTo('experience');
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });

    // Apprenticeship is modelled now (slice 6b4), so the option is live for a magus …
    expect(await $(FUNDING_LIFE_STAGES).isEnabled()).toBe(true);
    // … and the old refusal is gone: no node, and nothing pointing at one.
    expect(await $('#ability-funding-magus-reason').isExisting()).toBe(false);
    expect(await describedByIds(FUNDING_LIFE_STAGES)).not.toContain('ability-funding-magus-reason');

    await $(FUNDING_LIFE_STAGES).click();
    await $(AGE_INPUT).waitForExist({ timeout: STEP_TIMEOUT });

    // The age a magus is asked for is its age AT the Gauntlet, which the field cannot
    // say for itself — so the note says it, and is announced.
    const note = await $(GAUNTLET_NOTE);
    await note.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await note.getAttribute('role')).toBe('status');
    expect(clean(await note.getText()).length).toBeGreaterThan(0);

    await $(AGE_INPUT).setValue('25');
    // A magus's plan needs its native language too, both because the engine demands it
    // and because an unnamed one shuts the step (see above), and the figures it earns
    // are read on the Abilities step, where the XP bar is. Next rather than a rail jump:
    // this magus is fresh, so the rail has not reached that step yet.
    await $(NATIVE_LANGUAGE).setValue('German');
    await advanceWizardTo('abilities');
    await $(LATER_LIFE).waitForExist({ timeout: STEP_TIMEOUT });

    // Later life stops where apprenticeship begins, so a magus of 25 lived five
    // later-life years, worth 75 — not the twenty a companion of 25 lives.
    await browser.waitUntil(async () => (await textOf(LATER_LIFE)).includes('75'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a magus of 25 should have lived five later-life years, worth 75',
    });
    expect(await textOf(LATER_LIFE)).toContain('5');
    // The 240 points of apprenticeship are the general pool: they alone may buy Arts
    // as well as Abilities (Core Rules.md:2435).
    expect(await textOf(XP_POOL_TOTAL)).toBe('240');
    expect(await textOf(APPRENTICESHIP)).toContain('240');
  });
});
