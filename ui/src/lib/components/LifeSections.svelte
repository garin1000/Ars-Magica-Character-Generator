<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import { lifeSectionLabel, U32_MAX, type Translate } from '../derive';
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

  function label(section: LifeSection, index: number): string {
    return lifeSectionLabel(section, index, t);
  }

  /** The number typed into the years field, with a blank field meaning `null`. */
  function count(event: Event): number | null {
    const raw = (event.currentTarget as HTMLInputElement).value;
    return raw === '' ? null : Number(raw);
  }
</script>

<div class="life-sections">
  <ol
    class="life-section-list"
    aria-label={store.t('life-sections-label')}
    data-testid="life-sections-list"
  >
    {#each sections as section, i (i)}
      <li data-testid="life-section-{i}">
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
              oninput={(event) => store.setLifeSectionYears(i, count(event))}
              use:commitStored={{ read: () => String(sections[i]?.years ?? '') }}
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
  <button type="button" onclick={() => store.addLifeSection()} data-testid="life-section-add">
    {store.t('life-section-add')}
  </button>
</div>
