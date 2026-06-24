<script lang="ts">
  import { store } from '../state.svelte';
  import { characteristicPointsUsed } from '../derive';
  import { CHARACTERISTICS, type Characteristic } from '../types';

  const rules = $derived(store.ruleset?.ruleset.characteristic_rules ?? null);
  const min = $derived(rules ? Math.min(...rules.costs.map((c) => c.score)) : -3);
  const max = $derived(rules ? Math.max(...rules.costs.map((c) => c.score)) : 3);
  const used = $derived(characteristicPointsUsed(rules, store.entity.characteristics));

  function scoreOf(characteristic: Characteristic): number {
    return store.entity.characteristics?.[characteristic] ?? 0;
  }

  function bonusOf(characteristic: Characteristic): number {
    return store.effective?.characteristic_bonuses?.[characteristic] ?? 0;
  }

  function descriptionOf(characteristic: Characteristic): string {
    return store.entity.characteristic_descriptions?.[characteristic] ?? '';
  }

  function adjust(characteristic: Characteristic, delta: number) {
    const next = Math.max(min, Math.min(max, scoreOf(characteristic) + delta));
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
        <span class="spinner-label">{store.t(`characteristic-${characteristic}`)}</span>
        <span class="spinner">
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('characteristic-decrement')}
            disabled={scoreOf(characteristic) <= min}
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
            disabled={scoreOf(characteristic) >= max}
            onclick={() => adjust(characteristic, 1)}
            data-testid="char-inc-{characteristic}"
          >
            +
          </button>
          {#if bonusOf(characteristic) !== 0}
            <span class="eff-badge" data-testid="char-eff-{characteristic}">
              {store.t('effective-score', {
                score: fmt(scoreOf(characteristic) + bonusOf(characteristic)),
              })}
            </span>
          {/if}
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
