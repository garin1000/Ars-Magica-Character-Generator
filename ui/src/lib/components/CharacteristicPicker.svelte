<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned } from '../derive';
  import { tooltip } from '../actions';
  import { CHARACTERISTICS, type Characteristic } from '../types';
  import Spinner from './Spinner.svelte';

  const rules = $derived(store.ruleset?.ruleset.characteristic_rules ?? null);
  // The cost table spans the absolute ±5 range; the *buyable* range per
  // characteristic is the base cap/floor, widened by Great/Poor Characteristic.
  const tableMax = $derived(rules ? Math.max(...rules.costs.map((c) => c.score)) : 3);
  const tableMin = $derived(rules ? Math.min(...rules.costs.map((c) => c.score)) : -3);
  // Engine-authoritative: the point-buy cost comes from the effective-scores
  // payload rather than a second copy of the table here (audit finding VA1). Zero
  // until that call returns, matching how `budget` treats its own grant below.
  const used = $derived(store.effective?.characteristic_points_used ?? 0);
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

  // The bought score the engine's effective values were computed against — NOT the
  // live one the spinner shows (#16). The badge compares the two halves of a pair,
  // so both must come from the same generation: against the live score, raising a
  // characteristic flashed a phantom "-> +0" badge for the length of the debounce,
  // because the engine had not yet been asked about the new value.
  // @see AppStore.readSettled
  function settledScoreOf(characteristic: Characteristic): number {
    return store.readSettled((e) => e.characteristics?.[characteristic] ?? 0);
  }

  // Effective score after aging drops AND free virtue deltas. Falls back to the
  // settled bought score when the engine reports no change — the engine owns the
  // floor clamp, so the UI never re-implements it.
  function effectiveOf(characteristic: Characteristic): number {
    return (
      store.effective?.characteristic_effective?.[characteristic] ?? settledScoreOf(characteristic)
    );
  }

  // Aging-drop count for the tooltip breakdown (0 when none).
  function agingDropOf(characteristic: Characteristic): number {
    return store.effective?.characteristic_aging_drops?.[characteristic] ?? 0;
  }

  // Free virtue/flaw effective-score delta (Giant Blood +1 Str/Sta, Dwarf -1)
  // for the tooltip breakdown. 0 when no virtue/flaw affects this characteristic.
  function bonusOf(characteristic: Characteristic): number {
    return (
      store.effective?.characteristic_bonuses?.find((b) => b.characteristic === characteristic)
        ?.bonus ?? 0
    );
  }

  // Tooltip breakdown for the effective score: a bought → effective summary plus
  // the reasons that apply (aging drop and/or free virtue delta).
  function effectiveTooltip(characteristic: Characteristic) {
    const drops = agingDropOf(characteristic);
    const bonus = bonusOf(characteristic);
    const list: string[] = [];
    if (drops !== 0) {
      list.push(store.t('characteristic-effective-tooltip-aging', { drops: String(drops) }));
    }
    if (bonus !== 0) {
      list.push(store.t('characteristic-effective-tooltip-virtue', { bonus: formatSigned(bonus) }));
    }
    return {
      text: store.t('characteristic-effective-tooltip-summary', {
        // The settled bought score, so the "bought -> effective" summary reads as
        // one coherent pair rather than two generations (#16).
        bought: formatSigned(settledScoreOf(characteristic)),
        effective: formatSigned(effectiveOf(characteristic)),
      }),
      listLabel: store.t('characteristic-effective-tooltip-breakdown-label'),
      list,
    };
  }

  // Derived Size (base 0), shown only when a virtue/flaw moves it off 0.
  const size = $derived(store.effective?.size ?? 0);
</script>

<!-- `.char-panel` centres ITSELF (app.css), so this one component sits identically
     in the editor's centring `.tab-panel` and in the wizard's stretching `.vf-tab`
     (guided-creation-review-2026-08 #27). -->
<section class="panel char-panel" data-testid="char-panel">
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
        <Spinner
          decLabel={store.t('characteristic-decrement', {
            name: store.t(`characteristic-${characteristic}`),
          })}
          decTestid="char-dec-{characteristic}"
          decDisabled={scoreOf(characteristic) <= floorOf(characteristic)}
          onDec={() => adjust(characteristic, -1)}
          incLabel={store.t('characteristic-increment', {
            name: store.t(`characteristic-${characteristic}`),
          })}
          incTestid="char-inc-{characteristic}"
          incDisabled={scoreOf(characteristic) >= capOf(characteristic)}
          onInc={() => adjust(characteristic, 1)}
        >
          {#snippet children()}
            <span class="spinner-value" data-testid="char-value-{characteristic}">
              {formatSigned(scoreOf(characteristic))}
              {#if effectiveOf(characteristic) !== settledScoreOf(characteristic)}<span
                  class="eff-badge char-effective"
                  data-testid="char-effective-{characteristic}"
                  use:tooltip={effectiveTooltip(characteristic)}
                  >→ {formatSigned(effectiveOf(characteristic))}</span
                >{/if}
            </span>
          {/snippet}
        </Spinner>
        <input
          type="text"
          class="char-description"
          placeholder={store.t('characteristic-description-label')}
          aria-label={store.t('characteristic-description-for', {
            name: store.t(`characteristic-${characteristic}`),
          })}
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
        {store.t('characteristic-size', { size: formatSigned(size) })}
      </p>
    {/if}
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>
