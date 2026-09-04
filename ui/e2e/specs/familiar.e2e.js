// End-to-end: the familiar is a CREATURE STATBLOCK, not a name plus three cords.
// Driven through the real binary: animal, a negative Size, Magic Might, a
// Characteristic, a Personality Trait, the three cords and one bond-invested power
// are all enterable; the totals panel's bonding level folds the negative Size in;
// the cord points follow the 5/15/30/50/75 curve; NO validation issue appears (the
// guidance-only contract, end to end); and NO power-levels budget bar is rendered
// (Core:10866 — there is no limit). Then the whole statblock round-trips through
// save + Open.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { clean, startCharacter } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const POSSESSIONS_TAB = '[data-testid="tab-possessions"]';
const TOTALS_TAB = '[data-testid="tab-totals"]';

const BINDING = '[data-testid="derived-familiar-binding"]';
const CORDS = '[data-testid="derived-familiar-cords"]';
const INVESTED = '[data-testid="derived-familiar-invested"]';

/** Wait for a (debounced) engine read-out to contain every fragment. */
async function waitForText(selector, ...fragments) {
  await browser.waitUntil(
    async () => {
      const text = clean(await $(selector).getText());
      return fragments.every((f) => text.includes(f));
    },
    {
      timeout: 10000,
      timeoutMsg: async () =>
        `${selector} never showed ${fragments.join(' + ')}; read ` +
        `"${clean(await $(selector).getText())}"`,
    },
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

describe('familiar', () => {
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
    // A raven is Size -4 (Core:17829-17856). The rendered value must carry the
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

    // Human intelligence at Int -3, gained from the bond (Core:10854).
    await set('familiar-char-int', '-3');
    expect(await $('[data-testid="familiar-char-int"]').getValue()).toBe('-3');

    // The bond's Loyal (partner) +3, entered by hand — never auto-applied.
    await $('[data-testid="familiar-personality-add"]').click();
    await set('familiar-personality-name-0', 'Loyal (Marcus)');
    const inc = await $('[data-testid="familiar-personality-inc-0"]');
    for (let i = 0; i < 3; i++) await inc.click();
    expect(clean(await $('[data-testid="familiar-personality-value-0"]').getText())).toBe('+3');

    // Cords 3 / 2 / 1 → 30 + 15 + 5 = 50 points off the curve (Core:10836).
    await set('familiar-cord-gold', '3');
    await set('familiar-cord-silver', '2');
    await set('familiar-cord-bronze', '1');

    // One power invested in the bond, at level 20.
    await $('[data-testid="familiar-power-add"]').click();
    await set('familiar-power-name-0', 'Mental communication');
    await set('familiar-power-level-0', '20');

    // NO budget bar for the invested powers: the character's own powers get a
    // `power-levels-used` read-out, but Core:10866 sets no limit on what may be
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
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });

    // Every statblock field reaches disk, at the engine's current schema — the
    // statblock fields are additive, so 5.5c bumped nothing of its own.
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(16);
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
    await $('[data-testid="open-button"]').click();
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
