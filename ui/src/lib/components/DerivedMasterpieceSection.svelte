<script lang="ts">
  import { store } from '../state.svelte';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.is_magus}` block).
  // Self-gated on `d.masterpiece`, matching the original inline `{#if}` exactly.

  let { d }: { d: DerivedTotals } = $props();

  // Localized display name for a catalogue id (an Art here), id as fallback.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }
</script>

<!-- Masterpiece (lesser enchanted item cap) -->
{#if d.masterpiece}
  <div class="detail-section">
    <h3 class="detail-label">{store.t('derived-section-masterpiece')}</h3>
    <p data-testid="derived-masterpiece">
      {store.t('derived-masterpiece-cap')}: {d.masterpiece.cap}
      ({store.t('derived-lab-total')}
      {d.masterpiece.lab_total} ·
      {name(d.masterpiece.technique)} / {name(d.masterpiece.form)})
    </p>
    <p class="hint">{store.t('derived-masterpiece-note')}</p>
  </div>
{/if}

<style>
  /* `.detail-section` is a shared global in app.css; `.hint` follows the
     project's per-component convention (see `TalismanPanel`, `FamiliarPanel`,
     `MagicPossessions`). */
  .hint {
    opacity: 0.7;
    font-size: 0.85em;
    font-style: italic;
  }
</style>
