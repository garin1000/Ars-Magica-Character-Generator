// End-to-end: the companion-shaped editor surface — Characteristic caps and
// Ability bonuses, illegal-selection reporting and its per-mode variants,
// mechanical Virtue/Flaw effects, mutual exclusion, warping-owed fills, the
// core tabbed edit/save/reload flow, per-character fields, and the
// window.close() bridge for a DIRTY document.
//
// A2 merged ten previously separate spec files into this one, all built on
// `startCharacter('companion')` (a few also touch 'grog'/'magus' for contrast),
// so the setup each used to pay for separately is paid once per describe rather
// than once per file. Each describe below keeps its own local selectors and
// helpers, scoped to its block, exactly as when it was a standalone file.
//
// `validation modes` ends on Silent, and `ValidationMode` is app-wide state that
// survives `startCharacter()` exactly like the UI language — it is not part of
// what a fresh character resets. Its `after` hook restores `enforced` so every
// later describe here gets the default mode its assertions assume (greying,
// blocking, and `data-code` findings actually rendering). Found by running the
// merged file end to end: five failures, all in describes after `validation
// modes`, all explainable by findings being suppressed under Silent.
//
// `window.close() bridge — unsaved changes` MUST stay last: with unsaved edits,
// `guard_blocks_quit` raises a native GTK confirmation dialog that WebDriver
// cannot dismiss, and that dialog is left open for the rest of this worker's app
// instance.

import { $, $$, browser, expect } from '@wdio/globals';
import fs from 'node:fs';

import { clean, isRowBlocked, runDocumentAction, startCharacter } from '../helpers.js';
import { e2eFile } from '../wdio.conf.js';

// End-to-end: Great Characteristic raises a Characteristic's buy cap (it grants
// no free point — the score must be bought past +3), and Puissant Ability (+2)
// shows as an "effective" badge on its target ability.
describe('characteristic cap + ability bonus', () => {
  it('Great Characteristic opens the cap, Puissant shows a badge', async () => {
    await startCharacter('companion');

    // Open the Characteristics tab; raise Strength to its base cap of +3.
    await $('[data-testid="tab-characteristics"]').waitForExist({ timeout: 30000 });
    await $('[data-testid="tab-characteristics"]').click();
    const strInc = await $('[data-testid="char-inc-str"]');
    await strInc.waitForExist({ timeout: 30000 });
    await strInc.click();
    await strInc.click();
    await strInc.click();
    expect(await $('[data-testid="char-value-str"]').getText()).toBe('+3');
    // At the base cap the increment button is disabled — no headroom yet.
    expect(await strInc.isEnabled()).toBe(false);

    // Add Great Characteristic and target Strength via the characteristic picker.
    await $('[data-testid="tab-virtues_flaws"]').click();
    const addGreat = await $('[data-testid="add-virtue.great_characteristic"]');
    await addGreat.waitForExist({ timeout: 10000 });
    await addGreat.click();
    // Match by testid prefix: the index suffix is the entity-array position,
    // which depends on what else is selected, so don't pin it.
    const charParam = await $('[data-testid^="param-virtue.great_characteristic-characteristic"]');
    await charParam.waitForExist({ timeout: 5000 });
    await charParam.selectByAttribute('value', 'characteristic.str');

    // Back on Characteristics: the cap is now +4, so increment is enabled again
    // and clicking it buys Strength up to +4 (no free point was granted).
    await $('[data-testid="tab-characteristics"]').click();
    const strInc2 = await $('[data-testid="char-inc-str"]');
    await strInc2.waitForClickable({ timeout: 10000 });
    await strInc2.click();
    expect(await $('[data-testid="char-value-str"]').getText()).toBe('+4');
    // +4 is the cap with one Great; the button disables again.
    expect(await strInc2.isEnabled()).toBe(false);

    // Buy Awareness up to 2, then add Puissant Ability targeting it.
    await $('[data-testid="tab-abilities"]').click();
    await $('[data-testid="xp-pool"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="xp-pool"]').setValue(30);
    await $('[data-testid="add-ability.awareness"]').click();
    const awarenessInc = await $('[data-testid="ability-inc-ability.awareness-0"]');
    await awarenessInc.waitForExist({ timeout: 5000 });
    await awarenessInc.click();
    await awarenessInc.click();
    expect(await $('[data-testid="ability-score-ability.awareness-0"]').getText()).toBe('2');

    await $('[data-testid="tab-virtues_flaws"]').click();
    await $('[data-testid="add-virtue.puissant_ability"]').click();
    const abilityParam = await $('[data-testid^="param-virtue.puissant_ability-ability"]');
    await abilityParam.waitForExist({ timeout: 5000 });
    await abilityParam.selectByAttribute('value', 'ability.awareness');

    // Back on Abilities: Awareness shows effective 4 (base 2, Puissant +2).
    await $('[data-testid="tab-abilities"]').click();
    const abilityBadge = await $('[data-testid="ability-eff-ability.awareness-0"]');
    await abilityBadge.waitForExist({ timeout: 10000 });
    expect(await abilityBadge.getText()).toContain('4');
  });
});

