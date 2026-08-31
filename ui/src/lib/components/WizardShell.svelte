<script lang="ts">
  import { store } from '../state.svelte';
  import { phaseHasBlockingIssue, phaseIsIncomplete } from '../derive';
  import ValidationPanel from './ValidationPanel.svelte';
  import WizardStep from './WizardStep.svelte';

  const phases = $derived(store.wizardPhases);
  const onLastStep = $derived(store.wizardStep === phases.length - 1);

  // In Advisory the engine downgrades every error to a warning, and in Silent it
  // reports none, so in both the wizard cannot gate at all. Say so rather than
  // letting the absent gate look like an approval.
  const enforced = $derived(store.mode === 'enforced');

  const blockedHintId = 'wizard-blocked-hint';

  // S22 (full-audit a11y): a rail step's `aria-describedby` used to reference
  // the SHARED `blockedHintId` above, but the element that id belongs to only
  // renders for the CURRENT step's own block reason (the `wizard-nav` footer,
  // below). A step other than the current one — blocked while the current step
  // is not — pointed at an id absent from the document: a dangling reference.
  // Each step now gets its own self-contained id and `.sr-only` hint text,
  // mirroring the per-step incomplete marker just below it in the rail.
  function blockedHintIdFor(phase: (typeof phases)[number]): string {
    return `wizard-blocked-hint-${phase}`;
  }

  function blocked(phase: (typeof phases)[number]): boolean {
    return phaseHasBlockingIssue(store.result?.issues ?? [], phase);
  }

  // A step the player has recorded nothing for. Deliberately unrelated to
  // `blocked`: the gate is about what the rules forbid, this is about what is
  // still empty, and an empty step is marked but never held shut.
  function incomplete(phase: (typeof phases)[number]): boolean {
    return phaseIsIncomplete(store.result, phase);
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
            aria-describedby={blocked(phase) ? blockedHintIdFor(phase) : undefined}
            data-blocked={blocked(phase) ? 'true' : undefined}
            data-incomplete={incomplete(phase) ? 'true' : undefined}
            disabled={i > store.wizardFurthest}
            onclick={() => store.wizardGoTo(i)}
            data-testid="wizard-step-{phase}"
          >
            {store.t(`phase-${phase}`)}
            <!-- Inside the button, so the marker is part of its accessible name:
                 `data-incomplete` alone would be styling only. The test id avoids
                 the `wizard-step-` prefix, which names the rail's steps.

                 SCREEN-READER-ONLY (guided-creation-review-2026-08 #10): the flag is
                 the engine's `completeness.incomplete_phases`, not "step not opened
                 yet", so on a fresh character EVERY step carried the words at once —
                 noise on the surface that has to stay scannable. `.sr-only` is the
                 right utility rather than `.hidden-reserved`: this marker must still
                 be ANNOUNCED and must take no space, the opposite trade to the
                 on-step hint below. The Fluent key is kept, not deleted. -->
            {#if incomplete(phase)}
              <span class="sr-only" data-testid="wizard-incomplete-{phase}">
                {store.t('wizard-step-incomplete-label')}
              </span>
            {/if}
            {#if blocked(phase)}
              <!-- Self-contained: this id is the one `aria-describedby` above
                   references, so it can never dangle regardless of which step is
                   current. Same text as the nav footer's own blocked hint. -->
              <span
                class="sr-only"
                id={blockedHintIdFor(phase)}
                data-testid="wizard-blocked-hint-{phase}"
              >
                {store.t('wizard-blocked-hint')}
              </span>
            {/if}
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

  <!-- Said on the step as well as in the rail, because the rail's mark is easy to
       miss on the step you are standing on. It is a statement, never a gate: Next
       stays exactly as enabled as the findings leave it.

       ALWAYS MOUNTED, hidden by visibility alone (guided-creation-review-2026-08 #2,
       an instance of cross-cutting theme 1). The flag is the engine's
       `completeness.incomplete_phases`, which flips on the very FIRST recorded value
       — so mounting this paragraph conditionally guaranteed that the first `+` click
       collapsed a line directly above the step's input surface and moved the control
       being clicked, worst on the characteristics step where clicks repeat. Keeping
       the box reserves its space, so the surface below never moves. `aria-hidden`
       rides along with the hidden state because the sentence is FALSE once something
       is recorded: it must not be read out either, and the rail's own per-step marker
       is what remains in the accessibility tree. -->
  <p
    class="hint wizard-incomplete-hint"
    class:hidden-reserved={!store.wizardPhaseIncomplete}
    aria-hidden={store.wizardPhaseIncomplete ? undefined : 'true'}
    data-testid="wizard-incomplete-hint"
  >
    {store.t('wizard-step-incomplete-hint')}
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
