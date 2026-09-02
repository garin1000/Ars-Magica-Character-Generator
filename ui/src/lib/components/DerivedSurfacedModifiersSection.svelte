<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned } from '../derive';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Always
  // rendered (not magus-only). Self-gated on `d.surfaced_modifiers.length > 0`,
  // matching the original inline `{#if}` exactly.

  let { d }: { d: DerivedTotals } = $props();

  // Surfaced-modifier detail: enum scalars go through Fluent; free-text
  // (ability-roll subject) is shown as entered.
  function detailLabel(family: string, detail: string): string {
    if (family === 'ability_roll') return detail;
    return store.t(`derived-detail-${detail}`);
  }
</script>

{#if d.surfaced_modifiers.length > 0}
  <div class="detail-section">
    <h3 class="detail-label">{store.t('derived-section-surfaced')}</h3>
    <ul class="derived-list" data-testid="derived-surfaced">
      {#each d.surfaced_modifiers as m, i (m.family + m.detail + i)}
        <li>
          <span>{store.t(`derived-surfaced-${m.family}`)}: {detailLabel(m.family, m.detail)}</span>
          {#if m.amount !== 0}<span class="value">{formatSigned(m.amount)}</span>{/if}
        </li>
      {/each}
    </ul>
  </div>
{/if}
