// review-final.json finding #1 (MAJOR): the wizard dead end a V/F item's
// AbilityMin/AbilityCategoryScoreMin/ArtMin/AnyArtMin prerequisite created —
// `canAdvance`/`goTo` block forward progress on an error attributed to the
// CURRENT phase (virtues_flaws) when the data that can actually satisfy it
// only arrives on a LATER phase (abilities/arts). The fix is entirely
// engine-side (`crates/arm-rules/src/validation/prereq.rs`'s
// `prereq_resolution_phase`, which now attributes such an issue's `phase` to
// Abilities/Arts instead of VirtuesFlaws); `WizardNavigation`'s own gating
// logic was already correct given correct input (it keys purely on
// `issue.phase` vs. the step being asked about), so these are regression/
// contract tests pinning that contract, not a RED-before-fix cycle at this
// layer — the RED lived in the Rust engine tests
// (`crates/arm-rules/tests/d81_exclusions.rs`,
// `crates/arm-rules/tests/x5b_ability_minimums.rs`).
//
// This is also the first test file `WizardNavigation` has ever had (the
// finding's own point: "this UX consequence was never caught").

import { describe, expect, it } from 'vitest';
import { WizardNavigation, type WizardNavigationHost } from './wizard-navigation.svelte';
import type { CreationPhase, EntityTypeProfile, LocalizedRuleset, ValidationIssue } from './types';

function profile(phases: CreationPhase[]): EntityTypeProfile {
  return {
    id: 'magus',
    budget: { virtue_points: 10, flaw_points: 10 },
    creation_phases: phases,
  };
}

function rulesetWith(p: EntityTypeProfile): LocalizedRuleset {
  return {
    ruleset: {
      id: 'test',
      version: '1',
      point_items: {},
      type_profiles: { [p.id]: p },
      magnitude_points: { free: 0, minor: 1, major: 3 },
      ability_category_order: [],
    },
    i18n: {},
  };
}

function issue(
  code: string,
  phase: CreationPhase,
  context = 'flaw.broken_vessel',
): ValidationIssue {
  return { severity: 'error', code, phase, args: {}, context };
}

/** A magus wizard with the real phase order `wizardPhases` resolves for the
 * `magus` profile (`rules/core/character_types.json`): `virtues_flaws` sits
 * well before both `abilities` and `arts`. `issues` is mutable so a test can
 * swap the engine's reported findings mid-run without rebuilding the host.
 * `recorded` collects every phase the navigation reports as newly reached. */
function makeHost(issues: ValidationIssue[], recorded: CreationPhase[] = []): WizardNavigationHost {
  const p = profile(['concept', 'virtues_flaws', 'experience', 'abilities', 'arts']);
  const rs = rulesetWith(p);
  return {
    ruleset: () => rs,
    entityTypeId: () => p.id,
    result: () => ({ issues }),
    phasesInForce: () => null,
    recordFurthestPhase: (phase) => recorded.push(phase),
  };
}

describe('WizardNavigation / a prereq issue attributed to a LATER phase (the fix)', () => {
  // The engine's fixed behavior: `flaw.broken_vessel`'s unmet Any[AbilityCategoryScoreMin,
  // AnyArtMin] is reported on `arts` (the later of the two phases that can
  // satisfy it), not on `virtues_flaws` (the phase the item was selected on).
  const fixedIssues = [issue('prereq_not_met', 'arts')];

  it('canAdvance is true on virtues_flaws: the issue belongs to a later phase', () => {
    const nav = new WizardNavigation(makeHost(fixedIssues));
    nav.step = nav.phases.indexOf('virtues_flaws');
    expect(nav.canAdvance).toBe(true);
  });

  it('next() walks all the way to arts despite the still-unresolved issue', () => {
    const nav = new WizardNavigation(makeHost(fixedIssues));
    nav.step = nav.phases.indexOf('virtues_flaws');
    nav.next(); // -> experience
    nav.next(); // -> abilities
    nav.next(); // -> arts
    expect(nav.phase).toBe('arts');
  });

  it('canAdvance is false once the owning phase (arts) is actually reached', () => {
    const nav = new WizardNavigation(makeHost(fixedIssues));
    nav.step = nav.phases.indexOf('arts');
    expect(nav.canAdvance).toBe(false);
  });

  it('a forward rail jump from virtues_flaws reaches the owning phase (arts)', () => {
    const nav = new WizardNavigation(makeHost(fixedIssues));
    nav.furthest = nav.phases.indexOf('arts');
    nav.step = nav.phases.indexOf('virtues_flaws');
    nav.goTo(nav.phases.indexOf('arts'));
    expect(nav.phase).toBe('arts');
  });

  it('is cleared once the issue resolves (e.g. an Art is bought) and Finish is reachable', () => {
    const nav = new WizardNavigation(makeHost([]));
    nav.step = nav.phases.indexOf('arts');
    expect(nav.canAdvance).toBe(true);
    expect(nav.canFinish).toBe(true);
  });
});

