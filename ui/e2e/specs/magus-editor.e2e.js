// End-to-end: magus-only editor surfaces — Hermetic Arts, Hermetic Houses, the
// selected-list frame's scroll containment, the Totals tab's duplicate-key
// regression guard, repeated parameterized Virtues, Mythic Companion types, and
// per-type profile budgets/category rules.
//
// A2 merged seven previously separate spec files into this one. Each describe
// keeps its own local selectors and helpers scoped to its block, exactly as
// when it was a standalone file, since several of them reused the same
// constant names (`VF_TAB`, `ARTS_TAB`, `VIRTUE_BUDGET`,
// `FLAW_BUDGET`) with the same values but no reason to share a single
// module-level binding.
//
// `hermetic arts`'s closing test switches the UI to German and restores `en` in
// its own body before this file's later describes run — the same pattern kept
// at `wizard-flow.e2e.js` (ex-`tab-area.e2e.js`) and `magus-possessions.e2e.js`
// (ex-`export-markdown.e2e.js`).

import { $, $$, browser, expect } from '@wdio/globals';

import {
  clean,
  isRowBlocked,
  setLanguage,
  SETTLE_TIMEOUT,
  startCharacter,
  STEP_TIMEOUT,
  waitForBalancePoints,
} from '../helpers.js';

// End-to-end: Hermetic Arts (magus-only). The Arts tab appears only for a magus,
// an Art is bought against the shared XP pool, and Puissant Art adds +3 to the
// targeted Art's effective score.
describe('hermetic arts', () => {
  const ARTS_TAB = '[data-testid="tab-arts"]';
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';

  it('shows the Arts tab only for a magus', async () => {
    // A companion has no Arts tab...
    await startCharacter('companion');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    expect(await $(ARTS_TAB).isExisting()).toBe(false);

    // ...while a magus does (capability flag, not the type id).
    await startCharacter('magus');
    await $(ARTS_TAB).waitForExist({ timeout: 5000 });
  });

  it('buys an Art against the shared XP pool', async () => {
    await startCharacter('magus');
    await $(ARTS_TAB).click();

    // Fund the shared pool. All 15 Arts are always shown — raise Creo to 5
    // directly (no pick step); triangular cost = 15 XP.
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('20');

    const inc = await $('[data-testid="art-inc-art.creo"]');
    await inc.waitForExist({ timeout: 5000 });
    for (let i = 0; i < 5; i++) await inc.click();

    await browser.waitUntil(
      async () => clean(await $('[data-testid="art-xp-spent"]').getText()).includes('15'),
      { timeout: 5000, timeoutMsg: 'Creo 5 should cost 15 Art XP' },
    );
    expect(clean(await $('[data-testid="art-xp-available"]').getText())).toContain('5');
  });

  // Regression: the bar must show an OVERSPENT pool as a negative Available. This
  // needs the real binary — `xp_general_used` alone can never express it (it is a
  // max-flow value capped by the pool), so a unit test with a hand-written
  // `general_used > pool` payload asserts a state the engine cannot produce and
  // passes even when the app shows 0. Only the live IPC payload proves it.
  it('shows an overspent XP pool as a negative Available', async () => {
    // Deliberately NO startCharacter here: this test carries on from the magus the
    // previous one funded, because reaching an overspent pool needs that state.
    await $(ARTS_TAB).click();

    // Carrying on from the previous test: pool 20, Creo 5 (15 XP). Raising Creo to
    // 7 costs 28 XP (triangular), so the 20-point pool is overspent by 8.
    const inc = await $('[data-testid="art-inc-art.creo"]');
    await inc.waitForExist({ timeout: 5000 });
    for (let i = 0; i < 2; i++) await inc.click();

    const spent = await $('[data-testid="art-xp-spent"]');
    const available = await $('[data-testid="art-xp-available"]');
    await browser.waitUntil(
      async () =>
        clean(await spent.getText()).includes('28') &&
        // ASCII hyphen-minus, never U+2212.
        clean(await available.getText()).includes('-8'),
      {
        timeout: 5000,
        timeoutMsg: 'Creo 7 (28 XP) against a 20-point pool should read Available: -8',
      },
    );

    // Both figures carry the stable semantic signal (styling classes can change
    // freely without breaking this assertion).
    expect(await spent.getAttribute('data-overspent')).toBe('true');
    expect(await available.getAttribute('data-overspent')).toBe('true');

    // Restore a legal pool so later specs start from a funded state.
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.setValue('40');
    await browser.waitUntil(async () => clean(await available.getText()).includes('12'), {
      timeout: 5000,
      timeoutMsg: 'a 40-point pool should cover Creo 7 (28 XP) with 12 left',
    });
  });

  it('applies Puissant Art as a +3 effective bonus on the targeted Art', async () => {
    await startCharacter('magus');

    // Add Puissant Art on the Virtues & Flaws tab and target Ignem.
    await $(VF_TAB).click();
    const addPuissant = await $('[data-testid="add-virtue.puissant_art"]');
    await addPuissant.waitForExist({ timeout: 10000 });
    await addPuissant.click();
    const target = await $('[data-testid^="param-virtue.puissant_art-art-"]');
    await target.waitForExist({ timeout: 5000 });
    await target.selectByAttribute('value', 'art.ignem');

    // Raise Ignem to 2 on the Arts tab → effective 2 + 3 = 5.
    await $(ARTS_TAB).click();
    const inc = await $('[data-testid="art-inc-art.ignem"]');
    await inc.waitForExist({ timeout: 5000 });
    await inc.click();
    await inc.click();

    const eff = await $('[data-testid="art-eff-art.ignem"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('5'), {
      timeout: 5000,
      timeoutMsg: 'Puissant Art should make Ignem 2 read as effective 5',
    });
  });

  it('shows the single-row XP summary and localizes it to German (Issue G)', async () => {
    await startCharacter('magus');
    await $(ARTS_TAB).click();

    // The general-pool total is the only editable field and retains its value.
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('30');
    await browser.waitUntil(async () => (await pool.getValue()) === '30', {
      timeout: 5000,
      timeoutMsg: 'the editable general-pool total should retain the entered value',
    });

    // English: the Available readout renders in the same row with its label.
    const available = await $('[data-testid="art-xp-available"]');
    await available.waitForExist({ timeout: 5000 });
    expect(clean(await available.getText())).toContain('Available');

    // German: switching the language re-localizes the same XP row. The control
    // lives in the settings dialog since C4, so the switch goes through the helper
    // that opens it, chooses and closes it again.
    await setLanguage('de');
    await browser.waitUntil(async () => clean(await available.getText()).includes('Verfügbar'), {
      timeout: 5000,
      timeoutMsg: 'the XP row Available label should localize to German',
    });

    // Restore English so later specs run against the default locale.
    await setLanguage('en');
  });
});

