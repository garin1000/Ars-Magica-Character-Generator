<script lang="ts">
  import AgingSchedulePanel from './AgingSchedulePanel.svelte';
  import LivingConditionsPicker from './LivingConditionsPicker.svelte';
  import AgingRollCalculator from './AgingRollCalculator.svelte';
  import AgingRecordPanel from './AgingRecordPanel.svelte';
  import LongevityPanel from './LongevityPanel.svelte';
</script>

<!-- The whole aging surface, in the order a year is actually resolved: what the
     rules owe this character (the schedule and the standing half of the aging
     total), the Living Conditions that are the one term of that total the player
     chooses, the roll itself (the die, the total it makes, and applying it), then
     what aging has already done to the character (apparent age, Decrepitude,
     points, log), and finally the Longevity Ritual, whose bonus is subtracted from
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

     `display: contents` inside `.character-details` (see app.css) keeps each block
     its own item in that multi-column flow. -->
<div class="aging-panel" data-testid="aging-panel">
  <AgingSchedulePanel />
  <LivingConditionsPicker />
  <AgingRollCalculator />
  <AgingRecordPanel />
  <LongevityPanel />
</div>
