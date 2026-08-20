// End-to-end: a GROG walked through the guided wizard from the startup screen to
// Finish — the milestone's own promise, checked on the shipped binary (6b8d).
//
// A grog declares the shortest flow of the four types: concept, type,
// characteristics, virtues & flaws, abilities, aging. No Arts, no spells, no
// House, no Personality Traits — the wizard's rail is the profile, so the walk
// asserts exactly that rather than a list written down here.
//
// What this spec claims, and no earlier spec could: the character that comes out
// is COMPLETE (every declared phase holds a choice, by the engine's own
// completeness report) as well as legal (no error-severity finding anywhere), and
// it survives a save and a reload unchanged.
//
// NOTE: requires the production binary; the display comes from your desktop
// session or, when DISPLAY is unset, the Xvfb one WebdriverIO starts
// (see e2e/README.md). The wdio `onPrepare` hook builds `target/release/arm-app`.

import { $, expect } from '@wdio/globals';

import {
  expectCompleteAndErrorFree,
  finishIntoEditor,
  saveAndReopen,
  walkEveryPhase,
} from '../wizard-walk.js';

// A grog's budget is 3 points of Virtues against 3 of Flaws, and its permitted
// categories are general/personality/social status — so one Minor general Flaw
// funding one Minor general Virtue is the smallest legal, fully balanced pick.
const PLAN = {
  name: 'Aldus the Gatekeeper',
  concept: 'A turb sergeant with sharp eyes and a ruined face.',
  // 3 + 3 + 1 = the ruleset's 7 starting Characteristic points, spent exactly.
  characteristics: { int: 2, sta: 2, per: 1 },
  flaws: ['flaw.disfigured'],
  virtues: ['virtue.keen_vision'],
  xpPool: '75',
  abilities: ['ability.awareness', 'ability.athletics'],
  age: '25',
};

describe('guided wizard: grog', () => {
  it('walks a grog through every declared phase to a complete, legal character', async () => {
    const phases = await walkEveryPhase('grog', PLAN);

    // The type's flow is a subset: a grog has no Arts, spells or House to choose,
    // and the rail must not offer steps its profile never declared.
    expect(phases).not.toContain('arts');
    expect(phases).not.toContain('spells');
    expect(phases).not.toContain('house_specialisation');

    await expectCompleteAndErrorFree();
    await finishIntoEditor();
    expect(await $('[data-testid="identity-name"]').getValue()).toBe(PLAN.name);

    const saved = await saveAndReopen();
    expect(saved.type_id).toBe('grog');
    expect(saved.name).toBe(PLAN.name);
    expect(saved.age).toBe(25);
    expect(saved.characteristics.int).toBe(2);
    expect(saved.selections.map((s) => s.ref).sort()).toEqual([
      'flaw.disfigured',
      'virtue.keen_vision',
    ]);
    expect(saved.ability_scores.map((a) => a.ability).sort()).toEqual([
      'ability.athletics',
      'ability.awareness',
    ]);

    // Reloaded from those choices alone, the character is still the one that was
    // built: the wizard stored decisions, not resolved values.
    expect(await $('[data-testid="identity-name"]').getValue()).toBe(PLAN.name);
  });
});
