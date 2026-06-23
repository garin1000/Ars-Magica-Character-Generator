<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityXpSpent, groupAbilitiesByCategory } from '../derive';
  import type { AbilityScore } from '../types';

  const groups = $derived(store.ruleset ? groupAbilitiesByCategory(store.ruleset) : []);
  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  // Whole-point scores the advancement table can price, ascending (0 included).
  const scoreOptions = $derived([0, ...advancement.map((r) => r.score).sort((a, b) => a - b)]);
  const xpSpent = $derived(abilityXpSpent(advancement, store.entity.ability_scores));

  function entryFor(abilityId: string): AbilityScore | undefined {
    return store.entity.ability_scores?.find((a) => a.ability === abilityId);
  }

  function name(abilityId: string): string {
    return store.ruleset?.i18n[abilityId]?.name ?? abilityId;
  }

  function onScore(abilityId: string, event: Event) {
    const score = Number((event.currentTarget as HTMLSelectElement).value);
    store.setAbility(abilityId, score, entryFor(abilityId)?.specialty ?? undefined);
  }

  function onSpecialty(abilityId: string, event: Event) {
    const specialty = (event.currentTarget as HTMLInputElement).value;
    const score = entryFor(abilityId)?.score ?? 0;
    // Only persist a specialty once the ability has a score to attach it to.
    if (score > 0) store.setAbility(abilityId, score, specialty);
  }

  function onBank(event: Event) {
    store.setUnspentXp(Number((event.currentTarget as HTMLInputElement).value));
  }
</script>

<section class="panel">
  <h2>{store.t('abilities-title')}</h2>
  {#if store.ruleset}
    <p class="points" data-testid="ability-xp-spent">
      {store.t('characteristic-points', { used: String(xpSpent), budget: '—' })}
    </p>
    <label class="bank">
      <span>{store.t('unspent-xp')}</span>
      <input
        type="number"
        min="0"
        value={store.entity.unspent_xp ?? 0}
        oninput={onBank}
        data-testid="unspent-xp"
      />
    </label>

    {#each groups as group (group.category)}
      <h3 class="category">{store.t(`ability-category-${group.category}`)}</h3>
      <ul class="ability-list">
        {#each group.abilities as ability (ability.id)}
          {@const entry = entryFor(ability.id)}
          <li>
            <span class="ability-name">{name(ability.id)}</span>
            <select
              aria-label={store.t('ability-score-label')}
              value={entry?.score ?? 0}
              onchange={(e) => onScore(ability.id, e)}
              data-testid="ability-score-{ability.id}"
            >
              {#each scoreOptions as score (score)}
                <option value={score}>{score}</option>
              {/each}
            </select>
            <input
              type="text"
              class="specialty"
              placeholder={store.t('ability-specialty-label')}
              value={entry?.specialty ?? ''}
              disabled={!entry}
              oninput={(e) => onSpecialty(ability.id, e)}
              data-testid="ability-specialty-{ability.id}"
            />
          </li>
        {/each}
      </ul>
    {/each}
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
