<script lang="ts">
  import { store } from '../state.svelte';
  import { AVAILABLE_LANGS, type Lang } from '../i18n';

  function onChange(event: Event) {
    const lang = (event.currentTarget as HTMLSelectElement).value as Lang;
    void store.setLang(lang);
  }
</script>

<label class="field">
  <span>{store.t('language-label')}</span>
  <select value={store.lang} onchange={onChange} data-testid="language-select">
    {#each AVAILABLE_LANGS as lang (lang)}
      <option value={lang}>{store.t(`language-name-${lang}`)}</option>
    {/each}
  </select>
</label>
