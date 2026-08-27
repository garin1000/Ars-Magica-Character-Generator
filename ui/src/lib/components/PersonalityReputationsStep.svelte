<script lang="ts">
  import PersonalityTraits from './PersonalityTraits.svelte';
  import Reputations from './Reputations.svelte';
</script>

<!-- The `personality_reputations` phase is these two sections and nothing more — no
     age, aging, Warping or Twilight surfaces come along. The editor's matching tab
     mounts this same composition (App.svelte), so the step and the tab cannot
     drift (#28). -->
<section class="panel character-details">
  <PersonalityTraits />
  <Reputations />
</section>
