// End-to-end: the four per-type guided-wizard walks — grog, companion, mythic
// companion, magus — each driven from the startup screen to Finish by the shared
// `walkEveryPhase` in `e2e/wizard-walk.js`, plus the window.close()-adjacent
// `RunEvent::ExitRequested` bridge for a CLEAN document.
//
// A2 merged these five previously separate spec files into one: the four walks
// are structurally the same file (only the `plan` differs), so they were the
// cheapest merge in the whole consolidation. Each describe keeps its own local
// `PLAN` (and, where relevant, `HOUSE`/`SPELL`/`MYTHIC_TYPE`) scoped to its
// block, since every walk uses those same names for a different value.
//
// `RunEvent::ExitRequested bridge — no unsaved changes` MUST stay last: a clean
// quit calls `app.exit(0)` and ends this worker's WebDriver session along with
// the process, so nothing may run after it in this file.

import { $, browser, expect } from '@wdio/globals';

import { startCharacter } from '../helpers.js';
import {
  expectCompleteAndErrorFree,
  finishIntoEditor,
  saveAndReopen,
  walkEveryPhase,
} from '../wizard-walk.js';

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
describe('guided wizard: grog', () => {
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
    // #33 gave grogs the `personality_reputations` phase, which their profile had been
    // missing. 'Loyal' is the rules' own example for the type: "'Loyal' is a
    // particularly important Trait, as it reflects the grog's attachment to the
    // covenant" (Core Rules.md:1075) — the same passage that says a grog's traits
    // matter more than a magus's, which is why leaving the phase out was backwards.
    personalityTrait: 'Loyal',
    age: '25',
  };

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

// End-to-end: a COMPANION walked through the guided wizard from the startup
// screen to Finish (6b8d).
//
// A companion adds one step to the grog's flow — Personality Traits and
// Reputations — and trebles the Virtue/Flaw budget. Everything else is the same
// shape, which is exactly why the phase list comes from the profile rather than
// from a list written here.
describe('guided wizard: companion', () => {
  // 10 points of Flaws buy 10 of Virtues for a companion, so one Minor Flaw funding
  // one Minor Virtue sits well inside the budget rather than against its ceiling —
  // this walk is about the flow being completable, not about the balance check,
  // which `magus-editor.e2e.js`'s `phase-3 virtue/flaw effects` describe and
  // `wizard-flow.e2e.js`'s `wizard` describe already drive.
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

// End-to-end: a MYTHIC COMPANION walked through the guided wizard from the
// startup screen to Finish (6b8d).
//
// The mythic companion is the only non-magus type with a step that hands the
// character a package rather than asking for a single value: `mythic_type` sits
// second in its flow, before Characteristics and before Virtues & Flaws, and
// picking a type seeds free grants, required Virtues and a required Flaw that
// spend the same 10 Flaw / 20 Virtue budget every Mythic Companion gets. So this
// walk is the one that proves a step whose choice rewrites two later steps still
// leaves the flow completable.
describe('guided wizard: mythic companion', () => {
  // Spirit Votary is the smallest of the four shipped types — two free grants
  // (Spirit Votary, Second Sight), one required Virtue (Spiritual Pact) and one
  // required Flaw (Pagan) — so the walk is about the flow rather than about
  // untangling a seven-item package. Its package plus the hand-picked Minor pair
  // sit well inside the 10 Flaw / 20 Virtue ceilings (4 F funding 4 V).
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

// End-to-end: a MAGUS walked through the guided wizard from the startup screen to
// Finish (6b8d). The last and longest of the four per-type walks.
//
// Ten declared phases, three of which no other type has: House & specialisation
// (placed before Virtues & Flaws, because its free Minor Virtue lands in that
// budget), Arts, and Spells. It is also the only type the Order holds to minimum
// Abilities — Parma Magica 1, Magic Theory 1 and a dead language 1 are blocking
// findings for every magus (Core Rules.md:2437) — so the Abilities step here has
// to settle a real error rather than merely record a choice.
//
// `wizard-flow.e2e.js`'s `wizard` describe already drives a magus through the
// flow's MACHINERY (gating, back/forward, the validation mode that lifts the
// gate, the incompleteness mark). This spec drives it to the other end instead:
// every phase filled, no error left standing, nothing still marked incomplete.
describe('guided wizard: magus', () => {
  // House Tytalus grants one fixed free Minor Virtue (Self-Confident) and leaves no
  // specialisation choice open, so the House step is a single select — the point
  // here is that the step is reachable and its choice sticks, not the grant
  // machinery, which `magus-editor.e2e.js`'s `hermetic houses` describe drives in
  // full.
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

// End-to-end: the no-unsaved-changes half of the `RunEvent::ExitRequested`
// bridge — the companion case to `grog-wizard-aging.e2e.js`'s
// `RunEvent::ExitRequested bridge — unsaved changes` describe (read that
// describe's header for the full context on why this exists and what it does and
// does not prove).
//
// `guard_blocks_quit` (`crates/arm-app/src/main.rs`) returns `false` — allow —
// when the entity is not dirty, so `request_exit` must actually call
// `app.exit(0)` and end the process. Without this half, a "fix" that made
// `request_exit` an unconditional no-op (rather than actually routing to
// `guard_blocks_quit`/`app.exit(0)`) would pass the dirty-side spec vacuously —
// the app would never quit either way — and never be caught.
//
// MUST BE THE LAST DESCRIBE IN THIS FILE: a clean quit ends this app instance
// and its WebDriver session, so nothing may run after it in the same worker.
describe('RunEvent::ExitRequested bridge — no unsaved changes', () => {
  it('a clean entity actually quits on a request_exit IPC call', async () => {
    // Creating a character is not itself a dirty edit ("a freshly created
    // character is not dirty" — `ui/src/lib/state.svelte.ts`), so no confirmation
    // dialog stands in the way.
    await startCharacter('grog');

    // No dialog blocks it, so the process should actually exit — and the WebDriver
    // session riding on it along with it. The teardown can land *during* this call
    // rather than after it, exactly as in `app-shell.e2e.js`'s
    // `window.close() bridge — no unsaved changes` describe: the session dies
    // while `execute` is still in flight, so the call itself rejects. That
    // rejection is not a test failure, it IS the evidence the quit happened.
    let sessionGone = false;
    try {
      await browser.execute(() => window.__TAURI_INTERNALS__.invoke('request_exit'));
    } catch {
      sessionGone = true;
    }

    if (!sessionGone) {
      // The exit can also complete just after the call returns, so fall back to
      // polling until the session stops answering.
      await browser.waitUntil(
        async () => {
          try {
            await browser.getWindowHandles();
            return false;
          } catch {
            return true;
          }
        },
        {
          timeout: 10000,
          timeoutMsg: 'request_exit did not quit the app when the document was clean',
        },
      );
    }
  });
});
