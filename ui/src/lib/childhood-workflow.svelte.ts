// The Sample Childhood package the player is drafting: which package, and the
// values typed into its parameter slots, before the draft is submitted.
// Extracted out of `AppStore` (VA6). `applyChildhoodPackage` DOES replace the
// whole entity wholesale on acceptance (the Ability rows the package writes
// and the record of the package taken arrive together), routed through the
// `host` so `AppStore` stays the sole owner of `entity`.

import * as ipc from './ipc';
import type { AppError, Entity, LocalizedRuleset, ValidationIssue } from './types';

/**
 * The Sample Childhood package the player is *considering*, plus the values typed
 * into its parameter slots — the in-progress form, before it is applied.
 *
 * Deliberately UI state and never part of the entity. The entity records only the
 * package actually taken (`life_stages.childhood_package`) and the Ability rows the
 * application writes, whose `parameter` values *are* these slot values; keeping a
 * parallel draft on the entity would give one decision two representations that can
 * diverge — and would dirty the document for merely opening a picker. Held on the
 * store rather than in the component so it survives a tab switch.
 */
export interface ChildhoodDraft {
  /** The package the picker has selected; `null` while none is chosen. */
  packageId: string | null;
  /** Slot key (`area_a`, `language`, …) -> the player's value. Blanks are absent. */
  slots: Record<string, string>;
}

/** A fresh, empty childhood draft (the initial/reset state). */
export function defaultChildhoodDraft(): ChildhoodDraft {
  return { packageId: null, slots: {} };
}

/** The slice of `AppStore` the childhood workflow needs, as live accessors so
 * it always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface ChildhoodWorkflowHost {
  ruleset: () => LocalizedRuleset | null;
  entity: () => Entity;
  setEntity: (entity: Entity) => void;
  revalidate: () => Promise<void>;
  setError: (error: AppError | null) => void;
}

export class ChildhoodWorkflow {
  #host: ChildhoodWorkflowHost;

  constructor(host: ChildhoodWorkflowHost) {
    this.#host = host;
  }

  /**
   * The in-progress Sample Childhood choice (see {@link ChildhoodDraft}). UI
   * state: never part of the entity, so it is never saved and drafting never
   * dirties the document.
   */
  draft = $state<ChildhoodDraft>(defaultChildhoodDraft());

  /**
   * Why the last attempt to take a Sample Childhood package was refused, for the
   * picker to render beside the offending slots. Empty when there is nothing to say.
   *
   * Deliberately its own field rather than part of the store's validation result:
   * these findings describe the *command input* the player just submitted, not
   * the state of the entity — which is unchanged by a rejection. Cleared on the
   * next apply and by every draft edit, since a rejection pointing at a field the
   * user has just corrected is worse than none at all.
   */
  rejections = $state<ValidationIssue[]>([]);

  /**
   * Select the Sample Childhood package the player is considering, or clear the
   * consideration with `null`. Draft state only: nothing is written to the
   * entity until {@link apply}.
   *
   * Switching packages drops the slot values the new package does not declare, so a
   * stale answer from the previous one can never be submitted. Slots both packages
   * ask for survive, so comparing two childhoods does not mean re-typing the shared
   * answers. Clearing drops the selection and every slot.
   *
   * Not validated and not debounced: a draft is a form the engine has not been shown
   * yet, so there is nothing to check until it is submitted. Editing the draft does
   * retire the last rejection, which was about the form as it stood before the edit.
   */
  setDraftPackage(packageId: string | null): void {
    this.rejections = [];
    if (packageId === null) {
      this.draft = defaultChildhoodDraft();
      return;
    }
    this.draft = { packageId, slots: this.#prunedSlots(packageId) };
  }

  /**
   * Answer one of the drafted package's parameter slots (the Area Lore region, the
   * language, …). A blank or whitespace-only value deletes the key rather than
   * storing an empty string: an unanswered slot is absent, which is what the
   * engine's "slot unanswered" rejection is about. Draft state only, like
   * {@link setDraftPackage}.
   */
  setDraftSlot(slot: string, value: string): void {
    this.rejections = [];
    const answer = value.trim();
    const slots = { ...this.draft.slots };
    if (answer) {
      slots[slot] = answer;
    } else {
      delete slots[slot];
    }
    this.draft.slots = slots;
  }

  /** The parameter slot keys the given package's entries declare. */
  #slotKeys(packageId: string): Set<string> {
    const keys = new Set<string>();
    for (const entry of this.#host.ruleset()?.ruleset.childhoods?.[packageId]?.entries ?? []) {
      if (entry.slot) keys.add(entry.slot);
    }
    return keys;
  }

  /** Drafted slot answers kept only where the target package still asks for them. */
  #prunedSlots(packageId: string): Record<string, string> {
    const asked = this.#slotKeys(packageId);
    const kept: Record<string, string> = {};
    for (const [slot, answer] of Object.entries(this.draft.slots)) {
      if (asked.has(slot)) kept[slot] = answer;
    }
    return kept;
  }

  /**
   * Take the drafted Sample Childhood package: submit it with its slot answers and
   * keep whatever the engine decides. A no-op while no package is drafted — there is
   * nothing to submit.
   *
   * The engine owns the whole mechanic, so this only routes its two outcomes. On
   * `applied` the returned entity replaces the current one wholesale (the Ability
   * rows the package writes and the record of the package taken arrive together);
   * `dirty` needs no help, since it derives from the entity snapshot. On `rejected`
   * the entity is left exactly as it was and the findings land in {@link rejections}
   * for the picker to show against the offending slots.
   */
  async apply(): Promise<void> {
    const packageId = this.draft.packageId;
    if (!packageId) return;
    this.rejections = [];
    try {
      const outcome = await ipc.applyChildhoodPackage(
        $state.snapshot(this.#host.entity()),
        packageId,
        $state.snapshot(this.draft.slots),
      );
      if (outcome.status === 'rejected') {
        this.rejections = outcome.issues;
        return;
      }
      this.#host.setEntity(outcome.entity);
      await this.#host.revalidate();
    } catch (e) {
      this.#host.setError(e as AppError);
    }
  }
}
