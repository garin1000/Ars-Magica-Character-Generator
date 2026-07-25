<script lang="ts">
  import { store } from '../state.svelte';
  import type { EquipmentSlot } from '../types';

  // The chosen equipment slots (references to catalogue ids) — the selected side.
  const equipment = $derived(store.entity.equipment ?? []);

  function nameOf(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  // The specialization toggle is meaningful only for an equipped weapon whose
  // combat Ability carries a non-empty specialty on this character — otherwise it
  // is a dead toggle (Core:7122, :7139), so we render it only when applicable.
  function specializationApplicable(slot: EquipmentSlot): boolean {
    if (!slot.equipped) return false;
    const weapon = store.ruleset?.ruleset?.weapons?.[slot.item];
    if (!weapon) return false;
    return (store.entity.ability_scores ?? []).some(
      (a) => a.ability === weapon.ability && (a.specialty ?? '').trim() !== '',
    );
  }
</script>

<section class="panel">
  {#if equipment.length === 0}
    <p class="empty">{store.t('equipment-empty')}</p>
  {:else}
    <ul class="equipment-list" data-testid="equipment-list">
      {#each equipment as slot, i (i)}
        <li>
          <span class="equipment-name" data-testid="equipment-name-{i}">{nameOf(slot.item)}</span>
          <label class="checkbox inline">
            <input
              type="checkbox"
              checked={slot.equipped ?? false}
              onchange={(e) =>
                store.setEquipmentEquipped(i, (e.currentTarget as HTMLInputElement).checked)}
              data-testid="equipment-equipped-{i}"
            />
            <span>{store.t('equipment-equipped-label')}</span>
          </label>
          {#if specializationApplicable(slot)}
            <label class="checkbox inline">
              <input
                type="checkbox"
                checked={slot.specialization_applies ?? false}
                onchange={(e) =>
                  store.setEquipmentSpecialization(
                    i,
                    (e.currentTarget as HTMLInputElement).checked,
                  )}
                data-testid="equipment-specialization-{i}"
              />
              <span>{store.t('equipment-specialization-label')}</span>
            </label>
          {/if}
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('equipment-remove')}
            onclick={() => store.removeEquipmentAt(i)}
            data-testid="equipment-remove-{i}"
          >
            ×
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>
