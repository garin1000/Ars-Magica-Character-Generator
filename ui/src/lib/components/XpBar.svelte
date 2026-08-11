<script lang="ts">
  import { store } from '../state.svelte';
  import { restrictedPoolLabel, totalXpSpent } from '../derive';

  // One XP summary shared by the Abilities and Arts tabs: both spend from the
  // SAME `entity.xp_pool`, so this single component drives both, differing only
  // in the data-testid prefix ('' for Abilities, 'art-' for Arts) the e2e suite
  // keys off. Pool source, used source and restricted sub-budgets are identical
  // across the two instances.
  let { prefix = '' }: { prefix?: string } = $props();

  // A life-stage plan IS the guided-funding switch (there is no stored flag), and
  // the engine makes a plan and a typed pool mutually exclusive
  // (`life_stage_xp_pool_conflict`). So under a plan the editable total must go:
  // offering it would offer a value the engine rejects. Both instances read the
  // same `entity.xp_pool`, so the Arts bar takes the identical guided shape —
  // correct, since Arts spend that very pool.
  const guided = $derived(store.entity.life_stages != null);
  // The engine's life-stage budget, or null when the plan yields none yet (no age
  // typed, or a ruleset without life-stage rules).
  const lifeStage = $derived(store.effective?.life_stage ?? null);
  // What the player typed, which under a plan should be nothing at all.
  const typedPool = $derived(store.entity.xp_pool ?? 0);
  // The general pool: later life's experience under a plan, the typed total
  // otherwise — the engine's own `base_general`.
  const pool = $derived(guided ? (lifeStage?.later_life_xp ?? 0) : typedPool);
  const clearHintId = $derived(`${prefix}xp-pool-clear-hint`);
  // The engine's authoritative slice of the spend FUNDED from the general pool.
  // `restricted_xp_pools` cover the rest and are reported separately. The local
  // `totalXpSpent` fallback only covers the first frame before the effective-
  // scores call returns (no restricted grant is assumed then).
  const generalFunded = $derived(
    store.effective?.xp_general_used ??
      (store.ruleset ? totalXpSpent(store.ruleset, store.entity) : 0),
  );
  // Demand the pools cannot fund at all — the engine's overspend, the same figure
  // the `not_enough_xp` issue reports as its shortfall.
  //
  // This is load-bearing: `xp_general_used` is a max-flow value whose source edge
  // is capped by the pool, so it can NEVER exceed the pool and `pool -
  // xp_general_used` can never be negative, however far the spend overshoots. An
  // overspend only shows up as unfunded demand, so it must be added back in to get
  // the true general-pool charge.
  const overspend = $derived(
    Math.max((store.effective?.xp_total_demand ?? 0) - (store.effective?.xp_max_flow ?? 0), 0),
  );
  // What the general pool is actually being asked for. Unfundable demand is by
  // definition general demand (the general pool may fund any spend, so it is
  // exhausted before anything goes unfunded).
  const generalUsed = $derived(generalFunded + overspend);
  // Not clamped: overspending shows a negative value in bold red (the `over`
  // class covers both the label and the number).
  const available = $derived(pool - generalUsed);
  const restricted = $derived(store.effective?.restricted_xp_pools ?? []);

  function onPool(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setXpPool(raw === '' ? 0 : Number(raw));
  }
</script>

<div class="xp-summary">
  <span class="xp-pool">
    <span class="xp-pool-label">{store.t('xp-pool')}</span>
    <span class="xp-pool-used" class:over-value={available < 0} data-testid="{prefix}xp-spent"
      >{generalUsed}</span
    >
    <span class="xp-pool-total">
      {#if guided}
        <!-- Read-only text, not a disabled input: assistive tech must not announce
             a control the player cannot use. -->
        <span data-testid="{prefix}xp-pool-total">{pool}</span>
      {:else}
        <input
          type="number"
          min="0"
          max="4294967295"
          placeholder="0"
          value={pool || ''}
          oninput={onPool}
          data-testid="{prefix}xp-pool"
        />
      {/if}
    </span>
  </span>
  <span class="xp-available" class:over={available < 0} data-testid="{prefix}xp-available">
    {store.t('xp-available', { available: String(available) })}
  </span>
  {#if guided}
    {#if lifeStage}
      <span class="xp-life-stage" data-testid="{prefix}life-stage-later-life">
        {store.t('life-stage-later-life', {
          years: String(lifeStage.later_life_years),
          rate: String(lifeStage.later_life_rate),
          xp: String(lifeStage.later_life_xp),
        })}
      </span>
    {:else}
      <!-- Announced: this row arrives in response to an edit elsewhere (the age),
           so its appearance must reach a screen reader. -->
      <span class="xp-life-stage" role="status" data-testid="{prefix}life-stage-no-budget">
        {store.t('life-stage-no-budget')}
      </span>
    {/if}
  {/if}
  {#each restricted as restrictedPool, i (i)}
    <span
      class="xp-restricted"
      class:over={restrictedPool.used > restrictedPool.amount}
      data-testid="{prefix}restricted-xp-{i}"
    >
      {store.ruleset
        ? store.t('restricted-xp-pool', {
            eligibility: restrictedPoolLabel(store.ruleset, restrictedPool, store.t),
            used: String(restrictedPool.used),
            amount: String(restrictedPool.amount),
          })
        : ''}
    </span>
  {/each}
  {#if guided && typedPool > 0}
    <!-- Escape hatch for a hand-edited save that carries both a plan and a typed
         pool: guided mode shows no field to correct one, so without this the
         engine's conflict error would be inescapable from the UI. -->
    <button
      type="button"
      class="xp-pool-clear"
      aria-describedby={clearHintId}
      onclick={() => store.setXpPool(0)}
      data-testid="{prefix}xp-pool-clear"
    >
      {store.t('xp-pool-clear')}
    </button>
    <span class="xp-pool-clear-hint" id={clearHintId} data-testid="{prefix}xp-pool-clear-hint">
      {store.t('xp-pool-clear-hint')}
    </span>
  {/if}
</div>
