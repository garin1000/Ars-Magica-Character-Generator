<script lang="ts">
  import { store } from '../state.svelte';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round): one of
  // its ~11 largely independent stat sections. `d` is passed down rather than
  // re-derived here because the parent's own `{#if !store.ruleset || !d}`
  // gate already establishes it is loaded before any section mounts, and
  // every section needs its own different slice of the same object.
  let { d }: { d: DerivedTotals } = $props();
</script>

<div class="detail-section">
  <h3 class="detail-label">{store.t('derived-section-summary')}</h3>
  <dl class="derived-grid">
    <dt>{store.t('derived-size')}</dt>
    <dd data-testid="derived-size">{d.size}</dd>
    <dt>{store.t('derived-section-decrepitude')}</dt>
    <dd data-testid="derived-decrepitude">{d.decrepitude_score}</dd>
    <dt>{store.t('derived-section-warping')}</dt>
    <dd data-testid="derived-warping">{d.warping_score} ({d.warping_points})</dd>
  </dl>
</div>
