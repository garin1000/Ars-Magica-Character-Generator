<script lang="ts">
  import { store } from '../state.svelte';
  import { artAbbreviation, artLabel, groupArtsByType, maxArtScore } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';
  import type { Art } from '../types';
  import Spinner from './Spinner.svelte';

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

  // The bought score `bonusOf` was computed against — NOT the live one the spinner
  // shows (#16). The spinner is direct feedback and must move on the keystroke; the
  // badge is a bought+bonus pair, and mixing a fresh half with a stale one renders a
  // total that is true of no character. @see AppStore.readSettled
  function settledScoreOf(artId: string): number {
    return store.readSettled((e) => e.art_scores?.find((a) => a.art === artId)?.score ?? 0);
  }

  function tip(artId: string): TooltipContent {
    return { text: store.ruleset?.i18n[artId]?.description ?? undefined };
  }
</script>

<!-- S17 (full-audit a11y): the tab's own <h2> — the columns below stayed <h3>
     directly under the app's single <h1> with no <h2> between (a heading
     hierarchy gap) until this was added. Reuses `tab-arts`, the same label the
     App.svelte tab button already carries, so it names nothing the user does
     not already read on the tab strip. -->
<section class="panel">
  <h2>{store.t('tab-arts')}</h2>
  {#if store.ruleset}
    <div class="art-grid">
      {#each columns as column, c (c)}
        <div class="art-column">
          <h3 class="category">{store.t(column.labelKey)}</h3>
          <ul class="art-list">
            {#each column.arts as art (art.id)}
              {@const score = scoreOf(art.id)}
              {@const bonus = bonusOf(art.id)}
              <li>
                <span class="item-name" use:tooltip={tip(art.id)}>
                  {name(art.id)}{#if abbr(art.id)}<span class="art-abbr">({abbr(art.id)})</span
                    >{/if}
                </span>
                <Spinner
                  decLabel={store.t('art-decrement', { name: name(art.id) })}
                  decTestid="art-dec-{art.id}"
                  decDisabled={score <= 0}
                  onDec={() => store.adjustArt(art.id, -1, max)}
                  incLabel={store.t('art-increment', { name: name(art.id) })}
                  incTestid="art-inc-{art.id}"
                  incDisabled={score >= max}
                  onInc={() => store.adjustArt(art.id, 1, max)}
                >
                  {#snippet children()}
                    <span class="spinner-value" data-testid="art-score-{art.id}">{score}</span>
                  {/snippet}
                </Spinner>
                {#if bonus !== 0}
                  <span class="eff-slot">
                    <span class="eff-badge" data-testid="art-eff-{art.id}">
                      {store.t('effective-score', {
                        score: String(settledScoreOf(art.id) + bonus),
                      })}
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
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
