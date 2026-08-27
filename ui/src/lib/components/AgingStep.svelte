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

     `.character-details` is a CSS multi-column flow in which source order governs
     placement (see app.css), so the age stays first and the aging cluster
     contiguous. -->
<section class="panel character-details" data-testid="aging-step">
  <AgeFields />
  <AgingPanel />
</section>
