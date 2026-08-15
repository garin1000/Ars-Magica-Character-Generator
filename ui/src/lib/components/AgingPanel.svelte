<script lang="ts">
  import AgingSchedulePanel from './AgingSchedulePanel.svelte';
  import LivingConditionsPicker from './LivingConditionsPicker.svelte';
  import AgingRollCalculator from './AgingRollCalculator.svelte';
  import AgingRecordPanel from './AgingRecordPanel.svelte';
</script>

<!-- The whole aging surface, in the order a year is actually resolved: what the
     rules owe this character (the schedule and the standing half of the aging
     total), the Living Conditions that are the one term of that total the player
     chooses, the roll itself (the die, the total it makes, and applying it), and
     then what aging has already done to the character (apparent age, Decrepitude,
     points, log). Composed once so the Details tab and the guided aging step
     mount the same thing and can never drift.
     `display: contents` inside `.character-details` (see app.css) keeps each block
     its own item in that multi-column flow. -->
<div class="aging-panel" data-testid="aging-panel">
  <AgingSchedulePanel />
  <LivingConditionsPicker />
  <AgingRollCalculator />
  <AgingRecordPanel />
</div>
