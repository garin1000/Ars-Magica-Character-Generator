<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { store } from './lib/state.svelte';
  import { updateCloseGuard } from './lib/ipc';
  import type { AppError } from './lib/types';
  import LanguageSelector from './lib/components/LanguageSelector.svelte';
  import ModeToggle from './lib/components/ModeToggle.svelte';
  import StartScreen from './lib/components/StartScreen.svelte';
  import CharacterBanner from './lib/components/CharacterBanner.svelte';
  import WizardShell from './lib/components/WizardShell.svelte';
  import VirtueFlawTab from './lib/components/VirtueFlawTab.svelte';
  import CharacteristicPicker from './lib/components/CharacteristicPicker.svelte';
  import AbilityTab from './lib/components/AbilityTab.svelte';
  import ExperienceStep from './lib/components/ExperienceStep.svelte';
  import XpBar from './lib/components/XpBar.svelte';
  import ArtGrid from './lib/components/ArtGrid.svelte';
  import SpellTab from './lib/components/SpellTab.svelte';
  import SpellBudgetBar from './lib/components/SpellBudgetBar.svelte';
  import MagicPossessions from './lib/components/MagicPossessions.svelte';
  import SupernaturalBeing from './lib/components/SupernaturalBeing.svelte';
  import EquipmentTab from './lib/components/EquipmentTab.svelte';
  import CharacterDetails from './lib/components/CharacterDetails.svelte';
  import PersonalityReputationsStep from './lib/components/PersonalityReputationsStep.svelte';
  import AgingPanel from './lib/components/AgingPanel.svelte';
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
    | 'experience'
    | 'abilities'
    | 'arts'
    | 'spells'
    | 'possessions'
    | 'house_specialisation'
    | 'mythic_type'
    | 'supernatural'
    | 'personality_reputations'
    | 'aging'
    | 'equipment'
    | 'details'
    | 'totals';
  // The tab list mirrors the wizard's phase list (guided-creation review #28):
  // every phase with an editor counterpart is exactly one tab, in phase order, and
  // each tab mounts the very component its step does. The tabs with no phase
  // (Magic Items, Equipment, Totals) follow at the end.
  //
  // Left-to-right: Details (`concept`), Characteristics, Virtues & Flaws,
  // Experience, Abilities, the magus-only Arts and Spells, Personality &
  // Reputations, Aging, then the magus-only Magic Items/House, the
  // mythic-companion-only Type tab, Equipment and Totals — each gated on the
  // profile's capability flag (never the type id), so any future capable type gets
  // them automatically.
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
  // The Experience tab is gated on the RULESET shipping life-stage rules, not on
  // the character type: `LifeStagePanel` gates on exactly this, so every type can
  // choose where its experience comes from, and reading the same flag in both
  // places is what stops the tab and its content from ever disagreeing. A ruleset
  // with no life-stage rules leaves the typed pool the only funding source, and
  // that lives on the Abilities tab's XP bar.
  const hasLifeStageRules = $derived((store.ruleset?.ruleset.life_stages ?? null) !== null);
  const tabs = $derived<{ id: Tab; key: string }[]>([
    { id: 'details', key: 'tab-details' },
    { id: 'characteristics', key: 'tab-characteristics' },
    { id: 'virtues_flaws', key: 'tab-virtues-flaws' },
    ...(hasLifeStageRules ? [{ id: 'experience' as Tab, key: 'tab-experience' }] : []),
    { id: 'abilities', key: 'tab-abilities' },
    ...(isMagus
      ? [
          { id: 'arts' as Tab, key: 'tab-arts' },
          { id: 'spells' as Tab, key: 'tab-spells' },
        ]
      : []),
    // Personality Traits, Reputations and the aging surface apply to every type,
    // ungated: a grog ages and has traits like anyone else, and a loaded character
    // carrying either must have somewhere to edit it.
    { id: 'personality_reputations', key: 'tab-personality-reputations' },
    { id: 'aging', key: 'tab-aging' },
    ...(isMagus
      ? [
          { id: 'possessions' as Tab, key: 'tab-possessions' },
          { id: 'house_specialisation' as Tab, key: 'tab-house-specialisation' },
        ]
      : []),
    ...(hasMythicType ? [{ id: 'mythic_type' as Tab, key: 'tab-mythic-type' }] : []),
    ...(hasMight ? [{ id: 'supernatural' as Tab, key: 'tab-supernatural' }] : []),
    // Equipment applies to every type (grogs especially carry weapons and armor).
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

  // WAI-ARIA Tabs keyboard pattern: Left/Right (and Home/End) move among tabs
  // and activate the one they land on ("automatic activation"), and only the
  // active tab sits in the page's Tab order (`tabindex="0"`; the rest are
  // `-1`) — a screen-reader or keyboard user tabbing into the bar lands once,
  // then arrows across it, rather than tabbing through every tab button.
  async function focusTab(index: number): Promise<void> {
    if (tabs.length === 0) return;
    const wrapped = ((index % tabs.length) + tabs.length) % tabs.length;
    tab = tabs[wrapped].id;
    await tick();
    document.getElementById(`tab-${tabs[wrapped].id}`)?.focus();
  }
  function onTabsKeydown(event: KeyboardEvent): void {
    const current = tabs.findIndex((t) => t.id === tab);
    if (current === -1) return;
    if (event.key === 'ArrowRight') {
      event.preventDefault();
      void focusTab(current + 1);
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault();
      void focusTab(current - 1);
    } else if (event.key === 'Home') {
      event.preventDefault();
      void focusTab(0);
    } else if (event.key === 'End') {
      event.preventDefault();
      void focusTab(tabs.length - 1);
    }
  }

  onMount(() => {
    void store.init();
  });

  // Keep `<html lang>` in sync with the active UI language: index.html's
  // static `lang="en"` is only the pre-mount fallback, and assistive tech uses
  // this attribute to pick pronunciation/voice rules, so it must track a
  // runtime language switch rather than staying English forever (S3).
  $effect(() => {
    document.documentElement.lang = store.lang;
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
  // changes, so Rust can prompt before discarding on any quit path. A rejection
  // here (IPC hiccup) must not vanish silently: it would leave the backend's
  // belief about the dirty state stale on a load-bearing guard, with no
  // diagnostic — so surface it the same way every other IPC failure does.
  $effect(() => {
    const { dirty, labels } = store.closeGuardPayload();
    void updateCloseGuard(dirty, labels).catch((e: unknown) => {
      store.error = e as AppError;
    });
  });

  // Focus restoration for the New/Open discard-changes prompt (the WAI-ARIA
  // dialog pattern: "when it closes, focus returns to the element that
  // triggered it"). `DiscardPrompt` itself moves focus onto Cancel the instant
  // it opens, so capturing `document.activeElement` reactively off
  // `discardPromptOpen` would race that effect. Tracking real `focusin` events
  // instead sidesteps the race entirely: while the prompt is open every focus
  // move happens *inside* it (the rest of the shell is `inert`), so those
  // events are ignored and `lastFocusOutsideDialog` keeps whatever had focus
  // just before New/Open (or Ctrl+N/Ctrl+O) opened it — the button clicked, or
  // wherever the keyboard shortcut left focus.
  let lastFocusOutsideDialog: HTMLElement | null = null;
  function trackFocus(event: FocusEvent): void {
    if (store.discardPromptOpen) return;
    if (event.target instanceof HTMLElement) lastFocusOutsideDialog = event.target;
  }
  $effect(() => {
    if (store.discardPromptOpen) return;
    const toFocus = lastFocusOutsideDialog;
    // Cancel leaves the document exactly as it was, so the trigger is still
    // there; Discard/confirm may have navigated away (e.g. New resets to the
    // startup screen), in which case the old element is disconnected and
    // `.focus()` is skipped — the sensible fallback is to leave focus alone
    // rather than force it onto an arbitrary substitute.
    if (toFocus?.isConnected) toFocus.focus();
  });
</script>

<svelte:window onkeydown={handleShortcut} onfocusin={trackFocus} />

<!-- Everything interactive lives in the shell so a single `inert` can switch the
     whole app off while a native file dialog is open. The dialogs are parented to
     the window but are NOT input-modal on Linux (rfd has no modal flag, and tao's
     cross-platform Window has no `set_enabled`), so without this the user could
     keep editing the character behind an open Save/Open/Export dialog. `inert`
     drops focus and assistive-tech access; the overlay below swallows the clicks.
     Also inert while the discard-changes prompt is open (a plain HTML dialog with
     no native modality of its own): otherwise a keyboard user tabbing forward
     would walk through the whole live, visually-obscured app before ever
     reaching the prompt's own Cancel/Discard buttons. -->
<div
  class="app-shell"
  inert={store.busy || store.discardPromptOpen}
  aria-busy={store.busy}
  data-testid="app-shell"
>
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

    <!-- The strip is one line and ellipsizes a label too long for the room it has
         (app.css), so each tab carries its full label in `title` as well — the same
         Fluent string as the visible text, never a second wording. -->
    <div class="tabbar" role="tablist" tabindex="-1" onkeydown={onTabsKeydown}>
      {#each tabs as t (t.id)}
        <button
          type="button"
          role="tab"
          id="tab-{t.id}"
          class="tab"
          class:active={tab === t.id}
          aria-selected={tab === t.id}
          aria-controls="tabpanel-{t.id}"
          tabindex={tab === t.id ? 0 : -1}
          onclick={() => (tab = t.id)}
          data-testid="tab-{t.id}"
          title={store.t(t.key)}
        >
          {store.t(t.key)}
        </button>
      {/each}
    </div>

    <!-- `role="tabpanel"` lives on the inner div, not `<main>`: `<main>` is a
         non-interactive landmark element, and the linter (rightly) rejects
         assigning an interactive/widget role to one — the div carries the
         ARIA semantics, `<main>` keeps its plain landmark role. -->
    <main class="tab-content">
      <div
        class="tab-panel"
        role="tabpanel"
        id="tabpanel-{tab}"
        aria-labelledby="tab-{tab}"
        tabindex="0"
      >
        {#if tab === 'characteristics'}
          <CharacteristicPicker />
        {:else if tab === 'virtues_flaws'}
          <div class="vf-tab">
            <BalanceBar />
            <VirtueFlawTab />
          </div>
        {:else if tab === 'experience'}
          <!-- The same component the wizard's `experience` step mounts, so the two
               surfaces cannot drift (#28). It scrolls for the same reason the step
               does (`WizardStep`'s `scroll: true`): the panel is long, and it used to
               ride above the ability lists as an auto-height sibling, where the
               childhood Apply button was clipped away by `.tab-content`'s overflow
               with no scrollport to recover it. -->
          <div class="vf-tab">
            <XpBar />
            <div class="tab-scroll">
              <ExperienceStep />
            </div>
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
            <SpellBudgetBar />
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
        {:else if tab === 'personality_reputations'}
          <!-- The wizard's `personality_reputations` step is exactly these two
               sections in a `.character-details` panel, so the tab mounts the step's
               own composition rather than restating it. -->
          <div class="vf-tab">
            <div class="tab-scroll">
              <PersonalityReputationsStep />
            </div>
          </div>
        {:else if tab === 'aging'}
          <!-- `AgingPanel` is the whole aging surface, Longevity Ritual included, and
               is what the wizard's aging step mounts too. The step adds the age above
               it because no other guided surface offers one under flat funding; here
               the age stays on Details, beside the identity it belongs with. The
               `.character-details` wrapper is the grid the panel's
               `display: contents` children are laid out by (app.css), the same one
               the step provides. -->
          <div class="vf-tab">
            <div class="tab-scroll">
              <section class="panel character-details">
                <AgingPanel />
              </section>
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
      </div>
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
