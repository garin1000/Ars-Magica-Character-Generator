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
 * swap the engine's reported findings mid-run without rebuilding the host. */
function makeHost(issues: ValidationIssue[]): WizardNavigationHost {
  const p = profile(['concept', 'virtues_flaws', 'experience', 'abilities', 'arts']);
  const rs = rulesetWith(p);
  return {
    ruleset: () => rs,
    entityTypeId: () => p.id,
    result: () => ({ issues }),
    phasesInForce: () => null,
    recordFurthestPhase: () => {},
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

  it('goTo clamps a forward rail jump at the owning phase, not earlier', () => {
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
