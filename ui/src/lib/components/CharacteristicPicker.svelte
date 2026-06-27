<script lang="ts">
  import { store } from '../state.svelte';
  import { characteristicPointsUsed } from '../derive';
  import { tooltip } from '../actions';
  import { CHARACTERISTICS, type Characteristic } from '../types';

  const rules = $derived(store.ruleset?.ruleset.characteristic_rules ?? null);
  // The cost table spans the absolute ±5 range; the *buyable* range per
  // characteristic is the base cap/floor, widened by Great/Poor Characteristic.
  const tableMax = $derived(rules ? Math.max(...rules.costs.map((c) => c.score)) : 3);
  const tableMin = $derived(rules ? Math.min(...rules.costs.map((c) => c.score)) : -3);
  const used = $derived(characteristicPointsUsed(rules, store.entity.characteristics));

  function scoreOf(characteristic: Characteristic): number {
    return store.entity.characteristics?.[characteristic] ?? 0;
  }

  // Per-characteristic buy limits from the engine (entity-dependent: Great raises
  // the cap, Poor lowers the floor). Until they arrive, fall back to the
  // ruleset's base cap/floor, then to the table bounds.
  function capOf(characteristic: Characteristic): number {
    return store.effective?.characteristic_caps?.[characteristic] ?? rules?.base_max ?? tableMax;
  }

  function floorOf(characteristic: Characteristic): number {
    return store.effective?.characteristic_floors?.[characteristic] ?? rules?.base_min ?? tableMin;
  }

  function descriptionOf(characteristic: Characteristic): string {
    return store.entity.characteristic_descriptions?.[characteristic] ?? '';
  }

  function adjust(characteristic: Characteristic, delta: number) {
    const next = Math.max(
      floorOf(characteristic),
      Math.min(capOf(characteristic), scoreOf(characteristic) + delta),
    );
    store.setCharacteristic(characteristic, next);
  }

  function fmt(score: number): string {
    return score > 0 ? `+${score}` : `${score}`;
  }
</script>

<section class="panel char-panel">
  {#if rules}
    <p class="points" data-testid="characteristic-points">
      {store.t('characteristic-points', { used: String(used), budget: String(rules.start_points) })}
    </p>
    <div class="char-grid">
      {#each CHARACTERISTICS as characteristic (characteristic)}
        <span
          class="spinner-label"
          use:tooltip={{ text: store.t(`characteristic-desc-${characteristic}`) }}
          >{store.t(`characteristic-${characteristic}`)}</span
        >
        <span class="spinner">
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('characteristic-decrement')}
            disabled={scoreOf(characteristic) <= floorOf(characteristic)}
            onclick={() => adjust(characteristic, -1)}
            data-testid="char-dec-{characteristic}"
          >
            −
          </button>
          <span class="spinner-value" data-testid="char-value-{characteristic}">
            {fmt(scoreOf(characteristic))}
          </span>
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('characteristic-increment')}
            disabled={scoreOf(characteristic) >= capOf(characteristic)}
            onclick={() => adjust(characteristic, 1)}
            data-testid="char-inc-{characteristic}"
          >
            +
          </button>
        </span>
        <input
          type="text"
          class="char-description"
          placeholder={store.t('characteristic-description-label')}
          value={descriptionOf(characteristic)}
          oninput={(e) =>
            store.setCharacteristicDescription(
              characteristic,
              (e.currentTarget as HTMLInputElement).value,
            )}
          data-testid="char-desc-{characteristic}"
        />
      {/each}
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
