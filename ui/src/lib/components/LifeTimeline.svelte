<script lang="ts">
  import { store } from '../state.svelte';
  import { displayName, lifeSectionLabel, type Translate } from '../derive';
  import type { TimelineEntry, TimelineEntryKind } from '../types';

  // The character's life as the engine lays it out (`EffectiveScores.life_timeline`):
  // one item per span, oldest first, each stating its ages and its filled / earned
  // figures in words. The fill bar is decoration only. Every number is the engine's.

  const timeline = $derived(store.effective?.life_timeline ?? null);
  // manual-testing-findings #9: the engine always drafts a Remaining span, even
  // with no years left in it (`life_timeline.rs::open_span_drafts`), so a
  // character whose sections use up the whole span would otherwise show an
  // empty "Remaining (ages N-N): 0 / 0 XP" entry. Drop it once it holds nothing.
  const visibleEntries = $derived(
    (timeline?.entries ?? []).filter((e) => !(e.kind.kind === 'remaining' && e.years < 1)),
  );
  // The chips name their Virtue or Flaw from the catalogue, never by its id.
  const ruleset = $derived(store.ruleset);
  const sections = $derived(store.entity.life_stages?.life_sections ?? []);
  const t: Translate = (key, args) => store.t(key, args);

  function spanLabel(kind: TimelineEntryKind): string {
    switch (kind.kind) {
      case 'fixed':
        return store.t(`life-section-fixed-${kind.section}`);
      case 'named':
        return lifeSectionLabel(sections[kind.index], kind.index, t);
      case 'remaining':
        return store.t('life-section-remaining');
    }
  }

  /** The span in words: label, ages, and what it filled of what it earned. */
  function describe(entry: TimelineEntry): string {
    const span = {
      section: spanLabel(entry.kind),
      from: String(entry.start_age),
      to: String(entry.end_age),
      filled: String(entry.filled_xp),
      levelsFilled: String(entry.filled_spell_levels),
    };
    const capacity = entry.capacity;
    if (capacity.kind === 'shared') {
      return store.t('life-timeline-entry-points', { ...span, points: String(capacity.points) });
    }
    if (capacity.spell_levels > 0) {
      return store.t('life-timeline-entry-xp-spells', {
        ...span,
        xp: String(capacity.xp),
        levels: String(capacity.spell_levels),
      });
    }
    return store.t('life-timeline-entry-xp', { ...span, xp: String(capacity.xp) });
  }

  /**
   * How full the span is, as a percentage of what it earned (0 for an empty
   * span). A Shared span's XP and spell levels are one traded budget
   * (ArMDE:2541), so both count. A Separate span's are NOT — apprenticeship's
   * "240 experience points, and 120 levels of spells" (ArMDE:2505) are two
   * budgets that cannot be traded, so the bar reads the XP figure alone; the
   * spell levels already appear as their own count in `describe()`'s words
   * (manual-testing-findings-2026-11-15 #17).
   */
  function fillPercent(entry: TimelineEntry): number {
    const capacity = entry.capacity;
    if (capacity.kind === 'shared') {
      if (capacity.points === 0) return 0;
      const filled = entry.filled_xp + entry.filled_spell_levels;
      return Math.min(100, Math.round((filled / capacity.points) * 100));
    }
    if (capacity.xp === 0) return 0;
    return Math.min(100, Math.round((entry.filled_xp / capacity.xp) * 100));
  }
</script>

{#if timeline}
  <div class="life-timeline-block">
    <ol
      class="life-timeline"
      aria-label={store.t('life-timeline-label')}
      data-testid="life-timeline"
    >
      {#each visibleEntries as entry, i (i)}
        <li
          class="life-timeline-entry"
          style="flex-grow: {Math.max(entry.years, 1)}"
          data-testid="life-timeline-entry-{i}"
        >
          <span class="life-timeline-bar" aria-hidden="true"
            ><span
              class="life-timeline-fill"
              aria-hidden="true"
              style="width: {fillPercent(entry)}%"
            ></span></span
          >
          <span class="life-timeline-text">{describe(entry)}</span>
        </li>
      {/each}
    </ol>
    {#if timeline.overrun_years > 0}
      <span
        class="life-timeline-overrun error-text"
        role="status"
        data-testid="life-timeline-overrun"
      >
        {store.t('life-timeline-overrun', { years: String(timeline.overrun_years) })}
      </span>
    {/if}
    {#if timeline.outside.length > 0 && ruleset}
      <!-- F23: its own labelled row, visually apart from the stage entries above —
           previously only an invisible aria-label marked this list, so the chips
           read as trailing text under Early childhood. -->
      <div class="life-timeline-outside-block" data-testid="life-timeline-outside-block">
        <span class="life-timeline-outside-heading" data-testid="life-timeline-outside-heading">
          {store.t('life-timeline-outside-heading')}
        </span>
        <ul
          class="life-timeline-outside"
          aria-label={store.t('life-timeline-outside-label')}
          data-testid="life-timeline-outside"
        >
          {#each timeline.outside as lump, i (i)}
            <li class="xp-restricted" data-testid="life-timeline-outside-{i}">
              {store.t('life-timeline-outside-chip', {
                item: displayName(ruleset, lump.item),
                used: String(lump.used),
                amount: String(lump.amount),
              })}
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>
{/if}
