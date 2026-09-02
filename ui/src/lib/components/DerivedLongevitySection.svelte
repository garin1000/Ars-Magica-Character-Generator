<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned } from '../derive';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.is_magus}` block).
  // Self-gated on `d.longevity`, matching the original inline `{#if}` exactly.

  let { d }: { d: DerivedTotals } = $props();
</script>

{#if d.longevity}
  <div class="detail-section">
    <h3 class="detail-label">{store.t('derived-section-longevity')}</h3>
    <p data-testid="derived-longevity">
      {store.t(`derived-longevity-${d.longevity.source}`)}:
      {#if d.longevity.entered}
        <!-- The stored bonus is a magnitude; this panel shows what it DOES, so
             it is negated into the aging-roll modifier and signed by
             formatSigned — a bonus of 0 reads "0", never "-0", and a stored
             negative reads "+n" rather than "--n". The Fluent string names the
             quantity, so it cannot be confused with the editor's "+7". -->
        {store.t('derived-longevity-aging-modifier', {
          modifier: formatSigned(-d.longevity.bonus),
        })}
      {:else}
        {store.t('derived-longevity-not-entered')}
      {/if}
      {#if d.longevity.hint}
        · {store.t('derived-longevity-suggested', {
          modifier: formatSigned(-d.longevity.hint.suggested_bonus),
        })}
        ({store.t('derived-lab-total')}
        {d.longevity.hint.lab_total}{#if d.longevity.hint.halved}, {store.t(
            'longevity-hint-halved',
          )}{/if})
      {/if}
      {#if d.longevity.bronze_cord > 0}
        · {store.t('derived-addend-bronze_cord')} {formatSigned(d.longevity.bronze_cord)}
      {/if}
    </p>
  </div>
{/if}
