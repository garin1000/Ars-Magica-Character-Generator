// End-to-end: save -> Open fidelity for choices no other spec carries through a
// file, plus the German rules text on a full sheet and its export.
//
// Every other round trip in the suite saves a character whose interesting state
// is Abilities, Arts or possessions. These four describes cover what none of
// them reopen:
//
//   1. D81's parameters — Incompatible Arts' four selects, a Magical Focus's
//      free-text focus and a spell marked within that focus — survive a save and
//      an Open, and are still LIVE after it (the barred cell, the focus-doubled
//      cap), not merely displayed;
//   2. a Mythic Companion's type, its choice grant and a swapped required Flaw
//      survive a save and an Open;
//   3. direct-unchecked entry (Silent mode) lets an illegal character be built,
//      saved and reopened intact, and Enforced reports it again afterwards;
//   4. the German locale renders a companion's rules text (Virtue, Flaw and
//      Ability names) on every tab with no raw id or Fluent key, and exports a
//      German sheet for a non-magus.
//
// Each round trip leaves through New and comes back through the startup screen's
// own Open button, so nothing on screen afterwards can be left over from the
// character that was saved. A saved document is clean, so that New must NOT ask
// to discard anything — asserted on the way, because a save that left the dirty
// flag set would make the unsaved-changes guard cry wolf on every quit.
//
// Session-scoped state: describe 3 changes the validation mode and describe 4
// the language; each restores the default in its own `after` hook (see
// `e2e/README.md`, "Spec ordering"). No describe here is terminal.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import {
  clean,
  isRowBlocked,
  runDocumentAction,
  setLanguage,
  setValidationMode,
  SETTLE_TIMEOUT,
  startCharacter,
  STEP_TIMEOUT,
  waitForBalancePoints,
  waitForIdle,
  waitUntilExplained,
} from '../helpers.js';
import { e2eExportFile, e2eFile } from '../wdio.conf.js';

const START_SCREEN = '[data-testid="start-screen"]';
const START_OPEN = '[data-testid="start-open"]';
const DISCARD_PROMPT = '[data-testid="discard-prompt"]';
const TAB_BAR = '[role="tablist"]';
const TAB_PANEL = '[role="tabpanel"]';
const NAME_INPUT = '[data-testid="identity-name"]';
const ISSUE_LIST = '[data-testid="issue-list"]';
const VF_TAB = '[data-testid="tab-virtues_flaws"]';
const ARTS_TAB = '[data-testid="tab-arts"]';
const SPELLS_TAB = '[data-testid="tab-spells"]';
const TOTALS_TAB = '[data-testid="tab-totals"]';
const ABILITIES_TAB = '[data-testid="tab-abilities"]';
const MYTHIC_TAB = '[data-testid="tab-mythic_type"]';

/** The finding codes the validation panel currently lists. */
async function issueCodes() {
  const items = await $$(`${ISSUE_LIST} li`);
  const codes = [];
  for (let i = 0; i < items.length; i++) {
    codes.push(await items[i].getAttribute('data-code'));
  }
  return codes;
}

/** Wait until the listed finding codes satisfy `predicate`, naming them on failure. */
async function waitForCodes(predicate, message) {
  let codes = [];
  await waitUntilExplained(
    async () => {
      codes = await issueCodes();
      return predicate(codes);
    },
    STEP_TIMEOUT,
    () => `${message}; showing [${codes.join(', ')}]`,
  );
  return codes;
}

/**
 * The whole text of one element, bidi-stripped. `textContent` rather than
 * `getText()`: the latter is empty for an `overflow: hidden` box and for anything
 * scrolled out of a scrolling list, and these lists scroll.
 */
async function textContentOf(selector) {
  const text = await browser.execute(
    (sel) => document.querySelector(sel)?.textContent ?? '',
    selector,
  );
  return clean(text);
}

async function clickTab(selector) {
  const tab = await $(selector);
  await tab.waitForExist({ timeout: STEP_TIMEOUT });
  await tab.click();
}

/** Wait for an input or select to read `value` back, i.e. for the edit to land. */
async function waitForValue(element, value, what) {
  await browser.waitUntil(async () => (await element.getValue()) === value, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `${what} should read '${value}'`,
  });
}

