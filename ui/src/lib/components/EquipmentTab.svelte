<script lang="ts">
  import { store } from '../state.svelte';
  import {
    filterEquipment,
    groupSelectedEquipmentByKind,
    type EquipmentKind,
    type SelectedEquipmentGroup,
  } from '../derive';
  import type { EquipmentSlot } from '../types';
  import SourcePicker from './SourcePicker.svelte';
  import SelectionList from './SelectionList.svelte';

  function nameOf(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  // === Source (Available) side ===

  // Filter state (free-text + kind) lives on the store so it survives the tab
  // switches that unmount this component, matching the other pickers.
  const filter = $derived(store.filters.equipment);

  // The catalogue as click-to-add rows, grouped by kind (weapons/shields/armor)
  // and name-sorted.
  const sourceGroups = $derived.by(() => {
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
    ).map((group) => ({
      key: group.kind,
      header: store.t(`equipment-group-${group.kind}`),
      items: group.ids,
    }));
  });

  // === Selected side ===

  // The chosen equipment slots (references to catalogue ids), grouped by kind so
  // the selected side carries the same headers as the available list (mirroring the
  // Abilities and V/F tabs). Each row keeps its original entity index for the
  // index-addressed mutators — the display order is never written back.
  const equipment = $derived(store.entity.equipment ?? []);

  const selectedGroups = $derived.by((): SelectedEquipmentGroup[] => {
    const rs = store.ruleset;
    if (!rs) return [];
    return groupSelectedEquipmentByKind(
      rs,
      {
        weapons: rs.ruleset.weapons ?? {},
        shields: rs.ruleset.shields ?? {},
        armor: rs.ruleset.armor ?? {},
      },
      equipment,
    );
  });

  const selectedColumns = $derived([
    {
      key: 'equipment',
      empty: equipment.length === 0,
      emptyText: store.t('equipment-empty'),
      emptyClass: 'empty',
      groups: selectedGroups.map((g, gi) => ({
        key: g.kind ?? 'unclassified',
        // An item absent from all three catalogues has no kind to label, so its
        // group renders header-less rather than showing a raw slug.
        header: g.kind ? store.t(`equipment-group-${g.kind}`) : undefined,
        listClass: 'equipment-list',
        // The `equipment-list` testid stays unique by living on the first group
        // only, now that the list is split per kind.
        ulTestid: gi === 0 ? 'equipment-list' : undefined,
        rows: g.entries.map((e) => ({ key: e.index, item: e })),
      })),
    },
  ]);

  // The specialization toggle is meaningful only for an equipped weapon whose
  // combat Ability carries a non-empty specialty on this character — otherwise it
  // is a dead toggle (Core:7122, :7139), so we render it only when applicable.
  function specializationApplicable(slot: EquipmentSlot): boolean {
    if (!slot.equipped) return false;
    const weapon = store.ruleset?.ruleset?.weapons?.[slot.item];
    if (!weapon) return false;
    return (store.entity.ability_scores ?? []).some(
      (a) => a.ability === weapon.ability && (a.specialty ?? '').trim() !== '',
    );
  }
</script>

<div class="region-row">
  <section class="region region-source">
    <h2 class="region-title">{store.t('available-title')}</h2>
    <SourcePicker
      groups={sourceGroups}
      getId={(id: string) => id}
      onAdd={(id: string) => store.addEquipment(id)}
    >
      {#snippet filters()}
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
      {/snippet}
      {#snippet row(id: string)}
        <span class="item-name">{nameOf(id)}</span>
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
          {#snippet row(item: { slot: EquipmentSlot; index: number })}
            {@const slot = item.slot}
            {@const i = item.index}
            <li>
              <span class="equipment-name" data-testid="equipment-name-{i}"
                >{nameOf(slot.item)}</span
              >
              <label class="checkbox inline">
                <input
                  type="checkbox"
                  checked={slot.equipped ?? false}
                  onchange={(e) =>
                    store.setEquipmentEquipped(i, (e.currentTarget as HTMLInputElement).checked)}
                  data-testid="equipment-equipped-{i}"
                />
                <span>{store.t('equipment-equipped-label')}</span>
              </label>
              {#if specializationApplicable(slot)}
                <label class="checkbox inline">
                  <input
                    type="checkbox"
                    checked={slot.specialization_applies ?? false}
                    onchange={(e) =>
                      store.setEquipmentSpecialization(
                        i,
                        (e.currentTarget as HTMLInputElement).checked,
                      )}
                    data-testid="equipment-specialization-{i}"
                  />
                  <span>{store.t('equipment-specialization-label')}</span>
                </label>
              {/if}
              <button
                type="button"
                class="icon-btn"
                aria-label={store.t('equipment-remove')}
                onclick={() => store.removeEquipmentAt(i)}
                data-testid="equipment-remove-{i}"
              >
                ×
              </button>
            </li>
          {/snippet}
        </SelectionList>
      </div>
    </div>
  </section>
</div>
