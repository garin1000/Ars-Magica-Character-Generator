<script lang="ts">
  import { store } from '../state.svelte';
  import type { ValidationMode } from '../types';

  const modes: ValidationMode[] = ['enforced', 'advisory', 'silent'];

  function onChange(event: Event) {
    const mode = (event.currentTarget as HTMLSelectElement).value as ValidationMode;
    void store.setMode(mode);
  }
</script>

<label class="field">
  <span>{store.t('mode-label')}</span>
  <select value={store.mode} onchange={onChange} data-testid="mode-select">
    {#each modes as mode (mode)}
      <option value={mode}>{store.t(`mode-${mode}`)}</option>
    {/each}
  </select>
</label>
