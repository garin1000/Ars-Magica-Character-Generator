<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName, formatSigned, paramHint } from '../derive';
  import type { CrisisAllowance, CrisisModifier } from '../ipc';
  import type { Characteristic } from '../types';

  // The Crisis sub-flow of the aging roll, extracted out of
  // `AgingRollCalculator.svelte` (V27, full-audit round): the largest and most
  // self-contained of that component's sub-flows, already bounded by its own
  // `.crisis` wrapper and its own `{#if preview.outcome.crisis}` gate in the
  // parent. Self-sufficient off the global store, mirroring how
  // `FamiliarPanel`/`TalismanPanel` mount beside `MagicPossessions` with no
  // props of their own — every value here already came from `store.agingPreview`
  // / `store.agingDraft` / `store.ruleset` in the parent, not from component-
  // local state, so there is nothing to thread through props.
  //
  // "Increase the character's Decrepitude first, and then roll on the Crisis
  // Table." (Ars Magica - Definitive Edition (Core Rules).md:16619) — the
  // parent still mounts this AFTER its own aging-points distribution UI for
  // that reason: the points placed there are the increase that comes first,
  // and they are a term of the total read below.
  // Source: Ars Magica - Definitive Edition (Core Rules).md:16619-16638

  const preview = $derived(store.agingPreview);
  const draft = $derived(store.agingDraft);
  const crisis = $derived(preview?.crisis ?? null);
  // "Roll a ten-sided die. Each number counts for its value, except that a zero
  // counts as ten." (`:474`) — the bounds are the ruleset's datum, not a literal.
  const crisisDieBounds = $derived(store.ruleset?.ruleset.aging?.crisis?.die ?? null);

  /** A Characteristic's name in words — `characteristic-<slug>`, never the slug. */
  function characteristicName(characteristic: Characteristic): string {
    return store.t(`characteristic-${characteristic}`);
  }

  /** A rules item's localized name — never its id. */
  function itemName(id: string): string {
    const localized = store.ruleset;
    return localized ? displayName(localized, id, undefined, paramHint(store.t)) : id;
  }

  /** Where a survival modifier comes from, in words: the item, or the cord. */
  function modifierSource(modifier: CrisisModifier): string {
    return modifier.source.kind === 'trait'
      ? itemName(modifier.source.item)
      : store.t(`crisis-modifier-${modifier.source.kind}`);
  }

  /** What the rules permit someone else to bring (`:16634`), stated in full. */
  function allowanceText(allowance: CrisisAllowance): string {
    return store.t(`crisis-allowance-${allowance.kind}`, {
      ability: itemName(allowance.ability),
      characteristic: characteristicName(allowance.characteristic),
      ease: allowance.ease_factor,
      botch: formatSigned(allowance.botch_penalty),
    });
  }

  function onCrisisDie(event: Event): void {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAgingCrisisDie(raw === '' ? null : Number(raw));
  }
</script>

<div class="crisis" data-testid="aging-crisis">
  <h4 class="detail-label">{store.t('crisis-label')}</h4>

  <label class="field">
    <span>{store.t('crisis-die-label')}</span>
    <!-- Bounds from `rules/core/aging.json`, never a literal: "a zero
         counts as ten" (`:474`) is the ruleset's datum. -->
    <input
      type="number"
      min={crisisDieBounds?.min}
      max={crisisDieBounds?.max}
      value={draft.crisisDie ?? ''}
      oninput={onCrisisDie}
      data-testid="crisis-die-input"
    />
  </label>

  {#if crisis}
    <!-- Announced: both figures change on every keystroke in the die. -->
    <p class="crisis-total" role="status" data-testid="crisis-total">
      {store.t('crisis-total-readout', { total: String(crisis.total.total) })}
    </p>
    <p class="crisis-parts" data-testid="crisis-total-parts">
      {store.t('crisis-total-parts', {
        die: formatSigned(crisis.total.die),
        age: formatSigned(crisis.total.age_modifier),
        decrepitude: formatSigned(crisis.total.decrepitude_score),
      })}
    </p>

    <!-- The row's own text is rules data keyed by its id; the severity is
         an engine enum, so it goes through `crisis-severity-<id>`. Neither
         is ever rendered as a slug. -->
    <p class="crisis-row" role="status" data-testid="crisis-row">
      {crisis.outcome.type === 'illness'
        ? store.t('crisis-row-readout-severity', {
            row: itemName(crisis.row),
            severity: store.t(`crisis-severity-${crisis.outcome.severity}`),
          })
        : store.t('crisis-row-readout', { row: itemName(crisis.row) })}
    </p>

    {#if crisis.survival}
      {@const survival = crisis.survival}
      <div class="crisis-survival" data-testid="crisis-survival">
        <p class="detail-label">{store.t('crisis-survival-label')}</p>
        {#if survival.ease_factor != null}
          <p data-testid="crisis-survival-ease-factor">
            {store.t('crisis-survival-ease-factor', { ease: survival.ease_factor })}
          </p>
        {:else}
          <!-- "Terminal illness. CrCo40 required to survive." (`:16632`) —
               an absent Ease Factor is NO roll, not an unbeatable one. -->
          <p data-testid="crisis-survival-no-roll">{store.t('crisis-survival-no-roll')}</p>
        {/if}
        <p data-testid="crisis-survival-ritual">
          {store.t('crisis-survival-ritual', { level: survival.ritual_level })}
        </p>

        {#if survival.modifiers.length > 0}
          <!-- Itemized, never summed away: the panel has to NAME each one.
               A character with no familiar carries no cord modifier at all,
               so no line appears rather than a "+0". -->
          <ul class="crisis-modifiers">
            {#each survival.modifiers as modifier, i (i)}
              <li data-testid="crisis-modifier-{i}">
                {store.t('crisis-modifier-row', {
                  source: modifierSource(modifier),
                  amount: formatSigned(modifier.amount),
                })}
              </li>
            {/each}
          </ul>
          <p class="crisis-modifier-total" data-testid="crisis-modifier-total">
            {store.t('crisis-modifier-total', {
              total: formatSigned(survival.modifier_total),
            })}
          </p>
        {/if}

        {#each survival.allowances as allowance, i (i)}
          <!-- Stated, never scored: the attendant's Medicine belongs to a
               character this sheet does not hold (`:16634`). -->
          <p class="hint" data-testid="crisis-allowance-{i}">{allowanceText(allowance)}</p>
        {/each}
      </div>
    {:else}
      <!-- "Bedridden for a week" (`:16626`) is time, not a roll: there is
           no survival read-out to give, and an empty one would read as
           "survivable on a 0". -->
      <p class="hint" data-testid="crisis-bedridden">{store.t('crisis-bedridden')}</p>
    {/if}
  {:else}
    <p class="hint" data-testid="crisis-die-unrolled">{store.t('crisis-die-unrolled')}</p>
  {/if}
</div>

<style>
  /* `.detail-label`, `.field` and `.hint` are shared globals in app.css. */

  .crisis-total {
    font-weight: 600;
  }

  .crisis-parts,
  .crisis-modifiers,
  .crisis-modifier-total {
    font-variant-numeric: tabular-nums;
  }

  /* The Crisis is a second reading inside the same roll, so it is set apart
     rather than run on from the aging outcome above it. */
  .crisis {
    border-left: 2px solid var(--border);
    padding-left: 0.6rem;
    margin-top: 0.4rem;
  }

  .crisis p {
    margin: 0.15rem 0;
  }

  .crisis-modifiers {
    list-style: none;
    margin: 0.15rem 0;
    padding: 0;
  }
</style>
