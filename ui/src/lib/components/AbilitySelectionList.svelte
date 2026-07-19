<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityDisplayName, groupAbilitySelectionsByCategory, maxAbilityScore } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';

  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  const max = $derived(maxAbilityScore(advancement));
  const scores = $derived(store.entity.ability_scores ?? []);

  // Bought abilities grouped by category and alpha-sorted within each group
  // (mirroring the picker). Original indices ride along for spinner/remove wiring.
  const groupedScores = $derived(
    store.ruleset
      ? groupAbilitySelectionsByCategory(
          store.ruleset,
          scores.map((entry, index) => ({ entry, index })),
        )
      : [],
  );

  function paramKey(abilityId: string): string | undefined {
    return store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
  }

  function name(abilityId: string, value: string | null | undefined): string {
    if (!store.ruleset) return abilityId;
    return abilityDisplayName(store.ruleset, abilityId, value, (key) =>
      store.t('param-hint', { label: store.t(`param-label-${key}`) }),
    );
  }

  function bonusOf(abilityId: string, parameter: string | null | undefined): number {
    return (
      store.effective?.ability_bonuses?.find(
        (b) => b.ability === abilityId && (b.parameter ?? null) === (parameter ?? null),
      )?.bonus ?? 0
    );
  }

  // A virtue-granted free starting score (e.g. Second Sight 1) is a floor on the
  // bought score, so it raises the effective score; granted abilities are plain.
  function floorOf(abilityId: string, parameter: string | null | undefined): number {
    if (parameter != null) return 0;
    return store.effective?.ability_score_floors?.find((f) => f.ability === abilityId)?.floor ?? 0;
  }

  // The effective score shown: max(bought, granted floor) + bonus.
  function effectiveOf(
    score: number,
    abilityId: string,
    parameter: string | null | undefined,
  ): number {
    return Math.max(score, floorOf(abilityId, parameter)) + bonusOf(abilityId, parameter);
  }

  // Description + example specialties as a hover/focus tooltip, matching the picker.
  function tip(abilityId: string): TooltipContent {
    const entry = store.ruleset?.i18n[abilityId];
    return {
      text: entry?.description ?? undefined,
      listLabel: store.t('ability-specialties-label'),
      list: entry?.specialties ?? [],
    };
  }
</script>

<section class="panel">
  {#if scores.length === 0}
    <p class="empty">{store.t('empty-selections-side')}</p>
  {:else}
    {#each groupedScores as group (group.category)}
      <h3 class="category">{store.t(`ability-category-${group.category}`)}</h3>
      <ul class="selection-list ability-selection">
        <!-- Key by the stable original index, never the mutable parameter/specialty:
             keying on a value the row's own inputs edit would recreate the input
             on every keystroke (losing focus, truncating typed area/language names)
             and collide for two fresh instances that both start blank. -->
        {#each group.entries as { entry, index: i } (i)}
          {@const key = paramKey(entry.ability)}
          <li>
            <span class="item-name" use:tooltip={tip(entry.ability)}
              >{name(entry.ability, entry.parameter)}</span
            >
            <span class="spinner">
              <button
                type="button"
                class="icon-btn"
                aria-label={store.t('ability-decrement')}
                disabled={entry.score <= 0}
                onclick={() => store.adjustAbilityAt(i, -1, max)}
                data-testid="ability-dec-{entry.ability}-{i}"
              >
                -
              </button>
              <span class="spinner-value" data-testid="ability-score-{entry.ability}-{i}">
                {entry.score}
              </span>
              <button
                type="button"
                class="icon-btn"
                aria-label={store.t('ability-increment')}
                disabled={entry.score >= max}
                onclick={() => store.adjustAbilityAt(i, 1, max)}
                data-testid="ability-inc-{entry.ability}-{i}"
              >
                +
              </button>
            </span>
            {#if effectiveOf(entry.score, entry.ability, entry.parameter) !== entry.score}
              <span class="eff-slot">
                <span class="eff-badge" data-testid="ability-eff-{entry.ability}-{i}">
                  {store.t('effective-score', {
                    score: String(effectiveOf(entry.score, entry.ability, entry.parameter)),
                  })}
                </span>
              </span>
            {:else}
              <span class="eff-slot" aria-hidden="true"></span>
            {/if}
            <input
              type="text"
              class="specialty"
              placeholder={store.t('ability-specialty-label')}
              value={entry.specialty ?? ''}
              oninput={(e) =>
                store.setAbilitySpecialtyAt(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="ability-specialty-{entry.ability}-{i}"
            />
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('ability-decrement')}
              onclick={() => store.removeAbilityAt(i)}
              data-testid="remove-{entry.ability}-{i}"
            >
              ×
            </button>
            {#if key}
              <input
                type="text"
                class="ability-param"
                placeholder={store.t(`param-label-${key}`)}
                value={entry.parameter ?? ''}
                oninput={(e) =>
                  store.setAbilityParameterAt(i, (e.currentTarget as HTMLInputElement).value)}
                data-testid="ability-param-{entry.ability}-{i}"
              />
            {/if}
          </li>
        {/each}
      </ul>
    {/each}
  {/if}
</section>
