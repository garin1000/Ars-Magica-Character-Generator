<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from './lib/state.svelte';
  import { updateCloseGuard } from './lib/ipc';
  import LanguageSelector from './lib/components/LanguageSelector.svelte';
  import ModeToggle from './lib/components/ModeToggle.svelte';
  import StartScreen from './lib/components/StartScreen.svelte';
  import CharacterBanner from './lib/components/CharacterBanner.svelte';
  import WizardShell from './lib/components/WizardShell.svelte';
  import VirtueFlawTab from './lib/components/VirtueFlawTab.svelte';
  import CharacteristicPicker from './lib/components/CharacteristicPicker.svelte';
  import AbilityTab from './lib/components/AbilityTab.svelte';
  import XpBar from './lib/components/XpBar.svelte';
  import ArtGrid from './lib/components/ArtGrid.svelte';
  import SpellTab from './lib/components/SpellTab.svelte';
  import MagicPossessions from './lib/components/MagicPossessions.svelte';
  import SupernaturalBeing from './lib/components/SupernaturalBeing.svelte';
  import EquipmentTab from './lib/components/EquipmentTab.svelte';
  import CharacterDetails from './lib/components/CharacterDetails.svelte';
  import DerivedTotalsPanel from './lib/components/DerivedTotalsPanel.svelte';
  import HouseSelector from './lib/components/HouseSelector.svelte';
  import MythicCompanionTypeSelector from './lib/components/MythicCompanionTypeSelector.svelte';
  import BalanceBar from './lib/components/BalanceBar.svelte';
  import ValidationPanel from './lib/components/ValidationPanel.svelte';
  import SaveLoadBar from './lib/components/SaveLoadBar.svelte';
  import DiscardPrompt from './lib/components/DiscardPrompt.svelte';
  import logoUrl from './lib/assets/logo.png';

  type Tab =
    | 'characteristics'
    | 'virtues_flaws'
    | 'abilities'
    | 'arts'
    | 'spells'
    | 'possessions'
    | 'house_specialisation'
    | 'mythic_type'
    | 'supernatural'
    | 'equipment'
    | 'details'
    | 'totals';
  // Left-to-right: Characteristics, Virtues & Flaws, Abilities, then the two
  // magus-only tabs (Arts, House) and the mythic-companion-only Type tab — each
  // gated on the profile's capability flag (never the type id), so any future
  // capable type gets them automatically.
  const isMagus = $derived(
    store.ruleset?.ruleset.type_profiles[store.entity.type_id]?.is_magus ?? false,
  );
  const hasMythicType = $derived(
    store.ruleset?.ruleset.type_profiles[store.entity.type_id]?.has_mythic_type ?? false,
  );
  // The Supernatural (Might) tab appears for a mythic-companion-capable type
  // (Devil Child, Nephilim) or once the character has an effective Might (any type
  // that took a Might Virtue such as Demonic Blood). Magi never have Might.
  const hasMight = $derived(
    !isMagus && (hasMythicType || (store.effective?.might ?? null) !== null),
  );
  const tabs = $derived<{ id: Tab; key: string }[]>([
    { id: 'details', key: 'tab-details' },
    { id: 'characteristics', key: 'tab-characteristics' },
    { id: 'virtues_flaws', key: 'tab-virtues-flaws' },
    { id: 'abilities', key: 'tab-abilities' },
    ...(isMagus
      ? [
          { id: 'arts' as Tab, key: 'tab-arts' },
          { id: 'spells' as Tab, key: 'tab-spells' },
          { id: 'possessions' as Tab, key: 'tab-possessions' },
          { id: 'house_specialisation' as Tab, key: 'tab-house-specialisation' },
        ]
      : []),
    ...(hasMythicType ? [{ id: 'mythic_type' as Tab, key: 'tab-mythic-type' }] : []),
    ...(hasMight ? [{ id: 'supernatural' as Tab, key: 'tab-supernatural' }] : []),
    // Equipment, Age, Confidence, Personality Traits and Reputations apply to
    // every type (grogs especially carry weapons and armor).
    { id: 'equipment', key: 'tab-equipment' },
    // The derived play-stat read-out applies to every type (read-only totals).
    { id: 'totals', key: 'tab-totals' },
  ]);
  let tab = $state<Tab>('details');

  // If the active tab disappears (e.g. switching away from magus on the Arts
  // tab), fall back to Characteristics so the content area is never blank.
  $effect(() => {
    if (!tabs.some((t) => t.id === tab)) tab = 'details';
  });

  onMount(() => {
    void store.init();
  });

  // Keep the document title localized rather than hardcoded in HTML. Once a file
  // is being tracked, show "name — app" (with an ASCII dirty marker for unsaved
  // edits) via a parametrized Fluent key — never string-composed here.
  $effect(() => {
    const name = store.currentFileName;
    if (name === null) {
      document.title = store.t('app-title');
    } else {
      const key = store.dirty ? 'app-title-document-dirty' : 'app-title-document';
      document.title = store.t(key, { name, app: store.t('app-title') });
    }
  });

  // Standard document-app keyboard shortcuts. Ctrl (or Cmd on macOS) + S/N/O,
  // with Shift+S for Save As. Cmd+Q keeps routing through the OS/close guard, so
  // it is deliberately not handled here.
  //
  // `inert` on the shell does not reach window-level key handlers, so the busy
  // check has to be here too: while a native dialog is open the shortcuts must be
  // as dead as the buttons they mirror (the store actions no-op as well).
  function handleShortcut(event: KeyboardEvent): void {
    if (store.busy) return;
    if (!(event.ctrlKey || event.metaKey)) return;
    const key = event.key.toLowerCase();
    if (key === 's') {
      // Save/Save As write the document being edited. On the startup screen there
      // is no document — only the placeholder entity — so an unguarded Ctrl+S
      // there would write a file for a character that does not exist. The Save
      // buttons are hidden on that screen; this window-level handler is not, so
      // the gate has to be repeated here. It stays live in the wizard, whose
      // character is a real one: the unsaved-changes guard promises the work can
      // be saved rather than lost.
      if (store.view === 'start') return;
      event.preventDefault();
      void (event.shiftKey ? store.saveAs() : store.save());
    } else if (key === 'n') {
      // Ctrl+N stays live on both screens. From the editor it discards (through
      // the unsaved-changes guard) and returns to the type choice; on the startup
      // screen it lands where it already is, which is harmless and keeps the
      // shortcut's meaning the same everywhere.
      event.preventDefault();
      void store.newDocument();
    } else if (key === 'o') {
      // Ctrl+O works on both screens: opening a character is the startup screen's
      // own first offer.
      event.preventDefault();
      void store.open();
    }
  }

  // Mirror the unsaved-changes flag (and the localized dialog strings) to the
  // backend close/quit guard. Re-runs whenever dirty flips or the language
  // changes, so Rust can prompt before discarding on any quit path.
  $effect(() => {
    const { dirty, labels } = store.closeGuardPayload();
    void updateCloseGuard(dirty, labels);
  });
