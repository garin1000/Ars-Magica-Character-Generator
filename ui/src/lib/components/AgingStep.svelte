<script lang="ts">
  import AgeFields from './AgeFields.svelte';
  import AgingPanel from './AgingPanel.svelte';
</script>

<!-- The `aging` phase: the age the whole schedule hangs on, above the aging
     surface itself (what the rules owe this character, what aging has already done
     to it, and the Longevity Ritual its totals are reduced by). A composition and
     nothing more — every input here is the very one the editor's Aging tab mounts,
     so the two flows cannot drift.

     `AgeFields` is what makes the step usable under flat (pool) funding, where no
     other wizard surface offers an age at all. In the editor the age stays on the
     Details tab, beside the identity it belongs with, so it is the one thing this
     step adds around `AgingPanel`.

     `.character-details` is an auto-placed CSS grid (see app.css) in which source
     order is placement order, so the age keeps the first cell — read first because
     it is what the whole schedule below it hangs on. Under grid each block owns its
     cell, so nothing here moves when one of them changes height. -->
<section class="panel character-details" data-testid="aging-step">
  <AgeFields />
  <AgingPanel />
</section>
