<script lang="ts">
  import { store } from '../state.svelte';
  import { restrictedPoolLabel, totalXpSpent } from '../derive';

  // Arts and Abilities share one XP bank (`entity.xp_pool`); this bar edits the
  // same pool and shows the combined spend, so the remaining total is consistent
  // across the Abilities and Arts tabs. "Spent" is the engine's authoritative,
  // Affinity-reduced total (the local fallback covers the first frame only).
  const pool = $derived(store.entity.xp_pool ?? 0);
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
    <input type="number" min="0" value={pool} oninput={onPool} data-testid="art-xp-pool" />
  </label>
  <span data-testid="art-xp-spent">{store.t('xp-spent', { spent: String(spent) })}</span>
  <span class:over={available < 0} data-testid="art-xp-available">
    {store.t('xp-available', { available: String(available) })}
  </span>
  {#each restricted as restrictedPool, i (i)}
    <span
      class:over={restrictedPool.used > restrictedPool.amount}
      data-testid="art-restricted-xp-{i}"
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
