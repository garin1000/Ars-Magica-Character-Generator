<script lang="ts">
  import AgingSchedulePanel from './AgingSchedulePanel.svelte';
  import AgingRecordPanel from './AgingRecordPanel.svelte';
</script>

<!-- The whole aging surface: what the rules owe this character (the schedule and
     the standing half of the aging total) above what aging has already done to it
     (apparent age, Decrepitude, points, log). Composed once so the Details tab and
     the guided aging step mount the same thing and can never drift.
     `display: contents` inside `.character-details` (see app.css) keeps each block
     its own item in that multi-column flow. -->
<div class="aging-panel" data-testid="aging-panel">
  <AgingSchedulePanel />
  <AgingRecordPanel />
</div>
