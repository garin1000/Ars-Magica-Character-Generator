<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityLabel, filterAbilities, groupAbilitiesByCategory } from '../derive';
  import { tooltip, type TooltipContent } from '../actions';
  import type { AbilityCategory } from '../types';

  // Filter state (free-text + category) lives on the store, so it survives tab
  // switches that unmount this component. Categories from the engine-surfaced order.
  const filter = $derived(store.filters.abilities);
  const categories = $derived(store.ruleset?.ruleset.ability_category_order ?? []);

  const groups = $derived.by(() => {
    const rs = store.ruleset;
    if (!rs) return [];
    const abilityFilter = {
      text: filter.search,
      categories: filter.category ? [filter.category as AbilityCategory] : undefined,
    };
    return groupAbilitiesByCategory(rs)
      .map((g) => ({
        category: g.category,
        abilities: filterAbilities(rs, g.abilities, abilityFilter, store.t),
      }))
      .filter((g) => g.abilities.length > 0);
  });
  const selected = $derived(new Set((store.entity.ability_scores ?? []).map((a) => a.ability)));

  // Supernatural Abilities are unlocked by a granting Virtue (one whose
  // `ability_score_grant` effect targets them) or by the Gift's one free slot.
  // The picker greys any Supernatural Ability the character can't currently take,
  // with a "requires a Virtue" reason. "Granted" is read straight from the
  // selected virtues (synchronous, like `selected`) so a just-added granting
  // Virtue enables its Ability without waiting for the async effective scores.
  const grantedSupernatural = $derived.by(() => {
    const granted = new Set<string>();
    const items = store.ruleset?.ruleset.point_items ?? {};
    for (const selection of store.entity.selections) {
      for (const effect of items[selection.ref]?.effects ?? []) {
        if (effect.type === 'ability_score_grant') granted.add(effect.ability);
      }
    }
    return granted;
  });
  const freeSupernaturalSlot = $derived(
    (store.effective?.supernatural_free_used ?? 0) <
      (store.effective?.supernatural_free_total ?? 0),
  );

  // A parameterized ability (e.g. (Area) Lore) can be selected repeatedly; a plain
  // one only once.
  function isParameterized(abilityId: string): boolean {
    return !!store.ruleset?.ruleset.abilities?.[abilityId]?.parameter;
  }

  function isSupernatural(abilityId: string): boolean {
    return store.ruleset?.ruleset.abilities?.[abilityId]?.category === 'supernatural';
  }

  // A Supernatural Ability with no granting Virtue and no free Gift slot cannot
  // be taken right now.
  function supernaturalLocked(abilityId: string): boolean {
    return (
      isSupernatural(abilityId) &&
      !grantedSupernatural.has(abilityId) &&
      !selected.has(abilityId) &&
      !freeSupernaturalSlot
    );
  }

  function isDisabled(abilityId: string): boolean {
    return (
      (!isParameterized(abilityId) && selected.has(abilityId)) || supernaturalLocked(abilityId)
    );
  }

  function name(abilityId: string): string {
    if (!store.ruleset) return abilityId;
    return abilityLabel(
      store.ruleset,
      abilityId,
      undefined,
      (key) => store.t('param-hint', { label: store.t(`param-label-${key}`) }),
      store.t('ability-requires-training-marker'),
    );
  }

  // Description + example specialties surface as a hover/focus tooltip, keeping
  // each row a single compact line.
  function tip(abilityId: string): TooltipContent {
    // A locked Supernatural Ability explains why, instead of its description.
    if (supernaturalLocked(abilityId)) {
      return { text: store.t('ability-requires-virtue') };
    }
    const entry = store.ruleset?.i18n[abilityId];
    return {
      text: entry?.description ?? undefined,
      listLabel: store.t('ability-specialties-label'),
      list: entry?.specialties ?? [],
    };
  }
</script>

<section class="panel">
  {#if store.ruleset}
    <div class="filter-bar">
      <input
        type="search"
        class="filter-search"
        placeholder={store.t('filter-search-placeholder')}
        bind:value={filter.search}
        data-testid="ability-search"
      />
      <select bind:value={filter.category} data-testid="ability-category-filter">
        <option value="">{store.t('filter-category-all')}</option>
        {#each categories as c (c)}
          <option value={c}>{store.t(`ability-category-${c}`)}</option>
        {/each}
      </select>
    </div>
    <div class="list-scroll">
      {#each groups as group (group.category)}
        <h3 class="category">{store.t(`ability-category-${group.category}`)}</h3>
        <ul class="item-list">
          {#each group.abilities as ability (ability.id)}
            <li>
              <button
                type="button"
                class="pick-row"
                disabled={isDisabled(ability.id)}
                onclick={() => store.addAbility(ability.id)}
                use:tooltip={tip(ability.id)}
                data-testid="add-{ability.id}"
              >
                <span class="item-name">{name(ability.id)}</span>
                <span class="pick-plus" aria-hidden="true">+</span>
              </button>
            </li>
          {/each}
        </ul>
      {/each}
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