describe("WizardNavigation / the pre-fix shape, for contrast (today's engine before the fix)", () => {
  // Documents the bug this finding reports: with the issue hard-coded to
  // `virtues_flaws` (every `prereq_not_met` issue's phase before this fix),
  // the player cannot leave the V/F step at all — a dead end, since the only
  // phases that could ever satisfy the prerequisite (abilities/arts) are
  // later and so never reached.
  const preFixIssues = [issue('prereq_not_met', 'virtues_flaws')];

  it('canAdvance is false on virtues_flaws — the player cannot leave the step', () => {
    const nav = new WizardNavigation(makeHost(preFixIssues));
    nav.step = nav.phases.indexOf('virtues_flaws');
    expect(nav.canAdvance).toBe(false);
  });

  it('back() still escapes to fix it (never a PERMANENT lock) by deselecting the item', () => {
    // `back()` is always ungated (`WizardNavigation.back`'s own doc comment),
    // so the player is never physically trapped — only unable to progress
    // forward while the Flaw stays selected. Pinning this keeps the "dead
    // end" finding precise: it is "no path forward with the item still
    // selected," not "no path at all."
    const nav = new WizardNavigation(makeHost(preFixIssues));
    nav.step = nav.phases.indexOf('virtues_flaws');
    nav.back();
    expect(nav.step).toBe(nav.phases.indexOf('concept'));
  });
});