/**
 * Save through the ARM_E2E_FILE seam and return the parsed document. Deleted
 * first, so a stale file from an earlier test can never satisfy the wait, and
 * parsed inside the wait so a half-written file is retried rather than thrown on.
 */
async function saveAndRead() {
  if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
  await runDocumentAction('save');
  let saved;
  await browser.waitUntil(
    () => {
      if (!fs.existsSync(e2eFile)) return false;
      try {
        saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
        return true;
      } catch {
        return false;
      }
    },
    { timeout: STEP_TIMEOUT, timeoutMsg: 'save did not write a parseable file' },
  );
  // The file lands before the frontend's save() settles; clicking on before the
  // busy scrim lifts would be swallowed by the `inert` shell (helpers.js).
  await waitForIdle();
  return saved;
}

/**
 * Leave the just-saved document through New — which must go straight to the
 * startup screen, because a saved document has nothing to discard — and open the
 * file again through the startup screen's own Open button. Waits for the reopened
 * character's name, then for the window title to show it with no dirty marker:
 * an Open is a load baseline, so the document it produces is clean.
 */
async function reopenSavedFile(name) {
  await runDocumentAction('new');
  await browser.waitUntil(
    async () => (await $(START_SCREEN).isExisting()) || (await $(DISCARD_PROMPT).isExisting()),
    { timeout: STEP_TIMEOUT, timeoutMsg: 'New neither reached the startup screen nor prompted' },
  );
  expect(await $(DISCARD_PROMPT).isExisting()).toBe(false);

  const open = await $(START_OPEN);
  await open.waitForClickable({ timeout: STEP_TIMEOUT });
  await open.click();
  await $(TAB_BAR).waitForExist({ timeout: 15000 });
  await browser.waitUntil(async () => (await $(NAME_INPUT).getValue()) === name, {
    timeout: STEP_TIMEOUT,
    timeoutMsg: `Open should load the saved character '${name}'`,
  });
  await browser.waitUntil(
    async () => {
      const title = await browser.execute(() => document.title);
      return title.includes(name) && !title.startsWith('*');
    },
    { timeout: STEP_TIMEOUT, timeoutMsg: 'a freshly opened document should be clean' },
  );
}

/** The window title still carries no dirty marker. */
async function expectClean() {
  expect((await browser.execute(() => document.title)).startsWith('*')).toBe(false);
}

/** A selection's two unordered (technique, form) pairs, order-insensitive. */
function incompatiblePairs(params) {
  return [
    `${params.technique_1}+${params.form_1}`,
    `${params.technique_2}+${params.form_2}`,
  ].sort();
}

