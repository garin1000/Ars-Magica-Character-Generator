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
  // Improved Characteristics raises the buy budget above the ruleset base; the
  // engine reports the grant (0 until the effective-scores call returns).
  const budget = $derived(
    (rules?.start_points ?? 0) + (store.effective?.characteristic_points_granted ?? 0),
  );

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

  // Free effective-score bonus (Giant Blood +1 Str/Sta, Dwarf -1), shown next to
  // the bought score. 0 when no virtue/flaw affects this characteristic.
  function bonusOf(characteristic: Characteristic): number {
    return (
      store.effective?.characteristic_bonuses?.find((b) => b.characteristic === characteristic)
        ?.bonus ?? 0
    );
  }

  // Derived Size (base 0), shown only when a virtue/flaw moves it off 0.
  const size = $derived(store.effective?.size ?? 0);
</script>

<section class="panel char-panel">
  {#if rules}
    <p class="points" data-testid="characteristic-points">
      {store.t('characteristic-points', { used: String(used), budget: String(budget) })}
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
            {#if bonusOf(characteristic) !== 0}<span
                class="char-bonus"
                data-testid="char-bonus-{characteristic}">({fmt(bonusOf(characteristic))})</span
              >{/if}
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
    {#if size !== 0}
      <p class="size-readout" data-testid="characteristic-size">
        {store.t('characteristic-size', { size: fmt(size) })}
      </p>
    {/if}
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
