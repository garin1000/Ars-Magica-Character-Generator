<script lang="ts">
  // Read-only play-stat read-out (M5/5i). Renders the numbers the `derived_totals`
  // command computes; it performs NO mechanics in JS. Every label goes through a
  // Fluent `derived-*` key; catalogue ids resolve to their localized display name.
  //
  // Split into one component per stat section (V26/G16, full-audit round):
  // this file used to hold all ~11 sections inline in 578 lines. `d` is
  // computed once here and passed down, rather than each child re-deriving
  // `store.derived` itself, because every child needs a different slice of
  // the SAME object and this file's own `{#if !store.ruleset || !d}` gate
  // already establishes it is loaded before any child mounts.
  import { store } from '../state.svelte';
  import DerivedSummarySection from './DerivedSummarySection.svelte';
  import DerivedAuraField from './DerivedAuraField.svelte';
  import DerivedLabCastingSection from './DerivedLabCastingSection.svelte';
  import DerivedPenetrationSection from './DerivedPenetrationSection.svelte';
  import DerivedMagicResistanceSection from './DerivedMagicResistanceSection.svelte';
  import DerivedLongevitySection from './DerivedLongevitySection.svelte';
  import DerivedMasterpieceSection from './DerivedMasterpieceSection.svelte';
  import DerivedFamiliarSection from './DerivedFamiliarSection.svelte';
  import DerivedCombatSection from './DerivedCombatSection.svelte';
  import DerivedVitalsSection from './DerivedVitalsSection.svelte';
  import DerivedSurfacedModifiersSection from './DerivedSurfacedModifiersSection.svelte';

  const d = $derived(store.derived);
</script>

<section class="panel derived-panel" data-testid="derived-panel">
  {#if !store.ruleset || !d}
    <p>{store.t('loading')}</p>
  {:else}
    <!-- S17 (full-audit a11y): the tab's own <h2> — the ~11 subsections below
         stayed <h3> directly under the app's single <h1> with no <h2> between
         (a heading hierarchy gap) until this was added. Reuses `tab-totals`,
         the same label App.svelte's tab button already carries. Every
         subsection still owns its own <h3>, unchanged by the split below. -->
    <h2>{store.t('tab-totals')}</h2>

    <DerivedSummarySection {d} />

    {#if d.is_magus}
      <DerivedAuraField />
      <DerivedLabCastingSection {d} />
      <DerivedPenetrationSection {d} />
      <DerivedMagicResistanceSection {d} />
      <DerivedLongevitySection {d} />
      <DerivedMasterpieceSection {d} />
      <DerivedFamiliarSection {d} />
    {/if}

    <DerivedCombatSection {d} />
    <DerivedVitalsSection {d} />
    <DerivedSurfacedModifiersSection {d} />
  {/if}
</section>
