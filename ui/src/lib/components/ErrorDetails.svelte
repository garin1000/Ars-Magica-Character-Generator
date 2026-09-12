<script lang="ts">
  import { store } from '../state.svelte';

  // The detail behind a ruleset failure (E4, open-todos row 25). `AppError::Ruleset`
  // carries the engine's own integrity diagnostics — one message per violation,
  // each naming the offending ids and the rulebook line to open — and until this
  // component existed the frontend declared the field and read it nowhere: the
  // whole payload was computed, shipped across IPC and dropped on the floor.
  //
  // Only a ruleset failure has one. `AppError::Export` already names its missing
  // keys INSIDE its localized sentence (`error-export`, `$missing`), and the other
  // variants carry a single message that says no more than their sentence does.
  //
  // An empty list is treated as no detail at all: a rejection shaped
  // `{ kind: 'ruleset' }` with no list reaches `#reloadRuleset`'s catch in the
  // tests, and a disclosure that promises details and opens on nothing is worse
  // than none.
  const messages = $derived.by(() => {
    const error = store.error;
    if (error?.kind !== 'ruleset') return [];
    return error.errors ?? [];
  });
</script>

{#if messages.length}
  <!-- A NATIVE <details>: collapsible, focusable, operable by Enter/Space and
       announced as a disclosure by screen readers, with no component, no state
       and no keyboard handling of our own. The app has no reusable disclosure to
       reuse and this needs none.

       Deliberately a SIBLING of the `role="alert"` sentence, never a child of
       it. `role="alert"` implies `aria-live="assertive"` + `aria-atomic="true"`:
       nesting this would make a screen reader interrupt the user and read the
       ENTIRE list — an integrity failure produces one message per violation —
       as a single atomic announcement, and re-announce all of it on every expand
       and collapse. The alert says the one sentence; the disclosure sits beside
       it, reached by Tab, read on demand. -->
  <details class="error-details" data-testid="error-details">
    <summary>{store.t('error-technical-details')}</summary>
    <!-- THE PAYLOAD IS ENGLISH ON PURPOSE — do not "fix" this into a
         mistranslation. These are the engine's rules-editor diagnostics, not UI
         text: they name ids exactly as they are spelled in `rules/core/*.json`
         and cite the rulebook file and line to open, so their value is being
         greppable, verbatim, against the data the reader is editing. They are
         built by over a hundred `format!` sites under
         `integrity.rs::validate_integrity` and have no Fluent keys. The audience is
         whoever edited `rules/` — `CLAUDE.md` names that directory a declared
         trust boundary and the rules data separable from the binary, so a power
         user editing it is a real audience — and this is shown as a developer
         diagnostic, in the same spirit as a stack trace. The same text also goes
         to stderr (`error.rs::AppError::write_ruleset_diagnostics`), which is the
         other half of the same decision. -->
    <ul>
      {#each messages as message, index (index)}
        <li>{message}</li>
      {/each}
    </ul>
  </details>
{/if}
