<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityDisplayName, groupAbilitiesByCategory } from '../derive';

  const groups = $derived(store.ruleset ? groupAbilitiesByCategory(store.ruleset) : []);
  const selected = $derived(new Set((store.entity.ability_scores ?? []).map((a) => a.ability)));

  // A parameterized ability (e.g. (Area) Lore) can be selected repeatedly; a plain
  // one only once.
  function isParameterized(abilityId: string): boolean {
    return !!store.ruleset?.ruleset.abilities?.[abilityId]?.parameter;
  }

  function name(abilityId: string): string {
    if (!store.ruleset) return abilityId;
    return abilityDisplayName(store.ruleset, abilityId, undefined, (key) =>
      store.t('param-hint', { label: store.t(`param-label-${key}`) }),
    );
  }
</script>

<section class="panel">
  {#if store.ruleset}
    {#each groups as group (group.category)}
      <h3 class="category">{store.t(`ability-category-${group.category}`)}</h3>
      <ul class="item-list">
        {#each group.abilities as ability (ability.id)}
          <li>
            <button
              type="button"
              class="pick-row"
              disabled={!isParameterized(ability.id) && selected.has(ability.id)}
              onclick={() => store.addAbility(ability.id)}
              data-testid="add-{ability.id}"
            >
              <span class="item-name">{name(ability.id)}</span>
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
