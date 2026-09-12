<script lang="ts">
  import { store } from '../state.svelte';
  import ErrorDetails from './ErrorDetails.svelte';
  import LanguageSelector from './LanguageSelector.svelte';

  // The character types on offer are ruleset data, never a hardcoded set: they are
  // exactly the profiles the loaded ruleset declares. `type_profiles` is a BTreeMap
  // on the Rust side, so its key order already IS id order — no sort is imposed
  // here. A label always maps through the Fluent `type-<id>` key; the slug itself
  // is never rendered.
  const typeIds = $derived(store.ruleset ? Object.keys(store.ruleset.ruleset.type_profiles) : []);

  // Nothing may be started while the ruleset is still loading (no profiles yet) or
  // while a native file dialog is open — the same guard `documentActionEnabled`
  // puts on every document action.
  const blocked = $derived(store.loading || store.busy);

  // The one error surface of this screen. The header's `ErrorBanner` is not
  // rendered here, so without this a failed ruleset load would leave the user
  // with no create buttons and no reason for their absence. Same `error-<kind>`
  // keys.
  const errorText = $derived(store.error ? store.t(`error-${store.error.kind}`) : null);

  // Initial focus: the first control of the screen, so keyboard and screen-reader
  // users land inside it instead of on <body>. Set from an effect rather than the
  // `autofocus` attribute, which the a11y lint rejects.
  let openButton = $state<HTMLButtonElement | null>(null);
  $effect(() => {
    openButton?.focus();
  });
</script>

<main class="start-screen" data-testid="start-screen" aria-labelledby="start-heading">
  <!-- The language, on the screen the app first opens on — a SECOND copy of the
       control the settings dialog holds, and deliberately so.
       C4 moved the language into that dialog, whose title, field labels and
       opening button are all rendered in the language the app is currently running
       in. On a first launch that is English, so a reader who cannot read "Settings"
       would be unable to reach the one control that fixes it — and because C4 also
       persists the choice, the first launch is when it matters most. A `<select>`
       showing "English / Deutsch" (each language's own endonym) requires reading no
       English at all, so it stays here where it cannot be missed.
       Its own `data-testid`: two instances of one control must be separately
       nameable, and the dialog's copy keeps the original id. -->
  <LanguageSelector testid="start-language-select" />

  <h2 id="start-heading">{store.t('start-title')}</h2>

  <!-- The alert carries the sentence; the disclosure beside it carries the
       engine's own diagnostics, which is the only thing that says WHICH id in
       WHICH file broke. A startup failure lands here and nowhere else — the
       header's `ErrorBanner` is not rendered on this screen — so without this the
       payload would be unreachable at exactly the moment it matters most.
       Not nested inside the alert: see `ErrorDetails.svelte`. -->
  {#if errorText}
    <p class="error-banner" role="alert" data-testid="start-error">{errorText}</p>
    <ErrorDetails />
  {/if}

  <section class="start-choice" aria-labelledby="start-open-heading">
    <h3 id="start-open-heading">{store.t('start-open-title')}</h3>
    <div class="start-types">
      <button
        bind:this={openButton}
        type="button"
        onclick={() => store.open()}
        disabled={blocked}
        data-testid="start-open"
      >
        {store.t('action-open')}
      </button>
      <!-- The same open, landing on the guided flow instead of the editor. A file
           saved mid-flow resumes where it was left; one built in the editor opens
           with every step reachable. -->
      <button
        type="button"
        onclick={() => store.openIntoWizard()}
        disabled={blocked}
        data-testid="start-open-wizard"
      >
        {store.t('action-open-into-wizard')}
      </button>
    </div>
    <p class="hint">{store.t('start-open-wizard-hint')}</p>
  </section>

  <section class="start-choice" aria-labelledby="start-create-heading">
    <h3 id="start-create-heading">{store.t('start-create-title')}</h3>
    <p class="hint">{store.t('start-create-hint')}</p>
    <p class="hint">{store.t('start-create-mode-hint')}</p>
    <div class="start-types">
      {#each typeIds as id (id)}
        <button
          type="button"
          onclick={() => store.createCharacter(id)}
          disabled={blocked}
          data-testid="start-create-{id}"
        >
          {store.t(`type-${id}`)}
        </button>
      {/each}
    </div>
  </section>

  <!-- The guided flow is entered per character type, exactly like direct creation:
       a type is fixed at creation, so it is chosen before the first step. -->
  <section class="start-choice" aria-labelledby="start-wizard-heading">
    <h3 id="start-wizard-heading">{store.t('start-wizard-title')}</h3>
    <p class="hint">{store.t('start-wizard-hint')}</p>
    <div class="start-types">
      {#each typeIds as id (id)}
        <button
          type="button"
          onclick={() => store.startWizard(id)}
          disabled={blocked}
          data-testid="start-wizard-{id}"
        >
          {store.t(`type-${id}`)}
        </button>
      {/each}
    </div>
  </section>
</main>
