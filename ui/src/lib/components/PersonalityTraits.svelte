<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned } from '../derive';

  const traits = $derived(store.entity.personality_traits ?? []);
</script>

<!-- Personality Traits. Extracted from CharacterDetails so the wizard's
     `personality_reputations` step can mount it (with Reputations) without the
     rest of the details panel. -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('personality-label')}</h3>
  <ul class="trait-list" data-testid="personality-list">
    {#each traits as trait, i (i)}
      <li>
        <input
          class="trait-name"
          placeholder={store.t('personality-name-placeholder')}
          aria-label={store.t('personality-name-placeholder')}
          value={trait.name}
          oninput={(e) =>
            store.setPersonalityTraitName(i, (e.currentTarget as HTMLInputElement).value)}
          data-testid="personality-name-{i}"
        />
        <span class="spinner">
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('characteristic-decrement')}
            onclick={() => store.setPersonalityTraitValue(i, trait.value - 1)}
            data-testid="personality-dec-{i}"
          >
            -
          </button>
          <span class="spinner-value" data-testid="personality-value-{i}">
            {formatSigned(trait.value)}
          </span>
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('characteristic-increment')}
            onclick={() => store.setPersonalityTraitValue(i, trait.value + 1)}
            data-testid="personality-inc-{i}"
          >
            +
          </button>
        </span>
        <button
          type="button"
          class="icon-btn"
          aria-label={store.t('spell-remove')}
          onclick={() => store.removePersonalityTraitAt(i)}
          data-testid="personality-remove-{i}"
        >
          ×
        </button>
      </li>
    {:else}
      <li class="empty">{store.t('personality-empty')}</li>
    {/each}
  </ul>
  <button type="button" onclick={() => store.addPersonalityTrait()} data-testid="personality-add">
    {store.t('personality-add')}
  </button>
</div>
