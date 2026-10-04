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
//
// The same strip serves the colour-token scan further down for a second reason:
// this file's comments quote hex values in prose (the `--muted` (#c3bfde) note on
// the blocked source row), and a literal a browser never parses is not a literal
// the palette has to reach.
const stripBlockComments = (source: string): string => source.replace(/\/\*[\s\S]*?\*\//g, '');

const cssWithoutComments = stripBlockComments(appCss);

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

  // ── THE COLOUR TOKEN SYSTEM ─────────────────────────────────────────────────
  //
  // Every colour the app paints comes from a semantic custom property declared on
  // `:root` in this file, so swapping the palette is an edit to that one block and
  // to nothing else. Both halves of that sentence are pinned below, because both
  // were broken when the tokens were introduced:
  //
  //  * A `var()` naming a token nobody declared is not a harmless no-op. WITH a
  //    fallback it silently paints the fallback — `var(--surface, #fff)` gave both
  //    modals a white panel inside a dark app, so their `color: inherit` text came
  //    out white on white. WITHOUT one the declaration is invalid at
  //    computed-value time and resets to its initial value, which is why
  //    `border-left: 2px solid var(--border)` on the Crisis block drew no border
  //    at all: the unresolvable shorthand took `border-left-style` back to `none`.
  //  * A raw literal outside the token block is a colour the palette swap cannot
  //    reach, so it survives the swap and lands wrong against the new background.

  /** Every `.svelte` source under `src/`, comment-stripped, as `[fileName, text]`. */
  const svelteSources = (): Array<[string, string]> => {
    const found: Array<[string, string]> = [];
    const walk = (dir: string): void => {
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const path = `${dir}${entry.name}`;
        if (entry.isDirectory()) walk(`${path}/`);
        else if (entry.name.endsWith('.svelte'))
          found.push([entry.name, stripBlockComments(readFileSync(path, 'utf-8'))]);
      }
    };
    walk(fileURLToPath(new URL('./', import.meta.url)));
    return found;
  };

  /** Every source a colour can hide in: the stylesheet plus every component. */
  const styledSources = (): Array<[string, string]> => [
    ['app.css', cssWithoutComments],
    ...svelteSources(),
  ];

  /** The dark palette's selector — the bare root, i.e. the app's default. */
  const DARK = ':root';
  /** The light palette's selector. */
  const LIGHT = ":root[data-theme='light']";
  /** The two selectors allowed to hold colour literals, and no others. */
  const PALETTES: ReadonlySet<string> = new Set([DARK, LIGHT]);

  /** Every `:root…` declaration block in the stylesheet, as `[selector, body]`. */
  const paletteBlocks = (): Array<[string, string]> =>
    [...cssWithoutComments.matchAll(/^(:root[^{]*)\{([^}]*)\}/gm)].map(([, selector, body]) => [
      selector.trim(),
      body,
    ]);

  /** One palette's declaration block. */
  const paletteBody = (selector: string): string => {
    const found = paletteBlocks().find(([declared]) => declared === selector);
    expect(found, `app.css should declare a ${selector} palette block`).not.toBeUndefined();
    return found![1];
  };

  it('resolves every var() reference to a token declared on :root', () => {
    const declared = new Set(
      [...rootBlock().matchAll(/(--[a-z0-9-]+)\s*:/g)].map((match) => match[1]),
    );

    const undeclared: string[] = [];
    for (const [name, source] of styledSources()) {
      for (const match of source.matchAll(/var\(\s*(--[a-z0-9-]+)/g)) {
        const reference = `${name}: ${match[1]}`;
        if (!declared.has(match[1]) && !undeclared.includes(reference)) undeclared.push(reference);
      }
    }
    expect(undeclared.sort()).toEqual([]);
  });

  it('keeps every colour literal inside a :root palette block', () => {
    // One rule, so one test: a component's scoped `<style>` cannot declare a
    // `:root` block of its own, but it INHERITS the global tokens — that is the
    // whole point of putting them on the root — so its literal budget is simply
    // zero, and the two files are checked by the same assertion rather than by a
    // sibling test that could drift from it.
    //
    // EVERY palette block is exempt, not merely the first: the light palette is a
    // second `:root…` rule of nothing but literals, and a strip anchored on the
    // dark one alone reported all twenty-one of them. The exemption is by
    // SELECTOR NAME, so it cannot widen by accident — a rule that merely happens
    // to begin `:root` (`:root .banner`, say) still has a literal budget of zero.
    const colourLiteral = /#[0-9a-fA-F]{3,8}\b|\brgba?\(/g;
    const sources = styledSources();
    sources[0] = [
      'app.css',
      paletteBlocks()
        .filter(([selector]) => PALETTES.has(selector))
        .reduce((css, [, body]) => css.replace(body, ''), cssWithoutComments),
    ];

    const offenders: string[] = [];
    for (const [name, source] of sources) {
      for (const match of source.matchAll(colourLiteral)) offenders.push(`${name}: ${match[0]}`);
    }
    expect(offenders).toEqual([]);
  });

  // ── THE TWO PALETTES ────────────────────────────────────────────────────────
  //
  // The app ships a dark palette (the bare `:root` block) and a light one
  // (`:root[data-theme='light']`), and the frontend puts `data-theme` on <html>
  // — resolving `auto` against the OS itself, so CSS never has to know about
  // `prefers-color-scheme`. Everything below pins the properties that make that
  // swap safe rather than merely present.

  /** The custom properties a block declares, as name → value. */
  const declarations = (body: string): Map<string, string> =>
    new Map(
      [...body.matchAll(/(--[a-z0-9-]+)\s*:\s*([^;]+);/g)].map((match) => [
        match[1],
        match[2].trim().toLowerCase(),
      ]),
    );

  /** Does this token's value carry a colour, as against a length or a number? */
  const isColourValued = (value: string): boolean => /#[0-9a-f]{3,8}\b|\brgba?\(/.test(value);

  /**
   * A token's value under `selector`, falling back to `:root` exactly as the
   * cascade does — so a token the light palette forgets is READ at its dark
   * value here, which is precisely what the contrast assertions below must see
   * in order to fail on it.
   */
  const tokenValue = (selector: string, name: string): string => {
    const value =
      declarations(paletteBody(selector)).get(name) ?? declarations(paletteBody(DARK)).get(name);
    expect(value, `${name} should be reachable from ${selector}`).not.toBeUndefined();
    return value!;
  };

  it('declares a light palette holding every colour the dark palette declares', () => {
    const dark = declarations(paletteBody(DARK));
    const light = declarations(paletteBody(LIGHT));

    // A HALF-FILLED palette is the failure mode a theme switch actually has: the
    // missing token simply inherits its DARK value, so one white-on-white label
    // or one invisible border ships and nothing anywhere says so.
    const unthemed = [...dark]
      .filter(([, value]) => isColourValued(value))
      .map(([name]) => name)
      .filter((name) => !light.has(name));
    expect(unthemed.sort()).toEqual([]);

    // …and the reverse: a token the light block invents is simply UNDEFINED
    // whenever the app is dark, which is the `var(--surface, #fff)` bug C1 found,
    // reintroduced from the other side.
    const orphans = [...light.keys()].filter((name) => !dark.has(name));
    expect(orphans.sort()).toEqual([]);
  });

  it('switches color-scheme with the palette', () => {
    // This is not decoration. The stylesheet declares no `:focus-visible` rule
    // and styles no scrollbar, so `color-scheme` is the app's ENTIRE strategy for
    // the focus ring, the scrollbars and the native <select> popup — all three
    // are drawn by the engine from this one keyword. A light palette left under
    // `color-scheme: dark` gets a dark scrollbar, a dark dropdown and a focus
    // ring tuned for the wrong background.
    expect(paletteBody(DARK)).toMatch(/color-scheme:\s*dark;/);
    expect(paletteBody(LIGHT)).toMatch(/color-scheme:\s*light;/);
  });

  // ── CONTRAST, COMPUTED RATHER THAN ASSERTED IN A COMMENT ────────────────────
  //
  // Every accessibility claim this stylesheet makes about a colour used to live
  // in prose, derived by hand from the DARK palette. A second palette turns each
  // of those into a claim that can be false while its test stays green, so they
  // are computed here instead, per palette, from the tokens themselves.

  /** The 0-1 sRGB channels of a `#rrggbb` literal. */
  const channels = (hex: string): [number, number, number] => {
    const parsed = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/.exec(hex.trim().toLowerCase());
    expect(parsed, `'${hex}' should be a six-digit hex colour`).not.toBeNull();
    return [1, 2, 3].map((group) => parseInt(parsed![group], 16) / 255) as [number, number, number];
  };

  /** WCAG 2.x relative luminance of an opaque `#rrggbb` colour. */
  const luminance = (hex: string): number => {
    const linear = channels(hex).map((value) =>
      value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4,
    );
    return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
  };

  /** WCAG 2.x contrast ratio between two opaque `#rrggbb` colours. */
  const contrastRatio = (one: string, other: string): number => {
    const [lighter, darker] = [luminance(one), luminance(other)].sort((a, b) => b - a);
    return (lighter + 0.05) / (darker + 0.05);
  };

  /** `over` at `alpha` composited on opaque `under`, as `#rrggbb`. */
  const composite = (over: string, under: string, alpha: number): string => {
    const top = channels(over);
    const bottom = channels(under);
    const mixed = top.map((value, index) =>
      Math.round(255 * (alpha * value + (1 - alpha) * bottom[index]))
        .toString(16)
        .padStart(2, '0'),
    );
    return `#${mixed.join('')}`;
  };

  /**
   * The `opacity` that `rule` declares, resolved under `selector` — whether the
   * rule spells the number out or routes it through a palette token.
   */
  const dimDeclaredBy = (rule: RegExp, selector: string, what: string): number => {
    const block = rule.exec(cssWithoutComments);
    expect(block, `app.css should declare ${what}`).not.toBeNull();
    const declared = /opacity:\s*([^;]+);/.exec(block![1]);
    expect(declared, `${what} should declare an opacity`).not.toBeNull();
    const token = /^var\((--[a-z0-9-]+)\)$/.exec(declared![1].trim());
    return Number(token ? tokenValue(selector, token[1]) : declared![1]);
  };

  /** The blocked source row's dim, under `selector`. */
  const blockedDim = (selector: string): number =>
    dimDeclaredBy(
      /^\.pick-row\[aria-disabled='true'\] > \*\s*\{([^}]*)\}/m,
      selector,
      'the blocked source row dim',
    );

  // Finding #18's dim (see `.pick-row[aria-disabled='true']` below) was chosen
  // against the dark palette and its AA claim was recorded in a comment: "`--ink`
  // composited over `--panel` still clears 4.5:1". That derivation does not
  // survive a second palette — the SAME opacity moves dark ink toward a light
  // panel far faster in luminance terms than it moves light ink toward a dark
  // one, so 0.55 measures 5.11:1 dark and only 3.35:1 light, a WCAG 1.4.3
  // failure that no opacity band could ever have reported. The claim is computed
  // here, per palette, so it cannot rot again.
  it('keeps a dimmed blocked row above AA in BOTH palettes, and still reading as off', () => {
    for (const selector of [DARK, LIGHT]) {
      const ink = tokenValue(selector, '--ink');
      const panel = tokenValue(selector, '--panel');
      const dimmed = contrastRatio(composite(ink, panel, blockedDim(selector)), panel);

      expect(dimmed, `${selector}: dimmed --ink over --panel`).toBeGreaterThanOrEqual(4.5);
      // The other half of finding #18: the dim is the row's PRIMARY "you cannot
      // take this" cue, so it has to be a real luminance drop and not a token
      // one. Expressed against the same ink undimmed rather than as an opacity
      // number, because an opacity means different things on the two palettes —
      // which is the whole reason the first assertion needed re-deriving.
      expect(dimmed, `${selector}: dimmed vs full --ink`).toBeLessThanOrEqual(
        0.5 * contrastRatio(ink, panel),
      );
    }
  });

  // U5 (P7, `docs/open-todos.md`, 2026-09-13): "Table and panel backgrounds
  // should be a lighter beige, not white" — the light palette's `--panel` was
  // #fafaf8 (R-B of 2: a near-neutral, not a tint), and `.derived-table` and
  // every `.panel` (AgingPanel, VirtueFlawTab's source panels, …) paint from
  // it, so ONE token carries both. Proven rather than eyeballed, per P7's own
  // instruction: a warm bias wide enough to actually read as beige, and the
  // 4.5:1 obligation against `--ink` computed in BOTH palettes — the dark
  // palette's panel is already not white, so this only has to stay true of it,
  // not change to satisfy it.
  it('gives the light palette a beige panel, not a flat white one, and keeps it AA (P7)', () => {
    const panel = tokenValue(LIGHT, '--panel');
    const [r, g, b] = channels(panel).map((value) => Math.round(value * 255));
    expect(
      r - b,
      `light --panel ${panel} should read as a warm beige, not a neutral white`,
    ).toBeGreaterThanOrEqual(8);
    expect(r, `light --panel ${panel} should shade warm (R >= G >= B)`).toBeGreaterThanOrEqual(g);
    expect(g).toBeGreaterThanOrEqual(b);

    for (const selector of [DARK, LIGHT]) {
      expect(
        contrastRatio(tokenValue(selector, '--ink'), tokenValue(selector, '--panel')),
        `${selector}: --ink over --panel`,
      ).toBeGreaterThanOrEqual(4.5);
    }
  });

  // E4's technical-detail disclosure is new running text on the window
  // background — the start screen's error block and the header's both sit
  // directly on `--bg`, neither declaring a surface of its own. It dims nothing
  // (no `opacity`, so it is outside `DIMMED_TEXT` above), but the two tokens it
  // DOES choose still have to clear AA in both palettes, and `--muted` is the
  // quieter of the pair: it measures 8.44:1 on `--panel` dark and only 5.49:1
  // light, so a claim derived from the dark palette alone would prove nothing.
  // Computed here rather than asserted in the stylesheet's comment, exactly as
  // the two rules above are.
  it('keeps the technical-detail disclosure above AA in BOTH palettes', () => {
    const declaredColour = (rule: RegExp, what: string): string => {
      const block = rule.exec(cssWithoutComments);
      expect(block, `app.css should declare ${what}`).not.toBeNull();
      const declared = /[^-]color:\s*var\((--[a-z0-9-]+)\);/.exec(block![1]);
      expect(declared, `${what} should take its colour from a palette token`).not.toBeNull();
      return declared![1];
    };

    const payload = declaredColour(/^\.error-details\s*\{([^}]*)\}/m, 'the disclosure payload');
    const summary = declaredColour(
      /^\.error-details summary\s*\{([^}]*)\}/m,
      'the disclosure summary',
    );

    for (const selector of [DARK, LIGHT]) {
      const surface = tokenValue(selector, '--bg');
      for (const [what, token] of [
        ['payload', payload],
        ['summary', summary],
      ]) {
        expect(
          contrastRatio(tokenValue(selector, token), surface),
          `${selector}: disclosure ${what} (${token}) on --bg`,
        ).toBeGreaterThanOrEqual(4.5);
      }
    }
  });

  // Every OTHER place the stylesheet dims live text with an opacity, as
  // `[what, rule, the token it dims, the surface it sits on]`. Both entries have
  // the blocked row's problem: they dim a token that already sits close to the
  // light palette's AA floor (`--ink-dim` is 4.60:1 on `--bg`, `--muted` 5.49:1
  // on `--panel`), so ANY fraction under 1 takes them below it — where the dark
  // palette, whose same two tokens measure 9.13:1 and 8.44:1, has room to spare.
  //
  // `:disabled` rules are deliberately absent: WCAG 1.4.3 exempts an inactive
  // control, and a disabled wizard step that still measured 4.5:1 would not read
  // as disabled at all.
  const DIMMED_TEXT: Array<[string, RegExp, string, string]> = [
    [
      'the wizard rail step number',
      /^\.wizard-rail-step::before\s*\{([^}]*)\}/m,
      '--ink-dim',
      '--bg',
    ],
    [
      'the character banner placeholders',
      /^\.char-banner \.name-input::placeholder,[^{]*\{([^}]*)\}/m,
      '--muted',
      '--panel',
    ],
  ];

  it('keeps every other dimmed text rule above AA in BOTH palettes', () => {
    for (const [what, rule, token, surface] of DIMMED_TEXT) {
      for (const selector of [DARK, LIGHT]) {
        const ink = tokenValue(selector, token);
        const under = tokenValue(selector, surface);
        const dim = dimDeclaredBy(rule, selector, what);
        expect(
          contrastRatio(composite(ink, under, dim), under),
          `${selector}: ${what}`,
        ).toBeGreaterThanOrEqual(4.5);
      }
    }
  });

  /** An `rgba(r, g, b, a)` token, as `[#rrggbb, alpha]`. */
  const parseRgba = (value: string): [string, number] => {
    const parsed = /^rgba\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*([\d.]+)\s*\)$/.exec(
      value.trim().toLowerCase(),
    );
    expect(parsed, `'${value}' should be an rgba() wash`).not.toBeNull();
    const hex = [1, 2, 3]
      .map((group) => Number(parsed![group]).toString(16).padStart(2, '0'))
      .join('');
    return [`#${hex}`, Number(parsed![4])];
  };

  // The severity row tints were the one thing C1 could not carry over honestly,
  // and it said so in place: they are hand-mixed washes of #e2657a and #d8a657 —
  // colours that are NOT `--error` and `--warning`, merely near them. That is a
  // bug with two halves. The wash is not the severity colour it claims to be,
  // and a single alpha shared by both severities cannot produce the same cue for
  // both, because the two colours sit at different luminances against their own
  // background. Both halves are pinned here, per palette.
  //
  // `--bg`, not `--panel`: `.validation-bar` declares no background of its own,
  // so an issue row composites straight onto the window.
  it('washes a severity row with that palette own severity colour, at a matched strength', () => {
    for (const [tint, severity] of [
      ['--tint-error', '--error'],
      ['--tint-warning', '--warning'],
    ]) {
      const steps = [DARK, LIGHT].map((selector) => {
        const background = tokenValue(selector, '--bg');
        const [wash, alpha] = parseRgba(tokenValue(selector, tint));
        // Derived, never hand-mixed: the row's tint IS its border colour, so the
        // two channels of the severity cue can never drift apart.
        expect(wash, `${selector}: ${tint}`).toBe(tokenValue(selector, severity));
        return contrastRatio(composite(wash, background, alpha), background);
      });

      for (const [index, step] of steps.entries()) {
        const selector = [DARK, LIGHT][index];
        // Visible as a severity cue at a glance…
        expect(step, `${selector}: ${tint} step`).toBeGreaterThan(1.1);
        // …and never a smudge: the row's meaning is carried by its uppercase
        // badge and its left border, so the fill only has to group them.
        expect(step, `${selector}: ${tint} step`).toBeLessThan(1.4);
      }
      // The SAME cue in both palettes — which a shared alpha cannot deliver, and
      // is the reason each palette tunes its own.
      expect(steps[1], `${tint}: light vs dark step`).toBeCloseTo(steps[0], 1);
    }
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
  // `grog-wizard-aging.e2e.js`'s `the guided aging step` asserts no
  // `.character-details` track comes out under 360px. A
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

  // manual-testing-findings-2026-09-03 #20/#22/#33. Three passes over one problem,
  // and this is the third:
  //
  //  1. The blocks were auto-placed items of the `.character-details` grid, so a
  //     SHORT block (the schedule) shared a row with the TALL roll calculator. A grid
  //     row is as tall as its tallest item and `align-items: start` leaves the short
  //     one at the top of it, so the leftover showed as a screen-third of emptiness.
  //  2. #22 answered that by spanning every block full width — which removed the
  //     columns instead of the gap, and left a single tall stack.
  //  3. #33 puts the columns back as WRAPPERS. Each wrapper is one grid item and its
  //     own independent block container, so its height is its own content's and a
  //     short column never pays for a tall neighbour. The row-height coupling that
  //     caused (1) cannot arise, because no two aging blocks are siblings in the grid
  //     any more — only the three wrappers are.
  //
  // The wrapper is a flex column with its own `gap`: `.character-details` zeroes the
  // `.detail-section + .detail-section` margin (blocks there are grid items held
  // apart by the grid's gap), so blocks stacked inside a plain wrapper would touch.
  it('lays the aging surface out as independent column wrappers', () => {
    const rule = /\.character-details \.aging-column\s*\{([^}]*)\}/.exec(cssWithoutComments);
    expect(rule).not.toBeNull();
    // A block container of its own — not `display: contents`, which would put the
    // blocks straight back into the outer grid and restore the row coupling.
    expect(rule![1]).toMatch(/display:\s*flex;/);
    expect(rule![1]).toMatch(/flex-direction:\s*column;/);
    // Its own vertical rhythm, since the grid's gap does not reach inside it.
    expect(rule![1]).toMatch(/gap:\s*[\d.]+rem;/);
    // A grid item's automatic minimum is min-content, so one unbreakable token
    // (German's "(Langlebigkeitsritual)") would otherwise force the track wider than
    // its 1fr share and push the third column onto a second row.
    expect(rule![1]).toMatch(/min-width:\s*0;/);
  });

  // …and the #22 full-width span is GONE. It is the one rule that would silently undo
  // the wrappers: with it in place every block inside a wrapper that is not itself a
  // grid item is unaffected, but a re-added `> *` span on the panel would collapse the
  // three tracks back to one. Asserted as an absence because that is the regression.
  it('no longer spans every aging stage across the whole panel', () => {
    expect(cssWithoutComments).not.toMatch(
      /\.character-details \.aging-(panel|record) > \*[^{]*\{[^}]*grid-column:\s*1\s*\/\s*-1/,
    );
  });

  // A `<select>` (the aging year), a text input (the ritual's focus) or a textarea
  // stretched across a whole track is neither readable nor pointable. The cap is
  // still wanted after #33 — the panel degrades to two tracks and then one as the
  // window narrows, and a one-track wrapper at the 900px minimum window is ~840px
  // wide, which is exactly the case the cap was written for. At three tracks it is
  // barely binding (26rem = 331px against a 431px track), which is the right shape
  // for a maximum. One selector, not two: `.aging-record` renders inside
  // `.aging-panel`, so the second was always redundant.
  it('keeps a control on an aging stage to a readable measure', () => {
    const rule = /\.character-details \.aging-panel \.field\s*\{([^}]*)\}/.exec(cssWithoutComments);
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
  // the behaviour is verified by the e2e scroll assertion in
  // `magus-apprenticeship.e2e.js` ("keeps the XP bar on screen while the abilities
  // step scrolls"), which shrinks the window until the step really overflows.
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

  // W3 (finding 11): the Spells tab and step carry TWO bars, XpBar above
  // SpellBudgetBar. Two sibling stickies both at `top: 0` land on the same strip and
  // the later one paints over the earlier, so the pair rides in one `.bar-stack`
  // that is itself the pinned box — sticky, opaque and raised exactly like a bar.
  it('pins the two-bar stack as one box, under the same rule as every bar', () => {
    const block = /^((?:\.[\w-]+,\s*\n)*\.[\w-]+)\s*\{([^}]*)\}/m;
    const sticky = [...appCss.matchAll(new RegExp(block, 'gm'))].find(
      (match) => /position:\s*sticky;/.test(match[2]) && /\.xp-summary\b/.test(match[1]),
    );
    expect(sticky).toBeDefined();
    expect(sticky![1].split(/,\s*/).map((s) => s.trim())).toContain('.bar-stack');
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

  // E4: the ruleset diagnostics disclosure (`ErrorDetails.svelte`). One of its two
  // mounts is the HEADER, which `.app-header` keeps to a single row of chrome — an
  // integrity failure can carry a hundred messages, and an unbounded list expanding
  // there would push the whole app down the window. The open panel is therefore a
  // scrollport of its own, so opening it costs a bounded amount of height wherever
  // it is mounted.
  it('bounds the technical-detail panel so opening it cannot swallow the window', () => {
    const block = /^\.error-details ul\s*\{([^}]*)\}/m.exec(cssWithoutComments);
    expect(block, 'app.css should declare a .error-details ul rule').not.toBeNull();
    expect(block![1]).toMatch(/max-height:\s*[\d.]+rem;/);
    expect(block![1]).toMatch(/overflow-y:\s*auto;/);
    // And a measure: an integrity message is a full sentence naming ids and a
    // rulebook file, which spans the whole window without one.
    expect(block![1]).toMatch(/max-width:\s*[\d.]+rem;/);
  });

  // The summary is a POINTER TARGET, and a native `<summary>` does not get a
  // pointer cursor on its own.
  it('marks the disclosure summary as the control it is', () => {
    const block = /^\.error-details summary\s*\{([^}]*)\}/m.exec(cssWithoutComments);
    expect(block, 'app.css should declare a .error-details summary rule').not.toBeNull();
    expect(block![1]).toMatch(/cursor:\s*pointer;/);
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
    // The band is the DARK palette's, unchanged: 0.55 is low enough to read as
    // "off" at a glance and high enough that `--ink` over `--panel` still clears
    // 4.5:1 (5.09:1 at 0.55; 3.90:1 at 0.45). It is read through `blockedDim`
    // rather than off the rule because the number now lives in `--dim-blocked` —
    // the light palette needs a different one, and "keeps a dimmed blocked row
    // above AA in BOTH palettes" above derives each from the tokens instead of
    // restating a hand-computed ratio that only ever held for one of them.
    const dim = blockedDim(DARK);
    expect(dim).toBeGreaterThanOrEqual(0.5);
    expect(dim).toBeLessThanOrEqual(0.6);
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

  // ── The header is chrome, and chrome is charged to the content (C5) ─────────
  //
  // The header cost 113px of a window whose `minHeight` is 900 (tauri.conf.json),
  // and it did so unconditionally: `.brand` declared `flex: 1 1 100%`, so the
  // controls were pushed onto a SECOND row at every width — a wrap that no
  // available width could ever satisfy. Above and below that sat 1rem of padding
  // each, around a 2.25rem logo.
  //
  // The rendered height is measured in the real engine (`e2e/specs/app-shell.e2e.js`,
  // "header real estate"), because a stylesheet can only pin the numbers that were
  // chosen, never the box they produce. These are the fast guards on those numbers.

  it('keeps the whole header on a single row', () => {
    // Asserted as an ABSENCE, because the bug was a rule and not a missing one:
    // `.brand` existed only to hold `flex: 1 1 100%`, and nothing in the header
    // may claim a full flex line again — a 100% basis in a wrap container is a
    // row break that no width can undo.
    expect(cssWithoutComments).not.toMatch(/^\.brand\s*\{/m);
    expect(ruleBody('app-header')).not.toMatch(/flex-basis:\s*100%/);
    expect(ruleBody('controls')).not.toMatch(/flex(-basis)?:[^;]*100%/);
  });

  it('budgets the header a chrome strip of padding, not a content band', () => {
    const [vertical] = paddingPx(ruleBody('app-header'));
    // 1rem above AND below was a quarter of the old height on its own. Half a
    // root step is the band a separator rule needs to read as a strip.
    expect(vertical).toBeLessThanOrEqual(0.5 * ROOT_FONT_PX);
    // A floor too: at zero the row's controls would touch the window edge and the
    // bottom rule.
    expect(vertical).toBeGreaterThan(0);
  });

  // U4 (P5, `docs/open-todos.md`, 2026-09-13): the logo left the header for
  // `.char-banner`, right-bound beside the character-type/name column, "sized
  // to the combined height of the character-type and character-name lines" —
  // a different ceiling than the header's own control row, which no longer
  // constrains it at all now that it is not there. Modelled the same way the
  // retired header test modelled the control row: each line's own font-size at
  // an ordinary ~1.2 line-height, with padding/border left out of the model
  // exactly as that one left out everything but the line box it was budgeting.
  it('sizes the logo to the type-plus-name lines it now sits beside, not the header row (P5)', () => {
    const logoPx = lengthPx(/height:\s*([^;]+);/.exec(ruleBody('app-logo'))![1]);

    const typeLinePx = scaleStepRem('font-small') * ROOT_FONT_PX * 1.2;
    const nameLinePx = 1.4 * ROOT_FONT_PX * 1.2; // `.char-banner .name-input`'s own font-size
    const twoLineCeilingPx = typeLinePx + nameLinePx;

    expect(logoPx).toBeGreaterThan(0);
    // Tall enough to read as spanning BOTH lines, not a sliver dwarfed by them.
    expect(logoPx).toBeGreaterThanOrEqual(nameLinePx);
    expect(logoPx).toBeLessThanOrEqual(twoLineCeilingPx);
  });

  // P5's placement: right-bound beside the type/name column, which the
  // structural tests in `CharacterBanner.test.ts` already pin (source order +
  // an intervening `.char-banner-main` wrapper). This is the layout rule that
  // makes "right-bound" true: the row is a flex row, and the main column's
  // `flex: 1` is what pushes the logo to its end — not an explicit
  // `margin-left: auto` on the logo, which would be a second, driftable answer
  // to the same question.
  it('lays the banner out as a row so the logo can sit right-bound beside the column (P5)', () => {
    expect(ruleBody('char-banner')).not.toMatch(/flex-direction:\s*column/);
    expect(ruleBody('char-banner-main')).toMatch(/flex-direction:\s*column/);
    expect(ruleBody('char-banner-main')).toMatch(/flex:\s*1/);
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
  // manual-testing-findings-2026-09 #1: at the then-default 1100x800 window the
  // strip ran out of room and the tab titles ellipsized — which is why every tab
  // also carries its full label in `title` (App.svelte). GERMAN is the binding case,
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
  // The e2e counterpart (`e2e/specs/wizard-flow.e2e.js`'s `tab area at a short
  // window height`) measures the real thing in
  // a real engine; this test is the fast guard that catches a longer German label
  // or a loosened rule long before the binary is built.
  const ROOT_FONT_PX = 12.75; // `:root { font-size: 12.75px }` — see the type scale above
  const EM_PER_CHARACTER = 0.53;
  // The budget stays at 1100 even though #33 widened the default window to 1400
  // (crates/arm-app/tauri.conf.json), and that is deliberate: the strip is a
  // separate concern from the aging surface, and letting an unrelated window change
  // loosen its budget by 300px would silently retire the guard #1 bought. 1100 is
  // where the strip was measured to fit and is a realistic narrow working width —
  // the window is resizable, and its `minWidth` is 900. It is NOT pinned to 900
  // because the strip does not fit there and never has: the German thirteen-tab set
  // models at ~1020px of labels, padding and gaps against an 881px strip, so at 900
  // the labels ellipsize and fall back on their `title`. Tightening this to 900
  // would be a real finding about the tab strip, not a side effect of this change.
  const NARROW_WINDOW_PX = 1100;

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

  it('fits the widest (German) tab set inside a 1100px-wide window', () => {
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
    const strip = NARROW_WINDOW_PX - 2 * barHorizontal;

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

  // ── The derived read-out tables need the figures the rest of the app has ────
  //
  // Sabine 10 (full-audit round 1): `.derived-table th, .derived-table td` already
  // right-aligns every cell, which is only worth doing if the digits are the same
  // width — proportional figures put the units column of "7" and "11" in different
  // places even flush right, and the Lab/Casting grid is the densest numeric
  // surface the app has. Eleven other rules in this stylesheet already declare
  // `tabular-nums` for exactly this; the tables were the omission.
  //
  // The rule is a GROUPED selector, so `selectorBody`'s single-selector pattern
  // does not reach it — matched here by its own two-line head instead.
  it('sets tabular figures on the right-aligned derived-table cells', () => {
    const block = /^\.derived-table th,\s*\n\.derived-table td\s*\{([^}]*)\}/m.exec(
      cssWithoutComments,
    );
    expect(block, 'app.css should declare a grouped .derived-table th/td rule').not.toBeNull();
    const body = block![1];
    // The alignment is the reason the figures have to be tabular, so both halves
    // are pinned together: dropping either one alone re-opens the defect.
    expect(body).toMatch(/text-align:\s*right;/);
    expect(body).toMatch(/font-variant-numeric:\s*tabular-nums;/);
  });

  // ── The Ability parameter combo is one line (L4, try-out finding 5) ─────────
  //
  // A catalogued parameter (Dead Language, Area Lore, …) renders a dropdown —
  // catalogue values, linked Virtues, "Other…" — and, while "Other…" is chosen
  // (the default), a free-text field. The field carried `.ability-param`'s
  // `flex: 1 0 100%`, written for the bare text field of an UNcatalogued
  // parameter, so it broke onto a line of its own under the dropdown and the
  // pair read as two unrelated controls. Wrapped together (the markup half is
  // pinned in `lib/components/AbilityTab.test.ts`), the pair takes that full
  // line instead and splits it: dropdown at its content width, field the rest.
  it('lays the parameter dropdown and its "Other…" field out on one line of their own', () => {
    const combo = selectorBody('.ability-selection .ability-param-combo');
    // The pair, not the field, now claims the row's parameter line.
    expect(combo).toMatch(/flex:\s*1\s+0\s+100%;/);
    expect(combo).toMatch(/display:\s*flex;/);
    expect(combo).toMatch(/align-items:\s*center;/);
    expect(combo).toMatch(/gap:/);
    expect(combo).toMatch(/min-width:\s*0;/);
    // ONE line: a wrapping combo would put the field back under the dropdown.
    expect(combo).not.toMatch(/flex-wrap:\s*wrap/);

    // Dropdown left, sized to its content: neither grows nor shrinks.
    const select = selectorBody('.ability-selection .ability-param-combo .ability-param-select');
    expect(select).toMatch(/flex:\s*0\s+0\s+auto;/);
    // Field right, taking whatever is left — and allowed to go narrower than its
    // intrinsic width, or a long placeholder would push the line wider than the row.
    // Three classes deep so it outranks the two-class full-width rule below
    // regardless of source order.
    const field = selectorBody('.ability-selection .ability-param-combo .ability-param');
    expect(field).toMatch(/flex:\s*1\s+1\s+auto;/);
    expect(field).toMatch(/min-width:\s*0;/);
  });

  // The other user of `.ability-param`: a parameter with no catalogue renders the
  // bare text field, no dropdown, and that one keeps its own full-width line.
  it('keeps the bare parameter field of an uncatalogued Ability on a full line', () => {
    expect(selectorBody('.ability-selection .ability-param')).toMatch(/flex:\s*1\s+0\s+100%;/);
  });

  // Try-out finding 18 (U2): a labelled parameter group — Incompatible Arts'
  // "Combination N" — stacked its Technique and Form selects one under the other,
  // because each `.param` label is a column-flex block. The members now share ONE
  // row beneath the group label (the markup half is pinned in
  // `lib/components/ParameterPicker.test.ts`), so each pair reads as a pair.
  it('lays a labelled parameter group out as one row of its member controls', () => {
    const row = ruleBody('param-group-row');
    expect(row).toMatch(/display:\s*flex;/);
    // A row, not the column every `.param` label is.
    expect(row).not.toMatch(/flex-direction:\s*column/);
    // ONE line: a wrapping row would drop the Form back under its Technique.
    expect(row).not.toMatch(/flex-wrap:\s*wrap/);
    // The selects line up on their tops, not stretched to the tallest member.
    expect(row).toMatch(/align-items:\s*flex-start;/);
  });

  // ── The issues footer is a FIXED band (U3, after-deadline answer 8) ─────────
  //
  // `.validation-bar` was `max-height: 30vh`, i.e. as tall as its issue list up to
  // a cap — so an add whose validation brought a finding grew it by a row and
  // squeezed the tab area from below, and a quick second click near the bottom of
  // the tab landed on the footer that had just moved under the pointer
  // (`waitForBalancePoints` in `e2e/helpers.js` exists only to dodge that). The
  // band is now one height whatever it holds — the empty state included, since
  // the footer is always rendered (App.svelte, WizardShell.svelte) — so the tab
  // area above it never moves. The height is a model of the rows it holds, at the
  // ~1.2 line box this file uses for every other line-height estimate.
  const LINE_BOX = 1.2;

  /** The height a rule declares (not `min-`/`max-height`), in CSS px. */
  function ownHeightPx(body: string): number {
    const declared = /(?:^|[^-])height:\s*([^;]+);/.exec(body);
    expect(declared, 'the rule should declare its own height').not.toBeNull();
    return lengthPx(declared![1]);
  }

  /** One `.issue` row: padding, its single line box, and the gap below it. */
  function issueRowPx(): number {
    const row = ruleBody('issue');
    const [vertical] = paddingPx(row);
    const margin = /margin-bottom:\s*([^;]+);/.exec(row);
    expect(margin, '.issue should space its rows with margin-bottom').not.toBeNull();
    return 2 * vertical + fontSizePx(row) * LINE_BOX + lengthPx(margin![1]);
  }

  it('gives the issues footer a fixed height rather than one that follows its rows', () => {
    const bar = ruleBody('validation-bar');
    // A height in rem, so it tracks the type the rows are set in.
    expect(bar).toMatch(/(?:^|[^-])height:\s*[\d.]+rem;/);
    // Neither bound: a min or max height is exactly a height that moves.
    expect(bar).not.toMatch(/(max|min)-height:/);
    // …and not a share of the window, which is a height that moves on resize.
    expect(bar).not.toMatch(/vh/);
    // On a flex item a height is only the basis: without this the column could
    // still shrink the band when the window runs short.
    expect(bar).toMatch(/flex-shrink:\s*0;|flex:\s*(none|0\s+0\s+auto);/);
  });

  it('sizes the issues footer for about four issue rows under its heading', () => {
    const bar = ruleBody('validation-bar');
    const [vertical] = paddingPx(bar);
    const border = lengthPx(/border-top:\s*([\d.]+px)/.exec(bar)![1]);
    const heading = selectorBody('.validation-docked h2');
    const headingGap = /margin:\s*0\s+0\s+([^;\s]+);/.exec(heading);
    expect(headingGap, 'the heading should space itself from the list').not.toBeNull();
    const headingPx = fontSizePx(heading) * LINE_BOX + lengthPx(headingGap![1]);

    // `* { box-sizing: border-box }`, so the declared height includes padding and
    // border, and what is left under the heading is the scrolling issue area.
    const issueArea = ownHeightPx(bar) - 2 * vertical - border - headingPx;
    expect(issueArea, 'four whole rows fit').toBeGreaterThanOrEqual(4 * issueRowPx());
    expect(issueArea, 'not a fifth').toBeLessThan(5 * issueRowPx());
  });

  it('scrolls the issues inside the fixed footer, under a heading that stays put', () => {
    // A flex column, so the panel inside can be handed exactly the band's height…
    const bar = ruleBody('validation-bar');
    expect(bar).toMatch(/display:\s*flex;/);
    expect(bar).toMatch(/flex-direction:\s*column;/);
    // …the docked panel fills it and passes it on to its body…
    const panel = ruleBody('validation-docked');
    expect(panel).toMatch(/display:\s*flex;/);
    expect(panel).toMatch(/flex-direction:\s*column;/);
    expect(panel).toMatch(/flex:\s*1/);
    expect(panel).toMatch(/min-height:\s*0;/);
    // …and the body, not the band and not the heading, is the one scrollport.
    const body = selectorBody('.validation-docked .validation-body');
    expect(body).toMatch(/flex:\s*1/);
    expect(body).toMatch(/min-height:\s*0;/);
    expect(body).toMatch(/overflow-y:\s*auto;/);
    // The list's own 7.5rem cap was a SECOND scrollport nested in the first, and a
    // height of its own that the band would have to be kept in step with.
    expect(cssWithoutComments).not.toMatch(
      /\.validation-docked \.issue-list\s*\{[^}]*(max-height|overflow)/,
    );
  });

  // Try-out finding 2: a long description scrolls inside the tooltip, but the
  // popup ignored the pointer, so its scrollbar could never be reached. The
  // hover bridge (`actions.ts::tooltip`) lets the pointer travel into it; the
  // popup must therefore accept the pointer, and its text must keep scrolling.
  it('lets the pointer into the tooltip popup, whose text scrolls', () => {
    const pop = selectorBody('.tooltip-pop');
    expect(pop, 'the popup must not let the pointer fall through it').not.toMatch(
      /pointer-events:\s*none/,
    );
    expect(pop).toMatch(/pointer-events:\s*auto;/);

    const text = selectorBody('.tooltip-pop .tooltip-text');
    expect(text).toMatch(/max-height:\s*[^;]+;/);
    expect(text).toMatch(/overflow-y:\s*auto;/);
  });

  // A wheel scroll that reaches the end of the tooltip text would otherwise chain
  // to the page — and a page scroll closes the popup under the reader's pointer.
  it('keeps a wheel scroll inside the tooltip text from scrolling the page', () => {
    const text = selectorBody('.tooltip-pop .tooltip-text');
    expect(text).toMatch(/overscroll-behavior:\s*contain;/);
  });
});
