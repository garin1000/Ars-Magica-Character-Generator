<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';

  // A generic "Selections" list: the shared skeleton every chosen-item list uses
  // (V/F, Abilities, Equipment, Spells). It renders one or more side-by-side
  // columns, each a panel with an optional heading, an optional header block
  // (e.g. the Spells budget bar), and either an empty message or grouped rows.
  //
  // Two grouping modes fall out of the same shape:
  //  - a category-grouped list (V/F, Abilities) supplies several groups, each with
  //    an `<h3 class="category">` header;
  //  - a flat/ordered list (Spells, Equipment) supplies a single header-less group.
  //
  // The caller carries each row's ORIGINAL entity index inside the row item, so
  // the row snippet's index-addressed mutators always target the right entity row
  // regardless of display order or per-side splitting. Row bodies (including
  // granted/mandatory markers) are the caller's `row` snippet.

  /** A row to render, keyed for `{#each}`; `item` is handed to the `row` snippet. */
  interface Row {
    key: string | number;
    item: T;
  }

  /** A group of rows under an optional `<h3 class="category">` header. */
  interface Group {
    key: string;
    /** Localized category header (never a raw id); omitted → header-less (flat). */
    header?: string;
    /** `<ul>` classes; defaults to `selection-list`. */
    listClass?: string;
    /** Optional `data-testid` on the `<ul>` (Spells/Equipment set one). */
    ulTestid?: string;
    rows: Row[];
  }

  /** One column (side) of the selections region — a single panel. */
  interface Column {
    key: string;
    /** Optional panel heading (V/F uses it for the per-side Virtues/Flaws title). */
    title?: string;
    /** Optional `data-testid` on the panel (V/F uses `selection-list-{side}`). */
    panelTestid?: string;
    /** Whether to show the empty message instead of the groups. */
    empty?: boolean;
    emptyText?: string;
    /** Class on the empty `<p>` (`muted` for V/F, `empty` elsewhere). */
    emptyClass?: string;
    /** Optional block rendered above the groups (the Spells budget/mastery bar). */
    header?: Snippet;
    groups: Group[];
  }

  let {
    columns,
    row,
  }: {
    columns: Column[];
    /** Renders the full `<li>` for a row (name, controls, markers, sub-pickers). */
    row: Snippet<[T]>;
  } = $props();
</script>

{#snippet columnPanel(col: Column)}
  <section class="panel" data-testid={col.panelTestid}>
    {#if col.title}
      <h2>{col.title}</h2>
    {/if}
    {#if col.header}
      {@render col.header()}
    {/if}
    {#if col.empty}
      <p class={col.emptyClass ?? 'empty'}>{col.emptyText}</p>
    {:else}
      {#each col.groups as group (group.key)}
        {#if group.header}
          <h3 class="category">{group.header}</h3>
        {/if}
        <ul class={group.listClass ?? 'selection-list'} data-testid={group.ulTestid}>
          {#each group.rows as r (r.key)}
            {@render row(r.item)}
          {/each}
        </ul>
      {/each}
    {/if}
  </section>
{/snippet}

{#if columns.length > 1}
  <!-- Two-or-more columns per side (V/F: Virtues beside Flaws) through one
       instance — the layout the tab previously built by nesting two components. -->
  <div class="region-columns">
    {#each columns as col (col.key)}
      {@render columnPanel(col)}
    {/each}
  </div>
{:else if columns.length === 1}
  {@render columnPanel(columns[0])}
{/if}
