<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName } from '../derive';
  import { reserveTagSpace } from '../actions';
  import type { ItemKind } from '../types';
  import ParameterPicker from './ParameterPicker.svelte';

  // One list per side: chosen Virtues (virtue/boon) and Flaws (flaw/hook),
  // mirroring ItemPicker so source and selected columns line up.
  let { side }: { side: 'virtue' | 'flaw' } = $props();

  const kinds: ItemKind[] = $derived(side === 'virtue' ? ['virtue', 'boon'] : ['flaw', 'hook']);
  const titleKey = $derived(side === 'virtue' ? 'items-virtues-title' : 'items-flaws-title');

  const selections = $derived(
    store.entity.selections.filter((selection) => {
      const item = store.ruleset?.ruleset.point_items[selection.ref];
      return item ? kinds.includes(item.kind) : false;
    }),
  );
</script>

<section class="panel">
  <h2>{store.t(titleKey)}</h2>
  {#if selections.length === 0}
    <p class="muted">{store.t('empty-selections-side')}</p>
  {:else}
    <ul class="selection-list" data-testid="selection-list-{side}">
      {#each selections as selection (selection.ref)}
        {@const item = store.ruleset?.ruleset.point_items[selection.ref]}
        <li>
          <div class="selection-row">
            <span class="name-wrap" use:reserveTagSpace>
              {#if item}
                <span class="badges">
                  <span class="badge type">{store.t(`category-${item.category}`)}</span>
                  <span class="badge">{store.t(`magnitude-${item.magnitude}`)}</span>
                </span>
              {/if}
              <span class="item-name">
                {store.ruleset
                  ? displayName(store.ruleset, selection.ref, selection.params, (key) =>
                      store.t('param-hint', { label: store.t(`param-label-${key}`) }),
                    )
                  : selection.ref}
              </span>
            </span>
            <button
              type="button"
              class="icon-btn"
              onclick={() => store.removeSelection(selection.ref)}
              data-testid="remove-{selection.ref}"
            >
              −
            </button>
          </div>
          {#if item?.parameters && item.parameters.length > 0}
            <ParameterPicker {selection} params={item.parameters} />
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</section>