// ---------------------------------------------------------------------------
// 1. D81 parameters through a file.
//
// The Incompatible Arts pairs are compared as an unordered set: the Flaw
// declares its two (technique, form) groups unordered
// (`unordered_param_groups`, rules/core/virtues_flaws.json), so canonical
// serialization is free to swap which pair is "1" and which is "2" — that is
// the same Flaw, and pinning the slot order would test the sort, not the data.
// ---------------------------------------------------------------------------
describe('D81 parameters survive a save and an Open', () => {
  const NAME = 'Focus Roundtrip';
  const FOCUS = 'Fire';
  const EXPECTED_PAIRS = ['art.intellego+art.herbam', 'art.perdo+art.vim'].sort();
  const SPELL = 'spell.lamp_without_flame';

  async function raiseArt(artId, times) {
    const inc = await $(`[data-testid="art-inc-${artId}"]`);
    await inc.waitForExist({ timeout: STEP_TIMEOUT });
    for (let i = 0; i < times; i++) await inc.click();
  }

  /** The four Incompatible Arts selects' values, as two unordered pairs. */
  async function shownPairs() {
    const value = async (key) =>
      $(`[data-testid^="param-flaw.incompatible_arts-${key}-"]`).getValue();
    return incompatiblePairs({
      technique_1: await value('technique_1'),
      form_1: await value('form_1'),
      technique_2: await value('technique_2'),
      form_2: await value('form_2'),
    });
  }

  it('saves the four selects, the focus text and the within-focus mark', async () => {
    await startCharacter('magus');
    await $(NAME_INPUT).setValue(NAME);

    // Minor Magical Focus with its free-text focus.
    await clickTab(VF_TAB);
    const addFocus = await $('[data-testid="add-virtue.minor_magical_focus"]');
    await addFocus.waitForExist({ timeout: STEP_TIMEOUT });
    await addFocus.click();
    const focusParam = await $('[data-testid^="param-virtue.minor_magical_focus-focus-"]');
    await focusParam.waitForExist({ timeout: STEP_TIMEOUT });
    await focusParam.setValue(FOCUS);
    await waitForValue(focusParam, FOCUS, 'the Magical Focus parameter');

    // Incompatible Arts: (Intellego, Herbam) and (Perdo, Vim) — neither the
    // first option of any select, so a dropped value cannot read back as itself.
    await (await $('[data-testid="add-flaw.incompatible_arts"]')).click();
    const picks = [
      ['technique_1', 'art.intellego'],
      ['form_1', 'art.herbam'],
      ['technique_2', 'art.perdo'],
      ['form_2', 'art.vim'],
    ];
    for (const [key, value] of picks) {
      const select = await $(`[data-testid^="param-flaw.incompatible_arts-${key}-"]`);
      await select.waitForExist({ timeout: STEP_TIMEOUT });
      await select.selectByAttribute('value', value);
      await waitForValue(select, value, `Incompatible Arts ${key}`);
    }

    // Creo 3 / Ignem 3: plain cap 9, focus-doubled cap 12, so Lamp Without Flame
    // (CrIg 10) is addable only within the focus (d81-features.e2e.js).
    await clickTab(ARTS_TAB);
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: STEP_TIMEOUT });
    await pool.setValue('20');
    await raiseArt('art.creo', 3);
    await raiseArt('art.ignem', 3);

    await clickTab(SPELLS_TAB);
    const withinFocusAdd = await $(`[data-testid="add-within-focus-${SPELL}"]`);
    await withinFocusAdd.waitForExist({ timeout: STEP_TIMEOUT });
    await withinFocusAdd.click();
    const marker = await $(`[data-testid^="spell-within-focus-${SPELL}-"]`);
    await marker.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await marker.isSelected()).toBe(true);

    const saved = await saveAndRead();
    const focus = saved.selections.find((s) => s.ref === 'virtue.minor_magical_focus');
    expect(focus?.params?.focus).toBe(FOCUS);
    const incompatible = saved.selections.find((s) => s.ref === 'flaw.incompatible_arts');
    expect(incompatible).toBeDefined();
    expect(incompatiblePairs(incompatible.params)).toEqual(EXPECTED_PAIRS);
    const spell = (saved.spells ?? []).find((s) => s.spell === SPELL);
    expect(spell?.within_focus).toBe(true);
  });

  it('reopens with every one of them shown again', async () => {
    await reopenSavedFile(NAME);

    await clickTab(VF_TAB);
    const focusParam = await $('[data-testid^="param-virtue.minor_magical_focus-focus-"]');
    await focusParam.waitForExist({ timeout: STEP_TIMEOUT });
    await waitForValue(focusParam, FOCUS, 'the reopened Magical Focus parameter');

    await $('[data-testid^="param-flaw.incompatible_arts-technique_1-"]').waitForExist({
      timeout: STEP_TIMEOUT,
    });
    let pairs = [];
    await waitUntilExplained(
      async () => {
        pairs = await shownPairs();
        return JSON.stringify(pairs) === JSON.stringify(EXPECTED_PAIRS);
      },
      STEP_TIMEOUT,
      () => `the reopened Incompatible Arts pairs read ${pairs.join(', ')}`,
    );

    await clickTab(SPELLS_TAB);
    const marker = await $(`[data-testid^="spell-within-focus-${SPELL}-"]`);
    await marker.waitForExist({ timeout: STEP_TIMEOUT });
    await browser.waitUntil(async () => marker.isSelected(), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: 'the reopened spell should still be marked within focus',
    });

    // Looking was not editing.
    await expectClean();
  });

  it('keeps them live: a barred cell reads Unusable, the focus holds the cap', async () => {
    // A reloaded Incompatible Arts pair still bars its Casting Total cell — the
    // parameter reached the engine, not just the select.
    await clickTab(TOTALS_TAB);
    const technique = await $('[data-testid="derived-technique-select"]');
    await technique.waitForExist({ timeout: STEP_TIMEOUT });
    await technique.selectByAttribute('value', 'art.perdo');
    await $('[data-testid="derived-form-select"]').selectByAttribute('value', 'art.vim');
    const unusable = await $('[data-testid="derived-casting-unusable"]');
    await unusable.waitForExist({ timeout: STEP_TIMEOUT });
    expect(clean(await unusable.getText())).toBe('Unusable');

    // The reloaded focus mark is what keeps level 10 under the doubled cap. The
    // magus has no House, so `house_unset` is listed whatever else happens: its
    // presence proves the panel holds a result for THIS character before the
    // absence of the over-cap error is read off it.
    await waitForCodes(
      (codes) => codes.includes('house_unset') && !codes.includes('spell_level_exceeds_cap'),
      'the reopened within-focus spell should sit under the focus-doubled cap',
    );

    // Unmarking it must raise the over-cap error — proof that the mark loaded
    // from the file was doing that work, rather than the cap being loose.
    await clickTab(SPELLS_TAB);
    await $(`[data-testid^="spell-within-focus-${SPELL}-"]`).click();
    await waitForCodes(
      (codes) => codes.includes('spell_level_exceeds_cap'),
      'unmarking the reopened spell should exceed the plain cap',
    );
  });
});

