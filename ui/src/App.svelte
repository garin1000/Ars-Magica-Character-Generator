<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/state.svelte';
  import LanguageSelector from './lib/components/LanguageSelector.svelte';
  import ModeToggle from './lib/components/ModeToggle.svelte';
  import ItemPicker from './lib/components/ItemPicker.svelte';
  import SelectionList from './lib/components/SelectionList.svelte';
  import BalanceBar from './lib/components/BalanceBar.svelte';
  import ValidationPanel from './lib/components/ValidationPanel.svelte';
  import SaveLoadBar from './lib/components/SaveLoadBar.svelte';

  onMount(() => {
    void store.init();
  });

  // Keep the document title localized rather than hardcoded in HTML.
  $effect(() => {
    document.title = store.t('app-title');
  });
</script>

<header class="app-header">
  <h1>{store.t('app-title')}</h1>
  <div class="controls">
    <LanguageSelector />
    <ModeToggle />
    <SaveLoadBar />
  </div>
  <BalanceBar />
</header>

<main class="layout">
  <ItemPicker />
  <SelectionList />
  <ValidationPanel />
</main>
