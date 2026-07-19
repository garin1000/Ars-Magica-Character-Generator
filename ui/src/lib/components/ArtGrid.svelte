<script lang="ts">
  import { store } from '../state.svelte';
  import { artAbbreviation, artLabel, groupArtsByType, maxArtScore } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';
  import type { Art } from '../types';

  const max = $derived(maxArtScore(store.ruleset?.ruleset.art_advancement ?? []));

  // All 15 Arts are always present for a magus — no pick step. Laid out as the
  // character sheet's three columns: Techniques, then the Forms split into two
  // halves of five (both groups already sorted by localized name).
  const columns = $derived.by((): { labelKey: string; arts: Art[] }[] => {
    if (!store.ruleset) return [];
    const groups = groupArtsByType(store.ruleset);
    const techniques = groups.find((g) => g.artType === 'technique')?.arts ?? [];
    const forms = groups.find((g) => g.artType === 'form')?.arts ?? [];
    const half = Math.ceil(forms.length / 2);
    return [
      { labelKey: 'art-type-technique', arts: techniques },
      { labelKey: 'art-type-form', arts: forms.slice(0, half) },
      { labelKey: 'art-type-form', arts: forms.slice(half) },
    ];
  });

  function name(artId: string): string {
    return store.ruleset ? artLabel(store.ruleset, artId) : artId;
  }

  function abbr(artId: string): string {
    return store.ruleset ? artAbbreviation(store.ruleset, artId) : '';
  }

  function scoreOf(artId: string): number {
    return store.entity.art_scores?.find((a) => a.art === artId)?.score ?? 0;
  }

  function bonusOf(artId: string): number {
    return store.effective?.art_bonuses?.find((b) => b.art === artId)?.bonus ?? 0;
  }

  function tip(artId: string): TooltipContent {
    return { text: store.ruleset?.i18n[artId]?.description ?? undefined };
  }
</script>

<section class="panel art-grid">
  {#if store.ruleset}
    {#each columns as column, c (c)}
      <div class="art-column">
        <h3 class="category">{store.t(column.labelKey)}</h3>
        <ul class="art-list">
          {#each column.arts as art (art.id)}
            {@const score = scoreOf(art.id)}
            {@const bonus = bonusOf(art.id)}
            <li>
              <span class="item-name" use:tooltip={tip(art.id)}>
                {name(art.id)}{#if abbr(art.id)}<span class="art-abbr">({abbr(art.id)})</span>{/if}
              </span>
              <span class="spinner">
                <button
                  type="button"
                  class="icon-btn"
                  aria-label={store.t('art-decrement')}
                  disabled={score <= 0}
                  onclick={() => store.adjustArt(art.id, -1, max)}
                  data-testid="art-dec-{art.id}"
                >
                  -
                </button>
                <span class="spinner-value" data-testid="art-score-{art.id}">{score}</span>
                <button
                  type="button"
                  class="icon-btn"
                  aria-label={store.t('art-increment')}
                  disabled={score >= max}
                  onclick={() => store.adjustArt(art.id, 1, max)}
                  data-testid="art-inc-{art.id}"
                >
                  +
                </button>
              </span>
              {#if bonus !== 0}
                <span class="eff-slot">
                  <span class="eff-badge" data-testid="art-eff-{art.id}">
                    {store.t('effective-score', { score: String(score + bonus) })}
                  </span>
                </span>
              {:else}
                <span class="eff-slot" aria-hidden="true"></span>
              {/if}
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