// ---------------------------------------------------------------------------
// 2. A Mythic Companion's type choices through a file.
//
// Devil Child (rules/core/mythic_companion_types.json) carries all three kinds
// of choice the Type tab offers: the type itself, a `choice` grant
// (`devil_child_free_minor`: Demonic Might or Demonic Powers) and a required
// Flaw (`flaw.tragic_life`) that may be swapped for another Major Story Flaw.
// The test takes the NON-default answer to both, so neither can come back from
// the file by falling through to a default.
// ---------------------------------------------------------------------------
describe('a Mythic Companion type survives a save and an Open', () => {
  const NAME = 'Mythic Roundtrip';
  const TYPE = 'mythic_type.devil_child';
  const CHOICE = '[data-testid="mythic-choice-devil_child_free_minor"]';
  const FLAW_SWAP = '[data-testid="mythic-required-flaw-flaw.tragic_life"]';
  let substitute;
  let substituteName;

  it('saves the type, the second free Minor and a substitute required Flaw', async () => {
    await startCharacter('mythic_companion');
    await $(NAME_INPUT).setValue(NAME);

    await clickTab(MYTHIC_TAB);
    const typeSelect = await $('[data-testid="mythic-type-select"]');
    await typeSelect.waitForExist({ timeout: STEP_TIMEOUT });
    await typeSelect.selectByAttribute('value', TYPE);
    await waitForValue(typeSelect, TYPE, 'the Mythic Companion type');

    // Option index 1 is Demonic Powers (index 0, Demonic Might, is the default).
    const choice = await $(CHOICE);
    await choice.waitForExist({ timeout: STEP_TIMEOUT });
    await choice.selectByAttribute('value', '1');
    await waitForValue(choice, '1', 'the free Minor choice');

    // Any substitute the menu offers other than the default. Read off the menu
    // rather than named, so the catalogue's set of Major Story Flaws stays data.
    const swap = await $(FLAW_SWAP);
    await swap.waitForExist({ timeout: STEP_TIMEOUT });
    const options = await swap.$$('option');
    for (let i = 0; i < options.length && substitute === undefined; i++) {
      const value = await options[i].getAttribute('value');
      if (value === 'flaw.tragic_life') continue;
      substitute = value;
      substituteName = clean(await options[i].getProperty('textContent')).trim();
    }
    expect(substitute).toBeDefined();
    await swap.selectByAttribute('value', substitute);
    await waitForValue(swap, substitute, 'the swapped required Flaw');

    const saved = await saveAndRead();
    expect(saved.type_id).toBe('mythic_companion');
    expect(saved.mythic_type).toBe(TYPE);
    expect(saved.mythic_choices?.devil_child_free_minor?.ref).toBe('virtue.demonic_powers');
    const refs = saved.selections.map((s) => s.ref);
    expect(refs).toContain(substitute);
    expect(refs).not.toContain('flaw.tragic_life');
    // The rest of the required package rides along as bought selections.
    expect(refs).toContain('virtue.demonic_blood');
  });

  it('reopens on the same type, choice and substitute', async () => {
    await reopenSavedFile(NAME);

    await clickTab(MYTHIC_TAB);
    const typeSelect = await $('[data-testid="mythic-type-select"]');
    await typeSelect.waitForExist({ timeout: STEP_TIMEOUT });
    await waitForValue(typeSelect, TYPE, 'the reopened Mythic Companion type');
    await $('[data-testid="mythic-granted-virtue.devil_child"]').waitForExist({
      timeout: STEP_TIMEOUT,
    });
    const choice = await $(CHOICE);
    await choice.waitForExist({ timeout: STEP_TIMEOUT });
    await waitForValue(choice, '1', 'the reopened free Minor choice');
    const swap = await $(FLAW_SWAP);
    await swap.waitForExist({ timeout: STEP_TIMEOUT });
    await waitForValue(swap, substitute, 'the reopened required Flaw');

    // The substitute is on the Flaws list, and the default it replaced is not.
    // Read by name: a type's required rows may carry no remove control.
    await clickTab(VF_TAB);
    const flawList = '[data-testid="selection-list-flaw"]';
    await $(flawList).waitForExist({ timeout: STEP_TIMEOUT });
    expect(substituteName.length).toBeGreaterThan(0);
    await browser.waitUntil(async () => (await textContentOf(flawList)).includes(substituteName), {
      timeout: STEP_TIMEOUT,
      timeoutMsg: `the reopened Flaws list should name ${substituteName}`,
    });
    expect(await textContentOf(flawList)).not.toContain('Tragic Life');

    await expectClean();
  });
});

