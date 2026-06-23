<script lang="ts">
  import { store } from '../state.svelte';
  import { characteristicPointsUsed } from '../derive';
  import { CHARACTERISTICS, type Characteristic } from '../types';

  // The point-buy cost table (and thus the legal score range) is data on the
  // ruleset; fall back to an empty list until it loads.
  const rules = $derived(store.ruleset?.ruleset.characteristic_rules ?? null);
  const scoreOptions = $derived(
    rules ? [...rules.costs].map((c) => c.score).sort((a, b) => b - a) : [],
  );
  const used = $derived(characteristicPointsUsed(rules, store.entity.characteristics));

  function scoreOf(characteristic: Characteristic): number {
    return store.entity.characteristics?.[characteristic] ?? 0;
  }

  function onChange(characteristic: Characteristic, event: Event) {
    const value = Number((event.currentTarget as HTMLSelectElement).value);
    store.setCharacteristic(characteristic, value);
  }

  function fmt(score: number): string {
    return score > 0 ? `+${score}` : `${score}`;
  }
</script>

<section class="panel">
  <h2>{store.t('characteristics-title')}</h2>
  {#if rules}
    <p class="points" data-testid="characteristic-points">
      {store.t('characteristic-points', { used: String(used), budget: String(rules.start_points) })}
    </p>
    <ul class="char-list">
      {#each CHARACTERISTICS as characteristic (characteristic)}
        <li>
          <label for="char-{characteristic}">{store.t(`characteristic-${characteristic}`)}</label>
          <select
            id="char-{characteristic}"
            value={scoreOf(characteristic)}
            onchange={(e) => onChange(characteristic, e)}
            data-testid="char-{characteristic}"
          >
            {#each scoreOptions as score (score)}
              <option value={score}>{fmt(score)}</option>
            {/each}
          </select>
        </li>
      {/each}
    </ul>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
