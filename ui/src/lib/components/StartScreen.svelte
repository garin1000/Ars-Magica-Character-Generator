<script lang="ts">
  import { store } from '../state.svelte';

  // The character types on offer are ruleset data, never a hardcoded set: they are
  // exactly the profiles the loaded ruleset declares. `type_profiles` is a BTreeMap
  // on the Rust side, so its key order already IS id order — no sort is imposed
  // here. A label always maps through the Fluent `type-<id>` key; the slug itself
  // is never rendered.
  const typeIds = $derived(store.ruleset ? Object.keys(store.ruleset.ruleset.type_profiles) : []);

  // Nothing may be started while the ruleset is still loading (no profiles yet) or
  // while a native file dialog is open — the same guard the document toolbar puts
  // on its buttons.
  const blocked = $derived(store.loading || store.busy);

  // The one error surface of this screen. SaveLoadBar's banner is not rendered
  // here, so without this a failed ruleset load would leave the user with no
  // create buttons and no reason for their absence. Same `error-<kind>` keys.
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
  <h2 id="start-heading">{store.t('start-title')}</h2>

  {#if errorText}
    <p class="error-banner" role="alert" data-testid="start-error">{errorText}</p>
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
