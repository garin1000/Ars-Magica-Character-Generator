<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName, I32_MAX, I32_MIN, paramHint, U32_MAX } from '../derive';
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
  /**
   * Whether the engine wrote this row, as opposed to the player typing it.
   *
   * A year resolved by `aging::resolve_year` always carries both the die the player
   * entered and the total it made; a hand-written row carries neither, and neither
   * does a row loaded from a schema-14 save, which knew only `{ year, effect }`.
   * Both fields are checked rather than `age`, so nothing is summarised from a
   * half-filled record.
   */
  function isRecorded(entry: AgingLogEntry): boolean {
    return entry.die != null && entry.total != null;
  }

  /**
   * What the row's free-text box asks for. On an engine-recorded row the summary
   * beside it already states the roll, so the box offers itself as the optional note
   * it now is; on a hand-written row the free text IS the record, and keeps asking
   * for the description.
   */
  function notePlaceholder(entry: AgingLogEntry): string {
    return store.t(
      isRecorded(entry) ? 'aging-log-note-placeholder' : 'aging-log-effect-placeholder',
    );
  }

  /**
   * What the year's roll did, in words — built from the structured fields, never
   * from stored prose (guided-creation-review-2026-08 #27).
   *
   * The engine records the roll precisely and deliberately leaves `effect` empty
   * ("the structured fields are the record", `aging.rs`), which left the player
   * copying out by hand what the app already knew. This renders that record instead.
   * It is NOT written back: saves store choices, and generated prose in the file
   * would freeze one language into it — a German player opening an English-authored
   * character would read English.
   *
   * The die and the total are in it because they are what the outcome was read off:
   * without them a recorded year cannot be checked against the table, and the Crisis
   * line beside it already reports its own die and total the same way.
   *
   * The awards are listed in `CHARACTERISTICS` order — the order the Aging Points
   * list below uses — not in whatever order the map happens to iterate, and each
   * goes through `characteristic-<slug>`. A roll that awarded nothing says so:
   * silence would be indistinguishable from an unrecorded row. Both point and
   * apparent-age sentences are the calculator's own strings, so the preview and the
   * record read identically.
   */
  function summaryText(entry: AgingLogEntry): string {
    const parts = [
      store.t('aging-log-roll', { total: String(entry.total), die: String(entry.die) }),
    ];
    const awarded = CHARACTERISTICS.filter(
      (characteristic) => (entry.points?.[characteristic] ?? 0) > 0,
    );
    if (awarded.length === 0) parts.push(store.t('aging-log-points-none'));
    for (const characteristic of awarded) {
      parts.push(
        store.t('aging-outcome-points_fixed', {
          points: entry.points?.[characteristic] ?? 0,
          characteristic: charLabel(characteristic),
        }),
      );
    }
    parts.push(
      store.t(
        entry.apparent_age_increased
          ? 'aging-outcome-apparent_age'
          : 'aging-outcome-no_apparent_aging',
      ),
    );
    return parts.join(' ');
  }

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

<!-- What aging has already done to this character: the per-year log, then the
     running totals it explains — the apparent age, the derived Decrepitude, its
     narrative and the accrued aging points. Extracted from CharacterDetails so the
     editor and the guided aging step mount the one surface and can never drift
     apart.
     `display: contents` inside `.character-details` (see app.css) keeps each of the
     two blocks below its own item of that grid — a wrapper box here would make the
     whole cluster one grid item, stacked in a single cell. Each of them then takes a
     full-width row of that grid, as every aging block does since #22. -->