// ---------------------------------------------------------------------------
// 3. Direct-unchecked entry: Silent mode keeps an illegal character.
//
// Silent is the "without rule checking" input mode: nothing greys, nothing is
// reported, and the player may record a character the rules forbid. That
// character is still the player's data, so it must survive a save and an Open
// exactly — and the same engine must report it again the moment the mode goes
// back to Enforced (CLAUDE.md: one evaluation path, the mode only governs
// enforcement). A companion taking Blatant Gift (a Hermetic Flaw the companion
// profile forbids) and Gentle Gift (its declared incompatible counterpart) is
// illegal twice over, deterministically.
// ---------------------------------------------------------------------------
describe('direct-unchecked (Silent) entry keeps an illegal character through a file', () => {
  const NAME = 'Unchecked Roundtrip';

  // The mode is app-wide and persisted; every later describe assumes Enforced.
  after(async () => {
    await setValidationMode('enforced');
  });

  it('takes an illegal pair in Silent mode and reports nothing', async () => {
    await startCharacter('companion');
    await setValidationMode('silent');
    await $(NAME_INPUT).setValue(NAME);

    await clickTab(VF_TAB);
    const blatant = await $('[data-testid="add-flaw.blatant_gift"]');
    await blatant.waitForExist({ timeout: STEP_TIMEOUT });
    await blatant.click();
    await $('[data-testid^="remove-flaw.blatant_gift-"]').waitForExist({ timeout: STEP_TIMEOUT });

    // In Enforced this row greys the moment Blatant Gift is taken
    // (companion-editor.e2e.js); unchecked, it stays takeable.
    const gentle = await $('[data-testid="add-virtue.gentle_gift"]');
    await gentle.waitForExist({ timeout: STEP_TIMEOUT });
    expect(await isRowBlocked(gentle)).toBe(false);
    await gentle.click();
    await $('[data-testid^="remove-virtue.gentle_gift-"]').waitForExist({ timeout: STEP_TIMEOUT });

    await $('[data-testid="no-issues"]').waitForExist({ timeout: STEP_TIMEOUT });
    expect((await issueCodes()).length).toBe(0);
  });

  it('saves the illegal character exactly as entered', async () => {
    const saved = await saveAndRead();
    const refs = saved.selections.map((s) => s.ref);
    expect(refs).toContain('flaw.blatant_gift');
    expect(refs).toContain('virtue.gentle_gift');
    expect(saved.type_id).toBe('companion');
  });

  it('reopens it intact, still unchecked', async () => {
    await reopenSavedFile(NAME);

    await clickTab(VF_TAB);
    await $('[data-testid^="remove-flaw.blatant_gift-"]').waitForExist({ timeout: STEP_TIMEOUT });
    await $('[data-testid^="remove-virtue.gentle_gift-"]').waitForExist({ timeout: STEP_TIMEOUT });
    await $('[data-testid="no-issues"]').waitForExist({ timeout: STEP_TIMEOUT });
    await expectClean();
  });

  it('reports both violations as errors once Enforced is back', async () => {
    await setValidationMode('enforced');
    await waitForCodes(
      (codes) => codes.includes('forbidden_category') && codes.includes('incompatible'),
      'Enforced should report the reopened forbidden Flaw and the incompatible pair',
    );
    for (const code of ['forbidden_category', 'incompatible']) {
      expect(await $(`${ISSUE_LIST} li[data-code="${code}"]`).getAttribute('data-severity')).toBe(
        'error',
      );
    }
    // Changing how violations are SHOWN is not an edit to the character.
    await expectClean();
  });
});

