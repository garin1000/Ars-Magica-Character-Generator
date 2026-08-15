<script lang="ts">
  import AgeFields from './AgeFields.svelte';
  import AgingPanel from './AgingPanel.svelte';
  import LongevityPanel from './LongevityPanel.svelte';
</script>

<!-- The `aging` phase: the age the whole schedule hangs on, above the aging
     surface itself (what the rules owe this character, and what aging has already
     done to it). A composition and nothing more — every input here is the very
     one the editor's Details tab mounts, so the two flows cannot drift.

     `AgeFields` is what makes the step usable under flat (pool) funding, where no
     other wizard surface offers an age at all.

     The Longevity Ritual joins them here, and ONLY here. Its bonus is a term of
     the AGING TOTAL (Core Rules.md:16567-16569), but its editor home is the
     Possessions tab, which is magus-gated (App.svelte) and which no
     `CreationPhase` maps to — so this is the only guided surface that can reach
     it. It is mounted on the step rather than inside `AgingPanel` on purpose:
     inside, it would render on the editor's Details tab as well, giving a magus
     two on-screen homes for one ritual. The panel needs no gate of its own — it
     reads `entity.longevity_ritual` directly, and only the Creo Corpus
     *suggestion* is magus-only (derived.rs), which is exactly right, because
     "You can perform Longevity Rituals for others, even for non-magi"
     (Core Rules.md:10672).

     `.character-details` is a CSS multi-column flow in which source order governs
     placement (see app.css), so the age stays first and the aging cluster
     contiguous. -->
<section class="panel character-details" data-testid="aging-step">
  <AgeFields />
  <AgingPanel />
  <LongevityPanel />
</section>
