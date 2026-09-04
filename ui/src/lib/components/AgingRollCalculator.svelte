<script lang="ts">
  import { store } from '../state.svelte';
  import { formatSigned, resolveIssueArgs } from '../derive';
  import { CHARACTERISTICS } from '../types';
  import type { AgingNote, AgingPointAward } from '../ipc';
  import type { AgingScheduleYear, Characteristic } from '../types';
  import AgingCrisisPanel from './AgingCrisisPanel.svelte';

  // One year's aging roll, from the die the player rolled to the character it
  // makes.
  //
  // "AGING TOTAL: Stress die (no botch) + age/10 (round up) / - Living Conditions
  // modifier / - Longevity Ritual modifier"
  // Source: Ars Magica - Definitive Edition (Core Rules).md:16567-16569
  //
  // The app NEVER rolls: the engine carries no `rand` dependency, so the player
  // rolls at the table and types the result. A stress die explodes, so the input
  // has a floor of 0 and no ceiling — and no lookup table in JS could stand in for
  // the engine, which is why the total and the outcome are asked for over IPC
  // rather than derived here.
  //
  // The die is never written to the character: it lives in `store.agingDraft`,
  // which is UI-only state, so opening the calculator and typing in it cannot
  // dirty the document. What the character keeps is the year's *result*, appended
  // to its aging log by the engine's single writer when Apply is pressed.

  const aging = $derived(store.effective?.aging ?? null);
  const schedule = $derived(aging?.schedule ?? []);

  // There is deliberately no per-year "Take back age N" list here any more
  // (manual-testing-findings-2026-09-03 #19). One button per recorded year is fine
  // at three and unusable at forty, on the one surface whose complaint was its
  // height. Every recorded year already has a row in the aging log, and that row's ×
  // hands the year to the same `aging::revert_year` (`store.removeAgingLogEntryAt`)
  // under the same `aging-revert` wording — and `revert_year` takes any recorded
  // year, not only the latest, so nothing about the undo is narrowed.

  const draft = $derived(store.agingDraft);
  const preview = $derived(store.agingPreview);

  /** A year's option label: its age, its calendar year where one exists, and
   *  whether the log already records it. Never a bare number. */
  function yearLabel(year: AgingScheduleYear): string {
    const args = { age: String(year.age), recorded: year.recorded ? 'yes' : 'no' };
    return year.year == null
      ? store.t('aging-year-option', args)
      : store.t('aging-year-option-dated', { ...args, year: String(year.year) });
  }

  /** A Characteristic's name in words — `characteristic-<slug>`, never the slug. */
  function characteristicName(characteristic: Characteristic): string {
    return store.t(`characteristic-${characteristic}`);
  }

  // Every term of the total as the engine reported it, each already signed. The
  // two modifiers are SUBTRACTED (`:16571`: "a high Longevity Ritual modifier and
  // a high Living Conditions modifier both indicate longer life"), so their stored
  // sign is flipped for display; the Virtue/Flaw modifier is ADDED with its own.
  const parts = $derived(
    preview
      ? {
          die: formatSigned(preview.total.die),
          age: formatSigned(preview.total.age_modifier),
          conditions: formatSigned(-preview.total.living_conditions.total),
          longevity: formatSigned(-preview.total.longevity_bonus),
          traits: formatSigned(preview.total.trait_modifier),
        }
      : null,
  );

  // The awards the table leaves to the player. A `named` row places its own points
  // (`:16607` "1 Aging Point in Str and Sta"), so only the other two kinds are
  // distributed here.
  const openAwards = $derived(
    (preview?.outcome.awards ?? []).filter((award) => award.target.kind !== 'named'),
  );
  // "Gain sufficient Aging Points (in any Characteristics)" — plural, so the
  // points may be spread over several Characteristics, and reaching the next level
  // of Decrepitude costs five of them. Forcing them all into one target would force
  // Characteristic drops the player may legally avoid.
  // Source: Ars Magica - Definitive Edition (Core Rules).md:16602, :16611, :16615
  const owed = $derived(openAwards.reduce((sum, award) => sum + (award.points ?? 0), 0));
  // `points` is null only when the next Decrepitude level lies beyond the
  // advancement table. Reported as unpriceable, never silently costed at zero — and
  // never applied, since there is no number to match.
  const unpriceable = $derived(openAwards.some((award) => award.points == null));
  const placed = $derived(
    Object.values(draft.distribution).reduce((sum, points) => sum + (points ?? 0), 0),
  );
  const canApply = $derived(preview != null && !unpriceable && placed === owed);

  /** What one award costs, in words: the Characteristic named, or the choice left. */
  function awardText(award: AgingPointAward): string {
    switch (award.target.kind) {
      case 'named':
        return store.t('aging-outcome-points_fixed', {
          points: award.points ?? 0,
          characteristic: characteristicName(award.target.characteristic),
        });
      case 'player_choice':
        return store.t('aging-outcome-points_any', { points: award.points ?? 0 });
      case 'next_decrepitude_level':
        return award.points == null
          ? store.t('aging-outcome-decrepitude_unpriceable')
          : store.t('aging-outcome-decrepitude_and_crisis', { points: award.points });
    }
  }

  function onYear(event: Event): void {
    const raw = (event.currentTarget as HTMLSelectElement).value;
    store.setAgingYear(raw === '' ? null : Number(raw));
  }

  function onDie(event: Event): void {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAgingDie(raw === '' ? null : Number(raw));
  }

  function onDistribute(characteristic: Characteristic, event: Event): void {
    const raw = (event.currentTarget as HTMLInputElement).value;
    store.setAgingDistribution(characteristic, raw === '' ? null : Number(raw));
  }

  /** What the applied year had to say — `aging-note-<kind>`, never the tag. */
  function noteText(note: AgingNote): string {
    return store.t(`aging-note-${note.kind}`);
  }

  // The Crisis sub-flow (Core Rules.md:16619-16638) is `AgingCrisisPanel.svelte`
  // (V27, full-audit round) — self-sufficient off the store, so nothing about
  // it lives here beyond the `{#if preview.outcome.crisis}` mount gate below.
