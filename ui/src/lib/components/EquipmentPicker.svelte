<script lang="ts">
  import { store } from '../state.svelte';
  import { filterEquipment, type EquipmentKind } from '../derive';

  function nameOf(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  // Filter state (free-text + kind) lives on the store so it survives the tab
  // switches that unmount this component, matching the other pickers.
  const filter = $derived(store.filters.equipment);

  // The catalogue as click-to-add rows, grouped by kind (weapons/shields/armor)
  // and name-sorted — the source side, mirroring the Abilities picker.
  const groups = $derived.by(() => {
    const rs = store.ruleset;
    if (!rs) return [];
    return filterEquipment(
      rs,
      {
        weapons: rs.ruleset.weapons ?? {},
        shields: rs.ruleset.shields ?? {},
        armor: rs.ruleset.armor ?? {},
      },
      { text: filter.search, kind: filter.kind ? (filter.kind as EquipmentKind) : undefined },
      store.t,
    );
  });
</script>

<section class="panel">
  {#if store.ruleset}
    <div class="filter-bar">
      <input
        type="search"
        class="filter-search"
        placeholder={store.t('filter-search-placeholder')}
        bind:value={filter.search}
        data-testid="equipment-search"
      />
      <select bind:value={filter.kind} data-testid="equipment-group-filter">
        <option value="">{store.t('filter-category-all')}</option>
        <option value="weapons">{store.t('equipment-group-weapons')}</option>
        <option value="shields">{store.t('equipment-group-shields')}</option>
        <option value="armor">{store.t('equipment-group-armor')}</option>
      </select>
    </div>
    <div class="list-scroll">
      {#each groups as group (group.kind)}
        <h3 class="category">{store.t(`equipment-group-${group.kind}`)}</h3>
        <ul class="item-list">
          {#each group.ids as id (id)}
            <li>
              <button
                type="button"
                class="pick-row"
                onclick={() => store.addEquipment(id)}
                data-testid="add-{id}"
              >
                <span class="item-name">{nameOf(id)}</span>
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
