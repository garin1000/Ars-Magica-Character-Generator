<script lang="ts">
  import { store } from '../state.svelte';

  const errorText = $derived(store.error ? store.t(`error-${store.error.kind}`) : null);
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