// End-to-end: Puissant (Ability) targets one specific instance of a
// parameterized ability ((Area) Lore). The +2 must attach to exactly the chosen
// area's row and not bleed onto the character's other areas.
describe('Puissant Ability targets one ability instance', () => {
  // Add an (Area) Lore instance at row `i`, name its area, and raise it to `score`.
  async function addLore(i, area, score) {
    await $('[data-testid="add-ability.area_lore"]').click();
    const param = await $(`[data-testid="ability-param-ability.area_lore-${i}"]`);
    await param.waitForExist({ timeout: 5000 });
    await param.setValue(area);
    const inc = await $(`[data-testid="ability-inc-ability.area_lore-${i}"]`);
    for (let n = 0; n < score; n++) await inc.click();
  }

  it('boosts only the chosen (Area) Lore, not the others', async () => {
    // The row indices below are the ability-array positions, so this needs a
    // character with no abilities yet.
    await startCharacter('companion');

    // Abilities tab: fund XP, then add three distinct Area Lore instances.
    const abilitiesTab = await $('[data-testid="tab-abilities"]');
    await abilitiesTab.waitForExist({ timeout: 30000 });
    await abilitiesTab.click();
    await $('[data-testid="xp-pool"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="xp-pool"]').setValue(100);

    await addLore(0, 'Brandenburg', 2);
    await addLore(1, 'Berlin', 2);
    await addLore(2, 'Bavaria', 1);
    expect(await $('[data-testid="ability-score-ability.area_lore-0"]').getText()).toBe('2');

    // Virtues tab: add Puissant twice, targeting Brandenburg and Bavaria.
    await $('[data-testid="tab-virtues_flaws"]').click();
    const add = await $('[data-testid="add-virtue.puissant_ability"]');
    await add.waitForExist({ timeout: 10000 });
    await add.click();
    await add.click();

    const first = await $('[data-testid="param-virtue.puissant_ability-ability-0"]');
    await first.waitForExist({ timeout: 5000 });
    await first.selectByVisibleText('Brandenburg Lore');
    const second = await $('[data-testid="param-virtue.puissant_ability-ability-1"]');
    await second.selectByVisibleText('Bavaria Lore');

    // Back on Abilities: the two targeted instances show their score + 2, and the
    // untargeted Berlin row has no effective badge at all.
    await $('[data-testid="tab-abilities"]').click();
    const brandenburg = await $('[data-testid="ability-eff-ability.area_lore-0"]');
    await brandenburg.waitForExist({ timeout: 10000 });
    expect(await brandenburg.getText()).toContain('4'); // 2 + 2

    const bavaria = await $('[data-testid="ability-eff-ability.area_lore-2"]');
    await bavaria.waitForExist({ timeout: 10000 });
    expect(await bavaria.getText()).toContain('3'); // 1 + 2

    // Berlin was never targeted: no Puissant bonus bled onto it.
    expect(await $('[data-testid="ability-eff-ability.area_lore-1"]').isExisting()).toBe(false);
  });
});

// End-to-end: an illegal selection surfaces a localized error in the validation
// panel.
//
// Why a forbidden-category flaw rather than an over-budget case: the shipped
// sample ruleset (rules/core/) has too few selectable virtues to exceed the
// companion's 10-point virtue budget, so over_budget_virtues is not reachable
// purely through clicks. `flaw.blatant_gift` is in the `hermetic` category,
// which the default `companion` profile forbids, giving a deterministic
// error-severity issue through the same render path the over-budget case uses.
describe('validation errors', () => {
  it('shows a localized, error-severity issue for a forbidden-category selection', async () => {
    // A fresh companion, so "no issues before any selection" below is about this
    // character rather than whatever the previous spec left behind.
    await startCharacter('companion');

    // The shared validation bar (bottom) reports for the whole character; the V/F
    // add buttons live in the Virtues & Flaws tab.
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });

    // No issues before any selection.
    await expect($('[data-testid="no-issues"]')).toExist();

    await vfTab.click();
    const addForbidden = await $('[data-testid="add-flaw.blatant_gift"]');
    await addForbidden.waitForExist({ timeout: 10000 });
    await addForbidden.click();

    // The issue list now contains at least one error-severity issue.
    const issueList = await $('[data-testid="issue-list"]');
    await issueList.waitForExist({ timeout: 5000 });

    const errors = await $$('[data-severity="error"]');
    expect(await errors.length).toBeGreaterThan(0);

    // The message is localized (names the offending item, never a raw id or the
    // i18n key): a raw slug rendered as a label would violate the strict
    // data-kind separation in CLAUDE.md.
    const firstError = errors[0];
    const text = await firstError.getText();
    expect(text).not.toContain('issue-');
    expect(text).not.toContain('flaw.blatant_gift');
    expect(text).toContain('Blatant Gift');
  });
});

