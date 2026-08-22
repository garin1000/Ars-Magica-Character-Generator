<script lang="ts">
  import { store } from '../state.svelte';

  const reputations = $derived(store.entity.reputations ?? []);
  // Reputation input is offered only for the kinds a V/F grants (Core:2514).
  const grants = $derived(store.effective?.reputation_grants ?? []);

  function repKindLabel(kind: string): string {
    return store.t(`reputation-type-${kind}`);
  }
</script>

<!-- Reputations. Extracted from CharacterDetails alongside PersonalityTraits, so
     the wizard's `personality_reputations` step is those two and nothing else. -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('reputations-label')}</h3>
  <ul class="reputation-list" data-testid="reputation-list">
    {#each reputations as reputation, i (i)}
      <li>
        <span class="reputation-tag">
          {repKindLabel(reputation.kind)}
          {reputation.score}
        </span>
        <input
          class="reputation-content"
          placeholder={store.t('reputation-content-placeholder')}
          aria-label={store.t('reputation-content-placeholder')}
          value={reputation.content}
          oninput={(e) =>
            store.setReputationContent(i, (e.currentTarget as HTMLInputElement).value)}
          data-testid="reputation-content-{i}"
        />
        <button
          type="button"
          class="icon-btn"
          aria-label={store.t('spell-remove')}
          onclick={() => store.removeReputationAt(i)}
          data-testid="reputation-remove-{i}"
        >
          ×
        </button>
      </li>
    {/each}
  </ul>
  {#if grants.length === 0}
    <p class="empty" data-testid="reputation-empty">{store.t('reputation-empty')}</p>
  {:else}
    {#each grants as grant, gi (gi)}
      <button
        type="button"
        onclick={() => store.addReputation(grant.kind, grant.score)}
        data-testid="reputation-add-{grant.kind}"
      >
        {store.t('reputation-add', {
          kind: repKindLabel(grant.kind),
          score: String(grant.score),
        })}
      </button>
    {/each}
  {/if}
</div>
