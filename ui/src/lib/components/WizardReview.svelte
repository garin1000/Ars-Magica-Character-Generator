<script lang="ts">
  import { store } from '../state.svelte';
  import ValidationPanel from './ValidationPanel.svelte';

  const clean = $derived((store.result?.issues ?? []).length === 0);
  const outstanding = $derived(store.wizardIncompletePhases);
</script>

<!-- The wizard's closing step. Its validation panel is deliberately UNFILTERED:
     this is the only place in the flow where a finding no creation phase owns
     (equipment, Might, Warping) — or one belonging to a phase this
     character type never declares — can be seen at all. Finish gates on the same
     whole-character view. -->
<section class="panel wizard-review">
  <h3 class="detail-label">{store.t('wizard-review-title')}</h3>

  {#if clean}
    <p data-testid="wizard-review-clean">{store.t('wizard-review-clean')}</p>
  {:else}
    <ValidationPanel />
  {/if}

  <!-- Legal is not finished. The steps gate on errors only, so anything merely
       left empty walked through — this names which ones, without asking for any
       of them: Finish is not held on this list. -->
  {#if outstanding.length > 0}
    <p class="hint" data-testid="wizard-review-incomplete">
      {store.t('wizard-review-incomplete')}
    </p>
    <ul class="wizard-review-outstanding">
      {#each outstanding as phase (phase)}
        <li data-testid="wizard-review-incomplete-{phase}">{store.t(`phase-${phase}`)}</li>
      {/each}
    </ul>
  {:else}
    <p class="hint" data-testid="wizard-review-complete">{store.t('wizard-review-complete')}</p>
  {/if}

  <p class="hint" data-testid="wizard-review-hint">{store.t('wizard-review-hint')}</p>
</section>