// End-to-end: the three ValidationModes change how the SAME illegal entity is
// reported. Enforced keeps errors, Advisory downgrades them to warnings, Silent
// clears the panel entirely.
describe('validation modes', () => {
  // `flaw.blatant_gift` (hermetic category) is forbidden for the `companion`
  // profile, so it yields a deterministic error in Enforced mode.
  const FORBIDDEN = '[data-testid="add-flaw.blatant_gift"]';
  const MODE_SELECT = '[data-testid="mode-select"]';

  async function severities() {
    // Index-based loop: in webdriverio v9 the awaited `$$` result's `.map` does
    // not yield a plain iterable, so `Promise.all(items.map(...))` throws. Element
    // indexing (`items[i]`) and `.length` are stable, so read each in turn.
    const items = await $$('[data-testid="issue-list"] li');
    const result = [];
    for (let i = 0; i < items.length; i++) {
      result.push(await items[i].getAttribute('data-severity'));
    }
    return result;
  }

  // Small polling helper (validation is debounced + async, so the DOM settles a
  // tick after the mode change).
  async function browserWaitFor(predicate, timeout = 5000) {
    await browser.waitUntil(predicate, { timeout, timeoutMsg: 'condition not met in time' });
  }

  // A2 fix: this describe ends on Silent, and `ValidationMode` is app-wide state
  // that survives `startCharacter()` exactly like the UI language (`helpers.js`'s
  // own "App-wide state" note) — it is not reset by creating a new character. Every
  // describe below this one in the shared session assumes the default Enforced
  // mode (greying/blocking behaviour, `data-code` findings actually rendering), so
  // without this restore they silently ran in Silent mode instead and their
  // findings-panel assertions failed. Caught by a real e2e run, not by inspection.
  after(async () => {
    await $(MODE_SELECT).selectByAttribute('value', 'enforced');
  });

  it('reports the same illegal entity differently per mode', async () => {
    // A companion whose only issue is the forbidden flaw added below, so the
    // per-mode severity counts are unambiguous.
    await startCharacter('companion');

    // V/F add buttons live in the Virtues & Flaws tab; the mode select and the
    // shared validation bar are visible throughout the editor.
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();
    const addForbidden = await $(FORBIDDEN);
    await addForbidden.waitForExist({ timeout: 10000 });
    await addForbidden.click();

    const modeSelect = await $(MODE_SELECT);

    // Enforced (default): at least one error-severity issue, no warnings.
    await modeSelect.selectByAttribute('value', 'enforced');
    await $('[data-testid="issue-list"]').waitForExist({ timeout: 5000 });
    let sev = await severities();
    expect(sev).toContain('error');
    expect(sev).not.toContain('warning');

    // Advisory: every issue is downgraded to a warning; no errors remain.
    await modeSelect.selectByAttribute('value', 'advisory');
    await browserWaitFor(async () => {
      const s = await severities();
      return s.length > 0 && !s.includes('error');
    });
    sev = await severities();
    expect(sev).toContain('warning');
    expect(sev).not.toContain('error');

    // Silent: the panel reports no issues at all.
    await modeSelect.selectByAttribute('value', 'silent');
    await $('[data-testid="no-issues"]').waitForExist({ timeout: 5000 });
    expect(await $$('[data-testid="issue-list"] li').then((l) => l.length)).toBe(0);
  });
});

