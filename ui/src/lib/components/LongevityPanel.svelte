<script lang="ts">
  import { store } from '../state.svelte';
  import type { LongevitySource } from '../types';

  const longevity = $derived(store.entity.longevity_ritual ?? null);

  function num(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }
</script>

<div class="detail-section">
  <h3 class="detail-label">{store.t('longevity-label')}</h3>
  {#if longevity}
    <div class="longevity-source" role="radiogroup" aria-label={store.t('longevity-source-label')}>
      {#each ['self_made', 'external'] as source (source)}
        <label class="radio">
          <input
            type="radio"
            name="longevity-source"
            value={source}
            checked={longevity.source === source}
            onchange={() => store.setLongevitySource(source as LongevitySource)}
            data-testid="longevity-source-{source}"
          />
          <span>{store.t(`longevity-source-${source}`)}</span>
        </label>
      {/each}
    </div>
    {#if longevity.source === 'external'}
      <label class="field inline">
        <span>{store.t('longevity-bonus-label')}</span>
        <input
          type="number"
          value={longevity.bonus ?? 0}
          oninput={(e) => store.setLongevityBonus(num(e))}
          data-testid="longevity-bonus"
        />
      </label>
    {:else}
      <p class="empty" data-testid="longevity-self-made-note">
        {store.t('longevity-self-made-note')}
      </p>
    {/if}
    <button
      type="button"
      onclick={() => store.removeLongevityRitual()}
      data-testid="longevity-remove"
    >
      {store.t('longevity-remove')}
    </button>
  {:else}
    <button
      type="button"
      onclick={() => store.addLongevityRitual('self_made')}
      data-testid="longevity-add"
    >
      {store.t('longevity-add')}
    </button>
  {/if}
</div>

<style>
  /* `.detail-section` (spacing within a section) and `.field.inline` (label
     beside its input) are shared globals in app.css. */

  /* Section-level action buttons (Add / Remove ritual) are direct children of a
     `.detail-section`, which is a column flex with the default
     `align-items: stretch` — so without this they stretch to the full panel
     width. Shrink them to their label. */
  .detail-section > button {
    align-self: flex-start;
  }

  /* Longevity source radios on one row. */
  .longevity-source {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .radio {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .empty {
    color: var(--muted);
  }
</style>
