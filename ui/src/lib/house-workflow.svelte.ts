// The magus's Hermetic House: which House, and the specialisation picks its
// grants leave open. Extracted out of `AppStore` (VA6 pattern), which keeps the
// composition root and the dirty/close-guard surface.
//
// Sibling to `mythic-workflow.svelte.ts::MythicWorkflow`, which does the same
// job for a Mythic Companion's type — the two are parallel in shape (a kind,
// plus the choice-keyed picks that kind defines) and deliberately separate, so
// neither has to reason about the other's catalogue.
//
// Every edit is an in-place write to the host's own `entity` — the module never
// holds a copy, so `AppStore` stays the sole owner of the document and `dirty`
// keeps comparing the very object these methods mutate.

import type { Entity, LocalizedRuleset, Selection } from './types';

/** The slice of `AppStore` the House workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface HouseWorkflowHost {
  entity: () => Entity;
  ruleset: () => LocalizedRuleset | null;
  /** The host's immediate validation pass, for the discrete House choice. */
  revalidate: () => Promise<void>;
  /** The host's debounced validation pass, shared with every other picker edit. */
  scheduleValidate: () => void;
}

export class HouseWorkflow {
  #host: HouseWorkflowHost;

  constructor(host: HouseWorkflowHost) {
    this.#host = host;
  }

  /**
   * Select the Hermetic House (or clear it with `null`). A discrete action, so
   * it validates immediately like `state.svelte.ts::AppStore.setMode`. Switching
   * House drops any specialisation picks whose `choice_key` the new House no
   * longer defines, so a stale pick from the previous House can't linger in the
   * save; clearing the House drops them all.
   */
  async setHouse(house: string | null): Promise<void> {
    const entity = this.#host.entity();
    if ((entity.house ?? null) === house) return;
    entity.house = house;
    entity.house_choices = this.#prunedChoices(house);
    await this.#host.revalidate();
  }

  /**
   * Set the specialisation pick for one of the current House's grants, keyed by
   * the grant's `choice_key` (a menu option for a `choice` grant, or a chosen
   * Virtue/Flaw for an `open` one). Debounced like the other picker edits.
   */
  setChoice(choiceKey: string, selection: Selection): void {
    const entity = this.#host.entity();
    entity.house_choices = { ...(entity.house_choices ?? {}), [choiceKey]: selection };
    this.#host.scheduleValidate();
  }

  /** The `choice_key`s the given House's `choice`/`open` grants define. */
  #choiceKeys(house: string | null): Set<string> {
    const keys = new Set<string>();
    if (!house) return keys;
    for (const grant of this.#host.ruleset()?.ruleset.houses?.[house]?.grants ?? []) {
      if (grant.kind === 'choice' || grant.kind === 'open') keys.add(grant.choice_key);
    }
    return keys;
  }

  /** Existing picks kept only where the target House still defines their key. */
  #prunedChoices(house: string | null): Record<string, Selection> {
    const valid = this.#choiceKeys(house);
    const kept: Record<string, Selection> = {};
    for (const [key, pick] of Object.entries(this.#host.entity().house_choices ?? {})) {
      if (valid.has(key)) kept[key] = pick;
    }
    return kept;
  }
}