// End-to-end: Hermetic Houses (magus-only). Selecting a House auto-grants its
// free Virtue — free of the point budget — Mystery Houses seed a Supernatural
// Ability at an effective floor of 1, a `choice` grant offers its options and an
// unresolved pick raises a localized issue, and an `open` grant is additive (it
// never charges the bought budget).
//
// Coverage boundary (mirrors validation-errors in `companion-editor.e2e.js`):
// two Phase-4 facts are NOT click-reachable in the shipped `rules/core/` seed,
// so they are asserted by the Rust unit/integration tests (plan step 10), not
// here:
//   * The `too_many_major_hermetic_virtues` cap TRIP — the seed has exactly one
//     Major Hermetic Virtue (`virtue.gentle_gift`, non-repeatable), so a second
//     bought one cannot be reached through clicks. What IS reachable and checked
//     below: a House's granted (Minor) Virtue never contributes to the cap.
//   * Ex Miscellanea's `ex_misc_major_virtue` open pick — the seed has no Major
//     non-Hermetic Virtue, so that picker is empty. The reachable Ex-Misc fact
//     checked below is that an `open` grant is additive to the bought budget.
describe('hermetic houses', () => {
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const ABILITIES_TAB = '[data-testid="tab-abilities"]';
  const ARTS_TAB = '[data-testid="tab-arts"]';
  const HOUSE_TAB = '[data-testid="tab-house_specialisation"]';
  const HOUSE_SELECT = '[data-testid="house-select"]';

  async function selectHouse(value) {
    await $(HOUSE_TAB).click();
    const select = await $(HOUSE_SELECT);
    await select.waitForExist({ timeout: 10000 });
    await select.selectByAttribute('value', value);
  }

  // Add an Ability from the picker (grant-seeded abilities must be added before
  // the abilities view can show their granted effective floor — same as the
  // Second Sight grant in `companion-editor.e2e.js`).
  async function addAbility(id) {
    await $(ABILITIES_TAB).click();
    const add = await $(`[data-testid="add-${id}"]`);
    await add.waitForExist({ timeout: 10000 });
    await add.click();
  }

  it('shows the House tab only for a magus', async () => {
    // A companion has no House tab (gated on the capability flag).
    await startCharacter('companion');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    expect(await $(HOUSE_TAB).isExisting()).toBe(false);

    // A magus has one, and the HouseSelector renders behind it.
    await startCharacter('magus');
    await $(HOUSE_TAB).waitForExist({ timeout: 5000 });
    await $(HOUSE_TAB).click();
    await $('[data-testid="house-selector"]').waitForExist({ timeout: 5000 });
    await $(HOUSE_SELECT).waitForExist({ timeout: 5000 });
  });

  it('grants a fixed Virtue for free — Bjornaer confers Heartbeast', async () => {
    await startCharacter('magus');

    // Record the bought-virtue balance BEFORE choosing a House. A granted Virtue
    // must not change it (compute_balance counts bought selections only).
    await $(VF_TAB).click();
    const balance = await $('[data-testid="balance-virtues"]');
    await balance.waitForExist({ timeout: 10000 });
    const before = clean(await balance.getText());

    await selectHouse('house.bjornaer');

    // The free Virtue shows read-only in the House panel…
    const granted = await $('[data-testid="house-granted-virtue.heartbeast"]');
    await granted.waitForExist({ timeout: 5000 });

    // …and its ability_score_grant floors Heartbeast at effective 1 for 0 XP.
    // Match by testid prefix: the index suffix is the ability-array position, which
    // is not this spec's subject.
    await addAbility('ability.heartbeast');
    const eff = await $('[data-testid^="ability-eff-ability.heartbeast-"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('1'), {
      timeout: 5000,
      timeoutMsg: 'granted Heartbeast should read as effective 1',
    });
    expect(await $('[data-testid^="ability-score-ability.heartbeast-"]').getText()).toBe('0');

    // The bought-virtue balance is unchanged — the grant is free and, being a
    // Minor Virtue, never touches the Major-Hermetic-Virtue cap either.
    await $(VF_TAB).click();
    expect(clean(await balance.getText())).toBe(before);
  });

  // guided-creation-review-2026-08 #9: the granted rows used to be appended as a
  // HEADER-LESS group below the chosen ones, so they appeared under whichever
  // category heading happened to sort last — a Hermetic granted Virtue read as
  // Supernatural. They now join the ordinary category-grouped list.
  //
  // The heading is compared against the row's OWN category badge rather than a
  // hardcoded word, so the assertion holds in either locale and names no label.
  it('lists a granted Virtue under its own category heading, inline with the bought rows', async () => {
    await startCharacter('magus');
    // Bjornaer grants Heartbeast, a MINOR HERMETIC Virtue.
    await selectHouse('house.bjornaer');

    await $(VF_TAB).click();
    // Buy a Hermetic Virtue too, so the granted row has a bought neighbour inside
    // its own group and "inline with the bought rows" is actually observable.
    const add = await $('[data-testid="add-virtue.hermetic_prestige"]');
    await add.waitForExist({ timeout: 10000 });
    await add.waitForClickable({ timeout: 10000 });
    await add.click();

    const granted = await $('[data-testid^="granted-selection-virtue.heartbeast"]');
    await granted.waitForExist({ timeout: 10000 });

    // Read the group the granted row actually sits in. `closest('ul')` plus a walk
    // back over the previous siblings is the only way to tell "under this heading"
    // from "after this heading": the bug was precisely a row rendering in a list
    // that had no heading of its own.
    const group = await browser.waitUntil(
      async () => {
        const info = await browser.execute(() => {
          const row = document.querySelector(
            '[data-testid^="granted-selection-virtue.heartbeast"]',
          );
          if (!row) return null;
          const list = row.closest('ul');
          if (!list) return null;
          let heading = null;
          for (let node = list.previousElementSibling; node; node = node.previousElementSibling) {
            // A preceding <ul> means this list has no heading of its own — the
            // header-less group that #9 removed.
            if (node.tagName === 'UL') break;
            if (node.tagName === 'H3') {
              heading = node.textContent.trim();
              break;
            }
          }
          const rows = [...list.children];
          return {
            heading,
            category: row.querySelector('.badge.type')?.textContent.trim() ?? null,
            marker: row.querySelector('.row-marker')?.textContent.trim() ?? '',
            hasRemoveButton: row.querySelector('button') !== null,
            // Rows the player bought carry a remove button; the granted one does not.
            boughtNeighbours: rows.filter((li) => li !== row && li.querySelector('button')).length,
            names: rows.map((li) => li.querySelector('.item-name')?.textContent.trim() ?? ''),
          };
        });
        return info && info.category ? info : false;
      },
      {
        timeout: 10000,
        timeoutMsg: 'the granted Virtue row never rendered with its category badge',
      },
    );

    // Its group's heading IS its own category — not whatever sorted last.
    expect(clean(group.heading ?? '')).toBe(clean(group.category));
    // Inline with the bought rows of that same category …
    expect(group.boughtNeighbours).toBeGreaterThan(0);
    // … and alpha-ordered among them: Heartbeast before Hermetic Prestige.
    const names = group.names.map(clean);
    expect(names.findIndex((n) => n.includes('Heartbeast'))).toBeLessThan(
      names.findIndex((n) => n.includes('Hermetic Prestige')),
    );
    // The "Granted" marker is the row's only distinction: no remove button.
    expect(group.marker.length).toBeGreaterThan(0);
    expect(group.hasRemoveButton).toBe(false);
  });

  it('seeds a Mystery Ability at an effective floor of 1 — Merinita → Faerie Magic', async () => {
    await startCharacter('magus');
    await selectHouse('house.merinita');

    await addAbility('ability.faerie_magic');
    // Match by testid prefix — the ability-array index is not the subject.
    const eff = await $('[data-testid^="ability-eff-ability.faerie_magic-"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('1'), {
      timeout: 5000,
      timeoutMsg: 'granted Faerie Magic should read as effective 1',
    });
    // No experience was charged — the bought score stays 0.
    expect(await $('[data-testid^="ability-score-ability.faerie_magic-"]').getText()).toBe('0');
  });

  it('offers a choice grant, flags an unresolved pick, and applies the chosen bonus', async () => {
    await startCharacter('magus');
    await selectHouse('house.flambeau');

    // The choice select offers both Puissant Perdo and Puissant Ignem.
    const choice = await $('[data-testid="house-choice-flambeau_puissant"]');
    await choice.waitForExist({ timeout: 5000 });
    const options = await choice.$$('option');
    const labels = [];
    for (const opt of options) labels.push(clean(await opt.getText()));
    expect(labels.some((l) => l.includes('Perdo'))).toBe(true);
    expect(labels.some((l) => l.includes('Ignem'))).toBe(true);

    // With no pick made, the House flags the choice as unresolved.
    await browser.waitUntil(
      async () => await $('[data-code="house_choice_unresolved"]').isExisting(),
      { timeout: 5000, timeoutMsg: 'an unresolved choice should raise house_choice_unresolved' },
    );

    // Pick Puissant Ignem (match by label, not a positional value).
    const ignemIndex = labels.findIndex((l) => l.includes('Ignem'));
    await choice.selectByIndex(ignemIndex);

    // The issue clears once the choice resolves.
    await browser.waitUntil(
      async () => !(await $('[data-code="house_choice_unresolved"]').isExisting()),
      { timeout: 5000, timeoutMsg: 'resolving the choice should clear the issue' },
    );

    // The granted Puissant adds +3 to Ignem. The effective badge only renders for
    // Arts present in the score list (art_bonuses is keyed by bought Arts), so
    // raise Ignem to 2 first, then it reads effective 2 + 3 = 5.
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('20');
    const inc = await $('[data-testid="art-inc-art.ignem"]');
    await inc.waitForExist({ timeout: 5000 });
    await inc.click();
    await inc.click();

    const artEff = await $('[data-testid="art-eff-art.ignem"]');
    await artEff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await artEff.getText()).includes('5'), {
      timeout: 5000,
      timeoutMsg: 'granted Puissant Ignem should make Ignem 2 read effective 5',
    });
  });

  it('adds an open grant on top of the bought budget — Ex Miscellanea', async () => {
    await startCharacter('magus');

    await $(VF_TAB).click();
    const balance = await $('[data-testid="balance-virtues"]');
    await balance.waitForExist({ timeout: 10000 });
    const before = clean(await balance.getText());

    await selectHouse('house.ex_miscellanea');

    // The Minor-Hermetic-Virtue open picker offers eligible items; pick the first
    // real option (the prompt option carries an empty value).
    const open = await $('[data-testid="house-open-ex_misc_minor_virtue"]');
    await open.waitForExist({ timeout: 5000 });
    const options = await open.$$('option');
    let picked = null;
    for (const opt of options) {
      const value = await opt.getValue();
      if (value) {
        picked = value;
        break;
      }
    }
    expect(picked).not.toBe(null);
    await open.selectByAttribute('value', picked);

    // The granted Virtue is "in addition to the normal allowance" — the bought
    // virtue balance is untouched.
    await $(VF_TAB).click();
    expect(clean(await balance.getText())).toBe(before);
  });

  it('lets a parameterized open pick choose its parameter — Puissant (Art)', async () => {
    await startCharacter('magus');
    await selectHouse('house.ex_miscellanea');

    // Puissant Art is a Minor Hermetic Virtue, so the Ex Misc Minor slot admits
    // it — and it declares an Art parameter that must be named.
    const open = await $('[data-testid="house-open-ex_misc_minor_virtue"]');
    await open.waitForExist({ timeout: 5000 });
    await open.selectByAttribute('value', 'virtue.puissant_art');

    // Unparameterized, the engine reports the parameter missing…
    await browser.waitUntil(async () => await $('[data-code="missing_param"]').isExisting(), {
      timeout: 5000,
      timeoutMsg: 'a parameterized open pick with no parameter should report missing_param',
    });

    // …and the picker under the pick resolves it.
    const param = await $('[data-testid="param-virtue.puissant_art-art-ex_misc_minor_virtue"]');
    await param.waitForExist({ timeout: 5000 });
    await param.selectByAttribute('value', 'art.ignem');

    await browser.waitUntil(async () => !(await $('[data-code="missing_param"]').isExisting()), {
      timeout: 5000,
      timeoutMsg: 'choosing the parameter should clear missing_param',
    });
  });
});

