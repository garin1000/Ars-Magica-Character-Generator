<script lang="ts">
  import { store } from '../state.svelte';
  import ErrorDetails from './ErrorDetails.svelte';

  // The app's one failed-operation surface for the editor and the wizard: save,
  // load, export, validate and the native-menu build all report here. (The
  // startup screen has its own, `start-error` in `StartScreen.svelte`, because a
  // ruleset that failed to load leaves no header to hang this one on.)
  //
  // It lived in `SaveLoadBar` until C3c removed that toolbar. It was never a
  // document action and had no reason to go to the native menu with the five
  // that were, so it stayed in the header as a component of its own.
  //
  // The export error carries the specific missing Fluent/catalogue keys
  // (`AppError.missing`, mirroring `AppError::Export` in
  // `crates/arm-app/src/error.rs`) — pass them through so `error-export` can
  // name them, instead of a passive "some text is missing" with no way to act
  // on it or file a useful bug report.
  const errorText = $derived.by(() => {
    const error = store.error;
    if (!error) return null;
    if (error.kind === 'export') {
      return store.t('error-export', { missing: error.missing.join(', ') });
    }
    return store.t(`error-${error.kind}`);
  });
</script>

{#if errorText}
  <!-- A ruleset failure reaches this surface as well as the start screen's, which
       is why the disclosure is mounted here too: `setLang` reloads the ruleset for
       the new language, so a rules-editor who has just broken an
       `rules/i18n/<lang>/*.json` file and switches the UI language is told about it
       HERE, in the editor, with their character still on screen.
       The alert wraps the SENTENCE only and the disclosure is its sibling — see
       `ErrorDetails.svelte` for why nesting it inside an assertive, atomic live
       region would be an accessibility defect. -->
  <div class="error-banner-block">
    <span class="error-banner" role="alert" data-testid="error">{errorText}</span>
    <ErrorDetails />
  </div>
{/if}
