<script lang="ts">
  import { store } from '../state.svelte';
  import type { ParameterDef, Selection } from '../types';

  let { selection, params }: { selection: Selection; params: ParameterDef[] } = $props();

  function onInput(key: string, event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value.trim();
    store.setParam(selection.ref, key, value);
  }
</script>

{#each params as param (param.key)}
  <label class="param">
    <span>{store.t('param-prompt', { param: param.key })}</span>
    <input
      type="text"
      placeholder={store.t('param-placeholder')}
      value={selection.params?.[param.key] ?? ''}
      oninput={(e) => onInput(param.key, e)}
      data-testid="param-{selection.ref}-{param.key}"
    />
  </label>
{/each}
