// End-to-end: the talisman is an ITEM, not a bare attunement list. Driven through
// the real binary: identity + attunement + instilled effect are all enterable, the
// capacity read-out equals highest Technique + highest Form, and the item-level
// budget is UNMOVED by an instilled effect (the budget non-goal, visible in the UI).
// Then the schema-14 shape on disk, and the legacy-save migration through the
// shipped binary's own Open path.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds
// `target/release/arm-app`.

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { startCharacter } from '../helpers.js';

const e2eFile = path.resolve(os.tmpdir(), 'arm-e2e-character.json');

const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const ARTS_TAB = '[data-testid="tab-arts"]';
const POSSESSIONS_TAB = '[data-testid="tab-possessions"]';

const CAPACITY = '[data-testid="talisman-capacity"]';
const CAPACITY_NOTE = '[data-testid="talisman-capacity-note"]';
const ITEM_LEVELS = '[data-testid="item-level-used"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

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

describe('talisman', () => {
  it('enters an item, derives its capacity, and leaves the item-level budget alone', async () => {
    await startCharacter('magus');
    await $(VF_TAB).waitForExist({ timeout: 30000 });

    // Creo 5 / Perdo 2 (Techniques) and Corpus 4 / Ignem 3 (Forms) from the shared
    // Art XP pool. Highest Technique 5 + highest Form 4 = 9 pawns; the lower Arts
    // exist so "highest" is actually exercised rather than "the only one bought".
    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('200');
    const inc = async (art, times) => {
      const button = await $(`[data-testid="art-inc-${art}"]`);
      await button.waitForExist({ timeout: 5000 });
      for (let i = 0; i < times; i++) await button.click();
    };
    await inc('art.creo', 5);
    await inc('art.perdo', 2);
    await inc('art.corpus', 4);
    await inc('art.ignem', 3);

    // Magic Items tab: no talisman yet.
    await $(POSSESSIONS_TAB).click();
    await $('[data-testid="talisman-empty-item"]').waitForExist({ timeout: 10000 });
    // The item-level budget starts at 0 used — the baseline for the non-goal check.
    await waitForText(ITEM_LEVELS, '0');
    const itemLevelsBefore = clean(await $(ITEM_LEVELS).getText());

    await $('[data-testid="talisman-add"]').click();

    // Identity (shape and material).
    const description = await $('[data-testid="talisman-description"]');
    await description.waitForExist({ timeout: 10000 });
    await description.setValue('An ash staff shod with silver');

    // The capacity read-out: highest Technique 5 + highest Form 4 = 9 pawns, with
    // both contributing scores spelled out in the note.
    await waitForText(CAPACITY, '9');
    await waitForText(CAPACITY_NOTE, '5', '4');

    // An attunement (descriptor + shape bonus).
    await $('[data-testid="talisman-attunement-add"]').click();
    const attunement = await $('[data-testid="talisman-desc-0"]');
    await attunement.waitForExist({ timeout: 5000 });
    await attunement.setValue('Controlling things at a distance');
    await $('[data-testid="talisman-bonus-0"]').setValue('4');

    // An instilled effect at level 15.
    await $('[data-testid="talisman-effect-add"]').click();
    const effectName = await $('[data-testid="talisman-effect-name-0"]');
    await effectName.waitForExist({ timeout: 5000 });
    await effectName.setValue('Wielding the Invisible Sling');
    await $('[data-testid="talisman-effect-level-0"]').setValue('15');

    // THE NON-GOAL, VISIBLE IN THE UI: 15 levels instilled in the talisman must not
    // touch the item-level budget, which belongs to the Redcap-only Virtues and can
    // never fund a magus's talisman. The read-out is unchanged.
    await waitForText(CAPACITY, '9');
    expect(clean(await $(ITEM_LEVELS).getText())).toBe(itemLevelsBefore);

    // Raising Corpus raises the capacity: highest Form becomes 5 → 10 pawns.
    await $(ARTS_TAB).click();
    await $('[data-testid="art-inc-art.corpus"]').click();
    await $(POSSESSIONS_TAB).click();
    await waitForText(CAPACITY, '10');

    // The schema-14 shape reaches disk: a nested talisman, no legacy key.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(15);
    expect(saved.talisman_attunements).toBeUndefined();
    expect(saved.talisman.description).toBe('An ash staff shod with silver');
    expect(saved.talisman.attunements).toEqual([
      { description: 'Controlling things at a distance', bonus: 4 },
    ]);
    expect(saved.talisman.effects).toEqual([{ name: 'Wielding the Invisible Sling', level: 15 }]);
  });

  it('migrates a legacy schema-13 save through the shipped Open path', async () => {
    // The engine-boundary half of this proof lives in `arm-app`'s command tests;
    // this is the UI half — a real Open of a real legacy file in the real binary.
    fs.writeFileSync(
      e2eFile,
      JSON.stringify({
        schema_version: 13,
        ruleset: { id: 'arm5-core', version: '2024.1' },
        entity_kind: 'character',
        type_id: 'magus',
        selections: [{ ref: 'virtue.the_gift' }, { ref: 'virtue.hermetic_magus' }],
        talisman_attunements: [{ description: 'Projecting bolts and missiles', bonus: 3 }],
      }),
    );

    await $('[data-testid="open-button"]').click();
    // The document is dirty from the previous test, so confirm discarding first.
    const discard = await $('[data-testid="discard-confirm"]');
    if (await discard.isExisting()) await discard.click();

    // The legacy attunement now lives under the talisman, so the item's own controls
    // are present and the attunement row carries the migrated values.
    await $(POSSESSIONS_TAB).click();
    const attunement = await $('[data-testid="talisman-desc-0"]');
    await attunement.waitForExist({ timeout: 15000 });
    expect(await attunement.getValue()).toBe('Projecting bolts and missiles');
    expect(await $('[data-testid="talisman-bonus-0"]').getValue()).toBe('3');
    // The item exists, so its identity field renders — nothing was invented into it.
    expect(await $('[data-testid="talisman-description"]').getValue()).toBe('');
  });

  // The remove controls are only ever CLICKED here: the panel's unit tests render to
  // a string (SSR), so no handler runs there. A row-remove wired to the wrong list or
  // off by one would delete the player's other row — silent data loss — and every
  // other gate would stay green. Each removal is proved by which row SURVIVES.
  it('removes the clicked row, and then the whole talisman', async () => {
    await $(POSSESSIONS_TAB).click();

    // The migrated attunement plus a second one, then remove the FIRST: the second
    // must survive and slide up to index 0.
    await $('[data-testid="talisman-attunement-add"]').click();
    const second = await $('[data-testid="talisman-desc-1"]');
    await second.waitForExist({ timeout: 10000 });
    await second.setValue('Warding against wood');
    await $('[data-testid="talisman-remove-0"]').click();
    await browser.waitUntil(
      async () =>
        (await $('[data-testid="talisman-desc-0"]').getValue()) === 'Warding against wood',
      {
        timeout: 10000,
        timeoutMsg: 'removing attunement row 0 should leave the second attunement behind',
      },
    );
    expect(await $('[data-testid="talisman-desc-1"]').isExisting()).toBe(false);

    // Same for the instilled effects, which are a separate list on the same item.
    await $('[data-testid="talisman-effect-add"]').click();
    const firstEffect = await $('[data-testid="talisman-effect-name-0"]');
    await firstEffect.waitForExist({ timeout: 10000 });
    await firstEffect.setValue('Lamp Without Flame');
    await $('[data-testid="talisman-effect-add"]').click();
    const secondEffect = await $('[data-testid="talisman-effect-name-1"]');
    await secondEffect.waitForExist({ timeout: 10000 });
    await secondEffect.setValue('Endurance of the Berserkers');
    await $('[data-testid="talisman-effect-remove-0"]').click();
    await browser.waitUntil(
      async () =>
        (await $('[data-testid="talisman-effect-name-0"]').getValue()) ===
        'Endurance of the Berserkers',
      {
        timeout: 10000,
        timeoutMsg: 'removing effect row 0 should leave the second effect behind',
      },
    );
    expect(await $('[data-testid="talisman-effect-name-1"]').isExisting()).toBe(false);

    // And the item itself: the empty state comes back.
    await $('[data-testid="talisman-remove-item"]').click();
    await $('[data-testid="talisman-empty-item"]').waitForExist({ timeout: 10000 });
    expect(await $('[data-testid="talisman-description"]').isExisting()).toBe(false);
  });
});
