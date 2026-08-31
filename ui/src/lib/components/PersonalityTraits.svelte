<script lang="ts">
  import { store } from '../state.svelte';

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
            aria-label={store.t('characteristic-decrement', { name: trait.name })}
            onclick={() => store.setPersonalityTraitValue(i, trait.value - 1)}
            data-testid="personality-dec-{i}"
          >
            -
          </button>
          <!-- Bound number input, not just the spinner: a Personality Trait's
               |value| can reach 6 (a Personality Flaw, Core Rules), which is up to
               twelve clicks from 0 with no other way in — every other scored
               control with a wide/signed range (aging points, Talisman
               bonus/level) offers direct entry too (S29, full-audit UX). The
               range mirrors the store's own clamp (`setPersonalityTraitValue`). -->
          <input
            type="number"
            class="spinner-value-input"
            min="-6"
            max="6"
            value={trait.value}
            aria-label={store.t('personality-value-label', { name: trait.name })}
            oninput={(e) =>
              store.setPersonalityTraitValue(
                i,
                Number((e.currentTarget as HTMLInputElement).value) || 0,
              )}
            data-testid="personality-value-{i}"
          />
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('characteristic-increment', { name: trait.name })}
            onclick={() => store.setPersonalityTraitValue(i, trait.value + 1)}
            data-testid="personality-inc-{i}"
          >
            +
          </button>
        </span>
        <button
          type="button"
          class="icon-btn"
          aria-label={store.t('remove-item', { name: trait.name })}
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
