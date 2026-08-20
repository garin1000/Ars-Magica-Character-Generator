// End-to-end: a MYTHIC COMPANION walked through the guided wizard from the
// startup screen to Finish (6b8d). The third of the four per-type walks; the
// shared driving lives in `e2e/wizard-walk.js`.
//
// The mythic companion is the only non-magus type with a step that hands the
// character a package rather than asking for a single value: `mythic_type` sits
// second in its flow, before Characteristics and before Virtues & Flaws, and
// picking a type seeds free grants, required Virtues and a required Flaw, and
// raises the point ceilings. So this walk is the one that proves a step whose
// choice rewrites two later steps still leaves the flow completable.
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

// Spirit Votary is the smallest of the four shipped types — two free grants
// (Spirit Votary, Second Sight), one required Virtue (Spiritual Pact) and one
// required Flaw (Pagan) — so the walk is about the flow rather than about
// untangling a seven-item package. Its +7 Flaw points leave the hand-picked Minor
// pair well inside the ceilings.
const MYTHIC_TYPE = 'mythic_type.spirit_votary';

const PLAN = {
  name: 'Sigrún of the Whispering Wood',
  concept: 'A votary who speaks for the spirits of a valley nobody else will enter.',
  mythicType: MYTHIC_TYPE,
  characteristics: { int: 2, sta: 2, per: 1 },
  flaws: ['flaw.disfigured'],
  virtues: ['virtue.keen_vision'],
  xpPool: '120',
  abilities: ['ability.awareness', 'ability.folk_ken'],
  personalityTrait: 'Reverent',
  age: '30',
};

describe('guided wizard: mythic companion', () => {
  it('walks a mythic companion through every declared phase to a complete, legal character', async () => {
    const phases = await walkEveryPhase('mythic_companion', PLAN);

    // The type step is the mythic companion's own, and the profile places it
    // BEFORE Virtues & Flaws — its package lands in that budget.
    expect(phases).toContain('mythic_type');
    expect(phases.indexOf('mythic_type')).toBeLessThan(phases.indexOf('virtues_flaws'));
    expect(phases).not.toContain('arts');

    await expectCompleteAndErrorFree();
    await finishIntoEditor();

    const saved = await saveAndReopen();
    expect(saved.type_id).toBe('mythic_companion');
    expect(saved.mythic_type).toBe(MYTHIC_TYPE);

    // SAVES STORE CHOICES, NOT RESOLVED VALUES. The type's required Virtue and
    // Flaw are seeded as ordinary bought selections, because a player may swap the
    // required Flaw for a substitute — they are choices. Its *free grants* (Spirit
    // Votary itself, Second Sight) are not written down at all: they follow from
    // `mythic_type` and are re-derived on load, exactly like a House's free Virtue.
    const refs = saved.selections.map((selection) => selection.ref);
    expect(refs.sort()).toEqual([
      'flaw.disfigured',
      'flaw.pagan',
      'virtue.keen_vision',
      'virtue.spiritual_pact',
    ]);

    expect(await $('[data-testid="identity-name"]').getValue()).toBe(PLAN.name);
  });
});
