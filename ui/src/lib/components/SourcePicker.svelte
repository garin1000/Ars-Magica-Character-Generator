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
      {#each groups as group (group.key)}
        <h3 class="category" data-testid={group.headerTestid}>{group.header}</h3>
        <ul class="item-list">
          {#each group.items as item (getId(item))}
            {@const blocked = disabled?.(item) ?? false}
            <li>
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
  /* The "why is this greyed out" tooltip must stay reachable by keyboard and
     screen-reader users, so a blocked row uses `aria-disabled` (button stays
     focusable and keeps firing `focusin`) rather than native `disabled` — see
     `actions.ts`'s `tooltip` action, which shows on `mouseenter`/`focusin`.
     These rules mirror app.css's `.pick-row:disabled` look for the
     `aria-disabled="true"` state, using `:global()` so they apply regardless of
     Svelte's per-component style scoping (app.css itself is out of this
     component's scope, so the equivalent selector lives here instead). */
  :global(.pick-row[aria-disabled='true']) {
    cursor: default;
  }
  :global(.pick-row[aria-disabled='true']:hover) {
    background: transparent;
  }
  :global(.pick-row[aria-disabled='true'] .pick-plus) {
    color: var(--muted);
  }
</style>
