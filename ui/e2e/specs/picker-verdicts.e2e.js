// End-to-end: real-binary coverage for four manual-test findings
// (docs/manual-testing-findings-2026-11-15.md), all of them the picker's
// Enforced-mode Add-button verdict surfacing a rule the engine already knew
// but the UI did not read:
//
//   F7  — a grog's hard per-type Major-Virtue/Flaw cap (ArMDE:2898) must
//         refuse Redcap (Major, ArMDE:4915-4916) before the pick, not only
//         after (`VfAddBlockReason::ExceedsCap`,
//         `validation/selections.rs::vf_add_verdicts`).
//   F8  — Brutal Artist's House-Jerbiton prerequisite (ArMDE:5830-5833) must
//         read an UNSET House as unmet, not unevaluated
//         (`vf_add_verdicts`'s `unset_house_is_false`).
//   F25 — an Arcane Ability (Magic Theory) needs an authorizing Virtue
//         (ArMDE:2462) before a companion may add it at all, mirroring the
//         Virtue/Flaw picker's own gate
//         (`AbilityAddBlockReason::RequiresVirtue`,
//         `validation/authorization.rs::ability_add_verdicts`).
//   F27 — Vengeful Powers ("may be taken as a Hermetic Flaw", ArMDE:7048)
//         must not be offered under a Hermetic heading to a character the
//         profile forbids that category for, while staying offered under its
//         other, permitted heading (Story)
//         (`validation/selections.rs::forbidden_categories_in_force`).
//
// Each describe builds its own character via `startCharacter` and is
// independent of the others, per the shared helper's own rationale.

import { $, $$, browser, expect } from '@wdio/globals';

import {
  dismissTooltip,
  hoverForTooltip,
  isRowBlocked,
  setValidationMode,
  startCharacter,
  STEP_TIMEOUT,
} from '../helpers.js';

const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
const HOUSE_TAB = '[data-testid="tab-house_specialisation"]';
const HOUSE_SELECT = '[data-testid="house-select"]';
const TOOLTIP_REASON = '[data-testid="tooltip-reason"]';

// End-to-end: F7 (manual-test findings 2026-11-15 #7). A grog's budget sets
// `max_major_virtues: 0` (ArMDE:2898, "may not take Major Virtues or Flaws"),
// a HARD cap — so Redcap (Major, Social Status, ArMDE:4915-4916) must be
// refused by the Add button itself in Enforced mode, not merely rejected by
// validation after the pick.
describe("a grog's Add button refuses a Virtue past the hard magnitude cap (F7)", () => {
  it('greys out Redcap for a grog, with the cap-reached reason, and the click adds nothing', async () => {
    await startCharacter('grog');
    // Pinned explicitly: ValidationMode persists in the worker's settings
    // file, so a retry (or an earlier spec that left this worker in another
    // mode) must not silently start this test from the wrong one.
    await setValidationMode('enforced');

    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(VF_TAB).click();
    const addRedcap = await $('[data-testid="add-virtue.redcap"]');
    await addRedcap.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => isRowBlocked(addRedcap), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: "Redcap should be add-blocked for a grog's zero Major-Virtue cap",
    });

    await hoverForTooltip(addRedcap);
    const reason = await $(TOOLTIP_REASON);
    await reason.waitForExist({ timeout: 5000 });
    expect((await reason.getText()).trim()).toBe('Maximum for this character type already reached');
    await dismissTooltip(addRedcap);

    // Enforced mode's own refusal (`onclick` no-ops when `aria-disabled`) means
    // the illegal pick is never reached by a click at all.
    await addRedcap.click();
    expect(await $('[data-testid^="remove-virtue.redcap-"]').isExisting()).toBe(false);
  });
});

// End-to-end: F8 (manual-test findings 2026-11-15 #8, Norbert's ruling
// 2026-11-15). Brutal Artist requires Order membership and House Jerbiton
// (ArMDE:5830-5833); a fresh magus's House is unset, and an UNSET House must
// count as a definitely unmet prerequisite for the Add button's purposes
// (greyed, same as any other magus's House), not the usual "unevaluated"
// reading an already-held item gets. Choosing Jerbiton then opens it.
describe('Brutal Artist refuses an unset House, and opens once Jerbiton is chosen (F8)', () => {
  it('greys out Brutal Artist with no House set, then allows and completes the pick for Jerbiton', async () => {
    await startCharacter('magus');
    await setValidationMode('enforced');

    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(VF_TAB).click();
    const addBrutalArtist = await $('[data-testid="add-flaw.brutal_artist"]');
    await addBrutalArtist.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => isRowBlocked(addBrutalArtist), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Brutal Artist should be add-blocked while the House is unset',
    });

    await hoverForTooltip(addBrutalArtist);
    const reason = await $(TOOLTIP_REASON);
    await reason.waitForExist({ timeout: 5000 });
    expect((await reason.getText()).trim()).toBe('Does not meet a requirement');
    await dismissTooltip(addBrutalArtist);

    await $(HOUSE_TAB).click();
    const houseSelect = await $(HOUSE_SELECT);
    await houseSelect.waitForExist({ timeout: 10000 });
    await houseSelect.selectByAttribute('value', 'house.jerbiton');

    await $(VF_TAB).click();
    const addAfterHouse = await $('[data-testid="add-flaw.brutal_artist"]');
    await browser.waitUntil(async () => !(await isRowBlocked(addAfterHouse)), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Brutal Artist should become addable once House Jerbiton is chosen',
    });
    await addAfterHouse.click();
    await $('[data-testid^="remove-flaw.brutal_artist-"]').waitForExist({
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the pick should actually complete once the prerequisite is met',
    });
  });
});