// End-to-end: the Selected list must scroll INSIDE the golden frame, never bleed
// out of it. All four list tabs share one skeleton (`.region-row` ->
// `.region-selected` -> `.selected-frame`), so this checks it from two sides:
// Virtues & Flaws (two grouped columns inside the frame) and Equipment (a single
// flat list). Geometry is the whole subject, so it can only be tested against a
// real layout engine — vitest renders these components to an SSR string and never
// resolves a box.
//
// The reported bleed was NOT a sizing bug: the frame has always been bounded and
// scrolled internally (client 299 / scroll 2275 with 32 rows at an 800px window).
// It was a CLIP bug. `overflow` clips at the PADDING box — the inner edge of the
// border — so while the frame was itself the scrollport, its own padding belonged
// to the scrolling area: scrolled rows and the panel's opaque background moved into
// that padding and were sliced flush against the 1px gold border, covering it. The
// frame now keeps the border and padding while a nested scrollport does the
// scrolling, which reserves that padding as an unscrollable gap. The third test
// below pins exactly that.
describe('selected frame', () => {
  // The frame and the tab area carry no data-testid: they ARE layout elements, and
  // what this spec asserts is precisely the geometry of `.selected-frame` inside
  // `main.tab-content` (ui/src/app.css). So both are addressed by class here —
  // unlike a state/content assertion, the class itself is the subject under test,
  // so there is no more-stable alternative to key on.
  const FRAME = '.region-selected > .selected-frame';
  const TAB_CONTENT = 'main.tab-content';

  // Enough rows that the list cannot possibly fit: the window is 1400x900 (see
  // crates/arm-app/tauri.conf.json), which leaves the frame well under 700px, while
  // a selected V/F row is ~44px and an equipment row ~34px. The exact numbers do not
  // matter — `expectContainedScrollport` asserts the overflow really happened, so an
  // insufficient count fails loudly instead of passing vacuously.
  const VF_ROWS = 16;
  const EQUIPMENT_ROWS = 20;

  /**
   * Live geometry of the selected frame, the scrollport inside it, and the tab area
   * that must contain it. The scrollport is found by behaviour, not by class — the
   * frame itself once played that role — so these assertions survive a change of
   * which element carries the `overflow`.
   */
  async function frameMetrics() {
    const metrics = await browser.execute(
      (frameSelector, contentSelector) => {
        const frame = document.querySelector(frameSelector);
        const content = document.querySelector(contentSelector);
        if (!frame || !content) return null;
        const scrolls = (el) => {
          const overflowY = getComputedStyle(el).overflowY;
          return (
            (overflowY === 'auto' || overflowY === 'scroll') && el.scrollHeight > el.clientHeight
          );
        };
        const scrollport = [frame, ...frame.querySelectorAll('*')].find(scrolls) ?? null;
        const frameRect = frame.getBoundingClientRect();
        const contentRect = content.getBoundingClientRect();
        const portRect = scrollport ? scrollport.getBoundingClientRect() : null;
        return {
          scrollHeight: scrollport ? scrollport.scrollHeight : frame.scrollHeight,
          clientHeight: scrollport ? scrollport.clientHeight : frame.clientHeight,
          frameTop: frameRect.top,
          frameBottom: frameRect.bottom,
          contentBottom: contentRect.bottom,
          // Unscrollable space between the frame's border and the scrolling area, i.e.
          // the gap the padding is supposed to reserve on each edge.
          gapTop: portRect ? portRect.top - frameRect.top : 0,
          gapBottom: portRect ? frameRect.bottom - portRect.bottom : 0,
        };
      },
      FRAME,
      TAB_CONTENT,
    );
    if (metrics === null) throw new Error('the selected frame or the tab area is not rendered');
    return metrics;
  }

  /**
   * The three things a correct frame does, given a list too long to fit.
   * Assertion 0 is the precondition that makes the other two meaningful.
   */
  function expectContainedScrollport(metrics) {
    // 0. The list genuinely does not fit: its content is taller than the tallest
    //    frame that could ever sit between the frame's top and the bottom of the tab
    //    area. Without this, a frame with a handful of rows satisfies everything
    //    below without exercising the bug at all.
    const tallestPossibleFrame = metrics.contentBottom - metrics.frameTop;
    expect(metrics.scrollHeight).toBeGreaterThan(tallestPossibleFrame);

    // 1. The overflow is scrollable INSIDE the frame: there is a scrollport, and it
    //    is shorter than its content. A frame that grew to the full list height would
    //    have no scrollport at all.
    expect(metrics.scrollHeight).toBeGreaterThan(metrics.clientHeight);

    // 1b. The scrolling area stops short of the border on both edges. `overflow` clips
    //     at the padding box, so if the frame is its own scrollport its padding scrolls
    //     WITH the content: rows and the panel's opaque background then slide right up
    //     against the 1px gold border and cover it — the reported "bleeds out over the
    //     golden frame". A nested scrollport keeps that padding unscrollable. 4px is
    //     half the 0.5rem padding, so this rejects the flush clip without pinning the
    //     exact padding.
    expect(metrics.gapTop).toBeGreaterThanOrEqual(4);
    expect(metrics.gapBottom).toBeGreaterThanOrEqual(4);

    // 2. And the frame stays inside the tab area, so its bottom border is on screen
    //    rather than clipped by `.tab-content { overflow: hidden }`. One pixel of
    //    tolerance for sub-pixel rect rounding.
    expect(metrics.frameBottom).toBeLessThanOrEqual(metrics.contentBottom + 1);

    // 3. The frame did not collapse the other way either: a `1fr` row that resolved
    //    to zero would satisfy 1 and 2 while showing an empty frame. 40px is under
    //    one row's height, so this only rejects a collapse.
    expect(metrics.clientHeight).toBeGreaterThan(40);
  }

  async function clickTab(id) {
    const tab = await $(`[data-testid="tab-${id}"]`);
    await tab.waitForExist({ timeout: 10000 });
    await tab.click();
  }

  /**
   * The `data-testid`s of the first `count` ENABLED add controls in the active
   * source list, read in one round trip. Ids are never hardcoded: catalogue size and
   * contents are data, so the spec asks the rendered list what it can add.
   *
   * `SourcePicker` rows stay natively enabled and signal "blocked" through
   * `aria-disabled` rather than the `disabled` attribute (see `isRowBlocked` in
   * `helpers.js`), so the filter below reads that attribute directly rather than
   * the DOM `disabled` property, which these rows never set.
   */
  async function enabledAddIds(prefix, count) {
    const ids = await browser.execute(
      (testidPrefix, limit) =>
        Array.from(document.querySelectorAll(`[data-testid^="${testidPrefix}"]`))
          .filter((button) => button.getAttribute('aria-disabled') !== 'true')
          .slice(0, limit)
          .map((button) => button.getAttribute('data-testid')),
      prefix,
      count,
    );
    expect(ids.length).toBe(count);
    return ids;
  }

  it('scrolls a long Virtues list inside the frame instead of past its bottom border', async () => {
    // A magus: the widest V/F catalogue, and the mandatory traits (The Gift,
    // Hermetic Magus) add their own rows to the selected side. Freshly created, so
    // the row counts below are this spec's own picks and nothing else.
    await startCharacter('magus');

    await clickTab('virtues_flaws');
    const frame = await $(FRAME);
    await frame.waitForExist({ timeout: 10000 });

    // Fill the Virtues column past the frame's height. Over-budget is fine — the
    // engine reports it in the issues panel; nothing here depends on a legal build.
    // Spare candidates are fetched because a pick can grey a later one out (Major
    // vs Minor of the same Virtue and friends), and each is re-checked before the
    // click so an incompatible row is skipped rather than silently swallowed.
    for (const id of await enabledAddIds('add-virtue.', VF_ROWS + 8)) {
      if ((await $$('[data-testid^="remove-virtue."]')).length >= VF_ROWS) break;
      const add = await $(`[data-testid="${id}"]`);
      if (!(await isRowBlocked(add))) await add.click();
    }
    await browser.waitUntil(
      async () => (await $$('[data-testid^="remove-virtue."]')).length >= VF_ROWS,
      {
        timeout: 15000,
        timeoutMsg: `the selected side should hold ${VF_ROWS} chosen Virtues`,
      },
    );

    expectContainedScrollport(await frameMetrics());
  });

  it('scrolls a long Equipment list inside the frame instead of past its bottom border', async () => {
    // Its own character again: the previous test filled the Virtues column, and the
    // row count below must be only what this test carries.
    await startCharacter('magus');

    await clickTab('equipment');
    const frame = await $(FRAME);
    await frame.waitForExist({ timeout: 10000 });

    // Carried equipment is duplicable (`addEquipment` appends a slot per click), so
    // one catalogue entry clicked repeatedly fills the list — no assumption about
    // how many weapons/shields/armor the rules ship.
    const [addId] = await enabledAddIds('add-', 1);
    const add = await $(`[data-testid="${addId}"]`);
    // One click, then its row, before the next: back-to-back clicks can lose one
    // to the issues footer growing under the pointer (see `waitForBalancePoints`
    // in helpers.js). Equipment spends no Virtue points, so the row count is the
    // settle signal here.
    const carried = async () => (await $$('[data-testid^="equipment-name-"]')).length;
    const before = await carried();
    for (let i = 0; i < EQUIPMENT_ROWS; i++) {
      await add.click();
      await browser.waitUntil(async () => (await carried()) === before + i + 1, {
        timeout: SETTLE_TIMEOUT,
        timeoutMsg: `Add click ${i + 1} of ${EQUIPMENT_ROWS} never rendered carried item ${i}`,
      });
    }
    await browser.waitUntil(
      async () => (await $$('[data-testid^="equipment-name-"]')).length >= EQUIPMENT_ROWS,
      {
        timeout: 15000,
        timeoutMsg: `the selected side should hold ${EQUIPMENT_ROWS} carried items`,
      },
    );

    expectContainedScrollport(await frameMetrics());
    // No clean-up needed: every spec after this one creates its own character
    // through `startCharacter`, which discards these rows.
  });
});

