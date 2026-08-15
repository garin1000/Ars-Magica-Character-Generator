<script lang="ts">
  import { store } from '../state.svelte';
  import ValidationPanel from './ValidationPanel.svelte';

  const clean = $derived((store.result?.issues ?? []).length === 0);
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

  <p class="hint" data-testid="wizard-review-incomplete">
    {store.t('wizard-review-incomplete')}
  </p>
  <p class="hint" data-testid="wizard-review-hint">{store.t('wizard-review-hint')}</p>
</section>
