<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import {
    invalidLifeSectionIndices,
    lifeSectionInvalidText,
    lifeSectionLabel,
    U32_MAX,
    type Translate,
  } from '../derive';
  import type { LifeSection } from '../types';
  import LabYearsFields from './LabYearsFields.svelte';

  // The player's named spans of the character's life, oldest first: one row each
  // (name, years, move up/down, remove), then an add button. For a magus the
  // sections fall after the Gauntlet, so each carries its own lab years, and the
  // plan-level counts become the "Remaining" row's — the years outside every section.

  interface Props {
    /** Whether each section (and Remaining) carries lab years — a magus's post-Gauntlet span. */
    labYears: boolean;
    /** Closes every lab-year count without taking it out of the tab order. */
    labYearsReadonly?: boolean;
    /** The id of the note explaining a read-only state, if any. */
    labYearsDescribedBy?: string;
  }

  let { labYears, labYearsReadonly = false, labYearsDescribedBy }: Props = $props();

  const plan = $derived(store.entity.life_stages ?? null);
  const sections = $derived(plan?.life_sections ?? []);
  const t: Translate = (key, args) => store.t(key, args);

  // manual-testing-findings #9: the Remaining row holds the post-Gauntlet years
  // no named section claims. Once the sections use up the whole span there is
  // nothing left to show it for. `null` means "the engine has not sent a budget
  // yet", not "no years" — same convention `LifeStagePanel`'s no-years gate
  // uses — so the row stays up optimistically until the engine says otherwise.
  const postGauntletYears = $derived(store.effective?.life_stage?.post_gauntlet_years ?? null);
  const claimedYears = $derived(sections.reduce((sum, s) => sum + (s.years ?? 0), 0));
  const remainingYears = $derived(
    postGauntletYears == null ? null : Math.max(0, postGauntletYears - claimedYears),
  );
  const showRemaining = $derived(labYears && (remainingYears == null || remainingYears >= 1));

  // F15 (manual-testing-findings-2026-11-15 #15): a section whose lab years
  // exceed its own years is marked in the row itself, like AbilityTab's
  // `.invalid-selection` rows — not only the issues footer.
  const invalidSections = $derived(invalidLifeSectionIndices(store.result));

  function label(section: LifeSection, index: number): string {
    return lifeSectionLabel(section, index, t);
  }

  // manual-testing-findings-2026-11-15 #9 (F9): `setLifeSectionYears` reads a blank
  // field as 0 (the field is "required" — there is no stored `null`), so an empty
  // read must never be forwarded while typing: forwarding it immediately wrote 0,
  // and the plain `value={String(section.years)}` binding snapped the field back
  // to "0" before the player could type a replacement. The birth-year pattern's "or
  // nothing" clause (`actions.ts::commitStored`'s own doc) is for exactly this case.
  function onYearsInput(index: number, event: Event) {
    const raw = (event.currentTarget as HTMLInputElement).value;
    if (raw === '') return;
    const typed = Number(raw);
    if (!Number.isFinite(typed) || typed < 0) return;
    store.setLifeSectionYears(index, typed);
  }

  // Commit (blur or Enter): the required fallback-to-0 still applies, exactly as
  // `onYearsInput` would have, had the field never been left blank.
  function commitYears(index: number, raw: string) {
    store.setLifeSectionYears(index, raw === '' ? null : Number(raw));
  }
</script>

<div class="life-sections">
  <ol
    class="life-section-list"
    aria-label={store.t('life-sections-label')}
    data-testid="life-sections-list"
  >
    {#each sections as section, i (i)}
      <li data-testid="life-section-{i}" class:invalid-selection={invalidSections.has(i)}>
        {#if invalidSections.has(i)}
          <!-- WCAG 1.4.1: the red tint on `.invalid-selection` is not the only
               signal — a visible glyph plus words for a screen reader, matching
               AbilityTab's own invalid-row marker. -->
          <span class="invalid-glyph" aria-hidden="true">!</span>
          <span class="sr-only">{lifeSectionInvalidText(store.result, i, t)}</span>
        {/if}
        <fieldset class="life-section">
          <legend class="sr-only">{label(section, i)}</legend>
          <label class="field inline">
            <span>{store.t('life-section-name-label')}</span>
            <input
              type="text"
              placeholder={store.t('life-section-unnamed', { n: String(i + 1) })}
              value={section.name}
              oninput={(event) =>
                store.setLifeSectionName(i, (event.currentTarget as HTMLInputElement).value)}
              data-testid="life-section-{i}-name-input"
            />
          </label>
          <label class="field inline">
            <span>{store.t('life-section-years-label')}</span>
            <input
              type="number"
              min="0"
              max={U32_MAX}
              value={String(section.years)}
              oninput={(event) => onYearsInput(i, event)}
              use:commitStored={{
                read: () => String(sections[i]?.years ?? ''),
                commit: (raw) => commitYears(i, raw),
              }}
              data-testid="life-section-{i}-years-input"
            />
          </label>
          <button
            type="button"
            class="icon-btn"
            disabled={i === 0}
            aria-label={store.t('life-section-move-up', { section: label(section, i) })}
            onclick={() => store.moveLifeSection(i, 'up')}
            data-testid="life-section-{i}-up"
          >
            {store.t('life-section-move-up-glyph')}
          </button>
          <button
            type="button"
            class="icon-btn"
            disabled={i === sections.length - 1}
            aria-label={store.t('life-section-move-down', { section: label(section, i) })}
            onclick={() => store.moveLifeSection(i, 'down')}
            data-testid="life-section-{i}-down"
          >
            {store.t('life-section-move-down-glyph')}
          </button>
          <button
            type="button"
            class="icon-btn"
            aria-label={store.t('remove-item', { name: label(section, i) })}
            onclick={() => store.removeLifeSectionAt(i)}
            data-testid="life-section-{i}-delete"
          >
            ×
          </button>
          {#if labYears}
            <LabYearsFields
              value={section.lab_years}
              testidPrefix="life-section-{i}-lab-years"
              legend={store.t('life-section-lab-years-legend', { section: label(section, i) })}
              readonly={labYearsReadonly}
              describedBy={labYearsDescribedBy}
              onchange={(bucket, years) => store.setLifeSectionLabYears(i, bucket, years)}
            />
          {/if}
        </fieldset>
      </li>
    {/each}
  </ol>
  <button type="button" onclick={() => store.addLifeSection()} data-testid="life-section-add">
    {store.t('life-section-add')}
  </button>
  <!-- manual-testing-findings #6: Remaining comes AFTER Add section, not before —
       it is the last line of the whole surface. -->
  {#if showRemaining}
    <div class="life-section-remaining" data-testid="life-section-remaining">
      <LabYearsFields
        value={plan?.post_gauntlet_lab_years}
        testidPrefix="life-stage-lab-years"
        legend={store.t('life-section-lab-years-legend', {
          section: store.t('life-section-remaining'),
        })}
        readonly={labYearsReadonly}
        describedBy={labYearsDescribedBy}
        onchange={(bucket, years) => store.setPostGauntletLabYears(bucket, years)}
      />
    </div>
  {/if}
</div>
