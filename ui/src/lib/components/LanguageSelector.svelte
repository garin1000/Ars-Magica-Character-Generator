<script lang="ts">
  import { store } from '../state.svelte';
  import { AVAILABLE_LANGS, type Lang } from '../i18n';

  // Mounted TWICE on the startup screen, which is deliberate — see
  // StartScreen.svelte for the first-run reason. The two instances need distinct
  // `data-testid`s so the e2e suite can name one without ambiguity, hence the
  // prop; the settings dialog's copy keeps the original id, because that is the
  // one every existing spec already reaches for.
  let { testid = 'language-select' }: { testid?: string } = $props();

  function onChange(event: Event) {
    const lang = (event.currentTarget as HTMLSelectElement).value as Lang;
    void store.setLang(lang);
  }
</script>

<label class="field">
  <span>{store.t('language-label')}</span>
  <select value={store.lang} onchange={onChange} data-testid={testid}>
    {#each AVAILABLE_LANGS as lang (lang)}
      <option value={lang}>{store.t(`language-name-${lang}`)}</option>
    {/each}
  </select>
</label>
