<script lang="ts">
  import AgeFields from './AgeFields.svelte';
  import IdentityFields from './IdentityFields.svelte';
  import SagaYearField from './SagaYearField.svelte';
</script>

<!-- The `concept` phase: who the character is, and when.
     A composition, like `ExperienceStep`, `AgingStep` and
     `PersonalityReputationsStep` — the wizard's step table declares one component
     per phase, so a step that needs two surfaces needs a component that mounts both.

     `IdentityFields` then `AgeFields`, in that order, mirroring the editor's Details
     tab (`CharacterDetails`) verbatim: guided-creation-review-2026-08 #24 gave the
     age ONE canonical home, and this is it. Before Slice 12 the wizard's only age
     field lived on the aging step, three phases past the birth year it belongs
     beside — and a second one appeared in the life-stage panel under guided funding.

     `SagaYearField` last: it is the year the two above are measured against, and it
     is app state rather than part of the character, so it reads after the character's
     own fields rather than in among them. It is deliberately NOT on the editor's
     Details tab — the editor's layout is unchanged by this slice. -->
<section class="panel character-details" data-testid="concept-step">
  <IdentityFields />
  <AgeFields />
  <SagaYearField />
</section>
