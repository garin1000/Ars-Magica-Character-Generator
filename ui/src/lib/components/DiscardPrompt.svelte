<script lang="ts">
  import { store } from '../state.svelte';

  // The FALLBACK discard-changes confirmation. Every discard — New, Open,
  // window close, app quit — is confirmed by the native dialog Rust owns
  // (C3b); this modal renders only where no native answer arrives, and the
  // store's `discardPromptOpen` flag is true for exactly those two cases:
  //
  //  * the `e2e-testing` build declines to show a native dialog, because no
  //    WebDriver capability can dismiss one and New sits in essentially every
  //    spec's setup (`ui/e2e/helpers.js`'s `returnToStartScreen`). Its
  //    `data-testid`s below are that seam's entire surface — see
  //    ConfirmPrompt.svelte's header for the standing warning against
  //    refactoring them away;
  //  * the confirmation IPC call failed, where treating a broken bridge as
  //    "yes, discard" would silently destroy an unsaved character.
  //
  // A single, non-stacking modal; the buttons resolve the pending promise.

  // Initial focus on the non-destructive default (Cancel), mirroring
  // StartScreen.svelte's `openButton?.focus()` effect — the a11y lint rejects
  // `autofocus`, so this is the sanctioned way to move focus on open. Guarded on
  // `discardPromptOpen` since the button only exists (and `bind:this` only
  // fires) while the modal is rendered.
  let cancelButton = $state<HTMLButtonElement | null>(null);
  $effect(() => {
    if (store.discardPromptOpen) cancelButton?.focus();
  });

  // Escape always cancels, never discards — the destructive choice must never
  // be reachable as a side effect of dismissing the dialog.
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      store.resolveDiscardPrompt(false);
    }
  }
</script>

{#if store.discardPromptOpen}
  <div class="modal-backdrop" role="presentation" data-testid="discard-prompt">
    <div
      class="modal"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="discard-title"
      aria-describedby="discard-message"
      tabindex="-1"
      onkeydown={onKeydown}
    >
      <h2 id="discard-title">{store.t('discard-changes-title')}</h2>
      <p id="discard-message">{store.t('discard-changes-message')}</p>
      <div class="modal-actions">
        <button
          type="button"
          bind:this={cancelButton}
          onclick={() => store.resolveDiscardPrompt(false)}
          data-testid="discard-cancel"
        >
          {store.t('discard-changes-cancel')}
        </button>
        <button
          type="button"
          class="danger"
          onclick={() => store.resolveDiscardPrompt(true)}
          data-testid="discard-confirm"
        >
          {store.t('discard-changes-confirm')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--scrim-modal);
    z-index: 1000;
  }
  .modal {
    /* `--surface-raised`, not the darker `--surface-overlay`: this panel has no
       border, only a shadow, so it needs to sit LIGHTER than the scrimmed app
       behind it to read as raised at all. It used to name an undeclared
       `--surface` with a `#fff` fallback, which is what the browser actually
       painted — a white panel under `color: inherit` ink, i.e. white on white. */
    background: var(--surface-raised);
    color: inherit;
    border-radius: 8px;
    padding: 1.5rem;
    max-width: 28rem;
    box-shadow: var(--shadow-modal);
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
