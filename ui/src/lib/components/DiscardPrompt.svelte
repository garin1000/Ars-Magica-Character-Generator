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
  let dialog = $state<HTMLDivElement | null>(null);
  $effect(() => {
    if (store.discardPromptOpen) cancelButton?.focus();
  });

  /** This dialog's own focusable controls, in tab order — identical selector to
   *  `ConfirmPrompt.svelte`/`SettingsDialog.svelte`. */
  function focusable(): HTMLElement[] {
    if (!dialog) return [];
    return [
      ...dialog.querySelectorAll<HTMLElement>('select, button, input, textarea, a[href]'),
    ].filter((element) => !element.hasAttribute('disabled') && element.tabIndex !== -1);
  }

  // A REAL focus trap (S1/E1, full-audit round 1) — lifted from
  // SettingsDialog.svelte, which is the right template here: like that dialog,
  // this one renders OUTSIDE the app shell while `App.svelte` marks the shell
  // `inert`, so a local `onkeydown` is enough (no `svelte:window` listener is
  // needed the way ConfirmPrompt's is, since that one renders INSIDE the live
  // shell). `inert` only keeps Tab from LEAVING the shell; it does nothing to
  // make Tab LOOP between Cancel/Discard once the ring's last control is
  // reached, which is the gap this closes.
  //
  // Only the two ENDS of the ring are handled. Calling `preventDefault` on every
  // Tab would mean reimplementing the browser's own focus order by hand, which
  // is how a trap starts skipping controls it does not know about.
  //
  // Escape always cancels, never discards — the destructive choice must never
  // be reachable as a side effect of dismissing the dialog.
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      store.resolveDiscardPrompt(false);
      return;
    }
    if (event.key !== 'Tab') return;
    const controls = focusable();
    if (controls.length === 0) return;
    const first = controls[0];
    const last = controls[controls.length - 1];
    const active = document.activeElement;
    const inside = active instanceof HTMLElement && dialog?.contains(active) === true;
    const leaving = event.shiftKey ? active === first : active === last;
    if (!leaving && inside) return;
    event.preventDefault();
    (event.shiftKey ? last : first).focus();
  }
</script>

{#if store.discardPromptOpen}
  <div class="modal-backdrop" role="presentation" data-testid="discard-prompt">
    <div
      class="modal"
      bind:this={dialog}
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
