<script lang="ts">
  import { store } from '../state.svelte';
  import { artLabel, maxArtScore } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';

  const advancement = $derived(store.ruleset?.ruleset.art_advancement ?? []);
  const max = $derived(maxArtScore(advancement));
  const scores = $derived(store.entity.art_scores ?? []);

  function name(artId: string): string {
    return store.ruleset ? artLabel(store.ruleset, artId) : artId;
  }

  function bonusOf(artId: string): number {
    return store.effective?.art_bonuses?.find((b) => b.art === artId)?.bonus ?? 0;
  }

  function tip(artId: string): TooltipContent {
    return { text: store.ruleset?.i18n[artId]?.description ?? undefined };
  }
</script>

<section class="panel">
  {#if scores.length === 0}
    <p class="empty">{store.t('empty-selections-side')}</p>
  {:else}
    <ul class="selection-list ability-selection">
      {#each scores as entry, i (i)}
        <li>
          <span class="item-name" use:tooltip={tip(entry.art)}>{name(entry.art)}</span>
          <span class="spinner">
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('art-decrement')}
              disabled={entry.score <= 0}
              onclick={() => store.adjustArtAt(i, -1, max)}
              data-testid="art-dec-{entry.art}-{i}"
            >
              −
            </button>
            <span class="spinner-value" data-testid="art-score-{entry.art}-{i}">
              {entry.score}
            </span>
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('art-increment')}
              disabled={entry.score >= max}
              onclick={() => store.adjustArtAt(i, 1, max)}
              data-testid="art-inc-{entry.art}-{i}"
            >
              +
            </button>
            {#if bonusOf(entry.art) !== 0}
              <span class="eff-badge" data-testid="art-eff-{entry.art}-{i}">
                {store.t('effective-score', {
                  score: String(entry.score + bonusOf(entry.art)),
                })}
              </span>
            {/if}
          </span>
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('art-decrement')}
            onclick={() => store.removeArtAt(i)}
            data-testid="remove-{entry.art}-{i}"
          >
            ×
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>
