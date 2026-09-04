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
//
// NOTE: requires the production binary; the display comes from your desktop session
// or, when DISPLAY is unset, the Xvfb one WebdriverIO starts (see e2e/README.md).
// The wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, browser, expect } from '@wdio/globals';

import {
  advanceWizardTo,
  BOOT_TIMEOUT,
  startCharacter,
  startWizard,
  STEP_TIMEOUT,
} from '../helpers.js';

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

describe('layout stability under first interaction', () => {
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