// End-to-end: Phase-3 virtue/flaw mechanical effects — Improved Characteristics
// (+3 buy budget), restricted XP pools (Warrior), Affinity (reduced Art XP), and
// an ability_score_grant floor (Second Sight 1).
describe('phase-3 virtue/flaw effects', () => {
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const CHARS_TAB = '[data-testid="tab-characteristics"]';
  const ABILITIES_TAB = '[data-testid="tab-abilities"]';
  const ARTS_TAB = '[data-testid="tab-arts"]';

  async function addVirtue(id) {
    await $(VF_TAB).click();
    const add = await $(`[data-testid="add-${id}"]`);
    await add.waitForExist({ timeout: 10000 });
    await add.click();
  }

  it('Improved Characteristics raises the Characteristic-buy budget by 3', async () => {
    await startCharacter('companion');
    await $(VF_TAB).waitForExist({ timeout: 30000 });

    // Base budget is 7.
    await $(CHARS_TAB).click();
    const points = await $('[data-testid="characteristic-points"]');
    await points.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => clean(await points.getText()).includes('/ 7'), {
      timeout: 5000,
      timeoutMsg: 'base Characteristic budget should be 7',
    });

    await addVirtue('virtue.improved_characteristics');

    await $(CHARS_TAB).click();
    await browser.waitUntil(async () => clean(await points.getText()).includes('/ 10'), {
      timeout: 5000,
      timeoutMsg: 'Improved Characteristics should raise the budget to 10',
    });
  });

  it('Warrior adds a restricted XP pool eligible for Martial Abilities', async () => {
    await startCharacter('companion');
    await addVirtue('virtue.warrior');

    await $(ABILITIES_TAB).click();
    const pool = await $('[data-testid="restricted-xp-0"]');
    await pool.waitForExist({ timeout: 10000 });
    // 50 restricted points, none yet spent.
    await browser.waitUntil(async () => clean(await pool.getText()).includes('0 / 50'), {
      timeout: 5000,
      timeoutMsg: 'Warrior should show a 0 / 50 restricted pool',
    });

    // Buy a Martial Ability at 1 (5 XP): the Warrior pool funds it, so none of it
    // draws on the general pool. In the single-row layout the restricted sub-budget
    // rises to "5 / 50" while the read-only general-used figure stays 0 — the general
    // and restricted portions are shown separately, not merged into one "Spent".
    const add = await $('[data-testid="add-ability.single_weapon"]');
    await add.waitForExist({ timeout: 5000 });
    await add.click();
    const inc = await $('[data-testid="ability-inc-ability.single_weapon-0"]');
    await inc.waitForExist({ timeout: 5000 });
    await inc.click();

    await browser.waitUntil(async () => clean(await pool.getText()).includes('5 / 50'), {
      timeout: 5000,
      timeoutMsg: 'buying a Martial Ability should draw on the Warrior pool',
    });
    await browser.waitUntil(
      async () => clean(await $('[data-testid="xp-spent"]').getText()).trim() === '0',
      {
        timeout: 5000,
        timeoutMsg: 'the general-used figure should stay 0 when the Warrior pool funds the spend',
      },
    );

    // Removing the bought Ability must free the pool again — the other half of the
    // restricted-pool contract, and cheap to check while it is set up.
    await $('[data-testid="remove-ability.single_weapon-0"]').click();
    await browser.waitUntil(async () => clean(await pool.getText()).includes('0 / 50'), {
      timeout: 5000,
      timeoutMsg: 'removing the Martial Ability should free the Warrior pool again',
    });
  });

  it('Second Sight confers the Ability at a free effective floor of 1', async () => {
    await startCharacter('companion');
    await addVirtue('virtue.second_sight');

    // Add the Second Sight Ability at 0; the grant floors its effective score at 1.
    await $(ABILITIES_TAB).click();
    const add = await $('[data-testid="add-ability.second_sight"]');
    await add.waitForExist({ timeout: 10000 });
    await add.click();

    const eff = await $('[data-testid="ability-eff-ability.second_sight-0"]');
    await eff.waitForExist({ timeout: 5000 });
    await browser.waitUntil(async () => clean(await eff.getText()).includes('1'), {
      timeout: 5000,
      timeoutMsg: 'Second Sight should read as effective 1 at score 0',
    });
    // The bought score is still 0 — the grant cost no experience.
    expect(await $('[data-testid="ability-score-ability.second_sight-0"]').getText()).toBe('0');
  });

  it('Affinity with Art reduces the XP charged for that Art', async () => {
    await startCharacter('magus');

    // Affinity with Creo, then raise Creo to 5 (table cost 15 → charged 10).
    await addVirtue('virtue.affinity_art');
    const target = await $('[data-testid^="param-virtue.affinity_art-art-"]');
    await target.waitForExist({ timeout: 5000 });
    await target.selectByAttribute('value', 'art.creo');

    await $(ARTS_TAB).click();
    const pool = await $('[data-testid="art-xp-pool"]');
    await pool.waitForExist({ timeout: 10000 });
    await pool.setValue('30');
    const inc = await $('[data-testid="art-inc-art.creo"]');
    await inc.waitForExist({ timeout: 5000 });
    for (let i = 0; i < 5; i++) await inc.click();

    await browser.waitUntil(
      async () => clean(await $('[data-testid="art-xp-spent"]').getText()).includes('10'),
      { timeout: 5000, timeoutMsg: 'Affinity should reduce Creo 5 from 15 to 10 XP' },
    );
  });
});

// End-to-end: mutually exclusive Virtues/Flaws cannot both be picked in Enforced
// mode — the counterpart's Add button greys out — while Advisory leaves the pick
// open and reports the clash as an issue instead. Covers both flavours of
// exclusion: a hand-authored clique (Gentle vs Blatant Gift) and a Major/Minor
// magnitude pair of the same Flaw (Ambitious).
describe('mutually exclusive Virtues/Flaws', () => {
  const MODE_SELECT = '[data-testid="mode-select"]';
  const INCOMPATIBLE_ISSUE = '[data-testid="issue-list"] li[data-code="incompatible"]';

  async function addButton(ref) {
    const button = await $(`[data-testid="add-${ref}"]`);
    await button.waitForExist({ timeout: 10000 });
    return button;
  }

  async function waitForEnabled(ref, enabled) {
    const button = await addButton(ref);
    await browser.waitUntil(async () => (await isRowBlocked(button)) === !enabled, {
      timeout: 5000,
      timeoutMsg: `expected add-${ref} to be ${enabled ? 'enabled' : 'greyed out'}`,
    });
  }

  before(async () => {
    // One companion for both blocks: the first clears its picks again, so the
    // second starts from the clean sheet its comment describes.
    await startCharacter('companion');
    const vfTab = await $('[data-testid="tab-virtues_flaws"]');
    await vfTab.waitForExist({ timeout: 30000 });
    await vfTab.click();
  });

  it('greys out an excluded counterpart in Enforced mode, but not in Advisory', async () => {
    // Gentle Gift and Blatant Gift exclude each other via `incompatible_with`.
    const blatant = await addButton('flaw.blatant_gift');
    expect(await isRowBlocked(blatant)).toBe(false);
    await blatant.click();

    // Enforced (default): the counterpart is no longer takeable, so the illegal
    // combination cannot be reached by clicking at all.
    await waitForEnabled('virtue.gentle_gift', false);

    // Advisory: the pick is allowed again and the clash is reported instead.
    const modeSelect = await $(MODE_SELECT);
    await modeSelect.selectByAttribute('value', 'advisory');
    await waitForEnabled('virtue.gentle_gift', true);
    await (await addButton('virtue.gentle_gift')).click();
    await $(INCOMPATIBLE_ISSUE).waitForExist({ timeout: 5000 });

    // Back to Enforced: the same state now reports at error severity.
    await modeSelect.selectByAttribute('value', 'enforced');
    await browser.waitUntil(
      async () => (await $(INCOMPATIBLE_ISSUE).getAttribute('data-severity')) === 'error',
      { timeout: 5000, timeoutMsg: 'expected an error-severity incompatibility issue' },
    );

    // Clear both picks so the magnitude-pair case starts from a clean sheet.
    for (const ref of ['virtue.gentle_gift', 'flaw.blatant_gift']) {
      const remove = await $(`[data-testid^="remove-${ref}-"]`);
      await remove.waitForExist({ timeout: 5000 });
      await remove.click();
    }
    await $('[data-testid="no-issues"]').waitForExist({ timeout: 5000 });
  });

  it('greys out the Major variant of an already selected Minor Flaw', async () => {
    // A character may take only one magnitude of the same Virtue/Flaw.
    await (await addButton('flaw.ambitious_minor')).click();
    await waitForEnabled('flaw.ambitious_major', false);

    // Removing the Minor variant frees the Major one again.
    const remove = await $('[data-testid^="remove-flaw.ambitious_minor-"]');
    await remove.waitForExist({ timeout: 5000 });
    await remove.click();
    await waitForEnabled('flaw.ambitious_major', true);
  });
});

