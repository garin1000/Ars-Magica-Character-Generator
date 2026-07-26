<script lang="ts">
  import { store } from '../state.svelte';
  import type { LongevitySource } from '../types';

  const aura = $derived(store.entity.aura ?? 0);
  const devices = $derived(store.entity.devices ?? []);
  const familiar = $derived(store.entity.familiar ?? null);
  const attunements = $derived(store.entity.talisman_attunements ?? []);
  const longevity = $derived(store.entity.longevity_ritual ?? null);
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

    <div class="detail-section">
      <h3 class="detail-label">{store.t('familiar-label')}</h3>
      {#if familiar}
        <input
          class="familiar-name"
          placeholder={store.t('familiar-name-placeholder')}
          value={familiar.name}
          oninput={(e) => store.setFamiliarName((e.currentTarget as HTMLInputElement).value)}
          data-testid="familiar-name"
        />
        <div class="cord-row">
          {#each [['gold', 'familiar-cord-gold'], ['silver', 'familiar-cord-silver'], ['bronze', 'familiar-cord-bronze']] as [cord, key] (cord)}
            <label class="field inline">
              <span>{store.t(key)}</span>
              <input
                type="number"
                min="0"
                value={familiar[`cord_${cord}` as 'cord_gold' | 'cord_silver' | 'cord_bronze'] ?? 0}
                oninput={(e) => store.setFamiliarCord(cord as 'gold' | 'silver' | 'bronze', num(e))}
                data-testid="familiar-cord-{cord}"
              />
            </label>
          {/each}
        </div>
        <button type="button" onclick={() => store.removeFamiliar()} data-testid="familiar-remove">
          {store.t('familiar-remove')}
        </button>
      {:else}
        <button type="button" onclick={() => store.addFamiliar()} data-testid="familiar-add">
          {store.t('familiar-add')}
        </button>
      {/if}
    </div>

    <div class="detail-section">
      <h3 class="detail-label">{store.t('talisman-label')}</h3>
      <ul class="talisman-list" data-testid="talisman-list">
        {#each attunements as attunement, i (i)}
          <li>
            <input
              class="talisman-desc"
              placeholder={store.t('talisman-desc-placeholder')}
              value={attunement.description}
              oninput={(e) =>
                store.setTalismanDescription(i, (e.currentTarget as HTMLInputElement).value)}
              data-testid="talisman-desc-{i}"
            />
            <label class="field inline">
              <span>{store.t('talisman-bonus-label')}</span>
              <input
                type="number"
                value={attunement.bonus}
                oninput={(e) => store.setTalismanBonus(i, num(e))}
                data-testid="talisman-bonus-{i}"
              />
            </label>
            <button
              type="button"
              class="icon-btn"
              aria-label={store.t('spell-remove')}
              onclick={() => store.removeTalismanAttunementAt(i)}
              data-testid="talisman-remove-{i}"
            >
              ×
            </button>
          </li>
        {:else}
          <li class="empty">{store.t('talisman-empty')}</li>
        {/each}
      </ul>
      <button
        type="button"
        onclick={() => store.addTalismanAttunement()}
        data-testid="talisman-add"
      >
        {store.t('talisman-add')}
      </button>
    </div>

    <div class="detail-section">
      <h3 class="detail-label">{store.t('longevity-label')}</h3>
      {#if longevity}
        <div
          class="longevity-source"
          role="radiogroup"
          aria-label={store.t('longevity-source-label')}
        >
          {#each ['self_made', 'external'] as source (source)}
            <label class="radio">
              <input
                type="radio"
                name="longevity-source"
                value={source}
                checked={longevity.source === source}
                onchange={() => store.setLongevitySource(source as LongevitySource)}
                data-testid="longevity-source-{source}"
              />
              <span>{store.t(`longevity-source-${source}`)}</span>
            </label>
          {/each}
        </div>
        {#if longevity.source === 'external'}
          <label class="field inline">
            <span>{store.t('longevity-bonus-label')}</span>
            <input
              type="number"
              value={longevity.bonus ?? 0}
              oninput={(e) => store.setLongevityBonus(num(e))}
              data-testid="longevity-bonus"
            />
          </label>
        {:else}
          <p class="empty" data-testid="longevity-self-made-note">
            {store.t('longevity-self-made-note')}
          </p>
        {/if}
        <button
          type="button"
          onclick={() => store.removeLongevityRitual()}
          data-testid="longevity-remove"
        >
          {store.t('longevity-remove')}
        </button>
      {:else}
        <button
          type="button"
          onclick={() => store.addLongevityRitual('self_made')}
          data-testid="longevity-add"
        >
          {store.t('longevity-add')}
        </button>
      {/if}
    </div>
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

  /* Section-level action buttons (Add device / Add familiar / Remove …) are
     direct children of a `.detail-section`, which is a column flex with the
     default `align-items: stretch` — so without this they stretch to the full
     panel width. Shrink them to their label. Row-level icon buttons (the × on a
     list row) live inside an `<li>`, not as direct children, so are unaffected. */
  .detail-section > button {
    align-self: flex-start;
  }

  .budget-readout {
    margin: 0;
    color: var(--muted);
    font-variant-numeric: tabular-nums;
  }

  /* Item rows: a growing name/description input, an inline number field and the
     remove button, all baseline-aligned on one line. */
  .device-list,
  .talisman-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .device-list li,
  .talisman-list li {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .device-name,
  .talisman-desc,
  .familiar-name {
    flex: 1;
    min-width: 0;
  }

  .device-list li.empty,
  .talisman-list li.empty {
    color: var(--muted);
    display: block;
  }

  /* Familiar bond cords sit in a wrapping row of inline number fields. */
  .cord-row {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  /* Longevity source radios on one row. */
  .longevity-source {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .radio {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .empty {
    color: var(--muted);
  }
</style>
