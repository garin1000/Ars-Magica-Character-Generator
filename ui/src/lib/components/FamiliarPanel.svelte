<script lang="ts">
  import { store } from '../state.svelte';

  const familiar = $derived(store.entity.familiar ?? null);

  function num(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }
</script>

<div class="detail-section">
  <h3 class="detail-label">{store.t('familiar-label')}</h3>
  {#if familiar}
    <input
      class="familiar-name"
      placeholder={store.t('familiar-name-placeholder')}
      value={familiar.name}
      oninput={(e) => store.setFamiliarName((e.currentTarget as HTMLInputElement).value)}
      data-testid="familiar-name"
    />
    <div class="cord-row">
      {#each [['gold', 'familiar-cord-gold'], ['silver', 'familiar-cord-silver'], ['bronze', 'familiar-cord-bronze']] as [cord, key] (cord)}
        <label class="field inline">
          <span>{store.t(key)}</span>
          <input
            type="number"
            min="0"
            value={familiar[`cord_${cord}` as 'cord_gold' | 'cord_silver' | 'cord_bronze'] ?? 0}
            oninput={(e) => store.setFamiliarCord(cord as 'gold' | 'silver' | 'bronze', num(e))}
            data-testid="familiar-cord-{cord}"
          />
        </label>
      {/each}
    </div>
    <button type="button" onclick={() => store.removeFamiliar()} data-testid="familiar-remove">
      {store.t('familiar-remove')}
    </button>
  {:else}
    <button type="button" onclick={() => store.addFamiliar()} data-testid="familiar-add">
      {store.t('familiar-add')}
    </button>
  {/if}
</div>

<style>
  /* `.detail-section` (spacing within a section) and `.field.inline` (label
     beside its input) are shared globals in app.css. */

  /* Section-level action buttons (Add / Remove familiar) are direct children of
     a `.detail-section`, which is a column flex with the default
     `align-items: stretch` — so without this they stretch to the full panel
     width. Shrink them to their label. */
  .detail-section > button {
    align-self: flex-start;
  }

  .familiar-name {
    flex: 1;
    min-width: 0;
  }

  /* Familiar bond cords sit in a wrapping row of inline number fields. */
  .cord-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }
</style>
