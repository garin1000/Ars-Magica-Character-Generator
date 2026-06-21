<script lang="ts">
  import { store } from '../state.svelte';

  const errorText = $derived(store.error ? store.t(`error-${store.error.kind}`) : null);
</script>

<div class="saveload">
  <button type="button" onclick={() => store.save()} data-testid="save-button">
    {store.t('action-save')}
  </button>
  <button type="button" onclick={() => store.load()} data-testid="load-button">
    {store.t('action-load')}
  </button>
  {#if errorText}
    <span class="error" role="alert" data-testid="error">{errorText}</span>
  {/if}
</div>
