// End-to-end: the Markdown character-sheet export (M5.6). Builds a magus in the
// real binary, clicks Export, and reads the `.md` the app wrote off disk. This is
// the only layer that proves the whole chain — the frontend's Fluent label map
// travelling over real IPC, the engine's formatter, and the file write — so it
// asserts entered values, engine-computed read-outs, the deliberately excluded
// sections, both languages, and that an export is not a save.
//
// The native save dialog can't be driven by WebDriver, so the app writes to the
// ARM_E2E_EXPORT_FILE seam (see crates/arm-app/src/commands.rs), a fixed `.md`
// path separate from the JSON save file the other specs round-trip.

import { $, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { isRowBlocked, startCharacter } from '../helpers.js';
import { e2eExportFile as exportFile, e2eFile as saveFile } from '../wdio.conf.js';

const LANG_SELECT = '[data-testid="language-select"]';
const STATUS = '[data-testid="doc-status"]';
const NAME = 'Marcus of Bonisagus';

// The last section a character with no aging annotations emits, so its presence
// means the write has fully landed rather than merely started.
const EN_LAST_SECTION = '## Magic Items';
const DE_LAST_SECTION = '## Magische Gegenstände';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

async function setLang(value) {
  await $(LANG_SELECT).selectByAttribute('value', value);
  await browser.waitUntil(async () => (await $(LANG_SELECT).getValue()) === value, {
    timeout: 5000,
    timeoutMsg: `language should switch to ${value}`,
  });
}

async function clickTab(id) {
  const tab = await $(`[data-testid="tab-${id}"]`);
  await tab.waitForExist({ timeout: 10000 });
  await tab.click();
}

/**
 * Click Export and return what landed on disk. The file is deleted first, so
 * "was written" can never pass on a stale export from an earlier assertion.
 */
async function exportSheet(marker) {
  if (fs.existsSync(exportFile)) fs.unlinkSync(exportFile);
  await $('[data-testid="export-button"]').click();
  await browser.waitUntil(
    () => fs.existsSync(exportFile) && fs.readFileSync(exportFile, 'utf-8').includes(marker),
    { timeout: 15000, timeoutMsg: `export did not write a document containing "${marker}"` },
  );
  return fs.readFileSync(exportFile, 'utf-8');
}

/** The body of one `## ` section, so a shared label can be asserted in context. */
function section(md, heading) {
  const start = md.indexOf(`## ${heading}\n`);
  if (start < 0) throw new Error(`missing section: ${heading}`);
  const rest = md.slice(start + heading.length + 4);
  const end = rest.indexOf('\n## ');
  return end < 0 ? rest : rest.slice(0, end);
}

describe('markdown export', () => {
  it('builds a magus worth exporting', async () => {
    // A magus of this spec's own, from the startup screen: specs share one app
    // instance, so nothing here may be inherited from the previous file. The
    // language is app-wide state and does carry over, so pin it to English.
    await startCharacter('magus');
    await setLang('en');

    await $('[data-testid="identity-name"]').setValue(NAME);

    // Intelligence +2 and Stamina +2 — the latter is what gives Soak a non-zero
    // total, so the computed Soak section is emitted at all.
    await clickTab('characteristics');
    const intInc = await $('[data-testid="char-inc-int"]');
    await intInc.waitForExist({ timeout: 10000 });
    await intInc.click();
    await intInc.click();
    const staInc = await $('[data-testid="char-inc-sta"]');
    await staInc.click();
    await staInc.click();
    expect(await $('[data-testid="char-value-sta"]').getText()).toBe('+2');

    // A balanced minor Virtue / minor Flaw pair.
    await clickTab('virtues_flaws');
    const addVirtue = await $('[data-testid="add-virtue.keen_vision"]');
    await addVirtue.waitForExist({ timeout: 10000 });
    await addVirtue.click();
    await $('[data-testid="add-flaw.poor_student"]').click();

    // Single Weapon 2 with a specialty — also the Ability the equipped Long Sword
    // rolls, so it shows up again in the computed Combat row.
    await clickTab('abilities');
    const pool = await $('[data-testid="xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('200');
    await $('[data-testid="add-ability.single_weapon"]').click();
    const abilityInc = await $('[data-testid="ability-inc-ability.single_weapon-0"]');
    await abilityInc.waitForExist({ timeout: 5000 });
    await abilityInc.click();
    await abilityInc.click();
    await $('[data-testid="ability-specialty-ability.single_weapon-0"]').setValue('longsword');

    // Creo 5, and Ignem 12 so the Creo Ignem per-spell cap (Cr + Ig + Int +
    // Magic Theory + 3) clears Pilum of Fire's level 20 and its add control is
    // not greyed.
    await clickTab('arts');
    const creoInc = await $('[data-testid="art-inc-art.creo"]');
    await creoInc.waitForExist({ timeout: 10000 });
    for (let i = 0; i < 5; i++) await creoInc.click();
    const ignemInc = await $('[data-testid="art-inc-art.ignem"]');
    for (let i = 0; i < 12; i++) await ignemInc.click();

    await clickTab('spells');
    const addSpell = await $('[data-testid="add-spell.pilum_of_fire"]');
    await addSpell.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => !(await isRowBlocked(addSpell)), {
      timeout: 5000,
      timeoutMsg: 'Creo 5 / Ignem 12 should clear the cap on Pilum of Fire',
    });
    await addSpell.click();

    // A carried weapon. Picking one equips it, which is what produces a Combat
    // line, so this only confirms the state rather than toggling it.
    await clickTab('equipment');
    const addWeapon = await $('[data-testid="add-weapon.sword_long"]');
    await addWeapon.waitForExist({ timeout: 10000 });
    await addWeapon.click();
    const equipped = await $('[data-testid="equipment-equipped-0"]');
    await equipped.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => await equipped.isSelected(), {
      timeout: 5000,
      timeoutMsg: 'a newly carried weapon should be equipped',
    });

    // One magic possession: an assumed aura and an enchanted device.
    await clickTab('possessions');
    const aura = await $('[data-testid="aura-input"]');
    await aura.waitForExist({ timeout: 10000 });
    await aura.setValue('3');
    await $('[data-testid="device-add"]').click();
    const deviceName = await $('[data-testid="device-name-0"]');
    await deviceName.waitForExist({ timeout: 5000 });
    await deviceName.setValue('Ring of the Warding Flame');
    await $('[data-testid="device-level-0"]').setValue('20');
  });

  it('writes the entered values and the engine-computed read-outs', async () => {
    const md = await exportSheet(EN_LAST_SECTION);

    // The H1 is the character's name; the subtitle carries the localized type.
    expect(md.startsWith(`# ${NAME}\n`)).toBe(true);
    expect(md).toContain('*Magus');

    // English section headings, all resolved from the Fluent keys.
    for (const heading of [
      '## Characteristics',
      '## Virtues & Flaws',
      '## Abilities',
      '## Arts',
      '## Spells',
      '## Equipment',
      '## Combat',
      '## Soak',
      EN_LAST_SECTION,
    ]) {
      expect(md).toContain(heading);
    }

    // Entered values: the Ability score with its specialty, the Art, and the spell
    // with its short Art-and-level code (Creo + Ignem + level 20).
    expect(md).toMatch(/\| Single Weapon \| longsword \| 2 \|/);
    expect(md).toMatch(/\| Creo \| 5 \|/);
    expect(md).toMatch(/\| Pilum of Fire \| CrIg20 \|/);
    // Every Art is on the sheet, not only the two that were scored.
    expect(md).toMatch(/\| Terram \| 0 \|/);
    expect(section(md, 'Magic Items')).toContain('Ring of the Warding Flame');
    // A Virtue row carries its type (the item's category) beside its magnitude.
    expect(md).toMatch(/\| Keen Vision \| General \| Minor \|/);

    // Computed read-outs: the Combat line for the equipped weapon (Attack 10 =
    // Dex 0 + Single Weapon 2 + specialty 1 + weapon 4 ... engine-owned, so only
    // the row's identity is pinned here) and the Soak total from Stamina +2.
    expect(section(md, 'Combat')).toMatch(/\| Long Sword \| Single Weapon \|/);
    const soak = section(md, 'Soak');
    expect(soak).toContain('- **Stamina**: +2');
    expect(soak).toContain('- **Total**: +2');

    // Excluded by design: the sheet carries no Lab, Casting or Penetration totals.
    expect(md).not.toContain('Lab Totals');
    expect(md).not.toContain('Casting Totals');
    expect(md).not.toContain('Penetration');

    // No raw slug may reach the document: every label the engine printed came from
    // the frontend's Fluent bundle, so a key it failed to send would show here.
    for (const slug of [
      'type-magus',
      'param-label-',
      'derived-section-',
      'export-col-',
      'magnitude-',
      'category-',
      'characteristic-int',
      'identity-name',
    ]) {
      expect(md).not.toContain(slug);
    }
  });

  it('writes German headings when the UI is German, and overwrites cleanly', async () => {
    await setLang('de');
    const de = await exportSheet(DE_LAST_SECTION);

    for (const heading of [
      '## Eigenschaften',
      '## Tugenden & Fehler',
      '## Fertigkeiten',
      '## Künste',
      '## Zauber',
      '## Ausrüstung',
      '## Kampf',
      '## Absorption',
      DE_LAST_SECTION,
    ]) {
      expect(de).toContain(heading);
    }
    expect(de).not.toContain('## Characteristics');

    // Restore English before the spec ends: specs share one app instance and run
    // in filename order, so a German session would leak into every later spec.
    await setLang('en');

    // Exporting again over the existing file replaces it rather than appending —
    // one H1 and one Soak section, not two.
    await $('[data-testid="export-button"]').click();
    await browser.waitUntil(() => fs.readFileSync(exportFile, 'utf-8').includes(EN_LAST_SECTION), {
      timeout: 15000,
      timeoutMsg: 'the second export did not replace the German document',
    });
    const md = fs.readFileSync(exportFile, 'utf-8');
    expect(md.match(/^# /gm).length).toBe(1);
    expect(md.match(/^## Soak$/gm).length).toBe(1);
  });

  it('does not mark the document saved, and leaves the current file alone', async () => {
    // Save first, so the document has a known current file and a clean baseline.
    if (fs.existsSync(saveFile)) fs.unlinkSync(saveFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(saveFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const status = await $(STATUS);
    await browser.waitUntil(
      async () => {
        const text = clean(await status.getText());
        return text.includes('arm-e2e-character.json') && !text.startsWith('*');
      },
      { timeout: 5000, timeoutMsg: 'the header should show the saved file with no dirty marker' },
    );

    // Edit again so the document is unmistakably dirty, then export.
    await clickTab('characteristics');
    await $('[data-testid="char-inc-int"]').click();
    await browser.waitUntil(async () => clean(await status.getText()).startsWith('*'), {
      timeout: 5000,
      timeoutMsg: 'editing should show the dirty marker',
    });
    const before = clean(await status.getText());

    await exportSheet(EN_LAST_SECTION);

    // The export changed nothing about the document's identity or dirtiness.
    expect(clean(await status.getText())).toBe(before);

    // And Save still writes the JSON to the file the export never touched.
    fs.unlinkSync(saveFile);
    await $('[data-testid="save-button"]').click();
    await browser.waitUntil(() => fs.existsSync(saveFile), {
      timeout: 10000,
      timeoutMsg: 'save after an export did not write the JSON file',
    });
    expect(JSON.parse(fs.readFileSync(saveFile, 'utf-8')).name).toBe(NAME);
  });
});
