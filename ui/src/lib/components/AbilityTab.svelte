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
    unboughtModifiedAbilities,
    UNBOUGHT_ROW_INDEX,
    type IndexedAbilityScore,
  } from '../derive';
  import { tooltip, withReason, type TooltipContent } from '../actions';
  import type { Ability, AbilityCategory, AbilityScore } from '../types';
  import MagusMinimumAbilities from './MagusMinimumAbilities.svelte';
  import SourcePicker from './SourcePicker.svelte';
  import SelectionList from './SelectionList.svelte';
  import Spinner from './Spinner.svelte';

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

  // The engine's age → max-Ability-score cap. Its one home since Slice 12 (#24):
  // it used to be echoed on both the Details tab and the life-stage panel, neither
  // of which shows an Ability score. It constrains the lists below, so it is read
  // beside them.
  const ageCap = $derived(store.effective?.age_ability_cap ?? null);

  // === Selected side ===

  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  const max = $derived(maxAbilityScore(advancement));
  const scores = $derived(store.entity.ability_scores ?? []);

  // Display-only rows for an Ability the engine reports a bonus or a granted floor
  // for that has no bought row to carry it (#17) — a Puissant Ability applies at 0
  // bought points, and without this the tab was silent about it. They go through
  // the same grouping below, so each lands under its own category header exactly
  // where its bought row will appear once it is bought.
  const unboughtRows = $derived(
    unboughtModifiedAbilities(
      scores,
      store.effective?.ability_bonuses ?? [],
      store.effective?.ability_score_floors ?? [],
    ),
  );

  // Bought abilities (plus the unbought-but-modified rows) grouped by category and
  // alpha-sorted within each group (mirroring the picker). Original indices ride
  // along for spinner/remove wiring.
  const groupedScores = $derived(
    store.ruleset
      ? groupAbilitySelectionsByCategory(store.ruleset, [
          ...scores.map((entry, index) => ({ entry, index })),
          ...unboughtRows,
        ])
      : [],
  );

  // Abilities an error-severity issue points at (e.g. a supernatural ability whose
  // granting Virtue was removed after it was bought) — their rows render red.
  const invalidIds = $derived(invalidSelectionIds(store.result));

  const selectedColumns = $derived([
    {
      key: 'abilities',
      // An unbought-but-modified row is still something to show, so the "nothing
      // selected" message yields to it.
      empty: scores.length === 0 && unboughtRows.length === 0,
      emptyText: store.t('empty-selections-side'),
      emptyClass: 'empty',
      groups: groupedScores.map((g) => ({
        key: g.category,
        header: store.t(`ability-category-${g.category}`),
        listClass: 'selection-list ability-selection',
        // Unbought rows all share UNBOUGHT_ROW_INDEX, so they are keyed by ability
        // id instead — `{#each}` keys must stay unique.
        rows: g.entries.map((e) => ({ key: rowKey(e), item: e })),
      })),
    },
  ]);

  // A row's identity in `{#each}` keys and in test ids: its entity-array index, or
  // the ability id / the literal `unbought` for a display-only row, which has no
  // index. Never a bare index for those — `ability-eff-<id>-0` must keep meaning
  // "the first bought row", which several e2e specs address by exact index.
  function rowKey(item: IndexedAbilityScore): string | number {
    return item.index === UNBOUGHT_ROW_INDEX ? `unbought-${item.entry.ability}` : item.index;
  }

  function rowSuffix(index: number): string {
    return index === UNBOUGHT_ROW_INDEX ? 'unbought' : String(index);
  }

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

  // The bought score `bonusOf`/`floorOf` were computed against — NOT the live one
  // the spinner shows (#16). The spinner is direct feedback and moves on the
  // keystroke; the badge is a bought+modifier pair, and mixing a fresh half with a
  // stale one renders a total true of no character. Matched by ability + parameter
  // rather than by row index, so the pairing survives a row being removed above it.
  // @see AppStore.readSettled
  function settledScoreOf(entry: AbilityScore): number {
    return store.readSettled(
      (e) =>
        e.ability_scores?.find(
          (a) => a.ability === entry.ability && (a.parameter ?? null) === (entry.parameter ?? null),
        )?.score ?? 0,
    );
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

<!-- The age → max-Ability-score ceiling, read where it bites (Slice 12, #24). Like
     the checklist above it, an auto-height sibling of `.region-row` and never a
     wrapper, and one single line so it costs the lists as little height as a
     statement can. Announced: it moves in response to an age typed on another step. -->
{#if ageCap != null}
  <p class="hint age-cap" role="status" data-testid="age-cap-note">
    {store.t('age-cap-note', { cap: String(ageCap) })}
  </p>
{/if}

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
          aria-label={store.t('filter-search-placeholder')}
          bind:value={filter.search}
          data-testid="ability-search"
        />
        <select
          bind:value={filter.category}
          aria-label={store.t('ability-category-filter-label')}
          data-testid="ability-category-filter"
        >
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
            <!-- A display-only row (#17): the character has a bonus or a granted
                 floor for this Ability but has not bought it, so there is no
                 entity row behind it. Every mutator below is index-addressed and
                 no-ops at UNBOUGHT_ROW_INDEX (no array position matches), but the
                 controls are withheld rather than left to fail silently. -->
            {@const unbought = i === UNBOUGHT_ROW_INDEX}
            {@const id = rowSuffix(i)}
            {@const key = paramKey(entry.ability)}
            {@const invalid = invalidIds.has(entry.ability)}
            <!-- The bought score the badge below is paired with (#16) — held to the
                 generation the modifiers were computed for, while `entry.score`
                 stays live for the spinner. -->
            {@const settledScore = settledScoreOf(entry)}
            <li class:invalid-selection={invalid}>
              {#if invalid}
                <!-- WCAG 1.4.1: the red tint on `.invalid-selection` is not the only
                     signal — a visible glyph plus words for a screen reader, matching
                     ValidationPanel's own non-colour severity marker. -->
                <span class="invalid-glyph" aria-hidden="true">!</span>
                <span class="sr-only">{store.t('ability-invalid-selection')}</span>
              {/if}
              <span class="item-name" use:tooltip={selectedTip(entry.ability)}
                >{selectedName(entry.ability, entry.parameter)}</span
              >
              <Spinner
                decLabel={store.t('ability-decrement', {
                  name: selectedName(entry.ability, entry.parameter),
                })}
                decTestid="ability-dec-{entry.ability}-{id}"
                decDisabled={unbought || entry.score <= 0}
                onDec={() => store.adjustAbilityAt(i, -1, max)}
                incLabel={store.t('ability-increment', {
                  name: selectedName(entry.ability, entry.parameter),
                })}
                incTestid="ability-inc-{entry.ability}-{id}"
                incDisabled={unbought || entry.score >= max}
                onInc={() => store.adjustAbilityAt(i, 1, max)}
              >
                {#snippet children()}
                  <span class="spinner-value" data-testid="ability-score-{entry.ability}-{id}">
                    {entry.score}
                  </span>
                {/snippet}
              </Spinner>
              {#if effectiveOf(settledScore, entry.ability, entry.parameter) !== settledScore}
                <span class="eff-slot">
                  <span class="eff-badge" data-testid="ability-eff-{entry.ability}-{id}">
                    {store.t('effective-score', {
                      score: String(effectiveOf(settledScore, entry.ability, entry.parameter)),
                    })}
                  </span>
                </span>
              {:else}
                <span class="eff-slot" aria-hidden="true"></span>
              {/if}
              {#if unbought}
                <!-- Why the row is inert, in words rather than by greyed controls
                     alone (WCAG 1.4.1) — and where the user buys it. -->
                <span class="ability-unbought muted">{store.t('ability-unbought-marker')}</span>
              {:else}
                <input
                  type="text"
                  class="specialty"
                  placeholder={store.t('ability-specialty-label')}
                  aria-invalid={invalid ? 'true' : undefined}
                  value={entry.specialty ?? ''}
                  oninput={(e) =>
                    store.setAbilitySpecialtyAt(i, (e.currentTarget as HTMLInputElement).value)}
                  data-testid="ability-specialty-{entry.ability}-{id}"
                />
                <button
                  type="button"
                  class="icon-btn"
                  aria-label={store.t('remove-item', {
                    name: selectedName(entry.ability, entry.parameter),
                  })}
                  onclick={() => store.removeAbilityAt(i)}
                  data-testid="remove-{entry.ability}-{id}"
                >
                  ×
                </button>
              {/if}
              {#if key && !unbought}
                <input
                  type="text"
                  class="ability-param"
                  placeholder={store.t(`param-label-${key}`)}
                  aria-invalid={invalid ? 'true' : undefined}
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
