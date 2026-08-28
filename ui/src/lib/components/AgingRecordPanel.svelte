<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName, paramHint } from '../derive';
  import { CHARACTERISTICS, type AgingLogEntry, type Characteristic } from '../types';

  const apparentAge = $derived(store.entity.apparent_age ?? null);
  // Decrepitude is derived by the engine from the sum of aging points; hidden at 0.
  const decrepitude = $derived(store.effective?.decrepitude_score ?? 0);
  const decrepitudeEffect = $derived(store.entity.decrepitude_effect ?? '');
  const agingPoints = $derived(store.entity.aging_points ?? {});
  const agingLog = $derived(store.entity.aging_log ?? []);

  function onApparentAge(event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setApparentAge(raw === '' ? null : Number(raw));
  }

  function charLabel(characteristic: Characteristic): string {
    return store.t(`characteristic-${characteristic}`);
  }

  function numValue(event: Event): number {
    return Number((event.currentTarget as HTMLInputElement).value || 0);
  }

  /** An emptied field reads as "not given", not as 0 — same as the age field. */
  function optionalNumValue(event: Event): number | null {
    const raw = (event.currentTarget as HTMLInputElement).value;
    return raw === '' ? null : Number(raw);
  }

  /**
   * What the Crisis Table was asked and what it answered, in words — or that it
   * has not been asked yet.
   *
   * "**Crisis:** Increase the character's Decrepitude first, and then roll on the
   * Crisis Table." Source: Ars Magica - Definitive Edition (Core Rules).md:16619
   *
   * A `crisis` with no `crisis_row` is a Crisis the aging row demanded and nobody
   * has rolled — a real state, not an incomplete record, because the aging roll
   * happened whether or not the second die was thrown. The row travels as an id,
   * so its text comes from `rules/i18n/<lang>/aging.json`; the severity is an
   * engine enum and goes through `crisis-severity-<id>`. Neither is ever printed
   * as its slug.
   */
  function crisisText(entry: AgingLogEntry): string {
    if (entry.crisis_row == null) return store.t('aging-log-crisis-unrolled');
    const localized = store.ruleset;
    const args = {
      row: localized
        ? displayName(localized, entry.crisis_row, undefined, paramHint(store.t))
        : entry.crisis_row,
      total: String(entry.crisis_total ?? ''),
      die: String(entry.crisis_die ?? ''),
    };
    return entry.crisis_severity == null
      ? store.t('aging-log-crisis', args)
      : store.t('aging-log-crisis-severity', {
          ...args,
          severity: store.t(`crisis-severity-${entry.crisis_severity}`),
        });
  }
</script>

<!-- What aging has already done to this character: the apparent age, the derived
     Decrepitude, the accrued aging points and the per-year log. Extracted from
     CharacterDetails so the editor and the guided aging step mount the one
     surface and can never drift apart.
     `display: contents` inside `.character-details` (see app.css) keeps each block
     below its own item of that grid — a wrapper box would make the whole cluster
     one grid item, stacked in a single cell. -->
