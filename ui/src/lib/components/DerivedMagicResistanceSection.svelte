<script lang="ts">
  import { store } from '../state.svelte';
  import { addendBreakdown } from '../derive';
  import { tooltip } from '../actions';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.is_magus}` block).

  let { d }: { d: DerivedTotals } = $props();

  // Localized display name for a catalogue id (a Form here), id as fallback.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }
</script>

<!-- Magic Resistance (per Form) -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('derived-section-magic-resistance')}</h3>
  <ul class="derived-list" data-testid="derived-magic-resistance">
    {#each d.magic_resistance as mr (mr.form)}
      <li>
        <span>{name(mr.form)}</span>
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <span
          class="value"
          tabindex="0"
          use:tooltip={{ text: addendBreakdown(mr.addends, store.t) }}>{mr.total}</span
        >
      </li>
    {/each}
  </ul>
</div>
