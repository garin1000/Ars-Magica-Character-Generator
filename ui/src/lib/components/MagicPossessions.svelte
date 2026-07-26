<script lang="ts">
  import { store } from '../state.svelte';
  import FamiliarPanel from './FamiliarPanel.svelte';
  import TalismanPanel from './TalismanPanel.svelte';
  import LongevityPanel from './LongevityPanel.svelte';

  const aura = $derived(store.entity.aura ?? 0);
  const devices = $derived(store.entity.devices ?? []);
  // Item-level budget used/remaining is engine-authoritative, never recomputed here.
  const itemBudget = $derived(store.effective?.item_level_budget ?? 0);
  const itemUsed = $derived(store.effective?.item_level_used ?? 0);

  function onAura(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAura(raw === '' ? null : Number(raw));
  }

  function num(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value);
  }
</script>

<section class="panel magic-possessions">
  {#if store.ruleset}
    <div class="detail-field">
      <label class="field">
        <span>{store.t('aura-label')}</span>
        <input type="number" value={aura} oninput={onAura} data-testid="aura-input" />
      </label>
    </div>

    <div class="detail-section">
      <h3 class="detail-label">{store.t('possessions-devices-label')}</h3>
      <p class="budget-readout" data-testid="item-level-used">
        {store.t('item-level-used', { used: String(itemUsed), budget: String(itemBudget) })}
      </p>
      <ul class="device-list" data-testid="device-list">
        {#each devices as device, i (i)}
          <li>
            <input
              class="device-name"
              placeholder={store.t('device-name-placeholder')}
              value={device.name}
              oninput={(e) => store.setDeviceName(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="device-name-{i}"
            />
            <label class="field inline">
              <span>{store.t('device-level-label')}</span>
              <input
                type="number"
                min="0"
                value={device.level}
                oninput={(e) => store.setDeviceLevel(i, num(e))}
                data-testid="device-level-{i}"
              />
            </label>
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('spell-remove')}
              onclick={() => store.removeDeviceAt(i)}
              data-testid="device-remove-{i}"
            >
              ×
            </button>
          </li>
        {:else}
          <li class="empty">{store.t('devices-empty')}</li>
        {/each}
      </ul>
      <button type="button" onclick={() => store.addDevice()} data-testid="device-add">
        {store.t('device-add')}
      </button>
    </div>

    <FamiliarPanel />

    <TalismanPanel />

    <LongevityPanel />
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>

<style>
  /* Top-level vertical rhythm between the aura field and each item section.
     `.detail-section` (spacing within a section) and `.field.inline` (label
     beside its input) are shared globals in app.css. */
  .magic-possessions {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    /* Keep the input area a readable column on a wide screen rather than spanning
       the whole viewport; centered within the scrolling tab. */
    max-width: 40rem;
    margin-inline: auto;
  }

  /* Section-level action buttons (Add device) are direct children of a
     `.detail-section`, which is a column flex with the default
     `align-items: stretch` — so without this they stretch to the full panel
     width. Shrink them to their label. Row-level icon buttons (the × on a
     list row) live inside an `<li>`, not as direct children, so are unaffected.
     The extracted panels each carry their own copy of this rule, because Svelte
     styles are component-scoped. */
  .detail-section > button {
    align-self: flex-start;
  }

  .budget-readout {
    margin: 0;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  /* Device rows: a growing name input, an inline number field and the remove
     button, all baseline-aligned on one line. */
  .device-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .device-list li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .device-name {
    flex: 1;
    min-width: 0;
  }

  .device-list li.empty {
    color: var(--muted);
    display: block;
  }

  .empty {
    color: var(--muted);
  }
</style>