// End-to-end (Issue E): the off-budget Virtues/Flaws a character owes from its
// Warping Score ("Effects of Warping", Core:16547-16561). A warped non-magus
// surfaces one picker per owed slot on the Details tab; a magus (exempt — Twilight
// instead) never shows the section, even at the same Warping Score.
//
// Each block creates its own character, so it neither inherits state nor needs
// to restore any.
describe('warping-owed Virtues/Flaws', () => {
  const DETAILS_TAB = '[data-testid="tab-details"]';
  const WARPING_POINTS = '[data-testid="warping-points-input"]';
  const WARPING_OWED = '[data-testid="warping-owed"]';
  const MINOR_FLAW_FILL = '[data-testid="warping-fill-warping.minor_flaw.0"]';
  // One group per owed kind, labelled with what its slots expect.
  const MINOR_FLAW_GROUP = '[data-testid="warping-owed-group-warping-slot-minor-flaw"]';
  const VIRTUE_GROUP = '[data-testid="warping-owed-group-warping-slot-supernatural-virtue"]';
  const SLOT_SELECT = 'select[data-testid^="warping-fill-"]';
  const FORM_PARAM =
    '[data-testid="param-virtue.master_of_form_creatures-form-warping.supernatural_virtue.0"]';
  const MISSING_PARAM = '[data-code="missing_param"]';

  it('surfaces an owed Minor Flaw picker for a warped non-magus', async () => {
    await startCharacter('companion');
    await $(DETAILS_TAB).click();

    // 5 Warping Points → Warping Score 1 → owes one Minor Flaw.
    await $(WARPING_POINTS).waitForExist({ timeout: 10000 });
    await $(WARPING_POINTS).setValue('5');

    await $(WARPING_OWED).waitForExist({
      timeout: 10000,
    });
    await $(MINOR_FLAW_FILL).waitForExist({
      timeout: 10000,
    });

    // Filling the slot resolves cleanly (the fill is off-budget).
    await $(MINOR_FLAW_FILL).selectByAttribute('value', 'flaw.ability_block');
  });

  it('groups the owed slots per kind and resolves a parameterized fill', async () => {
    // Its own companion: the slot counts asserted below must be produced by this
    // test's Warping Score alone, with no fill left over from the previous one.
    await startCharacter('companion');
    await $(DETAILS_TAB).click();

    // 75 Warping Points → Warping Score 5 → owes 2 Minor Flaws + 1 supernatural
    // Minor Virtue (Core:16553-16559), in two labelled groups.
    await $(WARPING_POINTS).waitForExist({ timeout: 10000 });
    await $(WARPING_POINTS).setValue('75');
    await $(MINOR_FLAW_GROUP).waitForExist({ timeout: 10000 });
    await $(VIRTUE_GROUP).waitForExist({ timeout: 10000 });
    // Count only the slot selects: a parameterized pick adds its own control.
    await browser.waitUntil(
      async () => (await $$(`${MINOR_FLAW_GROUP} ${SLOT_SELECT}`)).length === 2,
      { timeout: 10000, timeoutMsg: 'the Minor Flaw group should hold both owed slots' },
    );
    expect((await $$(`${VIRTUE_GROUP} ${SLOT_SELECT}`)).length).toBe(1);

    // Master of (Form) Creatures is a supernatural Minor Virtue whose Form must be
    // named; unnamed, the engine reports the parameter missing.
    const virtueFill = await $(`${VIRTUE_GROUP} ${SLOT_SELECT}`);
    await virtueFill.selectByAttribute('value', 'virtue.master_of_form_creatures');
    await browser.waitUntil(async () => await $(MISSING_PARAM).isExisting(), {
      timeout: 10000,
      timeoutMsg: 'a parameterized owed fill with no parameter should report missing_param',
    });

    const param = await $(FORM_PARAM);
    await param.waitForExist({ timeout: 10000 });
    await param.selectByAttribute('value', 'art.ignem');
    await browser.waitUntil(async () => !(await $(MISSING_PARAM).isExisting()), {
      timeout: 10000,
      timeoutMsg: 'choosing the Form should clear missing_param',
    });
  });

  it('hides the section for a magus at the same Warping Score', async () => {
    // The comparison only means something at the SAME Warping Score, and a freshly
    // created magus starts at 0 — so give this one the previous test's 75 points
    // (Warping Score 5) before asserting the section is absent.
    await startCharacter('magus');
    await $(DETAILS_TAB).click();
    await $(WARPING_POINTS).waitForExist({ timeout: 10000 });
    await $(WARPING_POINTS).setValue('75');
    await browser.waitUntil(async () => (await $(WARPING_POINTS).getValue()) === '75', {
      timeout: 10000,
      timeoutMsg: 'the magus should carry the same 75 Warping Points',
    });

    await browser.waitUntil(async () => !(await $(WARPING_OWED).isExisting()), {
      timeout: 10000,
      timeoutMsg: 'a magus must not show the owed-warping section (Twilight instead)',
    });
  });
});

