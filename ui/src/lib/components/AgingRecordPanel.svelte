<script lang="ts">
  import { store } from '../state.svelte';
  import { CHARACTERISTICS, type Characteristic } from '../types';

  const apparentAge = $derived(store.entity.apparent_age ?? null);
  // Decrepitude is derived by the engine from the sum of aging points; hidden at 0.
  const decrepitude = $derived(store.effective?.decrepitude_score ?? 0);
  const decrepitudeEffect = $derived(store.entity.decrepitude_effect ?? '');
  const agingPoints = $derived(store.entity.aging_points ?? {});
  const agingLog = $derived(store.entity.aging_log ?? []);

  function onApparentAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setApparentAge(raw === '' ? null : Number(raw));
  }

  function charLabel(characteristic: Characteristic): string {
    return store.t(`characteristic-${characteristic}`);
  }

  function numValue(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value || 0);
  }

  /** An emptied field reads as "not given", not as 0 — same as the age field. */
  function optionalNumValue(event: Event): number | null {
    const raw = (event.currentTarget as HTMLInputElement).value;
    return raw === '' ? null : Number(raw);
  }
</script>

<!-- What aging has already done to this character: the apparent age, the derived
     Decrepitude, the accrued aging points and the per-year log. Extracted from
     CharacterDetails so the editor and the guided aging step mount the one
     surface and can never drift apart.
     `display: contents` inside `.character-details` (see app.css) keeps each block
     below its own item in that multi-column flow — a wrapper box would drag the
     whole cluster into a single unbreakable column. -->
<div class="aging-record" data-testid="aging-record">
  <div class="detail-field">
    <label class="field">
      <span>{store.t('apparent-age-label')}</span>
      <input
        type="number"
        min="1"
        max="4294967295"
        value={apparentAge ?? ''}
        oninput={onApparentAge}
        data-testid="apparent-age-input"
      />
    </label>
  </div>

  {#if decrepitude > 0}
    <div class="detail-field">
      <span class="detail-label">{store.t('decrepitude-label')}</span>
      <span data-testid="decrepitude-readout">
        {store.t('decrepitude-readout', { score: String(decrepitude) })}
      </span>
    </div>
  {/if}

  <div class="detail-field">
    <label class="field">
      <span>{store.t('decrepitude-effect-label')}</span>
      <input
        type="text"
        value={decrepitudeEffect}
        oninput={(e) => store.setDecrepitudeEffect((e.currentTarget as HTMLInputElement).value)}
        data-testid="decrepitude-effect-input"
      />
    </label>
  </div>

  <div class="detail-section">
    <h3 class="detail-label">{store.t('aging-label')}</h3>
    <p class="detail-label">{store.t('aging-points-heading')}</p>
    <ul class="aging-list" data-testid="aging-points-list">
      {#each CHARACTERISTICS as characteristic (characteristic)}
        <li>
          <span class="char-name">{charLabel(characteristic)}</span>
          <input
            type="number"
            min="0"
            max="255"
            value={agingPoints[characteristic] ?? 0}
            oninput={(e) => store.setAgingPoints(characteristic, numValue(e))}
            data-testid="aging-points-{characteristic}"
          />
        </li>
      {/each}
    </ul>
    <p class="detail-label" data-testid="aging-points-note">
      {store.t('aging-points-note')}
    </p>

    <p class="detail-label">{store.t('aging-log-heading')}</p>
    <ul class="twilight-list" data-testid="aging-log-list">
      {#each agingLog as entry, i (i)}
        <li>
          <input
            type="number"
            class="aging-log-year"
            min="-2147483648"
            max="2147483647"
            aria-label={store.t('aging-log-year-label')}
            value={entry.year ?? ''}
            oninput={(e) => store.setAgingLogEntryYear(i, optionalNumValue(e))}
            data-testid="aging-log-year-{i}"
          />
          <input
            class="twilight-desc"
            placeholder={store.t('aging-log-effect-placeholder')}
            value={entry.effect}
            oninput={(e) =>
              store.setAgingLogEntryEffect(i, (e.currentTarget as HTMLInputElement).value)}
            data-testid="aging-log-effect-{i}"
          />
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('spell-remove')}
            onclick={() => store.removeAgingLogEntryAt(i)}
            data-testid="aging-log-remove-{i}"
          >
            ×
          </button>
        </li>
      {:else}
        <li class="empty" data-testid="aging-log-empty">{store.t('aging-log-empty')}</li>
      {/each}
    </ul>
    <button type="button" onclick={() => store.addAgingLogEntry()} data-testid="aging-log-add">
      {store.t('aging-log-add')}
    </button>
  </div>
</div>