// ---------------------------------------------------------------------------
// 4. The German locale on a full companion sheet, and its export.
//
// app-shell.e2e.js proves German CHROME (one region title); magus-possessions
// proves German export HEADINGS for a magus. Neither looks at the rules text a
// German user actually reads — the names of the Virtues, Flaws and Abilities
// they picked — nor at a non-magus export. German names here are the shipped
// `rules/i18n/de/` values (Keen Vision -> Scharfe Sicht, Poor Student ->
// Schlechter Schüler, Awareness -> Aufmerksamkeit) and `locales/de/main.ftl`
// (companion -> Gefährte).
// ---------------------------------------------------------------------------
describe('German locale: a full companion sheet and its export', () => {
  const NAME = 'Hilde';

  // A raw catalogue id (`virtue.keen_vision`) or an unresolved Fluent key
  // (`category-general`, `tab-details`, …) rendered as text — the CLAUDE.md
  // data-kind rule's two failure shapes. Fluent keys are lowercase-hyphenated
  // and start with one of the prefixes the .ftl files use; German prose
  // capitalizes its nouns, so these patterns do not collide with real text.
  const RAW_ID =
    /\b(virtue|flaw|ability|art|spell|house|mythic_type|characteristic|weapon|armor|shield)\.[a-z_]+/;
  const FLUENT_KEY =
    /\b(tab|category|magnitude|type|identity|derived|export|param|issue|mythic|ability|abilities|spell|art|vf|xp|balance|wizard|confidence|personality|reputation|aging|equipment)-[a-z][a-z0-9-]*/;

  after(async () => {
    await setLanguage('en');
  });

  it('renders the picked Virtue, Flaw and Ability under their German names', async () => {
    await setLanguage('de');
    await startCharacter('companion');
    await $(NAME_INPUT).setValue(NAME);

    const typeLabel = await textContentOf('[data-testid="character-type"]');
    expect(typeLabel).toContain('Gefährte');

    await clickTab(VF_TAB);
    // Each add settles (row + balance) before the next click
    // (`waitForBalancePoints`); the issues footer itself is a fixed height (U3).
    await waitForBalancePoints('virtues', 0);
    await (await $('[data-testid="add-virtue.keen_vision"]')).click();
    await $('[data-testid^="remove-virtue.keen_vision-"]').waitForExist({
      timeout: SETTLE_TIMEOUT,
    });
    await waitForBalancePoints('virtues', 1);
    await (await $('[data-testid="add-flaw.poor_student"]')).click();
    await $('[data-testid^="remove-flaw.poor_student-"]').waitForExist({ timeout: STEP_TIMEOUT });
    await waitForBalancePoints('flaws', 1);
    const virtues = await textContentOf('[data-testid="selection-list-virtue"]');
    const flaws = await textContentOf('[data-testid="selection-list-flaw"]');
    expect(virtues).toContain('Scharfe Sicht');
    expect(virtues).not.toContain('Keen Vision');
    expect(flaws).toContain('Schlechter Schüler');
    expect(flaws).not.toContain('Poor Student');

    await clickTab(ABILITIES_TAB);
    const pool = await $('[data-testid="xp-pool"]');
    await pool.waitForExist({ timeout: STEP_TIMEOUT });
    await pool.setValue('30');
    await (await $('[data-testid="add-ability.awareness"]')).click();
    const inc = await $('[data-testid="ability-inc-ability.awareness-0"]');
    await inc.waitForExist({ timeout: STEP_TIMEOUT });
    await inc.click();
    await inc.click();
    await browser.waitUntil(
      async () =>
        (await textContentOf('[data-testid="ability-score-ability.awareness-0"]')) === '2',
      { timeout: STEP_TIMEOUT, timeoutMsg: 'Aufmerksamkeit should reach 2' },
    );
    const abilities = await textContentOf(TAB_PANEL);
    expect(abilities).toContain('Aufmerksamkeit');
    expect(abilities).not.toContain('Awareness');
  });

  it('shows no raw id or Fluent key on any tab', async () => {
    const tabs = await $$(`${TAB_BAR} [role="tab"]`);
    const ids = [];
    for (let i = 0; i < tabs.length; i++) ids.push(await tabs[i].getAttribute('data-testid'));
    expect(ids.length).toBeGreaterThan(0);

    const leaks = [];
    for (const id of ids) {
      await clickTab(`[data-testid="${id}"]`);
      await browser.waitUntil(
        async () => (await $(`[data-testid="${id}"]`).getAttribute('aria-selected')) === 'true',
        { timeout: STEP_TIMEOUT, timeoutMsg: `${id} did not become the selected tab` },
      );
      // The whole app root — tab strip, banner, panel, findings: everything on
      // screen but the native menu (which app-shell.e2e.js reads in German).
      const label = await textContentOf(`[data-testid="${id}"]`);
      const text = await textContentOf('#app');
      // Guard against a vacuous pass: the scan really read the rendered app.
      expect(label.trim().length).toBeGreaterThan(0);
      expect(text).toContain(label.trim());
      expect(text).toContain('Gefährte'); // the banner's type label
      for (const pattern of [RAW_ID, FLUENT_KEY]) {
        const hit = text.match(pattern);
        if (hit) leaks.push(`${id} ("${label}"): ${hit[0]}`);
      }
    }
    expect(leaks).toEqual([]);
  });

  it('exports a German sheet for the companion', async () => {
    if (fs.existsSync(e2eExportFile)) fs.unlinkSync(e2eExportFile);
    await runDocumentAction('export');
    let md = '';
    await browser.waitUntil(
      () => {
        if (!fs.existsSync(e2eExportFile)) return false;
        md = fs.readFileSync(e2eExportFile, 'utf-8');
        // Confidence is the last section a companion with no annotations emits,
        // so its heading means the write has fully landed.
        return md.includes('## Selbstvertrauen');
      },
      { timeout: 15000, timeoutMsg: 'export did not write the German companion sheet' },
    );
    await waitForIdle();
    md = fs.readFileSync(e2eExportFile, 'utf-8');

    expect(md.startsWith(`# ${NAME}\n`)).toBe(true);
    expect(md).toContain('Gefährte');
    // No `## Eigenschaften`: every Characteristic is still 0, and the exporter
    // omits an empty section (crates/arm-rules/src/export.rs module docs).
    for (const heading of [
      '## Tugenden & Fehler',
      '## Fertigkeiten',
      '## Wunden',
      '## Selbstvertrauen',
    ]) {
      expect(md).toContain(heading);
    }
    // A non-magus sheet carries no Hermetic sections at all.
    expect(md).not.toContain('## Künste');
    expect(md).not.toContain('## Zauber');

    // The rules text in German, with the German category and magnitude beside it.
    expect(md).toMatch(/\| Scharfe Sicht \| Allgemein \| Klein \|/);
    expect(md).toContain('Schlechter Schüler');
    expect(md).toMatch(/\| Aufmerksamkeit \|[^\n]*\| 2 \|/);
    for (const english of ['Keen Vision', 'Poor Student', 'Awareness', '## Abilities']) {
      expect(md).not.toContain(english);
    }
    expect(md).not.toMatch(RAW_ID);
  });
});
