// Guided-wizard step navigation: which step is current, how far the player has
// advanced, and whether the rail may move forward. Extracted out of `AppStore`
// (VA6) as a self-contained module — it never touches the entity, only reads
// the ruleset/entity-type/validation-result it is given through `host`, so the
// caller (AppStore) stays the single owner of that shared state.

import {
  firstBlockedPhaseIndex,
  incompletePhases,
  phaseHasBlockingIssue,
  wizardPhases,
} from './derive';
import type { CreationPhase, LocalizedRuleset, ValidationResult } from './types';

/** The read-only slice of `AppStore` the wizard rail needs, as live accessors
 * (not values) so the getters below always read the host's *current* state —
 * exactly as if they were still declared directly on `AppStore`. */
export interface WizardNavigationHost {
  ruleset: () => LocalizedRuleset | null;
  entityTypeId: () => string;
  result: () => ValidationResult | null;
}

export class WizardNavigation {
  #host: WizardNavigationHost;

  constructor(host: WizardNavigationHost) {
    this.#host = host;
  }

  /**
   * Index of the wizard's current step within {@link phases}.
   *
   * Navigation state, deliberately not part of the entity: moving through the
   * flow is not an edit, so it never dirties the document and a save records no
   * progress through it.
   */
  step = $state(0);

  /**
   * The furthest step reached by advancing. Raised only by {@link next}, never
   * lowered by going back, so the rail can offer every step the user has
   * already seen while still refusing to skip ahead into unseen ones.
   */
  furthest = $state(0);

  /**
   * The wizard's steps for the current character: the type profile's own
   * ordered phases, then the terminal `review` step. Empty when no profile is
   * loaded.
   *
   * A plain getter rather than `$derived`: it is read through `this.#host`,
   * which is only assigned in the constructor body, and class field
   * initializers (where a `$derived` value would live) all run *before* that
   * assignment regardless of source order. A getter defers the read to call
   * time, well after construction, and stays just as reactive — Svelte tracks
   * the underlying `$state` reads made *during* the call, not which construct
   * performed the call.
   */
  get phases(): CreationPhase[] {
    return wizardPhases(this.#host.ruleset()?.ruleset.type_profiles[this.#host.entityTypeId()]);
  }

  /** The phase the wizard is currently on. */
  get phase(): CreationPhase {
    return this.phases[this.step] ?? 'review';
  }

  /**
   * Whether the wizard may advance: the current phase carries no error.
   *
   * Errors only, so a warning never gates — which means a phase can be legal but
   * empty (a magus may pass the House step with no House, since `house_unset` is
   * an advisory). In Advisory mode the engine downgrades every error to a warning
   * and in Silent mode it reports none, so in both the wizard stops gating
   * entirely; the validation-mode control is the intended escape hatch.
   */
  get canAdvance(): boolean {
    return !phaseHasBlockingIssue(this.#host.result()?.issues ?? [], this.phase);
  }

  /**
   * Whether the wizard may finish: no error remains anywhere in the character.
   *
   * Deliberately wider than {@link canAdvance}: the findings no creation phase
   * owns (equipment, Might, Warping) gate no single step, and a phase the
   * character type never declares has no step at all — Finish is where both
   * still have to be answered.
   */
  get canFinish(): boolean {
    return !(this.#host.result()?.issues ?? []).some((i) => i.severity === 'error');
  }

  /**
   * The steps of this flow the player has recorded nothing for, in rail order,
   * as the engine reports them.
   *
   * Purely informational, and deliberately kept out of {@link canAdvance} and
   * {@link canFinish}: legal is not the same as finished, so an empty step is
   * marked, never blocked.
   */
  get incompletePhases(): CreationPhase[] {
    return incompletePhases(this.#host.result());
  }

  /** Whether the step currently on screen is one of those. */
  get phaseIncomplete(): boolean {
    return this.incompletePhases.includes(this.phase);
  }

  /** Advance one step, unless the current phase holds an error. */
  next(): void {
    if (!this.canAdvance) return;
    if (this.step >= this.phases.length - 1) return;
    this.step += 1;
    this.furthest = Math.max(this.furthest, this.step);
  }

  /**
   * Step back one. Never gated: the user must always be able to reach the step
   * that needs fixing, including the one they have just broken.
   */
  back(): void {
    this.step = Math.max(this.step - 1, 0);
  }

  /**
   * Jump to an already-visited step from the rail.
   *
   * Forward jumps are held to the same gate as Next: the jump clamps to the
   * first blocking phase between here and there, **including the step being
   * left**. Otherwise the rail would be a way around the very gate that blocks
   * Next. Backward jumps are free, like {@link back}.
   */
  goTo(step: number): void {
    if (step < 0 || step > this.furthest) return;
    if (step <= this.step) {
      this.step = step;
      return;
    }
    const blocked = firstBlockedPhaseIndex(
      this.phases,
      this.#host.result()?.issues ?? [],
      this.step,
      step,
    );
    this.step = blocked ?? step;
  }

  /** Send the rail back to the first step. */
  reset(): void {
    this.step = 0;
    this.furthest = 0;
  }
}
