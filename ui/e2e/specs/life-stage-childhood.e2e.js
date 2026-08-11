// End-to-end: funding Abilities from the character's life stages, and taking a
// Sample Childhood package (slice 6b3). Driven on a COMPANION — the type the
// rulebook's 15-experience-per-year later life applies to unchanged, and the one
// whose short phase list puts the Abilities step two clicks away.
//
// The whole flow runs on the wizard's `abilities` step, which mounts the very
// AbilityTab the editor's Abilities tab does, so one narrative covers both
// surfaces: switch the funding source, type an age and a native language, take
// the Traveling Childhood, and watch the two childhood pools fill. It closes on a
// save/load round-trip (guided funding must survive a reload) and on a magus,
// where the guided mode is refused with a reason.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`, so this cannot run without that build step.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { advanceWizardTo, startWizard } from '../helpers.js';

const PANEL = '[data-testid="life-stage-panel"]';
const FUNDING_POOL = '[data-testid="ability-funding-pool"]';
const FUNDING_LIFE_STAGES = '[data-testid="ability-funding-life_stages"]';
const AGE_INPUT = '[data-testid="life-stage-age-input"]';
const NATIVE_LANGUAGE = '[data-testid="native-language-input"]';
const XP_POOL_INPUT = '[data-testid="xp-pool"]';
const XP_POOL_TOTAL = '[data-testid="xp-pool-total"]';
const LATER_LIFE = '[data-testid="life-stage-later-life"]';
const RESTRICTED_0 = '[data-testid="restricted-xp-0"]';
const RESTRICTED_1 = '[data-testid="restricted-xp-1"]';
const PACKAGE_SELECT = '[data-testid="childhood-package-select"]';
const PACKAGE_PREVIEW = '[data-testid="childhood-package-preview"]';
const APPLY = '[data-testid="childhood-apply"]';
const TAKEN = '[data-testid="childhood-taken"]';
const NEXT = '[data-testid="wizard-next"]';
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
const TAB_BAR = '[role="tablist"]';
// The docked step panel — scoped, because the childhood picker renders the
// engine's package rejections with `data-code` too.
const DOCKED_ISSUES = '[data-testid="issue-list"]';

// Traveling Childhood is the only shipped package with three slots
// (`rules/core/childhoods.json`): two Area Lore regions and one language.
const TRAVELING = 'childhood.traveling';
const AREA_LORE_SCORES = '[data-testid^="ability-score-ability.area_lore-"]';

const STEP_TIMEOUT = 10000;
const BOOT_TIMEOUT = 30000;

const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

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

/** Every id in an element's `aria-describedby`, in order. */
async function describedByIds(selector) {
  const attribute = await $(selector).getAttribute('aria-describedby');
  return (attribute ?? '').split(/\s+/).filter((id) => id.length > 0);
}

