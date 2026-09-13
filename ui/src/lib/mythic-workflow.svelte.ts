// The Mythic Companion's type: which type, the required package it seeds into
// the V/F selections, and the grant picks the type leaves open. Extracted out of
// `AppStore` (VA6 pattern), which keeps the composition root and the
// dirty/close-guard surface.
//
// Sibling to `house-workflow.svelte.ts::HouseWorkflow`, which does the same job
// for a magus's Hermetic House.
//
// Every edit is an in-place write to the host's own `entity` — the module never
// holds a copy, so `AppStore` stays the sole owner of the document and `dirty`
// keeps comparing the very object these methods mutate.

import { sameSelection } from './derive';
import type { Entity, LocalizedRuleset, Selection } from './types';

/** The slice of `AppStore` the Mythic Companion workflow needs, as live
 * accessors so it always reads/writes the host's *current* state — `entity`
 * stays owned by `AppStore` alone, never copied here. */
export interface MythicWorkflowHost {
  entity: () => Entity;
  ruleset: () => LocalizedRuleset | null;
  /** The host's immediate validation pass, for the discrete type choice. */
  revalidate: () => Promise<void>;
  /** The host's debounced validation pass, shared with every other picker edit. */
  scheduleValidate: () => void;
}

export class MythicWorkflow {
  #host: MythicWorkflowHost;

  constructor(host: MythicWorkflowHost) {
    this.#host = host;
  }

  /**
   * Select the Mythic Companion type (or clear it with `null`). A discrete
   * action, so it validates immediately. Switching auto-manages the type's
   * required package: it removes the previous type's seeded required Virtues/
   * Flaws that the new type doesn't require, then seeds the new type's package
   * (its required Virtues + each required Flaw's rules default) as ordinary
   * budgeted selections — so a direct-entry mythic companion starts legal, with
   * the required Flaws swappable via {@link setRequiredFlaw}. The free
   * status/Minor Virtue are point-free grants derived engine-side (never in
   * `selections`); a `choice` free-Minor (Devil Child's Might/Powers) defaults to
   * its first option. Mirrors `house-workflow.svelte.ts::HouseWorkflow.setHouse`.
   */
  async setType(mythicType: string | null): Promise<void> {
    const entity = this.#host.entity();
    if ((entity.mythic_type ?? null) === mythicType) return;
    const previousPackage = this.#package(entity.mythic_type ?? null);
    const nextPackage = this.#package(mythicType);
    // Drop the previous type's seeded package rows the new type doesn't require.
    const kept = (entity.selections ?? []).filter(
      (s) =>
        !(
          previousPackage.some((p) => sameSelection(p, s)) &&
          !nextPackage.some((p) => sameSelection(p, s))
        ),
    );
    entity.mythic_type = mythicType;
    entity.mythic_choices = this.#defaultedChoices(mythicType);
    // Seed the new type's required package (budgeted) where not already present.
    for (const pkg of nextPackage) {
      if (!kept.some((s) => sameSelection(s, pkg))) kept.push(pkg);
    }
    entity.selections = kept;
    await this.#host.revalidate();
  }

  /**
   * Set a Mythic Companion type grant pick keyed by the grant's `choice_key`
   * (e.g. Devil Child's Demonic Might-or-Powers free Minor). Debounced like the
   * other picker edits. Mirrors
   * `house-workflow.svelte.ts::HouseWorkflow.setChoice`.
   */
  setChoice(choiceKey: string, selection: Selection): void {
    const entity = this.#host.entity();
    entity.mythic_choices = {
      ...(entity.mythic_choices ?? {}),
      [choiceKey]: selection,
    };
    this.#host.scheduleValidate();
  }

  /**
   * Swap a required Flaw for a "suitable substitute agreed with the troupe":
   * removes the currently-selected required Flaw (`previousRef`) and adds the
   * chosen substitute (`nextRef`) as a budgeted selection. A discrete dropdown
   * action, so it validates immediately.
   */
  async setRequiredFlaw(previousRef: string, nextRef: string): Promise<void> {
    if (previousRef === nextRef) return;
    const entity = this.#host.entity();
    const selections = [...(entity.selections ?? [])];
    const idx = selections.findIndex((s) => s.ref === previousRef);
    if (idx >= 0) selections.splice(idx, 1);
    if (!selections.some((s) => s.ref === nextRef)) selections.push({ ref: nextRef });
    entity.selections = selections;
    await this.#host.revalidate();
  }

  /** The budgeted required package (required Virtues + each Flaw's default). */
  #package(mythicType: string | null): Selection[] {
    if (!mythicType) return [];
    const t = this.#host.ruleset()?.ruleset.mythic_companion_types?.[mythicType];
    if (!t) return [];
    return [...(t.required_virtues ?? []), ...(t.required_flaws ?? []).map((f) => f.default)];
  }

  /**
   * Mythic-type grant picks kept where the target type still defines their key,
   * with each `choice` grant defaulted to its first option so the free Minor
   * Virtue is granted without an extra step.
   */
  #defaultedChoices(mythicType: string | null): Record<string, Selection> {
    const grants = mythicType
      ? (this.#host.ruleset()?.ruleset.mythic_companion_types?.[mythicType]?.grants ?? [])
      : [];
    const validKeys = new Set(
      grants.filter((g) => g.kind === 'choice' || g.kind === 'open').map((g) => g.choice_key),
    );
    const kept: Record<string, Selection> = {};
    for (const [key, pick] of Object.entries(this.#host.entity().mythic_choices ?? {})) {
      if (validKeys.has(key)) kept[key] = pick;
    }
    for (const g of grants) {
      if (g.kind === 'choice' && !kept[g.choice_key]) kept[g.choice_key] = g.options[0];
    }
    return kept;
  }
}
