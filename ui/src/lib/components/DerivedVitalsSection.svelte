<script lang="ts">
  import { store } from '../state.svelte';
  import { addendBreakdown } from '../derive';
  import { tooltip } from '../actions';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Always
  // rendered (not magus-only). Soak/Encumbrance, Fatigue and Wounds are kept
  // TOGETHER here rather than as three more files: each is a handful of lines
  // of pure read-only display with no logic of its own, so three separate
  // components would be pure file-count churn for no clarity gained — the
  // "split that obscures" the governing rule warns against, just inverted
  // (too many tiny files rather than one long one).

  let { d }: { d: DerivedTotals } = $props();
</script>

<!-- Soak & Encumbrance -->
<div class="detail-section">
  <dl class="derived-grid">
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <dt tabindex="0" use:tooltip={{ text: addendBreakdown(d.soak.addends, store.t) }}>
      {store.t('derived-section-soak')}
    </dt>
    <dd data-testid="derived-soak">{d.soak.total}</dd>
    <dt>{store.t('derived-section-encumbrance')}</dt>
    <dd data-testid="derived-encumbrance">
      {d.encumbrance.total} ({store.t('derived-load')}
      {d.encumbrance.load}, {store.t('derived-burden')}
      {d.encumbrance.burden})
    </dd>
  </dl>
</div>

<!-- Fatigue -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('derived-section-fatigue')}</h3>
  <ul class="derived-inline" data-testid="derived-fatigue">
    {#each d.fatigue as f (f.level)}
      <li>{store.t(`derived-fatigue-${f.level}`)}: {f.penalty}</li>
    {/each}
  </ul>
</div>

<!-- Wound ranges -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('derived-section-wounds')}</h3>
  <ul class="derived-list wound-list" data-testid="derived-wounds">
    {#each d.wounds as w (w.level)}
      <li>
        <span class="wound-level">{store.t(`derived-wound-${w.level}`)}</span>
        <span class="value">{w.min}{w.max != null ? `–${w.max}` : '+'}</span>
        <span class="derived-focus">{w.penalty != null ? w.penalty : ''}</span>
      </li>
    {/each}
  </ul>
</div>
