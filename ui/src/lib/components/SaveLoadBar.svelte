<script lang="ts">
  import { store } from '../state.svelte';

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

<div class="saveload">
  <button
    type="button"
    onclick={() => store.newDocument()}
    disabled={store.busy}
    data-testid="new-button"
  >
    {store.t('action-new')}
  </button>
  <button
    type="button"
    onclick={() => store.open()}
    disabled={store.busy}
    data-testid="open-button"
  >
    {store.t('action-open')}
  </button>
  <button
    type="button"
    onclick={() => store.save()}
    disabled={store.busy}
    data-testid="save-button"
  >
    {store.t('action-save')}
  </button>
  <button
    type="button"
    onclick={() => store.saveAs()}
    disabled={store.busy}
    data-testid="save-as-button"
  >
    {store.t('action-save-as')}
  </button>
  <button
    type="button"
    onclick={() => store.exportMarkdown()}
    disabled={store.busy}
    data-testid="export-button"
  >
    {store.t('action-export')}
  </button>
  {#if errorText}
    <span class="error" role="alert" data-testid="error">{errorText}</span>
  {/if}
</div>
