<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName, groupByCategory } from '../derive';
  import { reserveTagSpace, tooltip, type TooltipContent } from '../actions';
  import type { ItemKind, PointItem } from '../types';

  // One picker per side: Virtues (virtue/boon) on the left, Flaws (flaw/hook)
  // next. Mirrors the virtue/flaw split used by balance().
  let { side }: { side: 'virtue' | 'flaw' } = $props();

  const kinds: ItemKind[] = $derived(side === 'virtue' ? ['virtue', 'boon'] : ['flaw', 'hook']);
  const titleKey = $derived(side === 'virtue' ? 'items-virtues-title' : 'items-flaws-title');

  const groups = $derived(store.ruleset ? groupByCategory(store.ruleset, kinds) : []);
  const selectedRefs = $derived(new Set(store.entity.selections.map((s) => s.ref)));

  // A repeatable item (one with a target parameter, or with max_per_target > 1)
  // can be added several times, so its Add button never deactivates. Mirrors the
  // predicate in store.addSelection.
  function repeatable(item: PointItem): boolean {
    return !!item.parameters?.length || (item.max_per_target ?? 1) > 1;
  }

  // Tooltip from the item's localized rules text (full description if present,
  // else the short summary). A no-op when neither exists.
  function tip(itemId: string): TooltipContent {
    const entry = store.ruleset?.i18n[itemId];
    return { text: entry?.description ?? entry?.summary ?? undefined };
  }
</script>

<section class="panel">
  <h2>{store.t(titleKey)}</h2>
  {#if store.ruleset}
    {#each groups as group (group.category)}
      <h3 class="category">{store.t(`category-${group.category}`)}</h3>
      <ul class="item-list">
        {#each group.items as item (item.id)}
          <li>
            <button
              type="button"
              class="pick-row"
              disabled={!repeatable(item) && selectedRefs.has(item.id)}
              onclick={() => store.addSelection(item.id)}
              use:tooltip={tip(item.id)}
              data-testid="add-{item.id}"
            >
              <span class="name-wrap" use:reserveTagSpace>
                <span class="badges">
                  <span class="badge">{store.t(`magnitude-${item.magnitude}`)}</span>
                </span>
                <span class="item-name">
                  {displayName(store.ruleset, item.id, undefined, (key) =>
                    store.t('param-hint', { label: store.t(`param-label-${key}`) }),
                  )}
                </span>
              </span>
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
