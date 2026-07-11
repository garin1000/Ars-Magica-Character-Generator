<script lang="ts">
  import { store } from '../state.svelte';

  const age = $derived(store.entity.age ?? null);
  const ageCap = $derived(store.effective?.age_ability_cap ?? null);
  // Confidence is derived (type default + V/F); grogs have none (0/0) → hidden.
  const confScore = $derived(store.effective?.confidence_score ?? 0);
  const confPoints = $derived(store.effective?.confidence_points ?? 0);
  const showConfidence = $derived(confScore > 0 || confPoints > 0);
  // Warping is derived from V/F (Warped by Magic → 1/5); hidden when 0/0.
  const warpScore = $derived(store.effective?.warping_score ?? 0);
  const warpPoints = $derived(store.effective?.warping_points ?? 0);
  const showWarping = $derived(warpScore > 0 || warpPoints > 0);
  const traits = $derived(store.entity.personality_traits ?? []);
  const reputations = $derived(store.entity.reputations ?? []);
  // Reputation input is offered only for the kinds a V/F grants (Core:2514).
  const grants = $derived(store.effective?.reputation_grants ?? []);

  function onAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAge(raw === '' ? null : Number(raw));
  }

  function repKindLabel(kind: string): string {
    return store.t(`reputation-type-${kind}`);
  }
</script>

<section class="panel character-details">
  {#if store.ruleset}
    <div class="detail-field">
      <label class="field">
        <span>{store.t('age-label')}</span>
        <input type="number" min="1" value={age ?? ''} oninput={onAge} data-testid="age-input" />
      </label>
      {#if ageCap != null}
        <span class="age-cap" data-testid="age-cap-note">
          {store.t('age-cap-note', { cap: String(ageCap) })}
        </span>
      {/if}
    </div>

    {#if showConfidence}
      <div class="detail-field">
        <span class="detail-label">{store.t('confidence-label')}</span>
        <span data-testid="confidence-readout">
          {store.t('confidence-readout', { score: String(confScore), points: String(confPoints) })}
        </span>
      </div>
    {/if}

    {#if showWarping}
      <div class="detail-field">
        <span class="detail-label">{store.t('warping-label')}</span>
        <span data-testid="warping-readout">
          {store.t('warping-readout', { score: String(warpScore), points: String(warpPoints) })}
        </span>
      </div>
    {/if}

    <div class="detail-section">
      <h3 class="detail-label">{store.t('personality-label')}</h3>
      <ul class="trait-list" data-testid="personality-list">
        {#each traits as trait, i (i)}
          <li>
            <input
              class="trait-name"
              placeholder={store.t('personality-name-placeholder')}
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
                −
              </button>
              <span class="spinner-value" data-testid="personality-value-{i}">
                {trait.value > 0 ? `+${trait.value}` : trait.value}
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
      <button
        type="button"
        onclick={() => store.addPersonalityTrait()}
        data-testid="personality-add"
      >
        {store.t('personality-add')}
      </button>
    </div>

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
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