// End-to-end: the Totals tab must render for a character who carries the SAME
// weapon twice. `combat_totals` emits one line per equipped slot
// (`crates/arm-rules/src/derived/combat.rs::combat_totals`) and
// `addEquipment` never dedups, so two slots holding one catalogue id
// produce two combat lines with the same
// `line.weapon`. Keying that `{#each}` by `line.weapon` made Svelte throw
// `each_key_duplicate` — in production as well as dev — which aborted the whole
// panel's render: the tab button lit up, the Totals panel never mounted, and the
// previously selected tab's DOM was never torn down. That is the reported symptom
// ("clicking Totals does nothing"), and only the real binary can catch it: vitest
// renders through `svelte/server`, which does not validate `{#each}` keys.
//
// The spec then equips a shield, which splits every one-handed weapon into a
// with-shield and a bare line — a second, now-ordinary source of duplicate
// `line.weapon` values, and the reason that `{#each}` must stay unkeyed forever.
describe('totals tab', () => {
  const EQUIPMENT_TAB = '[data-testid="tab-equipment"]';
  const TOTALS_TAB = '[data-testid="tab-totals"]';

  const COMBAT = '[data-testid="derived-combat"]';
  const COMBAT_ROWS = '[data-testid="derived-combat"] tbody tr';
  const EQUIPMENT_LIST = '[data-testid="equipment-list"]';

  // The catalogue weapon carried twice, and its English display name.
  const WEAPON = 'weapon.sword_long';
  const WEAPON_NAME = 'Long Sword';

  // The shield equipped afterwards, splitting each weapon's line in two.
  const SHIELD = 'shield.round';
  const SHIELD_NAME = 'Round Shield';

  it('renders the panel when the same weapon is carried twice', async () => {
    // A magus, as reported — the type that renders the panel's widest surface
    // (Lab/Casting Totals, Penetration, Magic Resistance) above the combat table.
    // Freshly created, so the row counts below are only what this spec carries.
    await startCharacter('magus');

    // The language is app-wide state, survives a new character and since C4 also
    // survives a restart, so pin it: the English weapon names below are asserted
    // exactly.
    await setLanguage('en');

    // Carry the same Long Sword twice. Nothing dedups this, and it is what a real
    // character does: several instances of one catalogue weapon.
    await $(EQUIPMENT_TAB).click();
    const addWeapon = await $(`[data-testid="add-${WEAPON}"]`);
    await addWeapon.waitForExist({ timeout: 10000 });
    await addWeapon.click();
    await addWeapon.click();

    // Two selected slots, both wielded (adding wields by default, K5), so the
    // engine emits two combat lines carrying the same `weapon` id.
    const secondSlot = await $('[data-testid="equipment-name-1"]');
    await secondSlot.waitForExist({ timeout: 10000 });
    expect(clean(await $('[data-testid="equipment-name-0"]').getText())).toBe(WEAPON_NAME);
    expect(clean(await secondSlot.getText())).toBe(WEAPON_NAME);
    expect(await $('[data-testid="equipment-loadout-0"]').getValue()).toBe('wielded');
    expect(await $('[data-testid="equipment-loadout-1"]').getValue()).toBe('wielded');

    // THE REGRESSION: the Totals panel must actually mount. A duplicate-key throw
    // leaves this waiting forever.
    await $(TOTALS_TAB).click();
    const combat = await $(COMBAT);
    await combat.waitForExist({ timeout: 15000 });

    // Both slots get their own row — the duplicate ids are two real lines, not one.
    await browser.waitUntil(async () => (await $$(COMBAT_ROWS)).length === 2, {
      timeout: 10000,
      timeoutMsg: 'the combat table should hold one row per equipped slot',
    });
    const rowNames = [];
    for (const cell of await $$(`${COMBAT_ROWS} th`)) {
      rowNames.push(clean(await cell.getText()));
    }
    expect(rowNames).toEqual([WEAPON_NAME, WEAPON_NAME]);

    // And the switch really happened: the Equipment tab's content is gone. The
    // failure mode was not a blank panel but the PREVIOUS tab left on screen,
    // because the aborted render never tore its DOM down.
    expect(await $(EQUIPMENT_LIST).isExisting()).toBe(false);
    await expect($('[data-testid="derived-panel"]')).toExist();

    // Equipping a shield doubles every one-handed weapon's lines: with the shield,
    // then bare. That makes `line.weapon` duplicate for a THIRD reason, so this is
    // also the only guard left against the `each_key_duplicate` regression above.
    await $(EQUIPMENT_TAB).click();
    const addShield = await $(`[data-testid="add-${SHIELD}"]`);
    await addShield.waitForExist({ timeout: 10000 });
    await addShield.click();

    await $(TOTALS_TAB).click();
    await $(COMBAT).waitForExist({ timeout: 15000 });
    await browser.waitUntil(async () => (await $$(COMBAT_ROWS)).length === 4, {
      timeout: 10000,
      timeoutMsg: 'each weapon should gain a with-shield row beside its bare one',
    });
    const shieldedRowNames = [];
    for (const cell of await $$(`${COMBAT_ROWS} th`)) {
      shieldedRowNames.push(clean(await cell.getText()));
    }
    const WITH_SHIELD = `${WEAPON_NAME} & ${SHIELD_NAME}`;
    expect(shieldedRowNames).toEqual([WITH_SHIELD, WEAPON_NAME, WITH_SHIELD, WEAPON_NAME]);

    // The panel survived the new duplication source.
    await expect($('[data-testid="derived-panel"]')).toExist();
  });
});

