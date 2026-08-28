<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName, formatSigned, localizedSortKey, paramHint } from '../derive';

  // The Living Conditions a character lives under, as a checklist over the rules
  // catalogue.
  //
  // "AGING TOTAL: Stress die (no botch) + age/10 (round up) / - Living Conditions
  // modifier / - Longevity Ritual modifier"
  // Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16569
  //
  // The modifier is SUBTRACTED, so "a high Living Conditions modifier … indicate[s]
  // longer life" (`:16571`) — a good condition lowers the total. That is why every
  // row is shown with its own sign as the book prints it, and the sign is never
  // flipped here.
  //
  // A multi-select and not a radio group, because "Modifiers marked with an
  // asterisk are cumulative with each other" (`:16594`). That footnote is the only
  // thing the book says about combining rows; the unstarred rows being mutually
  // exclusive is inferred from it, so it is stated as guidance here and enforced
  // (as `living_conditions_conflict`) by the engine, never by disabling a box.

  // A ruleset shipping no aging block stands the whole subsystem down, so the
  // checklist is absent rather than empty.
  const rows = $derived.by(() => {
    const localized = store.ruleset;
    const conditions = localized?.ruleset.aging?.living_conditions ?? [];
    // The book's own order (`:16583-16592`): best conditions first, which puts the
    // cumulative rows together at the bottom. Ties break on the localized name so
    // the order is stable and reads as the active language sorts.
    return [...conditions].sort(
      (a, b) =>
        b.modifier - a.modifier ||
        localizedSortKey(localized!, a.id).localeCompare(localizedSortKey(localized!, b.id)),
    );
  });

  /** A row's localized rules name — never its id. */
  function conditionName(id: string): string {
    const localized = store.ruleset;
    return localized ? displayName(localized, id, undefined, paramHint(store.t)) : id;
  }

  const chosen = $derived(new Set(store.entity.living_conditions ?? []));

  // The ENGINE's resolved modifier, never a sum computed here: it also carries the
  // Virtue/Flaw `living_conditions` contributions (Mild Aging's +1, Poor Living
  // Conditions' -1), which have no row in this list, so a local sum would disagree
  // with the total the player is about to roll against.
  const total = $derived(store.effective?.aging?.living_conditions_modifier ?? null);

  const anyCumulative = $derived(rows.some((row) => row.cumulative));

  function onToggle(id: string, event: Event): void {
    store.setLivingCondition(id, (event.currentTarget as HTMLInputElement).checked);
  }
</script>

{#if rows.length > 0}
  <div class="detail-section living-conditions" data-testid="living-conditions">
    <h3 class="detail-label">{store.t('living-conditions-label')}</h3>
    <!-- The hint also carries the table's own "Average peasant 0" framing for an
         empty set. There is deliberately no separate "nothing chosen" line: the
         total below the list already states the figure, so an empty set was
         announced twice over, and a line that comes and goes is a height change
         the grid would rather not absorb. -->
    <p class="hint">{store.t('living-conditions-hint')}</p>

    <ul class="living-conditions-list">
      {#each rows as row (row.id)}
        <li>
          <label class="checkbox inline">
            <input
              type="checkbox"
              checked={chosen.has(row.id)}
              onchange={(event) => onToggle(row.id, event)}
              data-cumulative={row.cumulative ? 'true' : undefined}
              data-testid="living-condition-{row.id}"
            />
            <span>{conditionName(row.id)}</span>
          </label>
          <span class="modifier" data-testid="living-condition-modifier-{row.id}"
            >{formatSigned(row.modifier)}</span
          >
          {#if row.cumulative}
            <!-- The book marks these rows with an asterisk. A glyph alone carries
                 no meaning for assistive tech, so the word rides along with it. -->
            <span class="cumulative" aria-hidden="true">*</span>
            <span class="sr-only">{store.t('living-conditions-cumulative-label')}</span>
          {/if}
        </li>
      {/each}
    </ul>

    {#if anyCumulative}
      <p class="hint" data-testid="living-conditions-cumulative-note">
        {store.t('living-conditions-cumulative-note')}
      </p>
    {/if}

    {#if total != null}
      <!-- Announced: the figure changes the moment a box is ticked, and the change
           is the whole point of ticking it. -->
      <p class="living-conditions-total" role="status" data-testid="living-conditions-total">
        {store.t('living-conditions-total', { modifier: formatSigned(total) })}
      </p>
    {/if}
  </div>
{/if}

<style>
  /* `.detail-section`, `.checkbox.inline`, `.hint`, `.empty` and `.sr-only` are
     shared globals in app.css. */

  .living-conditions-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .living-conditions-list li {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
  }

  /* The modifier column: fixed width so the signs line up down the list, and
     tabular figures so +2 and -2 occupy the same space. */
  .modifier {
    font-variant-numeric: tabular-nums;
    min-width: 2.2ch;
    text-align: right;
  }

  .cumulative {
    color: var(--muted);
  }

  .living-conditions-total {
    font-weight: 600;
  }
</style>
