<script lang="ts">
  import { store } from '../state.svelte';
  import { commitStored } from '../actions';
  import { U32_MAX } from '../derive';
  import type { LabYears, LabYearsBucket } from '../types';

  // The post-Gauntlet lab work as four counts of years, one per number of lab
  // seasons a year held (row 56). A year's cost depends only on how many seasons it
  // held, never on when it fell, so the counts are the whole of the choice:
  // "For each season that your magus spends working on a lab project, the character
  // loses 10 points from the yearly 30 experience points, to a minimum of 0 if three
  // or four seasons are spent on lab work."
  // Source: ArMDE:2550
  //
  // Store-agnostic on purpose: the caller hands in the counts and takes the edits,
  // so a later surface can mount one per span of years under its own testid prefix.

  interface Props {
    /** The stored counts; absent means no year held any lab work. */
    value: LabYears | undefined;
    /** Each input's testid is `{testidPrefix}-{seasons}-input`. */
    testidPrefix: string;
    /** Closes every count without taking it out of the tab order. */
    readonly?: boolean;
    /** The id of the note explaining a read-only state, if any. */
    describedBy?: string;
    /** The group's legend; a life section names itself in it. */
    legend?: string;
    /** One count edited; `null` for a blank field. */
    onchange: (bucket: LabYearsBucket, count: number | null) => void;
  }

  let { value, testidPrefix, readonly = false, describedBy, legend, onchange }: Props = $props();

  // The four counts in the order of the seasons they stand for. A fixed taxonomy —
  // a year has four seasons — mirroring the engine's `LabYears` fields.
  const buckets: { seasons: number; bucket: LabYearsBucket }[] = [
    { seasons: 1, bucket: 'one_season' },
    { seasons: 2, bucket: 'two_seasons' },
    { seasons: 3, bucket: 'three_seasons' },
    { seasons: 4, bucket: 'four_seasons' },
  ];

  /** A stored count as an input value: absent reads as an empty field. */
  function fieldValue(bucket: LabYearsBucket): string {
    const count = value?.[bucket];
    return count == null ? '' : String(count);
  }

  /** The number typed into a count, with a blank field meaning `null`. */
  function count(event: Event): number | null {
    const raw = (event.currentTarget as HTMLInputElement).value;
    return raw === '' ? null : Number(raw);
  }
</script>

<fieldset class="lab-years">
  <legend>{legend ?? store.t('life-stage-lab-years-legend')}</legend>
  {#each buckets as { seasons, bucket } (bucket)}
    <label class="field inline">
      <span>{store.t(`life-stage-lab-years-${seasons}-label`)}</span>
      <input
        type="number"
        min="0"
        max={U32_MAX}
        {readonly}
        value={fieldValue(bucket)}
        aria-describedby={readonly ? describedBy : undefined}
        oninput={(event) => onchange(bucket, count(event))}
        use:commitStored={{ read: () => fieldValue(bucket) }}
        data-testid="{testidPrefix}-{seasons}-input"
      />
    </label>
  {/each}
</fieldset>
