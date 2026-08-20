// End-to-end: a COMPANION walked through the guided wizard from the startup
// screen to Finish (6b8d). The second of the four per-type walks; the shared
// driving lives in `e2e/wizard-walk.js`.
//
// A companion adds one step to the grog's flow — Personality Traits and
// Reputations — and trebles the Virtue/Flaw budget. Everything else is the same
// shape, which is exactly why the phase list comes from the profile rather than
// from a list written here.
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

// 10 points of Flaws buy 10 of Virtues for a companion, so one Minor Flaw funding
// one Minor Virtue sits well inside the budget rather than against its ceiling —
// this walk is about the flow being completable, not about the balance check,
// which `vf-effects` and `wizard` already drive.
const PLAN = {
  name: 'Isabeau of Aquitaine',
  concept: 'A physician who travels with the covenant and asks too many questions.',
  characteristics: { int: 2, sta: 2, per: 1 },
  flaws: ['flaw.disfigured'],
  virtues: ['virtue.keen_vision'],
  xpPool: '120',
  abilities: ['ability.awareness', 'ability.folk_ken'],
  personalityTrait: 'Curious',
  age: '30',
};

describe('guided wizard: companion', () => {
  it('walks a companion through every declared phase to a complete, legal character', async () => {
    const phases = await walkEveryPhase('companion', PLAN);

    // The step a grog does not get, and the magus-only ones a companion does not.
    expect(phases).toContain('personality_reputations');
    expect(phases).not.toContain('arts');
    expect(phases).not.toContain('house_specialisation');

    await expectCompleteAndErrorFree();
    await finishIntoEditor();

    const saved = await saveAndReopen();
    expect(saved.type_id).toBe('companion');
    expect(saved.name).toBe(PLAN.name);
    expect(saved.age).toBe(30);
    // The step's own choice: a named trait at +3, stored as the choice it is.
    expect(saved.personality_traits).toEqual([{ name: PLAN.personalityTrait, value: 3 }]);
    // A Reputation was deliberately NOT recorded: one needs a Virtue that grants
    // it, so the step counts as engaged on the trait alone.
    expect(saved.reputations ?? []).toEqual([]);

    expect(await $('[data-testid="identity-name"]').getValue()).toBe(PLAN.name);
  });
});
