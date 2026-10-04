<script lang="ts">
  import { store } from '../state.svelte';
  import {
    artAbbreviation,
    artLabel,
    effectiveBreakdownTooltip,
    groupArtsByType,
    maxArtScore,
    rowWarningFor,
    rowWarnings,
    rowWarningText,
    scoreSourceLines,
  } from '../derive';
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

  // X10b: the "Z" of the book's own "X (Z)" notation (ArMDE:1179) — XP already
  // banked toward the next score.
  function bankedXpOf(artId: string): number {
    return store.entity.art_scores?.find((a) => a.art === artId)?.banked_xp ?? 0;
  }

  // The bought score `bonusOf` was computed against — NOT the live one the spinner
  // shows (#16). The spinner is direct feedback and must move on the keystroke; the
  // badge is a bought+bonus pair, and mixing a fresh half with a stale one renders a
  // total that is true of no character. @see AppStore.readSettled
  function settledScoreOf(artId: string): number {
    return store.readSettled((e) => e.art_scores?.find((a) => a.art === artId)?.score ?? 0);
  }

  // Banked-XP warnings (I2): marked on the Art's own row, by id.
  const warnings = $derived(rowWarnings(store.result));

  // I3: why the effective badge differs from the bought score — each source item
  // the engine names (Puissant Art, Elemental Magic), by localized name.
  function effectiveTip(artId: string): TooltipContent {
    const sources = store.effective?.art_bonus_sources?.find((s) => s.art === artId)?.sources;
    const bought = settledScoreOf(artId);
    return effectiveBreakdownTooltip(
      { bought: String(bought), effective: String(bought + bonusOf(artId)) },
      store.ruleset ? scoreSourceLines(sources ?? [], store.ruleset, store.t) : [],
      store.t,
    );
  }

  function tip(artId: string): TooltipContent {
    return { text: store.ruleset?.i18n[artId]?.description ?? undefined };
  }
</script>

<!-- S17 (full-audit a11y): the tab's own <h2> — the columns below stayed <h3>
     directly under the app's single <h1> with no <h2> between (a heading
     hierarchy gap) until this was added. Reuses `tab-arts`, the same label the
     App.svelte tab button already carries, so it names nothing the user does
     not already read on the tab strip. `art-panel` hugs the grid and centres
     itself, as `char-panel` does for the characteristics (N6). -->
<section class="panel art-panel">
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
              {@const warning = rowWarningFor(warnings, art.id, null, {})}
              <li>
                {#if warning}
                  <!-- I2: a banked-XP warning on this Art — a warning glyph (text
                       presentation, U+FE0E) plus the finding in words for a screen
                       reader, so it is never carried by colour alone. -->
                  <span class="row-warning-glyph" aria-hidden="true">&#x26A0;&#xFE0E;</span>
                  <span class="sr-only">{rowWarningText(store.ruleset, warning, store.t)}</span>
                {/if}
                <!-- Deliberately focusable: `use:tooltip` opens on `focusin`, so this
                     is the only thing standing between a keyboard user and the Art's
                     rules text — which this popup is the app's ONLY rendering of
                     (Sabine 3). Matches DerivedLabCastingSection's own `<dt>`. -->
                <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                <span class="item-name" tabindex="0" use:tooltip={tip(art.id)}>
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
                <input
                  type="number"
                  min="0"
                  class="spinner-value-input banked-xp"
                  class:banked-xp-zero={bankedXpOf(art.id) === 0}
                  aria-label={store.t('art-banked-xp-label')}
                  value={bankedXpOf(art.id)}
                  oninput={(e) =>
                    store.setArtBankedXp(
                      art.id,
                      Number((e.currentTarget as HTMLInputElement).value),
                    )}
                  data-testid="art-banked-xp-{art.id}"
                /><span class="banked-xp-unit" aria-hidden="true">{store.t('xp-unit-abbr')}</span>
                {#if bonus !== 0}
                  <span class="eff-slot">
                    <!-- Deliberately focusable: which Virtue moved the score lives only
                         in this tooltip, and `use:tooltip` opens it on `focusin` (I3). -->
                    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                    <span
                      class="eff-badge"
                      data-testid="art-eff-{art.id}"
                      tabindex="0"
                      use:tooltip={effectiveTip(art.id)}
                    >
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
