<script lang="ts">
  import { store } from '../state.svelte';
  import { artAbbreviation, artLabel, groupArtsByType } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';

  const groups = $derived(store.ruleset ? groupArtsByType(store.ruleset) : []);
  const selected = $derived(new Set((store.entity.art_scores ?? []).map((a) => a.art)));

  function name(artId: string): string {
    return store.ruleset ? artLabel(store.ruleset, artId) : artId;
  }

  function abbr(artId: string): string {
    return store.ruleset ? artAbbreviation(store.ruleset, artId) : '';
  }

  // Description surfaces as a hover/focus tooltip, keeping each row compact.
  function tip(artId: string): TooltipContent {
    return { text: store.ruleset?.i18n[artId]?.description ?? undefined };
  }
</script>

<section class="panel">
  {#if store.ruleset}
    {#each groups as group (group.artType)}
      <h3 class="category">{store.t(`art-type-${group.artType}`)}</h3>
      <ul class="item-list">
        {#each group.arts as art (art.id)}
          <li>
            <button
              type="button"
              class="pick-row"
              disabled={selected.has(art.id)}
              onclick={() => store.addArt(art.id)}
              use:tooltip={tip(art.id)}
              data-testid="add-{art.id}"
            >
              <span class="item-name">{name(art.id)}</span>
              {#if abbr(art.id)}
                <span class="art-abbr" aria-hidden="true">{abbr(art.id)}</span>
              {/if}
              <span class="pick-plus" aria-hidden="true">+</span>
            </button>
          </li>
        {/each}
      </ul>
    {/each}
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
