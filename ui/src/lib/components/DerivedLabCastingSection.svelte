<script lang="ts">
  import { store } from '../state.svelte';
  import { addendBreakdown } from '../derive';
  import { tooltip } from '../actions';
  import type { DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.is_magus}` block).

  let { d }: { d: DerivedTotals } = $props();

  // Lab/Casting Total picker. The Technique and Form option lists are derived
  // from the engine's own combination tables (never a hardcoded Art list), so
  // the picker stays in sync with whatever Arts the ruleset defines. The user's
  // pick falls back to the first available Art until they choose. The selection
  // lives on the store's `filters` so it survives this panel unmounting on a tab
  // switch; it is view state, never part of the saved entity.
  function distinct(values: string[]): string[] {
    return [...new Set(values)];
  }

  const techniques = $derived(distinct(d.lab_totals.map((c) => c.technique)));
  const forms = $derived(distinct(d.lab_totals.map((c) => c.form)));
  const technique = $derived(store.filters.derivedArtPicker.technique || techniques[0] || '');
  const form = $derived(store.filters.derivedArtPicker.form || forms[0] || '');

  const labCell = $derived(
    d.lab_totals.find((c) => c.technique === technique && c.form === form) ?? null,
  );
  const castCell = $derived(
    d.casting_totals.find((c) => c.technique === technique && c.form === form) ?? null,
  );

  // Localized display name for a catalogue id (an Art here), id as fallback.
  function name(id: string): string {
    return store.ruleset?.i18n[id]?.name ?? id;
  }
</script>

<!-- Lab & Casting Totals: on-demand for one chosen Technique × Form. The
     engine still computes every combination; we look up the single picked
     entry instead of rendering the exhaustive tables. -->
<div class="detail-section">
  <h3 class="detail-label">{store.t('derived-section-lab-casting')}</h3>
  <div class="art-picker">
    <label class="field inline">
      <span>{store.t('derived-picker-technique')}</span>
      <select
        bind:value={store.filters.derivedArtPicker.technique}
        data-testid="derived-technique-select"
      >
        {#each techniques as t (t)}
          <option value={t}>{name(t)}</option>
        {/each}
      </select>
    </label>
    <label class="field inline">
      <span>{store.t('derived-picker-form')}</span>
      <select bind:value={store.filters.derivedArtPicker.form} data-testid="derived-form-select">
        {#each forms as f (f)}
          <option value={f}>{name(f)}</option>
        {/each}
      </select>
    </label>
  </div>

  {#if labCell}
    <dl class="derived-grid" data-testid="derived-lab-total">
      <!-- Deliberately focusable: the only way a keyboard/screen-reader user can
           reach this breakdown tooltip (see S7 in the a11y review — a bare `title`
           attribute is mouse-only and unreachable otherwise). -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <dt tabindex="0" use:tooltip={{ text: addendBreakdown(labCell.addends, store.t) }}>
        {store.t('derived-lab-total')}
      </dt>
      <dd>{labCell.total}{labCell.deficient ? ' ' + store.t('derived-deficient') : ''}</dd>
      {#if labCell.within_focus != null}
        <dt class="derived-focus">{store.t('derived-within-focus')}</dt>
        <dd class="derived-focus">{labCell.within_focus}</dd>
      {/if}
      {#if labCell.enchanting !== labCell.total}
        <!-- Only shown when Weak Enchanter halves this cell's total for
             enchanting; equal to `total` (and hidden) for everyone else, so
             the Flaw's effect is visible instead of a silently-unused number. -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <dt
          class="derived-focus"
          tabindex="0"
          use:tooltip={{ text: store.t('derived-lab-enchanting-hint') }}
        >
          {store.t('derived-lab-enchanting')}
        </dt>
        <dd class="derived-focus">{labCell.enchanting}</dd>
      {/if}
    </dl>
  {/if}

  {#if castCell}
    <div class="table-scroll">
      <table class="derived-table casting" data-testid="derived-casting-total">
        <thead>
          <tr>
            <th></th>
            <th>{store.t('derived-cast-formulaic')}</th>
            <th>{store.t('derived-cast-ritual')}</th>
            <th>{store.t('derived-cast-spont-fatiguing')}</th>
            <th>{store.t('derived-cast-spont-non-fatiguing')}</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <th tabindex="0" use:tooltip={{ text: addendBreakdown(castCell.addends, store.t) }}
              >{store.t('derived-section-casting')}{castCell.deficient
                ? ' ' + store.t('derived-deficient')
                : ''}</th
            >
            <td>{castCell.formulaic}</td>
            <td>{castCell.ritual}</td>
            <td>{castCell.spontaneous_fatiguing}</td>
            <td>{castCell.spontaneous_non_fatiguing}</td>
          </tr>
          {#if castCell.within_focus}
            <tr class="derived-focus">
              <th>{store.t('derived-within-focus')}</th>
              <td>{castCell.within_focus.formulaic}</td>
              <td>{castCell.within_focus.ritual}</td>
              <td>{castCell.within_focus.spontaneous_fatiguing}</td>
              <td>{castCell.within_focus.spontaneous_non_fatiguing}</td>
            </tr>
          {/if}
          <tr class="non-standard">
            <th
              >{store.t('derived-cast-non-standard')}{castCell.non_standard.deft_form
                ? ' ' + store.t('derived-deft-form')
                : ''}</th
            >
            <td>{store.t('derived-cast-silent')}: {castCell.non_standard.silent}</td>
            <td>{store.t('derived-cast-still')}: {castCell.non_standard.still}</td>
            <td colspan="2"
              >{store.t('derived-cast-silent-still')}: {castCell.non_standard.silent_and_still}</td
            >
          </tr>
        </tbody>
      </table>
    </div>
  {/if}
</div>
