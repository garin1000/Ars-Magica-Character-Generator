<script lang="ts">
  import { store } from '../state.svelte';
  import { restrictedPoolLabel, totalXpSpent } from '../derive';

  // One XP summary shared by the Abilities and Arts tabs: both spend from the
  // SAME `entity.xp_pool`, so this single component drives both, differing only
  // in the data-testid prefix ('' for Abilities, 'art-' for Arts) the e2e suite
  // keys off. Pool source, used source and restricted sub-budgets are identical
  // across the two instances.
  let { prefix = '' }: { prefix?: string } = $props();

  const pool = $derived(store.entity.xp_pool ?? 0);
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
      <input
        type="number"
        min="0"
        max="4294967295"
        placeholder="0"
        value={pool || ''}
        oninput={onPool}
        data-testid="{prefix}xp-pool"
      />
    </span>
  </span>
  <span class="xp-available" class:over={available < 0} data-testid="{prefix}xp-available">
    {store.t('xp-available', { available: String(available) })}
  </span>
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
</div>
