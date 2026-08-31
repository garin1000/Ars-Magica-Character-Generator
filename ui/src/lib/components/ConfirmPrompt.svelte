<script lang="ts">
  import { store } from '../state.svelte';

  // Generic, reusable confirmation dialog for a single destructive action
  // (S18/S30: familiar/talisman removal used to fire on one click, with no
  // confirmation and no undo). Mirrors DiscardPrompt.svelte's modal shape and
  // behavior — role=alertdialog, initial focus moved to the safe Cancel
  // control, Escape always cancels and never confirms — but is parameterized
  // by props instead of one dedicated app-wide store flag, so independent
  // instances can coexist: FamiliarPanel's and TalismanPanel's own
  // remove-confirmations are both mounted at once on the Magic Possessions
  // tab. DiscardPrompt itself is left untouched rather than refactored onto
  // this component: its exact `data-testid`s (`discard-prompt`,
  // `discard-cancel`, `discard-confirm`) are depended on across a wide e2e
  // surface (`e2e/helpers.js` and every wizard-walk spec), so reusing its
  // *pattern* here — rather than its literal markup — avoids putting that
  // surface at risk for an unrelated fix.
  let {
    open,
    titleKey,
    messageKey,
    confirmKey,
    cancelKey,
    testidPrefix,
    onConfirm,
    onCancel,
  }: {
    open: boolean;
    titleKey: string;
    messageKey: string;
    confirmKey: string;
    cancelKey: string;
    testidPrefix: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  // Initial focus on the non-destructive default (Cancel), exactly like
  // DiscardPrompt — the a11y lint rejects `autofocus`, so this is the
  // sanctioned way to move focus on open. Guarded on `open` since the button
  // only exists (and `bind:this` only fires) while the modal is rendered.
  let cancelButton = $state<HTMLButtonElement | null>(null);
  $effect(() => {
    if (open) cancelButton?.focus();
  });

  // Escape always cancels, never confirms — the destructive choice must never
  // be reachable as a side effect of dismissing the dialog.
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
    }
  }
</script>

{#if open}
  <div class="modal-backdrop" role="presentation" data-testid={`${testidPrefix}-prompt`}>
    <div
      class="modal"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby={`${testidPrefix}-title`}
      aria-describedby={`${testidPrefix}-message`}
      tabindex="-1"
      onkeydown={onKeydown}
    >
      <h2 id={`${testidPrefix}-title`}>{store.t(titleKey)}</h2>
      <p id={`${testidPrefix}-message`}>{store.t(messageKey)}</p>
      <div class="modal-actions">
        <button
          type="button"
          bind:this={cancelButton}
          onclick={onCancel}
          data-testid={`${testidPrefix}-cancel`}
        >
          {store.t(cancelKey)}
        </button>
        <button
          type="button"
          class="danger"
          onclick={onConfirm}
          data-testid={`${testidPrefix}-confirm`}
        >
          {store.t(confirmKey)}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Identical to DiscardPrompt.svelte's modal styling — see that component's
     header for why the two are not merged into one shared base. */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    z-index: 1000;
  }
  .modal {
    background: var(--surface, #fff);
    color: inherit;
    border-radius: 8px;
    padding: 1.5rem;
    max-width: 28rem;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.35);
  }
  .modal h2 {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
  }
  .modal p {
    margin: 0 0 1.25rem;
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
