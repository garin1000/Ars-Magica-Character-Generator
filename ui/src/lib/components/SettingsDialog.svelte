<script lang="ts">
  import { store } from '../state.svelte';
  import LanguageSelector from './LanguageSelector.svelte';
  import ThemeSelector from './ThemeSelector.svelte';
  import ModeToggle from './ModeToggle.svelte';
  import DefaultSagaYearField from './DefaultSagaYearField.svelte';

  // The app's preferences: language, palette, validation strictness, and — since C8 —
  // the saga year NEW documents start at, all persisted across restarts. Reachable
  // from the native menu's Settings item and from the header button on every screen.
  //
  // The fourth field is narrower than the setting it replaces, and that is the point.
  // Until C8 this file held THE saga year, machine-globally, which was wrong the
  // moment a storyguide ran two sagas: one number, correct for one of them, silently
  // wrong for every character of the other. A saga year is a property of the saga, so
  // it travels in the save; what belongs beside language and appearance is only the
  // default a fresh character is stamped with.
  //
  // **Its own component, not a refactor of the two existing modals.** ConfirmPrompt
  // carries a standing note explaining why DiscardPrompt must not be folded into a
  // shared base — its literal `data-testid`s are the e2e suite's entire seam for the
  // discard confirmation — and that reasoning applies with equal force in this
  // direction. What is borrowed here is the *pattern*, never the markup.
  //
  // Each field is the existing control, moved rather than re-implemented:
  // `ModeToggle` was the header's Validation dropdown and is unchanged, so the
  // validation mode gains no second definition anywhere. `LanguageSelector` is the
  // same component the start screen keeps a copy of (see StartScreen.svelte for why
  // the language deliberately appears twice).

  let dialog = $state<HTMLDivElement | null>(null);

  // FOCUS RESTORATION, keyed only on the open flag so it captures the invoker
  // before anything else has moved focus. Effects run in declaration order, so this
  // one reads `document.activeElement` while it is still the header button (or
  // wherever the keyboard was when the menu item fired) — the initial-focus effect
  // below has not run yet. The cleanup is what restores it: it runs when the flag
  // flips back, which is the WAI-ARIA dialog pattern's "focus returns to the element
  // that invoked it".
  //
  // `isConnected` guards the one case where that is wrong: an element the dialog's
  // own changes removed from the document. Focusing a detached node silently does
  // nothing; leaving focus where it is beats forcing it onto an arbitrary substitute.
  $effect(() => {
    if (!store.settingsOpen) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    return () => {
      if (opener?.isConnected) opener.focus();
    };
  });

  // Initial focus, once the dialog's DOM actually exists. Separate from the capture
  // above because it must depend on `dialog`, which `bind:this` fills in later — one
  // combined effect would re-run on that assignment and "capture" the control it had
  // just focused itself.
  $effect(() => {
    if (store.settingsOpen && dialog) focusable()[0]?.focus();
  });

  /** The dialog's own focusable controls, in tab order. */
  function focusable(): HTMLElement[] {
    if (!dialog) return [];
    return [
      ...dialog.querySelectorAll<HTMLElement>('select, button, input, textarea, a[href]'),
    ].filter((element) => !element.hasAttribute('disabled') && element.tabIndex !== -1);
  }

  // A REAL focus trap, which neither existing modal has. DiscardPrompt is contained
  // only by accident — `App.svelte` marks the shell `inert` while it is pending, and
  // that is what keeps Tab inside it — and ConfirmPrompt is not contained at all. An
  // untrapped dialog sends the next Tab into the browser's own chrome, where a
  // keyboard user has no way back and a screen-reader user is simply lost.
  //
  // Only the two ENDS of the ring are handled. Calling `preventDefault` on every Tab
  // would mean reimplementing the browser's focus order by hand, which is how a trap
  // starts skipping the controls it does not know about.
  function onKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      store.closeSettings();
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

{#if store.settingsOpen}
  <div class="modal-backdrop" role="presentation" data-testid="settings-dialog">
    <!-- `role="dialog"`, not the `alertdialog` the two confirmations use: nothing
         here is urgent and nothing is destructive. Escape closes, and closing
         answers no question — every choice inside took effect as it was made, so
         there is no Cancel to get wrong. -->
    <div
      class="modal"
      bind:this={dialog}
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
      tabindex="-1"
      onkeydown={onKeydown}
    >
      <h2 id="settings-title">{store.t('settings-title')}</h2>
      <div class="settings-fields">
        <LanguageSelector />
        <ThemeSelector />
        <ModeToggle />
        <!-- Last, and deliberately the only field here that is about characters
             rather than about the app: it seeds new documents, and the saga year
             itself lives on each document (see DefaultSagaYearField's header). -->
        <DefaultSagaYearField />
      </div>
      <div class="modal-actions">
        <button type="button" onclick={() => store.closeSettings()} data-testid="settings-close">
          {store.t('settings-close')}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* The modal shape of DiscardPrompt/ConfirmPrompt, restated rather than shared —
     see this component's header, and ConfirmPrompt's, for why the three are not
     merged. */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--scrim-modal);
    /* ABOVE the 1000 the two confirmations and `.tooltip-pop` share, and below the
       2000 of `.busy-overlay`. A tooltip popup can be open on the surface the
       dialog covers, and a tie at 1000 would leave which one paints on top to
       source order; the busy overlay stays higher because a native file dialog
       blocks the whole app, this modal included. It cannot collide with the two
       confirmations in practice — the shell is inert while either is pending — so
       this ordering is about the tooltip, deliberately. */
    z-index: 1100;
  }
  .modal {
    background: var(--surface-raised);
    color: inherit;
    border-radius: 8px;
    padding: 1.5rem;
    min-width: 22rem;
    max-width: 28rem;
    box-shadow: var(--shadow-modal);
  }
  .modal h2 {
    margin: 0 0 1rem;
    font-size: 1.1rem;
  }
  .settings-fields {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-bottom: 1.25rem;
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
  }
</style>
