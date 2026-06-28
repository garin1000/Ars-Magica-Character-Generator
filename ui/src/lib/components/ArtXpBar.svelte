<script lang="ts">
  import { store } from '../state.svelte';
  import { totalXpSpent } from '../derive';

  // Arts and Abilities share one XP bank (`entity.xp_pool`); this bar edits the
  // same pool and shows the combined spend, so the remaining total is consistent
  // across the Abilities and Arts tabs.
  const pool = $derived(store.entity.xp_pool ?? 0);
  const spent = $derived(store.ruleset ? totalXpSpent(store.ruleset, store.entity) : 0);
  const available = $derived(pool - spent);

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
</div>
