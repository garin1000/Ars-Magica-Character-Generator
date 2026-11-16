<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import { formatSigned } from '../derive';
  import type { LongevitySource } from '../types';

  const longevity = $derived(store.entity.longevity_ritual ?? null);

  // The suggestion is whatever the ENGINE computed — never recomputed here. The
  // Creo Corpus Lab Total behind it carries Puissant/Deficient Art adjustments and
  // the Difficult Longevity Ritual halving, so deriving it in TS would fork the
  // single evaluation path (the `itemBudget`/`itemUsed` pattern).
  const hint = $derived(store.derived?.longevity?.hint ?? null);

  // manual-testing-findings-2026-11-15 #9 (F9) + follow-up: forwarding an empty
  // read as `null` (so a genuine clear is never stored as a deliberate 0, exactly
  // as MagicPossessions' `onAura` does) is not enough on its own — a
  // `type="number"` input reads an INCOMPLETE entry, a bare "-" on the way to
  // "-4", back as the empty string too, so typing "-" would write `null`
  // immediately and the plain `value={longevity.bonus ?? ''}` binding would blank
  // the field before the digit after the sign could ever be typed. F9 fixed that
  // by deferring EVERY empty read to commit — but that made the genuine clear
  // depend on the `change` event (blur or Enter), and WebKitGTK does not fire
  // `change` on Enter for this element (confirmed against the shipped binary:
  // `magus-possessions.e2e.js`'s longevity-ritual test), leaving the field
  // visibly blank with the old bonus still stored underneath.
  //
  // `validity.badInput` tells the two empty reads apart without waiting for
  // commit at all: it is `false` for a genuine clear and `true` only for an
  // incomplete token the UA cannot parse (a bare "-"), so a genuine clear can
  // forward immediately — exactly like the other nullable fields already do
  // (`AgeFields`, `AgingRecordPanel`'s apparent age, `AgingRollCalculator`'s
  // distribution) — while the half-typed case still waits for commit.
  function onBonus(event: Event): void {
    const target = event.currentTarget as HTMLInputElement;
    const raw = target.value;
    if (raw === '') {
      if (!target.validity.badInput) store.setLongevityBonus(null);
      return;
    }
    const typed = Number(raw);
    if (!Number.isFinite(typed) || typed < -128 || typed > 127) return;
    store.setLongevityBonus(typed);
  }

  // Commit (blur or Enter): the genuine-clear-as-null rule still applies, exactly
  // as `onBonus` would have, had the field never left bounds.
  function commitBonus(raw: string): void {
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
        use:commitStored={{ read: () => longevity?.bonus, commit: commitBonus }}
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
        {#if hint.halved}
          <!-- #14 (2026-10-19 ruling): names BOTH the raw Lab Total and what it
               was halved to, so the steps read in order ("Lab Total 4, halved
               to 2 -> +1") instead of only showing the already-halved figure. -->
          {store.t('longevity-hint-halved-detail', {
            before: hint.lab_total_before_halving,
            after: hint.lab_total,
            bonus: formatSigned(hint.suggested_bonus),
          })}
        {:else}
          {store.t('longevity-hint', {
            bonus: formatSigned(hint.suggested_bonus),
            total: hint.lab_total,
          })}
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
</style>
