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

  // Why this test exists (guided-creation-review-2026-08 #20): the detail panels
  // were a CSS multi-column flow (`columns: 2`), which *flows* content between
  // columns — so any height change (one ticked Living Condition, one added aging-log
  // row) shifted the column break and made blocks MIGRATE to another column, a
  // whole-panel reflow. Grid auto-placement is order-stable under height changes:
  // each item owns its cell, so a block growing changes only row heights. The
  // difference is not observable by rendering (no stylesheet is attached under SSR,
  // and reflow needs a real layout engine), and reverting to `columns` would look
  // like a harmless simplification — so this is the mechanical guard.
  //
  // Anchored at the start of a line so only the element-level base rule can satisfy
  // it, not some descendant variant.
  it('lays the character-details panels out as a grid, not a multi-column flow', () => {
    const block = /^\.character-details\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/display:\s*grid;/);
    // auto-fit gives the 1/2/3-column response, so there is no breakpoint list.
    expect(block![1]).toMatch(/grid-template-columns:\s*repeat\(auto-fit,\s*minmax\(/);
    // Without this a short block stretches to its row's height.
    expect(block![1]).toMatch(/align-items:\s*start;/);
    // No `columns` anywhere on the class, in the base rule or in a media query —
    // the two layout models would fight.
    expect(appCss).not.toMatch(/^\s*columns:\s*\d/m);
  });

  // The aging log is the one block on the surface that grows without bound (one row
  // per aging roll, and a pre-play catch-up can owe 25), and the narrow column was
  // truncating its "effect" field. It gets a full-width row of its own with a bounded
  // scrollport. Vertical overflow only: WebKitGTK's overlay horizontal scrollbar
  // claims hit area without taking layout height, which has already cost this project
  // an unclickable control.
  it('gives the aging log a full-width row with its own bounded scrollport', () => {
    const block = /^\.character-details\s+\.aging-log-block\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/grid-column:\s*1\s*\/\s*-1;/);
    const scroll = /^\.character-details\s+\.aging-log-scroll\s*\{([^}]*)\}/m.exec(appCss);
    expect(scroll).not.toBeNull();
    expect(scroll![1]).toMatch(/max-height:/);
    expect(scroll![1]).toMatch(/overflow-y:\s*auto;/);
    expect(scroll![1]).not.toMatch(/overflow-x:/);
  });
});
