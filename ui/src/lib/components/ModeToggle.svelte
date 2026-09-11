<script lang="ts">
  import { store } from '../state.svelte';
  import type { ValidationMode } from '../types';

  // Unchanged since C4 MOVED it: this was the header's Validation dropdown and is
  // now a field of the settings dialog, which is what "the validation mode is the
  // existing control, relocated" means — it gains no second definition, and the
  // `mode-select` testid every e2e spec already names travels with it.
  //
  // `ValidationMode` is an ENGINE enum, so the option values are the engine's own
  // serde spelling and the labels go through `mode-<id>` Fluent keys; the slug is
  // never what is rendered.
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
