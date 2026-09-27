// The character's Ability rows: which Abilities are taken, their bought scores,
// their specialties and — for a parameterized Ability — the instance value that
// makes each row a distinct Ability. Extracted out of `AppStore` (VA6 pattern),
// which keeps the composition root and the dirty/close-guard surface.
//
// Deliberately NOT here: `xp_pool` and the funding mode. The pool is spent by
// Abilities and Arts alike (one shared pool, not two banks), and
// `setAbilityFunding` switches a document-wide mode that also clears the
// childhood and aging drafts — neither belongs to a single domain, so both stay
// on the composition root.
//
// Every edit is an in-place write to the host's own `entity` — the module never
// holds a copy, so `AppStore` stays the sole owner of the document and `dirty`
// keeps comparing the very object these methods mutate.

import type { AbilityParamValue, Entity, LocalizedRuleset } from './types';

/** The slice of `AppStore` the Ability workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface AbilityWorkflowHost {
  entity: () => Entity;
  ruleset: () => LocalizedRuleset | null;
  /** The host's debounced validation pass, shared with every other picker edit. */
  scheduleValidate: () => void;
}

export class AbilityWorkflow {
  #host: AbilityWorkflowHost;

  constructor(host: AbilityWorkflowHost) {
    this.#host = host;
  }

  /**
   * Select an ability (like a virtue/flaw): it enters at score 0, which costs no
   * XP — the first point is bought by raising it. A parameterized ability (e.g.
   * (Area) Lore) can be added several times (each instance gets its own value); a
   * plain ability is added once.
   */
  add(ability: string): void {
    const entity = this.#host.entity();
    const parameterized = !!this.#host.ruleset()?.ruleset.abilities?.[ability]?.parameter;
    const present = (entity.ability_scores ?? []).some((a) => a.ability === ability);
    if (!parameterized && present) return;
    entity.ability_scores = [...(entity.ability_scores ?? []), { ability, score: 0 }];
    this.#host.scheduleValidate();
  }

  /** Ability edits are by row index, since a parameterized ability has several rows. */
  removeAt(index: number): void {
    const entity = this.#host.entity();
    entity.ability_scores = (entity.ability_scores ?? []).filter((_, i) => i !== index);
    this.#host.scheduleValidate();
  }

  adjustAt(index: number, delta: number, max: number): void {
    const entity = this.#host.entity();
    entity.ability_scores = (entity.ability_scores ?? []).map((a, i) =>
      i === index ? { ...a, score: Math.max(0, Math.min(max, a.score + delta)) } : a,
    );
    this.#host.scheduleValidate();
  }

  setSpecialtyAt(index: number, specialty: string): void {
    const entity = this.#host.entity();
    const spec = specialty.trim() ? specialty.trim() : undefined;
    entity.ability_scores = (entity.ability_scores ?? []).map((a, i) =>
      i === index ? { ...a, specialty: spec } : a,
    );
    this.#host.scheduleValidate();
  }

  /**
   * The free-text input still writes only `Text` — the combo box that also
   * offers a catalogue entry or a linked Virtue/Flaw parameter is CV7's job
   * (design § 6.1/§ 6.3); this is the "Other…" escape it will fall back to.
   */
  setParameterAt(index: number, value: string): void {
    const entity = this.#host.entity();
    const trimmed = value.trim();
    const param: AbilityParamValue | undefined = trimmed ? { text: trimmed } : undefined;
    entity.ability_scores = (entity.ability_scores ?? []).map((a, i) =>
      i === index ? { ...a, parameter: param } : a,
    );
    this.#host.scheduleValidate();
  }

  /**
   * Writes a FULL parameter value directly — the combo box's other two
   * entries, a catalogue choice (`{id}`) or a link target (`{item, param}`)
   * (design § 6.1/§ 6.3, CV7). `setParameterAt` above stays the "Other…"
   * free-text escape's own write path; choosing a different combo entry
   * always REPLACES whatever was stored before, never merges with it — there
   * is no "keep both" state (design § 6.3).
   *
   */
  setParameterValueAt(index: number, value: AbilityParamValue | undefined): void {
    const entity = this.#host.entity();
    entity.ability_scores = (entity.ability_scores ?? []).map((a, i) =>
      i === index ? { ...a, parameter: value } : a,
    );
    this.#host.scheduleValidate();
  }
}
