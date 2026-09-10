import { describe, expect, it } from 'vitest';

import { buildBundle, translate, type Lang } from './i18n';
import { MENU_ACTIONS, menuLabels, type DocumentAction, type MenuLabels } from './menu';

function labelsFor(lang: Lang): MenuLabels {
  const bundle = buildBundle(lang);
  return menuLabels((key) => translate(bundle, key));
}

// The native menu is the easiest surface in the app to ship in English by
// accident: its text is assembled in Rust, where no Fluent bundle exists, and
// no other gate looks at it. The frontend is therefore the only place that may
// author it — and every slot has to be a real translation, not the Fluent key
// falling through (`translate` returns the key when a message is missing, which
// is exactly the "raw slug rendered as a label" CLAUDE.md forbids).
describe('the native menu labels', () => {
  it.each(['en', 'de'] as const)('resolves every slot to real text (%s)', (lang) => {
    const labels = labelsFor(lang);

    expect(Object.keys(labels).length).toBeGreaterThan(0);
    for (const [slot, text] of Object.entries(labels)) {
      expect(typeof text, slot).toBe('string');
      expect(text.length, slot).toBeGreaterThan(0);
      // A key that fell through looks like `menu-save-as`; a real label does not.
      expect(text, `${lang} menu label ${slot} fell through to its Fluent key`).not.toMatch(
        /^[a-z][a-z0-9-]*$/,
      );
    }
  });

  it('translates the menu rather than repeating English in German', () => {
    const en = labelsFor('en');
    const de = labelsFor('de');

    expect(de.file).toBe('Datei');
    expect(de.save).toBe('Speichern');
    expect(de.saveAs).not.toBe(en.saveAs);
    expect(de.settings).toBe('Einstellungen…');
    expect(de.closeWindow).toBe('Fenster schließen');
  });
});

// Building the menu is the one thing `set_app_menu` can fail at, and it fails
// on the OS side where nothing else in this app does. It reports as its own
// `AppError` kind so the banner can say what actually went wrong instead of
// borrowing `error-io`'s "a file could not be read or written".
describe('the menu-construction failure', () => {
  it.each(['en', 'de'] as const)('has a message of its own (%s)', (lang) => {
    const text = translate(buildBundle(lang), 'error-menu');
    expect(text).not.toBe('error-menu');
    expect(text.length).toBeGreaterThan(0);
  });
});

// The ids are Rust's (`arm_app::menu::ACTION_IDS`); this map is the frontend's
// half of that contract, and the Rust test `the_frontend_and_rust_agree_on_the
// _menu_contract` pins the two together.
describe('the menu action map', () => {
  it('routes every menu id to a distinct document action', () => {
    const ids = Object.keys(MENU_ACTIONS);
    const actions = Object.values(MENU_ACTIONS);

    expect(ids).toEqual([
      'menu.new',
      'menu.open',
      'menu.save',
      'menu.save-as',
      'menu.export',
      'menu.settings',
    ]);
    expect(new Set(actions).size).toBe(actions.length);
    expect(actions).toEqual<DocumentAction[]>([
      'new',
      'open',
      'save',
      'saveAs',
      'export',
      'settings',
    ]);
  });
});
