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

import { canonicalizeMultiRefValue, totalCopies } from './derive';
import type { EffectiveScores, Entity, LocalizedRuleset, PointItem, ValidationMode } from './types';

/** The slice of `AppStore` the selection workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface SelectionWorkflowHost {
  entity: () => Entity;
  ruleset: () => LocalizedRuleset | null;
  /** The last settled engine reading, for the granted half of a `max_total` count. */
  effective: () => EffectiveScores | null;
  /** The active validation mode — read only to decide whether a new row's
   *  `taken_as` defaults to a permitted reading (F27/E5). */
  mode: () => ValidationMode;
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
   * caller today. D10: an absent `max_total` means the engine's own default of
   * 1, not unlimited — see `derive.ts::atMaxTotalRefs`'s doc comment.
   */
  add(ref: string): void {
    const entity = this.#host.entity();
    const item = this.#host.ruleset()?.ruleset.point_items[ref];
    const repeatable = !!item?.parameters?.length || (item?.max_per_target ?? 1) > 1;
    const present = (entity.selections ?? []).some((s) => s.ref === ref);
    if (!repeatable && present) return;
    const max = item?.max_total ?? 1;
    const count = totalCopies(
      entity.selections ?? [],
      this.#host.effective()?.granted_selections ?? [],
      ref,
    );
    if (count >= max) return;
    const params = this.#defaultTakenAs(item);
    entity.selections = [...(entity.selections ?? []), params ? { ref, params } : { ref }];
    this.#host.scheduleValidate();
  }

  /**
   * F27/E5 (manual-test findings 2026-11-15 #27): a new row of an item whose
   * `category`-domain `taken_as` parameter carries a reading the profile
   * forbids IN FORCE (`EffectiveScores.forbidden_categories_in_force`)
   * defaults to the first PERMITTED one instead — Vengeful Powers added for a
   * Gift-less companion defaults to "story", never "hermetic". Enforced mode
   * only: Advisory/Silent leave `taken_as` unset, as before this fix, since
   * only Enforced mode refuses the forbidden reading at all. `undefined` when
   * the item has no `category`-domain parameter, or when every one of its
   * readings is forbidden (nothing permitted to default to).
   */
  #defaultTakenAs(item: PointItem | undefined): Record<string, string> | undefined {
    if (this.#host.mode() !== 'enforced') return undefined;
    const param = item?.parameters?.find((p) => p.domain === 'category');
    if (!param?.values?.length) return undefined;
    const forbidden = new Set(this.#host.effective()?.forbidden_categories_in_force ?? []);
    const permitted = param.values.find((value) => !forbidden.has(value));
    return permitted ? { [param.key]: permitted } : undefined;
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
   * Set one `multi_ref` parameter of one selection row to a whole SET of
   * values (D9 part 3, C5b) — a whole-set replace, since the picker always
   * recomputes the complete checked set on each toggle rather than adding or
   * removing one member at a time through this method.
   *
   * Canonicalized here, not by the caller: identity for a `multi_ref` value
   * is the sorted, deduplicated SET (§ 8), the same reason `setParamAt` above
   * trims — the write path is the one place that guarantees it regardless of
   * what a caller happens to hand in, so two players checking the same values
   * in different orders always write the identical array. An empty array is
   * kept (not dropped): C5a's `Multi(BTreeSet::new())` is a present-but-blank
   * value the engine reports as `missing_param`, distinct from the key being
   * absent entirely.
   */
  setMultiParamAt(index: number, key: string, values: Iterable<string>): void {
    const entity = this.#host.entity();
    const canonical = canonicalizeMultiRefValue(values);
    entity.selections = (entity.selections ?? []).map((s, i) =>
      i === index ? { ...s, params: { ...(s.params ?? {}), [key]: canonical } } : s,
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
