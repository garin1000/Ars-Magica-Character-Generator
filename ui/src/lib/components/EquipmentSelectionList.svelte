<script lang="ts">
  import { store } from '../state.svelte';

  // The chosen equipment slots (references to catalogue ids) — the selected side.
  const equipment = $derived(store.entity.equipment ?? []);

  function nameOf(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
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
