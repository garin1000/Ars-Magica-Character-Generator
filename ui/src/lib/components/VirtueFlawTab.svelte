<script lang="ts">
  import { store } from '../state.svelte';
  import {
    abilityDisplayName,
    displayName,
    filterItems,
    grantedSelectionsForSide,
    groupByCategory,
    groupSelectionsByCategory,
    incompatibleRefs,
    mandatoryTraitRefs,
  } from '../derive';
  import { reserveTagSpace, tooltip, withReason, type TooltipContent } from '../actions';
  import type { ItemKind, Magnitude, PointItem, Selection } from '../types';
  import SourcePicker from './SourcePicker.svelte';
  import SelectionList from './SelectionList.svelte';
  import ParameterPicker from './ParameterPicker.svelte';

  // The two sides: Virtues (virtue/boon) beside Flaws (flaw/hook), mirroring
  // balance(). Both the source pickers and the selected columns render one per
  // side through the same generic components.
  const SIDES = ['virtue', 'flaw'] as const;
  type Side = (typeof SIDES)[number];

  function kindsFor(side: Side): ItemKind[] {
    return side === 'virtue' ? ['virtue', 'boon'] : ['flaw', 'hook'];
  }
  function titleKey(side: Side): string {
    return side === 'virtue' ? 'items-virtues-title' : 'items-flaws-title';
  }

  // === Source (Available) side ===

  // Filter state lives on the store (per side), so it survives tab switches that
  // unmount this component. Magnitude options come from the engine-surfaced
  // taxonomy; the category options reuse the same category source the grouped
  // display uses (`groupByCategory`), never a hardcoded list.
  const magnitudes = $derived(
    store.ruleset ? Object.keys(store.ruleset.ruleset.magnitude_points) : [],
  );
  function categoriesFor(side: Side): string[] {
    return store.ruleset
      ? groupByCategory(store.ruleset, kindsFor(side)).map((g) => g.category)
      : [];
  }

  function sourceGroups(side: Side) {
    const rs = store.ruleset;
    if (!rs) return [];
    const filter = store.filters.vf[side];
    const itemFilter = {
      text: filter.search,
      categories: filter.category ? [filter.category] : undefined,
      magnitudes: filter.magnitude ? [filter.magnitude as Magnitude] : undefined,
      tainted: filter.taintedOnly || undefined,
    };
    return groupByCategory(rs, kindsFor(side))
      .map((g) => ({
        key: g.category,
        header: store.t(`category-${g.category}`),
        items: filterItems(rs, g.items, itemFilter, store.t),
      }))
      .filter((g) => g.items.length > 0);
  }
  const selectedRefs = $derived(new Set((store.entity.selections ?? []).map((s) => s.ref)));

  // A repeatable item (one with a target parameter, or with max_per_target > 1)
  // can be added several times, so its Add button never deactivates. Mirrors the
  // predicate in store.addSelection.
  function repeatable(item: PointItem): boolean {
    return !!item.parameters?.length || (item.max_per_target ?? 1) > 1;
  }

  // Items an already-selected V/F excludes (Major vs Minor of the same V/F,
  // Gentle vs Blatant Gift, the Dwarf/Small Frame/Large clique). Non-empty only
  // in enforced mode, where the Add button greys out; advisory/silent leave the
  // pick open and let the engine's `incompatible` issue report it.
  const blocked = $derived(
    store.ruleset
      ? incompatibleRefs(store.ruleset, store.entity.selections ?? [], store.mode)
      : new Map<string, string>(),
  );

  // Tooltip from the item's localized rules text (full description if present,
  // else the short summary). A no-op when neither exists.
  function tip(itemId: string): TooltipContent {
    const entry = store.ruleset?.i18n[itemId];
    return { text: entry?.description ?? entry?.summary ?? undefined };
  }

  // A blocked source row explains WHY above its normal description, naming the
  // selected item that excludes it. Undefined for a takeable row.
  function sourceTip(itemId: string): TooltipContent {
    const blocker = blocked.get(itemId);
    return withReason(
      tip(itemId),
      blocker && store.ruleset
        ? store.t('vf-blocked-incompatible', {
            other: displayName(store.ruleset, blocker, undefined, hint),
          })
        : undefined,
    );
  }

  // === Selected side ===

  // Traits the character type mandates (a magus's The Gift + Hermetic Magus):
  // auto-selected, shown with a "Required" marker and no remove button.
  const mandatory = $derived(
    mandatoryTraitRefs(store.ruleset?.ruleset.type_profiles[store.entity.type_id]),
  );

  // A chosen selection with its original entity index, or a read-only
  // House-granted row — the two row shapes a V/F column renders.
  type VfRow =
    | { kind: 'sel'; selection: Selection; index: number }
    | { kind: 'granted'; grant: Selection };

  function selectionsFor(side: Side): { selection: Selection; index: number }[] {
    const kinds = kindsFor(side);
    return (store.entity.selections ?? [])
      .map((selection, index) => ({ selection, index }))
      .filter(({ selection }) => {
        const item = store.ruleset?.ruleset.point_items[selection.ref];
        return item ? kinds.includes(item.kind) : false;
      });
  }

  const selectedColumns = $derived.by(() =>
    SIDES.map((side) => {
      const selections = selectionsFor(side);
      const groupedSelections = store.ruleset
        ? groupSelectionsByCategory(store.ruleset, selections)
        : [];
      // House-granted rows on this side (e.g. Bonisagus → Puissant Magic Theory):
      // derived at eval, not stored, so they show read-only below the chosen ones.
      const granted = store.ruleset
        ? grantedSelectionsForSide(store.ruleset, store.effective?.granted_selections, side)
        : [];
      const groups: {
        key: string;
        header?: string;
        rows: { key: string | number; item: VfRow }[];
      }[] = groupedSelections.map((g) => ({
        key: g.category,
        header: store.t(`category-${g.category}`),
        rows: g.entries.map((e) => ({
          key: e.index,
          item: { kind: 'sel', selection: e.selection, index: e.index } as VfRow,
        })),
      }));
      if (granted.length > 0) {
        // The granted rows sit in a header-less list below the chosen ones.
        groups.push({
          key: 'granted',
          rows: granted.map((grant, i) => ({
            // The index is part of the key because the ref is not unique: the
            // engine concatenates House, Mythic Companion, `grants_selection` and
            // warping grants without dedup (effective.rs `entity_grants`), so one
            // ref can be granted twice. A duplicate key makes Svelte throw
            // `each_key_duplicate`, which aborts this tab's render entirely.
            key: `granted-${i}-${grant.ref}`,
            item: { kind: 'granted', grant } as VfRow,
          })),
        });
      }
      return {
        key: side,
        title: store.t(titleKey(side)),
        panelTestid: `selection-list-${side}`,
        empty: selections.length === 0 && granted.length === 0,
        emptyText: store.t('empty-selections-side'),
        emptyClass: 'muted',
        groups,
      };
    }),
  );

  function hint(key: string): string {
    return store.t('param-hint', { label: store.t(`param-label-${key}`) });
  }

  // Resolve a filled param value (a ref slug) to its display label so the tag
  // reads "Great Perception" / "Puissant Brandenburg Lore", not the raw slug.
  // `params` is the whole selection's params so an ability target can pull in its
  // sibling instance value (the area/language).
  function resolveParamValue(params: Record<string, string> | undefined, value: string): string {
    if (!store.ruleset) return value;
    if (value.startsWith('characteristic.')) {
      return store.t(`characteristic-${value.slice('characteristic.'.length)}`);
    }
    const ability = store.ruleset.ruleset.abilities?.[value];
    if (ability) {
      // A parameterized ability target ((Area) Lore) fills its instance value.
      const instanceKey = ability.parameter ?? undefined;
      const instanceValue = instanceKey ? params?.[instanceKey] : undefined;
      return abilityDisplayName(store.ruleset, value, instanceValue, hint);
    }
    if (store.ruleset.i18n[value]) {
      return displayName(store.ruleset, value, undefined, hint);
    }
    return value;
  }
