// End-to-end: the aging CRISIS (slice 6b7c), driven on a GROG — the same rail
// `aging.e2e.js` uses, and for the same reason: the grog rail is the shortest the
// ruleset declares, so nothing between the start screen and the step under test can
// colour the result. What this spec adds is the second die, which until 6b7c the
// shipped app had nowhere to put: every Crisis it applied went in owed and unrolled,
// whatever the player had thrown.
//
// The arithmetic is the rulebook's, and both halves of it are hand-checkable:
//
//   "AGING TOTAL: Stress die (no botch) + age/10 (round up) / - Living Conditions
//    modifier / - Longevity Ritual modifier" (`:16567-16569`).
//   "13 | Gain sufficient Aging Points (in any Characteristics) to reach the next
//    level in Decrepitude, and Crisis" (`:16602`).
//   "**Crisis:** Increase the character's Decrepitude first, and then roll on the
//    Crisis Table." (`:16619`)
//   "CRISIS TOTAL: Simple die + age/10 (round up) + Decrepitude Score" (`:16621`).
//   "15 | **Minor illness**. Stamina stress roll against an Ease Factor of 3 or
//    CrCo20 to survive." (`:16628`)
//
// For this grog of 40, rolling for age 36 with no Living Conditions and a ritual
// worth 0:
//
//   age modifier   ceil(36 / 10)      = +4
//   stress die 9   9 + 4              = 13   → next Decrepitude level, and a Crisis
//   five points placed                       → Decrepitude Score 1 (five xp buys 1)
//   simple die 10  10 + 4 + 1         = 15   → Minor illness, Ease Factor 3, CrCo20
//
// THE ORDER IS THE RULE. The five points are `:16619`'s increase, so they are a term
// of the crisis total — which is why the panel withholds a reading until they are
// placed, and why this spec types the crisis die BEFORE placing them and asserts
// there is no total yet.
//
// (Ars Magica - Definitive Edition (Core Rules).md.)
//
// NOTE: requires the production binary; the display comes from your desktop session
// or, when DISPLAY is unset, the Xvfb one WebdriverIO starts (see e2e/README.md).

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { advanceWizardTo, currentWizardPhase, startWizard } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const AGE_INPUT = '[data-testid="age-input"]';
const SCHEDULE = '[data-testid="aging-schedule"]';
const DIE_INPUT = '[data-testid="aging-die-input"]';
const AGING_TOTAL = '[data-testid="aging-total"]';
const OUTCOME_CRISIS = '[data-testid="aging-outcome-crisis"]';
const DISTRIBUTE_STA = '[data-testid="aging-distribute-sta"]';
const APPLY = '[data-testid="aging-apply"]';
const REVERT_36 = '[data-testid="aging-revert-36"]';
const LONGEVITY_ADD = '[data-testid="longevity-add"]';
const LONGEVITY_BONUS = '[data-testid="longevity-bonus"]';

const CRISIS = '[data-testid="aging-crisis"]';
const CRISIS_DIE = '[data-testid="crisis-die-input"]';
const CRISIS_UNROLLED = '[data-testid="crisis-die-unrolled"]';
const CRISIS_TOTAL = '[data-testid="crisis-total"]';
const CRISIS_PARTS = '[data-testid="crisis-total-parts"]';
const CRISIS_ROW = '[data-testid="crisis-row"]';
const CRISIS_SURVIVAL = '[data-testid="crisis-survival"]';
const CRISIS_EASE = '[data-testid="crisis-survival-ease-factor"]';
const CRISIS_RITUAL = '[data-testid="crisis-survival-ritual"]';
const CRISIS_ALLOWANCE = '[data-testid="crisis-allowance-0"]';
const CRISIS_MODIFIER_0 = '[data-testid="crisis-modifier-0"]';
const AGING_NOTE_0 = '[data-testid="aging-note-0"]';
const LOG_CRISIS_0 = '[data-testid="aging-log-crisis-0"]';
const LOG_EMPTY = '[data-testid="aging-log-empty"]';

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

