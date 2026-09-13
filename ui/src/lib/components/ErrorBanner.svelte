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

{#if store.migrationNotice}
  <!-- The load-time schema migration's report, and NOT an error: it is the
       ordinary outcome of opening an older save, so it renders on its own and
       never waits on `errorText`.
       `role="status"` (polite) rather than the sentence above's assertive
       `role="alert"` — the user has just opened a file and is reading it, so
       this waits its turn. It is a SIBLING of the alert, not a child, for the
       same reason `ErrorDetails` is: that span is assertive and atomic, so
       folding a second unrelated sentence into it would have the whole thing
       re-announced, and would tie this notice's lifetime to an error's.
       Why it exists at all: the migration is LOSSY. The engine reconstructs the
       smallest Aging Point total that still reproduces the recorded scores, so
       the original total is unrecoverable — and the next Save writes the
       reconstruction back as the document's own figures. -->
  <div class="error-banner-block">
    <span class="migration-notice" role="status" data-testid="migration-notice"
      >{store.migrationNotice}</span
    >
  </div>
{/if}
