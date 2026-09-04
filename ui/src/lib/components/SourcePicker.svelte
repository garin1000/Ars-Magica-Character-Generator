<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';
  import { store } from '../state.svelte';
  import { tooltip, type TooltipContent } from '../actions';

  // A generic "Available" source picker: the shared skeleton every source list
  // uses (V/F, Abilities, Equipment, Spells) — a fixed filter bar on top and a
  // category-grouped, click-to-add list below it. Everything surface-specific is
  // supplied by the caller: the normalized groups, the filter-bar contents
  // (`filters` snippet, so each list keeps its own controls and exact testids),
  // the per-row body (`row` snippet), and the add/disable/tooltip behavior.

  /** One category group of source rows, already filtered and ordered. */
  interface SourceGroup {
    /** Stable `{#each}` key for the group. */
    key: string;
    /** Localized category header (never a raw id). */
    header: string;
    /** Optional `data-testid` on the group header (only Spells sets one today). */
    headerTestid?: string;
    items: T[];
  }

  let {
    title,
    groups,
    getId,
    onAdd,
    disabled,
    tip,
    row,
    filters,
  }: {
    /** Optional panel heading (V/F uses it for the per-side Virtues/Flaws title). */
    title?: string;
    groups: SourceGroup[];
    /** The item's rules id — drives the `add-{id}` testid and the `{#each}` key. */
    getId: (item: T) => string;
    onAdd: (item: T) => void;
    /** Whether the add control is greyed (already taken / not currently allowed). */
    disabled?: (item: T) => boolean;
    /** Hover/focus tooltip content for the row (omitted → no tooltip). */
    tip?: (item: T) => TooltipContent | undefined;
    /** Renders the row body inside the add button (name, badges, level tag, …). */
    row: Snippet<[T]>;
    /** Renders the filter-bar controls (search + selects), owning their testids. */
    filters: Snippet;
  } = $props();
</script>

<section class="panel">
  {#if title}
    <h2>{title}</h2>
  {/if}
  {#if store.ruleset}
    <div class="filter-bar">
      {@render filters()}
    </div>
    <div class="list-scroll">
      {#if groups.length === 0}
        <!-- Otherwise a filter matching nothing renders a completely blank list
             area — indistinguishable from a slow-loading or broken panel. -->
        <p class="empty" data-testid="source-no-results">{store.t('filter-no-results')}</p>
      {/if}
      {#each groups as group (group.key)}
        <h3 class="category" data-testid={group.headerTestid}>{group.header}</h3>
        <ul class="item-list">
          {#each group.items as item (getId(item))}
            {@const blocked = disabled?.(item) ?? false}
            <li>
              <!-- `aria-disabled`, never the native `disabled` attribute: a
                   natively disabled button is not focusable, and the row must stay
                   focusable or the tooltip explaining WHY it is blocked (opened on
                   `focusin` by `use:tooltip`, which also points `aria-describedby`
                   at it) becomes unreachable by keyboard. It is also the single
                   marker every blocked treatment keys on — screen readers announce
                   it as "unavailable", and `app.css` hangs the dim, the muted glyph
                   and the suppressed hover off the same attribute. -->
              <button
                type="button"
                class="pick-row"
                aria-disabled={blocked}
                onclick={() => {
                  if (!blocked) onAdd(item);
                }}
                use:tooltip={tip?.(item)}
                data-testid="add-{getId(item)}"
              >
                {@render row(item)}
                <span class="pick-plus" aria-hidden="true">+</span>
              </button>
            </li>
          {/each}
        </ul>
      {/each}
    </div>
  {:else}
    <p>{store.t('loading')}</p>
  {/if}
</section>

<style>
  /* The blocked-row treatment (dim + muted glyph + no hover highlight) is NOT
     here: it belongs with `.pick-row`/`.pick-plus` themselves, which live in
     `app.css`, and splitting one look across two files is what let a dead
     `.pick-row:disabled` rule sit there unnoticed. See the
     `.pick-row[aria-disabled='true']` block in `app.css` (and
     `src/app.css.test.ts`, which guards it) for the treatment and the reasoning. */

  /* Matches the `.empty` treatment other lists give their own empty state
     (AbilityTab's Selected side, EquipmentTab, Reputations); each keeps its own
     copy because Svelte styles are component-scoped. */
  .empty {
    color: var(--muted);
  }
</style>
