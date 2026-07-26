// End-to-end: the Longevity Ritual is a STORED player-entered value with a live
// suggestion beside it. The point of the slice, driven through the real binary:
// the hint must move when Creo / the aura / a halving Flaw move, while the entered
// bonus must not budge. Also covers the deleted `aura != 0` gate, the halved
// marker, the focus field + sterility note, and that the source radio no longer
// discards the entered bonus.
//
// NOTE: requires a display + the production binary (see e2e/README.md). The wdio
// `onPrepare` hook builds `target/release/arm-app`.

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

const TYPE_SELECT = '[data-testid="type-select"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const ARTS_TAB = '[data-testid="tab-arts"]';
const POSSESSIONS_TAB = '[data-testid="tab-possessions"]';
const TOTALS_TAB = '[data-testid="tab-totals"]';

const HINT = '[data-testid="longevity-hint"]';
const HALVED = '[data-testid="longevity-hint-halved"]';
const BONUS = '[data-testid="longevity-bonus"]';
const FOCUS = '[data-testid="longevity-focus"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function hintText() {
  return clean(await $(HINT).getText());
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
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(TYPE_SELECT).selectByAttribute('value', 'magus');

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

    // Magic Items tab: aura 5, then add a self-made ritual.
    await $(POSSESSIONS_TAB).click();
    const auraInput = await $('[data-testid="aura-input"]');
    await auraInput.waitForExist({ timeout: 10000 });
    await auraInput.setValue('5');
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
    await $(POSSESSIONS_TAB).click();
    await $(BONUS).setValue('9');
    await browser.waitUntil(
      async () => !(await $('[data-testid="longevity-not-entered"]').isExisting()),
      {
        timeout: 5000,
        timeoutMsg: 'the not-entered flag should clear once a bonus is typed',
      },
    );

    // Raise Creo by 1: THE HINT MOVES, THE ENTERED BONUS DOES NOT. This is the
    // whole reason the slice exists.
    await $(ARTS_TAB).click();
    await $('[data-testid="art-inc-art.creo"]').click();
    await $(POSSESSIONS_TAB).click();
    // Creo 6 + Corpus 5 + Aura 5 = 16 → ceil(16/5) = +4.
    await waitForHint('16', '+4');
    expect(await $(BONUS).getValue()).toBe('9');

    // Aura 0 still suggests a bonus — the removed `aura != 0` gate. The Aura
    // Modifier is a plain addend: 6 + 5 = 11 → ceil(11/5) = +3.
    await $('[data-testid="aura-input"]').setValue('0');
    await waitForHint('11', '+3');
    expect(await $(BONUS).getValue()).toBe('9');

    // Difficult Longevity Ritual halves the Lab Total: 11 → 5 → ceil(5/5) = +1,
    // and the hint is marked as halved.
    await $(VF_TAB).click();
    const addFlaw = await $('[data-testid="add-flaw.difficult_longevity_ritual"]');
    await addFlaw.waitForExist({ timeout: 10000 });
    await addFlaw.click();
    await $(POSSESSIONS_TAB).click();
    await waitForHint('5', '+1');
    await $(HALVED).waitForExist({ timeout: 5000 });
    expect(await $(BONUS).getValue()).toBe('9');

    // The focus, plus the sterility consequence the rules attach to it.
    await $(FOCUS).setValue('A draught of quicksilver at midwinter');
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
  });
});
