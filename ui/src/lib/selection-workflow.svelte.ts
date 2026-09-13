// The character's Virtue/Flaw selection rows: adding and removing them, and
// writing the parameters that give a repeatable item its target. Extracted out
// of `AppStore` (VA6 pattern), which keeps the composition root and the
// dirty/close-guard surface.
//
// Not the only writer of `entity.selections`, deliberately: a Mythic Companion
// type seeds and prunes its own required package
// (`mythic-workflow.svelte.ts::MythicWorkflow`), and an Art-domain parameter is
// written by `art-workflow.svelte.ts::ArtWorkflow`. Those are their domains'
// business; what lives here is the plain add/remove/parameter surface the V/F
// picker drives.
//
// Every edit is an in-place write to the host's own `entity` — the module never
// holds a copy, so `AppStore` stays the sole owner of the document and `dirty`
// keeps comparing the very object these methods mutate.

import { totalCopies } from './derive';
import type { EffectiveScores, Entity, LocalizedRuleset } from './types';

/** The slice of `AppStore` the selection workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface SelectionWorkflowHost {
  entity: () => Entity;
  ruleset: () => LocalizedRuleset | null;
  /** The last settled engine reading, for the granted half of a `max_total` count. */
  effective: () => EffectiveScores | null;
  /** The host's debounced validation pass, shared with every other picker edit. */
  scheduleValidate: () => void;
}

export class SelectionWorkflow {
  #host: SelectionWorkflowHost;

  constructor(host: SelectionWorkflowHost) {
    this.#host = host;
  }

  /**
   * Add a virtue/flaw selection. A repeatable item — one carrying parameters
   * (e.g. Great Characteristic) or with `max_per_target > 1` — can be added
   * several times, each instance choosing its own target; a plain item is added
   * once. Mirrors `ability-workflow.svelte.ts::AbilityWorkflow.add`.
   *
   * Also refuses once the item's bought+granted total already sits at its
   * `max_total` ceiling (Puissant Art, capped at two total across every Art
   * target) — the model-level guard behind the engine's
   * `too_many_selections` validator, so the store itself cannot be pushed past
   * it even though `VirtueFlawTab`'s disabled predicate is its only production
   * caller today.
   */
  add(ref: string): void {
    const entity = this.#host.entity();
    const item = this.#host.ruleset()?.ruleset.point_items[ref];
    const repeatable = !!item?.parameters?.length || (item?.max_per_target ?? 1) > 1;
    const present = (entity.selections ?? []).some((s) => s.ref === ref);
    if (!repeatable && present) return;
    if (item?.max_total !== undefined) {
      const count = totalCopies(
        entity.selections ?? [],
        this.#host.effective()?.granted_selections ?? [],
        ref,
      );
      if (count >= item.max_total) return;
    }
    entity.selections = [...(entity.selections ?? []), { ref }];
    this.#host.scheduleValidate();
  }

  /** Selection edits are by row index, since a repeatable item has several rows. */
  removeAt(index: number): void {
    const entity = this.#host.entity();
    entity.selections = (entity.selections ?? []).filter((_, i) => i !== index);
    this.#host.scheduleValidate();
  }

  /**
   * Set one parameter of one selection row.
   *
   * The value is **trimmed**, never case-folded. Parameter values decide a
   * selection's identity — the engine's duplicate key is the whole params map, and
   * `sameSelection` compares values byte-for-byte — so 'Wolf Shape ' would be a
   * second, distinct power, and the per-power cap would count them separately. The
   * engine trims the same way when a save is loaded
   * (`load_entity_migrating`), so the store and the file agree; capitalisation stays
   * the player's, since the rules ask for no folding. A value that trims to nothing
   * is kept as the empty string, which the engine reports as `missing_param`.
   */
  setParamAt(index: number, key: string, value: string): void {
    const entity = this.#host.entity();
    const trimmed = value.trim();
    entity.selections = (entity.selections ?? []).map((s, i) =>
      i === index ? { ...s, params: { ...(s.params ?? {}), [key]: trimmed } } : s,
    );
    this.#host.scheduleValidate();
  }

  /**
   * Point an ability-bonus selection (Puissant Ability) at a specific ability
   * *instance*. The ability id goes under the `ability` param; for a
   * parameterized ability the instance value (the area/language) goes under the
   * ability's own param key (so the bonus attaches to that one row). Switching to
   * a plain ability drops any stale instance key.
   *
   * The instance value is free text the player typed, so it is trimmed for the same
   * identity reason as {@link setParamAt}: ' Rhine ' and 'Rhine' are one Area Lore,
   * and `usedAbilityTargets` composes the instance into the target key it caps on.
   * A value that is nothing but whitespace names no instance, so it is dropped
   * rather than stored blank — leaving the engine's `missing_param` to name the key,
   * exactly as an unfilled instance box already does.
   */
  setAbilityBonusTarget(index: number, abilityId: string, parameter?: string | null): void {
    const entity = this.#host.entity();
    const instanceKey =
      this.#host.ruleset()?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
    const params: Record<string, string> = { ability: abilityId.trim() };
    const instance = parameter?.trim();
    if (instanceKey && instance) params[instanceKey] = instance;
    entity.selections = (entity.selections ?? []).map((s, i) =>
      i === index ? { ...s, params } : s,
    );
    this.#host.scheduleValidate();
  }
}
