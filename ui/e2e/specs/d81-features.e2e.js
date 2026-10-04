// End-to-end: independent verification of D81's spell/requisite/house features
// through the real release binary — written without reading implementation
// internals beyond what is needed to pick concrete, deterministic fixtures
// (Art scores, catalogue spell ids, validation codes) from the engine's own
// public doc comments and the shipped `rules/core/*.json` data.
//
// Five describes, one per acceptance criterion:
//   1. a requisite spell's Casting Total reflects the requisite fold
//      (ArMDE:12309-12313);
//   2. "add within focus" adds a spell already marked, and removing the
//      Magical Focus produces the over-cap error + the stale-mark warning
//      (D81.5/D81.17), and unticking the stale mark clears the warning (W4);
//   3. Incompatible Arts' four selects in pairs, a barred grid cell reading
//      "Unusable", and a spell using a barred combination raising an error
//      (D81.8/D81.15);
//   4. a Characteristic-floor prerequisite (Supernatural Beauty, Presence >= 1)
//      reports its issue when unmet;
//   5. Merinita's conditional Warping Point appears and disappears with a
//      faerie-related Virtue/Flaw (D81.14).
//
// Each describe builds its own character via `startCharacter` (per the shared
// helper's own rationale: a character's type is fixed at creation, and creating
// one discards whatever came before), so none depends on another's state.

import { $, $$, browser, expect } from '@wdio/globals';

import { clean, isRowBlocked, startCharacter, STEP_TIMEOUT } from '../helpers.js';

const ARTS_TAB = '[data-testid="tab-arts"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const SPELLS_TAB = '[data-testid="tab-spells"]';
const HOUSE_TAB = '[data-testid="tab-house_specialisation"]';
const TOTALS_TAB = '[data-testid="tab-totals"]';
const ISSUE_LIST = '[data-testid="issue-list"]';

/** Raise one Art's score by `times` clicks (score is set regardless of XP). */
async function raiseArt(artId, times) {
  const inc = await $(`[data-testid="art-inc-${artId}"]`);
  await inc.waitForExist({ timeout: STEP_TIMEOUT });
  for (let i = 0; i < times; i++) await inc.click();
}

async function issueCodes() {
  const items = await $$(`${ISSUE_LIST} li`);
  const codes = [];
  for (let i = 0; i < items.length; i++) {
    codes.push(await items[i].getAttribute('data-code'));
  }
  return codes;
}

// End-to-end: ArMDE:12309's base min rule — "you must use the lesser of your
// score in the requisite and your score in the spell's main Technique or
// Form". `spell.eyes_of_the_cat` (Mu/Co, requisite art.animal — a Form-class
// requisite) is the vehicle: Corpus 5 vs the lower Animal 2 must fold the Form
// side down to 2, so the Casting Total reads 5+2=7, never the naive 5+5=10.
describe('requisite fold in the Casting Total (D81, ArMDE:12309-12313)', () => {
  it("folds a Form-class requisite's lower score into the total", async () => {
    await startCharacter('magus');

    await $(ARTS_TAB).waitForExist({ timeout: 30000 });
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('40');
    await raiseArt('art.muto', 5);
    await raiseArt('art.corpus', 5);
    await raiseArt('art.animal', 2);

    // Per-spell cap = Muto(5) + folded-Form(min(Corpus 5, Animal 2)=2) + Int(0)
    // + Magic Theory(0) + 3 = 10, so level 5 is addable without raising the cap
    // any further.
    await $(SPELLS_TAB).click();
    const add = await $('[data-testid="add-spell.eyes_of_the_cat"]');
    await add.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => !(await isRowBlocked(add)), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'eyes_of_the_cat should be addable once Muto/Corpus/Animal are raised',
    });
    await add.click();

    const total = await $('[data-testid^="spell-casting-total-spell.eyes_of_the_cat-"]');
    await total.waitForExist({ timeout: STEP_TIMEOUT });
    await browser.waitUntil(async () => clean(await total.getText()) === 'Casting Total: 7', {
      timeout: STEP_TIMEOUT,
      timeoutMsg:
        'the requisite fold should read Casting Total 7 (folded Form 2 + Muto 5), not the naive 10',
    });
  });
});