<div class="aging-record" data-testid="aging-record">
  <!-- The log LEADS the record (manual-testing-findings-2026-09-03 #22/#24): it is
       what the player has just written to, it is where a year is taken back, and it
       is the block that has to be reachable without scrolling. The accumulated
       read-outs below are its running totals, which the engine maintains — worth
       having, not worth the top of the surface. It is a block of its own, and not
       part of the section below, because it is the one block here that grows without
       bound — one row per aging roll, and a pre-play catch-up can owe 25
       (Core Rules.md:2232). Its bounded scrollport (`.aging-log-scroll`, app.css)
       is what keeps that growth from moving anything at all: adding a year scrolls
       in place. -->
  <div class="detail-section aging-log-block" data-testid="aging-log-block">
    <h3 class="detail-label">{store.t('aging-log-heading')}</h3>
    <ul class="twilight-list aging-log-scroll" data-testid="aging-log-list">
      {#each agingLog as entry, i (i)}
        <li>
          <input
            type="number"
            class="aging-log-year"
            min={I32_MIN}
            max={I32_MAX}
            aria-label={store.t('aging-log-year-label')}
            value={entry.year ?? ''}
            oninput={(e) => store.setAgingLogEntryYear(i, optionalNumValue(e))}
            data-testid="aging-log-year-{i}"
          />
          <!-- The player's own note, and still theirs: the engine never writes here.
               On an engine-recorded row the summary below states what the roll did,
               so the box stops demanding a description of it and offers itself as the
               optional note it now is. A hand-written row keeps the old wording —
               there, the free text IS the record. -->
          <input
            class="twilight-desc"
            placeholder={notePlaceholder(entry)}
            aria-label={notePlaceholder(entry)}
            value={entry.effect}
            oninput={(e) =>
              store.setAgingLogEntryEffect(i, (e.currentTarget as HTMLInputElement).value)}
            data-testid="aging-log-effect-{i}"
          />
          <!-- The row's undo, and since #19 the ONLY one: the calculator's list of
               "Take back age N" buttons is gone, because one button per recorded
               year is unusable at forty. On an engine-recorded row (one carrying an
               `age`) this hands the year to `aging::revert_year`, which takes its
               Aging Points and its year of apparent age back off with it — so the
               accessible name is the calculator's own take-back wording rather than
               "Remove", which would announce a deletion and describe an undo. A
               hand-written row has nothing mechanical to undo and is removed, and
               says so. -->
          <button
            type="button"
            class="icon-btn"
            aria-label={entry.age == null
              ? store.t('remove-item', { name: entry.effect })
              : store.t('aging-revert', { age: String(entry.age) })}
            onclick={() => store.removeAgingLogEntryAt(i)}
            data-testid="aging-log-remove-{i}"
          >
            ×
          </button>
          {#if isRecorded(entry)}
            <!-- Read-only, on its own line beneath the row's controls: the engine's
                 record of the roll, not free text to edit. A year is taken back
                 whole (`revert_year`) rather than corrected field by field. -->
            <span class="aging-log-readout" data-testid="aging-log-summary-{i}">
              {summaryText(entry)}
            </span>
          {/if}
          {#if entry.crisis}
            <!-- Read-only: the four crisis fields are the engine's record of a
                 resolved roll, not free text to edit. The year is taken back
                 whole (`revert_year`) rather than corrected field by field. -->
            <span class="aging-log-readout" data-testid="aging-log-crisis-{i}">
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

  <!-- The running totals, as ONE stage rather than four separate items of the
       `.character-details` grid. Auto-placed individually they were scattered into
       whatever cells were left around the tall roll calculator, which is half of the
       "band of columns" #22 reported; grouped, they are a single full-width row that
       lays its own fields out across it (`.aging-state`, app.css). This wrapper DOES
       generate a box, unlike `.aging-record` above — that is the point of it. -->
  <div class="aging-state" data-testid="aging-state">
    <div class="detail-field">
      <label class="field">
        <span>{store.t('apparent-age-label')}</span>
        <input
          type="number"
          min="1"
          max={U32_MAX}
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
       per-year one-liners above. A textarea for the same reason `warping_effect` is
       one in CharacterDetails: it is the same kind of field, and the two must not
       offer different controls for the same kind of writing. -->
    <div class="detail-field">
      <label class="field">
        <span>{store.t('decrepitude-effect-label')}</span>
        <textarea
          class="decrepitude-effect"
          rows="3"
          value={decrepitudeEffect}
          oninput={(e) =>
            store.setDecrepitudeEffect((e.currentTarget as HTMLTextAreaElement).value)}
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
            <span class="char-name" id="aging-points-label-{characteristic}"
              >{charLabel(characteristic)}</span
            >
            <input
              type="number"
              min="0"
              max="255"
              value={agingPoints[characteristic] ?? 0}
              oninput={(e) => store.setAgingPoints(characteristic, numValue(e))}
              aria-labelledby="aging-points-label-{characteristic}"
              data-testid="aging-points-{characteristic}"
            />
          </li>
        {/each}
      </ul>
    </div>
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

  /* The engine's record of a resolved year — the roll, and beneath it a Crisis where
     one was rolled — sitting under its row rather than beside the editable fields:
     these are read-outs, not controls. Two lines and not one, because they are two
     rolls against two different tables, resolved in that order (`:16619`), and one
     run-on sentence would read as a single result. */
  .aging-log-readout {
    flex-basis: 100%;
    color: var(--muted);
    font-size: 0.9em;
  }
</style>
