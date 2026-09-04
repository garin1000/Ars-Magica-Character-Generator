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

  // manual-testing-findings-2026-09-03 #20/#22: on the aging surface the auto-fit
  // grid put SHORT blocks beside the TALL roll calculator, and `align-items: start`
  // then left the short block's cell with the calculator's height under it — the
  // "screen-third of empty space above the Aging heading". A grid row can only be as
  // tall as its tallest item, so the only construction that cannot produce that gap
  // is one item per row. Every aging block therefore takes the whole row, and the
  // horizontal space is reclaimed INSIDE each block (see the two rules below)
  // instead of between them. `> *` rather than a list of block classes so a block
  // added later cannot silently opt out and reintroduce the pairing.
  it('gives every aging stage a full-width row of its own', () => {
    const rule =
      /\.character-details \.aging-panel > \*,\s*\.character-details \.aging-record > \*\s*\{([^}]*)\}/.exec(
        appCss,
      );
    expect(rule).not.toBeNull();
    expect(rule![1]).toMatch(/grid-column:\s*1\s*\/\s*-1;/);
  });

  // A full-width stage would stretch a `<select>` or a text input across the whole
  // panel, which is neither readable nor pointable. The controls keep a measure.
  it('keeps a control on a full-width aging stage to a readable measure', () => {
    const rule =
      /\.character-details \.aging-panel \.field,\s*\.character-details \.aging-record \.field\s*\{([^}]*)\}/.exec(
        appCss,
      );
    expect(rule).not.toBeNull();
    expect(rule![1]).toMatch(/max-width:/);
  });

  // The accumulated record (apparent age, Decrepitude, its narrative, the eight
  // Aging Points) used to be four separate grid items scattered around the tall
  // calculator — half of the "band of columns" complaint. They are one stage now,
  // and that stage spends its width on its own fields rather than on a neighbour.
  it('lays the accumulated aging record out across its own stage width', () => {
    const rule = /^\.character-details \.aging-state\s*\{([^}]*)\}/m.exec(appCss);
    expect(rule).not.toBeNull();
    expect(rule![1]).toMatch(/display:\s*grid;/);
    expect(rule![1]).toMatch(/grid-template-columns:\s*repeat\(auto-fit,\s*minmax\(/);
  });

  // The Living Conditions checklist is ten-plus single-line rows. Stacked in one
  // column on a full-width stage it is the tallest thing between the top of the
  // surface and the log, which is exactly what #24/#25 asked to shorten. Flowed into
  // as many readable columns as the width allows it is a quarter of the height, and
  // `auto-fill` collapses to one column on a narrow window with no breakpoint.
  it('flows the Living Conditions checklist into as many columns as fit', () => {
    const rule = /^\.living-conditions-list\s*\{([^}]*)\}/m.exec(appCss);
    expect(rule).not.toBeNull();
    expect(rule![1]).toMatch(/display:\s*grid;/);
    expect(rule![1]).toMatch(/grid-template-columns:\s*repeat\(auto-fill,\s*minmax\(/);
  });

  // #26: the log grows one row per aging year and a magus can owe forty. The bound
  // is the log's OWN property, not something a layout class lends it — qualifying it
  // with `.character-details` made a correctness guarantee (nothing may push the rest
  // of the surface off screen) conditional on where the panel happens to be mounted,
  // and `AgingPanel`'s own comment contemplates a mount outside that wrapper.
  // Vertical overflow only: WebKitGTK's overlay horizontal scrollbar claims hit area
  // without taking layout height, which has already cost this project an unclickable
  // control.
  it('bounds the aging log scrollport wherever the panel is mounted', () => {
    const scroll = /^\.aging-log-scroll\s*\{([^}]*)\}/m.exec(appCss);
    expect(scroll).not.toBeNull();
    expect(scroll![1]).toMatch(/max-height:/);
    expect(scroll![1]).toMatch(/overflow-y:\s*auto;/);
    expect(scroll![1]).not.toMatch(/overflow-x:/);
    expect(appCss).not.toMatch(/\.character-details\s+\.aging-log-scroll/);
  });

  // guided-creation-review-2026-08 #16 (HIGH — the user had to "change them
  // blindfold"): every budget bar is mounted as an ordinary flow child of `.vf-tab`,
  // whose scrolling ancestor is `.tab-content`. So a step tall enough to scroll — the
  // ordinary case, since `.region-row` and `.list-scroll` carry min-height FLOORS
  // that no shrinking removes — carried the bar off the top of the screen while the
  // player spent against it.
  //
  // Sticky BEHAVIOUR cannot be asserted here: `render` from `svelte/server` attaches
  // no stylesheet, and happy-dom does no layout, so no computed position or scroll
  // offset exists to read. The stylesheet contract is the honest unit-level guard;
  // the behaviour is verified by the e2e scroll assertion in `abilities.e2e.js`.
  it('pins every budget bar to the top of its scrollport with an opaque background', () => {
    // ONE rule for all three bars (`.xp-summary` is XpBar and SpellBudgetBar,
    // `.balance` is BalanceBar), so a fourth bar cannot be added half-fixed and the
    // wizard's five mounts cannot drift from the editor's five.
    const block = /^\.xp-summary,\s*\n\.balance\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/position:\s*sticky;/);
    expect(block![1]).toMatch(/top:\s*0;/);
    // A background is not decoration: without one the rows scroll THROUGH the bar and
    // both are unreadable. It must resolve to an opaque colour, never `transparent`.
    const background = /background:\s*([^;]+);/.exec(block![1]);
    expect(background).not.toBeNull();
    expect(background![1].trim()).toBe('var(--bg)');
    // Sticky alone does not raise the bar above its later siblings: a positioned
    // element without a z-index still paints under content that comes after it in
    // the DOM, which is exactly the rows this bar has to cover.
    expect(block![1]).toMatch(/z-index:\s*[1-9]/);
  });

  // The other half of #16, and the half that actually decides whether sticky does
  // anything: a sticky box is confined to its CONTAINING BLOCK, which for every bar
  // is `.vf-tab`. While the overflow belonged to the ancestor `.tab-content` this box
  // was flex-shrunk to the tab area's height — measured at 0px against 538px of
  // content — so the bar had no travel and the sticky rule above was a silent no-op.
  // Owning the scroll here makes the box both scrollport and containing block, whose
  // constraint rectangle is the whole scrollable area.
  it('gives the step body the scrollport its sticky bar needs, with the full gutter pair', () => {
    const block = /^\.vf-tab\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/overflow-y:\s*auto;/);
    // Vertical only: WebKitGTK's overlay HORIZONTAL scrollbar claims hit area along
    // the bottom edge without taking layout height.
    expect(block![1]).toMatch(/overflow-x:\s*hidden;/);
    // BOTH halves of the gutter pair. `scrollbar-gutter` alone is a no-op in the
    // shipped WebKitGTK, whose overlay scrollbar takes hit area without taking
    // layout width — that is how an aging-log remove button became unclickable.
    expect(block![1]).toMatch(/scrollbar-gutter:\s*stable;/);
    expect(block![1]).toMatch(/padding-right:/);
    // NOT `min-height: min-content`, which was tried and measured wrong: it grew the
    // box to the full un-scrolled Available list (2743px) and collapsed every inner
    // `.list-scroll`. The box must stay shrinkable and scroll instead.
    expect(block![1]).toMatch(/min-height:\s*0;/);
  });

  // guided-creation-review-2026-08, cross-cutting theme 1: an element that enters or
  // leaves the FLOW on first interaction moves the control the user is mid-click on.
  // The house rule is to reserve the space instead, and this is the one utility that
  // does it. `display: none` and unmounting both fail the rule; `.sr-only` is the
  // opposite trade (announced, no space), so neither may be substituted here.
  it('hides an element without taking its space out of the flow', () => {
    const block = /^\.hidden-reserved\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/visibility:\s*hidden;/);
    // The whole point is that the box survives, so nothing here may remove it.
    expect(block![1]).not.toMatch(/display:\s*none/);
    expect(block![1]).not.toMatch(/position:\s*absolute/);
  });

  // guided-creation-review-2026-08 #17: the mastery abilities picker is much wider
  // than the score spinner, and as the spinner's sibling inside a content-width
  // column it widened that column the moment mastery reached 1 — the elastic
  // `.item-name` gave ground and the spinner slid sideways on a repeated-click
  // control. `.spell-list li` is a wrap flex container, so a 100% basis puts the
  // picker on a line of its own without touching the controls line's widths.
  it('gives the mastery abilities a full-width wrap line of their own', () => {
    const block = /^\.mastery-abilities\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/flex-basis:\s*100%;/);

    // The rejected fix, and the reason this is a contract test: a `min-width` floor
    // on the controls column would have stopped the shift by indenting EVERY
    // unmastered row permanently. The column wrapper it would sit on is gone
    // altogether, so the stylesheet must not name it again.
    expect(appCss).not.toContain('spell-mastery-block');
  });

  // guided-creation-review-2026-08 #27: the panel is mounted as a direct child of the
  // editor's `.tab-panel` (which centres its children) and, in the wizard, inside
  // `.vf-tab` (which is `width: 100%` and stretches them). Centring the panel ITSELF
  // makes the two mounts agree by construction, rather than by what happens to wrap
  // it. Auto inline margins centre it in both a flex column and ordinary flow.
  // guided-creation-review-2026-08 #6: `ValidationPanel.svelte` emits the bare
  // severity as a class (`<li class="issue error">`), and app.css also carried a
  // standalone one-word `.error { color; font-size }` rule written for BANNER text
  // (StartScreen's load failure). Both matched the row, and since `.issue` set
  // neither property, the banner rule won BY DEFAULT — error rows rendered smaller
  // and red-tinted while warning rows, having no `.warning` twin, did not. Severity
  // is meant to live in the left border, the background tint and the uppercase
  // badge, all of which are per-severity by design.
  it('gives every issue row its own typography, whatever its severity', () => {
    const block = /^\.issue\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    // Declared HERE, so no same-specificity rule elsewhere can supply them.
    expect(block![1]).toMatch(/font-size:/);
    expect(block![1]).toMatch(/color:/);
    // The per-severity rules stay colour/tint only — adding typography to either
    // would reintroduce the size difference from the other direction.
    const error = /^\.issue\.error\s*\{([^}]*)\}/m.exec(appCss);
    const warning = /^\.issue\.warning\s*\{([^}]*)\}/m.exec(appCss);
    expect(error).not.toBeNull();
    expect(warning).not.toBeNull();
    for (const severity of [error![1], warning![1]]) {
      expect(severity).not.toMatch(/font-size:/);
      // `border-left-color` is the colour channel and is expected; a plain
      // `color:` (the text colour) is what must not differ between severities.
      expect(severity).not.toMatch(/[^-]color:/);
    }
  });

  // The other half of #6: a bare one-word class carrying typography will keep
  // colliding with any severity-named class. The banner rule must be scoped to the
  // banners it was written for (or renamed), so no `.error`/`.warning` word class
  // can pick it up by accident again.
  it('never carries typography on a bare one-word severity class', () => {
    expect(appCss).not.toMatch(/^\.error\s*\{/m);
    expect(appCss).not.toMatch(/^\.warning\s*\{/m);
  });

  // guided-creation-review-2026-08 #8: the row separator was a `border-bottom` with
  // a `:last-child { border-bottom: none }` suppression. `:last-child` is scoped per
  // PARENT, and `SelectionList.svelte` opens a new `<ul>` per group — so the
  // suppression fired once per group instead of once per list. A HEADER-LESS group
  // (Spells' unknown Te/Fo bucket, Equipment's unclassified bucket) therefore butted
  // straight against its neighbour with no divider at all, because a header-less
  // group has no `<h3 class="category">` to draw its own boundary.
  //
  // The separator is now a `border-top` on rows after the first, which needs no
  // last-child exception, plus a boundary rule on adjacent lists — `ul + ul` matches
  // only when nothing sits between the two lists, i.e. exactly the header-less case.
  it('separates list rows without a per-parent last-child exception', () => {
    // No list may suppress its separator via `:last-child` again — that selector is
    // the bug, not the fix.
    expect(appCss).not.toMatch(/li:last-child\s*,?\s*(\n[^{]*)?\{[^}]*border-bottom:\s*none/);
    for (const list of ['item-list', 'selection-list', 'equipment-list']) {
      // One selector per line, ending in a comma or opening the block.
      const rows = new RegExp(`^\\.${list} li \\+ li(,| \\{)$`, 'm');
      expect(appCss).toMatch(rows);
    }
  });

  it('draws a boundary between two adjacent header-less lists', () => {
    // One rule for every grouped list, so a fourth list class cannot be added with
    // the boundary silently missing.
    const block =
      /^\.selection-list \+ \.selection-list,\n\.spell-list \+ \.spell-list,\n\.equipment-list \+ \.equipment-list\s*\{([^}]*)\}/m.exec(
        appCss,
      );
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/border-top:/);
  });

  // Finding #18: a source row that cannot be taken used to be greyed out; after the
  // switch from the native `disabled` attribute to `aria-disabled` (deliberate — the
  // row must stay focusable so its "why" tooltip stays reachable), the generic
  // `button:disabled { opacity }` stopped matching and the ONLY remaining cue was the
  // "+" glyph going from `--accent` (#d0ac63) to `--muted` (#c3bfde). `--muted` sits
  // right next to the normal ink `--ink` (#f3f1fb), so a blocked row was practically
  // indistinguishable from a takeable one — and a hue swap alone is a WCAG 1.4.1
  // failure regardless of how far apart the two hues are.
  //
  // The treatment is a luminance drop, not a hue change, and it is asserted here
  // because a stylesheet rule with no visible owner is exactly what a later cleanup
  // deletes — as the dead `.pick-row:disabled .pick-plus` rule this replaces shows.
  it('dims a blocked source row rather than only recolouring its glyph', () => {
    const dimmed = /^\.pick-row\[aria-disabled='true'\] > \*\s*\{([^}]*)\}/m.exec(appCss);
    expect(dimmed).not.toBeNull();
    const opacity = /opacity:\s*([\d.]+);/.exec(dimmed![1]);
    expect(opacity).not.toBeNull();
    // Low enough to read as "off" at a glance, high enough that `--ink` composited
    // over `--panel` still clears 4.5:1 (at 0.55 it is ~5.1:1; at 0.45 it is ~4.0:1).
    expect(Number(opacity![1])).toBeGreaterThanOrEqual(0.5);
    expect(Number(opacity![1])).toBeLessThanOrEqual(0.6);
  });

  // The dim goes on the row's CONTENT, not on the button box. `opacity` fades an
  // element's outline along with its content, and the whole reason these rows use
  // `aria-disabled` instead of `disabled` is that they stay focusable — fading the
  // focus ring of the one control a keyboard user must reach to learn WHY the row is
  // blocked would give back exactly what `aria-disabled` bought.
  it('keeps the blocked row itself unfaded so its focus ring survives', () => {
    const box = /^\.pick-row\[aria-disabled='true'\]\s*\{([^}]*)\}/m.exec(appCss);
    expect(box).not.toBeNull();
    expect(box![1]).not.toMatch(/opacity:/);
    expect(box![1]).toMatch(/cursor:\s*default;/);
    // The glyph keeps its muted colour as a SECOND cue on top of the dim, never as
    // the only one.
    const glyph = /^\.pick-row\[aria-disabled='true'\] \.pick-plus\s*\{([^}]*)\}/m.exec(appCss);
    expect(glyph).not.toBeNull();
    expect(glyph![1]).toMatch(/color:\s*var\(--muted\);/);
  });

  // The hover highlight is an "you can click this" affordance, so it must not fire on
  // a row that ignores the click. It has to be qualified on `[aria-disabled]`: these
  // rows are NEVER natively disabled, so any `:disabled` qualifier on `.pick-row` is
  // dead code that silently matches nothing.
  it('suppresses the hover affordance on a blocked source row', () => {
    expect(appCss).toMatch(/^\.pick-row:hover:not\(\[aria-disabled='true'\]\)\s*\{/m);
    expect(appCss).not.toMatch(/\.pick-row:disabled/);
  });

  it('centres the characteristics panel itself, in either mount', () => {
    const block = /^\.char-panel\s*\{([^}]*)\}/m.exec(appCss);
    expect(block).not.toBeNull();
    expect(block![1]).toMatch(/margin-inline:\s*auto;/);
    // Auto margins only centre a box narrower than its container, so the intrinsic
    // width is half the mechanism and not decoration.
    expect(block![1]).toMatch(/width:\s*fit-content;/);
    expect(block![1]).toMatch(/max-width:\s*100%;/);
  });
});
