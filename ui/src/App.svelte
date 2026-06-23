<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/state.svelte';
  import LanguageSelector from './lib/components/LanguageSelector.svelte';
  import ModeToggle from './lib/components/ModeToggle.svelte';
  import ItemPicker from './lib/components/ItemPicker.svelte';
  import CharacteristicPicker from './lib/components/CharacteristicPicker.svelte';
  import AbilityAllocator from './lib/components/AbilityAllocator.svelte';
  import SelectionList from './lib/components/SelectionList.svelte';
  import BalanceBar from './lib/components/BalanceBar.svelte';
  import ValidationPanel from './lib/components/ValidationPanel.svelte';
  import SaveLoadBar from './lib/components/SaveLoadBar.svelte';
  import logoUrl from './lib/assets/logo.png';

  onMount(() => {
    void store.init();
  });

  // Keep the document title localized rather than hardcoded in HTML.
  $effect(() => {
    document.title = store.t('app-title');
  });
</script>

<header class="app-header">
  <div class="brand">
    <img class="app-logo" src={logoUrl} alt={store.t('app-logo-alt')} />
    <h1>{store.t('app-title')}</h1>
  </div>
  <div class="controls">
    <LanguageSelector />
    <ModeToggle />
    <SaveLoadBar />
  </div>
  <BalanceBar />
</header>

<main class="layout">
  <section class="region region-source">
    <h2 class="region-title">{store.t('available-title')}</h2>
    <div class="region-columns">
      <ItemPicker side="virtue" />
      <ItemPicker side="flaw" />
    </div>
  </section>

  <section class="region region-traits">
    <h2 class="region-title">
      {store.t('characteristics-title')} &amp; {store.t('abilities-title')}
    </h2>
    <div class="region-columns">
      <CharacteristicPicker />
      <AbilityAllocator />
    </div>
  </section>

  <section class="region region-selected">
    <h2 class="region-title">{store.t('selections-title')}</h2>
    <div class="selected-frame">
      <div class="region-columns">
        <SelectionList side="virtue" />
        <SelectionList side="flaw" />
      </div>
    </div>
    <ValidationPanel docked />
  </section>
</main>