// End-to-end: D81.5's "add within focus" action — a spell whose level exceeds
// the plain per-spell cap but fits the Magical-Focus-doubled cap is added
// already marked `within_focus`. D81.17's stale-marker warning: once the
// Magical Focus is removed, the marker is inert, so the (now un-doubled) cap
// is exceeded (a creation-time ERROR) and the marker itself is flagged stale
// (a non-blocking warning) — both reported from the SAME predicate
// (`has_magical_focus`) that already neutralizes the marker's effect.
describe('add within focus, and the stale-mark warning on removal (D81.5/D81.17)', () => {
  it('adds a spell that fits only the focus-doubled cap, already marked', async () => {
    await startCharacter('magus');

    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(VF_TAB).click();
    const addFocus = await $('[data-testid="add-virtue.minor_magical_focus"]');
    await addFocus.waitForExist({ timeout: 10000 });
    await addFocus.click();
    const focusParam = await $('[data-testid^="param-virtue.minor_magical_focus-focus-"]');
    await focusParam.waitForExist({ timeout: STEP_TIMEOUT });
    await focusParam.setValue('Fire');

    // Creo 3 / Ignem 3: plain cap = 3+3+3=9, within-focus cap = 9+min(3,3)=12.
    // spell.lamp_without_flame (Cr/Ig, level 10, touch) fits only the latter.
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('20');
    await raiseArt('art.creo', 3);
    await raiseArt('art.ignem', 3);

    await $(SPELLS_TAB).click();
    const withinFocusAdd = await $('[data-testid="add-within-focus-spell.lamp_without_flame"]');
    await withinFocusAdd.waitForExist({ timeout: STEP_TIMEOUT });
    await withinFocusAdd.click();

    // Added already marked within_focus — the checkbox renders checked, and no
    // level-cap error is raised (10 <= the focus-doubled cap of 12).
    const marker = await $('[data-testid^="spell-within-focus-spell.lamp_without_flame-"]');
    await marker.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await marker.isSelected()).toBe(true);
    expect((await issueCodes()).includes('spell_level_exceeds_cap')).toBe(false);
  });

  it('removing the Magical Focus raises the over-cap error and the stale-mark warning', async () => {
    await startCharacter('magus');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(VF_TAB).click();
    const addFocus = await $('[data-testid="add-virtue.minor_magical_focus"]');
    await addFocus.waitForExist({ timeout: 10000 });
    await addFocus.click();
    const focusParam = await $('[data-testid^="param-virtue.minor_magical_focus-focus-"]');
    await focusParam.waitForExist({ timeout: STEP_TIMEOUT });
    await focusParam.setValue('Fire');

    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('20');
    await raiseArt('art.creo', 3);
    await raiseArt('art.ignem', 3);

    await $(SPELLS_TAB).click();
    const withinFocusAdd = await $('[data-testid="add-within-focus-spell.lamp_without_flame"]');
    await withinFocusAdd.waitForExist({ timeout: STEP_TIMEOUT });
    await withinFocusAdd.click();
    await $('[data-testid^="spell-within-focus-spell.lamp_without_flame-"]').waitForExist({
      timeout: STEP_TIMEOUT,
    });

    // Remove the Magical Focus: the marker is now stale (the entity holds no
    // focus at all) and the plain cap (9) is below the learned level (10).
    await $(VF_TAB).click();
    const remove = await $('[data-testid^="remove-virtue.minor_magical_focus-"]');
    await remove.waitForExist({ timeout: STEP_TIMEOUT });
    await remove.click();

    await browser.waitUntil(
      async () => {
        const codes = await issueCodes();
        return (
          codes.includes('spell_level_exceeds_cap') &&
          codes.includes('spell_within_focus_without_magical_focus')
        );
      },
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg:
          'removing the Magical Focus should raise both spell_level_exceeds_cap and ' +
          'spell_within_focus_without_magical_focus for the stale-marked spell',
      },
    );

    // W4 (try-out finding 17): the stale mark stays clearable — the checkbox
    // survives the Virtue's removal while the spell is marked. Unticking it
    // clears the stale-mark warning; the over-cap error stays, because level 10
    // still exceeds the plain cap of 9 without a focus.
    await $(SPELLS_TAB).click();
    const staleMarker = await $('[data-testid^="spell-within-focus-spell.lamp_without_flame-"]');
    await staleMarker.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await staleMarker.isSelected()).toBe(true);
    await staleMarker.click();

    await browser.waitUntil(
      async () => !(await issueCodes()).includes('spell_within_focus_without_magical_focus'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'unticking the stale within-focus mark should clear its warning',
      },
    );
    expect((await issueCodes()).includes('spell_level_exceeds_cap')).toBe(true);
  });
});