// End-to-end: drive the tabbed editor — set a Characteristic (spinner), pick a
// balanced V/F pair, buy an Ability against an XP pool — then save and reload.
describe('character editor', () => {
  it('edits across tabs, validates, and round-trips a save', async () => {
    await startCharacter('companion');

    // Open the Characteristics tab; its spinner appearing means the ruleset
    // has loaded.
    await $('[data-testid="tab-characteristics"]').waitForExist({ timeout: 30000 });
    await $('[data-testid="tab-characteristics"]').click();
    const intInc = await $('[data-testid="char-inc-int"]');
    await intInc.waitForExist({ timeout: 30000 });

    // Raise Intelligence to +2 with the spinner (0 -> +1 -> +2).
    await intInc.click();
    await intInc.click();
    expect(await $('[data-testid="char-value-int"]').getText()).toBe('+2');

    // Virtues & Flaws tab: a minor virtue funded by a minor flaw is balanced.
    await $('[data-testid="tab-virtues_flaws"]').click();
    await $('[data-testid="add-virtue.keen_vision"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="add-virtue.keen_vision"]').click();
    await $('[data-testid="add-flaw.poor_student"]').click();
    // Match by testid prefix: the suffix is the entity-array index, which shifts
    // after canonical save/load reordering of selections.
    const removeKeenVision = await $('[data-testid^="remove-virtue.keen_vision"]');
    await removeKeenVision.waitForExist({ timeout: 5000 });

    // Abilities tab: give an XP pool, then buy Awareness up to 2 (15 xp).
    await $('[data-testid="tab-abilities"]').click();
    await $('[data-testid="xp-pool"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="xp-pool"]').setValue(30);
    await $('[data-testid="add-ability.awareness"]').click();
    const awarenessInc = await $('[data-testid="ability-inc-ability.awareness-0"]');
    await awarenessInc.waitForExist({ timeout: 5000 });
    await awarenessInc.click();
    await awarenessInc.click();
    expect(await $('[data-testid="ability-score-ability.awareness-0"]').getText()).toBe('2');

    // Save through the ARM_E2E_FILE seam, then confirm the JSON on disk.
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.schema_version).toBe(16);
    expect(saved.selections.some((s) => s.ref === 'virtue.keen_vision')).toBe(true);
    expect(saved.characteristics.int).toBe(2);
    expect(
      saved.ability_scores.some((a) => a.ability === 'ability.awareness' && a.score === 2),
    ).toBe(true);
    expect(saved.xp_pool).toBe(30);

    // Back on the V/F tab, drop the virtue (making the document dirty), then open
    // the saved file again. Opening a dirty document prompts to discard first, so
    // confirm that; the reload then repopulates the entity, proving Open works.
    await $('[data-testid="tab-virtues_flaws"]').click();
    await removeKeenVision.click();
    await runDocumentAction('open');
    await $('[data-testid="discard-confirm"]').waitForExist({ timeout: 10000 });
    await $('[data-testid="discard-confirm"]').click();
    await $('[data-testid^="remove-virtue.keen_vision"]').waitForExist({ timeout: 10000 });
  });

  it('shows the locked reason ABOVE the description on a non-takeable ability', async () => {
    // A companion has no Gift free Supernatural slot, so a Supernatural Ability
    // with no granting Virtue is locked (greyed). Its tooltip must show the
    // "requires a Virtue" REASON and, below it, the ability's normal description
    // — reason first, not instead of the description.
    await startCharacter('companion');
    await $('[data-testid="tab-abilities"]').click();
    const row = await $('[data-testid="add-ability.second_sight"]');
    await row.waitForExist({ timeout: 10000 });
    await browser.waitUntil(async () => await isRowBlocked(row), {
      timeout: 5000,
      timeoutMsg: 'Second Sight should be greyed for a companion',
    });
    // Dispatch mouseenter directly: synthetic events are focus-independent under
    // parallel wdio (the webview window may be blurred), unlike pointer moveTo.
    await browser.execute((el) => {
      el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
    }, row);
    const reason = await $('[data-testid="tooltip-reason"]');
    await reason.waitForExist({ timeout: 5000 });
    expect((await reason.getText()).trim().length).toBeGreaterThan(0);
    const desc = await $('[data-testid="tooltip-text"]');
    await desc.waitForExist({ timeout: 5000 });
    expect((await desc.getText()).trim().length).toBeGreaterThan(0);
    // Dismiss the popup so it does not linger into later specs.
    await browser.execute((el) => {
      el.dispatchEvent(new MouseEvent('mouseleave', { bubbles: true }));
    }, row);
  });
});

