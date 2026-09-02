// End-to-end: Hermetic Houses (magus-only). Selecting a House auto-grants its
// free Virtue — free of the point budget — Mystery Houses seed a Supernatural
// Ability at an effective floor of 1, a `choice` grant offers its options and an
// unresolved pick raises a localized issue, and an `open` grant is additive (it
// never charges the bought budget). All against the real binary.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.
//
// Coverage boundary (mirrors validation-errors.e2e.js): two Phase-4 facts are
// NOT click-reachable in the shipped `rules/core/` seed, so they are asserted by
// the Rust unit/integration tests (plan step 10), not here:
//   * The `too_many_major_hermetic_virtues` cap TRIP — the seed has exactly one
//     Major Hermetic Virtue (`virtue.gentle_gift`, non-repeatable), so a second
//     bought one cannot be reached through clicks. What IS reachable and checked
//     below: a House's granted (Minor) Virtue never contributes to the cap.
//   * Ex Miscellanea's `ex_misc_major_virtue` open pick — the seed has no Major
//     non-Hermetic Virtue, so that picker is empty. The reachable Ex-Misc fact
//     checked below is that an `open` grant is additive to the bought budget.

import { $, expect, browser } from '@wdio/globals';

import { clean, startCharacter } from '../helpers.js';

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
// Second Sight grant in vf-effects.e2e.js).
async function addAbility(id) {
  await $(ABILITIES_TAB).click();
  const add = await $(`[data-testid="add-${id}"]`);
  await add.waitForExist({ timeout: 10000 });
  await add.click();
}

describe('hermetic houses', () => {
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
