<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned } from '../derive';
  import type { LongevitySource } from '../types';

  const longevity = $derived(store.entity.longevity_ritual ?? null);

  // The suggestion is whatever the ENGINE computed — never recomputed here. The
  // Creo Corpus Lab Total behind it carries Puissant/Deficient Art adjustments and
  // the Difficult Longevity Ritual halving, so deriving it in TS would fork the
  // single evaluation path (the `itemBudget`/`itemUsed` pattern).
  const hint = $derived(store.derived?.longevity?.hint ?? null);

  // An emptied `type="number"` input reads as '', and `Number('')` is 0 — so the
  // raw string is read here and an empty field passes `null` ("not entered"),
  // exactly as MagicPossessions' `onAura` does. Without this, clearing the box
  // would store a deliberate 0 there is no way back from.
  function onBonus(event: Event): void {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setLongevityBonus(raw === '' ? null : Number(raw));
  }

  function text(event: Event): string {
    return (event.currentTarget as HTMLInputElement).value;
  }
</script>

<div class="detail-section">
  <h3 class="detail-label">{store.t('longevity-label')}</h3>
  {#if longevity}
    <div class="longevity-source" role="radiogroup" aria-label={store.t('longevity-source-label')}>
      {#each ['self_made', 'external'] as source (source)}
        <label class="radio">
          <input
            type="radio"
            name="longevity-source"
            value={source}
            checked={longevity.source === source}
            onchange={() => store.setLongevitySource(source as LongevitySource)}
            data-testid="longevity-source-{source}"
          />
          <span>{store.t(`longevity-source-${source}`)}</span>
        </label>
      {/each}
    </div>

    <!-- The bonus is entered for BOTH sources: it was fixed by the Lab Total of the
         season the ritual was made, so it is a stored fact, not a live figure. -->
    <label class="field inline">
      <span>{store.t('longevity-bonus-label')}</span>
      <input
        type="number"
        min="-128"
        max="127"
        value={longevity.bonus ?? ''}
        oninput={onBonus}
        data-testid="longevity-bonus"
      />
    </label>
    {#if longevity.bonus == null}
      <p class="empty" data-testid="longevity-not-entered">{store.t('longevity-not-entered')}</p>
    {/if}

    {#if hint}
      <!-- Announced (role="status" implies aria-live="polite"): the suggestion
           shifts reactively as the Lab Total behind it changes (an Art bought, a
           Puissant Art added), matching AgingSchedulePanel's and
           LivingConditionsPicker's own reactive figures. -->
      <p class="hint" role="status" data-testid="longevity-hint">
        {store.t('longevity-hint', {
          bonus: formatSigned(hint.suggested_bonus),
          total: hint.lab_total,
        })}
        {#if hint.halved}
          <span class="halved" data-testid="longevity-hint-halved">
            {store.t('longevity-hint-halved')}
          </span>
        {/if}
      </p>
    {/if}

    <label class="field">
      <span>{store.t('longevity-focus-label')}</span>
      <input
        type="text"
        value={longevity.focus ?? ''}
        placeholder={store.t('longevity-focus-placeholder')}
        oninput={(e) => store.setLongevityFocus(text(e))}
        data-testid="longevity-focus"
      />
    </label>

    <p class="hint" data-testid="longevity-sterility-note">
      {store.t('longevity-sterility-note')}
    </p>

    <button
      type="button"
      onclick={() => store.removeLongevityRitual()}
      data-testid="longevity-remove"
    >
      {store.t('longevity-remove')}
    </button>
  {:else}
    <button
      type="button"
      onclick={() => store.addLongevityRitual('self_made')}
      data-testid="longevity-add"
    >
      {store.t('longevity-add')}
    </button>
  {/if}
</div>

<style>
  /* `.detail-section` (spacing within a section), `.field` / `.field.inline` (label
     above vs. beside its input) and `.hint` (muted advisory text) are shared
     globals in app.css. */

  /* Section-level action buttons (Add / Remove ritual) are direct children of a
     `.detail-section`, which is a column flex with the default
     `align-items: stretch` — so without this they stretch to the full panel
     width. Shrink them to their label. */
  .detail-section > button {
    align-self: flex-start;
  }

  /* Longevity source radios on one row. */
  .longevity-source {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem;
  }

  .radio {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .empty {
    color: var(--muted);
  }

  /* A halving is a rules consequence the player should not miss, so it is marked
     rather than folded silently into the suggested number. */
  .halved {
    font-weight: 600;
  }
</style>
