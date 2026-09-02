<script lang="ts">
  // The signed-bonus chip `XpBar` and `SpellBudgetBar` each reimplement
  // (V32, full-audit round): a positive Virtue/Flaw modifier is an extra pool
  // spent first, a negative one has no pool of its own and is charged to the
  // base instead — so the same "pool" vs. "charge" text is repeated with only
  // the bonus figure, the testid and the two Fluent keys differing.
  //
  // This is deliberately NOT a full unification of the two bars: XpBar's used
  // figure is charged against its own `base`, while SpellBudgetBar's is charged
  // against `base + lifeStage` (`alloc.denominator`) with the base itself shown
  // as a separate entry below — a real difference the bar's own comments call
  // out by name (guided-creation-review-2026-08 #18), not an accident of
  // copy-paste. Only the bonus chip is a byte-for-byte duplicate, so only the
  // bonus chip is factored out.
  let {
    bonus,
    testid,
    positiveText,
    negativeText,
  }: {
    bonus: number;
    testid: string;
    positiveText: string;
    negativeText: string;
  } = $props();
</script>

{#if bonus > 0}
  <span class="xp-restricted" data-testid={testid}>{positiveText}</span>
{:else if bonus < 0}
  <span class="xp-restricted over" data-testid={testid}>{negativeText}</span>
{/if}