// End-to-end: F25 (manual-test findings 2026-11-15 #25). Arcane/Academic/
// Martial Abilities need an authorizing Virtue (ArMDE:2462) before a
// companion may learn them at all; the Ability picker's Add button must
// refuse them up front in Enforced mode, mirroring the Virtue/Flaw picker's
// own category gate, and let the same pick through in Advisory.
describe('a companion cannot add an Arcane Ability with no authorizing Virtue (F25)', () => {
  it('greys out Magic Theory in Enforced mode, naming the Arcane category, and allows it in Advisory', async () => {
    await startCharacter('companion');
    await setValidationMode('enforced');

    await $(ABILITIES_TAB).waitForExist({ timeout: 30000 });
    await $(ABILITIES_TAB).click();
    const addMagicTheory = await $('[data-testid="add-ability.magic_theory"]');
    await addMagicTheory.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => isRowBlocked(addMagicTheory), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Magic Theory should be add-blocked for a companion with no authorizing Virtue',
    });

    await hoverForTooltip(addMagicTheory);
    const reason = await $(TOOLTIP_REASON);
    await reason.waitForExist({ timeout: 5000 });
    expect(await reason.getText()).toContain('Arcane');
    await dismissTooltip(addMagicTheory);

    await setValidationMode('advisory');
    await browser.waitUntil(async () => !(await isRowBlocked(addMagicTheory)), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Magic Theory should become addable in Advisory mode',
    });
    await addMagicTheory.click();
    await $('[data-testid^="remove-ability.magic_theory-"]').waitForExist({
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the pick should actually complete once Enforced mode is no longer gating it',
    });

    // The mode persists in the worker's settings file; restore the default.
    await setValidationMode('enforced');
  });
});

// End-to-end: F27 (manual-test findings 2026-11-15 #27, Norbert's ruling).
// Vengeful Powers carries both `story` and `hermetic` categories and "may be
// taken as a Hermetic Flaw" (ArMDE:7048), but a Gift-less companion forbids
// `hermetic` outright — so it must not be offered under a Hermetic heading
// for one, while staying offered under its other, permitted Story heading.
describe('Vengeful Powers lists under Story but not Hermetic for a Gift-less companion (F27)', () => {
  // The item's add button, and the category heading its enclosing `<ul>`
  // renders directly after (`VirtueFlawTab.svelte` alternates one `<h3
  // class="category">` per `<ul class="item-list">`, so the immediately
  // preceding element sibling of a button's own list is that section's
  // heading).
  async function headingTextFor(testid) {
    return browser.execute((selector) => {
      const button = document.querySelector(selector);
      if (!button) return null;
      const list = button.closest('ul.item-list');
      const heading = list ? list.previousElementSibling : null;
      return heading ? heading.textContent.trim() : null;
    }, `[data-testid="${testid}"]`);
  }

  it('offers exactly one Add button for Vengeful Powers, filed under Story', async () => {
    await startCharacter('companion');
    await setValidationMode('enforced');

    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(VF_TAB).click();
    const add = await $('[data-testid="add-flaw.vengeful_powers"]');
    await add.waitForExist({ timeout: 10000 });

    // Exactly one row exists at all — it is not duplicated under both
    // headings, which the forbidden-category drop would otherwise still
    // have left it to be.
    expect((await $$('[data-testid="add-flaw.vengeful_powers"]')).length).toBe(1);

    await browser.waitUntil(
      async () => (await headingTextFor('add-flaw.vengeful_powers')) !== null,
      {
        timeout: STEP_TIMEOUT,
        timeoutMsg: 'Vengeful Powers should render under some category heading',
      },
    );
    expect(await headingTextFor('add-flaw.vengeful_powers')).toBe('Story');

    // A Gift-less companion forbids `hermetic` outright, but `story` is
    // permitted — so the row that remains is not itself add-blocked.
    expect(await isRowBlocked(add)).toBe(false);
  });
});