// tryout-findings-2026-10-03 #9(a) (HIGH): Abilities, Arts and Spell Mastery draw on
// ONE experience pool, and the engine files the pool's `not_enough_xp` on the
// Abilities step. So overspending on Arts put an error on Abilities, and the old
// gate then held the player on Abilities: Next was shut, and a rail click to the
// already-visited Arts step was clamped back to Abilities. The only way out was to
// lower Abilities, go to Arts, lower Arts, come back and re-raise Abilities.
//
// The rule now: a step already visited (index <= `furthest`) is reachable in both
// directions, by Next and by the rail, whatever errors stand. A blocking error
// gates only a move PAST `furthest`, onto a step never reached. Finish keeps its
// own, wider gate (`canFinish`: no error anywhere).
describe('WizardNavigation / a visited step stays reachable while an error stands (#9a)', () => {
  const overspend = [issue('not_enough_xp', 'abilities', '')];

  /** The repro: Abilities and Arts both visited, the player back on Abilities. */
  function backOnAbilities(issues: ValidationIssue[], recorded: CreationPhase[] = []) {
    const nav = new WizardNavigation(makeHost(issues, recorded));
    nav.furthest = nav.phases.indexOf('arts');
    nav.step = nav.phases.indexOf('abilities');
    return nav;
  }

  it('canAdvance is true on a blocked step when the next step was already visited', () => {
    const nav = backOnAbilities(overspend);
    expect(nav.canAdvance).toBe(true);
  });

  it('next() moves from the blocked Abilities step onto the visited Arts step', () => {
    const nav = backOnAbilities(overspend);
    nav.next();
    expect(nav.phase).toBe('arts');
  });

  it('a rail jump from the blocked Abilities step lands on the visited Arts step', () => {
    const nav = backOnAbilities(overspend);
    nav.goTo(nav.phases.indexOf('arts'));
    expect(nav.phase).toBe('arts');
  });

  it('a rail jump forward crosses a blocked step in between, up to furthest', () => {
    const nav = new WizardNavigation(makeHost([issue('unbalanced_virtues', 'virtues_flaws')]));
    nav.furthest = nav.phases.indexOf('arts');
    nav.step = nav.phases.indexOf('concept');
    nav.goTo(nav.phases.indexOf('arts'));
    expect(nav.phase).toBe('arts');
  });

  it('next() walks across a blocked step in between, up to furthest', () => {
    const nav = new WizardNavigation(makeHost([issue('unbalanced_virtues', 'virtues_flaws')]));
    nav.furthest = nav.phases.indexOf('arts');
    nav.step = nav.phases.indexOf('virtues_flaws');
    nav.next(); // -> experience
    nav.next(); // -> abilities
    nav.next(); // -> arts
    expect(nav.phase).toBe('arts');
  });

  it('moving within visited ground records no new progress', () => {
    const recorded: CreationPhase[] = [];
    const nav = backOnAbilities(overspend, recorded);
    nav.next();
    expect(nav.furthest).toBe(nav.phases.indexOf('arts'));
    expect(recorded).toEqual([]);
  });

  it('still refuses to advance past furthest from a step holding an error', () => {
    const nav = new WizardNavigation(makeHost(overspend));
    nav.furthest = nav.phases.indexOf('abilities');
    nav.step = nav.phases.indexOf('abilities');
    expect(nav.canAdvance).toBe(false);
    nav.next();
    expect(nav.phase).toBe('abilities');
  });

  it('reaches furthest by next(), then stops there while that step holds an error', () => {
    const nav = backOnAbilities([issue('not_enough_xp', 'abilities', ''), issue('x', 'arts', '')]);
    nav.next(); // -> arts, already visited
    expect(nav.phase).toBe('arts');
    expect(nav.canAdvance).toBe(false);
    nav.next(); // review was never reached, and arts holds an error
    expect(nav.phase).toBe('arts');
  });

  it('still refuses a rail jump past furthest', () => {
    const nav = new WizardNavigation(makeHost([]));
    nav.furthest = nav.phases.indexOf('abilities');
    nav.step = nav.phases.indexOf('abilities');
    nav.goTo(nav.phases.indexOf('arts'));
    expect(nav.phase).toBe('abilities');
  });

  it('keeps Finish shut while the error stands, wherever the player is', () => {
    const nav = backOnAbilities(overspend);
    nav.furthest = nav.phases.indexOf('review');
    nav.goTo(nav.phases.indexOf('review'));
    expect(nav.phase).toBe('review');
    expect(nav.canFinish).toBe(false);
  });
});

// W2 (tryout-findings-2026-10-03 #9b, #11): the engine now lists every step that
// spends the shared pool in the overspend's `also_phases`, so an overspend made on
// Arts holds Arts itself. With #9a's rule that must gate only the move past
// `furthest` — never trap the player on visited ground.
describe('WizardNavigation / the shared-pool overspend also blocks Arts (#9b)', () => {
  const sharedOverspend: ValidationIssue[] = [
    { ...issue('not_enough_xp', 'abilities', ''), also_phases: ['arts', 'spells'] },
  ];

  function onArtsAsFurthest(): WizardNavigation {
    const nav = new WizardNavigation(makeHost(sharedOverspend));
    nav.furthest = nav.phases.indexOf('arts');
    nav.step = nav.phases.indexOf('arts');
    return nav;
  }

  it('holds Next shut on Arts when Arts is the furthest step reached', () => {
    const nav = onArtsAsFurthest();
    expect(nav.canAdvance).toBe(false);
    nav.next();
    expect(nav.phase).toBe('arts');
  });

  it('does not trap the player: Back to Abilities, then Next onto Arts again', () => {
    const nav = onArtsAsFurthest();
    nav.back();
    expect(nav.phase).toBe('abilities');
    expect(nav.canAdvance).toBe(true);
    nav.next();
    expect(nav.phase).toBe('arts');
  });
});
