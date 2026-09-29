<script lang="ts">
  import { store } from '../state.svelte';
  import logoUrl from '../assets/logo.png';

  // The character type is chosen once, when the character is created, so the
  // banner displays it and never offers to change it. `translate()` echoes an
  // unknown key back, so a save naming a type the loaded ruleset has no profile
  // for gets its own key instead of rendering the raw slug. (The engine reports
  // that save's `unknown_type` separately, so the real problem is still shown.)
  const profile = $derived(store.ruleset?.ruleset.type_profiles[store.entity.type_id]);
  const typeName = $derived(
    profile ? store.t(`type-${store.entity.type_id}`) : store.t('type-unknown'),
  );
</script>

<!-- Shared chrome above both the editor and the guided wizard: the character's
     immutable type plus its name and description. The wizard edits the same
     entity, so the banner it shows is the same one.

     U4 (P5+P6, `docs/open-todos.md`): the Ars Magica Open License logo and the
     guided-creation entry both moved here from `App.svelte`'s header. The row
     is now TWO columns — `.char-banner-main` (type/name/description, stacked)
     and the logo, right-bound beside it — rather than one, so the logo can be
     sized to the first two lines alone (app.css) without the description row
     (and the button riding along it) pulling that ceiling down with it. -->
<section class="char-banner">
  <div class="char-banner-main">
    <!-- The type and nothing else (manual-testing-findings #3). The Virtue/Flaw budget
         prose and the Gift-policy sentence used to ride along here; they explained the
         rules rather than showing the character, and the budget numbers are already on
         the Virtues & Flaws balance bar where they are acted on. This banner sits above
         every tab and every wizard step, so every line it does not spend is height the
         surfaces below get back. -->
    <p class="char-type" data-testid="character-type">
      <span class="char-type-label">{store.t('type-label')}</span>
      <span class="char-type-value">{typeName}</span>
    </p>
    <input
      class="name-input"
      value={store.entity.name ?? ''}
      oninput={(e) => store.setIdentity('name', (e.currentTarget as HTMLInputElement).value)}
      placeholder={store.t('identity-name-placeholder')}
      aria-label={store.t('identity-name')}
      data-testid="identity-name"
    />
    <div class="char-banner-desc-row">
      <input
        class="desc-input"
        value={store.entity.description ?? ''}
        oninput={(e) =>
          store.setIdentity('description', (e.currentTarget as HTMLInputElement).value)}
        placeholder={store.t('identity-description-placeholder')}
        aria-label={store.t('identity-description')}
        data-testid="identity-description"
      />
      <!-- Take the character on screen into the guided flow (#31). Offered only
           from the editor — the wizard is where it leads — and only for a type
           the loaded ruleset declares a profile for, since the profile's phases
           ARE the rail. Moved here from the header by U4/P6, onto the same row
           as the one-liner description. -->
      {#if store.view === 'editor' && store.canEnterWizard}
        <button
          type="button"
          onclick={() => store.enterWizard()}
          disabled={store.busy}
          data-testid="wizard-continue-button"
        >
          {store.t('action-continue-in-wizard')}
        </button>
      {/if}
    </div>
  </div>
  <!-- The Ars Magica Open License attribution mark. Moved here from the
       header's upper right by U4/P5: right-bound beside the type/name column,
       sized to their combined height (app.css) rather than to whatever the
       header's controls happened to be. -->
  <img class="app-logo" src={logoUrl} alt={store.t('app-logo-alt')} />
</section>
