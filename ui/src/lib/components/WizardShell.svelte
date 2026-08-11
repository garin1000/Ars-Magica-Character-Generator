<script lang="ts">
  import { store } from '../state.svelte';
  import { phaseHasBlockingIssue } from '../derive';
  import ValidationPanel from './ValidationPanel.svelte';
  import WizardStep from './WizardStep.svelte';

  const phases = $derived(store.wizardPhases);
  const onLastStep = $derived(store.wizardStep === phases.length - 1);

  // In Advisory the engine downgrades every error to a warning, and in Silent it
  // reports none, so in both the wizard cannot gate at all. Say so rather than
  // letting the absent gate look like an approval.
  const enforced = $derived(store.mode === 'enforced');

  const blockedHintId = 'wizard-blocked-hint';

  function blocked(phase: (typeof phases)[number]): boolean {
    return phaseHasBlockingIssue(store.result?.issues ?? [], phase);
  }
</script>

<!-- The guided flow: a rail of the character type's creation phases, one step's
     input surface, that step's own findings, and the navigation. Back is always
     available; forward is Next (gated on this step's errors) or a rail click to an
     already-visited step, which the store clamps at the first blocking phase in
     between. The closing Review step swaps Next for Finish. -->
<div class="wizard" data-testid="wizard">
  <nav class="wizard-rail" aria-label={store.t('wizard-rail-label')} data-testid="wizard-rail">
    <ol>
      {#each phases as phase, i (phase)}
        <li>
          <button
            type="button"
            class="wizard-rail-step"
            class:active={i === store.wizardStep}
            aria-current={i === store.wizardStep ? 'step' : undefined}
            aria-describedby={blocked(phase) ? blockedHintId : undefined}
            data-blocked={blocked(phase) ? 'true' : undefined}
            disabled={i > store.wizardFurthest}
            onclick={() => store.wizardGoTo(i)}
            data-testid="wizard-step-{phase}"
          >
            {store.t(`phase-${phase}`)}
          </button>
        </li>
      {/each}
    </ol>
  </nav>

  <!-- Test id deliberately not `wizard-step-*`: that prefix names the rail's
       steps, and this is the count, not one of them. -->
  <p class="wizard-progress" data-testid="wizard-progress">
    {store.t('wizard-step-progress', {
      current: String(store.wizardStep + 1),
      total: String(phases.length),
    })}
  </p>

  <main class="tab-content wizard-body">
    <WizardStep phase={store.wizardPhase} />
  </main>

  <footer class="validation-bar">
    <!-- Filtered to this step, except on the closing step, which is the only place
         a finding no creation phase owns can be seen. -->
    <ValidationPanel docked phase={onLastStep ? undefined : store.wizardPhase} />
  </footer>

  <nav class="wizard-nav">
    <button
      type="button"
      disabled={store.wizardStep === 0}
      onclick={() => store.wizardBack()}
      data-testid="wizard-back"
    >
      {store.t('wizard-back')}
    </button>

    {#if !enforced}
      <p class="hint" data-testid="wizard-unchecked-hint">{store.t('wizard-unchecked-hint')}</p>
    {:else if !store.wizardCanAdvance}
      <p class="hint" id={blockedHintId} data-testid="wizard-blocked-hint">
        {store.t('wizard-blocked-hint')}
      </p>
    {/if}

    {#if onLastStep}
      <button
        type="button"
        class="primary"
        disabled={!store.wizardCanFinish}
        onclick={() => store.finishWizard()}
        data-testid="wizard-finish"
      >
        {store.t('wizard-finish')}
      </button>
    {:else}
      <button
        type="button"
        class="primary"
        disabled={!store.wizardCanAdvance}
        onclick={() => store.wizardNext()}
        data-testid="wizard-next"
      >
        {store.t('wizard-next')}
      </button>
    {/if}
  </nav>
</div>
