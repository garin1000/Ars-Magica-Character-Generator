<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName, groupByCategory } from '../derive';
  import type { ItemKind } from '../types';

  // One picker per side: Virtues (virtue/boon) on the left, Flaws (flaw/hook)
  // next. Mirrors the virtue/flaw split used by balance().
  let { side }: { side: 'virtue' | 'flaw' } = $props();

  const kinds: ItemKind[] = $derived(side === 'virtue' ? ['virtue', 'boon'] : ['flaw', 'hook']);
  const titleKey = $derived(side === 'virtue' ? 'items-virtues-title' : 'items-flaws-title');

  const groups = $derived(store.ruleset ? groupByCategory(store.ruleset, kinds) : []);
  const selectedRefs = $derived(new Set(store.entity.selections.map((s) => s.ref)));
</script>

<section class="panel">
  <h2>{store.t(titleKey)}</h2>
  {#if store.ruleset}
    {#each groups as group (group.category)}
      <h3 class="category">{store.t(`category-${group.category}`)}</h3>
      <ul class="item-list">
        {#each group.items as item (item.id)}
          <li>
            <span class="item-name">{displayName(store.ruleset, item.id)}</span>
            <span class="badge">{store.t(`magnitude-${item.magnitude}`)}</span>
            <button
              type="button"
              disabled={selectedRefs.has(item.id)}
              onclick={() => store.addSelection(item.id)}
              data-testid="add-{item.id}"
            >
              {store.t('action-add')}
            </button>
          </li>
        {/each}
      </ul>
    {/each}
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
