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
  /**
   * Record the furthest phase reached, so a save carries the flow's progress
   * (#31). A callback rather than a write from here: the entity stays the host's
   * to own, which is what keeps this module free of it.
   */
  recordFurthestPhase: (phase: CreationPhase) => void;
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
   * Whether this run is exempt from the flow's gates: every step reachable, no
   * forward jump clamped, no Next or Finish held shut, findings still shown.
   *
   * Set only by {@link restore}, and only for a character the wizard never built —
   * one with no stored phase slug, or one whose slug this build no longer declares
   * (#31). Such a character was assembled in the editor and never passed these
   * gates in the first place, so holding it to them would lock the very steps it
   * needs to reach in order to be fixed.
   */
  ungated = $state(false);

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
    if (this.ungated) return true;
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
    if (this.ungated) return true;
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

  /**
   * Advance one step, unless the current phase holds an error.
   *
   * The only place {@link furthest} rises, and therefore the only place the
   * progress written onto the entity changes (#31). That is deliberate: `back()`
   * and {@link goTo} leave both alone, so browsing the rail costs nothing, and a
   * document is marked changed only by a step deliberately advanced past — even
   * one left empty, since a step can be legally empty.
   */
  next(): void {
    if (!this.canAdvance) return;
    if (this.step >= this.phases.length - 1) return;
    this.step += 1;
    if (this.step <= this.furthest) return;
    this.furthest = this.step;
    const reached = this.phases[this.furthest];
    if (reached) this.#host.recordFurthestPhase(reached);
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
    if (step <= this.step || this.ungated) {
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

  /** Send the rail back to the first step, gates on. */
  reset(): void {
    this.step = 0;
    this.furthest = 0;
    this.ungated = false;
  }

  /**
   * Open the rail for a character that already exists, from the furthest phase its
   * document recorded (#31).
   *
   * Two outcomes, and the branch is decided by whether `storedPhase` names a step
   * of THIS rail:
   *
   * - **It does** — the character was built by the wizard. Land on that phase and
   *   make it the ceiling, so the run resumes exactly as it was left: nothing past
   *   it was ever reached, and a forward jump clamps at the first blocking phase
   *   just as it did the first time through.
   * - **It does not, or there is none** — the character was built in the editor, or
   *   comes from a build whose phase list differed (Slice 2 removed `type`, so saves
   *   carrying it exist). It never passed these gates, so it is not held to them:
   *   the whole rail is open and {@link ungated} lifts the clamp. Both halves are
   *   needed — `furthest` alone leaves every step refused by `goTo`'s own guard,
   *   and the flag alone leaves them all locked at step 0.
   *
   * Resolved against {@link phases} rather than the profile's `creation_phases`
   * alone, so the wizard's own terminal `review` step round-trips too.
   */
  restore(storedPhase: string | undefined): void {
    const phases = this.phases;
    const stored = storedPhase === undefined ? -1 : phases.indexOf(storedPhase as CreationPhase);
    if (stored < 0) {
      this.step = 0;
      this.furthest = Math.max(phases.length - 1, 0);
      this.ungated = true;
      return;
    }
    this.step = stored;
    this.furthest = stored;
    this.ungated = false;
  }
}
