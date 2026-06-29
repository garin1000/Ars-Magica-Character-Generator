<script lang="ts">
  import { store } from '../state.svelte';
  import { restrictedPoolLabel, totalXpSpent } from '../derive';

  const pool = $derived(store.entity.xp_pool ?? 0);
  // Abilities and Arts share one bank. "Spent" is the engine's authoritative,
  // Affinity-reduced total (the local fallback is only for the first frame before
  // the effective-scores call returns). "Available" reflects the general pool net
  // of what the allocation draws from it (restricted pools cover the rest).
  const spent = $derived(
    store.effective?.xp_total_demand ??
      (store.ruleset ? totalXpSpent(store.ruleset, store.entity) : 0),
  );
  const generalUsed = $derived(store.effective?.xp_general_used ?? spent);
  const available = $derived(pool - generalUsed);
  const restricted = $derived(store.effective?.restricted_xp_pools ?? []);

  function onPool(event: Event) {
    store.setXpPool(Number((event.currentTarget as HTMLInputElement).value));
  }
</script>

<div class="xp-summary">
  <label class="xp-pool">
    <span>{store.t('xp-pool')}</span>
    <input type="number" min="0" value={pool} oninput={onPool} data-testid="xp-pool" />
  </label>
  <span data-testid="xp-spent">{store.t('xp-spent', { spent: String(spent) })}</span>
  <span class:over={available < 0} data-testid="xp-available">
    {store.t('xp-available', { available: String(available) })}
  </span>
  {#each restricted as restrictedPool, i (i)}
    <span class:over={restrictedPool.used > restrictedPool.amount} data-testid="restricted-xp-{i}">
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
