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

  let cancelButton = $state<HTMLButtonElement | null>(null);
  let dialog = $state<HTMLDivElement | null>(null);

  // FOCUS RESTORATION, keyed only on `open` so it captures the invoker before
  // anything else has moved focus: effects run in declaration order, so this one
  // reads `document.activeElement` while it is still the Remove button that opened
  // the prompt — the initial-focus effect below has not run yet. The cleanup is
  // what restores it, running when `open` flips back, which is the WAI-ARIA dialog
  // pattern's "focus returns to the element that invoked it". Without it, answering
  // the prompt left focus on `<body>` and a keyboard user restarted from the top of
  // the document (Sabine 2).
  //
  // `isConnected` guards the one case where restoring is wrong: confirming a removal
  // deletes the very control that invoked it. Focusing a detached node silently does
  // nothing, so leaving focus where it is beats forcing it onto a substitute.
  $effect(() => {
    if (!open) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    return () => {
      if (opener?.isConnected) opener.focus();
    };
  });

  // Initial focus on the non-destructive default (Cancel), exactly like
  // DiscardPrompt — the a11y lint rejects `autofocus`, so this is the
  // sanctioned way to move focus on open. Guarded on `open` since the button
  // only exists (and `bind:this` only fires) while the modal is rendered.
  $effect(() => {
    if (open) cancelButton?.focus();
  });

  /** This dialog's own focusable controls, in tab order. */
  function focusable(): HTMLElement[] {
    if (!dialog) return [];
    return [
      ...dialog.querySelectorAll<HTMLElement>('select, button, input, textarea, a[href]'),
    ].filter((element) => !element.hasAttribute('disabled') && element.tabIndex !== -1);
  }

  // A REAL focus trap, lifted from SettingsDialog.svelte (which says in prose that
  // this component had none). Only the two ENDS of the ring are handled: calling
  // `preventDefault` on every Tab would mean reimplementing the browser's focus
  // order by hand, which is how a trap starts skipping the controls it does not
  // know about.
  //
  // Escape always cancels, never confirms — the destructive choice must never
  // be reachable as a side effect of dismissing the dialog.
  function onKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      onCancel();
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

  // …and the trap has to listen at the WINDOW, which is the one place this dialog
  // differs from the other two. `DiscardPrompt` and `SettingsDialog` render outside
  // the app shell and `App.svelte` marks that shell `inert` while either is open, so
  // focus cannot be outside them in the first place. This one renders INSIDE the
  // shell (FamiliarPanel.svelte, TalismanPanel.svelte), so the same `inert` would
  // cover the prompt itself and leave it unanswerable — the trade the comment above
  // `<SettingsDialog />` in App.svelte already spells out. A keydown from a control
  // behind the modal never reaches the dialog's own handler, so the window catches
  // it and pulls focus back in; an event from inside the dialog is left to the
  // element handler, which is what keeps the two from double-handling one press.
  function onWindowKeydown(event: KeyboardEvent) {
    if (event.target instanceof Node && dialog?.contains(event.target) === true) return;
    onKeydown(event);
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

{#if open}
  <div class="modal-backdrop" role="presentation" data-testid={`${testidPrefix}-prompt`}>
    <div
      class="modal"
      bind:this={dialog}
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
    background: var(--scrim-modal);
    z-index: 1000;
  }
  .modal {
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
