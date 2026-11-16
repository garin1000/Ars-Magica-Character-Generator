<script lang="ts">
  import AgingPanel from './AgingPanel.svelte';
</script>

<!-- The `aging` phase: the aging surface itself — what the rules owe this character,
     what aging has already done to it, and the Longevity Ritual its totals are
     reduced by. A composition and nothing more; every input here is the very one the
     editor's Aging tab mounts, so the two flows cannot drift.

     It used to mount `AgeFields` above the panel, because under flat (pool) funding
     no other wizard surface offered an age at all. Slice 12 (#24) gave the age one
     canonical home instead: the Concept step, beside the identity and the birth year
     it is linked to, under BOTH funding models — which is what made this copy
     removable rather than merely redundant. The age the schedule below hangs on is
     therefore entered three steps earlier, and read from the same `entity.age`.

     `.character-details` is an auto-placed CSS grid (see app.css) in which source
     order is placement order. Under grid each block owns its cell, so nothing here
     moves when one of them changes height. No "panel" class (#21, F21): the aging
     surface paints its background per column instead of behind the whole grid, and
     "aging-grid" forces exactly three equal tracks — see AgingPanel.svelte's own
     doc comment. -->
<section class="character-details aging-grid" data-testid="aging-step">
  <AgingPanel />
</section>
