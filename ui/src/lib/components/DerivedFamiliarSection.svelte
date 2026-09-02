<script lang="ts">
  import { store } from '../state.svelte';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.is_magus}` block).
  // Self-gated on `d.familiar`, matching the original inline `{#if}` exactly.

  let { d }: { d: DerivedTotals } = $props();

  // Localized display name for a catalogue id (Technique/Form here), id as
  // fallback.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }
</script>

<!-- Familiar bond. Guidance only, like Masterpiece: the level the bond needs,
     the magus's best bonding Lab Total, what the cords cost, and the total
     level invested. The within-focus figure is conditional — whether this
     beast falls inside the focus is a troupe judgment (Core:10818) — and the
     invested levels deliberately get no budget bar (:10866, no limit). -->
{#if d.familiar}
  <div class="detail-section">
    <h3 class="detail-label">{store.t('derived-section-familiar')}</h3>
    <p data-testid="derived-familiar-binding">
      {store.t('derived-familiar-binding-level')}: {d.familiar.binding_level}
      ({store.t('derived-lab-total')}
      {d.familiar.binding.lab_total} ·
      {name(d.familiar.binding.technique)} / {name(
        d.familiar.binding.form,
      )}{#if d.familiar.binding.lab_total_within_focus != null}, {store.t('derived-within-focus')}
        {d.familiar.binding.lab_total_within_focus}{/if})
    </p>
    <p class="hint" data-testid="derived-familiar-reaches">
      {d.familiar.binding.lab_total_reaches_level
        ? store.t('derived-familiar-reaches')
        : store.t('derived-familiar-falls-short')}
    </p>
    <p data-testid="derived-familiar-cords">
      {store.t('derived-familiar-cord-points')}: {d.familiar.cord_points_spent}
    </p>
    <p class="hint" data-testid="derived-familiar-cords-fit">
      {d.familiar.binding.cord_points_within_lab_total
        ? store.t('derived-familiar-cords-fit')
        : store.t('derived-familiar-cords-exceed')}
    </p>
    <p data-testid="derived-familiar-invested">
      {store.t('derived-familiar-invested-levels')}: {d.familiar.invested_power_levels}
    </p>
    <p class="hint">{store.t('derived-familiar-note')}</p>
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