// End-to-end: a repeatable parameterized virtue (Great Characteristic) can be
// added several times, each instance keeps its own chosen target, and once all
// targets are set the "missing parameter" error clears. Reproduces the report
// that the error stuck around even after every instance had a Characteristic.
describe('repeated parameterized virtues', () => {
  const MISSING_PARAM = 'missing the parameter';
  // Every wait below uses the shared SETTLE_TIMEOUT (helpers.js). The two finding
  // waits used to allow 5s for a debounced validate (`VALIDATE_DEBOUNCE_MS`) plus
  // three parallel IPC round trips, which is tighter than the row waits they follow.
  const paramSelector = (i) =>
    `[data-testid="param-virtue.great_characteristic-characteristic-${i}"]`;

  async function errorTexts() {
    const errors = await $$('[data-severity="error"]');
    const texts = [];
    for (let i = 0; i < errors.length; i++) {
      texts.push(await errors[i].getText());
    }
    return texts;
  }

  const waitForVirtuePoints = (points) => waitForBalancePoints('virtues', points);

  it('keeps a distinct target per Great Characteristic instance', async () => {
    // The instance indices below are the selections-array positions, so this needs
    // a character carrying no Great Characteristic yet.
    await startCharacter('companion');

    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();

    const add = await $('[data-testid="add-virtue.great_characteristic"]');
    await add.waitForExist({ timeout: 10000 });

    // Add six instances; the Add button must stay enabled across repeats.
    //
    // ONE CLICK, THEN ITS SETTLED VALIDATION, before the next click. These used
    // to be six back-to-back clicks, and one of them now and then never reached
    // the button: instance 5 (or, with the clicks spaced, instance 1) was "still
    // not existing" after 20s. Caught with an in-page event probe: the lost
    // click's mousedown landed on `li.issue.error` at the very point the Add
    // button's centre had been. The issues panel is the app-wide footer
    // (`App.svelte`, `.validation-bar`), and it grows as an add's debounced
    // validation lands with new findings, squeezing the tab area from below —
    // and this row sits at the bottom edge of the Available list. WebDriver
    // checks for an obscuring element BEFORE it dispatches, so a validation that
    // lands in between moves the footer under the pointer and the click goes to
    // it with no error at all.
    //
    // So each click waits for the add before it to be fully validated. The
    // balance bar reads `effective.virtue_points`, which `revalidate` publishes in
    // the same guarded write as the issue list, so it showing `n` Virtue points
    // means the footer already has its height for `n` instances. A Minor Virtue
    // is one point and a fresh companion holds none, so `n` is the instance
    // count. A zero budget means the first engine pass has not landed yet.
    const targets = ['int', 'per', 'str', 'sta', 'pre', 'com'];
    await waitForVirtuePoints(0);
    for (let i = 0; i < targets.length; i++) {
      await add.click();
      await $(paramSelector(i)).waitForExist({
        timeout: SETTLE_TIMEOUT,
        timeoutMsg: `Add click ${i + 1} of ${targets.length} never rendered instance ${i}`,
      });
      await waitForVirtuePoints(i + 1);
    }

    // While unfilled, a missing-parameter error must be reported (and the panel
    // must keep rendering — six identical issues must not crash the keyed list).
    await browser.waitUntil(
      async () => (await errorTexts()).some((t) => t.includes(MISSING_PARAM)),
      {
        timeout: SETTLE_TIMEOUT,
        timeoutMsg: 'expected a missing-parameter error while instances are unfilled',
      },
    );

    // Each instance gets its own characteristic (all distinct, so under the cap).
    // Every row already exists (the add loop waited for it); confirm the control
    // is enabled, not merely present, before selecting.
    for (let i = 0; i < targets.length; i++) {
      const param = await $(paramSelector(i));
      await param.waitForExist({ timeout: SETTLE_TIMEOUT });
      await param.waitForEnabled({ timeout: SETTLE_TIMEOUT });
      await param.selectByAttribute('value', `characteristic.${targets[i]}`);
    }

    // The selected value must persist in the DOM for every instance.
    for (let i = 0; i < targets.length; i++) {
      const param = await $(paramSelector(i));
      expect(await param.getValue()).toBe(`characteristic.${targets[i]}`);
    }

    // Once every instance has a target, the missing-parameter error must clear.
    await browser.waitUntil(
      async () => !(await errorTexts()).some((t) => t.includes(MISSING_PARAM)),
      {
        timeout: SETTLE_TIMEOUT,
        timeoutMsg: 'missing-parameter error did not clear after all targets were set',
      },
    );
  });
});

