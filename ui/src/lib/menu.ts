// The frontend half of the native application menu.
//
// Rust owns the menu's SHAPE — which sections exist, in which order, and where
// Settings sits on which desktop (`crates/arm-app/src/menu.rs`). This module
// owns its TEXT and its meaning: the labels, resolved from Fluent, and the map
// from a menu item's id to the store action it runs. That split is the same one
// `update_close_guard` already uses for the discard dialog — no user-facing
// string is authored in Rust.

/**
 * A document action the native menu offers.
 *
 * Also what its keyboard chord runs, and that is the same thing rather than a
 * second one: since C7 every shortcut is the menu item's own accelerator, so a
 * press arrives here as an ordinary activation of that item.
 */
export type DocumentAction = 'new' | 'open' | 'save' | 'saveAs' | 'export' | 'settings';

/**
 * The Tauri event Rust announces a chosen menu item on. Must match
 * `arm_app::menu::MENU_ACTION_EVENT`; the Rust test
 * `the_frontend_and_rust_agree_on_the_menu_contract` pins the pair.
 */
export const MENU_ACTION_EVENT = 'menu://action';

/**
 * Menu item id (`arm_app::menu::ACTION_IDS`) -> the store action it runs.
 * Nothing here reimplements a file operation: each action names the very store
 * method `runDocumentAction` dispatches to. Since C3c removed the toolbar and
 * C7 made every shortcut the menu item's accelerator, this table is the ONE
 * route these actions have — a chord and a click both arrive through it.
 */
export const MENU_ACTIONS: Record<string, DocumentAction> = {
  'menu.new': 'new',
  'menu.open': 'open',
  'menu.save': 'save',
  'menu.save-as': 'saveAs',
  'menu.export': 'export',
  'menu.settings': 'settings',
};

/** Localized menu text. Mirrors `arm_app::menu::MenuLabels` field for field. */
export interface MenuLabels {
  app: string;
  file: string;
  edit: string;
  window: string;
  new: string;
  open: string;
  save: string;
  saveAs: string;
  export: string;
  settings: string;
  quit: string;
  services: string;
  hide: string;
  hideOthers: string;
  showAll: string;
  undo: string;
  redo: string;
  cut: string;
  copy: string;
  paste: string;
  selectAll: string;
  minimize: string;
  fullscreen: string;
  closeWindow: string;
}

/** Whether each action may run. Mirrors `arm_app::menu::MenuFlags`. */
export type MenuFlags = Record<DocumentAction, boolean>;

/**
 * Resolve every menu label for the active language.
 *
 * The macOS application menu is titled with the application's own name, so it
 * reuses `app-title` rather than restating it under a second key.
 */
export function menuLabels(t: (key: string) => string): MenuLabels {
  return {
    app: t('app-title'),
    file: t('menu-file'),
    edit: t('menu-edit'),
    window: t('menu-window'),
    new: t('menu-new'),
    open: t('menu-open'),
    save: t('menu-save'),
    saveAs: t('menu-save-as'),
    export: t('menu-export'),
    settings: t('menu-settings'),
    quit: t('menu-quit'),
    services: t('menu-services'),
    hide: t('menu-hide'),
    hideOthers: t('menu-hide-others'),
    showAll: t('menu-show-all'),
    undo: t('menu-undo'),
    redo: t('menu-redo'),
    cut: t('menu-cut'),
    copy: t('menu-copy'),
    paste: t('menu-paste'),
    selectAll: t('menu-select-all'),
    minimize: t('menu-minimize'),
    fullscreen: t('menu-fullscreen'),
    closeWindow: t('menu-close-window'),
  };
}
