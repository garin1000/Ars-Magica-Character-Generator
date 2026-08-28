<script lang="ts">
  import { store } from '../state.svelte';
  import { requirementAbilityLabel } from '../derive';
  import type { MagusMinimumAbility } from '../types';

  // What the Order demands of every magus, as a checklist.
  //
  // "Magi must have the following minimum Abilities: Parma Magica 1, Magic Theory 1,
  // Latin 1. Characters with lower scores would not be admitted to the Order."
  // Source: Ars Magica - Definitive Edition (Core Rules).md:2437
  //
  // The recommended package below it is advice, priced at "Total Cost: 90 experience
  // points" (`:2451-2461`), so falling short of it is a warning and not a refusal.
  //
  // The rows are the ENGINE's — the same reading its `magus_minimum_ability` /
  // `magus_recommended_ability` findings come from, so the list and the findings can
  // never disagree — and they are empty for every type but a magus. That is the whole
  // gate: this component needs no `is_magus` test of its own.
  const localized = $derived(store.ruleset);
  const rows = $derived(store.effective?.magus_minimum_abilities ?? []);
  const required = $derived(rows.filter((row) => row.requirement === 'required'));
  const recommended = $derived(rows.filter((row) => row.requirement === 'recommended'));
  const unmet = $derived(rows.filter((row) => !row.met).length);
  // What the recommended package costs, off the rules data rather than a literal in a
  // locale file — the number lives in `rules/core/life_stages.json`.
  const recommendedXp = $derived(
    localized?.ruleset.life_stages?.apprenticeship?.recommended_xp ?? null,
  );

  /**
   * The instance of a parameterized Ability the row is about: the one the requirement
   * names, or else the highest-scoring instance the character actually bought — the
   * very row the engine's `score` was read from. Without it a "Latin 1" row could only
   * show the "(Language)" hint while the character plainly has Latin.
   */
  function instanceOf(row: MagusMinimumAbility): string | null {
    if (row.parameter) return row.parameter;
    let best: { score: number; parameter: string | null } | null = null;
    for (const bought of store.entity.ability_scores ?? []) {
      if (bought.ability !== row.ability) continue;
      if (!best || bought.score > best.score) {
        best = { score: bought.score, parameter: bought.parameter ?? null };
      }
    }
    return best?.parameter ?? null;
  }

  /**
   * The row as a whole sentence: which Ability, at what score, met or not.
   *
   * The Ability label goes through `requirementAbilityLabel`, the same path the
   * `issue-magus_minimum_ability` message takes, so the checklist and the finding
   * word the demand identically. That is also where the rules' own exemplar is
   * named — `:2437` says "Latin 1" while the enforced check is "any Dead
   * Language 1", so a magus is told what the rules mean by it.
   */
  function statusOf(row: MagusMinimumAbility): string {
    if (!localized) return '';
    return store.t(row.met ? 'magus-minimum-met' : 'magus-minimum-unmet', {
      ability: requirementAbilityLabel(
        localized,
        row.ability,
        instanceOf(row),
        row.exemplar,
        store.t,
      ),
      min: String(row.min_score),
      score: String(row.score),
    });
  }
</script>

{#if localized && rows.length > 0}
  <!-- h3: the Available/Selected region titles own the h2 level on this tab. -->
  <section class="magus-minimums" data-testid="magus-minimums">
    <h3>{store.t('magus-minimums-label')}</h3>
    <!-- ONE live region for the whole checklist: a row-level one would announce a
         sentence per row on every keystroke in the Ability list. -->
    <p class="magus-minimums-summary" role="status" data-testid="magus-minimums-summary">
      {store.t('magus-minimums-summary', { unmet: String(unmet), total: String(rows.length) })}
    </p>
    <ul class="magus-minimum-list">
      {#each required as row (row.ability)}
        <!-- The status is in the sentence itself; `data-met` only mirrors it for
             styling and for the e2e suite, and never carries it alone. -->
        <li data-met={row.met} data-testid="magus-minimum-{row.ability}">{statusOf(row)}</li>
      {/each}
    </ul>
    {#if recommended.length > 0}
      <h3>{store.t('magus-recommended-label')}</h3>
      <ul class="magus-minimum-list">
        {#each recommended as row (row.ability)}
          <li data-met={row.met} data-testid="magus-recommended-{row.ability}">{statusOf(row)}</li>
        {/each}
      </ul>
      {#if recommendedXp != null}
        <p class="magus-recommended-hint" data-testid="magus-recommended-hint">
          {store.t('magus-recommended-hint', { xp: String(recommendedXp) })}
        </p>
      {/if}
    {/if}
  </section>
{/if}