// Mirror of finding 34's spec above, for the OTHER cap: max_total bounds copies
// of an item TOTAL across every distinct parameter target (Puissant Art, capped
// at two — rules/core/virtues_flaws.json), distinct from max_per_target (one
// copy per identical target, which virtue.great_characteristic above tests and
// which carries no max_total of its own — its Add button must stay enabled
// indefinitely, unaffected by this cap).
//
// A magus, not a companion: Puissant Art is Hermetic-categoried, and the
// companion profile forbids that category (rules/core/character_types.json) —
// the Available row would still render (categories are not filtered there),
// but taking it would raise an unrelated category_not_permitted finding. Magus
// permits Hermetic, so this stays a clean test of the max_total cap alone.
describe('max_total copy cap on the Available list', () => {
  it('greys out Puissant Art once both of its two total copies are taken', async () => {
    await startCharacter('magus');

    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();

    const add = await $('[data-testid="add-virtue.puissant_art"]');
    await add.waitForExist({ timeout: 10000 });

    // The param selects are named `param-virtue.puissant_art-art-{index}`, where
    // {index} is the position in entity.selections — NOT a per-virtue counter. A
    // magus's mandatory free traits (Hermetic Magus + The Gift, seeded on
    // creation from the type profile — see the `hermetic arts` describe above)
    // already occupy the first slots, so the two Puissant Art instances land at
    // whatever index follows them, never 0 and 1. Match by prefix and pick by how
    // many exist so far, rather than hardcoding an offset.
    const puissantArtParams = () => $$('[data-testid^="param-virtue.puissant_art-art-"]');

    // First copy, targeting Ignem — one of two, so the row must stay takeable.
    await add.click();
    await browser.waitUntil(async () => (await puissantArtParams()).length === 1, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'expected one Puissant Art param select after the first Add click',
    });
    const firstArt = (await puissantArtParams())[0];
    await firstArt.selectByAttribute('value', 'art.ignem');
    expect(await isRowBlocked(add)).toBe(false);

    // Second copy, targeting Perdo — a DIFFERENT target, so max_per_target (one
    // copy per identical target) never fires; only max_total (two total) does.
    await add.click();
    await browser.waitUntil(async () => (await puissantArtParams()).length === 2, {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'expected two Puissant Art param selects after the second Add click',
    });
    const secondArt = (await puissantArtParams())[1];
    await secondArt.selectByAttribute('value', 'art.perdo');

    await browser.waitUntil(async () => isRowBlocked(add), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'Puissant Art did not grey out once both of its two total copies were taken',
    });
  });
});

