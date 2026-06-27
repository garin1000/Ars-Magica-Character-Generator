<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityDisplayName, displayName } from '../derive';
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

<section class="panel">
  <h2>{store.t(titleKey)}</h2>
  {#if selections.length === 0}
    <p class="muted">{store.t('empty-selections-side')}</p>
  {:else}
    <ul class="selection-list" data-testid="selection-list-{side}">
      {#each selections as { selection, index } (index)}
        {@const item = store.ruleset?.ruleset.point_items[selection.ref]}
        <li>
          <div class="selection-row">
            <span class="name-wrap" use:reserveTagSpace use:tooltip={tip(selection.ref)}>
              {#if item}
                <span class="badges">
                  <span class="badge type">{store.t(`category-${item.category}`)}</span>
                  <span class="badge">{store.t(`magnitude-${item.magnitude}`)}</span>
                </span>
              {/if}
              <span class="item-name">
                {store.ruleset
                  ? displayName(
                      store.ruleset,
                      selection.ref,
                      selection.params,
                      hint,
                      (_key, value) => resolveParamValue(selection.params, value),
                    )
                  : selection.ref}
              </span>
            </span>
            <button
              type="button"
              class="icon-btn"
              onclick={() => store.removeSelectionAt(index)}
              data-testid="remove-{selection.ref}-{index}"
            >
              −
            </button>
          </div>
          {#if item?.parameters && item.parameters.length > 0}
            <ParameterPicker {selection} {index} params={item.parameters} />
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>
