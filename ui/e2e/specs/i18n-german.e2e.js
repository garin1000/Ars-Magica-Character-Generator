// End-to-end: German localization. Switching the UI to German must re-render
// real German rules text AND chrome — and a spell tooltip must never be empty,
// even where a German description is not yet translated (the app falls back to
// English). Guards the class of bug where German-only breakage shipped green
// because no test ran the app in German. Drives the real binary.

import { $, browser, expect } from '@wdio/globals';

const LANG_SELECT = '[data-testid="language-select"]';
const TYPE_SELECT = '[data-testid="type-select"]';
const SPELLS_TAB = '[data-testid="tab-spells"]';

// Fluent wraps interpolated values in Unicode bidi isolation marks; strip them.
function clean(text) {
  return text.replace(/[⁦-⁩]/g, '');
}

describe('German localization', () => {
  it('renders German chrome and a non-empty German spell tooltip', async () => {
    // Wait for the ruleset to load (the language selector is always mounted).
    await $(LANG_SELECT).waitForExist({ timeout: 30000 });

    // Switch to German and confirm the language actually changed.
    await $(LANG_SELECT).selectByAttribute('value', 'de');
    await browser.waitUntil(async () => (await $(LANG_SELECT).getValue()) === 'de', {
      timeout: 5000,
      timeoutMsg: 'language should switch to German',
    });

    // A magus shows the Spells tab; its "Available" region title must render in
    // German ("Verfügbar"), proving UI chrome re-localizes.
    await $(TYPE_SELECT).selectByAttribute('value', 'magus');
    await $(SPELLS_TAB).waitForExist({ timeout: 10000 });
    await $(SPELLS_TAB).click();
    const available = await $('[data-testid="available-title"]');
    await available.waitForExist({ timeout: 5000 });
    // `.region-title` uppercases via CSS for display, so compare case-insensitively.
    expect(
      clean(await available.getText())
        .trim()
        .toLowerCase(),
    ).toBe('verfügbar');

    // Hover a spell row and assert the description tooltip has text (German where
    // translated, English fallback otherwise) — never the empty tooltip bug.
    const row = await $('[data-testid="add-spell.pilum_of_fire"]');
    await row.waitForExist({ timeout: 10000 });
    await browser.execute((el) => {
      el.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
    }, row);
    const pop = await $('.tooltip-pop .tooltip-text');
    await pop.waitForExist({ timeout: 5000 });
    expect((await pop.getText()).trim().length).toBeGreaterThan(0);
  });
});