</script>

<!-- Only the item's two intrinsic tags (category + magnitude) stack in the
     right-edge overlay; a provenance marker (Required/Granted) is rendered as a
     separate inline chip in the row (see below), so the vertical stack never
     grows past two and bleeds into neighbouring rows. -->
{#snippet nameWrap(ref: string, params: Record<string, string> | undefined)}
  {@const item = store.ruleset?.ruleset.point_items[ref]}
  <span class="name-wrap" use:reserveTagSpace use:tooltip={tip(ref)}>
    {#if item}
      <span class="badges">
        <span class="badge type">{store.t(`category-${item.category}`)}</span>
        <span class="badge">{store.t(`magnitude-${item.magnitude}`)}</span>
      </span>
    {/if}
    <span class="item-name">
      {store.ruleset
        ? displayName(store.ruleset, ref, params, hint, (_key, value) =>
            resolveParamValue(params, value),
          )
        : ref}
    </span>
  </span>
{/snippet}

<div class="region-row">
  <section class="region region-source">
    <h2 class="region-title">{store.t('available-title')}</h2>
    <div class="region-columns">
      {#each SIDES as side (side)}
        <SourcePicker
          title={store.t(titleKey(side))}
          groups={sourceGroups(side)}
          getId={(it: PointItem) => it.id}
          onAdd={(it: PointItem) => store.addSelection(it.id)}
          disabled={(it: PointItem) =>
            (!repeatable(it) && selectedRefs.has(it.id)) || blocked.has(it.id)}
          tip={(it: PointItem) => sourceTip(it.id)}
        >
          {#snippet filters()}
            <input
              type="search"
              class="filter-search"
              placeholder={store.t('filter-search-placeholder')}
              bind:value={store.filters.vf[side].search}
              data-testid="vf-search-{side}"
            />
            <select
              bind:value={store.filters.vf[side].category}
              data-testid="vf-category-filter-{side}"
            >
              <option value="">{store.t('filter-category-all')}</option>
              {#each categoriesFor(side) as c (c)}
                <option value={c}>{store.t(`category-${c}`)}</option>
              {/each}
            </select>
            <select
              bind:value={store.filters.vf[side].magnitude}
              data-testid="vf-magnitude-filter-{side}"
            >
              <option value="">{store.t('filter-magnitude-all')}</option>
              {#each magnitudes as m (m)}
                <option value={m}>{store.t(`magnitude-${m}`)}</option>
              {/each}
            </select>
            <label class="filter-check">
              <input
                type="checkbox"
                bind:checked={store.filters.vf[side].taintedOnly}
                data-testid="vf-tainted-filter-{side}"
              />
              {store.t('vf-tag-tainted')}
            </label>
          {/snippet}
          {#snippet row(it: PointItem)}
            <span class="name-wrap" use:reserveTagSpace>
              <span class="badges">
                <span class="badge">{store.t(`magnitude-${it.magnitude}`)}</span>
              </span>
              <span class="item-name">
                {displayName(store.ruleset!, it.id, undefined, (key) =>
                  store.t('param-hint', { label: store.t(`param-label-${key}`) }),
                )}
              </span>
            </span>
          {/snippet}
        </SourcePicker>
      {/each}
    </div>
  </section>
  <section class="region region-selected">
    <h2 class="region-title">{store.t('selections-title')}</h2>
    <div class="selected-frame">
      <!-- The frame carries the border and its padding; this inner box does the
           scrolling, so the padding stays a gap the rows cannot scroll into. -->
      <div class="selected-scroll">
        <SelectionList columns={selectedColumns}>
          {#snippet row(item: VfRow)}
            {#if item.kind === 'sel'}
              {@const selection = item.selection}
              {@const index = item.index}
              {@const pointItem = store.ruleset?.ruleset.point_items[selection.ref]}
              {@const required = mandatory.has(selection.ref)}
              <li>
                <div class="selection-row">
                  {@render nameWrap(selection.ref, selection.params)}
                  {#if required}
                    <span class="row-marker">{store.t('selection-required-label')}</span>
                  {:else}
                    <button
                      type="button"
                      class="icon-btn"
                      onclick={() => store.removeSelectionAt(index)}
                      data-testid="remove-{selection.ref}-{index}"
                    >
                      -
                    </button>
                  {/if}
                </div>
                {#if pointItem?.parameters && pointItem.parameters.length > 0}
                  <ParameterPicker {selection} {index} params={pointItem.parameters} />
                {/if}
              </li>
            {:else}
              <li data-testid="granted-selection-{item.grant.ref}">
                <div class="selection-row">
                  {@render nameWrap(item.grant.ref, item.grant.params)}
                  <span class="row-marker">{store.t('house-granted-label')}</span>
                </div>
              </li>
            {/if}
          {/snippet}
        </SelectionList>
      </div>
    </div>
  </section>
</div>