// End-to-end: Incompatible Arts (ArMDE:6290-6292) records two Technique+Form
// combinations across four selects (technique_1/form_1/technique_2/form_2,
// grouped in pairs); a barred combination's grid cell reads "Unusable" instead
// of leaking a number, and any known spell touching a barred combination is a
// creation-time ERROR (D81.15) "even if one or both are requisites". This uses
// the rulebook's own worked example pair, Intellego+Herbam / Intellego+Animal.
describe('Incompatible Arts: barred pairs, the Unusable cell, and the spell error (D81.8/D81.15)', () => {
  it('records the two combinations across four selects, bars the grid cell, and flags a spell using one', async () => {
    await startCharacter('magus');

    // Cheap Arts so the barred spell (level 5) is addable: cap = 1+1+3=5.
    await $(ARTS_TAB).waitForExist({ timeout: 30000 });
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('10');
    await raiseArt('art.intellego', 1);
    await raiseArt('art.animal', 1);

    await $(VF_TAB).click();
    const addFlaw = await $('[data-testid="add-flaw.incompatible_arts"]');
    await addFlaw.waitForExist({ timeout: 10000 });
    await addFlaw.click();

    // The four selects, in their two pairs (Intellego+Herbam, Intellego+Animal).
    const technique1 = await $('[data-testid^="param-flaw.incompatible_arts-technique_1-"]');
    const form1 = await $('[data-testid^="param-flaw.incompatible_arts-form_1-"]');
    const technique2 = await $('[data-testid^="param-flaw.incompatible_arts-technique_2-"]');
    const form2 = await $('[data-testid^="param-flaw.incompatible_arts-form_2-"]');
    await technique1.waitForExist({ timeout: STEP_TIMEOUT });
    await technique1.selectByAttribute('value', 'art.intellego');
    await form1.selectByAttribute('value', 'art.herbam');
    await technique2.selectByAttribute('value', 'art.intellego');
    await form2.selectByAttribute('value', 'art.animal');

    // The pair is legal (two genuinely different combinations) — no
    // param-groups-not-distinct complaint.
    expect((await issueCodes()).includes('param_groups_not_distinct')).toBe(false);

    // The (Intellego, Animal) grid cell on the Totals tab must read "Unusable".
    await $(TOTALS_TAB).click();
    await $('[data-testid="derived-technique-select"]').waitForExist({ timeout: STEP_TIMEOUT });
    await $('[data-testid="derived-technique-select"]').selectByAttribute('value', 'art.intellego');
    await $('[data-testid="derived-form-select"]').selectByAttribute('value', 'art.animal');
    const unusable = await $('[data-testid="derived-casting-unusable"]');
    await unusable.waitForExist({ timeout: STEP_TIMEOUT });
    expect(clean(await unusable.getText())).toBe('Unusable');

    // A known spell touching that barred combination directly
    // (spell.image_of_the_beast: Intellego/Animal, level 5) is a creation-time
    // error.
    await $(SPELLS_TAB).click();
    const addSpell = await $('[data-testid="add-spell.image_of_the_beast"]');
    await addSpell.waitForExist({ timeout: STEP_TIMEOUT });
    await browser.waitUntil(async () => !(await isRowBlocked(addSpell)), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'image_of_the_beast should be addable at cap 5 with Intellego/Animal both at 1',
    });
    await addSpell.click();

    await browser.waitUntil(
      async () => (await issueCodes()).includes('spell_uses_incompatible_arts'),
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg:
          'a spell directly using a barred Technique+Form combination should raise spell_uses_incompatible_arts',
      },
    );
  });
});

