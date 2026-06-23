<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName } from '../derive';
  import ParameterPicker from './ParameterPicker.svelte';
</script>

<section class="panel">
  <h2>{store.t('selections-title')}</h2>
  {#if store.entity.selections.length === 0}
    <p class="muted">{store.t('empty-selections')}</p>
  {:else}
    <ul class="selection-list" data-testid="selection-list">
      {#each store.entity.selections as selection (selection.ref)}
        {@const item = store.ruleset?.ruleset.point_items[selection.ref]}
        <li>
          <div class="selection-row">
            <span class="item-name">
              {store.ruleset
                ? displayName(store.ruleset, selection.ref, selection.params)
                : selection.ref}
            </span>
            {#if item}
              <span class="badge type">{store.t(`category-${item.category}`)}</span>
              <span class="badge">{store.t(`magnitude-${item.magnitude}`)}</span>
            {/if}
            <button
              type="button"
              onclick={() => store.removeSelection(selection.ref)}
              data-testid="remove-{selection.ref}"
            >
              {store.t('action-remove')}
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
