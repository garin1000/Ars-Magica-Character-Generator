<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { store } from './lib/state.svelte';
  import { onMenuAction, setAppMenu, updateCloseGuard } from './lib/ipc';
  import { MENU_ACTIONS, menuLabels } from './lib/menu';
  import type { AppError } from './lib/types';
  import SettingsDialog from './lib/components/SettingsDialog.svelte';
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
  import ErrorBanner from './lib/components/ErrorBanner.svelte';
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
  // Experience, Abilities, the Hermetically-trained-only Arts and Spells,
  // Personality & Reputations, Aging, then the trained-only Magic Items, the
  // Order-only House tab, the mythic-companion-only Type tab, Equipment and
  // Totals — each gated on a resolved fact, never the type id, so any future
  // capable type gets them automatically.
  //
  // A2/D56 § 6: gated on `phases_in_force` (`EffectiveScores`), the engine's own
  // per-entity resolution of the profile's conditional `creation_phases` —
  // not a client-derived `isMagus`/`hasMythicType` boolean read off the bare
  // profile flags. The bare-flag reading used to bundle Arts/Spells/Possessions
  // (Hermetic training) and House (Order membership) under one flag, which is
  // wrong the instant a character is trained without being an Order member (or
  // the reverse) — row 20(c)'s conflation, fixed here at the tab-list layer by
  // reading each fact separately rather than patched around in the component.
  const phasesInForce = $derived(new Set(store.effective?.phases_in_force ?? []));
  // Hermetic training has no creation phase of its own to read: Arts and Spells
  // are both conditioned on it (`hermetically_trained`), so either's presence in
  // the resolved set already IS the fact. Possessions (magic items/Talisman)
  // keys on the identical condition and has no wizard phase of its own either
  // (it has never been a guided-wizard step, see the tabs-with-no-phase note
  // above) — reading the Arts phase's resolution for it is one fact read twice,
  // not a second client-side evaluator.
  const hermeticallyTrained = $derived(phasesInForce.has('arts'));
  const hasMythicType = $derived(phasesInForce.has('mythic_type'));
  // The Supernatural (Might) tab appears for a mythic-companion-capable type
  // (Devil Child, Nephilim) or once the character has an effective Might (any type
  // that took a Might Virtue such as Demonic Blood). Magi never have Might.
  const hasMight = $derived(
    !hermeticallyTrained && (hasMythicType || (store.effective?.might ?? null) !== null),
  );
  // Focus Power is a *Supernatural* Virtue (ArMDE:3895-3896) that grants no Might
  // and is open to every character type, magi included — so the tab must also
  // open on the Focus Power pool, or a companion or magus who bought the Virtue
  // would have nowhere to enter the power it gives him. Existing focus powers
  // count too, so a loaded save always has somewhere to edit them.
  const hasFocusPower = $derived(
    (store.effective?.focus_points_budget ?? 0) > 0 || (store.entity.focus_powers ?? []).length > 0,
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
    ...(hermeticallyTrained
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
    ...(hermeticallyTrained ? [{ id: 'possessions' as Tab, key: 'tab-possessions' }] : []),
    ...(phasesInForce.has('house_specialisation')
      ? [{ id: 'house_specialisation' as Tab, key: 'tab-house-specialisation' }]
      : []),
    ...(hasMythicType ? [{ id: 'mythic_type' as Tab, key: 'tab-mythic-type' }] : []),
    ...(hasMight || hasFocusPower ? [{ id: 'supernatural' as Tab, key: 'tab-supernatural' }] : []),
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
  /**
   * Put the element's full text in `title`, but ONLY while the element is
   * actually clipping it.
   *
   * Sabine 14: the strip is one line and ellipsizes a label too long for the room
   * it has (app.css), so a clipped tab needs its full label somewhere readable —
   * but an unconditional `title` is a real cost paid at every width. A `<button>`
   * with no `aria-label` takes its accessible NAME from its content, which makes
   * `title` its accessible DESCRIPTION, so assistive tech reads the same string
   * twice on every tab stop. Measuring means the description exists only in the
   * state that ever justified it: German's thirteen-tab magus set at the 900px
   * the window is resizable down to (tauri.conf.json).
   *
   * `ResizeObserver` and not a one-shot read, for the same reason
   * `actions.ts::reserveTagSpace` uses one: the strip re-lays-out on every window
   * resize, and a measurement taken once at mount would be a stale answer the
   * moment the user drags the window edge. `update` re-measures when the label
   * itself changes, which is what a UI-language switch does.
   *
   * Local to this component rather than added to `actions.ts`: the tab strip is
   * its only consumer (YAGNI), and it belongs with the strip it measures.
   */
  function titleWhenClipped(node: HTMLElement, label: string) {
    let current = label;
    const apply = () => {
      // `scrollWidth` is the content's full width, `clientWidth` the box it got;
      // they differ exactly when `text-overflow: ellipsis` has something to cut.
      if (node.scrollWidth > node.clientWidth) node.setAttribute('title', current);
      else node.removeAttribute('title');
    };
    const observer = new ResizeObserver(apply);
    observer.observe(node);
    apply();
    return {
      update(next: string) {
        current = next;
        apply();
      },
      destroy() {
        observer.disconnect();
      },
    };
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

    // A menu click is a native event: Rust receives it and forwards the item's
    // id, because the document — and therefore what the id means — lives here.
    // The unsubscribe arrives asynchronously, so teardown may run before it
    // does; `unlisten` is then set on a listener nobody holds any more, which
    // is why it is also called from the `.then` when the component is already
    // gone.
    let unlisten: (() => void) | undefined;
    let torndown = false;
    void onMenuAction(runMenuAction).then((stop) => {
      if (torndown) stop();
      else unlisten = stop;
    });
    return () => {
      torndown = true;
      unlisten?.();
    };
  });

  /** Route a native menu item to the store action it names, gate included. */
  function runMenuAction(id: string): void {
    const action = MENU_ACTIONS[id];
    // An id this build does not know (a renamed item, a stale menu) is ignored
    // rather than thrown: the menu is chrome, and a throw here would surface
    // as an unhandled rejection with nothing the user could do about it.
    if (action === undefined) return;
    void store.runDocumentAction(action);
  }

  // Install the native menu, and REINSTALL it whenever its text or its enabled
  // state changes. Reading `store.t` here is what subscribes this effect to the
  // active language: a menu built once at startup would keep that language for
  // the life of the session, behind a UI the user had already switched. The
  // flags come from the store's single document-action predicate, which a
  // native menu can consult no other way — it is neither a button carrying
  // `disabled` nor a window key event.
  //
  // A rejection is surfaced rather than swallowed: a menu that simply never
  // appeared reads as "this app has no menu", not as "something went wrong",
  // and `AppError::Menu` exists precisely so the banner can tell the user which
  // of the two it is.
  $effect(() => {
    void setAppMenu(menuLabels(store.t), store.menuFlags()).catch((e: unknown) => {
      store.error = e as AppError;
    });
  });

  // Keep `<html lang>` in sync with the active UI language: index.html's
  // static `lang="en"` is only the pre-mount fallback, and assistive tech uses
  // this attribute to pick pronunciation/voice rules, so it must track a
  // runtime language switch rather than staying English forever (S3).
  $effect(() => {
    document.documentElement.lang = store.lang;
  });

  // Name the RESOLVED palette on <html>. `app.css` ships a dark `:root` and a
  // light `:root[data-theme='light']`, and this attribute is the entire switch
  // between them — `color-scheme` included, which matters more than it looks:
  // the stylesheet declares no `:focus-visible` rule and styles no scrollbar, so
  // that one keyword is what tells the engine which focus ring, which scrollbars
  // and which native <select> popup to draw.
  //
  // The decision is made HERE rather than by a `prefers-color-scheme` media
  // query so that `auto` and an explicit choice take the same path and the light
  // palette exists in exactly one place. A media query would have needed a
  // second copy of it, and two copies drift.
  $effect(() => {
    document.documentElement.dataset.theme = store.resolvedTheme;
  });

  // …and `auto` means LIVE. Reading the OS preference once at startup would
  // leave a user whose desktop flips to light at dusk sitting in a dark app
  // until they restarted it, which is the one thing `auto` promises not to do.
  // The effect returns the store's own teardown so Svelte unsubscribes on
  // destroy: a `MediaQueryList` outlives whatever registered a listener on it,
  // so a leaked one keeps writing into the store for the life of the webview.
  $effect(() => store.watchSystemTheme());

  // Keep the document title localized rather than hardcoded in HTML. Once a file
  // is being tracked, show "name — app" (with an ASCII dirty marker for unsaved
  // edits) via a parametrized Fluent key — never string-composed here.
  //
  // S4 (full-audit UX): `document.title` alone never reached the native OS
  // window's own title bar — a Tauri window's chrome does not mirror the HTML
  // document title, only `getCurrentWindow().setTitle(...)` does. So the same
  // computed string now goes to both. A rejection here is swallowed rather than
  // surfaced through `store.error` like the close guard below: an OS title-bar
  // update failing is cosmetic, not the safety-critical unsaved-changes
  // guarantee that guard exists for.
  $effect(() => {
    const name = store.currentFileName;
    const title =
      name === null
        ? store.t('app-title')
        : store.t(store.dirty ? 'app-title-document-dirty' : 'app-title-document', {
            name,
            app: store.t('app-title'),
          });
    document.title = title;
    void getCurrentWindow()
      .setTitle(title)
      .catch(() => {});
  });

  // NO KEYBOARD SHORTCUT HANDLER LIVES HERE (C7), and that is the design.
  //
  // Every document chord — Ctrl/Cmd+N/O/S, Shift+S for Save As, Shift+E for
  // Export, Ctrl/Cmd+, for Settings — is declared as the native menu item's
  // accelerator and nowhere else (`accelerator_for`,
  // `crates/arm-app/src/menu.rs`). The OS binds it, the OS dispatches it, and
  // the OS draws it beside the label, which is what makes the shortcuts
  // discoverable in the first place. The id then arrives on `menu://action`
  // and runs the very `store.runDocumentAction` this file's deleted
  // `handleShortcut` used to call, so nothing about the route past dispatch
  // changed.
  //
  // A handler here would be a SECOND owner of the same chord. On GTK the accel
  // group is matched on the toplevel window before the key reaches the focused
  // widget, so both would fire and one Ctrl+N on a dirty document would raise
  // two discard prompts — the hazard that made C3a and C3c ship the menu with
  // no accelerators at all. `the document chords belong to the native menu,
  // not the webview` (`App.client.test.ts`) is what keeps this file from
  // growing one back.
  //
  // The availability gate is unchanged and still single: `MenuFlags` carries
  // `store.documentActionEnabled` to the OS, and GTK refuses to activate an
  // insensitive item's accelerator, so a withheld action is withheld from the
  // keyboard too.

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

  // Focus restoration for the New/Open discard confirmation (the WAI-ARIA
  // dialog pattern: "when it closes, focus returns to the element that
  // triggered it"). The in-app fallback prompt moves focus onto Cancel the
  // instant it opens, so capturing `document.activeElement` reactively off the
  // open flag would race that effect. Tracking real `focusin` events instead
  // sidesteps the race entirely: while a confirmation is pending every focus
  // move happens *inside* it (the rest of the shell is `inert`), so those
  // events are ignored and `lastFocusOutsideDialog` keeps whatever had focus
  // just before New/Open (or Ctrl+N/Ctrl+O) raised it — the button clicked, or
  // wherever the keyboard shortcut left focus.
  //
  // Keyed on `discardConfirmPending`, not on the in-app modal: since C3b the
  // confirmation is normally the NATIVE dialog, which has no in-app flag at all,
  // and focus must come back the same way once that one closes.
  //
  // `settingsOpen` joins the guard for the same reason: that dialog restores focus
  // itself, and letting its controls be recorded here would leave this effect
  // pointing at a control that no longer exists the next time a discard is
  // confirmed.
  let lastFocusOutsideDialog: HTMLElement | null = null;
  function trackFocus(event: FocusEvent): void {
    if (store.discardConfirmPending || store.settingsOpen) return;
    if (event.target instanceof HTMLElement) lastFocusOutsideDialog = event.target;
  }
  $effect(() => {
    if (store.discardConfirmPending) return;
    const toFocus = lastFocusOutsideDialog;
    // Cancel leaves the document exactly as it was, so the trigger is still
    // there; Discard/confirm may have navigated away (e.g. New resets to the
    // startup screen), in which case the old element is disconnected and
    // `.focus()` is skipped — the sensible fallback is to leave focus alone
    // rather than force it onto an arbitrary substitute.
    if (toFocus?.isConnected) toFocus.focus();
  });
</script>

<svelte:window onfocusin={trackFocus} />

<!-- Everything interactive lives in the shell so a single `inert` can switch the
     whole app off while a native file dialog is open. The dialogs are parented to
     the window but are NOT input-modal on Linux (rfd has no modal flag, and tao's
     cross-platform Window has no `set_enabled`), so without this the user could
     keep editing the character behind an open Save/Open/Export dialog. `inert`
     drops focus and assistive-tech access; the overlay below swallows the clicks.
     Also inert while a discard confirmation is pending — for the native dialog
     because it is no more input-modal than the file dialogs above, and for the
     in-app fallback because otherwise a keyboard user tabbing forward would walk
     through the whole live, visually-obscured app before ever reaching the
     prompt's own Cancel/Discard buttons. The settings dialog gets the same
     containment for that last reason — it has a real focus trap of its own, and
     `inert` is the belt to its braces: the trap closes the ring, while `inert`
     also takes the shell out of the accessibility tree so nothing behind the
     dialog is announced.
     `ConfirmPrompt` is deliberately NOT in this predicate, and cannot be: unlike
     the two modals above it renders *inside* the shell (`FamiliarPanel.svelte`,
     `TalismanPanel.svelte` mount it beside the control it guards), so switching the
     shell off would switch the prompt off with it and leave a confirmation nobody
     could answer — exactly the trade the comment above `<SettingsDialog />` states.
     It closes its own ring instead, at the window rather than at the dialog, which
     buys the same containment without needing the shell inert (Sabine 2). -->
<div
  class="app-shell"
  inert={store.busy || store.discardConfirmPending || store.settingsOpen}
  aria-busy={store.busy}
  data-testid="app-shell"
>
  <!-- ONE ROW (C5). The header used to be two: a `.brand` wrapper declaring
       `flex: 1 1 100%` pushed the controls onto a line of their own at every
       width, and it carried 1rem of padding around a 2.25rem logo and an `<h1>`
       repeating the app's name. That cost 97px of a window whose minimum height
       is 900, on every screen, to say nothing the title bar was not already
       saying. -->
  <header class="app-header">
    <!-- THE APP'S NAME, ANNOUNCED BUT NOT DRAWN. The visible `<h1>` is gone — the
         effect above puts the very same `app-title` in the OS window title, and
         C3a's menu puts it in the macOS application menu, so a third rendering
         inside the window was duplicated chrome.
         It is `.sr-only` rather than deleted, because deleting it would take the
         document's ONLY top-level heading with it, and with it the app's only
         accessible name in-window: the logo beside it is the Ars Magica Open
         License mark and its alt says exactly that, which is the right name for
         what the image is and the wrong one for what the application is. -->
    <h1 class="sr-only">{store.t('app-title')}</h1>
    <!-- Active save's file name + ASCII dirty marker, on-screen (not only in the
         OS window title). Reuses the derived currentFileName/dirty state; a null
         file name means the document has never been saved. Hidden on the startup
         screen, where it would report an unsaved document that does not exist.
         First on the row, where the eye starts — it is the only on-screen surface
         for unsaved state since C3c retired the toolbar. -->
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
    <!-- The app's own preferences — language, appearance, validation strictness —
         live behind ONE button now (C4), on every screen, because none of the
         three is about a character. The language dropdown and the Validation
         dropdown used to sit here loose; they are the settings dialog's fields
         now, unchanged, so neither gained a second definition.

         New/Open/Save/Save As/Export are NOT here. C3c retired the toolbar that
         held them: they are the native menu's (C3a) and the keyboard's, and a
         third rendering of the same five actions was a third copy of the gate
         `store.documentActionEnabled` exists to be the only answer to. What
         stayed is what was never a document action — the way into the settings,
         the entry into the guided flow, and the error banner.

         `documentActionEnabled('settings')` is what disables it, not `store.busy`
         directly: the menu item, the button and the dispatcher all have to answer
         the same question, and the store is where that answer lives. -->
    <div class="controls">
      <button
        type="button"
        onclick={() => store.runDocumentAction('settings')}
        disabled={!store.documentActionEnabled('settings')}
        data-testid="settings-button"
      >
        {store.t('settings-open')}
      </button>
      {#if store.view !== 'start'}
        <div class="header-actions">
          <!-- Take the character on screen into the guided flow (#31). Offered
               only from the editor — the wizard is where it leads — and only for
               a type the loaded ruleset declares a profile for, since the
               profile's phases ARE the rail. -->
          {#if store.view === 'editor' && store.canEnterWizard}
            <button
              type="button"
              onclick={() => store.enterWizard()}
              disabled={store.busy}
              data-testid="wizard-continue-button"
            >
              {store.t('action-continue-in-wizard')}
            </button>
          {/if}
          <ErrorBanner />
        </div>
      {/if}
    </div>
    <!-- The Ars Magica Open License attribution mark, in the upper right and last
         in reading order: it names neither the app nor anything the user acts on,
         so it comes after the status and after every control rather than leading
         the header the way it used to. Small enough that the controls, not the
         mark, set the row's height (app.css). -->
    <img class="app-logo" src={logoUrl} alt={store.t('app-logo-alt')} />
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
         (app.css), so a tab carries its full label in `title` as well — the same
         Fluent string as the visible text, never a second wording. `title` STAYS
         even though the strip's type and padding were re-budgeted so the labels fit
         in full at the default window: the window is resizable down to 900px
         (tauri.conf.json), where German's thirteen-tab magus set truncates again.
         But it is attached ONLY WHILE THE LABEL IS ACTUALLY CLIPPED (Sabine 14) —
         see `titleWhenClipped` above for why an unconditional one is an a11y cost
         at every other width. -->

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
          use:titleWhenClipped={store.t(t.key)}
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

<!-- The FALLBACK discard confirmation. Since C3b every discard — New, Open,
     close, quit — is confirmed by the native dialog Rust owns; this renders only
     where no native answer arrives: the `e2e-testing` build, which declines so
     WebDriver has something it can click, and an IPC failure, where treating a
     broken bridge as "yes, discard" would silently destroy unsaved work.
     Outside the shell: it is the one modal the user must still be able to answer
     (it never overlaps a native dialog — `busy` is false while it is open). -->
<DiscardPrompt />

<!-- The app's preferences (C4). Outside the shell for the same reason the prompt
     is: the shell goes `inert` while it is open, and a dialog inside an inert
     subtree could not be answered. It owns its own focus trap and its own focus
     restoration, so it needs nothing from the block above. -->
<SettingsDialog />
