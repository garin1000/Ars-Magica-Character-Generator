<script lang="ts">
  import { store } from '../state.svelte';
  import { abilityXpSpent } from '../derive';

  const advancement = $derived(store.ruleset?.ruleset.advancement ?? []);
  const pool = $derived(store.entity.xp_pool ?? 0);
  const spent = $derived(abilityXpSpent(advancement, store.entity.ability_scores));
  const available = $derived(pool - spent);

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
</div>
