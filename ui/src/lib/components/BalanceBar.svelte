<script lang="ts">
  import { store } from '../state.svelte';
  import { balance } from '../derive';

  const b = $derived(store.ruleset ? balance(store.ruleset, store.entity) : null);
  // The budget ceilings are engine-authoritative: a Mythic Companion type's
  // bonus points (Devil Child +7 F / +3 V) raise them above the profile base,
  // so prefer the engine's effective ceilings when present, else the base.
  const virtueBudget = $derived(store.effective?.virtue_budget ?? b?.virtueBudget ?? 0);
  const flawBudget = $derived(store.effective?.flaw_budget ?? b?.flawBudget ?? 0);
</script>

{#if b}
  <!-- Announced when either side crosses over budget, matching XpBar's
       role="status" treatment of its own over-budget state. -->
  <div class="balance" role="status" data-testid="balance">
    <span class:over={b.virtuePoints > virtueBudget} data-testid="balance-virtues">
      {store.t('balance-virtues', { used: b.virtuePoints, budget: virtueBudget })}
    </span>
    <span class:over={b.flawPoints > flawBudget} data-testid="balance-flaws">
      {store.t('balance-flaws', { used: b.flawPoints, budget: flawBudget })}
    </span>
  </div>
{/if}
