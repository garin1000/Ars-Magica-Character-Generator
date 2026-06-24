<script lang="ts">
  import { store } from '../state.svelte';
  import { CHARACTERISTICS, type ParameterDef, type Selection } from '../types';

  let { selection, params }: { selection: Selection; params: ParameterDef[] } = $props();

  function onInput(key: string, event: Event) {
    const value = (event.currentTarget as HTMLInputElement).value.trim();
    store.setParam(selection.ref, key, value);
  }

  function onSelect(key: string, event: Event) {
    store.setParam(selection.ref, key, (event.currentTarget as HTMLSelectElement).value);
  }
</script>

{#each params as param (param.key)}
  <label class="param">
    <span>{store.t('param-prompt', { param: store.t(`param-label-${param.key}`) })}</span>
    {#if param.domain === 'characteristic'}
      <select
        value={selection.params?.[param.key] ?? ''}
        onchange={(e) => onSelect(param.key, e)}
        data-testid="param-{selection.ref}-{param.key}"
      >
        <option value="" disabled>{store.t('param-placeholder')}</option>
        {#each CHARACTERISTICS as characteristic (characteristic)}
          <option value="characteristic.{characteristic}">
            {store.t(`characteristic-${characteristic}`)}
          </option>
        {/each}
      </select>
    {:else}
      <input
        type="text"
        placeholder={store.t('param-placeholder')}
        value={selection.params?.[param.key] ?? ''}
        oninput={(e) => onInput(param.key, e)}
        data-testid="param-{selection.ref}-{param.key}"
      />
    {/if}
  </label>
{/each}