// End-to-end: per-character fields (Phase 7). A "Details" tab (always present)
// holds identity, age and a read-only Confidence readout, and a "Personality &
// Reputations" tab beside it holds those two — split out in Slice 3 so the
// editor's tabs mirror the wizard's phases (guided-creation review #28). The
// age → Ability cap, the Personality ±3/±6 range, the Reputation grant gate, and
// the Supernatural-Ability greying are all exercised.
//
// The `it` blocks run as one ordered narrative: the first creates the companion
// the rest go on editing, right through to the closing save round-trip. Only
// that first one calls `startCharacter`.
describe('character details', () => {
  const DETAILS_TAB = '[data-testid="tab-details"]';
  const PERSONALITY_TAB = '[data-testid="tab-personality_reputations"]';
  const VF_TAB = '[data-testid="tab-virtues_flaws"]';
  const ABILITIES_TAB = '[data-testid="tab-abilities"]';
  const CONFIDENCE = '[data-testid="confidence-readout"]';

  async function codeExists(code) {
    return (await $$(`[data-code="${code}"]`).length) > 0;
  }

  it('shows a Details tab with Confidence for a companion, hidden for a grog', async () => {
    // Grogs have no Confidence — the readout is absent for one entirely.
    await startCharacter('grog');
    await $(VF_TAB).waitForExist({ timeout: 30000 });
    await $(DETAILS_TAB).click();
    await browser.waitUntil(async () => !(await $(CONFIDENCE).isExisting()), {
      timeout: 5000,
      timeoutMsg: 'grog must not show a Confidence readout',
    });

    // A companion has it, at the default Score 1 / 3 points. This is also the
    // character the rest of the file goes on editing.
    await startCharacter('companion');
    await $(DETAILS_TAB).click();
    await $(CONFIDENCE).waitForExist({ timeout: 10000 });
    const text = clean(await $(CONFIDENCE).getText());
    expect(text).toContain('1');
    expect(text).toContain('3');
  });

  it('flags an Ability above the age cap', async () => {
    await $(DETAILS_TAB).click();
    await $('[data-testid="age-input"]').setValue('25'); // cap 5
    await $(ABILITIES_TAB).click();
    await $('[data-testid="add-ability.awareness"]').click();
    const inc = await $('[data-testid="ability-inc-ability.awareness-0"]');
    await inc.waitForExist({ timeout: 10000 });
    for (let i = 0; i < 6; i++) await inc.click(); // raise to 6, over the age-25 cap
    await browser.waitUntil(async () => codeExists('ability_above_age_cap'), {
      timeout: 5000,
      timeoutMsg: 'expected ability_above_age_cap for score 6 at age 25',
    });
  });

  it('enforces the Personality Trait range, widened by a Major Personality Flaw', async () => {
    await $(PERSONALITY_TAB).click();
    await $('[data-testid="personality-add"]').click();
    const inc = await $('[data-testid="personality-inc-0"]');
    for (let i = 0; i < 4; i++) await inc.click(); // +4, beyond the ±3 default
    await browser.waitUntil(async () => codeExists('personality_trait_out_of_range'), {
      timeout: 5000,
      timeoutMsg: 'a +4 trait should be out of range without a Major Personality Flaw',
    });

    // Pagan is a Major Personality Flaw; it lifts one trait to ±6.
    await $(VF_TAB).click();
    await $('[data-testid="add-flaw.pagan"]').click();
    await browser.waitUntil(async () => !(await codeExists('personality_trait_out_of_range')), {
      timeout: 5000,
      timeoutMsg: 'a Major Personality Flaw should allow the ±6 trait',
    });
  });

  it('greys a Supernatural Ability until the Gift opens a free slot', async () => {
    await $(ABILITIES_TAB).click();
    const secondSight = await $('[data-testid="add-ability.second_sight"]');
    await secondSight.waitForExist({ timeout: 10000 });
    // No Gift, no granting Virtue → locked.
    expect(await isRowBlocked(secondSight)).toBe(true);

    // Taking The Gift opens the one free Supernatural slot for a companion.
    await $(VF_TAB).click();
    await $('[data-testid="add-virtue.the_gift"]').click();
    await $(ABILITIES_TAB).click();
    await browser.waitUntil(
      async () => !(await isRowBlocked(await $('[data-testid="add-ability.second_sight"]'))),
      { timeout: 5000, timeoutMsg: 'The Gift should unlock one free Supernatural Ability' },
    );
  });

  it('offers Reputation input only once a granting Flaw is taken', async () => {
    await $(PERSONALITY_TAB).click();
    await expect($('[data-testid="reputation-empty"]')).toExist();

    // Infamous grants a Local Reputation. Findings 29/30: the grant IS the row —
    // there is no add button to press (pressing it twice used to make two
    // identical rows), so the description input exists as soon as the Flaw is
    // taken, and the row names the Flaw that put it there. The old spec drove
    // `reputation-add-local` and then typed into `reputation-content-0`; this
    // one types into the same input, having asserted that no click was needed to
    // conjure it. The saved-file assertion below is unchanged, so the
    // round-tripped row is still proven identical.
    await $(VF_TAB).click();
    await $('[data-testid="add-flaw.infamous"]').click();
    await $(PERSONALITY_TAB).click();
    const content = await $('[data-testid="reputation-content-0"]');
    await content.waitForExist({ timeout: 5000 });
    await expect($('[data-testid="reputation-add-local"]')).not.toExist();
    // The row says WHY it is there, by the Flaw's name and not its id.
    const source = await $('[data-testid="reputation-source-0"]').getText();
    expect(source).toContain('Infamous');
    expect(source).not.toContain('flaw.infamous');
    // No remove control on a granted row — the Flaw owns it, not the player.
    await expect($('[data-testid="reputation-remove-0"]')).not.toExist();

    await content.setValue('dragon slayer');
  });

  it('edits name and description in the header banner and concept on Details', async () => {
    // Name + short description live in the always-visible header banner.
    await $('[data-testid="identity-name"]').setValue('Marcus of Bonisagus');
    await $('[data-testid="identity-description"]').setValue('Knight of the Teutonic Order');
    // Concept is a multi-line textarea on the Details tab.
    await $(DETAILS_TAB).click();
    await $('[data-testid="identity-concept"]').setValue('A grim knight turned magus.');
  });

  it('round-trips the new fields through a save', async () => {
    if (fs.existsSync(e2eFile)) fs.unlinkSync(e2eFile);
    await runDocumentAction('save');
    await browser.waitUntil(() => fs.existsSync(e2eFile), {
      timeout: 10000,
      timeoutMsg: 'save did not write the file',
    });
    const saved = JSON.parse(fs.readFileSync(e2eFile, 'utf-8'));
    expect(saved.age).toBe(25);
    expect(saved.name).toBe('Marcus of Bonisagus');
    expect(saved.description).toBe('Knight of the Teutonic Order');
    expect(saved.concept).toBe('A grim knight turned magus.');
    expect(saved.personality_traits.length).toBeGreaterThan(0);
    expect(saved.reputations.some((r) => r.kind === 'local' && r.content === 'dragon slayer')).toBe(
      true,
    );
  });
});