</script>

{#if aging && schedule.length > 0}
  <div class="detail-section aging-calculator" data-testid="aging-calculator">
    <h3 class="detail-label">{store.t('aging-roll-label')}</h3>

    <label class="field">
      <span>{store.t('aging-year-label')}</span>
      <select onchange={onYear} data-testid="aging-year-select">
        {#each schedule as year (year.age)}
          <option value={year.age} selected={year.age === store.agingYear}>
            {yearLabel(year)}
          </option>
        {/each}
      </select>
    </label>

    <label class="field">
      <span>{store.t('aging-die-label')}</span>
      <!-- No `max`: a stress die explodes, so any total is reachable. The floor is
           0 because the die is never negative; every modifier that can go below it
           is a separate term of the total. -->
      <input
        type="number"
        min="0"
        value={draft.die ?? ''}
        oninput={onDie}
        data-testid="aging-die-input"
      />
    </label>
    <p class="hint" data-testid="aging-die-hint">{store.t('aging-die-hint')}</p>

    {#if preview && parts}
      <!-- Announced: both figures change on every keystroke in the die field. -->
      <p class="aging-total" role="status" data-testid="aging-total">
        {store.t('aging-total-readout', { total: String(preview.total.total) })}
      </p>
      <p class="aging-parts" data-testid="aging-total-parts">
        {store.t('aging-total-parts', parts)}
      </p>

      {#if preview.total.capped_by_longevity}
        <!-- "treats all rolls of 10 or more as rolls of 9 until he reaches the age
             of 35 … he is at no risk of actually aging before any other
             characters." The clamp is on the TOTAL, and this says it bit THIS roll.
             Source: Ars Magica - Definitive Edition (Core Rules).md:16575 -->
        <p class="hint" data-testid="aging-die-capped">
          {store.t('aging-die-capped', {
            uncapped: String(preview.total.uncapped_total),
            total: String(preview.total.total),
          })}
        </p>
      {/if}

      <div class="aging-outcome" role="status" data-testid="aging-outcome">
        <!-- "Otherwise, the character's apparent age increases by one year."
             Source: Ars Magica - Definitive Edition (Core Rules).md:16577 -->
        <p>
          {store.t(
            preview.outcome.apparent_age_increases
              ? 'aging-outcome-apparent_age'
              : 'aging-outcome-no_apparent_aging',
          )}
        </p>
        {#each preview.outcome.awards as award, i (i)}
          <p>{awardText(award)}</p>
        {/each}
      </div>

      {#if preview.outcome.crisis}
        <p class="warning" data-testid="aging-outcome-crisis">
          {store.t('aging-outcome-crisis-note')}
        </p>
      {/if}

      {#if owed > 0}
        <div class="aging-distribute" data-testid="aging-distribute">
          <p>{store.t('aging-distribute', { points: owed })}</p>
          <ul class="aging-distribute-list">
            {#each CHARACTERISTICS as characteristic (characteristic)}
              <li>
                <label class="field inline">
                  <span>{characteristicName(characteristic)}</span>
                  <input
                    type="number"
                    min="0"
                    value={draft.distribution[characteristic] ?? ''}
                    oninput={(event) => onDistribute(characteristic, event)}
                    data-testid="aging-distribute-{characteristic}"
                  />
                </label>
              </li>
            {/each}
          </ul>
          <p class="aging-remaining" role="status" data-testid="aging-distribute-remaining">
            {store.t('aging-distribute-remaining', { placed, owed })}
          </p>
        </div>
      {/if}

      {#if preview.outcome.crisis}
        <!-- "Increase the character's Decrepitude first, and then roll on the
             Crisis Table." (`:16619`) The panel sits AFTER the distribution for
             that reason: the points placed above are the increase that comes
             first, and they are a term of the total read below.
             Source: Ars Magica - Definitive Edition (Core Rules).md:16619-16638 -->
        <AgingCrisisPanel />
      {/if}

      <div class="aging-actions">
        <button
          type="button"
          disabled={!canApply}
          onclick={() => store.applyAgingRoll()}
          data-testid="aging-apply"
        >
          {store.t('aging-apply')}
        </button>
        <button
          type="button"
          onclick={() => store.clearAgingDraft()}
          data-testid="aging-calculator-clear"
        >
          {store.t('aging-calculator-clear')}
        </button>
      </div>
    {/if}

    <p class="hint" data-testid="aging-calculator-note">{store.t('aging-calculator-note')}</p>

    {#if store.agingNotes.length > 0}
      <!-- What the year just applied had to TELL the player, as opposed to what it
           wrote. A spent Longevity Ritual (`:16573`) is the only one today, and it
           exists precisely because the engine leaves the ritual entry alone — so
           this is the sole place the player can hear about it. -->
      <ul class="aging-notes" role="status" data-testid="aging-notes">
        {#each store.agingNotes as note, i (i)}
          <li data-testid="aging-note-{i}">{noteText(note)}</li>
        {/each}
      </ul>
    {/if}

    {#if store.agingRejections.length > 0}
      <!-- The engine refuses a year already rolled, an under-allocated
           distribution or a revert of a year nothing records. Its findings
           localize through the same `issue-<code>` + resolved-args path the
           validation panel uses, so one wording serves both surfaces. -->
      <ul class="issue-list" data-testid="aging-rejections">
        {#each store.agingRejections as issue, i (i)}
          {@const rawArgs = { ...issue.args, ...(issue.context ? { context: issue.context } : {}) }}
          <li class="issue {issue.severity}" data-severity={issue.severity} data-code={issue.code}>
            {store.t(
              `issue-${issue.code}`,
              store.ruleset ? resolveIssueArgs(store.ruleset, rawArgs, store.t) : rawArgs,
            )}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  /* `.detail-section`, `.field`, `.hint`, `.issue-list` and `.warning` are shared
     globals in app.css. */

  .aging-total {
    font-weight: 600;
  }

  .aging-parts,
  .aging-remaining {
    font-variant-numeric: tabular-nums;
  }

  .aging-notes {
    list-style: none;
    margin: 0.15rem 0;
    padding: 0;
  }

  .aging-outcome p,
  .aging-distribute p {
    margin: 0.15rem 0;
  }

  .aging-distribute-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .aging-distribute-list input {
    width: 4ch;
  }

  .aging-actions {
    display: flex;
    gap: 0.4rem;
    margin-top: 0.3rem;
  }
</style>
