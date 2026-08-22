<script lang="ts">
  import { store } from '../state.svelte';

  // Fully engine-authoritative: both halves of the bar read `EffectiveScores`
  // (`virtue_budget`/`flaw_budget` from `effective_point_ceilings`,
  // `virtue_points`/`flaw_points` from `compute_balance`) rather than
  // re-deriving either in TypeScript. Audit finding G1 (round 4): the spent
  // side used to be a second, independent implementation of a rule the engine
  // already owns — the same defect class already fixed once for
  // `characteristic_points_used` (VA1/GF1/GD4) — with no engine cross-check to
  // catch it drifting.
  const virtuePoints = $derived(store.effective?.virtue_points ?? 0);
  const flawPoints = $derived(store.effective?.flaw_points ?? 0);
  const virtueBudget = $derived(store.effective?.virtue_budget ?? 0);
  const flawBudget = $derived(store.effective?.flaw_budget ?? 0);
</script>

<!-- Gated on the ruleset rather than `store.effective`: the bar mounts as soon
     as the Virtues & Flaws surface does, showing 0/0 for the brief window
     before the first `effective_scores` round trip resolves rather than
     staying hidden — the same brief-flash trade-off `characteristic_points_used`
     already accepted (audit VA1/GF1/GD4) once its TS fallback was removed. -->
{#if store.ruleset}
  <!-- Announced when either side crosses over budget, matching XpBar's
       role="status" treatment of its own over-budget state. -->
  <div class="balance" role="status" data-testid="balance">
    <span class:over={virtuePoints > virtueBudget} data-testid="balance-virtues">
      {store.t('balance-virtues', { used: virtuePoints, budget: virtueBudget })}
    </span>
    <span class:over={flawPoints > flawBudget} data-testid="balance-flaws">
      {store.t('balance-flaws', { used: flawPoints, budget: flawBudget })}
    </span>
  </div>
{/if}
