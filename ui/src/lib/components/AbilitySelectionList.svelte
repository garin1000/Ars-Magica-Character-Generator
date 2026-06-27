<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityDisplayName, maxAbilityScore } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';

  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  const max = $derived(maxAbilityScore(advancement));
  const scores = $derived(store.entity.ability_scores ?? []);

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
    <ul class="selection-list ability-selection">
      {#each scores as entry, i (i)}
        {@const key = paramKey(entry.ability)}
        <li>
          <span class="item-name" use:tooltip={tip(entry.ability)}
            >{name(entry.ability, entry.parameter)}</span
          >
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
          <span class="spinner">
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('ability-decrement')}
              disabled={entry.score <= 0}
              onclick={() => store.adjustAbilityAt(i, -1, max)}
              data-testid="ability-dec-{entry.ability}-{i}"
            >
              −
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
            {#if bonusOf(entry.ability, entry.parameter) !== 0}
              <span class="eff-badge" data-testid="ability-eff-{entry.ability}-{i}">
                {store.t('effective-score', {
                  score: String(entry.score + bonusOf(entry.ability, entry.parameter)),
                })}
              </span>
            {/if}
          </span>
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
        </li>
      {/each}
    </ul>
  {/if}
</section>