<div class="aging-record" data-testid="aging-record">
  <div class="detail-field">
    <label class="field">
      <span>{store.t('apparent-age-label')}</span>
      <input
        type="number"
        min="1"
        max="4294967295"
        value={apparentAge ?? ''}
        oninput={onApparentAge}
        data-testid="apparent-age-input"
      />
    </label>
  </div>

  {#if decrepitude > 0}
    <div class="detail-field">
      <span class="detail-label">{store.t('decrepitude-label')}</span>
      <span data-testid="decrepitude-readout">
        {store.t('decrepitude-readout', { score: String(decrepitude) })}
      </span>
    </div>
  {/if}

  <!-- The cumulative overall aging/decrepitude narrative, which grows over a
       character's life (Core Rules.md:16563-16577) — distinct from the aging log's
       per-year one-liners below. A textarea for the same reason `warping_effect` is
       one in CharacterDetails: it is the same kind of field, and the two must not
       offer different controls for the same kind of writing. -->
  <div class="detail-field">
    <label class="field">
      <span>{store.t('decrepitude-effect-label')}</span>
      <textarea
        class="decrepitude-effect"
        rows="3"
        value={decrepitudeEffect}
        oninput={(e) => store.setDecrepitudeEffect((e.currentTarget as HTMLTextAreaElement).value)}
        data-testid="decrepitude-effect-input"
      ></textarea>
    </label>
  </div>

  <div class="detail-section">
    <h3 class="detail-label">{store.t('aging-label')}</h3>
    <p class="detail-label">{store.t('aging-points-heading')}</p>
    <ul class="aging-list" data-testid="aging-points-list">
      {#each CHARACTERISTICS as characteristic (characteristic)}
        <li>
          <span class="char-name">{charLabel(characteristic)}</span>
          <input
            type="number"
            min="0"
            max="255"
            value={agingPoints[characteristic] ?? 0}
            oninput={(e) => store.setAgingPoints(characteristic, numValue(e))}
            data-testid="aging-points-{characteristic}"
          />
        </li>
      {/each}
    </ul>
    <p class="detail-label" data-testid="aging-points-note">
      {store.t('aging-points-note')}
    </p>
  </div>

  <!-- The log is a block of its own, and not part of the Aging section above,
       because it is the one block on this surface that grows without bound — one row
       per aging roll, and a pre-play catch-up can owe 25 (Core Rules.md:2232). As its
       own block it takes the grid's full-width row and a bounded scrollport
       (app.css), so adding a year scrolls in place instead of pushing every
       neighbouring block down. -->
  <div class="detail-section aging-log-block" data-testid="aging-log-block">
    <h3 class="detail-label">{store.t('aging-log-heading')}</h3>
    <ul class="twilight-list aging-log-scroll" data-testid="aging-log-list">
      {#each agingLog as entry, i (i)}
        <li>
          <input
            type="number"
            class="aging-log-year"
            min="-2147483648"
            max="2147483647"
            aria-label={store.t('aging-log-year-label')}
            value={entry.year ?? ''}
            oninput={(e) => store.setAgingLogEntryYear(i, optionalNumValue(e))}
            data-testid="aging-log-year-{i}"
          />
          <input
            class="twilight-desc"
            placeholder={store.t('aging-log-effect-placeholder')}
            aria-label={store.t('aging-log-effect-placeholder')}
            value={entry.effect}
            oninput={(e) =>
              store.setAgingLogEntryEffect(i, (e.currentTarget as HTMLInputElement).value)}
            data-testid="aging-log-effect-{i}"
          />
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('remove-item', { name: entry.effect })}
            onclick={() => store.removeAgingLogEntryAt(i)}
            data-testid="aging-log-remove-{i}"
          >
            ×
          </button>
          {#if entry.crisis}
            <!-- Read-only: the four crisis fields are the engine's record of a
                 resolved roll, not free text to edit. The year is taken back
                 whole (`revert_year`) rather than corrected field by field. -->
            <span class="aging-log-crisis" data-testid="aging-log-crisis-{i}">
              {crisisText(entry)}
            </span>
          {/if}
        </li>
      {:else}
        <li class="empty" data-testid="aging-log-empty">{store.t('aging-log-empty')}</li>
      {/each}
    </ul>
    <button type="button" onclick={() => store.addAgingLogEntry()} data-testid="aging-log-add">
      {store.t('aging-log-add')}
    </button>
  </div>
</div>

<style>
  /* A log row: year, effect, remove — and the Crisis read-out on its own line
     beneath them. The row must be a flex line for that `flex-basis: 100%` to mean
     anything, and it is what lets the effect field claim the rest of the row instead
     of keeping an `<input>`'s ~20-character default width, which was clipping the
     placeholder to "Describe the aging roll's e…". `min-width: 0` because a flex
     item's automatic minimum size is its content, which an input resists shrinking
     below. */
  .aging-log-block li {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .aging-log-block .twilight-desc {
    flex: 1;
    min-width: 0;
  }

  /* The engine's record of a resolved Crisis, sitting under its year's row rather
     than beside the editable fields — it is a read-out, not a control. */
  .aging-log-crisis {
    flex-basis: 100%;
    color: var(--muted);
    font-size: 0.9em;
  }
</style>
