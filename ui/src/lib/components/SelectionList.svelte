<script lang="ts">
  import { store } from '../state.svelte';
  import {
    abilityDisplayName,
    displayName,
    grantedSelectionsForSide,
    groupSelectionsByCategory,
    mandatoryTraitRefs,
  } from '../derive';
  import { reserveTagSpace, tooltip, type TooltipContent } from '../actions';
  import type { ItemKind } from '../types';
  import ParameterPicker from './ParameterPicker.svelte';

  // One list per side: chosen Virtues (virtue/boon) and Flaws (flaw/hook),
  // mirroring ItemPicker so source and selected columns line up.
  let { side }: { side: 'virtue' | 'flaw' } = $props();

  const kinds: ItemKind[] = $derived(side === 'virtue' ? ['virtue', 'boon'] : ['flaw', 'hook']);
  const titleKey = $derived(side === 'virtue' ? 'items-virtues-title' : 'items-flaws-title');

  // Carry the original entity index so repeated items address the right row even
  // after the per-side kind filter drops the others.
  const selections = $derived(
    store.entity.selections
      .map((selection, index) => ({ selection, index }))
      .filter(({ selection }) => {
        const item = store.ruleset?.ruleset.point_items[selection.ref];
        return item ? kinds.includes(item.kind) : false;
      }),
  );

  // Chosen selections grouped by category and alpha-sorted within each group
  // (mirroring the source picker). Original entity indices ride along for wiring.
  const groupedSelections = $derived(
    store.ruleset ? groupSelectionsByCategory(store.ruleset, selections) : [],
  );

  // Traits the character type mandates (a magus's The Gift + Hermetic Magus):
  // auto-selected, shown with a "Required" marker and no remove button.
  const mandatory = $derived(
    mandatoryTraitRefs(store.ruleset?.ruleset.type_profiles[store.entity.type_id]),
  );

  // House-granted rows on this side (e.g. Bonisagus → Puissant Magic Theory):
  // derived at eval, not stored, so they show read-only below the chosen ones.
  const granted = $derived(
    store.ruleset
      ? grantedSelectionsForSide(store.ruleset, store.effective?.granted_selections, side)
      : [],
  );

  function hint(key: string): string {
    return store.t('param-hint', { label: store.t(`param-label-${key}`) });
  }

  // Tooltip from the item's localized rules text (description, else summary).
  function tip(itemId: string): TooltipContent {
    const entry = store.ruleset?.i18n[itemId];
    return { text: entry?.description ?? entry?.summary ?? undefined };
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

<section class="panel" data-testid="selection-list-{side}">
  <h2>{store.t(titleKey)}</h2>
  {#if selections.length === 0 && granted.length === 0}
    <p class="muted">{store.t('empty-selections-side')}</p>
  {:else}
    {#each groupedSelections as group (group.category)}
      <h3 class="category">{store.t(`category-${group.category}`)}</h3>
      <ul class="selection-list">
        {#each group.entries as { selection, index } (index)}
          {@const item = store.ruleset?.ruleset.point_items[selection.ref]}
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
                  −
                </button>
              {/if}
            </div>
            {#if item?.parameters && item.parameters.length > 0}
              <ParameterPicker {selection} {index} params={item.parameters} />
            {/if}
          </li>
        {/each}
      </ul>
    {/each}
    {#if granted.length > 0}
      <ul class="selection-list">
        {#each granted as grant (grant.ref)}
          <li data-testid="granted-selection-{grant.ref}">
            <div class="selection-row">
              {@render nameWrap(grant.ref, grant.params)}
              <span class="row-marker">{store.t('house-granted-label')}</span>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</section>
