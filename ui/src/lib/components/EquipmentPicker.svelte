<script lang="ts">
  import { store } from '../state.svelte';
  import type { Armor, Shield, Weapon } from '../types';

  // The chosen equipment slots (references to catalogue ids).
  const equipment = $derived(store.entity.equipment ?? []);

  // The catalogue, split into the three groups, each sorted by localized name.
  function nameOf(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }

  const weapons = $derived<Weapon[]>(
    Object.values(store.ruleset?.ruleset.weapons ?? {}).sort((a, b) =>
      nameOf(a.id).localeCompare(nameOf(b.id)),
    ),
  );
  const shields = $derived<Shield[]>(
    Object.values(store.ruleset?.ruleset.shields ?? {}).sort((a, b) =>
      nameOf(a.id).localeCompare(nameOf(b.id)),
    ),
  );
  const armor = $derived<Armor[]>(
    Object.values(store.ruleset?.ruleset.armor ?? {}).sort((a, b) =>
      nameOf(a.id).localeCompare(nameOf(b.id)),
    ),
  );

  // The currently-picked catalogue id in the add-control.
  let picked = $state('');

  function add() {
    if (!picked) return;
    store.addEquipment(picked);
    picked = '';
  }
</script>

<section class="panel equipment-picker">
  {#if store.ruleset}
    <div class="detail-section">
      <h3 class="detail-label">{store.t('equipment-add-label')}</h3>
      <div class="equipment-add-row">
        <label class="field grow">
          <span class="sr-only">{store.t('equipment-add-label')}</span>
          <select bind:value={picked} data-testid="equipment-add-select">
            <option value="">{store.t('equipment-none')}</option>
            <optgroup label={store.t('equipment-group-weapons')}>
              {#each weapons as w (w.id)}
                <option value={w.id}>{nameOf(w.id)}</option>
              {/each}
            </optgroup>
            <optgroup label={store.t('equipment-group-shields')}>
              {#each shields as s (s.id)}
                <option value={s.id}>{nameOf(s.id)}</option>
              {/each}
            </optgroup>
            <optgroup label={store.t('equipment-group-armor')}>
              {#each armor as a (a.id)}
                <option value={a.id}>{nameOf(a.id)}</option>
              {/each}
            </optgroup>
          </select>
        </label>
        <button type="button" onclick={add} disabled={!picked} data-testid="equipment-add">
          {store.t('equipment-add')}
        </button>
      </div>
    </div>

    <div class="detail-section">
      <h3 class="detail-label">{store.t('equipment-carried-label')}</h3>
      <ul class="equipment-list" data-testid="equipment-list">
        {#each equipment as slot, i (i)}
          <li>
            <span class="equipment-name" data-testid="equipment-name-{i}">{nameOf(slot.item)}</span>
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
        {:else}
          <li class="empty">{store.t('equipment-empty')}</li>
        {/each}
      </ul>
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
