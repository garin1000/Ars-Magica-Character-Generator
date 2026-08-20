// End-to-end: a MAGUS walked through the guided wizard from the startup screen to
// Finish (6b8d). The last and longest of the four per-type walks; the shared
// driving lives in `e2e/wizard-walk.js`.
//
// Ten declared phases, three of which no other type has: House & specialisation
// (placed before Virtues & Flaws, because its free Minor Virtue lands in that
// budget), Arts, and Spells. It is also the only type the Order holds to minimum
// Abilities — Parma Magica 1, Magic Theory 1 and a dead language 1 are blocking
// findings for every magus (Core Rules.md:2437) — so the Abilities step here has
// to settle a real error rather than merely record a choice.
//
// `wizard.e2e.js` already drives a magus through the flow's MACHINERY (gating,
// back/forward, the validation mode that lifts the gate, the incompleteness
// mark). This spec drives it to the other end instead: every phase filled, no
// error left standing, nothing still marked incomplete.
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

// House Tytalus grants one fixed free Minor Virtue (Self-Confident) and leaves no
// specialisation choice open, so the House step is a single select — the point
// here is that the step is reachable and its choice sticks, not the grant
// machinery, which `houses.e2e.js` drives in full.
const HOUSE = 'house.tytalus';

// Creo 3 + Corpus 3, with Int +2 and Magic Theory 1, puts the Te+Fo+Int+MT+3 cap
// (Core Rules.md:2465) at 12 — enough for Bind Wound, a level 10 Creo Corpus spell
// and not a ritual. The 120 spell levels a magus starts with cover it many times.
const SPELL = 'spell.bind_wound';

const PLAN = {
  name: 'Marcus of Tytalus',
  concept: 'A young magus who argues with everyone, on principle.',
  house: HOUSE,
  characteristics: { int: 2, sta: 2, per: 1 },
  // A Hermetic Flaw funds the Minor Virtue and settles the guideline that every
  // magus should carry one (`missing_hermetic_flaw`, a warning).
  flaws: ['flaw.clumsy_magic'],
  virtues: ['virtue.keen_vision'],
  // Two Arts at 3 cost 30 experience each off the advancement table, three
  // Abilities at 1 cost 5 each; 150 leaves the shared Ability+Art pool in credit.
  magusMinimums: true,
  xpPool: '150',
  abilities: ['ability.awareness'],
  arts: { 'art.creo': 3, 'art.corpus': 3 },
  spells: [SPELL],
  personalityTrait: 'Contrary',
  age: '25',
};

describe('guided wizard: magus', () => {
  it('walks a magus through every declared phase to a complete, legal character', async () => {
    const phases = await walkEveryPhase('magus', PLAN);

    // The magus-only steps, and the placement that matters: the House step comes
    // before Virtues & Flaws because its free Minor Virtue is spent in that budget.
    expect(phases).toContain('arts');
    expect(phases).toContain('spells');
    expect(phases.indexOf('house_specialisation')).toBeLessThan(phases.indexOf('virtues_flaws'));

    await expectCompleteAndErrorFree();
    await finishIntoEditor();

    const saved = await saveAndReopen();
    expect(saved.type_id).toBe('magus');
    expect(saved.house).toBe(HOUSE);
    expect(saved.age).toBe(25);
    expect(saved.art_scores).toEqual([
      { art: 'art.corpus', score: 3 },
      { art: 'art.creo', score: 3 },
    ]);
    expect(saved.spells.map((spell) => spell.spell)).toEqual([SPELL]);
    // The Order's minimums, bought on the Abilities step, are in the save as
    // ordinary Ability rows — the dead language named as the instance it is.
    const abilities = saved.ability_scores.map((entry) => entry.ability);
    expect(abilities).toContain('ability.parma_magica');
    expect(abilities).toContain('ability.magic_theory');
    expect(abilities).toContain('ability.dead_language');

    expect(await $('[data-testid="identity-name"]').getValue()).toBe(PLAN.name);
  });
});
