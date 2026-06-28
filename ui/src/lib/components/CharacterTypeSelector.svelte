<script lang="ts">
  import { store } from '../state.svelte';

  // Type ids are data from the ruleset, never hardcoded; labels map through the
  // Fluent `type-<id>` key so the slug is never rendered directly.
  const typeIds = $derived(
    store.ruleset ? Object.keys(store.ruleset.ruleset.type_profiles).sort() : [],
  );

  function onChange(event: Event) {
    void store.setType((event.currentTarget as HTMLSelectElement).value);
  }
</script>

<label class="field">
  <span>{store.t('type-label')}</span>
  <select value={store.entity.type_id} onchange={onChange} data-testid="type-select">
    {#each typeIds as id (id)}
      <option value={id}>{store.t(`type-${id}`)}</option>
    {/each}
  </select>
</label>
