import { beforeEach, describe, expect, it } from 'vitest';
import { render } from 'svelte/server';

import { store } from '../state.svelte';
import LanguageSelector from './LanguageSelector.svelte';

function html(): string {
  return render(LanguageSelector, { props: {} }).body;
}

beforeEach(() => {
  store.lang = 'en';
});

// S1 (full-audit i18n): the language `<select>` rendered the raw locale code
// ('en'/'de') as its visible option text — the exact "raw ID/slug as a
// user-facing label" violation CLAUDE.md bans everywhere else. Each option now
// goes through Fluent, and shows the language's own endonym (the convention
// users expect from a language picker) rather than a translation of the name
// into the currently active language.
describe('LanguageSelector options (S1)', () => {
  it('never renders the raw locale code as an option label', () => {
    const body = html();
    // Neither the bare 'en' nor 'de' token appears as option text.
    expect(body).not.toMatch(/<option[^>]*>en<\/option>/);
    expect(body).not.toMatch(/<option[^>]*>de<\/option>/);
  });

  it('shows each language as its own endonym, in English UI', () => {
    const body = html();
    expect(body).toMatch(/<option value="en"[^>]*>English<\/option>/);
    expect(body).toMatch(/<option value="de"[^>]*>Deutsch<\/option>/);
  });

  it('shows each language as its own endonym, in German UI too', () => {
    store.lang = 'de';
    const body = html();
    // The endonym convention means the labels do not change with the active
    // language — English stays "English", Deutsch stays "Deutsch".
    expect(body).toMatch(/<option value="en"[^>]*>English<\/option>/);
    expect(body).toMatch(/<option value="de"[^>]*>Deutsch<\/option>/);
  });
});

// G12 (full-audit test-adequacy): LanguageSelector had no test file beyond the
// S1 i18n fix above; the one behavior actually worth asserting on a component
// this small is that it reflects the store's active language back as the
// select's own value, so switching languages does not desync the picker from
// the app it controls.
describe('LanguageSelector reflects the active language', () => {
  it('marks English selected when the store language is en', () => {
    store.lang = 'en';
    const select = /<select[^>]*data-testid="language-select"[\s\S]*?<\/select>/.exec(html())![0];
    expect(select).toMatch(/<option value="en"(?:(?!<option)[\s\S])*?selected/);
  });

  it('marks Deutsch selected when the store language is de', () => {
    store.lang = 'de';
    const select = /<select[^>]*data-testid="language-select"[\s\S]*?<\/select>/.exec(html())![0];
    expect(select).toMatch(/<option value="de"(?:(?!<option)[\s\S])*?selected/);
  });
});