// End-to-end: Mythic Companion types (mythic-companion-only). Creating a
// character of that type surfaces a "Type" tab (gated on the profile's
// `has_mythic_type` flag); picking a type auto-grants its free status + Minor
// Virtue (read-only) and seeds its required V/F package as bought selections,
// which spend the profile's own 20 V / 10 F budget — no type raises it. A
// required Flaw is swappable for a substitute.
describe('mythic companion types', () => {
  const MYTHIC_TAB = '[data-testid="tab-mythic_type"]';
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const MYTHIC_SELECT = '[data-testid="mythic-type-select"]';
  const VIRTUE_BUDGET = '[data-testid="balance-virtues"]';
  const FLAW_BUDGET = '[data-testid="balance-flaws"]';

  async function selectMythicType(value) {
    await $(MYTHIC_TAB).click();
    const select = await $(MYTHIC_SELECT);
    await select.waitForExist({ timeout: 10000 });
    await select.selectByAttribute('value', value);
  }

  it('shows the Type tab only for a mythic companion', async () => {
    await startCharacter('companion');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await expect($(MYTHIC_TAB)).not.toExist();

    await startCharacter('mythic_companion');
    await $(MYTHIC_TAB).waitForExist({ timeout: 10000 });
  });

  it('grants the free status + Minor Virtue and leaves the budget at the profile 20/10', async () => {
    await startCharacter('mythic_companion');
    await selectMythicType('mythic_type.devil_child');

    // On the Type tab: the free status Virtue is a read-only grant row, and the
    // free Minor is a defaulted choice (Demonic Might/Powers).
    await $('[data-testid="mythic-granted-virtue.devil_child"]').waitForExist({ timeout: 10000 });
    await expect($('[data-testid="mythic-choice-devil_child_free_minor"]')).toExist();

    // The balance bar lives on the Virtues & Flaws tab. Devil Child is the type
    // that used to claim a bonus; the bar must read the mythic companion
    // profile's own 20 V / 10 F, which no type changes (D32). The free status
    // Virtue and free Minor above are grants, so neither is counted against it.
    await $(VF_TAB).click();
    await browser.waitUntil(async () => clean(await $(VIRTUE_BUDGET).getText()).includes('/ 20'), {
      timeout: 5000,
      timeoutMsg: 'virtue budget is not the profile ceiling of 20',
    });
    expect(clean(await $(FLAW_BUDGET).getText())).toContain('/ 10');
  });

  it('seeds the required package and offers a required-Flaw substitute', async () => {
    await startCharacter('mythic_companion');
    await selectMythicType('mythic_type.devil_child');

    // The required Flaw swap dropdown is on the Type tab, with >1 substitute.
    const swap = await $('[data-testid="mythic-required-flaw-flaw.tragic_life"]');
    await swap.waitForExist({ timeout: 10000 });
    expect((await swap.$$('option')).length).toBeGreaterThan(1);

    // The required package is seeded as bought selections, shown on the V&F tab
    // (Demonic Blood in the Virtues list; Tragic Life in the Flaws list).
    await $(VF_TAB).click();
    await $('[data-testid="selection-list-virtue"]').waitForExist({ timeout: 10000 });
    expect(await $('[data-testid="selection-list-virtue"]').getText()).toContain('Demonic Blood');
    expect(await $('[data-testid="selection-list-flaw"]').getText()).toContain('Tragic Life');
  });
});

