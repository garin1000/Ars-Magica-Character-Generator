// The magus's spell list: which spells are known, at what level, with what
// target Form, plus each spell's bought Spell Mastery score and its mastery
// special abilities — and the per-character spell-levels budget override.
// Extracted out of `AppStore` (VA6 pattern), which keeps the composition root
// and the dirty/close-guard surface.
//
// Every edit is an in-place write to the host's own `entity` — the module never
// holds a copy, so `AppStore` stays the sole owner of the document and `dirty`
// keeps comparing the very object these methods mutate.

import { U32_MAX, U8_MAX, clampInt } from './clamp';
import type { Entity, LocalizedRuleset } from './types';

/** The slice of `AppStore` the spell workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface SpellWorkflowHost {
  entity: () => Entity;
  ruleset: () => LocalizedRuleset | null;
  /** The host's debounced validation pass, shared with every other picker edit. */
  scheduleValidate: () => void;
}

export class SpellWorkflow {
  #host: SpellWorkflowHost;

  constructor(host: SpellWorkflowHost) {
    this.#host = host;
  }

  /**
   * Add a spell to the magus's list. `level` is passed only for a General spell
   * (the chosen level); a fixed spell derives its level from the catalogue.
   * `parameter` names the target Form of a parametrized meta-magic Vim spell.
   *
   * Identity is (spell, level, parameter): the same base spell may be taken once
   * per distinct Form. A parametrized spell adds a fresh row each time (its Form
   * is chosen afterwards in the selected list, mirroring parametrized abilities),
   * so it is never blocked at add time and its source row never greys just
   * because one Form instance exists; exact (spell, level, Form) duplicates are
   * flagged by the engine's dedupe. A plain spell is added once per (level).
   */
  add(spellId: string, level?: number | null, parameter?: string | null): void {
    const entity = this.#host.entity();
    const lvl = typeof level === 'number' ? level : undefined;
    const param = parameter ?? undefined;
    const parameterized =
      (this.#host.ruleset()?.ruleset.spells?.[spellId]?.parameters?.length ?? 0) > 0;
    if (!parameterized) {
      const present = (entity.spells ?? []).some(
        (s) =>
          s.spell === spellId &&
          (s.level ?? undefined) === lvl &&
          (s.parameter ?? undefined) === param,
      );
      if (present) return;
    }
    entity.spells = [
      ...(entity.spells ?? []),
      {
        spell: spellId,
        ...(lvl === undefined ? {} : { level: lvl }),
        ...(param === undefined ? {} : { parameter: param }),
      },
    ];
    this.#host.scheduleValidate();
  }

  /**
   * Set (or clear) the target Form of a parametrized spell at `index` — part of
   * the spell's identity, so distinct Forms are distinct instances. The chosen
   * value is an Art id (e.g. `art.ignem`). Mirrors
   * `ability-workflow.svelte.ts::AbilityWorkflow.setParameterAt`.
   */
  setParameterAt(index: number, parameter: string | null): void {
    const entity = this.#host.entity();
    const param = parameter && parameter.trim() ? parameter.trim() : undefined;
    entity.spells = (entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, parameter: param } : s,
    );
    this.#host.scheduleValidate();
  }

  /**
   * Adjust the bought Spell Mastery score of the spell at `index` by `delta`,
   * clamped to [0, max]. Spent from the restricted Spell-Mastery XP pool; the
   * granted floor (Flawless Magic) is applied on top when computing the effective
   * mastery, so it is not stored here. Mirrors
   * `ability-workflow.svelte.ts::AbilityWorkflow.adjustAt`.
   */
  adjustMasteryAt(index: number, delta: number, max: number): void {
    const entity = this.#host.entity();
    entity.spells = (entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, mastery: Math.max(0, Math.min(max, (s.mastery ?? 0) + delta)) } : s,
    );
    this.#host.scheduleValidate();
  }

  /**
   * Add a Spell Mastery special ability (a `spell_mastery_ability.*` id) to the
   * spell at `index`. A repeatable ability (Precise/Quick/Quiet Casting) may be
   * added more than once; the count cap vs. effective mastery is enforced by the
   * engine, not here. Mirrors {@link adjustMasteryAt}.
   */
  addMasteryAbilityAt(index: number, abilityId: string): void {
    const entity = this.#host.entity();
    entity.spells = (entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, mastery_abilities: [...(s.mastery_abilities ?? []), abilityId] } : s,
    );
    this.#host.scheduleValidate();
  }

  /**
   * Remove the mastery special ability at position `abilityIndex` within the
   * spell at `index`. Index-addressed so a repeatable ability chosen several
   * times removes exactly one instance. Mirrors {@link addMasteryAbilityAt}.
   */
  removeMasteryAbilityAt(index: number, abilityIndex: number): void {
    const entity = this.#host.entity();
    entity.spells = (entity.spells ?? []).map((s, i) =>
      i === index
        ? {
            ...s,
            mastery_abilities: (s.mastery_abilities ?? []).filter((_, j) => j !== abilityIndex),
          }
        : s,
    );
    this.#host.scheduleValidate();
  }

  /**
   * Set the level of the (General) spell at `index`. The budget/used totals are
   * engine-authoritative, so no recompute happens here. Mirrors
   * {@link adjustMasteryAt}.
   */
  setLevelAt(index: number, level: number): void {
    const entity = this.#host.entity();
    // `SpellSelection.level` is the entity's narrowest number (u8), and a General
    // spell has no level 0, so the floor is 1 — matching the input's `min`.
    const clamped = clampInt(level, 1, U8_MAX);
    entity.spells = (entity.spells ?? []).map((s, i) =>
      i === index ? { ...s, level: clamped } : s,
    );
    this.#host.scheduleValidate();
  }

  /** Spell edits are by row index, since a General spell can appear at several levels. */
  removeAt(index: number): void {
    const entity = this.#host.entity();
    entity.spells = (entity.spells ?? []).filter((_, i) => i !== index);
    this.#host.scheduleValidate();
  }

  /**
   * Set (or clear) the per-character spell-levels budget override. A non-positive
   * or non-finite value clears it (`null`), so the engine falls back to the type
   * profile's base. The budget/used totals stay engine-authoritative — no
   * recompute happens here. Mirrors `state.svelte.ts::AppStore.setAge`.
   */
  setLevelsOverride(levels: number | null): void {
    const entity = this.#host.entity();
    entity.spell_levels_override =
      levels != null && Number.isFinite(levels) && levels > 0 ? clampInt(levels, 1, U32_MAX) : null;
    this.#host.scheduleValidate();
  }
}
