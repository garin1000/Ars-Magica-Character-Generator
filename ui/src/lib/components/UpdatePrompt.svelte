<script lang="ts">
  import { store } from '../state.svelte';

  // The update check's consent prompt (M6.6): asked on a first start and on the
  // first start of a newer version. Both answers are recorded with the running
  // version; Escape is "Don't check", so dismissing it sends nothing.
  //
  // SettingsDialog's containment (focus capture and return, a real trap) with
  // ConfirmPrompt's two-button shape, restated rather than shared — see those two
  // components for why the modals are not merged. Neither answer is destructive,
  // so neither carries `danger`. Rendered outside the app shell, which
  // `App.svelte` marks `inert` while this is open.

  const check = store.updateCheck;
  let dialog = $state<HTMLDivElement | null>(null);
  let declineButton = $state<HTMLButtonElement | null>(null);

  // Capture the element that had focus before the prompt took it, and give focus
  // back to it once the prompt closes. Declared before the initial-focus effect
  // so it reads `activeElement` first (see SettingsDialog.svelte).
  $effect(() => {
    if (!check.promptOpen) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    return () => {
      if (opener?.isConnected) opener.focus();
    };
  });

  // Initial focus on "Don't check", the answer that sends nothing, so a stray
  // Enter cannot opt the user in.
  $effect(() => {
    if (check.promptOpen) declineButton?.focus();
  });

  /** The dialog's own focusable controls, in tab order. */
  function focusable(): HTMLElement[] {
    if (!dialog) return [];
    return [
      ...dialog.querySelectorAll<HTMLElement>('select, button, input, textarea, a[href]'),
    ].filter((element) => !element.hasAttribute('disabled') && element.tabIndex !== -1);
  }

  function answer(enabled: boolean): void {
    void check.answerPrompt(enabled);
  }

  // Only the two ends of the ring are handled, as in SettingsDialog.
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      answer(false);
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

{#if check.promptOpen}
  <div class="modal-backdrop" role="presentation" data-testid="update-prompt">
    <div
      class="modal"
      bind:this={dialog}
      role="dialog"
      aria-modal="true"
      aria-labelledby="update-prompt-title"
      aria-describedby="update-prompt-message"
      tabindex="-1"
      onkeydown={onKeydown}
    >
      <h2 id="update-prompt-title">{store.t('update-prompt-title')}</h2>
      <p id="update-prompt-message">{store.t('update-prompt-message')}</p>
      <div class="modal-actions">
        <button
          type="button"
          bind:this={declineButton}
          onclick={() => answer(false)}
          data-testid="update-prompt-decline"
        >
          {store.t('update-prompt-decline')}
        </button>
        <button type="button" onclick={() => answer(true)} data-testid="update-prompt-enable">
          {store.t('update-prompt-enable')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* SettingsDialog's modal shape, restated (see its header). Same z-index: above
     tooltips, below the busy overlay. */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--scrim-modal);
    z-index: 1100;
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
