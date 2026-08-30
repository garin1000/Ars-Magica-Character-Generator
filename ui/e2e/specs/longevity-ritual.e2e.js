// End-to-end: the Longevity Ritual is a STORED player-entered value with a live
// suggestion beside it. The point of the slice, driven through the real binary:
// the hint must move when Creo / the aura / a halving Flaw move, while the entered
// bonus must not budge. Also covers the deleted `aura != 0` gate, the halved
// marker, the focus field + sterility note, and that the source radio no longer
// discards the entered bonus.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { startCharacter } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

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

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

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
 * to first. Mirrors `reach()` in life-stage-childhood.e2e.js.
 */
async function reach(selector) {
  const element = await $(selector);
  await element.waitForExist({ timeout: 10000 });
  // `block: 'center'`, not the default. A bare `scrollIntoView()` aligns to the
  // NEAREST edge, which in a short scrollport can leave the element's centre still
  // outside the visible box — and the centre is the point WebKitWebDriver hit-tests,
  // so the interaction is refused even though the element is on screen. The aging
  // surfaces nest three scrollports (see #34 in the plan), so the innermost is short
  // enough for that to happen here.
  await element.scrollIntoView({ block: 'center' });
  // And clickable, not merely existing and scrolled: this helper's whole promise is
  // that the caller can interact with what it returns. `longevity-bonus` failed this
  // way in three specs across three slices — `aging-crisis`, `aging`, and this one —
  // so the wait belongs in the shared helper rather than at each call site.
  await element.waitForClickable({ timeout: 10000 });
  return element;
}

/** Wait for the engine's (debounced) read-out to contain every fragment. */
async function waitForHint(...fragments) {
  await browser.waitUntil(
    async () => {
      const text = await hintText();
      return fragments.every((f) => text.includes(f));
    },
    {
      timeout: 10000,
      timeoutMsg: async () =>
        `hint never showed ${fragments.join(' + ')}; read "${await hintText()}"`,
    },
  );
}

describe('longevity ritual', () => {
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

    // The focus, plus the sterility consequence the rules attach to it.
    await (await reach(FOCUS)).setValue('A draught of quicksilver at midwinter');
    await $('[data-testid="longevity-sterility-note"]').waitForExist({ timeout: 5000 });

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
    await $('[data-testid="save-button"]').click();
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
    await $('[data-testid="longevity-remove"]').click();
    await $('[data-testid="longevity-add"]').waitForExist({ timeout: 10000 });
    expect(await $(BONUS).isExisting()).toBe(false);
    expect(await $(FOCUS).isExisting()).toBe(false);
  });
});