// End-to-end: a Characteristic-floor prerequisite (Supernatural Beauty
// requires Presence >= 1, ArMDE:5089-5095) is not grey-gated on the Available
// list — the engine cannot evaluate it ahead of the pick in the UI's current
// menu-filtering (that only narrows on House) — so taking it must report
// `prereq_not_met` once Presence is definitely below the floor.
//
// `AppStore.setCharacteristic` deletes the map entry whenever a score resolves
// to 0 (`state.svelte.ts`); since D83.4 the engine's `PrereqCtx` reads an
// ABSENT Characteristic as a definite 0 (plus free deltas), so an untouched
// Presence is refused too. The test decrements Presence to -1 so it also
// covers an explicitly stored entry below the floor.
describe('Characteristic-floor prerequisite reports its issue (Supernatural Beauty)', () => {
  it('flags Supernatural Beauty once Presence is explicitly below the floor', async () => {
    await startCharacter('companion');

    const charsTab = await $('[data-testid="tab-characteristics"]');
    await charsTab.waitForExist({ timeout: 30000 });
    await charsTab.click();
    const dec = await $('[data-testid="char-dec-pre"]');
    await dec.waitForExist({ timeout: 10000 });
    await dec.click();
    await browser.waitUntil(
      async () => clean(await $('[data-testid="char-value-pre"]').getText()) === '-1',
      { timeout: STEP_TIMEOUT, timeoutMsg: 'Presence should read -1 after one decrement' },
    );

    await $(VF_TAB).click();
    const add = await $('[data-testid="add-virtue.supernatural_beauty"]');
    await add.waitForExist({ timeout: 10000 });
    await add.click();

    const issue = await $(`${ISSUE_LIST} li[data-code="prereq_not_met"]`);
    await issue.waitForExist({
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Supernatural Beauty with Presence -1 should report prereq_not_met',
    });
  });
});

// End-to-end: Merinita's conditional Warping Point (ArMDE:2280/D81.14) — "Any
// magus in this House without a faerie-related Virtue or Flaw has a Warping
// Point" — appears the moment the House is chosen and disappears once a
// faerie-related Flaw (Faerie Friend) is taken. Read off the Totals tab's
// `derived-warping` readout ("Score { $score }, Points { $points }"), which is
// the engine-computed TOTAL (entity.warping_points + grants + the House's
// conditional point), never the raw input field.
describe("Merinita's Warping Point appears and disappears with a faerie-related V/F (D81.14)", () => {
  it('adds one conditional Warping Point for Merinita, and removes it once Faerie Friend is taken', async () => {
    await startCharacter('magus');

    await $(HOUSE_TAB).waitForExist({ timeout: 30000 });
    await $(HOUSE_TAB).click();
    const houseSelect = await $('[data-testid="house-select"]');
    await houseSelect.waitForExist({ timeout: 10000 });
    await houseSelect.selectByAttribute('value', 'house.merinita');

    await $(TOTALS_TAB).click();
    const warping = await $('[data-testid="derived-warping"]');
    await warping.waitForExist({ timeout: STEP_TIMEOUT });
    // `endsWith`: the points are the readout's last figure, so 'Points 12' cannot pass.
    const warpingPoints = async () => clean(await warping.getText()).trim();
    await browser.waitUntil(async () => (await warpingPoints()).endsWith('Points 1'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg:
        'a fresh Merinita magus with no faerie-related V/F should carry the conditional Warping Point',
    });

    await $(VF_TAB).click();
    const addFaerieFriend = await $('[data-testid="add-flaw.faerie_friend"]');
    await addFaerieFriend.waitForExist({ timeout: 10000 });
    await addFaerieFriend.click();

    await $(TOTALS_TAB).click();
    await browser.waitUntil(async () => (await warpingPoints()).endsWith('Points 0'), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'taking a faerie-related Flaw should clear the conditional Warping Point',
    });
  });
});
