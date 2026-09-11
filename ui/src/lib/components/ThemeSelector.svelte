<script lang="ts">
  import { store, THEMES, type Theme } from '../state.svelte';

  // Mirrors LanguageSelector/ModeToggle field for field, deliberately: all three
  // live side by side in the settings dialog, and a control that looked different
  // would read as a different KIND of choice.
  //
  // The options are `THEMES` — the taxonomy's one home — and every label goes
  // through a `theme-<id>` Fluent key. Rendering `auto`/`light`/`dark` directly
  // would be the same CLAUDE.md violation as hardcoding an English string.
  function onChange(event: Event) {
    const theme = (event.currentTarget as HTMLSelectElement).value as Theme;
    void store.setTheme(theme);
  }
</script>

<label class="field">
  <span>{store.t('theme-label')}</span>
  <select value={store.theme} onchange={onChange} data-testid="theme-select">
    {#each THEMES as theme (theme)}
      <option value={theme}>{store.t(`theme-${theme}`)}</option>
    {/each}
  </select>
</label>