describe('life-stage funding and Sample Childhoods', () => {
  it('offers a typed experience pool by default, with no life-stage rows', async () => {
    await startWizard('companion');
    await advanceWizardTo('abilities');

    // The panel rides on the step, mounted from AbilityTab — the same component the
    // editor's Abilities tab mounts.
    await $(PANEL).waitForExist({ timeout: BOOT_TIMEOUT });
    // Flat funding is the default and must stay byte-identical: an editable total,
    // and not a single life-stage row.
    expect(await $(XP_POOL_INPUT).getTagName()).toBe('input');
    expect(await $(XP_POOL_TOTAL).isExisting()).toBe(false);
    expect(await $(LATER_LIFE).isExisting()).toBe(false);
    // A companion may be built either way, so the guided option is live.
    expect(await $(FUNDING_LIFE_STAGES).isEnabled()).toBe(true);
  });

  it('counts later life from the age and shows the two childhood blocks', async () => {
    await $(FUNDING_LIFE_STAGES).click();
    // Under a plan the engine forbids a typed pool, so the input gives way to a
    // read-only total.
    await browser.waitUntil(async () => !(await $(XP_POOL_INPUT).isExisting()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'guided funding must retire the editable experience pool',
    });

    await $(AGE_INPUT).setValue('25');
    // Later life is (age - childhood years) x 15 = (25 - 5) x 15 = 300.
    await browser.waitUntil(async () => (await textOf(XP_POOL_TOTAL)) === '300', {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'age 25 should earn a companion 300 later-life experience',
    });
    const laterLife = await textOf(LATER_LIFE);
    expect(laterLife).toContain('20');
    expect(laterLife).toContain('15');
    expect(laterLife).toContain('300');
    expect(await textOf('[data-testid="life-stage-age-cap"]')).toContain('5');

    // A fresh companion has no restricted-pool Virtue (no Educated, Warrior or
    // Privileged Upbringing), so the only restricted pools on screen are the
    // childhood blocks and their indices are deterministic.
    //
    // The 75-point block is restricted to ONE Ability instance — the native
    // language — so it cannot exist before that language is named; until then the
    // 45-point spread is the whole of childhood on screen. (The plan's spec expected
    // both blocks here; the engine forms the native block from
    // `native_language_instance`, which needs the language. Both are asserted below,
    // as soon as it exists.)
    const spread = await textOf(RESTRICTED_0);
    expect(spread).toContain('0 / 45');
    expect(spread).toContain('Early childhood');
    // Localized, never the raw block slug.
    expect(spread).not.toContain('childhood_spread');
    expect(await $(RESTRICTED_1).isExisting()).toBe(false);
  });

  it('gates the step on the unset native language and lifts it once named', async () => {
    // The docked panel is about this step, and a life-stage error blocks Next just
    // like any other phase finding.
    await browser.waitUntil(async () => await issueExists('life_stage_native_language_unset'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a plan without a native language must report life_stage_native_language_unset',
    });
    await browser.waitUntil(async () => !(await $(NEXT).isEnabled()), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the unset native language did not block the Abilities step',
    });
    // The finding reads as a sentence, never as its code.
    const unset = await textOf(`${DOCKED_ISSUES} [data-code="life_stage_native_language_unset"]`);
    expect(unset.length).toBeGreaterThan(0);
    expect(unset).not.toContain('life_stage_native_language_unset');

    await $(NATIVE_LANGUAGE).setValue('German');

    // The error goes; what remains is the warning that those 75 points are still
    // unspent — which a Childhood package is about to spend.
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

    // With the language named, the 75-point block joins the 45-point spread, in that
    // order, both localized.
    const native = await textOf(RESTRICTED_0);
    expect(native).toContain('0 / 75');
    expect(native).toContain('Native language');
    expect(native).not.toContain('childhood_native_language');
    expect(await textOf(RESTRICTED_1)).toContain('0 / 45');
  });

  it('previews a drafted childhood through the plan and refuses an unanswered slot', async () => {
    await $(PACKAGE_SELECT).selectByAttribute('value', TRAVELING);
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
    await $('[data-testid="childhood-slot-area_a"]').setValue('Rhine');
    await $('[data-testid="childhood-slot-area_b"]').setValue('Provence');
    // The spread buys a Living Language "other than the character's native
    // language", so German is not an answer here.
    await $('[data-testid="childhood-slot-language"]').setValue('German');

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

    await $(languageSlot).setValue('Italian');
    await browser.waitUntil(async () => await $(APPLY).isEnabled(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a distinct childhood language should let the package be taken',
    });
    expect(await $(languageSlot).getAttribute('aria-invalid')).not.toBe('true');
  });

  it('takes the package, filling both childhood blocks exactly', async () => {
    await $(APPLY).click();

    // German 5 costs the whole 75-point native block; the spread's 45 buy the two
    // Area Lores, Folk Ken, Survival and Italian.
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
  });

  it('is idempotent: taking the same package twice adds no second set of rows', async () => {
    const before = await countOf(AREA_LORE_SCORES);
    await $(APPLY).click();
    // A package raises scores monotonically, so re-taking it must not duplicate a
    // row for the same Ability and answer.
    await browser.waitUntil(async () => (await countOf(AREA_LORE_SCORES)) === before, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 're-taking the package changed the Area Lore row count',
    });
    expect(await scoreOf('ability.area_lore', 'Rhine')).toBe('1');
    expect(await textOf(RESTRICTED_0)).toContain('75 / 75');
  });

  it('keeps the bought rows when the funding source is switched back and forth', async () => {
    await $(FUNDING_POOL).click();

    // The plan is gone, so the editable total is back — at 0, which the input
    // renders as its empty placeholder.
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

    await $(FUNDING_LIFE_STAGES).click();
    await $(XP_POOL_TOTAL).waitForExist({ timeout: STEP_TIMEOUT });
    // A new plan starts empty rather than resurrecting the dropped one, so the
    // native language has to be named again — and no drafted slot shows a fault.
    expect(await $(NATIVE_LANGUAGE).getValue()).toBe('');
    expect(await countOf('[data-testid^="childhood-slot-"][data-testid$="-reason"]')).toBe(0);
    // The draft went with the plan it belonged to: the select is back on its prompt
    // option, so nothing prefills a package for a plan that starts from nothing.
    expect(await $(PACKAGE_SELECT).getValue()).toBe('');
    expect(await $(PACKAGE_PREVIEW).isExisting()).toBe(false);
    // The rows still stand through the second switch.
    expect(await countOf(AREA_LORE_SCORES)).toBe(2);

    // Put the plan back the way the save below should record it.
    await $(AGE_INPUT).setValue('25');
    await $(NATIVE_LANGUAGE).setValue('German');
    await browser.waitUntil(async () => (await textOf(RESTRICTED_0)).includes('75 / 75'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'restoring the native language did not re-fund the childhood block',
    });
  });

  it('restores guided funding and the recorded package from a save', async () => {
    // Leaving the guided mode dropped the plan, and both the recorded package (the
    // plan IS the record) and the draft went with it, so re-take it from the select
    // and the slots.
    await $(PACKAGE_SELECT).selectByAttribute('value', TRAVELING);
    await $('[data-testid="childhood-slot-area_a"]').setValue('Rhine');
    await $('[data-testid="childhood-slot-area_b"]').setValue('Provence');
    await $('[data-testid="childhood-slot-language"]').setValue('Italian');
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

  it('refuses guided funding for a magus, with the reason reachable', async () => {
    await startWizard('magus');
    await advanceWizardTo('abilities');
    await $(PANEL).waitForExist({ timeout: STEP_TIMEOUT });

    // A magus earns experience in four periods and apprenticeship is not modelled
    // yet, so the engine refuses the combination — and the UI must not offer it.
    expect(await $(FUNDING_LIFE_STAGES).isEnabled()).toBe(false);
    const ids = await describedByIds(FUNDING_LIFE_STAGES);
    expect(ids).toContain('ability-funding-magus-reason');
    const reason = clean(await $('#ability-funding-magus-reason').getText());
    expect(reason.length).toBeGreaterThan(0);
    expect(reason).not.toContain('ability-funding-magus-reason');

    // So a magus keeps the editable pool.
    expect(await $(XP_POOL_INPUT).getTagName()).toBe('input');
    expect(await $(XP_POOL_TOTAL).isExisting()).toBe(false);
  });
});
