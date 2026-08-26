import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// The stylesheet itself is the unit under test, so it is read as text.
//
// NOT via `import css from './app.css?raw'`: vitest stubs CSS modules to an
// empty string unless `test.css` is enabled, and its check keys on the `.css`
// extension *regardless of the query*, so `?raw` yields `''` and every
// assertion below would fail vacuously. `readFileSync` is the mechanism
// ValidationPanel.test.ts already uses for the same reason.
const appCss = readFileSync(fileURLToPath(new URL('./app.css', import.meta.url)), 'utf-8');

// A base typography reset is not observable by rendering: `render()` from
// `svelte/server` emits markup with no stylesheet attached, and even a mounted
// component in `happy-dom` would need the sheet loaded and computed styles read
// — disproportionate for a static selector.
describe('app.css', () => {
  // Why this test exists (guided-creation-review-2026-08 #23 and #3): app.css
  // had no base rule for `p` or headings, so any unstyled `<p>`/`<h3>` kept the
  // UA `margin: 1em 0`. Flex and grid gaps do NOT collapse margins, so inside a
  // `.detail-section` (a flex column, `gap: 0.4rem`) two single-line paragraphs
  // sat 1em + 0.4rem + 1em ≈ 2.4rem apart where 0.4rem was intended. The reset
  // is app-wide and easy to mistake for dead code, so this is the mechanical
  // guard against a later "cleanup" deleting it.
  //
  // Anchored at the start of a line so a descendant variant (`.panel p, …`)
  // cannot satisfy it: only an element-level base rule can.
  it('declares a base margin reset for p and headings', () => {
    expect(appCss).toMatch(/^p,\s*h1,\s*h2,\s*h3,\s*h4,\s*h5,\s*h6\s*\{[^}]*margin:\s*0;/m);
  });
});
