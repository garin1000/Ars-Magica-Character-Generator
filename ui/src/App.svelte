<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/state.svelte';
  import LanguageSelector from './lib/components/LanguageSelector.svelte';
  import ModeToggle from './lib/components/ModeToggle.svelte';
  import CharacterTypeSelector from './lib/components/CharacterTypeSelector.svelte';
  import ItemPicker from './lib/components/ItemPicker.svelte';
  import SelectionList from './lib/components/SelectionList.svelte';
  import CharacteristicPicker from './lib/components/CharacteristicPicker.svelte';
  import AbilityPicker from './lib/components/AbilityPicker.svelte';
  import AbilitySelectionList from './lib/components/AbilitySelectionList.svelte';
  import AbilityXpBar from './lib/components/AbilityXpBar.svelte';
  import BalanceBar from './lib/components/BalanceBar.svelte';
  import ValidationPanel from './lib/components/ValidationPanel.svelte';
  import SaveLoadBar from './lib/components/SaveLoadBar.svelte';
  import logoUrl from './lib/assets/logo.png';

  type Tab = 'characteristics' | 'virtues_flaws' | 'abilities';
  // Left-to-right: Characteristics, Virtues & Flaws, Abilities.
  const tabs: { id: Tab; key: string }[] = [
    { id: 'characteristics', key: 'tab-characteristics' },
    { id: 'virtues_flaws', key: 'tab-virtues-flaws' },
    { id: 'abilities', key: 'tab-abilities' },
  ];
  let tab = $state<Tab>('characteristics');

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
    <CharacterTypeSelector />
    <LanguageSelector />
    <ModeToggle />
    <SaveLoadBar />
  </div>
</header>

<div class="tabbar" role="tablist">
  {#each tabs as t (t.id)}
    <button
      type="button"
      role="tab"
      class="tab"
      class:active={tab === t.id}
      aria-selected={tab === t.id}
      onclick={() => (tab = t.id)}
      data-testid="tab-{t.id}"
    >
      {store.t(t.key)}
    </button>
  {/each}
</div>

<main class="tab-content">
  {#if tab === 'characteristics'}
    <CharacteristicPicker />
  {:else if tab === 'virtues_flaws'}
    <div class="vf-tab">
      <BalanceBar />
      <div class="region-row">
        <section class="region region-source">
          <h2 class="region-title">{store.t('available-title')}</h2>
          <div class="region-columns">
            <ItemPicker side="virtue" />
            <ItemPicker side="flaw" />
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
        </section>
      </div>
    </div>
  {:else}
    <div class="vf-tab">
      <AbilityXpBar />
      <div class="region-row">
        <section class="region region-source">
          <h2 class="region-title">{store.t('available-title')}</h2>
          <AbilityPicker />
        </section>
        <section class="region region-selected">
          <h2 class="region-title">{store.t('selections-title')}</h2>
          <div class="selected-frame">
            <AbilitySelectionList />
          </div>
        </section>
      </div>
    </div>
  {/if}
</main>

<!-- One shared issues panel for the whole character, pinned below the tabs. -->
<footer class="validation-bar">
  <ValidationPanel docked />
</footer>
