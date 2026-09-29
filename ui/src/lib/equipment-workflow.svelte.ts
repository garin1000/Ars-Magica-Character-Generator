// The carried-equipment list: which catalogue weapons/shields/armor the
// character has, which of them are worn, and whether a weapon's Ability
// specialization applies. Extracted out of `AppStore` (VA6 pattern), which keeps
// the composition root and the dirty/close-guard surface.
//
// Every edit is an in-place write to the host's own `entity` — the module never
// holds a copy, so `AppStore` stays the sole owner of the document and `dirty`
// keeps comparing the very object these methods mutate.

import type { Entity, LoadoutState } from './types';

/** The slice of `AppStore` the equipment workflow needs, as live accessors so it
 * always reads/writes the host's *current* state — `entity` stays owned by
 * `AppStore` alone, never copied here. */
export interface EquipmentWorkflowHost {
  entity: () => Entity;
  /** The host's debounced validation pass, shared with every other picker edit. */
  scheduleValidate: () => void;
}

export class EquipmentWorkflow {
  #host: EquipmentWorkflowHost;

  constructor(host: EquipmentWorkflowHost) {
    this.#host = host;
  }

  /** Add a carried equipment slot referencing a catalogue weapon/shield/armor id.
   * Defaults to `wielded` — the same "adding equips by default" behavior K2's
   * boolean `equipped: true` default gave. */
  add(item: string): void {
    if (!item) return;
    const entity = this.#host.entity();
    entity.equipment = [...(entity.equipment ?? []), { item, loadout: 'wielded' }];
    this.#host.scheduleValidate();
  }

  removeAt(index: number): void {
    const entity = this.#host.entity();
    entity.equipment = (entity.equipment ?? []).filter((_, i) => i !== index);
    this.#host.scheduleValidate();
  }

  /** Set the slot's loadout state (K5): stowed / carried / wielded. */
  setLoadout(index: number, loadout: LoadoutState): void {
    const entity = this.#host.entity();
    entity.equipment = (entity.equipment ?? []).map((slot, i) =>
      i === index ? { ...slot, loadout } : slot,
    );
    this.#host.scheduleValidate();
  }

  /** Toggle whether the weapon's Ability specialization applies (+1 Atk/Def). */
  setSpecialization(index: number, specialization_applies: boolean): void {
    const entity = this.#host.entity();
    entity.equipment = (entity.equipment ?? []).map((slot, i) =>
      i === index ? { ...slot, specialization_applies } : slot,
    );
    this.#host.scheduleValidate();
  }
}
