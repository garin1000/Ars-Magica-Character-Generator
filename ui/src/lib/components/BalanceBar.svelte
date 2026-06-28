<script lang="ts">
  import { store } from '../state.svelte';
  import { balance } from '../derive';

  const b = $derived(store.ruleset ? balance(store.ruleset, store.entity) : null);
</script>

{#if b}
  <div class="balance" data-testid="balance">
    <span class:over={b.virtuePoints > b.virtueBudget} data-testid="balance-virtues">
      {store.t('balance-virtues', { used: b.virtuePoints, budget: b.virtueBudget })}
    </span>
    <span class:over={b.flawPoints > b.flawBudget} data-testid="balance-flaws">
      {store.t('balance-flaws', { used: b.flawPoints, budget: b.flawBudget })}
    </span>
  </div>
{/if}
