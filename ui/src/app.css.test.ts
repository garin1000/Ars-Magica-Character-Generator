import { readdirSync, readFileSync } from 'node:fs';
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

// Comments are stripped before any rule below is matched. A `[^}]*` body match
// stops at the first closing brace, and app.css comments quote CSS at length — one
// `button { font: inherit }` inside a comment truncated the `.tab` body and made
// these assertions read a rule that was right there in the file.
const cssWithoutComments = appCss.replace(/\/\*[\s\S]*?\*\//g, '');

// A base typography reset is not observable by rendering: `render()` from
// `svelte/server` emits markup with no stylesheet attached, and even a mounted
// component in `happy-dom` would need the sheet loaded and computed styles read
// — disproportionate for a static selector.
describe('app.css', () => {
  // ── THE TYPE SCALE ──────────────────────────────────────────────────────────
  //
  // The app ships as a native binary in an OS webview, so it is seen SIDE BY SIDE
  // with the platform's own chrome. At the 15px root it started from, every string
  // in the app was visibly larger than the Explorer window next to it and the
  // padding/gap rem values were inflated to match — the report was "wastes quite
  // some space", with the warning-explanation text (`.issue`) named as the size the
  // whole app should have used.
  //
  // So the root is REBASED to that size, not merely divided: 1rem is 12.75px, which
  // is what `.issue` computed to before (0.85 x 15). A plain division would have
  // taken the secondary sizes down with it — 0.7rem would have become 8.9px, well
  // under anything readable — so the sub-body steps are re-expressed as rem
  // fractions of the NEW root that hold their old pixel size. Nothing in the app
  // renders smaller than it did at the 15px root; only body text and above shrink.
  //
  // The steps are named variables and not literals so the next rebase is one block
  // rather than forty-odd scattered decimals — the same reason `--readout-max-width`
  // exists. Sizes at or above 1rem stay plain rem: those SHOULD track the root.
  const SCALE_STEPS = {
    'font-small': 12, // was 0.8rem/0.82rem/0.85rem/0.9rem/0.95rem of a 15px root
    'font-chrome': 11.25, // was 0.75rem — badges, params, tab labels
    'font-micro': 10.5, // was 0.7rem — the smallest text the app has ever shipped
  } as const;

  /** The `:root` declaration block, comments already stripped. */
  const rootBlock = (): string => {
    const block = /^:root\s*\{([^}]*)\}/m.exec(cssWithoutComments);
    expect(block, 'app.css should declare a :root block').not.toBeNull();
    return block![1];
  };

  /** The rem value of a `--font-*` scale step declared on `:root`. */
  const scaleStepRem = (name: string): number => {
    const declared = new RegExp(`--${name}:\\s*([\\d.]+)rem;`).exec(rootBlock());
    expect(declared, `:root should declare --${name} in rem`).not.toBeNull();
    return Number(declared![1]);
  };

  it('rebases the root on the size the warning text already used', () => {
    // 12.75px exactly: 0.85 x the old 15px root, i.e. what `.issue` computed to.
    expect(rootBlock()).toMatch(/font-size:\s*12\.75px;/);
  });

  it('holds every sub-body step at the pixel size it had on the 15px root', () => {
    const root = 12.75;
    let previous = root;
    for (const [name, expectedPx] of Object.entries(SCALE_STEPS)) {
      const px = scaleStepRem(name) * root;
      // The step is a fraction of the NEW root that reproduces the OLD pixel size,
      // so a rebase can never be the thing that made text smaller.
      expect(px, `--${name}`).toBeCloseTo(expectedPx, 1);
      // …and the steps stay strictly ordered, body first. Re-expressing them
      // independently is exactly how two of them end up inverted.
      expect(px, `--${name} must stay below the step above it`).toBeLessThan(previous);
      previous = px;
    }
    // The floor is absolute, not relative: 10.5px is the smallest text this app has
    // ever shipped and no rebase may go under it.
    expect(scaleStepRem('font-micro') * root).toBeGreaterThanOrEqual(10.5);
  });

  it('routes every sub-body size through the scale instead of a bare rem literal', () => {
    // A stray `font-size: 0.75rem` is not wrong at 15px and IS wrong at 12.75px —
    // it silently means 9.6px. The literals are gone so the next one is a review
    // question rather than an invisible regression.
    const componentsDir = fileURLToPath(new URL('./lib/components/', import.meta.url));
    const sources: Array<[string, string]> = [['app.css', cssWithoutComments]];
    for (const entry of readdirSync(componentsDir)) {
      if (!entry.endsWith('.svelte')) continue;
      sources.push([entry, readFileSync(`${componentsDir}${entry}`, 'utf-8')]);
    }

    const offenders: string[] = [];
    for (const [name, source] of sources) {
      for (const match of source.matchAll(/font-size:\s*(0?\.\d+)rem/g)) {
        offenders.push(`${name}: font-size: ${match[1]}rem`);
      }
    }
    expect(offenders).toEqual([]);
  });

  // WCAG 2.5.8 (AA) puts the pointer-target floor at 24x24 CSS px, and `.icon-btn`
  // — every `×` remove button and every +/- stepper in the app — was sized at
  // exactly 1.6rem, i.e. exactly 24px, on the 15px root. Rebasing the root without
  // touching it would have made it 20.4px: a square button is the one shape where
  // the type rebase lands directly on an accessibility floor.
  it('keeps the square glyph buttons at the WCAG pointer-target floor', () => {
    const button = /^\.icon-btn\s*\{([^}]*)\}/m.exec(cssWithoutComments);
    expect(button).not.toBeNull();
    const width = /width:\s*([\d.]+)rem;/.exec(button![1]);
    const height = /height:\s*([\d.]+)rem;/.exec(button![1]);
    expect(width).not.toBeNull();
    expect(height).not.toBeNull();
    expect(Number(width![1]) * 12.75).toBeGreaterThanOrEqual(24);
    expect(Number(height![1]) * 12.75).toBeGreaterThanOrEqual(24);
  });

  // The two content-derived column floors are the other place a rem value carries an
  // absolute measurement: 24rem was chosen as "360px holds a Living Conditions row,
  // the aging formula's longest German token and a labelled field", and
  // `aging.e2e.js` asserts no `.character-details` track comes out under 360px. A
  // rebase that left the number alone would have quietly moved the floor to 306px
  // and reintroduced the checklist overflow fixed in the same week.
  it('keeps the content-derived column floors at the 360px they were measured for', () => {
    for (const rule of ['character-details', 'living-conditions-list']) {
      const block = new RegExp(`^\\.${rule}\\s*\\{([^}]*)\\}`, 'm').exec(cssWithoutComments);
      expect(block, `.${rule} should exist`).not.toBeNull();
      const floorRem = parseFloat(/minmax\(\s*([\d.]+)rem/.exec(block![1])![1]);
      expect(floorRem * 12.75, `.${rule} column floor`).toBeGreaterThanOrEqual(360);
    }
  });

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

  /** The `rem` floor of the first `minmax()` in a rule body. */
  const columnFloorRem = (body: string): number =>
    parseFloat(/minmax\(\s*([\d.]+)rem/.exec(body)![1]);

  // Why this test exists: the #24/#25 reflow gave the checklist its own column floor
  // (18rem) while the row it has to hold was never re-measured — and a checklist
  // column is exactly one of the three things `.character-details`' own 24rem floor
  // was CHOSEN from ("a Living Conditions row … plus the checkbox and the signed
  // modifier column", see the comment on that rule). A narrower floor therefore
  // contradicts the only measurement anyone has taken of this content, and it did:
  // at the 1100px default window the 18rem floor fitted three 325px tracks under a
  // 358px row, so the widest names ran into the next column and 15px past the panel.
  // Coupling the two floors is the point — the number lives in one place, so the
  // checklist cannot drift narrower than the row it was sized for again.
  it('never flows the checklist into columns narrower than a row was measured for', () => {
    const list = /^\.living-conditions-list\s*\{([^}]*)\}/m.exec(appCss);
    const panel = /^\.character-details\s*\{([^}]*)\}/m.exec(appCss);
    expect(list).not.toBeNull();
    expect(panel).not.toBeNull();
    expect(columnFloorRem(list![1])).toBeGreaterThanOrEqual(columnFloorRem(panel![1]));
  });

  // …and a floor is a floor, not a guarantee: a longer localized name (German's are
  // half again as long as English's), a larger system font or a narrower window all
  // put a row over its column again. `.checkbox.inline` is shared with the
  // Equipped toggle, whose whole point is a two-word label that never breaks, so its
  // `white-space: nowrap` is inherited here by a checklist of full sentences — and a
  // grid track sized by `minmax()` does not grow to fit an unbreakable item, so the
  // row simply spills over its neighbour. Wrapping is what makes the overflow
  // impossible at ANY width in ANY language, rather than merely unlikely at this one.
  it('lets a long Living Conditions row wrap rather than spill out of its column', () => {
    const rule = /^\.living-conditions-list \.checkbox\.inline\s*\{([^}]*)\}/m.exec(appCss);
    expect(rule).not.toBeNull();
    expect(rule![1]).toMatch(/white-space:\s*normal;/);
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

  // ── The Abilities tab's selected-row grid ───────────────────────────────────
  //
  // manual-testing-findings-2026-09 #31. `.ability-selection li` is a flex line
  // with TWO GROWING items — `.item-name` (`flex: 1`) and the trailing
  // `.specialty` / `.ability-unbought` (`flex: 1 1 6rem`) — so a fixed-width item
  // the row omits is not merely absent, it is REDISTRIBUTED between those two.
  // The display-only (unbought) row drops the 1.9rem remove button and its 0.4rem
  // flex gap, i.e. 2.3rem = 29.3px, which the two growers split: `.item-name` came
  // out ~14.7px wider and carried the spinner, the effective badge and the marker
  // that far right of every bought row above and below it.
  //
  // None of this is observable by rendering — `render` from `svelte/server`
  // attaches no stylesheet and happy-dom performs no layout, so there is no box to
  // measure at any level below e2e. The stylesheet contract is the guard, and the
  // markup half (that the unbought branch actually emits the slot, aria-hidden and
  // unfocusable) is pinned in `lib/components/AbilityTab.test.ts`.

  /** The declaration block of a descendant-selector rule, comments stripped. */
  function selectorBody(selector: string): string {
    const pattern = new RegExp(`^${selector.replace(/\./g, '\\.')}\\s*\\{([^}]*)\\}`, 'm');
    const block = pattern.exec(cssWithoutComments);
    expect(block, `app.css should declare ${selector}`).not.toBeNull();
    return block![1];
  }

  /** A rule's own `width`, in CSS pixels. */
  function widthPx(body: string): number {
    const declared = /width:\s*([^;]+);/.exec(body);
    expect(declared, 'the rule should declare its own width').not.toBeNull();
    return lengthPx(declared![1]);
  }

  // The idiom the fix below copies, and which had no contract of its own either:
  // the badge slot is rendered on EVERY row, empty when no bonus applies, so the
  // controls after it do not move when a Puissant virtue is added or removed.
  it('reserves a fixed box for an absent effective-score badge', () => {
    const slot = ruleBody('eff-slot');
    // Neither grows nor shrinks — a slot that flexed would defeat its own purpose.
    expect(slot).toMatch(/flex:\s*0\s+0\s+auto;/);
    expect(widthPx(slot)).toBeGreaterThan(0);
  });

  it('reserves the same fixed box for the remove button an unbought row lacks', () => {
    const slot = ruleBody('remove-slot');
    // `.eff-slot`'s construction exactly, for the same reason.
    expect(slot).toMatch(/flex:\s*0\s+0\s+auto;/);
    // …and it is the width of the control it stands in for, read from that
    // control's own rule rather than restated, so the two cannot drift apart.
    expect(widthPx(slot)).toBe(widthPx(ruleBody('icon-btn')));
  });

  it('leaves an unbought ability row zero horizontal drift from a bought one', () => {
    const row = selectorBody('.ability-selection li');
    expect(row).toMatch(/display:\s*flex;/);
    const gapPx = lengthPx(/gap:\s*([^;]+);/.exec(row)![1]);

    // What the two row kinds spend to the right of the growing name column on the
    // items where they DIFFER: a remove button on a bought row, the slot standing
    // in for it on an unbought one. One flex gap each, because each is one item.
    const bought = gapPx + widthPx(ruleBody('icon-btn'));
    const unbought = gapPx + widthPx(ruleBody('remove-slot'));
    // Zero, not "smaller": free space is what the growers split, so any remainder
    // reappears as drift on `.item-name` at half its size.
    expect(unbought - bought).toBe(0);

    // The other half of the invariant: the marker must keep the specialty box's
    // exact flex share, or the split changes shape even with the width reserved.
    //
    // `.item-name` is declared twice at the top level (a wrapping block for the
    // Available picker's tagged rows, then the flex share these rows use), so the
    // grow factor is looked for across both rather than in whichever comes first.
    const nameFlex = [...cssWithoutComments.matchAll(/^\.item-name\s*\{([^}]*)\}/gm)]
      .map((block) => /flex:\s*([^;]+);/.exec(block[1])?.[1])
      .filter((declared) => declared !== undefined);
    expect(nameFlex).toEqual(['1']);
    const specialty = selectorBody('.ability-selection .specialty');
    const marker = selectorBody('.ability-selection .ability-unbought');
    for (const property of ['flex', 'min-width']) {
      const declared = new RegExp(`${property}:\\s*([^;]+);`);
      expect(declared.exec(marker)![1], property).toBe(declared.exec(specialty)![1]);
    }
  });

  it('sizes the unbought row marker with the scale step meant for row markers', () => {
    // `--font-chrome` is what the scale's own taxonomy assigns to row markers (see
    // the `:root` comment); the marker shipped one step up, on `--font-small`, the
    // step for secondary running text.
    expect(selectorBody('.ability-selection .ability-unbought')).toMatch(
      /font-size:\s*var\(--font-chrome\);/,
    );
  });

  // ── The edit-mode tab strip has to fit its widest label set ─────────────────
  //
  // manual-testing-findings-2026-09 #1: at the default 1100x800 window the strip
  // ran out of room and the tab titles ellipsized — which is why every tab also
  // carries its full label in `title` (App.svelte). GERMAN is the binding case,
  // not English: the magus tab set (thirteen tabs, the longest any type gets) is
  // ~146 characters of label where English is ~130.
  //
  // WHY ARITHMETIC AND NOT A RENDER. Text advance width cannot be observed here:
  // `render` from `svelte/server` attaches no stylesheet, and happy-dom performs
  // no layout, so there is no measurable box at any level below e2e. The budget
  // below is therefore a model — but a CALIBRATED one, against the single real
  // measurement this repo recorded. `.tabbar`'s own comment states the English
  // thirteen-tab strip as "~1350px of text" at the old geometry (15px type,
  // 0.75rem tab padding, 0.25rem gap). Subtracting that geometry
  // (13 x 2 x 11.25px padding + 12 x 3.75px gap = 337.5px) leaves ~1012px for 130
  // characters, i.e. 0.52em per character. An independent per-glyph sum over
  // DejaVu Sans (what `system-ui` usually resolves to in the shipped WebKitGTK)
  // puts the German set at 0.53em per character. The larger of the two is used,
  // so the model errs towards a wider string than reality.
  //
  // The e2e counterpart (`e2e/specs/tab-area.e2e.js`) measures the real thing in
  // a real engine; this test is the fast guard that catches a longer German label
  // or a loosened rule long before the binary is built.
  const ROOT_FONT_PX = 12.75; // `:root { font-size: 12.75px }` — see the type scale above
  const EM_PER_CHARACTER = 0.53;
  const DEFAULT_WINDOW_PX = 1100; // crates/arm-app/tauri.conf.json

  // The magus set, in App.svelte's order. Spelled out rather than derived: the
  // point is precisely which thirteen labels share one strip, and a type whose set
  // grows past this one is exactly the change that must fail here.
  const MAGUS_TAB_KEYS = [
    'tab-details',
    'tab-characteristics',
    'tab-virtues-flaws',
    'tab-experience',
    'tab-abilities',
    'tab-arts',
    'tab-spells',
    'tab-personality-reputations',
    'tab-aging',
    'tab-possessions',
    'tab-house-specialisation',
    'tab-equipment',
    'tab-totals',
  ];
  // The two tabs no magus has (mythic companion only) still share the strip with
  // the rest of their own set, so they count for the minimum-target check.
  const OTHER_TAB_KEYS = ['tab-mythic-type', 'tab-supernatural'];

  /** The declaration block of a top-level class rule, e.g. `tab` or `tabbar`. */
  function ruleBody(className: string): string {
    const block = new RegExp(`^\\.${className}\\s*\\{([^}]*)\\}`, 'm').exec(cssWithoutComments);
    expect(block, `app.css should declare a base .${className} rule`).not.toBeNull();
    return block![1];
  }

  /** One CSS length token (`0`, `0.375rem`, `2px`) in CSS pixels. */
  function lengthPx(token: string): number {
    const match = /^(-?[\d.]+)(rem|px)?$/.exec(token.trim());
    expect(match, `'${token}' should be a plain px/rem length`).not.toBeNull();
    return match![2] === 'rem' ? Number(match![1]) * ROOT_FONT_PX : Number(match![1]);
  }

  /**
   * A rule's own `font-size` in CSS pixels, whether it names a scale step or spells
   * a rem value out. Sub-body sizes go through `--font-*` (see the type scale at the
   * top of this file), so a raw `lengthPx` on the token would not resolve.
   */
  function fontSizePx(body: string): number {
    const declared = /font-size:\s*([^;]+);/.exec(body);
    expect(declared, 'the rule should declare its own font-size').not.toBeNull();
    const token = declared![1].trim();
    const step = /^var\(--(font-[a-z]+)\)$/.exec(token);
    return step ? scaleStepRem(step[1]) * ROOT_FONT_PX : lengthPx(token);
  }

  /** The `padding` shorthand's [vertical, horizontal] halves, in CSS pixels. */
  function paddingPx(body: string): [number, number] {
    const shorthand = /padding:\s*([^;]+);/.exec(body);
    expect(shorthand).not.toBeNull();
    const parts = shorthand![1].trim().split(/\s+/);
    expect(parts).toHaveLength(2);
    return [lengthPx(parts[0]), lengthPx(parts[1])];
  }

  /** One Fluent value from a shipped locale. */
  function ftlLabel(source: string, key: string): string {
    const entry = new RegExp(`^${key} = (.+)$`, 'm').exec(source);
    expect(entry, `${key} should be translated`).not.toBeNull();
    // NFC so a decomposed umlaut cannot inflate the character count.
    return entry![1].trim().normalize('NFC');
  }

  const locale = (lang: string) =>
    readFileSync(
      fileURLToPath(new URL(`../../locales/${lang}/main.ftl`, import.meta.url)),
      'utf-8',
    );

  it('gives the tab strip its own type size — small enough to fit, large enough to read', () => {
    const tab = ruleBody('tab');

    // Declared HERE and not inherited: `button { font: inherit }` otherwise hands
    // every tab the root body size, which is what overflowed the strip.
    const fontPx = fontSizePx(tab);

    // A floor, not just a ceiling, and an ABSOLUTE one — a relative floor
    // (`0.7 * ROOT_FONT_PX`) would have silently followed the root down to 8.9px
    // during the 15px → 12.75px rebase, which is exactly the failure the rebase had
    // to avoid. Below ~10.5px short nav labels stop being comfortably readable and
    // no amount of fitting is worth that; above the body size the strip would be
    // shouting over the content it labels.
    expect(fontPx).toBeGreaterThanOrEqual(10.5);
    expect(fontPx).toBeLessThan(ROOT_FONT_PX);

    const [vertical, horizontal] = paddingPx(tab);
    // The horizontal padding is what the labels are competing with: thirteen tabs
    // spend it twenty-six times over.
    expect(horizontal).toBeLessThanOrEqual(0.5 * ROOT_FONT_PX);
    // Smaller type shrinks the button in BOTH directions, and the vertical one buys
    // no room at all — so the vertical padding compensates instead of following the
    // font down. Roughly the line box plus both paddings.
    expect(fontPx * 1.2 + 2 * vertical).toBeGreaterThanOrEqual(32);

    // WCAG 2.5.8 (AA) puts the floor at 24x24 CSS px. The narrowest tab in either
    // shipped locale is the one that decides it.
    const shortest = Math.min(
      ...['en', 'de'].flatMap((lang) => {
        const source = locale(lang);
        return [...MAGUS_TAB_KEYS, ...OTHER_TAB_KEYS].map((key) => ftlLabel(source, key).length);
      }),
    );
    expect(shortest * EM_PER_CHARACTER * fontPx + 2 * horizontal).toBeGreaterThanOrEqual(24);
  });

  it('fits the widest (German) tab set inside the default window', () => {
    const tab = ruleBody('tab');
    const tabbar = ruleBody('tabbar');

    const fontPx = fontSizePx(tab);
    const [, horizontal] = paddingPx(tab);
    const [, barHorizontal] = paddingPx(tabbar);
    const gapPx = lengthPx(/gap:\s*([^;]+);/.exec(tabbar)![1]);

    const german = locale('de');
    const characters = MAGUS_TAB_KEYS.reduce(
      (total, key) => total + ftlLabel(german, key).length,
      0,
    );

    const labels = characters * EM_PER_CHARACTER * fontPx;
    const padding = 2 * horizontal * MAGUS_TAB_KEYS.length;
    const gaps = gapPx * (MAGUS_TAB_KEYS.length - 1);
    const strip = DEFAULT_WINDOW_PX - 2 * barHorizontal;

    expect(labels + padding + gaps).toBeLessThanOrEqual(strip);

    // German must stay the binding case: if English ever needed more room than
    // German, the arithmetic above would be guarding the wrong locale.
    const englishSource = locale('en');
    const english = MAGUS_TAB_KEYS.reduce(
      (total, key) => total + ftlLabel(englishSource, key).length,
      0,
    );
    expect(characters).toBeGreaterThan(english);
  });
});