describe('the aging crisis', () => {
  it('sends a grog of 40 to the Crisis Table and asks for the second die', async () => {
    await startWizard('grog');
    await advanceWizardTo('aging');
    expect(await currentWizardPhase()).toBe('aging');
    await $(SCHEDULE).waitForExist({ timeout: BOOT_TIMEOUT });
    await $(AGE_INPUT).setValue('40');

    // A ritual worth 0 leaves the AGING TOTAL alone and is still a ritual the Crisis
    // spends (`:16573`). The ritual is part of `AgingPanel`, so this guided step and
    // the editor's Aging tab both reach it; the step is where the Crisis is resolved,
    // so this is where the note can be provoked.
    await $(LONGEVITY_ADD).waitForExist({ timeout: STEP_TIMEOUT });
    await $(LONGEVITY_ADD).click();
    // Clickable, not merely existing: the field appears the moment the ritual is
    // added above, so the aging grid is still relaying out around it and for a frame
    // the input is in the DOM but not yet interactable. `waitForExist` alone let
    // `setValue` race that frame — the whole spec failed on its first attempt and
    // passed on the retry, which is the shape a real defect hides in.
    await $(LONGEVITY_BONUS).waitForClickable({ timeout: STEP_TIMEOUT });
    await $(LONGEVITY_BONUS).setValue('0');

    // 9 + 4 = 13, which is the first Crisis row (`:16602`).
    await $(DIE_INPUT).setValue('9');
    await browser.waitUntil(async () => (await textOf(AGING_TOTAL)).includes('13'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'a stress die of 9 at age 36 should total 13',
    });
    expect(await textOf(OUTCOME_CRISIS)).not.toHaveLength(0);

    // The panel appears with the row that demanded it, never before.
    await $(CRISIS).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await $(CRISIS_DIE).isExisting()).toBe(true);
    // "a zero counts as ten" (`:474`) — offered as the ruleset's own bounds.
    expect(await $(CRISIS_DIE).getAttribute('min')).toBe('1');
    expect(await $(CRISIS_DIE).getAttribute('max')).toBe('10');
  });

  it('withholds the reading until the Aging Points are placed', async () => {
    // THE LOAD-BEARING ASSERTION of the whole slice. "Increase the character's
    // Decrepitude first, and then roll on the Crisis Table" (`:16619`) — those five
    // points ARE the increase, and they are a term of the crisis total. Read off the
    // character standing here, the same die answers 14; the app would then show 14
    // and write 15. So with the die typed and nothing placed, there is no reading.
    await $(CRISIS_DIE).setValue('10');
    await browser.waitUntil(async () => await $(CRISIS_UNROLLED).isExisting(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'an unplaced distribution should leave the Crisis unread',
    });
    expect(await $(CRISIS_TOTAL).isExisting()).toBe(false);

    // Placing them is what makes the reading honest — and it is 15, not 14.
    await $(DISTRIBUTE_STA).setValue('5');
    await browser.waitUntil(
      async () =>
        (await $(CRISIS_TOTAL).isExisting()) && (await textOf(CRISIS_TOTAL)).includes('15'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'a simple die of 10 with the year’s five points placed should total 15',
      },
    );

    // Every term, as the engine reported it: the die, the age, and the Decrepitude
    // this very year raised.
    const parts = await textOf(CRISIS_PARTS);
    expect(parts).toContain('+10');
    expect(parts).toContain('+4');
    expect(parts).toContain('+1');
    // ASCII hyphen-minus only, never the mathematical minus.
    expect(parts).not.toContain('−');
  });

  it('names the row and says what surviving it would take', async () => {
    // "15 | **Minor illness**." (`:16628`) — the row's text comes from
    // `rules/i18n/<lang>/aging.json` keyed by its id, and the severity through
    // Fluent. Neither may reach the screen as a slug.
    const row = await textOf(CRISIS_ROW);
    expect(row).toContain('Minor illness');
    expect(row).not.toContain('crisis.');

    // "Stamina stress roll against an Ease Factor of 3 or CrCo20 to survive."
    await $(CRISIS_SURVIVAL).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(CRISIS_EASE)).toContain('3');
    expect(await textOf(CRISIS_RITUAL)).toContain('20');

    // "An Int + Medicine roll against an Ease Factor of 6 … if the doctor botches the
    // character must subtract 3" (`:16634`) — stated, never scored, because the
    // Medicine belongs to a character this sheet does not hold.
    const allowance = await textOf(CRISIS_ALLOWANCE);
    expect(allowance).toContain('Medicine');
    expect(allowance).toContain('6');
    expect(allowance).toContain('-3');
    expect(allowance).not.toContain('ability.medicine');
    expect(allowance).not.toContain('−');

    // This grog carries no familiar and no aging Virtue, so he brings NOTHING to the
    // roll — and the panel shows no modifier line at all rather than a "+0".
    expect(await $(CRISIS_MODIFIER_0).isExisting()).toBe(false);
  });

  it('writes the Crisis onto the character, and says what it spent', async () => {
    await $(APPLY).click();

    // The five points land where the player placed them, and the log gains the year.
    await browser.waitUntil(
      async () => (await $('[data-testid="aging-points-sta"]').getValue()) === '5',
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'applying a total of 13 should award five Aging Points',
      },
    );
    expect(await $(LOG_EMPTY).isExisting()).toBe(false);

    // The log reads the Crisis back — the row named, and both figures beside it.
    const logged = await textOf(LOG_CRISIS_0);
    expect(logged).toContain('Minor illness');
    expect(logged).toContain('15');
    expect(logged).toContain('10');
    expect(logged).not.toContain('crisis.minor_illness');

    // "its power is spent, and the focal ritual must be performed again" (`:16573`) —
    // reported, because the engine leaves the ritual entry exactly where it found it,
    // so this note is the only place the player can hear about it.
    await $(AGING_NOTE_0).waitForExist({ timeout: STEP_TIMEOUT });
    expect(await textOf(AGING_NOTE_0)).not.toHaveLength(0);
    expect(await $(LONGEVITY_BONUS).isExisting()).toBe(true);
  });

  it('saves the resolved Crisis and takes the whole year back off', async () => {
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'save did not write the file',
    });

    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.aging_log).toEqual([
      {
        age: 36,
        effect: '',
        die: 9,
        total: 13,
        points: { sta: 5 },
        apparent_age_increased: true,
        crisis: true,
        crisis_die: 10,
        crisis_total: 15,
        crisis_row: 'crisis.minor_illness',
        crisis_severity: 'minor',
      },
    ]);
    // NO DRAFT STATE: both dice are the player's, and only the log entry's own
    // recorded fields may carry them.
    const withoutLog = { ...saved };
    delete withoutLog.aging_log;
    expect(JSON.stringify(withoutLog)).not.toContain('crisis');

    // A mistyped die has to be recoverable, and a Crisis is undone with the year that
    // caused it — never by editing a figure out from under the entry.
    await $(REVERT_36).click();
    await browser.waitUntil(
      async () => (await $('[data-testid="aging-points-sta"]').getValue()) === '0',
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'reverting age 36 should take its five Aging Points back',
      },
    );
    expect(await $(LOG_EMPTY).isExisting()).toBe(true);
    expect(await $(LOG_CRISIS_0).isExisting()).toBe(false);
  });
});