// End-to-end: the window.close() bypass fix (N1 / round-2 GA2, E2), unsaved-changes
// half.
//
// WebKitGTK and WebView2 (wry's Linux and Windows backends) hard-wire the JS-visible
// `window.close()` DOM method to destroy the native window directly, bypassing
// `WindowEvent::CloseRequested` — and therefore the unsaved-changes guard — entirely
// (see `crates/arm-app/src/main.rs`'s `window_close_bridge_plugin` doc comment). The
// fix shadows `window.close` so it invokes the `request_close` command instead, which
// calls the real `WebviewWindow::close()` and is subject to the same confirmation as
// every other close path.
//
// This is NOT a real window-manager close (title-bar click, Alt+F4) — WebDriver
// cannot generate that event — but it does not need to be: `window.close()` called
// from page script is exactly the call the shadow intercepts, and is reachable from
// any future frontend code, which is the entire point of the fix.
//
// MUST BE THE LAST DESCRIBE IN THIS FILE: with unsaved edits, `guard_blocks_quit`
// raises a native GTK confirmation dialog that WebDriver cannot dismiss (native
// dialogs cannot be driven by WebDriver — see `ui/e2e/wdio.conf.js`'s note on
// save/load). That dialog is left open for the rest of this worker's app instance,
// so nothing may run after this describe in this file.
describe('window.close() bridge — unsaved changes', () => {
  const NAME_INPUT = '[data-testid="identity-name"]';
  const TAB_BAR = '[role="tablist"]';

  it('a dirty entity survives window.close() called from page script', async () => {
    await startCharacter('companion');
    await $(NAME_INPUT).setValue('Bridge test dirty');

    // Before the shadow existed, this call destroyed the window immediately, with
    // no chance for the guard — or this assertion — to run at all.
    await browser.execute(() => window.close());

    // The shadow invokes `request_close`, which calls the real
    // `WebviewWindow::close()` -> `WindowEvent::CloseRequested` -> `guard_blocks_quit`,
    // which raises the native confirmation dialog and calls `prevent_close()` while
    // the entity is dirty. Poll rather than a fixed sleep: confirm the window stays
    // alive and the edit survives once the IPC round trip has had time to land.
    await browser.waitUntil(async () => (await $(NAME_INPUT).getValue()) === 'Bridge test dirty', {
      timeout: 5000,
      interval: 250,
      timeoutMsg:
        'the edit did not survive window.close() with unsaved changes — the window may have been destroyed',
    });
    expect(await $(TAB_BAR).isExisting()).toBe(true);
  });
});