// End-to-end: a character's type is fixed at creation, and the type it was
// created as is what drives its profile — budget and category rules alike.
// Verifies the shipped types (grog, companion, mythic companion, magus) produce
// different validation, and that the editor offers no way to change the type
// afterwards.
describe('character types', () => {
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const VIRTUE_BUDGET = '[data-testid="balance-virtues"]';
  const FLAW_BUDGET = '[data-testid="balance-flaws"]';
  const CHARACTER_TYPE = '[data-testid="character-type"]';
  // `flaw.blatant_gift` is in the Hermetic category: forbidden for a companion,
  // permitted for a magus. A deterministic way to observe category rules.
  const HERMETIC_FLAW = '[data-testid="add-flaw.blatant_gift"]';

  async function budget(selector) {
    return clean(await $(selector).getText());
  }

  async function codes() {
    const items = await $$('[data-testid="issue-list"] li');
    const result = [];
    for (let i = 0; i < items.length; i++) {
      result.push(await items[i].getAttribute('data-code'));
    }
    return result;
  }

  /** Create a character of `type` and open its Virtues & Flaws tab (the balance bar). */
  async function startOnVfTab(type) {
    await startCharacter(type);
    const vfTab = await $(VF_TAB);
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();
    await $(VIRTUE_BUDGET).waitForExist({ timeout: 10000 });
  }

  it('gives each created type its own profile budget in the balance bar', async () => {
    // Companion: 10 / 10.
    await startOnVfTab('companion');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 10'), {
      timeout: 5000,
      timeoutMsg: 'companion virtue budget not 10',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 10');

    // Grog: 3 / 3.
    await startOnVfTab('grog');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 3'), {
      timeout: 5000,
      timeoutMsg: 'grog virtue budget not 3',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 3');

    // Mythic Companion: 20 virtue points (2:1 funding) / 10 flaw points.
    await startOnVfTab('mythic_companion');
    await browser.waitUntil(async () => (await budget(VIRTUE_BUDGET)).includes('/ 20'), {
      timeout: 5000,
      timeoutMsg: 'mythic companion virtue budget not 20',
    });
    expect(await budget(FLAW_BUDGET)).toContain('/ 10');
  });

  it('applies the created type category rules to the same selection', async () => {
    // As a companion, the Hermetic flaw is a forbidden-category error.
    await startOnVfTab('companion');
    const addHermetic = await $(HERMETIC_FLAW);
    await addHermetic.waitForExist({ timeout: 10000 });
    await addHermetic.click();
    await browser.waitUntil(async () => (await codes()).includes('forbidden_category'), {
      timeout: 5000,
      timeoutMsg: 'companion should forbid the Hermetic flaw',
    });

    // As a magus, the Hermetic category is permitted: the same pick raises no
    // forbidden-category error, and a magus-specific issue (no House chosen yet →
    // `house_unset`) appears — a positive signal that this really is the magus
    // profile, not just an absence. (The required Hermetic Magus status is seeded
    // at creation, so `missing_required_trait` does not fire either.)
    await startOnVfTab('magus');
    const addAsMagus = await $(HERMETIC_FLAW);
    await addAsMagus.waitForExist({ timeout: 10000 });
    await addAsMagus.click();
    await browser.waitUntil(
      async () => {
        const seen = await codes();
        return !seen.includes('forbidden_category') && seen.includes('house_unset');
      },
      { timeout: 5000, timeoutMsg: 'the magus profile rules did not apply to the new character' },
    );
  });

  it('shows the type as a read-only label, with no selector to change it', async () => {
    await startCharacter('grog');

    // The banner names the type through its Fluent key, never the raw slug...
    const label = await $(CHARACTER_TYPE);
    await label.waitForExist({ timeout: 10000 });
    const text = clean(await label.getText());
    expect(text).toContain('Grog');
    expect(text).not.toContain('type-grog');

    // ...and it is a label, not a control: the header selector this file used to
    // drive is gone, and nothing editable replaced it.
    expect(await $('[data-testid="type-select"]').isExisting()).toBe(false);
    expect(
      (await $$(`${CHARACTER_TYPE} select, ${CHARACTER_TYPE} input, ${CHARACTER_TYPE} button`))
        .length,
    ).toBe(0);
  });
});
