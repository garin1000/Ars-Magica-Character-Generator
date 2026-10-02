<script lang="ts">
  import { store } from '../state.svelte';
  import { addendBreakdown } from '../derive';
  import { tooltip } from '../actions';
  import type { Addend, CastingTotal, DerivedTotals } from '../types';

  // Split out of `DerivedTotalsPanel.svelte` (V26, full-audit round). Magus-only
  // (the parent mounts this only inside its own `{#if d.hermetically_trained}` block).

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

  // Every term the engine surfaces for a casting cell, in one list. The row
  // `<th>` heads four cells holding four different totals, so its tooltip was
  // never going to equal any one of them; it reads as "here is every term that
  // feeds this row" instead (Sabine's presentation call on Gerda round-4
  // finding 3). That only works if the list is complete — `ritual_addends` and
  // the per-scope `casting_mod_addends` were computed, serialized and typed
  // with no renderer, so a Method Caster's +3 and a Ritual's Artes Liberales
  // were simply missing from the breakdown. The Lab tooltip above needs no
  // equivalent: `lab.rs::lab_totals` folds its `lab_mod` into the single
  // `addends` list, which is why the two disagreed.
  function castingBreakdown(cell: CastingTotal): Addend[] {
    return [...cell.addends, ...cell.ritual_addends, ...cell.casting_mod_addends];
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
      {#if labCell.unusable}
        <!-- D81.8: the number is withheld outright, not merely annotated — 0 is a
             legitimate Lab Total and would be indistinguishable from this marker. -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <dd
          class="derived-unusable"
          tabindex="0"
          use:tooltip={{ text: store.t('derived-unusable-tooltip') }}
          data-testid="derived-lab-unusable"
        >
          {store.t('derived-unusable')}
        </dd>
      {:else}
        <dd>{labCell.total}{labCell.deficient ? ' ' + store.t('derived-deficient') : ''}</dd>
      {/if}
      <!-- review-ui-today finding 1: `lab.rs::lab_totals` computes
           `within_focus`/`within_potent_field`/`enchanting` unconditionally
           (gated only on holding a Focus/Potent Magic Virtue, or on Weak
           Enchanter, independent of `unusable`) — mirroring how
           `casting.rs::casting_totals` computes its own within-focus/
           within-potent-field figures. The Casting table below withholds
           those via the shared `castValue` snippet; this snippet does the
           same job for every row of the Lab Total `<dl>` below the main
           total, so a barred cell cannot leak a real number through ANY of
           them. -->
      {#snippet labValue(value: number)}
        {#if labCell?.unusable}
          <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
          <span
            class="derived-unusable"
            tabindex="0"
            use:tooltip={{ text: store.t('derived-unusable-tooltip') }}
            data-testid="derived-lab-unusable"
          >
            {store.t('derived-unusable')}
          </span>
        {:else}
          {value}
        {/if}
      {/snippet}
      {#if labCell.within_focus != null}
        <dt class="derived-focus">{store.t('derived-within-focus')}</dt>
        <dd class="derived-focus">{@render labValue(labCell.within_focus)}</dd>
      {/if}
      {#if labCell.within_potent_field != null}
        <dt class="derived-focus">{store.t('derived-within-potent-field')}</dt>
        <dd class="derived-focus">{@render labValue(labCell.within_potent_field)}</dd>
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
        <dd class="derived-focus">{@render labValue(labCell.enchanting)}</dd>
      {/if}
    </dl>
  {/if}

  {#if castCell}
    <!-- D81.8: shared by every cast-type cell below — the number is withheld
         outright for a barred cell, same reasoning as the Lab Total above. One
         snippet rather than four inline `{#if}`s repeating the same markup. -->
    {#snippet castValue(value: number)}
      {#if castCell?.unusable}
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <span
          class="derived-unusable"
          tabindex="0"
          use:tooltip={{ text: store.t('derived-unusable-tooltip') }}
          data-testid="derived-casting-unusable"
        >
          {store.t('derived-unusable')}
        </span>
      {:else}
        {value}
      {/if}
    {/snippet}
    <div class="table-scroll">
      <!-- Both axes are scoped (Sabine 9): the table has a header ROW naming the
           four cast types and a header COLUMN naming the line, and with neither
           `scope` present the cell→header association is not inferable, so a
           screen reader reads bare numbers. The empty corner cell is scoped too —
           it heads the row-label column.
           This is only safe because the non-standard figures moved OUT of the
           table below: they lay on neither axis, so scoping them would have had
           the reader assert a "Formulaic — No voice" relationship the rules do
           not have. -->
      <table class="derived-table casting" data-testid="derived-casting-total">
        <thead>
          <tr>
            <th scope="col"></th>
            <th scope="col">{store.t('derived-cast-formulaic')}</th>
            <th scope="col">{store.t('derived-cast-ritual')}</th>
            <th scope="col">{store.t('derived-cast-spont-fatiguing')}</th>
            <th scope="col">{store.t('derived-cast-spont-non-fatiguing')}</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <th
              scope="row"
              tabindex="0"
              use:tooltip={{ text: addendBreakdown(castingBreakdown(castCell), store.t) }}
              >{store.t('derived-section-casting')}{castCell.deficient
                ? ' ' + store.t('derived-deficient')
                : ''}</th
            >
            <td>{@render castValue(castCell.formulaic)}</td>
            <td>{@render castValue(castCell.ritual)}</td>
            <td>{@render castValue(castCell.spontaneous_fatiguing)}</td>
            <td>{@render castValue(castCell.spontaneous_non_fatiguing)}</td>
          </tr>
          {#if castCell.within_focus}
            <tr class="derived-focus">
              <th scope="row">{store.t('derived-within-focus')}</th>
              <td>{@render castValue(castCell.within_focus.formulaic)}</td>
              <td>{@render castValue(castCell.within_focus.ritual)}</td>
              <td>{@render castValue(castCell.within_focus.spontaneous_fatiguing)}</td>
              <td>{@render castValue(castCell.within_focus.spontaneous_non_fatiguing)}</td>
            </tr>
          {/if}
          {#if castCell.within_potent_field}
            <tr class="derived-focus">
              <th scope="row">{store.t('derived-within-potent-field')}</th>
              <td>{@render castValue(castCell.within_potent_field.formulaic)}</td>
              <td>{@render castValue(castCell.within_potent_field.ritual)}</td>
              <td>{@render castValue(castCell.within_potent_field.spontaneous_fatiguing)}</td>
              <td>{@render castValue(castCell.within_potent_field.spontaneous_non_fatiguing)}</td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>

    <!-- The non-standard (Silent/Still) figures, OUT of the table above and in a
         list of their own (Sabine 9). They are all variants of one number — the
         Formulaic total — so they never lay on that table's axes: filed as a row,
         "No voice: 20" sat under the *Formulaic* column header, "No gestures: 23"
         under *Ritual*, and the combined figure under a `colspan="2"` spanning the
         two spontaneous columns. Here each number has a `<dt>` that says what it
         is, which is the association a screen reader can actually use, and the
         colon is the list's own semantics rather than punctuation baked into the
         markup. -->
    <div class="detail-section">
      <!-- `.detail-label` is the global heading class (app.css); `.detail-sublabel`
           is scoped to FamiliarPanel and would render unstyled here. `<h4>` under
           this section's `<h3>` keeps the hierarchy gapless — the same pairing
           AgingCrisisPanel uses. -->
      <h4 class="detail-label">
        {store.t('derived-cast-non-standard')}{castCell.non_standard.deft_form
          ? ' ' + store.t('derived-deft-form')
          : ''}
      </h4>
      <dl class="derived-grid" data-testid="derived-cast-non-standard">
        <dt>{store.t('derived-cast-silent')}</dt>
        <dd>{castCell.non_standard.silent}</dd>
        <dt>{store.t('derived-cast-still')}</dt>
        <dd>{castCell.non_standard.still}</dd>
        <dt>{store.t('derived-cast-silent-still')}</dt>
        <dd>{castCell.non_standard.silent_and_still}</dd>
      </dl>
    </div>
  {/if}
</div>
