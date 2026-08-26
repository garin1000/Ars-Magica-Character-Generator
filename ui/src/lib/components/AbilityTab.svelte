<script lang="ts">
  import { store } from '../state.svelte';
  import {
    abilityDisplayName,
    abilityLabel,
    filterAbilities,
    groupAbilitiesByCategory,
    groupAbilitySelectionsByCategory,
    invalidSelectionIds,
    maxAbilityScore,
    type IndexedAbilityScore,
  } from '../derive';
  import { tooltip, withReason, type TooltipContent } from '../actions';
  import type { Ability, AbilityCategory } from '../types';
  import MagusMinimumAbilities from './MagusMinimumAbilities.svelte';
  import SourcePicker from './SourcePicker.svelte';
  import SelectionList from './SelectionList.svelte';

  // === Source (Available) side ===

  // Filter state (free-text + category) lives on the store, so it survives tab
  // switches that unmount this component. Categories from the engine-surfaced order.
  const filter = $derived(store.filters.abilities);
  const categories = $derived(store.ruleset?.ruleset.ability_category_order ?? []);

  const sourceGroups = $derived.by(() => {
    const rs = store.ruleset;
    if (!rs) return [];
    const abilityFilter = {
      text: filter.search,
      categories: filter.category ? [filter.category as AbilityCategory] : undefined,
    };
    return groupAbilitiesByCategory(rs)
      .map((g) => ({
        key: g.category,
        header: store.t(`ability-category-${g.category}`),
        items: filterAbilities(rs, g.abilities, abilityFilter, store.t),
      }))
      .filter((g) => g.items.length > 0);
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
    for (const selection of store.entity.selections ?? []) {
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

  function sourceName(abilityId: string): string {
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
  function sourceTip(abilityId: string): TooltipContent {
    const entry = store.ruleset?.i18n[abilityId];
    // A locked Supernatural Ability explains WHY above its normal description,
    // not instead of it.
    return withReason(
      {
        text: entry?.description ?? undefined,
        listLabel: store.t('ability-specialties-label'),
        list: entry?.specialties ?? [],
      },
      supernaturalLocked(abilityId) ? store.t('ability-requires-virtue') : undefined,
    );
  }

  // === Selected side ===

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

  // Abilities an error-severity issue points at (e.g. a supernatural ability whose
  // granting Virtue was removed after it was bought) — their rows render red.
  const invalidIds = $derived(invalidSelectionIds(store.result));

  const selectedColumns = $derived([
    {
      key: 'abilities',
      empty: scores.length === 0,
      emptyText: store.t('empty-selections-side'),
      emptyClass: 'empty',
      groups: groupedScores.map((g) => ({
        key: g.category,
        header: store.t(`ability-category-${g.category}`),
        listClass: 'selection-list ability-selection',
        rows: g.entries.map((e) => ({ key: e.index, item: e })),
      })),
    },
  ]);

  function paramKey(abilityId: string): string | undefined {
    return store.ruleset?.ruleset.abilities?.[abilityId]?.parameter ?? undefined;
  }

  function selectedName(abilityId: string, value: string | null | undefined): string {
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
  function selectedTip(abilityId: string): TooltipContent {
    const entry = store.ruleset?.i18n[abilityId];
    return {
      text: entry?.description ?? undefined,
      listLabel: store.t('ability-specialties-label'),
      list: entry?.specialties ?? [],
    };
  }
</script>

<!-- What the Order demands of a magus (Core Rules.md:2437), read before the lists it
     is about — an auto-height SIBLING of `.region-row`, never a wrapper: the row must
     stay the only `flex: 1` child of `.vf-tab`, or the Available/Selected lists
     collapse.

     How Abilities are *funded* is no longer here. Slice 2 (guided-creation review
     #11) moved `LifeStagePanel` to the wizard's own `experience` step
     (`ExperienceStep.svelte`), because this tab was carrying three concerns in one
     bounded column and only the third needed height. Empty for every type but a
     magus, so nothing gates the checklist here. -->
<MagusMinimumAbilities />

<div class="region-row">
  <section class="region region-source">
    <h2 class="region-title">{store.t('available-title')}</h2>
    <SourcePicker
      groups={sourceGroups}
      getId={(a: Ability) => a.id}
      onAdd={(a: Ability) => store.addAbility(a.id)}
      disabled={(a: Ability) => isDisabled(a.id)}
      tip={(a: Ability) => sourceTip(a.id)}
    >
      {#snippet filters()}
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
      {/snippet}
      {#snippet row(a: Ability)}
        <span class="item-name">{sourceName(a.id)}</span>
      {/snippet}
    </SourcePicker>
  </section>
  <section class="region region-selected">
    <h2 class="region-title">{store.t('selections-title')}</h2>
    <div class="selected-frame">
      <!-- The frame carries the border and its padding; this inner box does the
           scrolling, so the padding stays a gap the rows cannot scroll into. -->
      <div class="selected-scroll">
        <SelectionList columns={selectedColumns}>
          {#snippet row(item: IndexedAbilityScore)}
            {@const entry = item.entry}
            {@const i = item.index}
            {@const key = paramKey(entry.ability)}
            <li class:invalid-selection={invalidIds.has(entry.ability)}>
              <span class="item-name" use:tooltip={selectedTip(entry.ability)}
                >{selectedName(entry.ability, entry.parameter)}</span
              >
              <span class="spinner">
                <button
                  type="button"
                  class="icon-btn"
                  aria-label={store.t('ability-decrement', {
                    name: selectedName(entry.ability, entry.parameter),
                  })}
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
                  aria-label={store.t('ability-increment', {
                    name: selectedName(entry.ability, entry.parameter),
                  })}
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
                aria-label={store.t('remove-item', {
                  name: selectedName(entry.ability, entry.parameter),
                })}
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
          {/snippet}
        </SelectionList>
      </div>
    </div>
  </section>
</div>
