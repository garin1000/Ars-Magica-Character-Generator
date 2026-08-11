<script lang="ts">
  import PersonalityTraits from './PersonalityTraits.svelte';
  import Reputations from './Reputations.svelte';
</script>

<!-- The `personality_reputations` phase is two sections of the editor's details
     panel, so the step is their composition and nothing more — no age, aging,
     Warping or Twilight surfaces come along. -->
<section class="panel character-details">
  <PersonalityTraits />
  <Reputations />
</section>
