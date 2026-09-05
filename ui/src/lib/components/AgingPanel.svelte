<script lang="ts">
  import { store } from '../state.svelte';
  import AgingSchedulePanel from './AgingSchedulePanel.svelte';
  import LivingConditionsPicker from './LivingConditionsPicker.svelte';
  import AgingRollCalculator from './AgingRollCalculator.svelte';
  import AgingRecordPanel from './AgingRecordPanel.svelte';
  import LongevityPanel from './LongevityPanel.svelte';

  // WHY THE WRAPPERS ARE GATED (#33). Each wrapper below is one grid item, so a
  // wrapper that renders while all of its blocks are `{#if}`-ed away is a whole
  // EMPTY TRACK — a column-wide hole between the two that did render. The character
  // who trips it is the ordinary one: a starting character too young to owe an aging
  // roll has an empty schedule, so `AgingRollCalculator` renders nothing and the
  // middle column would be that hole.
  //
  // The three conditions restate each block's own mount gate, and deliberately so:
  // the alternative is a wrapper that cannot know whether it holds anything (CSS
  // `:empty` does not see through Svelte's block anchors, and a
  // `display: contents`-when-empty trick would put the blocks back into the outer
  // grid, which is the coupling #33 exists to remove). They are kept honest by
  // `AgingPanel.test.ts`, which renders exactly those characters.
  const aging = $derived(store.effective?.aging ?? null);
  // `AgingSchedulePanel`: `{#if aging && formula}`, and its `formula` is non-null
  // whenever `aging` is.
  const hasSchedule = $derived(aging != null);
  // `LivingConditionsPicker`: `{#if rows.length > 0}`, its rows being the ruleset's
  // own Living Conditions catalogue.
  const hasConditions = $derived(
    (store.ruleset?.ruleset.aging?.living_conditions ?? []).length > 0,
  );
  // `AgingRollCalculator`: `{#if aging && schedule.length > 0}`.
  const hasRoll = $derived(aging != null && aging.schedule.length > 0);
</script>

<!-- The whole aging surface, in the order a year is actually resolved: what the
     rules owe this character (the schedule and the standing half of the aging
     total), the Living Conditions that are the one term of that total the player
     chooses, the roll itself (the die, the total it makes, and applying it), then
     what aging has already done to the character — the per-year log first, since
     that is what the roll has just written to and where a year is taken back, then
     the running totals it explains (apparent age, Decrepitude, points) — and finally
     the Longevity Ritual, whose bonus is subtracted from
     every aging total this character will ever roll
     (Ars Magica - Definitive Edition (Core Rules).md:16567-16569). Composed once so
     the editor's Aging tab and the guided aging step mount the same thing and can
     never drift.

     The ritual belongs here and used to be mounted twice outside instead — by
     `AgingStep` and by `MagicPossessions` — only because the editor had no aging
     tab: inside this panel it would have appeared on the editor's Details tab as
     well, giving a magus two on-screen homes for one ritual. The Aging tab (#28)
     removes that constraint. The panel needs no gate of its own: it reads
     `entity.longevity_ritual` directly, and only the Creo Corpus *suggestion* is
     magus-only (derived.rs), which is right, because "You can perform Longevity
     Rituals for others, even for non-magi" (Core Rules.md:10672).

     THREE COLUMNS, AS WRAPPERS (manual-testing-findings-2026-09-03 #33). This
     wrapper is `display: contents` inside `.character-details` (app.css), so the
     three `.aging-column` divs below — and the heading — are the items of that
     `auto-fit` grid, not this div.

     The history matters, because two arrangements were tried and reverted and
     neither may come back:

      * CSS MULTI-COLUMN (`columns: 2`), which guided-creation-review-2026-08 #20
        replaced with this grid: multi-column FLOWS content between columns, so any
        height change moved the break — ticking one Living Condition relaid the whole
        panel and the record block sat permanently split across it.
        `app.css.test.ts` bans `columns:` outright.
      * ONE BLOCK PER FULL-WIDTH ROW, which #22 introduced and which removed the
        columns rather than the gap. It was chosen because a grid row is as tall as
        its tallest item, so the short schedule auto-placed beside the tall roll
        calculator left the calculator's height as blank space under it.

     Wrapping is what dissolves that dilemma. Nothing flows between wrappers, so the
     multi-column failure cannot recur; and no two aging blocks are siblings in the
     grid any more, so the row-height coupling has nothing to couple — a wrapper is
     an independent block container whose height is its own content's. The tall block
     is given a column to itself for the same reason.

     Column order is source order, and source order is unchanged: the surface still
     reads schedule → conditions → roll → log → totals → ritual, so DOM order is the
     order on screen and the order the keyboard walks (the grid places items in
     source order, left to right and then down; nothing is moved by CSS).

     The panel degrades with the window: `auto-fit` gives two tracks and then one with
     no breakpoint, and at one track the three wrappers simply stack in that same
     order. -->
<div class="aging-panel" data-testid="aging-panel">
  <!-- S17 (full-audit a11y): the tab's own <h2> — every block below opened at
       <h3> directly under the app's single <h1> with no <h2> between (a
       heading hierarchy gap) until this was added. Reuses `tab-aging`, the
       same label App.svelte's tab button already carries. Fixing it HERE
       (rather than in each child) covers both the editor's Aging tab and the
       guided aging step in one place, per this file's own no-drift design.
       `.character-details-heading` (app.css) spans it across every grid
       column of the ancestor `.character-details` section — this `<div>` is
       `display: contents` (app.css), which promotes the h2 straight into
       that grid, or auto-placement would squeeze it into one cell like any
       other block here. -->
  <h2 class="character-details-heading">{store.t('tab-aging')}</h2>

  <!-- What the rules owe this character, and the one term of the aging total the
       player chooses. Two short blocks, so they share a column. -->
  {#if hasSchedule || hasConditions}
    <div class="aging-column" data-testid="aging-column-schedule">
      <AgingSchedulePanel />
      <LivingConditionsPicker />
    </div>
  {/if}

  <!-- The roll itself — the die, the total it makes, the outcome and Apply — alone
       in its column BECAUSE it is the tall one. Under #22's diagnosis a tall block
       is only a problem when something short shares its row; give it a column and
       there is nothing to share with. -->
  {#if hasRoll}
    <div class="aging-column" data-testid="aging-column-roll">
      <AgingRollCalculator />
    </div>
  {/if}

  <!-- What aging has already done, and the ritual that reduces every future total.
       The log LEADS, so it sits at the top of a column and is visible without
       scrolling — the whole of #24/#25 — with the running totals it explains under
       it and the ritual last. `AgingRecordPanel`'s own `display: contents` (app.css)
       drops its two blocks straight into this wrapper, so both land here. -->
  <div class="aging-column" data-testid="aging-column-record">
    <AgingRecordPanel />
    <LongevityPanel />
  </div>
</div>
