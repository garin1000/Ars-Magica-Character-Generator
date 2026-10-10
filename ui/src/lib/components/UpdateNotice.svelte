<script lang="ts">
  import { store } from '../state.svelte';

  // The newer-release notice (M6.6, decision 8). Polite and dismissible for the
  // session. Mounted inside the app shell, so the shell's `inert` gates it, and on
  // every screen, because the check usually answers while the start screen shows.
  //
  // The version is the only remote-derived value ever rendered: Rust parses it as
  // strict semver, and it goes through Fluent as a STRING arg so no number
  // formatting touches it. The link is a fixed URL Rust builds; nothing from the
  // response is used as one.
  const check = store.updateCheck;
</script>

{#if check.noticeVersion}
  <div class="update-notice" role="status" data-testid="update-notice">
    <span>{store.t('update-available', { version: check.noticeVersion })}</span>
    <button
      type="button"
      onclick={() => void check.openReleasePage()}
      data-testid="update-notice-open"
    >
      {store.t('update-open-release-page')}
    </button>
    <button type="button" onclick={() => check.dismiss()} data-testid="update-notice-dismiss">
      {store.t('update-dismiss')}
    </button>
  </div>
{/if}

<style>
  /* Compact, one row, in the ordinary ink of `.migration-notice`: nothing failed. */
  .update-notice {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: var(--font-small);
  }
</style>