</script>

<svelte:window onkeydown={handleShortcut} />

<!-- Everything interactive lives in the shell so a single `inert` can switch the
     whole app off while a native file dialog is open. The dialogs are parented to
     the window but are NOT input-modal on Linux (rfd has no modal flag, and tao's
     cross-platform Window has no `set_enabled`), so without this the user could
     keep editing the character behind an open Save/Open/Export dialog. `inert`
     drops focus and assistive-tech access; the overlay below swallows the clicks. -->
<div class="app-shell" inert={store.busy} aria-busy={store.busy} data-testid="app-shell">
  <header class="app-header">
    <div class="brand">
      <img class="app-logo" src={logoUrl} alt={store.t('app-logo-alt')} />
      <h1>{store.t('app-title')}</h1>
      <!-- Active save's file name + ASCII dirty marker, on-screen (not only in the
         OS window title). Reuses the derived currentFileName/dirty state; a null
         file name means the document has never been saved. Hidden on the startup
         screen, where it would report an unsaved document that does not exist. -->
      {#if store.view !== 'start'}
        <span class="doc-status" data-testid="doc-status">
          {#if store.currentFileName === null}
            {store.t(store.dirty ? 'app-document-unsaved-dirty' : 'app-document-unsaved')}
          {:else}
            {store.t(store.dirty ? 'app-document-name-dirty' : 'app-document-name', {
              name: store.currentFileName,
            })}
          {/if}
        </span>
      {/if}
    </div>
    <!-- The language applies to the whole app, so it is offered on both screens.
         The validation mode and the document toolbar act on a character, so they
         appear only once one exists. -->
    <div class="controls">
      <LanguageSelector />
      {#if store.view !== 'start'}
        <ModeToggle />
        <SaveLoadBar />
      {/if}
    </div>
  </header>

  {#if store.view === 'start'}
    <StartScreen />
  {:else if store.view === 'wizard'}
    <!-- The wizard edits the same character the editor would, so the banner sits
         above it too; the tab bar does not, because the flow's order is the point.
         The wizard docks its own step-scoped issues panel, so the whole-character
         footer below belongs to the editor branch only. -->
    <CharacterBanner />
    <WizardShell />
  {:else}
    <CharacterBanner />

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
          <VirtueFlawTab />
        </div>
      {:else if tab === 'abilities'}
        <div class="vf-tab">
          <XpBar />
          <AbilityTab />
        </div>
      {:else if tab === 'arts'}
        <div class="vf-tab">
          <XpBar prefix="art-" />
          <ArtGrid />
        </div>
      {:else if tab === 'spells'}
        <div class="vf-tab">
          <SpellTab />
        </div>
      {:else if tab === 'possessions'}
        <div class="vf-tab">
          <div class="tab-scroll">
            <MagicPossessions />
          </div>
        </div>
      {:else if tab === 'equipment'}
        <div class="vf-tab">
          <EquipmentTab />
        </div>
      {:else if tab === 'details'}
        <div class="vf-tab">
          <div class="tab-scroll">
            <CharacterDetails />
          </div>
        </div>
      {:else if tab === 'totals'}
        <div class="vf-tab">
          <div class="tab-scroll">
            <DerivedTotalsPanel />
          </div>
        </div>
      {:else if tab === 'mythic_type'}
        <div class="vf-tab">
          <MythicCompanionTypeSelector />
        </div>
      {:else if tab === 'supernatural'}
        <div class="vf-tab">
          <SupernaturalBeing />
        </div>
      {:else}
        <div class="vf-tab">
          <div class="tab-scroll">
            <HouseSelector />
          </div>
        </div>
      {/if}
    </main>

    <!-- One shared issues panel for the whole character, pinned below the tabs. -->
    <footer class="validation-bar">
      <ValidationPanel docked />
    </footer>
  {/if}
</div>

<!-- Click/hover blocker over the inert shell: a calm scrim that makes the blocked
     state visible and eats every pointer event. Purely decorative (the shell's
     `aria-busy` carries the meaning), so it needs no text and no Fluent key. -->
{#if store.busy}
  <div class="busy-overlay" data-testid="busy-overlay" aria-hidden="true"></div>
{/if}

<!-- Discard-changes confirmation for New/Open (close/quit uses the Rust dialog).
     Outside the shell: it is the one modal the user must still be able to answer
     (it never overlaps a native dialog — `busy` is false while it is open). -->
<DiscardPrompt />
